use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::IpAddr,
    path::PathBuf,
};

use color_eyre::{
    Section,
    eyre::{self, OptionExt},
};
use hashbrown::{HashMap, HashSet};
use nix::unistd::dup;
use parquet::{file::reader::SerializedFileReader, record::Field};
use serde::{Deserialize, Serialize};

use crate::allowlist::{AllowMode, PlainAllowList};

use super::{
    Dataset, IntermediateDatum,
    evaluation::{ClientEvaluationResult, DomainEvaluationResult, EvaluationResult},
};

pub trait OurReader: Default {
    fn iter(
        &self,
        file: &File,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>>;
}

#[derive(Debug, Default)]
pub struct OurParquetReader;
#[derive(Debug, Default)]
pub struct OurZstdReader;

impl OurReader for OurParquetReader {
    fn iter(
        &self,
        file: &File,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>> {
        let row_iter = SerializedFileReader::try_from(File::from(dup(file)?))?
            .into_iter()
            .with_batch_size(5000);

        // Read data
        Ok(Box::new(row_iter.map(|r| {
            r.map_err(eyre::ErrReport::from).and_then(|row| {
                let mut c = row.into_columns().into_iter();
                let (_, Field::TimestampMillis(date)) = c.next().unwrap() else {
                    panic!("invalid data type for date")
                };
                let _resolver = c.next().unwrap();
                let (_, Field::Str(client)) = c.next().unwrap() else {
                    panic!("invalid data type for client")
                };
                let (_, Field::Str(domain)) = c.next().unwrap() else {
                    panic!("invalid data type for client")
                };
                Ok(IntermediateDatum {
                    full: domain,
                    client: client.parse()?,
                    timestamp_ms: date as u64,
                })
            })
        })))
    }
}

impl OurReader for OurZstdReader {
    fn iter(
        &self,
        file: &File,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>> {
        let file = File::from(dup(file)?);
        let decoder = zstd::Decoder::new(file)?;
        let reader = BufReader::new(decoder);
        Ok(Box::new(reader.lines().skip(1).map(|line| {
            line.map_err(eyre::ErrReport::from).and_then(|line| {
                let mut iter = line.split(',');
                Ok(IntermediateDatum {
                    timestamp_ms: iter
                        .next()
                        .ok_or_eyre("missing timestamp")?
                        .trim_ascii()
                        .parse()?,
                    client: iter
                        .next()
                        .ok_or_eyre("missing client")?
                        .trim_ascii()
                        .parse()?,
                    full: iter
                        .next()
                        .ok_or_eyre("missing request")?
                        .trim_ascii()
                        .to_string(),
                })
            })
        })))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OurDatasetConfig {
    validation_set: String,
    validation_set_size: usize,
    validation_set_tp_clients: String,
    validation_set_tp_domains: String,
    validation_set_all_clients: String,
    validation_set_all_domains: String,
    training_set: String,
    training_set_size: usize,
    peacetime_set: String,
    peacetime_set_size: usize,
    peacetime_domain_allowlist: String,
    peacetime_bfcms_allowlist: String,
    client_oracle: String,
    tuning_client_oracle: String,
}

pub struct OurDataset<Reader> {
    reader: Reader,
    val_set: File,
    peacetime_set: File,
    train_set: File,
    val_set_size: usize,
    train_set_size: usize,
    peacetime_set_size: usize,
    val_all_clients: HashSet<IpAddr>,
    val_all_domains: HashSet<String>,
    val_tp_clients: HashSet<IpAddr>,
    val_tp_domains: HashSet<String>,
    peacetime_domain_allowlist: PathBuf,
    peacetime_bfcms_allowlist: PathBuf,
    client_oracle: HashMap<String, HashSet<IpAddr>>,
    tuning_client_oracle: HashMap<String, HashSet<IpAddr>>,
    // training_set: CsvDataset,
}

impl<Reader: OurReader> OurDataset<Reader> {
    fn iter(
        &self,
        file: &File,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>> {
        self.reader.iter(file)
    }

    pub fn load(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        let config: OurDatasetConfig = toml::from_str(&std::fs::read_to_string(path.as_ref())?)?;
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
        let train_file = File::open(path.join(&config.training_set))?;
        let peacetime_file = File::open(path.join(&config.peacetime_set))?;
        let peacetime_set_size = config.validation_set_size;
        let val_set_size = config.validation_set_size;
        let train_set_size = config.training_set_size;
        let val_all_clients =
            BufReader::new(File::open(path.join(&config.validation_set_all_clients))?)
                .lines()
                .map(|v| {
                    v.map_err(color_eyre::eyre::ErrReport::from).and_then(|v| {
                        v.trim().parse::<IpAddr>().map_err(|e| {
                            color_eyre::eyre::ErrReport::from(e)
                                .with_note(|| format!("cannot parse {v}"))
                        })
                    })
                })
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
            reader: Reader::default(),
            val_set: val_file,
            train_set: train_file,
            peacetime_set: peacetime_file,
            val_set_size,
            train_set_size,
            peacetime_set_size,
            val_all_clients,
            val_all_domains,
            val_tp_clients,
            val_tp_domains,
            peacetime_domain_allowlist: path.join(config.peacetime_domain_allowlist),
            peacetime_bfcms_allowlist: path.join(config.peacetime_bfcms_allowlist),
        })
    }
}

impl<T: OurReader> Dataset for OurDataset<T> {
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
    fn iter_train(
        &self,
    ) -> color_eyre::Result<Box<dyn Iterator<Item = color_eyre::Result<IntermediateDatum>>>> {
        self.iter(&self.train_set)
    }

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
        &self.tuning_client_oracle[domain]
    }
}
