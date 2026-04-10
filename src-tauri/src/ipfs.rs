use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;

use crate::image_pipeline::PipelineResult;
use crate::storage::PullMeta;

/// IPFS receipt — a verifiable proof of a pull, pinned to IPFS via Pinata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullReceipt {
    pub version: u8,
    pub date: String,
    pub character: String,
    pub anime_title: String,
    pub anime_rank: u32,
    pub rarity: String,
    pub color_mode: String,
    pub frame_count: usize,
    pub frames_hash: String, // SHA-256 hex of the frames.json content
    pub flavor_text: String,
}

/// Build a receipt from pull metadata and frame data.
pub fn build_receipt(meta: &PullMeta, pipeline: &PipelineResult) -> Result<PullReceipt, String> {
    // Hash the frame data for content verification
    let frames_json = serde_json::to_string(pipeline).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(frames_json.as_bytes());
    let hash = hasher.finalize();
    let frames_hash = hash.iter().map(|b| format!("{b:02x}")).collect::<String>();

    Ok(PullReceipt {
        version: 1,
        date: meta.date.clone(),
        character: meta.character.clone(),
        anime_title: meta.anime_title.clone(),
        anime_rank: meta.anime_rank,
        rarity: meta.rarity.clone(),
        color_mode: meta.color_mode.clone(),
        frame_count: meta.frame_count,
        frames_hash,
        flavor_text: meta.flavor_text.clone(),
    })
}

/// Pin a receipt to IPFS via Pinata's pinJSONToIPFS endpoint.
/// Returns the CID (content identifier) on success.
pub fn pin_receipt(receipt: &PullReceipt, jwt: &str) -> Result<String, String> {
    let payload = serde_json::json!({
        "pinataContent": receipt,
        "pinataMetadata": {
            "name": format!("dagashi-pull-{}-{}", receipt.date, receipt.character),
        }
    });

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://api.pinata.cloud/pinning/pinJSONToIPFS")
        .header("Authorization", format!("Bearer {jwt}"))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .map_err(|e| format!("pinata request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("pinata returned {status}: {body}"));
    }

    #[derive(Deserialize)]
    struct PinataResponse {
        #[serde(rename = "IpfsHash")]
        ipfs_hash: String,
    }

    let pin_resp: PinataResponse = resp
        .json()
        .map_err(|e| format!("failed to parse pinata response: {e}"))?;

    Ok(pin_resp.ipfs_hash)
}

/// Save receipt JSON and CID to the pull's directory.
pub fn save_receipt(date: &str, receipt: &PullReceipt, cid: &str) {
    let dir = crate::storage::pulls_dir().join(date);
    fs::create_dir_all(&dir).ok();

    // Save receipt
    if let Ok(json) = serde_json::to_string_pretty(receipt) {
        fs::write(dir.join("receipt.json"), json).ok();
    }

    // Save CID
    fs::write(dir.join("cid.txt"), cid).ok();
}

/// Load a pull's CID if it exists.
pub fn load_pull_cid(date: &str) -> Option<String> {
    let path = crate::storage::pulls_dir().join(date).join("cid.txt");
    fs::read_to_string(path).ok().filter(|s| !s.is_empty())
}
