use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;

use crate::config::LlmConfig;
use crate::gacha::Rarity;
use crate::stats::DailyStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSelection {
    pub character: String,
    pub scene: String,
    pub search_query: String,
    pub rarity: String,
    pub flavor_text: String,
}

const JSON_SCHEMA: &str = r#"{"type":"object","properties":{"character":{"type":"string"},"scene":{"type":"string"},"search_query":{"type":"string"},"rarity":{"type":"string"},"flavor_text":{"type":"string"}},"required":["character","scene","search_query","rarity","flavor_text"]}"#;

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
- The search_query should work well for finding a GIF on Giphy (include anime name + character name)
- The flavor_text should be a fun 1-2 sentence "reading" connecting the user's typing personality to the character
- Keep it playful and surprising"#,
        rarity = rarity.label(),
    );

    let claude_bin = find_claude_binary();
    let mut child = Command::new(&claude_bin)
        .args([
            "-p",
            "--model", &config.cli_model,
            "--effort", &config.cli_effort,
            "--output-format", "json",
            "--json-schema", JSON_SCHEMA,
            "--no-session-persistence",
            "--max-turns", "2",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run claude: {e}"))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(prompt.as_bytes()).map_err(|e| e.to_string())?;
    }

    let output = child.wait_with_output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("claude exited with error (status {}): stderr={} stdout={}", output.status, stderr, stdout));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

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
