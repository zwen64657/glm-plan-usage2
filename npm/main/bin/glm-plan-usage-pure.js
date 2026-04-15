#!/usr/bin/env node
"use strict";

const https = require("https");
const http = require("http");
const os = require("os");

// Config
const API_TIMEOUT = 5000;
const CACHE_TTL_MS = 120_000;

let cache = null;
let speedCache = null;

function getEnv(name) {
  return process.env[name] || "";
}

// Terminal character mode
const CharMode = {
  Emoji: "emoji",
  Ascii: "ascii"
};

// Detect the best character mode for the current terminal
function detectCharMode() {
  // Check environment variables first (user override)
  if (getEnv("GLM_FORCE_EMOJI")) {
    return CharMode.Emoji;
  }
  if (getEnv("GLM_FORCE_ASCII")) {
    return CharMode.Ascii;
  }

  // Detect Windows version
  if (os.platform() === "win32") {
    // Windows 11 (Build >= 22000) supports emoji properly
    // Windows 10 (Build < 22000) should use ASCII to avoid encoding issues
    if (isWindows11()) {
      return CharMode.Emoji;
    }
    // Windows 10: default to ASCII mode to avoid encoding issues
    // Users can override with GLM_FORCE_EMOJI=1 if they know their terminal supports it
    return CharMode.Ascii;
  }

  // On Linux/macOS, default to emoji mode
  return CharMode.Emoji;
}

