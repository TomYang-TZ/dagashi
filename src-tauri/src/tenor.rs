use regex::Regex;

const TENOR_SEARCH_URL: &str = "https://tenor.com/search";

/// Search Tenor for GIFs by scraping search results page. No API key needed.
/// Returns full-size GIF URLs in Tenor's ranked order.
pub fn search_gifs(query: &str, limit: usize) -> Vec<String> {
    let slug = query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-");
    let url = format!("{TENOR_SEARCH_URL}/{slug}-gifs?format=gifs");

    let html = match reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .build()
        .ok()
        .and_then(|c| c.get(&url).send().ok())
        .and_then(|r| r.text().ok())
    {
        Some(h) => h,
        None => return vec![],
    };

    // Thumbnails appear in ranked order in the HTML with AAAAM suffix.
    // Convert to medium-size by swapping AAAAM -> AAAAd (original size).
    // Thumbnail: https://media.tenor.com/{id}AAAAM/{slug}.gif
    // Original:  https://media.tenor.com/{id}AAAAd/{slug}.gif
    let re = match Regex::new(r#"https://media\.tenor\.com/([^"]+?)AAAAM/([^"]+?\.gif)"#) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let mut seen = std::collections::HashSet::new();
    re.captures_iter(&html)
        .filter_map(|cap| {
            let id = cap.get(1)?.as_str();
            let slug = cap.get(2)?.as_str();
            let full_url = format!("https://media.tenor.com/{id}AAAAd/{slug}");
            if seen.insert(full_url.clone()) {
                Some(full_url)
            } else {
                None
            }
        })
        .take(limit)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_returns_ranked_results() {
        let results = search_gifs("gintoki gintama", 5);
        assert!(!results.is_empty());
        assert!(results.len() <= 5);
        for url in &results {
            assert!(url.contains("tenor.com/"));
            assert!(url.ends_with(".gif"));
        }
    }

    #[test]
    fn search_niche_character() {
        let results = search_gifs("satoko houjou higurashi", 3);
        assert!(!results.is_empty());
    }
}
