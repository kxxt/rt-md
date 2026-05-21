use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::IpAddr,
    path::PathBuf,
};

use color_eyre::eyre::{self, OptionExt};
use hashbrown::{HashMap, HashSet};
use nix::unistd::dup;
use serde::{Deserialize, Serialize};

use crate::allowlist::{AllowMode, PlainAllowList};

use super::{
    Dataset, IntermediateDatum,
    evaluation::{ClientEvaluationResult, DomainEvaluationResult, EvaluationResult},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZizaDatasetConfig {
    validation_set: String,
    validation_set_size: usize,
    peacetime_set_size: usize,
    training_set_size: usize,
    validation_set_tp_clients: String,
    validation_set_tp_domains: String,
    validation_set_all_clients: String,
    validation_set_all_domains: String,
    peacetime_set: String,
    training_set: String,
    peacetime_domain_allowlist: String,
    peacetime_bfcms_allowlist: String,
    client_oracle: String,
    tuning_client_oracle: String,
}
pub struct ZizaDataset {
    client_oracle: HashMap<String, HashSet<IpAddr>>,
    tuning_client_oracle: HashMap<String, HashSet<IpAddr>>,
    val_set: File,
    train_set: File,
    peacetime_set: File,
    val_set_size: usize,
    train_set_size: usize,
    peacetime_set_size: usize,
    val_all_clients: HashSet<IpAddr>,
    val_all_domains: HashSet<String>,
    val_tp_clients: HashSet<IpAddr>,
    val_tp_domains: HashSet<String>,
    peacetime_domain_allowlist: PathBuf,
    peacetime_bfcms_allowlist: PathBuf,
    // training_set: CsvDataset,
}

pub struct CsvDatasetReader;

impl CsvDatasetReader {
    pub fn iter(file: File) -> Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>> {
        Box::new(BufReader::new(file).lines().skip(1).map(|line| {
            line.map_err(eyre::ErrReport::from).and_then(|line| {
                let mut iter = line.split(',').take(3);
                Ok(IntermediateDatum {
                    timestamp_ms: iter.next().ok_or_eyre("missing timestamp")?.parse()?,
                    full: iter.next().ok_or_eyre("missing request")?.to_string(),
                    client: iter.next().ok_or_eyre("missing client")?.parse()?,
                })
            })
        }))
    }
}

impl ZizaDataset {
    fn iter(
        &self,
        file: &File,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>> {
        Ok(CsvDatasetReader::iter(File::from(dup(file)?)))
    }

    pub fn load(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        let config: ZizaDatasetConfig = toml::from_str(&std::fs::read_to_string(path.as_ref())?)?;
        let path = path
            .as_ref()
            .parent()
            .ok_or_eyre("No parent found for config file path")?;
        let client_oracle = serde_json::from_reader(BufReader::new(File::open(
            path.join(&config.client_oracle),
        )?))?;
        let tuning_client_oracle = serde_json::from_reader(BufReader::new(File::open(
            path.join(&config.tuning_client_oracle),
        )?))?;
        let val_file = File::open(path.join(&config.validation_set))?;
        let training_file = File::open(path.join(&config.training_set))?;
        let peacetime_file = File::open(path.join(&config.peacetime_set))?;
        let val_all_clients =
            BufReader::new(File::open(path.join(&config.validation_set_all_clients))?)
                .lines()
                .map(|v| v.and_then(|v| v.trim().parse::<IpAddr>().map_err(io::Error::other)))
                .collect::<Result<_, _>>()?;
        let val_all_domains =
            BufReader::new(File::open(path.join(&config.validation_set_all_domains))?)
                .lines()
                .map(|v| {
                    v.map(|mut s| {
                        s.truncate(s.trim_end().len());
                        s
                    })
                })
                .collect::<Result<_, _>>()?;
        let val_tp_domains =
            BufReader::new(File::open(path.join(&config.validation_set_tp_domains))?)
                .lines()
                .map(|v| {
                    v.map(|mut s| {
                        s.truncate(s.trim_end().len());
                        s
                    })
                })
                .collect::<Result<_, _>>()?;
        let val_tp_clients =
            BufReader::new(File::open(path.join(&config.validation_set_tp_clients))?)
                .lines()
                .map(|v| v.and_then(|v| v.trim().parse::<IpAddr>().map_err(io::Error::other)))
                .collect::<Result<_, _>>()?;
        Ok(Self {
            client_oracle,
            tuning_client_oracle,
            val_set: val_file,
            peacetime_set: peacetime_file,
            train_set: training_file,
            val_set_size: config.validation_set_size,
            peacetime_set_size: config.peacetime_set_size,
            train_set_size: config.training_set_size,
            val_all_clients,
            val_all_domains,
            val_tp_clients,
            val_tp_domains,
            peacetime_domain_allowlist: path.join(config.peacetime_domain_allowlist),
            peacetime_bfcms_allowlist: path.join(config.peacetime_bfcms_allowlist),
        })
    }
}

impl Dataset for ZizaDataset {
    fn evaluate_clients(
        &self,
        detection: &HashSet<IpAddr>,
    ) -> Box<dyn EvaluationResult<Item = IpAddr> + 'static> {
        let tp_: Vec<_> = self
            .val_tp_clients
            .intersection(detection)
            .cloned()
            .collect();
        let fn_: Vec<_> = self.val_tp_clients.difference(detection).cloned().collect();
        let fp_: Vec<_> = detection
            .difference(&self.val_tp_clients)
            .cloned()
            .collect();
        let total = self.val_all_clients.len() as u64;
        Box::new(ClientEvaluationResult {
            tp_,
            fn_,
            fp_,
            total,
        })
    }

    fn evaluate_domains(
        &self,
        detection: &HashSet<String>,
    ) -> Box<dyn EvaluationResult<Item = std::string::String> + 'static> {
        let tp_: Vec<_> = self
            .val_tp_domains
            .intersection(detection)
            .cloned()
            .collect();
        let fn_: Vec<_> = self.val_tp_domains.difference(detection).cloned().collect();
        let fp_: Vec<_> = detection
            .difference(&self.val_tp_domains)
            .cloned()
            .collect();
        let total = self.val_all_domains.len() as u64;
        Box::new(DomainEvaluationResult {
            tp_,
            fn_,
            fp_,
            total,
        })
    }

    fn is_online(&self) -> bool {
        false
    }

    fn val_set_len(&self) -> usize {
        self.val_set_size
    }

    fn train_set_len(&self) -> usize {
        self.train_set_size
    }

    fn peacetime_set_len(&self) -> usize {
        self.peacetime_set_size
    }

    fn iter_train(
        &self,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>> {
        self.iter(&self.train_set)
    }

    fn iter_val(
        &self,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>> {
        self.iter(&self.val_set)
    }

    fn iter_peacetime(
        &self,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>> {
        self.iter(&self.peacetime_set)
    }

    fn generate_peace_time_domain_allowlist(
        &self,
        detection: &HashSet<String>,
        ibhh: bool,
    ) -> color_eyre::Result<()> {
        let mut w = BufWriter::new(File::create(if ibhh {
            &self.peacetime_domain_allowlist
        } else {
            &self.peacetime_bfcms_allowlist
        })?);
        for line in detection {
            let lower = line.to_ascii_lowercase();
            if !self.val_tp_domains.contains(&lower) {
                writeln!(w, "{}", lower)?;
            }
        }
        w.flush()?;
        drop(w);
        Ok(())
    }

    fn load_peace_time_domain_allowlist(
        &self,
    ) -> color_eyre::Result<crate::allowlist::PlainAllowList> {
        PlainAllowList::load(&self.peacetime_domain_allowlist, AllowMode::Domain)
    }

    fn load_peace_time_bfcms_allowlist(
        &self,
    ) -> color_eyre::Result<crate::allowlist::PlainAllowList> {
        PlainAllowList::load(&self.peacetime_bfcms_allowlist, AllowMode::Domain)
    }

    fn evaluate_clients_via_oracle(
        &self,
        detection: &HashSet<String>,
    ) -> Box<dyn EvaluationResult<Item = IpAddr>> {
        let mut tp_: HashSet<IpAddr> = HashSet::new();
        let mut fp_: HashSet<IpAddr> = HashSet::new();
        let mut fn_: HashSet<IpAddr> = HashSet::new();
        let fn_domains = self.val_tp_domains.difference(detection);
        for domain in detection {
            if self.val_tp_domains.contains(domain) {
                tp_.extend(self.client_oracle.get(domain).unwrap().iter())
            } else {
                fp_.extend(self.client_oracle.get(domain).unwrap().iter())
            }
        }
        for domain in fn_domains {
            fn_.extend(self.client_oracle.get(domain).unwrap().iter())
        }
        let eval = ClientEvaluationResult {
            tp_: tp_.into_iter().collect(),
            fn_: fn_.into_iter().collect(),
            fp_: fp_.into_iter().collect(),
            total: self.val_all_clients.len() as u64,
        };
        Box::new(eval)
    }

    fn tuning_client_oracle(&self, domain: &str) -> &HashSet<IpAddr> {
        if let Some(clients) = &self.tuning_client_oracle.get(domain) {
            clients
        } else {
            panic!("No entry found for domain {domain:?}");
        }
    }
}