// Check if running on Windows 11 (Build >= 22000)
function isWindows11() {
  try {
    const { execSync } = require("child_process");
    const buildStr = execSync("powershell -NoProfile -Command \"[System.Environment]::OSVersion.Version.Build\"", { encoding: "utf8" }).trim();
    const build = parseInt(buildStr, 10);
    return !isNaN(build) && build >= 22000; // Windows 11 starts from build 22000
  } catch (e) {
    // If detection fails, assume Windows 10 (safe default)
    return false;
  }
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
  const token = getEnv("ANTHROPIC_AUTH_TOKEN");
  const baseUrl = getEnv("ANTHROPIC_BASE_URL") || "https://open.bigmodel.cn/api/anthropic";
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

function fmtIsoReset(isoStr) {
  if (!isoStr) return "--:--";
  try {
    const d = new Date(isoStr);
    if (isNaN(d.getTime())) return "--:--";
    return `${d.getHours()}:${String(d.getMinutes()).padStart(2, "0")}`;
  } catch { return "--:--"; }
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

function format(stats, charMode, outputTps, inputTps) {
  // Character mapping based on mode
  const icons = charMode === CharMode.Ascii ? {
    token: "$",
    clock: "T",
    chart: "#",
    calendar: "%",
    globe: "M",
    lightning: "k",
    rocket: "v",
    inbox: "^"
  } : {
    token: "🔋",
    clock: "⏰",
    chart: "📊",
    calendar: "📅",
    globe: "🌐",
    lightning: "⚡",
    rocket: "🚀",
    inbox: "📥"
  };

  // When no stats available, show placeholder format
  if (!stats) {
    return `${color256(109)}\x1b[1mGLM ${icons.token} % · ${icons.clock} --:-- · ↓${icons.rocket} -- · ↑${icons.inbox} -- · ${icons.chart} 0 · ${icons.globe} / · ${icons.lightning}${reset()}`;
  }

  const parts = [];

  if (stats.tokenLimit) {
    parts.push(`${icons.token} ${stats.tokenLimit.percentage}% · ${icons.clock} ${fmtReset(stats.tokenLimit.nextResetTime)}`);
  }

  if (outputTps > 0) {
    parts.push(`↓${icons.rocket} ${outputTps.toFixed(1)}`);
  }

  if (inputTps > 0) {
    parts.push(`↑${icons.inbox} ${inputTps.toFixed(1)}`);
  }

  if (stats.callCount != null) {
    parts.push(`${icons.chart}·${stats.callCount}`);
  }

  if (stats.weeklyLimit) {
    parts.push(`${icons.calendar}·${stats.weeklyLimit.percentage}%`);
  }

  if (stats.mcpLimit) {
    parts.push(`${icons.globe}·${stats.mcpLimit.currentValue}/${stats.mcpLimit.usage}`);
  }

  if (stats.tokensUsed != null) {
    parts.push(`${icons.lightning}·${fmtTokens(stats.tokensUsed)}`);
  }

  if (parts.length === 0) return "";

  return `${color256(109)}\x1b[1mGLM ${parts.join(" · ")}${reset()}`;
}

// ===== MiniMax =====

function buildMiniMaxClient() {
  const token = getEnv("ANTHROPIC_AUTH_TOKEN");
  const baseUrl = getEnv("ANTHROPIC_BASE_URL") || "";
  if (!baseUrl.includes("minimaxi.com") && !baseUrl.includes("minimax.io")) return null;

  // Extract scheme + domain
  const match = baseUrl.match(/^(https?:\/\/[^/]+)/);
  const domain = match ? match[1] : baseUrl;

  // Read cookie: prefer MINIMAX_COOKIE, fall back to HERTZ_SESSION
  let cookie = getEnv("MINIMAX_COOKIE") || "";
  if (!cookie) {
    const hertz = getEnv("HERTZ_SESSION");
    if (hertz) cookie = "HERTZ-SESSION=" + hertz;
  }

  return {
    token,
    domain,
    cookie,
    async fetchRemains() {
      const url = `${domain}/v1/api/openplatform/coding_plan/remains`;
      const headers = {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      };
      if (cookie) headers["Cookie"] = cookie;
      return new Promise((resolve, reject) => {
        const mod = url.startsWith("https") ? https : http;
        const req = mod.get(url, { timeout: API_TIMEOUT, headers }, (res) => {
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
    },
  };
}

let minimaxCache = null;

async function fetchMiniMaxStats(client) {
  if (minimaxCache && Date.now() - minimaxCache.ts < CACHE_TTL_MS) return minimaxCache.data;

  let body = null;
  for (let attempt = 0; attempt < 3; attempt++) {
    body = await client.fetchRemains().catch(() => null);
    if (body) break;
    if (attempt < 2) await new Promise(r => setTimeout(r, 100));
  }
  if (!body) return null;

  // Find coding model: model_name starts with "MiniMax-M"
  const codingModel = (body.model_remains || []).find(m => m.model_name && m.model_name.startsWith("MiniMax-M"));
  if (!codingModel) return null;

  // API returns "remains" values, so usage_count = remaining, not used
  const intervalRemaining = codingModel.current_interval_usage_count;
  const intervalTotal = codingModel.current_interval_total_count;
  const intervalUsed = intervalTotal - intervalRemaining;
  const intervalPct = intervalTotal > 0
    ? Math.round((intervalUsed / intervalTotal) * 100)
    : 0;

  const hasWeekly = codingModel.current_weekly_total_count > 0;
  let weeklyPct = null;
  if (hasWeekly) {
    const weeklyRemaining = codingModel.current_weekly_usage_count;
    const weeklyTotal = codingModel.current_weekly_total_count;
    const weeklyUsed = weeklyTotal - weeklyRemaining;
    weeklyPct = weeklyTotal > 0 ? Math.round((weeklyUsed / weeklyTotal) * 100) : 0;
  }

  const result = {
    intervalPct,
    intervalUsed,
    intervalTotal,
    resetTime: codingModel.end_time || null,
    weeklyPct,
  };
  minimaxCache = { data: result, ts: Date.now() };
  return result;
}

function formatMiniMax(stats, charMode) {
  const icons = charMode === CharMode.Ascii
    ? { token: "$", clock: "T", chart: "#", calendar: "%" }
    : { token: "🔋", clock: "⏰", chart: "📊", calendar: "📅" };

  if (!stats) {
    return `${color256(208)}\x1b[1mMiniMax ${icons.token} % · ${icons.clock} --:-- · ${icons.chart} / · ${icons.calendar} %${reset()}`;
  }

  const parts = [];
  parts.push(`${icons.token} ${stats.intervalPct}% · ${icons.clock} ${fmtReset(stats.resetTime)}`);
  parts.push(`${icons.chart} ${stats.intervalUsed}/${stats.intervalTotal}`);
  if (stats.weeklyPct != null) {
    parts.push(`${icons.calendar}·${stats.weeklyPct}%`);
  }

  return `${color256(208)}\x1b[1mMiniMax ${parts.join(" · ")}${reset()}`;
}

// ===== Kimi =====

function buildKimiClient() {
  const token = getEnv("ANTHROPIC_API_KEY");
  const baseUrl = getEnv("ANTHROPIC_BASE_URL") || "";
  if (!baseUrl.includes("kimi.com")) return null;

  // Extract scheme + domain
  const match = baseUrl.match(/^(https?:\/\/[^/]+)/);
  const domain = match ? match[1] : baseUrl;

  return {
    token,
    domain,
    async fetchUsages() {
      return request(`${domain}/coding/v1/usages`, this.token);
    },
  };
}

let kimiCache = null;

async function fetchKimiStats(client) {
  if (kimiCache && Date.now() - kimiCache.ts < CACHE_TTL_MS) return kimiCache.data;

  let body = null;
  for (let attempt = 0; attempt < 3; attempt++) {
    body = await client.fetchUsages().catch(() => null);
    if (body && body.limits) break;
    if (attempt < 2) await new Promise(r => setTimeout(r, 100));
  }
  if (!body || !body.limits) return null;

  // Find 5-hour window: duration=300, time_unit=TIME_UNIT_MINUTE
  const fiveHour = body.limits.find(l => l.window && l.window.duration === 300 && (l.window.time_unit === "TIME_UNIT_MINUTE" || l.window.timeUnit === "TIME_UNIT_MINUTE"));
  // Find weekly window: duration=10080
  const weekly = body.limits.find(l => l.window && l.window.duration === 10080);

  if (!fiveHour || !weekly) return null;

  const fiveHourPct = fiveHour.detail.limit > 0
    ? Math.round(((fiveHour.detail.limit - fiveHour.detail.remaining) / fiveHour.detail.limit) * 100)
    : 0;

  const weeklyPct = weekly.detail.limit > 0
    ? Math.round(((weekly.detail.limit - weekly.detail.remaining) / weekly.detail.limit) * 100)
    : 0;

  // Kimi reset_time is an ISO 8601 string (e.g. "2026-03-30T18:00:00+08:00")
  const fiveHourReset = fiveHour.detail.resetTime || fiveHour.detail.reset_time || null;
  const weeklyReset = weekly.detail.resetTime || weekly.detail.reset_time || null;

  const result = {
    fiveHourPct,
    fiveHourReset,
    weeklyPct,
    weeklyReset,
  };
  kimiCache = { data: result, ts: Date.now() };
  return result;
}

function formatKimi(stats, charMode) {
  const icons = charMode === CharMode.Ascii
    ? { token: "$", clock: "T", calendar: "%" }
    : { token: "🔋", clock: "⏰", calendar: "📅" };

  if (!stats) {
    return `${color256(79)}\x1b[1mKimi ${icons.token} % · ${icons.clock} --:-- · ${icons.calendar} %${reset()}`;
  }

  const parts = [];
  parts.push(`${icons.token} ${stats.fiveHourPct}% · ${icons.clock} ${fmtIsoReset(stats.fiveHourReset)}`);
  parts.push(`${icons.calendar}·${stats.weeklyPct}%`);

  return `${color256(79)}\x1b[1mKimi ${parts.join(" · ")}${reset()}`;
}

// ===== Main =====

async function main() {
  const debug = process.env.GLM_DEBUG === "1";
  const logFile = require("fs").createWriteStream(require("path").join(require("os").homedir(), ".claude", "glm-plan-usage", "debug.log"), { flags: "a" });
  const log = (msg) => {
    const ts = new Date().toISOString();
    const line = `[${ts}] ${msg}\n`;
    if (debug) process.stderr.write(`[glm] ${msg}\n`);
    logFile.write(line);
  };

  // Detect character mode
  const charMode = detectCharMode();
  log(`char mode: ${charMode}`);

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

  // Detect platform from model.id
  const modelId = (input.model?.id || "").toLowerCase();
  const isGlm = modelId.includes("glm") || modelId.includes("chatglm");
  const isMiniMax = modelId.includes("minimax");
  const isKimi = modelId.includes("kimi");

  if (!isGlm && !isMiniMax && !isKimi) {
    log("not a supported model, skipping");
    return;
  }

  let output = "";

  if (isMiniMax) {
    const client = buildMiniMaxClient();
    if (client && client.token) {
      const stats = await fetchMiniMaxStats(client);
      output = formatMiniMax(stats, charMode);
    } else {
      output = formatMiniMax(null, charMode);
    }
  } else if (isKimi) {
    const client = buildKimiClient();
    if (client && client.token) {
      const stats = await fetchKimiStats(client);
      output = formatKimi(stats, charMode);
    } else {
      output = formatKimi(null, charMode);
    }
  } else {
    // GLM
    const client = buildClient();
    let stats = null;
    if (client.token) {
      stats = await fetchStats(client);
    }

    // Calculate TPS from context_window
    const totalInput = input.context_window?.total_input_tokens || 0;
    const totalOutput = input.context_window?.total_output_tokens || 0;
    let outputTps = 0;
    let inputTps = 0;
    const now = Date.now();

    if (speedCache && totalInput >= speedCache.totalInput && totalOutput >= speedCache.totalOutput) {
      const deltaMs = now - speedCache.ts;
      if (deltaMs > 100) {
        const deltaSec = deltaMs / 1000;
        const deltaOut = totalOutput - speedCache.totalOutput;
        const deltaIn = totalInput - speedCache.totalInput;
        const instantOut = deltaOut > 0 ? deltaOut / deltaSec : 0;
        const instantIn = deltaIn > 0 ? deltaIn / deltaSec : 0;
        const alpha = deltaMs < 30000 ? 0.5 : 1;
        outputTps = alpha * instantOut + (1 - alpha) * (speedCache.outputTps || 0);
        inputTps = alpha * instantIn + (1 - alpha) * (speedCache.inputTps || 0);
      } else {
        outputTps = speedCache.outputTps || 0;
        inputTps = speedCache.inputTps || 0;
      }
    }

    speedCache = { ts: now, totalInput, totalOutput, outputTps, inputTps };

    output = format(stats, charMode, outputTps, inputTps);
  }

  log(`output: ${output ? output.length + " chars" : "empty"}`);
  if (output) process.stdout.write(output);
}

main().catch(() => {});
