use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;

use crate::config::LlmConfig;
use crate::gacha::Rarity;
use crate::stats::DailyStats;

/// Clean up a specific claude -p session transcript.
fn cleanup_session_transcript(session_id: &str) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    let projects_dir = home.join(".claude").join("projects");
    if let Ok(dirs) = std::fs::read_dir(&projects_dir) {
        for entry in dirs.flatten() {
            let transcript = entry.path().join(format!("{session_id}.jsonl"));
            if transcript.exists() {
                std::fs::remove_file(&transcript).ok();
                break;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSelection {
    pub character: String,
    pub scene: String,
    pub search_query: String,
    pub rarity: String,
    pub flavor_text: String,
}

const JSON_SCHEMA: &str = r#"{"type":"object","properties":{"character":{"type":"string"},"scene":{"type":"string"},"search_query":{"type":"string"},"rarity":{"type":"string"},"flavor_text":{"type":"string"}},"required":["character","scene","search_query","rarity","flavor_text"]}"#;

/// Collect CLAUDE_*, AWS_*, ANTHROPIC_* env vars.
/// If not in current env (e.g. app launched from Finder), load from shell profile.
fn resolve_claude_env(home: &std::path::Path) -> Vec<(String, String)> {
    let mut envs: Vec<(String, String)> = std::env::vars()
        .filter(|(k, _)| k.starts_with("CLAUDE_") || k.starts_with("AWS_") || k.starts_with("ANTHROPIC_"))
        .collect();

    // If we got nothing, try loading from shell
    if envs.is_empty() {
        if let Ok(output) = Command::new("zsh")
            .args(["-lc", "env"])
            .env("HOME", home)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    if k.starts_with("CLAUDE_") || k.starts_with("AWS_") || k.starts_with("ANTHROPIC_") {
                        envs.push((k.to_string(), v.to_string()));
                    }
                }
            }
        }
    }

    envs
}

fn find_claude_binary() -> String {
    let candidates = [
        dirs::home_dir().map(|h| h.join(".local/bin/claude")),
        dirs::home_dir().map(|h| h.join(".claude/bin/claude")),
        Some(std::path::PathBuf::from("/usr/local/bin/claude")),
        Some(std::path::PathBuf::from("/opt/homebrew/bin/claude")),
    ];
    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }
    // Fall back to bare name and hope PATH works
    "claude".to_string()
}

pub fn select_character(
    stats: &DailyStats,
    rarity: &Rarity,
    anime_title: &str,
    recent_pulls: &[String],
    config: &LlmConfig,
) -> Result<CharacterSelection, String> {
    let stats_json = serde_json::to_string(stats).map_err(|e| e.to_string())?;
    let recent = if recent_pulls.is_empty() {
        "None yet.".to_string()
    } else {
        recent_pulls.join(", ")
    };

    let prompt = format!(
        r#"You are the Dagashi oracle — an anime gacha fortune teller.

ANIME: {anime_title}
RARITY TIER: {rarity} (more popular anime = rarer pull)

Based on these keystroke stats, pick a CHARACTER from {anime_title} and a specific SCENE or POSE.

KEYSTROKE STATS:
{stats_json}

RECENT PULLS (avoid repeating): {recent}

Rules:
- Pick a well-known character from this specific anime
- The scene should be iconic or funny — something fans would recognize
- The search_query MUST start with the character's full name and the anime title, then add 1-2 descriptive words for the scene/mood — e.g. "kaguya shinomiya kaguya-sama love is war smug" or "gintoki sakata gintama lazy eating". Do NOT add generic words like "gif", "anime", "scene", "moment", or "iconic".
- The flavor_text should be a fun 1-2 sentence "reading" connecting the user's typing personality to the character
- Keep it playful and surprising"#,
        rarity = rarity.label(),
    );

    let claude_bin = find_claude_binary();
    let home = dirs::home_dir().expect("no home dir");
    let mut child = Command::new(&claude_bin)
        .current_dir("/tmp")
        .env("HOME", &home)
        .env("PATH", format!("{}/.local/bin:/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin", home.display()))
        .envs(resolve_claude_env(&home))
        .args([
            "-p",
            "--model", &config.cli_model,
            "--effort", &config.cli_effort,
            "--output-format", "json",
            "--json-schema", JSON_SCHEMA,
            "--allowedTools", "",
            "--no-session-persistence",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run claude: {e}"))?;

    // Take stdin, write prompt, then drop to send EOF cleanly
    {
        let mut stdin = child.stdin.take().ok_or("failed to open stdin")?;
        stdin.write_all(prompt.as_bytes()).map_err(|e| e.to_string())?;
    } // stdin dropped here — sends EOF to claude process

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Clean up this session's transcript
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(sid) = parsed.get("session_id").and_then(|v| v.as_str()) {
            cleanup_session_transcript(sid);
        }
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("claude exited with error (status {}): stderr={} stdout={}", output.status, stderr, stdout));
    }

    // claude -p --output-format json returns a wrapper with multiple fields:
    //   {"type":"result", "result":"...", "structured_output":{...}, ...}
    // The structured_output field contains our --json-schema validated data.
    // Try in order: structured_output > parse result string > parse raw stdout
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("failed to parse claude output as JSON: {e}\nraw: {stdout}"))?;

    // Try structured_output first (present when --json-schema is used)
    if let Some(structured) = parsed.get("structured_output") {
        if let Ok(selection) = serde_json::from_value(structured.clone()) {
            return Ok(selection);
        }
    }

    // Try result field as JSON string
    if let Some(result_str) = parsed.get("result").and_then(|v| v.as_str()) {
        if let Ok(selection) = serde_json::from_str(result_str) {
            return Ok(selection);
        }
    }

    // Try parsing the whole output directly
    serde_json::from_value(parsed.clone())
        .map_err(|e| format!("failed to extract character selection: {e}\nraw: {stdout}"))
}
