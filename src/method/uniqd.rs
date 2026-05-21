use std::{borrow::Cow, fmt::Display, net::IpAddr};

use chrono::{DateTime, Utc};
use hyperloglogplus::{HyperLogLog, HyperLogLogPlus};
use unicase::UniCase;

use crate::{
    algo::salt::{Salt, salt},
    allowlist::AllowList,
    cache::TimeToIdleCache,
    dataset::Datum,
};

use super::{AlertSummary, DetectionMethod};

// pub for measuring resource usage.
pub struct Uniqd {
    counter: HyperLogLogPlus<Vec<u8>, fasthash::murmur2::Hash64_x64>,
    last_reset_time: u64,
    salt: Salt,
}

impl Uniqd {
    pub fn new(timestamp_ms: u64) -> color_eyre::Result<Self> {
        Ok(Self {
            counter: HyperLogLogPlus::new(12, fasthash::murmur2::Hash64_x64)?,
            last_reset_time: timestamp_ms,
            salt: salt()?,
        })
    }

    pub fn reset(&mut self, timestamp_ms: u64) -> color_eyre::Result<()> {
        self.counter = HyperLogLogPlus::new(12, fasthash::murmur2::Hash64_x64)?;
        self.last_reset_time = timestamp_ms;
        self.salt = salt()?;
        Ok(())
    }

    pub fn insert(&mut self) {
        todo!()
    }

    pub fn count(&mut self) -> u32 {
        self.counter.count() as u32
    }
}

pub struct UniqdMethod<C: TimeToIdleCache<Uniqd>> {
    clients: C,
    threshold: u32,
    detection_window: u64,
    allowlist: Box<dyn AllowList>,
    trust_rdns: bool,
}

#[derive(Debug)]
pub struct UniqdAlertSummary {
    ts: chrono::DateTime<Utc>,
    client: IpAddr,
    total: u32,
}

impl Display for UniqdAlertSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} Client {} is suspicious, total={}",
            self.ts.format("%c"),
            self.client,
            self.total,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UniqdAlertKind {}

impl Display for UniqdAlertKind {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl AlertSummary for UniqdAlertSummary {
    type AlertKind = UniqdAlertKind;

    fn kind(&self) -> Self::AlertKind {
        todo!()
    }

    fn domains(&self) -> impl Iterator<Item = (&str, f64)> {
        [].into_iter()
    }

    fn clients(&self) -> impl Iterator<Item = (IpAddr, u32)> {
        [(self.client, self.total)].into_iter()
    }
}

fn is_rdns(datum: &Datum<'_>) -> bool {
    // We assume all rdns lookups we process are valid
    // It is up to other IDS to block malformed rDNS lookups.
    UniCase::ascii(datum.suffix) == UniCase::ascii("in-addr.arpa")
        || UniCase::ascii(datum.suffix) == UniCase::ascii("ip6.arpa")
}

impl<C: TimeToIdleCache<Uniqd>> UniqdMethod<C> {
    pub fn new(
        threshold: u32,
        detection_window: u64,
        allowlist: impl AllowList + 'static,
        trust_rdns: bool,
    ) -> Self {
        Self {
            clients: C::new(2 * detection_window),
            threshold,
            detection_window,
            allowlist: Box::new(allowlist) as Box<dyn AllowList>,
            trust_rdns,
        }
    }

    fn process(
        uniqd: &mut Uniqd,
        datum: Datum<'_>,
        threshold: u32,
        window: u64,
        allow: &dyn AllowList,
        trust_rdns: bool,
    ) -> color_eyre::Result<(Option<UniqdAlertSummary>, f64)> {
        if allow.contains(datum.domain, datum.suffix) || datum.subdomain.is_empty() {
            // eprintln!("Allowed {}", datum.domain);
            // || datum.domain.ends_with(".ip6.arpa") || datum.domain.ends_with(".in-addr.arpa")
            // eprintln!("{} is safe", datum.domain);
            return Ok((None, 0.));
        }
        let domain: Cow<'_, str> = Cow::Owned(datum.domain.to_ascii_lowercase());
        let subdomain = datum.subdomain.to_ascii_lowercase();
        if trust_rdns && is_rdns(&datum) {
            return Ok((None, 0.));
        }

        // Reset mechanism
        if datum.timestamp_ms > uniqd.last_reset_time + window {
            // eprintln!("Reset");
            uniqd.reset(datum.timestamp_ms)?;
        }

        uniqd
            .counter
            .insert_any(&(subdomain.as_str(), &domain, &uniqd.salt));

        Ok({
            let client = datum.client;
            let total = uniqd.count();
            // Check if this client is bad
            if total > threshold {
                (
                    Some(UniqdAlertSummary {
                        ts: DateTime::from_timestamp_millis(datum.timestamp_ms as i64).unwrap(),
                        client,
                        total,
                    }),
                    total as f64,
                )
            } else {
                (None, total as f64)
            }
        })
    }
}

impl<C: TimeToIdleCache<Uniqd>> DetectionMethod for UniqdMethod<C> {
    type TAlert = UniqdAlertSummary;

    fn reset_interval(&self) -> u32 {
        self.detection_window as u32
    }

    fn process_single(
        &mut self,
        datum: Datum<'_>,
    ) -> color_eyre::Result<(Option<Self::TAlert>, f64)> {
        let now = datum.timestamp_ms;
        
        if let Some(uniqd) = self.clients.get_mut(datum.client, now) {
            Self::process(
                uniqd,
                datum,
                self.threshold,
                self.detection_window,
                self.allowlist.as_ref(),
                self.trust_rdns,
            )
        } else {
            // TODO: limit the number of clients
            let mut bfcms = Uniqd::new(datum.timestamp_ms)?;
            let client = datum.client;
            let r = Self::process(
                &mut bfcms,
                datum,
                self.threshold,
                self.detection_window,
                self.allowlist.as_ref(),
                self.trust_rdns,
            );
            self.clients.insert(client, bfcms, now);
            r
        }
    }
}
