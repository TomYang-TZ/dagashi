use serde::Deserialize;

// Giphy public SDK demo key — publicly available, rate-limited to 100 req/hr.
// Published in Giphy's SDK examples, meant for development/testing.
// We use ~1 request/day. User can override in config if this key is ever revoked.
const GIPHY_DEMO_KEY: &str = "GlVGYHkr3WSBnllca54iNt0yFbjz7L65";
const GIPHY_SEARCH_URL: &str = "https://api.giphy.com/v1/gifs/search";

#[derive(Deserialize)]
struct GiphyResponse {
    data: Vec<GiphyGif>,
}

#[derive(Deserialize)]
struct GiphyGif {
    images: GiphyImages,
}

#[derive(Deserialize)]
struct GiphyImages {
    original: GiphyOriginal,
}

#[derive(Deserialize)]
struct GiphyOriginal {
    url: String,
}

/// Search Giphy for GIFs matching query. Returns list of GIF URLs.
pub fn search_gifs(query: &str, limit: u32, api_key: Option<&str>) -> Vec<String> {
    let key = api_key.unwrap_or(GIPHY_DEMO_KEY);
    let url = format!(
        "{GIPHY_SEARCH_URL}?api_key={key}&q={}&limit={limit}&rating=g",
        urlencoded(query)
    );

    let resp = match reqwest::blocking::get(&url) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let body: GiphyResponse = match resp.json() {
        Ok(b) => b,
        Err(_) => return vec![],
    };

    body.data.into_iter().map(|g| g.images.original.url).collect()
}

fn urlencoded(s: &str) -> String {
    s.replace(' ', "+")
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace('#', "%23")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_returns_results() {
        let results = search_gifs("gintoki gintama", 3, None);
        assert!(!results.is_empty());
        assert!(results.len() <= 3);
    }

    #[test]
    fn search_empty_query_returns_list() {
        let results = search_gifs("xyznonexistent12345", 3, None);
        assert!(results.len() <= 3); // may be empty, that's fine
    }
}
