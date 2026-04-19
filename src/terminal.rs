use std::env;

/// Terminal character mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharMode {
    /// Use emoji characters
    Emoji,
    /// Use ASCII fallback characters
    Ascii,
}

/// Terminal detector for character mode selection
pub struct TerminalDetector;

impl TerminalDetector {
    /// Detect the best character mode for the current terminal
    pub fn detect() -> CharMode {
        // Check environment variables first (user override)
        if env::var("USAGE_FORCE_EMOJI").is_ok() {
            return CharMode::Emoji;
        }
        if env::var("USAGE_FORCE_ASCII").is_ok() {
            return CharMode::Ascii;
        }

        // Detect Windows version
        if cfg!(windows) {
            // Windows 11 (Build >= 22000) supports emoji properly
            // Windows 10 (Build < 22000) should use ASCII to avoid encoding issues
            if Self::is_windows_11() {
                return CharMode::Emoji;
            }
            // Windows 10: default to ASCII mode to avoid encoding issues
            // Users can override with USAGE_FORCE_EMOJI=1 if they know their terminal supports it
            return CharMode::Ascii;
        }

        // On Linux/macOS, default to emoji mode
        CharMode::Emoji
    }

    /// Check if running on Windows 11 (Build >= 22000)
    fn is_windows_11() -> bool {
        use std::process::Command;

        // Use PowerShell to get Windows version
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", "[System.Environment]::OSVersion.Version.Build"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                if let Ok(build_str) = String::from_utf8(o.stdout) {
                    if let Ok(build) = build_str.trim().parse::<u32>() {
                        return build >= 22000; // Windows 11 starts from build 22000
                    }
                }
            }
            _ => {}
        }

        // If detection fails, assume Windows 10 (safe default)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_default() {
        let mode = TerminalDetector::detect();
        // Should be either Emoji or Ascii depending on platform
        assert!(matches!(mode, CharMode::Emoji | CharMode::Ascii));
    }
}
