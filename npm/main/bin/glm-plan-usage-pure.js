#!/usr/bin/env node
"use strict";

const https = require("https");
const http = require("http");

// Config
const API_TIMEOUT = 5000;
const CACHE_TTL_MS = 120_000;

let cache = null;

function getEnv(name) {
  return process.env[name] || "";
}

function request(url, token) {
  return new Promise((resolve, reject) => {
    const mod = url.startsWith("https") ? https : http;
    const req = mod.get(url, {
      timeout: API_TIMEOUT,
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    }, (res) => {
      if (res.statusCode !== 200) {
        res.resume();
        return reject(new Error(`HTTP ${res.statusCode}`));
      }
      let data = "";
      res.on("data", (c) => (data += c));
      res.on("end", () => {
        try { resolve(JSON.parse(data)); }
        catch { reject(new Error("JSON parse error")); }
      });
    });
    req.on("error", reject);
    req.on("timeout", () => { req.destroy(); reject(new Error("timeout")); });
  });
}

function buildClient() {
  // Support both GLM_* and ANTHROPIC_* env vars (GLM_* takes priority)
  const token = getEnv("GLM_AUTH_TOKEN") || getEnv("ANTHROPIC_AUTH_TOKEN");
  const baseUrl = getEnv("GLM_BASE_URL") || getEnv("ANTHROPIC_BASE_URL") || "https://open.bigmodel.cn/api/anthropic";
  const apiUrl = baseUrl.replace(/\/api\/anthropic/, "/api").replace(/\/anthropic$/, "");

  // Detect platform and timezone offset (in hours)
  // Zhipu server expects Beijing time (UTC+8), ZAI server expects UTC (UTC+0)
  const isZhipu = baseUrl.includes("bigmodel.cn") || baseUrl.includes("zhipu");
  const tzOffsetMs = isZhipu ? 8 * 3600_000 : 0;

  return {
    token,
    apiUrl,
    tzOffsetMs,
    async fetchQuota() {
      return request(`${this.apiUrl}/monitor/usage/quota/limit`, this.token);
    },
    async fetchModelUsage(startTime, endTime) {
      const s = encodeURIComponent(startTime);
      const e = encodeURIComponent(endTime);
      return request(`${this.apiUrl}/monitor/usage/model-usage?startTime=${s}&endTime=${e}`, this.token);
    },
  };
}

function fmtReset(ms) {
  if (!ms) return "--:--";
  const d = new Date(ms);
  return `${d.getHours()}:${String(d.getMinutes()).padStart(2, "0")}`;
}

function fmtTokens(n) {
  if (n < 0) return "N/A";
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 10_000) return `${(n / 1_000).toFixed(1)}K`;
  return `${n}`;
}

async function fetchStats(client) {
  if (cache && Date.now() - cache.ts < CACHE_TTL_MS) return cache.data;

  // Retry logic (3 attempts)
  let quota = null;
  for (let attempt = 0; attempt < 3; attempt++) {
    quota = await client.fetchQuota().catch(() => null);
    if (quota && quota.success) break;
    if (attempt < 2) await new Promise(r => setTimeout(r, 100));
  }
  if (!quota || !quota.success) return null;

  const level = (quota.data?.level || "pro").toLowerCase();

  // Token usage (5h) - first TOKENS_LIMIT with unit=3
  const tokenLimit = quota.data?.limits?.find(l => l.type === "TOKENS_LIMIT" && l.unit === 3);
  // Weekly usage - TOKENS_LIMIT with unit=6
  const weeklyLimit = quota.data?.limits?.find(l => l.type === "TOKENS_LIMIT" && l.unit === 6);
  // MCP usage - TIME_LIMIT
  const mcpLimit = quota.data?.limits?.find(l => l.type === "TIME_LIMIT");

  // Get reset time for time window sync (sync with quota window)
  const resetTimeMs = tokenLimit?.nextResetTime;

  // Fetch model usage only when we have a proper billing window boundary
  // Without nextResetTime, a rolling window would include pre-reset stale data
  let callCount = null, tokensUsed = null;
  if (resetTimeMs) {
    try {
      const fmt = (d) => {
        const t = new Date(d.getTime() + client.tzOffsetMs);
        return `${t.getUTCFullYear()}-${String(t.getUTCMonth()+1).padStart(2,"0")}-${String(t.getUTCDate()).padStart(2,"0")} ${String(t.getUTCHours()).padStart(2,"0")}:${String(t.getUTCMinutes()).padStart(2,"0")}:${String(t.getUTCSeconds()).padStart(2,"0")}`;
      };

      const end = new Date(resetTimeMs);
      const start = new Date(end.getTime() - 5 * 3600_000);

      const modelUsage = await client.fetchModelUsage(fmt(start), fmt(end));
      if (modelUsage?.data?.totalUsage) {
        callCount = modelUsage.data.totalUsage.totalModelCallCount ?? null;
        tokensUsed = modelUsage.data.totalUsage.totalTokensUsage ?? null;
      }
    } catch { /* ignore */ }
  }

  const result = { level, tokenLimit, weeklyLimit, mcpLimit, callCount, tokensUsed };
  cache = { data: result, ts: Date.now() };
  return result;
}

