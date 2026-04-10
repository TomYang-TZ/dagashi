use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::RarityThresholds;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn label(&self) -> &'static str {
        match self {
            Rarity::Common => "common",
            Rarity::Uncommon => "uncommon",
            Rarity::Rare => "rare",
            Rarity::Epic => "epic",
            Rarity::Legendary => "legendary",
        }
    }
}

/// Base probabilities: Common 50%, Uncommon 30%, Rare 15%, Epic 4%, Legendary 1%
const BASE_WEIGHTS: [f64; 5] = [50.0, 30.0, 15.0, 4.0, 1.0];

/// Roll a rarity tier. Higher volume shifts odds toward rarer tiers.
/// Each tier gets a sigmoid boost based on volume vs threshold:
///   boost = volume^2 / (volume^2 + threshold^2)
/// This creates a smooth S-curve: negligible boost below threshold,
/// approaches 1.0 well above it. Higher tiers (higher threshold) need
/// more volume to get the same boost, preserving the rarity ladder.
/// Common weight shrinks as other tiers grow (via normalization).
pub fn roll_rarity(total_keystrokes: u64, thresholds: &RarityThresholds) -> Rarity {
    let vol = total_keystrokes as f64;
    let tier_thresholds = [
        thresholds.uncommon as f64,
        thresholds.rare as f64,
        thresholds.epic as f64,
        thresholds.legendary as f64,
    ];

    // Common stays at base weight; higher tiers get sigmoid boost
    let mut weights = vec![BASE_WEIGHTS[0]]; // Common: fixed
    for (i, thresh) in tier_thresholds.iter().enumerate() {
        let boost = vol * vol / (vol * vol + thresh * thresh); // 0..1 sigmoid
        // At threshold volume, boost = 0.5. Well above, approaches 1.0.
        // Multiply base weight by (1 + boost * multiplier)
        weights.push(BASE_WEIGHTS[i + 1] * (1.0 + boost * 3.0));
    }

    let total: f64 = weights.iter().sum();
    for w in &mut weights {
        *w /= total;
    }

    let mut rng = rand::thread_rng();
    let roll: f64 = rng.gen();
    let mut cumulative = 0.0;
    for (i, w) in weights.iter().enumerate() {
        cumulative += w;
        if roll < cumulative {
            return match i {
                0 => Rarity::Common,
                1 => Rarity::Uncommon,
                2 => Rarity::Rare,
                3 => Rarity::Epic,
                4 => Rarity::Legendary,
                _ => Rarity::Common,
            };
        }
    }
    Rarity::Common
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_thresholds() -> RarityThresholds {
        RarityThresholds {
            uncommon: 10_000,
            rare: 30_000,
            epic: 60_000,
            legendary: 100_000,
        }
    }

    #[test]
    fn roll_always_returns_valid_rarity() {
        let t = default_thresholds();
        for vol in [0, 100, 10_000, 50_000, 100_000, 500_000] {
            let r = roll_rarity(vol, &t);
            assert!(["common", "uncommon", "rare", "epic", "legendary"].contains(&r.label()));
        }
    }

    #[test]
    fn high_volume_shifts_distribution() {
        let t = default_thresholds();
        let mut rare_plus_low = 0;
        let mut rare_plus_high = 0;
        let n = 50_000;
        for _ in 0..n {
            if matches!(roll_rarity(100, &t), Rarity::Rare | Rarity::Epic | Rarity::Legendary) {
                rare_plus_low += 1;
            }
            if matches!(roll_rarity(200_000, &t), Rarity::Rare | Rarity::Epic | Rarity::Legendary) {
                rare_plus_high += 1;
            }
        }
        // With 50k samples, high volume should reliably produce more rare+ pulls
        assert!(
            rare_plus_high > rare_plus_low,
            "high={rare_plus_high} should be > low={rare_plus_low}"
        );
    }
}
