use hashbrown::HashMap;
use itertools::Itertools;

pub mod algo;
pub mod allowlist;
pub mod cache;
pub mod dataset;
pub mod domain;
pub mod method;
pub mod syslog;

pub fn threshold_tuning<K>(values: &HashMap<K, f64>, acceptable_fpr: f64, reset_interval: f64) {
    let acceptable_fp_count = (acceptable_fpr * values.len() as f64).ceil() as usize;
    let mut values = values
        .values()
        .copied()
        .sorted_by(|a, b| f64::total_cmp(b, a));
    let raw_threshold = values.nth(acceptable_fp_count).unwrap() + 1.;
    println!(
        "TunedRawThreshold for {}: {}",
        acceptable_fpr, raw_threshold
    );
    println!(
        "TunedThreshold {}: {}",
        acceptable_fpr,
        (raw_threshold / (reset_interval / 1000.))
    );
}
