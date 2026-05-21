use std::{
    borrow::Cow,
    fmt::Display,
    net::IpAddr,
};

use chrono::{DateTime, Utc};
use fastbloom::BloomFilter;
use itertools::Itertools;
use priority_queue::PriorityQueue;
use serde::Serialize;
use streaming_algorithms::Top;
use unicase::UniCase;

use crate::{
    algo::salt::{Salt, salt},
    allowlist::AllowList,
    cache::TimeToIdleCache,
    dataset::Datum,
};

use super::{AlertSummary, DetectionMethod};

// pub for measuring resource usage.
pub struct Bfcms {
    pub top: streaming_algorithms::Top<String, u32>,
    bf: BloomFilter,
    total: u32,
    last_reset_time: u64,
    salt: Salt,
}

impl Bfcms {
    pub fn new(timestamp_ms: u64) -> color_eyre::Result<Self> {
        Ok(Self {
            top: Top::new(75, 0.95, 2.0 / 100.0, ()),
            bf: BloomFilter::with_false_pos(0.01).expected_items(20000),
            total: 0,
            last_reset_time: timestamp_ms,
            salt: salt()?,
        })
    }

    pub fn reset(&mut self, timestamp_ms: u64) -> color_eyre::Result<()> {
        self.top.clear();
        self.bf.clear();
        self.total = 0;
        self.last_reset_time = timestamp_ms;
        self.salt = salt()?;
        Ok(())
    }
}

pub struct BfcmsMethod<C: TimeToIdleCache<Bfcms>> {
    clients: C,
    threshold: u32,
    detection_window: u64,
    allowlist: Box<dyn AllowList>,
    trust_rdns: bool,
    rdns_special_case: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BfcmsAlertSummary {
    ts: chrono::DateTime<Utc>,
    client: IpAddr,
    pub total: u32,
    pub top_domains: PriorityQueue<String, u32>,
}

impl Display for BfcmsAlertSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} Client {} is suspicious, total={}, top_domains={:?}",
            self.ts.format("%c"),
            self.client,
            self.total,
            self.top_domains
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BfcmsAlertKind {}

impl Display for BfcmsAlertKind {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl AlertSummary for BfcmsAlertSummary {
    type AlertKind = BfcmsAlertKind;

    fn kind(&self) -> Self::AlertKind {
        todo!()
    }

    fn domains(&self) -> impl Iterator<Item = (&str, f64)> {
        self.top_domains
            .iter()
            .map(|(x, y)| (x.as_str(), *y as f64))
    }

    fn clients(&self) -> impl Iterator<Item = (IpAddr, u32)> {
        [(self.client, self.total)].into_iter()
    }
}

/// Measure the information in the rDNS lookup
fn rdns_measure(
    datum: &Datum<'_>,
    trust_rdns: bool,
    rdns_special_case: bool,
) -> Option<(Cow<'static, str>, u32)> {
    // We assume all rdns lookups we process are valid
    // It is up to other IDS to block malformed rDNS lookups.
    if UniCase::ascii(datum.suffix) == UniCase::ascii("in-addr.arpa") {
        if trust_rdns {
            return Some(("".into(), 0));
        }
        // e.g. 1.1.1.in-addr.arpa
        // every digit in the lookup can only take 0 to 9,
        // so for this query there are 3 digits thus they can convey
        // log_2 (10^3) = 3 log_2 10 = 3 * 3.321928095 bits of information.
        // Here we just take 4 as an upper bound for the bits of information each digit can convey.
        //
        // Then we need to convert bits of information to the unit we used.
        // For all other lookups, every char can only take a value from [a-z0-9.-_].
        // In other words, they can take 39 different values.
        // So the conversion to this unit works as follows:
        //
        // log_39 10^3 = 1.8855
        //
        // I = n * ln11 / ln39
        //
        // where ln11 / ln39 is 0.6285099898418802, we will use 2/3,
        // which is a convergent of the continued fraction of ln10 / ln39

        // we attribute information to ipv4/16 or ipv6/48.
        let mut labels = datum
            .full
            .strip_suffix(datum.suffix)
            .unwrap_or("")
            .split('.')
            .rev()
            .skip(1); // Empty
        let first = labels.next();
        let second = labels.next();
        Some((
            format!(
                "RDNS {}.{}.0.0/16",
                first.unwrap_or_default(),
                second.unwrap_or_default()
            )
            .into(),
            if rdns_special_case {
                (labels.map(|v| v.len()).sum::<usize>() as f32 * f32::log(11.0, 39.0)).ceil() as u32
            } else {
                (labels.map(|v| v.len()).intersperse(1).sum::<usize>()) as u32
            },
        ))
    } else if UniCase::ascii(datum.suffix) == UniCase::ascii("ip6.arpa") {
        if trust_rdns {
            return Some(("".into(), 0));
        }
        // e.g. 0.0.0.0.7.1.4.1.0.0.2.ip6.arpa
        // each label contains only one hex digit, which can take one of 16 values.
        //
        // Following our calculation for the ipv4 case, here
        //
        // I = n * ln17 / ln39 = n * 0.7568014380674801
        //
        // we will use 25/33, which is a convergent of the continued fraction of ln16 / ln39
        let mut labels = datum
            .full
            .strip_suffix(datum.suffix)
            .unwrap_or("")
            .split('.')
            .rev()
            .skip(1)
            .peekable(); // Empty
        let mut a = [""; 12];
        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            a[i] = labels.next().unwrap_or_default();
        }
        if rdns_special_case
            && a[0] == "f"
            && ((a[1] == "d" || a[2] == "c") || (a[1] == "e" && a[2] == "8" && a[3] == "0"))
        {
            // ULA or Link-local
            return Some(("".into(), 0));
        }
        #[allow(clippy::needless_range_loop)]
        for i in 4..12 {
            a[i] = labels.next().unwrap_or_default();
        }
        Some((
            format!(
                "RDNS {}{}{}{}:{}{}{}{}:{}{}{}{}::/48",
                a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8], a[9], a[10], a[11],
            )
            .into(),
            if rdns_special_case {
                (labels.map(|v| v.len()).sum::<usize>() as f32 * f32::log(17.0, 39.0)).ceil() as u32
            } else {
                (labels.map(|v| v.len()).intersperse(1).sum::<usize>()) as u32
            },
        ))
    } else {
        None
    }
}

