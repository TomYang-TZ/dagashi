use serde::Deserialize;
use std::collections::HashMap;

const FANDOM_API_SUFFIX: &str = "fandom.com/api.php";

#[derive(Deserialize)]
struct WikiQueryResponse {
    query: Option<WikiQuery>,
}

#[derive(Deserialize)]
struct WikiQuery {
    pages: HashMap<String, WikiPage>,
}

#[derive(Deserialize)]
struct WikiPage {
    images: Option<Vec<WikiImage>>,
    imageinfo: Option<Vec<WikiImageInfo>>,
}

#[derive(Deserialize)]
struct WikiImage {
    title: String,
}

#[derive(Deserialize)]
struct WikiImageInfo {
    url: String,
}

/// Derive a fandom wiki subdomain from an anime title.
/// e.g. "Higurashi no Naku Koro ni Gou" -> "higurashi"
///      "Gintama" -> "gintama"
///      "Attack on Titan" -> "attackontitan"
fn fandom_subdomain(anime_title: &str) -> String {
    // Take the first word (or first two if short) as the subdomain.
    // Strip particles/suffixes common in anime titles.
    let lower = anime_title.to_lowercase();
    let skip = ["no", "na", "ni", "wo", "ga", "de", "the", "of", "on", "in", "a"];
    let words: Vec<&str> = lower
        .split_whitespace()
        .filter(|w| !skip.contains(w))
        .collect();

    // Use first meaningful word; most fandom wikis use a short slug
    let slug = if words.is_empty() {
        lower.split_whitespace().next().unwrap_or("anime").to_string()
    } else {
        words[0].to_string()
    };

    // Remove non-alphanumeric chars
    slug.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Get image URLs from a character's fandom wiki page.
pub fn get_character_images(character_name: &str, anime_title: &str) -> Vec<String> {
    let subdomain = fandom_subdomain(anime_title);
    let wiki_api = format!("https://{subdomain}.{FANDOM_API_SUFFIX}");
    get_character_images_from_wiki(character_name, &wiki_api)
}

fn get_character_images_from_wiki(character_name: &str, wiki_api: &str) -> Vec<String> {
    let title = character_name.replace(' ', "_");

    // Step 1: Get image filenames from the character page
    let url = format!(
        "{wiki_api}?action=query&titles={title}&prop=images&imlimit=50&format=json"
    );
    let filenames = match fetch_filenames(&url) {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    if filenames.is_empty() {
        return vec![];
    }

    // Step 2: Resolve filenames to direct URLs
    let mut urls = Vec::new();
    for filename in filenames {
        let url = format!(
            "{wiki_api}?action=query&titles={}&prop=imageinfo&iiprop=url&format=json",
            urlencoded(&filename)
        );
        if let Ok(image_url) = fetch_image_url(&url) {
            urls.push(image_url);
        }
    }
    urls
}

fn fetch_filenames(url: &str) -> Result<Vec<String>, String> {
    let resp: WikiQueryResponse = reqwest::blocking::get(url)
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let pages = resp.query.ok_or("no query")?.pages;
    let mut filenames = Vec::new();
    for page in pages.values() {
        if let Some(images) = &page.images {
            for img in images {
                filenames.push(img.title.clone());
            }
        }
    }
    Ok(filenames)
}

fn fetch_image_url(url: &str) -> Result<String, String> {
    let resp: WikiQueryResponse = reqwest::blocking::get(url)
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let pages = resp.query.ok_or("no query")?.pages;
    for page in pages.values() {
        if let Some(info) = &page.imageinfo {
            if let Some(first) = info.first() {
                return Ok(first.url.clone());
            }
        }
    }
    Err("no image url found".to_string())
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
    fn get_gintoki_images() {
        let urls = get_character_images("Gintoki Sakata", "Gintama");
        assert!(!urls.is_empty());
        for url in &urls {
            assert!(url.contains("static.wikia.nocookie.net"));
        }
    }

    #[test]
    fn fandom_subdomain_extracts_slug() {
        assert_eq!(fandom_subdomain("Gintama"), "gintama");
        assert_eq!(fandom_subdomain("Higurashi no Naku Koro ni Gou"), "higurashi");
        assert_eq!(fandom_subdomain("Attack on Titan"), "attack");
    }
}