function color256(code) {
  return `\x1b[38;5;${code}m`;
}

function reset() {
  return "\x1b[0m";
}

function format(stats) {
  // When no stats available, show zero usage
  if (!stats) {
    return `${color256(109)}\x1b[1m🪙 0% · 📊 0 · ⚡ 0${reset()}`;
  }

  const parts = [];

  if (stats.tokenLimit) {
    parts.push(`🪙 ${stats.tokenLimit.percentage}% (⏰ ${fmtReset(stats.tokenLimit.nextResetTime)})`);
  }

  if (stats.callCount != null) {
    parts.push(`📊 ${stats.callCount}`);
  }

  if (stats.weeklyLimit) {
    parts.push(`📅 ${stats.weeklyLimit.percentage}%`);
  }

  if (stats.mcpLimit) {
    parts.push(`🌐 ${stats.mcpLimit.currentValue}/${stats.mcpLimit.usage}`);
  }

  if (stats.tokensUsed != null) {
    parts.push(`⚡ ${fmtTokens(stats.tokensUsed)}`);
  }

  if (parts.length === 0) return "";

  return `${color256(109)}\x1b[1m${parts.join(" · ")}${reset()}`;
}

// Main
async function main() {
  const debug = process.env.GLM_DEBUG === "1";
  const logFile = require("fs").createWriteStream(require("path").join(require("os").homedir(), ".claude", "glm-plan-usage", "debug.log"), { flags: "a" });
  const log = (msg) => {
    const ts = new Date().toISOString();
    const line = `[${ts}] ${msg}\n`;
    if (debug) process.stderr.write(`[glm] ${msg}\n`);
    logFile.write(line);
  };

  // Read stdin
  let inputText = "";
  try {
    inputText = await new Promise((resolve, reject) => {
      const chunks = [];
      process.stdin.resume();
      process.stdin.on("data", (c) => chunks.push(c));
      process.stdin.on("end", () => resolve(Buffer.concat(chunks).toString()));
      process.stdin.on("error", reject);
      setTimeout(() => resolve(""), 1000);
    });
  } catch (e) { log(`stdin error: ${e.message}`); return; }

  log(`stdin: ${inputText.substring(0, 200)}`);

  let input;
  try {
    input = JSON.parse(inputText);
  } catch { input = {}; }

  log(`model: ${input.model?.id}`);

  // Only show for GLM models
  if (input.model?.id) {
    const id = input.model.id.toLowerCase();
    if (!id.includes("glm") && !id.includes("chatglm")) {
      log("not glm model, skipping");
      return;
    }
  }

  const client = buildClient();
  log(`token: ${client.token ? "present" : "MISSING"}`);

  let stats = null;
  if (client.token) {
    stats = await fetchStats(client);
    log(`stats: ${stats ? "ok" : "null"}`);
  }

  const output = format(stats);
  log(`output: ${output ? output.length + " chars" : "empty"}`);
  if (output) process.stdout.write(output);
}

main().catch(() => {});
