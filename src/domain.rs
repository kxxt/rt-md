use std::{io::Read, sync::Arc};

use addr::{dns::Name, parser::DnsName};
use arc_swap::ArcSwap;
use snafu::OptionExt;

use crate::dataset::{BadDomainSnafu, Datum, DatumParser};

pub trait DomainParser {
    fn parse_domain<'a>(&self, input: &'a str) -> addr::Result<'a, Name<'a>>;
}

#[derive(Debug)]
#[repr(transparent)]
pub struct DynamicDomainParser {
    list: ArcSwap<publicsuffix::List>,
}

impl DynamicDomainParser {
    pub fn insert(&mut self, suffix: &str) -> bool {
        let mut list = self.list.load().as_ref().to_owned();
        if list.append(suffix, publicsuffix::Type::Private).is_ok() {
            self.list.store(Arc::new(list));
            true
        } else {
            false
        }
    }
}

impl DynamicDomainParser {
    pub fn new<'a>(lists: impl Iterator<Item = &'a str>) -> color_eyre::Result<Self> {
        let mut text = String::new();
        for path in lists {
            let mut f = std::fs::File::open(path)?;
            f.read_to_string(&mut text)?;
            text.push('\n');
        }
        let list = publicsuffix::List::from_bytes(text.as_bytes())?;
        Ok(Self {
            list: ArcSwap::from_pointee(list),
        })
    }
}

#[derive(Debug, Clone)]
pub struct StaticDomainParser {
    list: publicsuffix::List,
}

impl StaticDomainParser {
    pub fn new<'a>(lists: impl Iterator<Item = &'a str>) -> color_eyre::Result<Self> {
        let mut text = String::new();
        for path in lists {
            let mut f = std::fs::File::open(path)?;
            f.read_to_string(&mut text)?;
            text.push('\n');
        }
        let list = publicsuffix::List::from_bytes(text.as_bytes())?;
        Ok(Self { list })
    }
}

impl DomainParser for StaticDomainParser {
    fn parse_domain<'a>(&self, input: &'a str) -> addr::Result<'a, Name<'a>> {
        self.list.parse_dns_name(input)
    }
}

impl DatumParser for StaticDomainParser {
    fn parse<'a>(
        &self,
        datum: &'a crate::dataset::IntermediateDatum,
    ) -> Result<Datum<'a>, super::dataset::Error> {
        let parsed = self
            .parse_domain(&datum.full)
            .ok()
            .context(BadDomainSnafu)?;
        let suffix = parsed.suffix().context(BadDomainSnafu)?;
        let domain = parsed.root().unwrap_or(suffix);
        let subdomain = parsed.prefix().unwrap_or_default();
        Ok(Datum {
            timestamp_ms: datum.timestamp_ms,
            full: &datum.full,
            domain,
            subdomain,
            suffix,
            client: datum.client,
        })
    }
}

impl DomainParser for DynamicDomainParser {
    fn parse_domain<'a>(&self, input: &'a str) -> addr::Result<'a, Name<'a>> {
        self.list.load().parse_dns_name(input)
    }
}

impl DatumParser for DynamicDomainParser {
    fn parse<'a>(
        &self,
        datum: &'a crate::dataset::IntermediateDatum,
    ) -> Result<Datum<'a>, super::dataset::Error> {
        let parsed = self
            .parse_domain(&datum.full)
            .ok()
            .context(BadDomainSnafu)?;
        let suffix = parsed.suffix().context(BadDomainSnafu)?;
        let domain = parsed.root().unwrap_or(suffix);
        let subdomain = parsed.prefix().unwrap_or_default();
        Ok(Datum {
            timestamp_ms: datum.timestamp_ms,
            full: &datum.full,
            domain,
            subdomain,
            suffix,
            client: datum.client,
        })
    }
}
