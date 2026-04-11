use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::RarityThresholds;
use crate::stats::DailyStats;

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

/// Roll whether this pull gets color or mono rendering.
/// Color is harder to get — probability scales with typing engagement:
///   - Volume: more keystrokes → higher chance (sigmoid, threshold ~20k)
///   - Peak intensity: concentrated bursts show focus (top hour / avg hour ratio)
///   - Character diversity: using more unique chars shows range
/// Base 30%, caps at ~70%.
pub fn roll_color(stats: &DailyStats) -> bool {
    let mut prob: f64 = 0.30; // base 30%

    // Factor 1: Volume boost (sigmoid, midpoint at 20k keystrokes)
    // At 20k, boost = 0.5 * 0.20 = 0.10. At 60k+, approaches 0.20.
    let vol = stats.total as f64;
    let vol_thresh = 20_000.0;
    let vol_boost = vol * vol / (vol * vol + vol_thresh * vol_thresh);
    prob += vol_boost * 0.20;

    // Factor 2: Peak hour intensity
    // If your top hour has 3x+ the average, you had a focused session → reward it.
    if !stats.hourly_volume.is_empty() {
        let active_hours: Vec<&u64> = stats.hourly_volume.iter().filter(|&&v| v > 0).collect();
        if active_hours.len() >= 2 {
            let avg = active_hours.iter().copied().sum::<u64>() as f64 / active_hours.len() as f64;
            let peak = *stats.hourly_volume.iter().max().unwrap_or(&0) as f64;
            if avg > 0.0 {
                let concentration = (peak / avg).min(5.0); // cap ratio at 5x
                // ratio of 3.0 → boost ~0.10, ratio of 5.0 → boost ~0.15
                prob += ((concentration - 1.0) / 4.0).max(0.0) * 0.15;
            }
        }
    }

    // Factor 3: Character diversity
    // More unique characters typed → small boost. 30+ unique chars is good variety.
    let unique_chars = stats.chars.len() as f64;
    let diversity_boost = (unique_chars / 50.0).min(1.0) * 0.10;
    prob += diversity_boost;

    // Cap at 70%
    prob = prob.min(0.70);

    let roll: f64 = rand::thread_rng().gen();
    eprintln!("[dagashi] Color probability: {:.1}% (rolled {:.3})", prob * 100.0, roll);
    roll < prob
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

    fn make_stats(total: u64, unique_chars: usize, hourly: Vec<u64>) -> DailyStats {
        let mut chars = std::collections::HashMap::new();
        for i in 0..unique_chars {
            chars.insert(format!("{}", (b'a' + (i % 26) as u8) as char), total / unique_chars.max(1) as u64);
        }
        DailyStats {
            date: "2026-04-10".to_string(),
            total,
            chars,
            hourly_volume: hourly,
            ..Default::default()
        }
    }

    #[test]
    fn color_base_rate_is_low_for_minimal_typing() {
        let stats = make_stats(100, 5, vec![100; 1]);
        let n = 10_000;
        let colors: usize = (0..n).filter(|_| roll_color(&stats)).count();
        let rate = colors as f64 / n as f64;
        // Should be close to base rate (~30-40%)
        assert!(rate < 0.55, "minimal typing color rate too high: {:.1}%", rate * 100.0);
    }

    #[test]
    fn color_rate_increases_with_engagement() {
        let low = make_stats(500, 5, vec![500; 1]);
        let high = make_stats(80_000, 40, {
            let mut h = vec![0u64; 24];
            h[10] = 30_000; // big peak
            h[14] = 20_000;
            h[16] = 15_000;
            h[20] = 15_000;
            h
        });
        let n = 20_000;
        let low_colors: usize = (0..n).filter(|_| roll_color(&low)).count();
        let high_colors: usize = (0..n).filter(|_| roll_color(&high)).count();
        assert!(
            high_colors > low_colors,
            "high engagement ({}) should beat low ({})",
            high_colors, low_colors
        );
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
