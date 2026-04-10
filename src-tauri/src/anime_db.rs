use serde::{Deserialize, Serialize};
use std::fs;

use crate::config;
use crate::gacha::Rarity;

const JIKAN_TOP_URL: &str = "https://api.jikan.moe/v4/top/anime";
const PAGES_TO_FETCH: u32 = 40; // 25 per page × 40 = 1000 anime
const CACHE_FILE: &str = "anime_db.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeEntry {
    pub mal_id: u64,
    pub title: String,
    pub members: u64,
    pub score: Option<f64>,
    pub popularity_rank: u32, // 1 = most popular
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeDb {
    pub anime: Vec<AnimeEntry>,
    pub fetched_at: String,
}

// Rarity tiers based on popularity rank:
//   Legendary: top 25 (rank 1-25) — the mega mainstream icons
//   Epic: rank 26-100
//   Rare: rank 101-300
//   Uncommon: rank 301-600
//   Common: rank 601-1000

pub fn rank_to_rarity(rank: u32) -> Rarity {
    match rank {
        1..=25 => Rarity::Legendary,
        26..=100 => Rarity::Epic,
        101..=300 => Rarity::Rare,
        301..=600 => Rarity::Uncommon,
        _ => Rarity::Common,
    }
}

/// Pick a random anime matching the given rarity tier.
pub fn pick_anime<'a>(db: &'a AnimeDb, rarity: &Rarity, seed: u64) -> Option<&'a AnimeEntry> {
    let matching: Vec<&AnimeEntry> = db
        .anime
        .iter()
        .filter(|a| rank_to_rarity(a.popularity_rank) == *rarity)
        .collect();

    if matching.is_empty() {
        return db.anime.last(); // fallback to least popular
    }

    let idx = (seed as usize) % matching.len();
    Some(matching[idx])
}

/// Load the anime database from cache, or fetch from Jikan API if stale/missing.
pub fn load_or_fetch() -> Result<AnimeDb, String> {
    let cache_path = config::data_dir().join(CACHE_FILE);

    // Try loading from cache
    if cache_path.exists() {
        if let Ok(data) = fs::read_to_string(&cache_path) {
            if let Ok(db) = serde_json::from_str::<AnimeDb>(&data) {
                // Use cache if less than 7 days old
                if let Ok(fetched) = chrono::NaiveDate::parse_from_str(&db.fetched_at, "%Y-%m-%d") {
                    let today = chrono::Local::now().date_naive();
                    if (today - fetched).num_days() < 14 {
                        eprintln!("[dagashi] Loaded {} anime from cache", db.anime.len());
                        return Ok(db);
                    }
                }
            }
        }
    }

    // Fetch from Jikan API
    eprintln!("[dagashi] Fetching anime database from Jikan API...");
    let db = fetch_from_jikan()?;
    eprintln!("[dagashi] Fetched {} anime", db.anime.len());

    // Save cache
    fs::create_dir_all(config::data_dir()).ok();
    if let Ok(json) = serde_json::to_string_pretty(&db) {
        fs::write(&cache_path, json).ok();
    }

    Ok(db)
}

fn fetch_from_jikan() -> Result<AnimeDb, String> {
    let mut all_anime = Vec::new();
    let client = reqwest::blocking::Client::new();

    for page in 1..=PAGES_TO_FETCH {
        let url = format!(
            "{JIKAN_TOP_URL}?type=tv&filter=bypopularity&limit=25&page={page}"
        );

        let resp = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .map_err(|e| format!("jikan request failed (page {page}): {e}"))?;

        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        let data = body
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or("no data array in jikan response")?;

        for entry in data {
            let title = entry
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();

            // Skip sequels/specials — deduplicate by base title
            // (simple heuristic: skip if title contains "Season", "Part", "2nd", "3rd", etc.)
            // We keep all for now and let the LLM pick interesting characters

            all_anime.push(AnimeEntry {
                mal_id: entry.get("mal_id").and_then(|v| v.as_u64()).unwrap_or(0),
                title,
                members: entry.get("members").and_then(|v| v.as_u64()).unwrap_or(0),
                score: entry.get("score").and_then(|v| v.as_f64()),
                popularity_rank: all_anime.len() as u32 + 1,
            });
        }

        let has_next = body
            .get("pagination")
            .and_then(|p| p.get("has_next_page"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !has_next {
            break;
        }

        // Jikan rate limit: 3 requests/second
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    Ok(AnimeDb {
        anime: all_anime,
        fetched_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_to_rarity_mapping() {
        assert_eq!(rank_to_rarity(1), Rarity::Legendary);
        assert_eq!(rank_to_rarity(25), Rarity::Legendary);
        assert_eq!(rank_to_rarity(26), Rarity::Epic);
        assert_eq!(rank_to_rarity(100), Rarity::Epic);
        assert_eq!(rank_to_rarity(101), Rarity::Rare);
        assert_eq!(rank_to_rarity(300), Rarity::Rare);
        assert_eq!(rank_to_rarity(301), Rarity::Uncommon);
        assert_eq!(rank_to_rarity(601), Rarity::Common);
        assert_eq!(rank_to_rarity(999), Rarity::Common);
    }
}
