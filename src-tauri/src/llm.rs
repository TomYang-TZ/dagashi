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

pub fn select_character(
    stats: &DailyStats,
    rarity: &Rarity,
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
        r#"You are the Dagashi oracle for the anime Gintama.

Based on these keystroke stats from today, pick a Gintama character and scene.

RARITY TIER ROLLED: {rarity}

KEYSTROKE STATS:
{stats_json}

TIER GUIDANCE:
- Common: Background Edo citizens, Justaway, Elizabeth idle
- Uncommon: Shinpachi, Otae, Hasegawa (MADAO)
- Rare: Kagura, Sadaharu, Kondo, Otose
- Epic: Gintoki, Hijikata, Okita, Takasugi
- Legendary: Iconic scenes — Gintoki vs Takasugi, Shiroyasha mode, Kagura Yato form

RECENT PULLS (avoid repeating): {recent}

Interpret the typing personality from the stats. Pick a character matching the tier.
The search_query should work well for finding a GIF on Giphy.
The flavor_text should be a fun 1-2 sentence "reading" of their typing personality."#,
        rarity = rarity.label(),
    );

    let mut child = Command::new("claude")
        .args([
            "-p",
            "--model", &config.cli_model,
            "--effort", &config.cli_effort,
            "--output-format", "json",
            "--json-schema", JSON_SCHEMA,
            "--no-session-persistence",
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
        return Err(format!("claude exited with error: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // claude -p --output-format json may wrap in {"type":"result","result":"..."}
    // or return the JSON directly with --json-schema. Handle both.
    let result_text = if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(result) = parsed.get("result").and_then(|v| v.as_str()) {
            result.to_string()
        } else {
            stdout.to_string()
        }
    } else {
        stdout.to_string()
    };

    serde_json::from_str(&result_text)
        .map_err(|e| format!("failed to parse character selection: {e}\nraw: {stdout}"))
}
