use std::{
    borrow::Cow,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use color_eyre::eyre::ContextCompat;
use hashbrown::HashSet;
use unicase::UniCase;

use super::AllowList;

#[derive(Debug, Clone)]
pub struct PlainAllowList {
    inner: hashbrown::HashSet<UniCase<Cow<'static, str>>>,
    mode: AllowMode,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AllowMode {
    Domain,
    Suffix,
    Both,
}

impl PlainAllowList {
    pub fn empty() -> Self {
        Self {
            inner: Default::default(),
            mode: AllowMode::Both,
        }
    }

    pub fn load(path: impl AsRef<Path>, mode: AllowMode) -> color_eyre::Result<Self> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);
        let mut inner = HashSet::new();
        for line in buf_reader.lines() {
            let line = line?;
            let domain = line.trim();
            inner.insert(UniCase::ascii(Cow::Owned(domain.to_string())));
        }
        Ok(Self { inner, mode })
    }

    pub fn load_tranco_csv(csv_path: impl AsRef<Path>) -> color_eyre::Result<Self> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(csv_path)?;
        let mut inner = HashSet::new();
        for record in reader.records() {
            let record = record?;
            let domain = record.iter().nth(1).wrap_err("Not enough columns!")?;
            inner.insert(UniCase::ascii(Cow::Owned(domain.to_string())));
        }
        Ok(Self {
            inner,
            mode: AllowMode::Both,
        })
    }

    pub fn add(&mut self, domain: &str) {
        self.inner
            .insert(UniCase::ascii(Cow::Owned(domain.to_string())));
    }
}

impl AllowList for PlainAllowList {
    fn contains(&self, domain: &str, suffix: &str) -> bool {
        if self.mode == AllowMode::Domain {
            self.inner.contains(&UniCase::ascii(Cow::Borrowed(domain)))
        } else if self.mode == AllowMode::Suffix {
            self.inner.contains(&UniCase::ascii(Cow::Borrowed(suffix)))
        } else {
            self.inner.contains(&UniCase::ascii(Cow::Borrowed(domain)))
                || self.inner.contains(&UniCase::ascii(Cow::Borrowed(suffix)))
        }
    }
}
