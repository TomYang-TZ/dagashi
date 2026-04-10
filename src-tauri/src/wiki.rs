use serde::Deserialize;
use std::collections::HashMap;

const WIKI_API: &str = "https://gintama.fandom.com/api.php";

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

/// Get image URLs from a Gintama character's wiki page.
pub fn get_character_images(character_name: &str) -> Vec<String> {
    let title = character_name.replace(' ', "_");

    // Step 1: Get image filenames from the character page
    let url = format!(
        "{WIKI_API}?action=query&titles={title}&prop=images&imlimit=50&format=json"
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
            "{WIKI_API}?action=query&titles={}&prop=imageinfo&iiprop=url&format=json",
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
        let urls = get_character_images("Gintoki Sakata");
        assert!(!urls.is_empty());
        for url in &urls {
            assert!(url.contains("static.wikia.nocookie.net"));
        }
    }
}
