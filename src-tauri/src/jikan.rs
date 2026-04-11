use serde::Deserialize;

const JIKAN_BASE: &str = "https://api.jikan.moe/v4";

#[derive(Deserialize)]
struct CharactersResponse {
    data: Vec<CharacterEntry>,
}

#[derive(Deserialize)]
struct CharacterEntry {
    character: CharacterInfo,
}

#[derive(Deserialize)]
struct CharacterInfo {
    name: String,
    images: CharacterImages,
}

#[derive(Deserialize)]
struct CharacterImages {
    jpg: ImageUrls,
}

#[derive(Deserialize)]
struct ImageUrls {
    image_url: Option<String>,
}

/// Fetch the character image URL from Jikan using the anime's MAL ID.
/// Searches the anime's character list for a name match and returns the image URL.
pub fn get_character_image(mal_id: u64, character_name: &str) -> Option<String> {
    let url = format!("{JIKAN_BASE}/anime/{mal_id}/characters");
    let resp = reqwest::blocking::Client::builder()
        .user_agent("Dagashi/0.1.0")
        .build()
        .ok()?
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let body: CharactersResponse = resp.json().ok()?;
    let char_lower = character_name.to_lowercase();

    // Try exact match first, then substring match
    let entry = body
        .data
        .iter()
        .find(|e| e.character.name.to_lowercase() == char_lower)
        .or_else(|| {
            // Jikan uses "Last, First" format — also check reversed
            body.data.iter().find(|e| {
                let name = e.character.name.to_lowercase();
                char_lower.split_whitespace().all(|part| name.contains(part))
            })
        });

    entry.and_then(|e| e.character.images.jpg.image_url.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_l_from_death_note() {
        // Death Note MAL ID = 1535
        let url = get_character_image(1535, "L Lawliet");
        assert!(url.is_some());
        assert!(url.unwrap().contains("myanimelist"));
    }
}
