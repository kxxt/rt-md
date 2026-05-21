use std::{fmt::Display, net::IpAddr};

use fasthash::FastHash;

use crate::{algo::whllpp::WeightedHyperLogLogPlusPlus, allowlist::AllowList, dataset::Datum};

use super::{AlertSummary, DetectionMethod};

pub struct Ibhh {
    counters: hashbrown::HashMap<String, WeightedHyperLogLogPlusPlus>,
    seeds: hashbrown::HashMap<String, u128>,
    k: usize,
    threshold: u128,
    orig_threshold: u128,
}

impl Ibhh {
    pub fn new(k: usize, threshold: u128) -> Self {
        Self {
            counters: Default::default(),
            seeds: Default::default(),
            k,
            threshold,
            orig_threshold: threshold,
        }
    }

    pub fn add_pair(&mut self, subdomain: &str, domain: &str) -> color_eyre::Result<()> {
        let subdomain = subdomain.to_lowercase();
        let domain = domain.to_lowercase();
        let hash = fasthash::murmur3::Hash128_x64::hash(subdomain.clone() + &domain);
        if let Some(counter) = self.counters.get_mut(&domain) {
            counter.insert(&subdomain);
            if hash < self.seeds[&domain] {
                self.seeds.insert(domain.to_string(), hash);
            }
        } else if hash < self.threshold {
            let mut counter = WeightedHyperLogLogPlusPlus::new()?;
            counter.insert(&subdomain);
            self.counters.insert(domain.to_string(), counter);
            self.seeds.insert(domain.to_string(), hash);

            if self.seeds.len() > self.k {
                let (max_seed_domain, &max_seed) =
                    self.seeds.iter().max_by(|a, b| a.1.cmp(b.1)).unwrap();
                // Make borrowck happy
                let max_seed_domain = max_seed_domain.to_owned();
                self.threshold = max_seed;
                self.counters.remove(&max_seed_domain);
                self.seeds.remove(&max_seed_domain);
            }
        }
        Ok(())
    }

    pub fn count_information(&mut self, domain: &str) -> f64 {
        if let Some(counter) = self.counters.get_mut(&domain.to_lowercase()) {
            counter.count()
        } else {
            0.0
        }
    }

    pub fn reset(&mut self) {
        self.threshold = self.orig_threshold;
        self.counters.clear();
        self.seeds.clear();
    }
}

pub struct IbhhMethod<L: AllowList> {
    ibhh: Ibhh,
    threshold: f64,
    time: u64,
    detection_window: u64,
    allowlist: L,
}

#[derive(Debug)]
pub struct IbhhAlertSummary {
    pub(super) client: IpAddr,
    pub(super) total: f64,
    pub(super) domain: String,
}

impl Display for IbhhAlertSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Domain {} is suspicious, total={}, client={}",
            self.domain, self.total, self.client
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IbhhAlertKind {}

impl Display for IbhhAlertKind {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl AlertSummary for IbhhAlertSummary {
    type AlertKind = IbhhAlertKind;

    fn kind(&self) -> Self::AlertKind {
        todo!()
    }

    fn domains(&self) -> impl Iterator<Item = (&str, f64)> {
        [(self.domain.as_str(), self.total)].into_iter()
    }

    fn clients(&self) -> impl Iterator<Item = (IpAddr, u32)> {
        [(self.client, self.total as u32)].into_iter()
    }
}

impl<L: AllowList> IbhhMethod<L> {
    pub fn new(threshold: f64, detection_window: u64, k: usize, allowlist: L) -> Self {
        Self {
            threshold,
            time: 0,
            detection_window,
            allowlist,
            ibhh: Ibhh::new(k, u128::MAX),
        }
    }
}

impl<L: AllowList> DetectionMethod for IbhhMethod<L> {
    type TAlert = IbhhAlertSummary;

    fn reset_interval(&self) -> u32 {
        self.detection_window as u32
    }

    fn process_single(
        &mut self,
        datum: Datum<'_>,
    ) -> color_eyre::Result<(Option<Self::TAlert>, f64)> {
        let time = datum.timestamp_ms;
        if time > self.time + self.detection_window {
            self.ibhh.reset();
            self.time = time;
        }
        self.ibhh.add_pair(datum.subdomain, datum.domain).unwrap();
        let count = self.ibhh.count_information(datum.domain);
        Ok(if count > self.threshold {
            if self.allowlist.contains(datum.domain, datum.suffix) {
                (None, 0.)
            } else {
                (
                    Some(IbhhAlertSummary {
                        client: datum.client,
                        total: count,
                        domain: datum.domain.to_ascii_lowercase(),
                    }),
                    count,
                )
            }
        } else {
            (None, count)
        })
    }
}