impl<C: TimeToIdleCache<Bfcms>> BfcmsMethod<C> {
    pub fn new(
        threshold: u32,
        detection_window: u64,
        allowlist: impl AllowList + 'static,
        trust_rdns: bool,
        rdns_special_case: bool,
    ) -> Self {
        Self {
            clients: C::new(2 * detection_window),
            threshold,
            detection_window,
            allowlist: Box::new(allowlist) as Box<dyn AllowList>,
            trust_rdns,
            rdns_special_case,
        }
    }

    fn process(
        bfcms: &mut Bfcms,
        datum: Datum<'_>,
        threshold: u32,
        window: u64,
        allow: &dyn AllowList,
        trust_rdns: bool,
        rdns_special_case: bool,
    ) -> color_eyre::Result<(Option<BfcmsAlertSummary>, f64)> {
        if allow.contains(datum.domain, datum.suffix) || datum.subdomain.is_empty() {
            return Ok((None, 0.));
        }
        let mut domain: Cow<'_, str> = Cow::Owned(datum.domain.to_ascii_lowercase());
        let subdomain = datum.subdomain.to_ascii_lowercase();
        Ok(
            if bfcms.bf.insert(&(subdomain.as_str(), &domain, &bfcms.salt)) {
                // This has been visited before. Not counting
                (None, 0.)
            } else {
                // Reset mechanism
                if datum.timestamp_ms > bfcms.last_reset_time + window {
                    bfcms.reset(datum.timestamp_ms)?;
                }
                let i = match rdns_measure(&datum, trust_rdns, rdns_special_case) {
                    Some((attribution_domain, i)) => {
                        if allow.contains(&attribution_domain, "") {
                            return Ok((None, 0.));
                        }
                        domain = attribution_domain;
                        i
                    }
                    None => subdomain.len() as u32,
                };

                if i == 0 {
                    return Ok((None, 0.));
                }

                // A new unique visit
                bfcms.top.push(domain.into_owned(), &i);
                let client = datum.client;
                bfcms.total = bfcms.total.saturating_add(i);

                // Re-check domains in alert in case of recently allowlisted domains
                let deduction = if allow.mutable() && bfcms.total > threshold {
                    bfcms
                        .top
                        .iter()
                        .filter_map(|(d, v)| allow.contains(d, "").then_some(*v))
                        .sum()
                } else {
                    0
                };
                let total = bfcms.total - deduction;

                // Check if this client is bad
                if total > threshold {
                    (
                        Some(BfcmsAlertSummary {
                            ts: DateTime::from_timestamp_millis(datum.timestamp_ms as i64).unwrap(),
                            client,
                            total,
                            top_domains: bfcms
                                .top
                                .iter()
                                .filter(|(d, _)| !allow.contains(d, ""))
                                .map(|(a, b)| (a.clone(), *b))
                                .collect(),
                        }),
                        total as f64,
                    )
                } else {
                    (None, total as f64)
                }
            },
        )
    }
}

impl<C: TimeToIdleCache<Bfcms>> DetectionMethod for BfcmsMethod<C> {
    type TAlert = BfcmsAlertSummary;

    fn reset_interval(&self) -> u32 {
        self.detection_window as u32
    }

    fn process_single(
        &mut self,
        datum: Datum<'_>,
    ) -> color_eyre::Result<(Option<Self::TAlert>, f64)> {
        let now = datum.timestamp_ms;
        
        if let Some(bfcms) = self.clients.get_mut(datum.client, now) {
            Self::process(
                bfcms,
                datum,
                self.threshold,
                self.detection_window,
                self.allowlist.as_ref(),
                self.trust_rdns,
                self.rdns_special_case,
            )
        } else {
            // TODO: limit the number of clients
            let mut bfcms = Bfcms::new(datum.timestamp_ms)?;
            let client = datum.client;
            let r = Self::process(
                &mut bfcms,
                datum,
                self.threshold,
                self.detection_window,
                self.allowlist.as_ref(),
                self.trust_rdns,
                self.rdns_special_case,
            );
            self.clients.insert(client, bfcms, now);
            r
        }
    }
}
