use std::{fmt::Debug, io, net::IpAddr, path::Path};

use enum_dispatch::enum_dispatch;
use evaluation::EvaluationResult;
use hashbrown::HashSet;
use snafu::Snafu;

use crate::{
    allowlist::PlainAllowList,
    dataset::{
        loader::DatasetConfig,
        ours::{OurDataset, OurZstdReader},
        ziza::ZizaDataset,
    },
};

pub mod evaluation;
pub mod loader;
pub mod ours;
pub mod ziza;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    OpenDataset {
        source: io::Error,
    },
    #[snafu(display("Datum does not provide enough data, missing {missing}"))]
    NotEnoughData {
        missing: &'static str,
    },
    BadDomain,
    #[snafu(whatever)]
    CorruptedDatum {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct IntermediateDatum {
    pub full: String,
    pub client: IpAddr,
    pub timestamp_ms: u64,
}

pub trait DatumParser {
    fn parse<'a>(&self, datum: &'a IntermediateDatum) -> Result<Datum<'a>, Error>;
}

pub fn load(config: impl AsRef<Path>) -> color_eyre::Result<ConcreteDataset> {
    let path = config.as_ref();
    let config: DatasetConfig = toml::from_str(&std::fs::read_to_string(path)?)?;
    Ok(match config {
        DatasetConfig::Ziza(_) => ConcreteDataset::Ziza(ZizaDataset::load(path)?),
        DatasetConfig::Ours(_) => ConcreteDataset::Ours(OurDataset::load(path)?),
    })
}

#[derive(Debug)]
pub struct Datum<'a> {
    pub timestamp_ms: u64,
    pub suffix: &'a str,
    pub full: &'a str,
    pub domain: &'a str,
    pub subdomain: &'a str,
    pub client: IpAddr,
}

pub type DatasetIter = dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>;

#[enum_dispatch]
pub trait Dataset {
    fn iter_peacetime(
        &self,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>>;
    fn iter_val(
        &self,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>>;
    fn iter_train(
        &self,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>>;
    fn val_set_len(&self) -> usize;
    fn train_set_len(&self) -> usize;
    fn peacetime_set_len(&self) -> usize;
    fn evaluate_clients(
        &self,
        detection: &HashSet<IpAddr>,
    ) -> Box<dyn EvaluationResult<Item = IpAddr>>;
    fn evaluate_domains(
        &self,
        detection: &HashSet<String>,
    ) -> Box<dyn EvaluationResult<Item = String>>;
    fn evaluate_clients_via_oracle(
        &self,
        detection: &HashSet<String>,
    ) -> Box<dyn EvaluationResult<Item = IpAddr>>;
    fn tuning_client_oracle(&self, domain: &str) -> &HashSet<IpAddr>;
    fn is_online(&self) -> bool;
    fn generate_peace_time_domain_allowlist(
        &self,
        detection: &HashSet<String>,
        ibhh: bool,
    ) -> color_eyre::Result<()>;
    fn load_peace_time_domain_allowlist(&self) -> color_eyre::Result<PlainAllowList>;
    fn load_peace_time_bfcms_allowlist(&self) -> color_eyre::Result<PlainAllowList>;
}

#[enum_dispatch(Dataset)]
pub enum ConcreteDataset {
    Ziza(ZizaDataset),
    Ours(OurDataset<OurZstdReader>),
}
