use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyStats {
    pub date: String,
    pub total: u64,
    pub chars: HashMap<String, u64>,
    pub categories: CategoryCounts,
    pub backspace_count: u64,
    pub shift_count: u64,
    pub capslock_count: u64,
    pub hourly_volume: Vec<u64>,
    pub regions: RegionCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryCounts {
    pub letter: u64,
    pub number: u64,
    pub symbol: u64,
    pub modifier: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegionCounts {
    pub left_hand: u64,
    pub right_hand: u64,
    pub home_row: u64,
}
