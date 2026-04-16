use serde::Deserialize;

const KLIPY_SEARCH_URL: &str = "https://api.klipy.com/api/v1";

#[derive(Deserialize)]
struct KlipyResponse {
    data: KlipyData,
}

#[derive(Deserialize)]
struct KlipyData {
    data: Vec<KlipyItem>,
}

#[derive(Deserialize)]
struct KlipyItem {
    #[serde(default)]
    title: String,
    file: KlipyFile,
}

#[derive(Deserialize)]
struct KlipyFile {
    hd: Option<KlipyFormats>,
    md: Option<KlipyFormats>,
}

#[derive(Deserialize)]
struct KlipyFormats {
    gif: Option<KlipyMedia>,
}

#[derive(Deserialize)]
struct KlipyMedia {
    url: String,
}

/// A search result with URL and title metadata for relevance filtering.
pub struct GifResult {
    pub url: String,
    pub title: String,
}

/// Search Klipy for GIFs matching query. Returns list of results with metadata.
pub fn search_gifs(query: &str, limit: usize, api_key: &str) -> Vec<GifResult> {
    let url = format!(
        "{KLIPY_SEARCH_URL}/{api_key}/gifs/search?q={}&per_page={limit}&content_filter=high",
        urlencoded(query)
    );

    let resp = match reqwest::blocking::Client::builder()
        .user_agent("Dagashi/0.1.0")
        .build()
        .ok()
        .and_then(|c| c.get(&url).send().ok())
    {
        Some(r) => r,
        None => return vec![],
    };

    if !resp.status().is_success() {
        eprintln!("[dagashi] Klipy search failed: {}", resp.status());
        return vec![];
    }

    let body: KlipyResponse = match resp.json() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[dagashi] Klipy parse error: {}", e);
            return vec![];
        }
    };

    body.data.data
        .into_iter()
        .filter_map(|item| {
            let url = item.file
                .hd
                .and_then(|f| f.gif)
                .or_else(|| item.file.md.and_then(|f| f.gif))
                .map(|m| m.url)?;
            Some(GifResult { url, title: item.title })
        })
        .take(limit)
        .collect()
}

fn urlencoded(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                String::from(b as char)
            }
            b' ' => "+".to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}
