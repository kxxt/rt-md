//! Weighted HyperLogLog++
//!
//! Adapted from https://github.com/akamai/Information-based-Heavy-Hitters-for-Real-Time-DNS-Exfiltration-Detection

use hyperloglogplus::{HyperLogLog, HyperLogLogPlus};

pub struct WeightedHyperLogLogPlusPlus {
    inner: HyperLogLogPlus<Vec<u8>, fasthash::murmur2::Hash64_x64>,
}

impl WeightedHyperLogLogPlusPlus {
    pub fn new() -> color_eyre::Result<Self> {
        Ok(Self {
            inner: HyperLogLogPlus::new(12, fasthash::murmur2::Hash64_x64)?,
        })
    }

    pub fn insert(&mut self, s: &str) {
        let mut owned = s.to_string().into_bytes();
        for i in 0..owned.len() {
            owned[i] += 1;
            self.inner.insert(&owned);
            owned[i] -= 1;
        }
    }

    pub fn count(&mut self) -> f64 {
        self.inner.count()
    }
}
