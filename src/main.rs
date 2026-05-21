use std::{
    net::IpAddr,
    time::{Duration, Instant},
};

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::{bail, eyre};
use dns_exf_detect::domain::StaticDomainParser;
use dns_exf_detect::method::{AlertSummary, DetectionMethod, IbhhMethodClass};
use dns_exf_detect::{
    allowlist::{AllowList, AllowMode, HomogeneousAllowLists, PlainAllowList},
    dataset, method,
};
use dns_exf_detect::{cache::SimulatedTimeToIdleCache, method::uniqd::Uniqd};
use dns_exf_detect::{
    dataset::{Dataset, DatasetIter, Datum, DatumParser},
    threshold_tuning,
};
use hashbrown::{HashMap, HashSet};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::Itertools;

use dns_exf_detect::{
    dataset::ConcreteDataset,
    method::{UniqdMethod, bfcms::Bfcms},
};

mod cli;

#[allow(clippy::too_many_arguments)]
pub fn domain_detection_run<T: DetectionMethod>(
    method: &mut T,
    dataset: &impl Dataset,
    iter: &mut DatasetIter,
    counter: &mut u64,
    bad_domain_cnt: &mut u64,
    alert_cnt: &mut u64,
    domain_parser: &StaticDomainParser,
    pgbar: ProgressBar,
    mut alert_cb: impl FnMut(&T::TAlert),
    bench: bool,
    tune: &[f64],
) -> Result<HashSet<String>, color_eyre::eyre::Error> {
    let mut alerted_domains = HashSet::new();
    let mut domain_values = HashMap::<String, _>::new();
    let mut all_clients = HashSet::new();
    for record in iter.progress_with(pgbar) {
        *counter += 1;
        let record = record?;
        let datum: Result<Datum, _> = domain_parser.parse(&record);
        let datum = match datum {
            Ok(v) => v,
            Err(dataset::Error::BadDomain) => {
                *bad_domain_cnt += 1;
                continue;
            }
            Err(e) => return Err(eyre!(e)),
        };
        if !bench {
            all_clients.insert(datum.client);
        }
        let domain = datum.domain;
        let (alert, value) = method.process_single(datum)?;
        if let Some(alert) = alert
            && !bench
        {
            let (domain, _) = alert.domains().next().unwrap();
            if alerted_domains.contains(domain) {
                // SKIP
            } else {
                *alert_cnt += 1;
                alerted_domains.insert(domain.to_string());
                alert_cb(&alert);
            }
        }
        if !tune.is_empty() {
            let entry = domain_values.entry_ref(domain).or_default();
            if *entry < value {
                *entry = value;
            }
        }
    }

    for &acceptable_fpr in tune {
        threshold_tuning(
            &domain_values,
            acceptable_fpr,
            method.reset_interval() as f64,
        );
        client_oracle_threshold_tuning(
            dataset,
            &domain_values,
            acceptable_fpr,
            method.reset_interval() as f64,
        );
    }

    Ok(alerted_domains)
}

fn main() -> color_eyre::Result<(), color_eyre::Report> {
    color_eyre::install()?;
    let mut cli = Cli::parse();
    let is_peacetime = matches!(cli.command, cli::Command::Peacetime {});
    let is_bench = matches!(cli.command, cli::Command::Bench {});
    let tune = match cli.command {
        cli::Command::Tune { ref acceptable_fpr } => acceptable_fpr.clone(),
        _ => vec![],
    };
    if is_bench {
        // We do not want to bench the IO, so be quiet for output.
        cli.quiet = true;
    }
    let suffixlists = ["psl/public_suffix_list.dat", "psl/old.dat", "psl/cdn.dat"];
    let domain_parser = StaticDomainParser::new(suffixlists.into_iter())?;
    let dataset = dataset::load(&cli.dataset)?;
    let mut val_iter = dataset.iter_val()?;
    let mut train_iter = dataset.iter_train()?;
    let mut peacetime_iter = dataset.iter_peacetime()?;
    let mut counter = 0;
    let mut alert_cnt = 0;
    let mut bad_domain_cnt = 0;
    let popularity_allowlist = if cli.skip_popularity_allowlist {
        PlainAllowList::empty()
    } else {
        PlainAllowList::load_tranco_csv("allowlist/popularity/top-1m.csv")?
    };
    let manual_allowlist = PlainAllowList::load("allowlist/manual.list", AllowMode::Domain)?;
    let internal_allowlist = if cli.skip_internal_allowlist {
        PlainAllowList::empty()
    } else {
        PlainAllowList::load("allowlist/local/local-tld.list", AllowMode::Suffix)?
    };
    let method = cli.method.as_str();
    let peacetime_allowlist = if !cli.skip_peacetime_allowlist {
        if method.starts_with("ibhh") && !is_peacetime {
            match dataset.load_peace_time_domain_allowlist() {
                Err(e) => {
                    eprintln!("Failed to load peace time domain allowlist: {e}");
                    eprintln!(
                        "You can use the train command to generate the peace time domain allowlist"
                    );
                    std::thread::sleep(Duration::from_secs(10));
                    PlainAllowList::empty()
                }
                Ok(l) => l,
            }
        } else if (method.starts_with("bfcms") || method.starts_with("uniqd")) && !is_peacetime {
            match dataset.load_peace_time_bfcms_allowlist() {
                Err(e) => {
                    eprintln!("Failed to load peace bfcms domain allowlist: {e}");
                    eprintln!(
                        "You can use the train command to generate peace time alerts and create an allowlist by yourself"
                    );
                    std::thread::sleep(Duration::from_secs(10));
                    PlainAllowList::empty()
                }
                Ok(l) => l,
            }
        } else {
            PlainAllowList::empty()
        }
    } else {
        PlainAllowList::empty()
    };
    let allowlist = HomogeneousAllowLists::new([
        internal_allowlist,
        popularity_allowlist,
        manual_allowlist,
        peacetime_allowlist,
    ]);
    let len = if !tune.is_empty() {
        dataset.train_set_len()
    } else if is_peacetime {
        dataset.peacetime_set_len()
    } else {
        dataset.val_set_len()
    };
    let progress_bar = ProgressBar::new(len as u64).with_style(ProgressStyle::with_template(
        "Elapsed: {elapsed} {wide_bar} Speed: {per_sec}",
    )?);

    let iter = if !tune.is_empty() {
        &mut train_iter as &mut DatasetIter
    } else if is_peacetime {
        &mut peacetime_iter as &mut DatasetIter
    } else {
        &mut val_iter as &mut DatasetIter
    };
    let allowlist = if is_peacetime {
        // Original ibHH does not use allowlists during peacetime
        Box::new(PlainAllowList::empty()) as Box<dyn AllowList>
    } else {
        Box::new(allowlist) as Box<dyn AllowList>
    };

    // Convert per second threshold to real threshold for the reset interval.
    println!("Threshold: {}", cli.threshold);
    cli.threshold *= cli.reset_interval as f64 / 1000.;
    println!("RawThreshold: {}", cli.threshold);

    let elapsed = eval(
        cli,
        allowlist,
        iter,
        &mut counter,
        &mut bad_domain_cnt,
        &mut alert_cnt,
        &domain_parser,
        dataset,
        progress_bar,
        is_bench,
        tune,
    )?;
    eprintln!(
        "processed {} queries in {} seconds, QPS: {}",
        counter,
        elapsed.as_secs_f64(),
        counter as f64 / elapsed.as_secs_f64()
    );
    eprintln!("Emitted {} alerts", alert_cnt);
    eprintln!("Skipped {} malformed domains", bad_domain_cnt);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn eval(
    cli: Cli,
    allowlist: impl AllowList + 'static,
    iter: &mut DatasetIter,
    counter: &mut u64,
    bad_domain_cnt: &mut u64,
    alert_cnt: &mut u64,
    domain_parser: &StaticDomainParser,
    dataset: ConcreteDataset,
    progress_bar: ProgressBar,
    is_bench: bool,
    tune: Vec<f64>,
) -> color_eyre::Result<Duration, color_eyre::Report> {
    let is_peacetime = matches!(cli.command, cli::Command::Peacetime { .. });
    let start;
    let is_tune = !tune.is_empty();
    let mut client_values = HashMap::new();
    match cli.method.as_str() {
        m @ "ibhh" => {
            let mut method = match m {
                "ibhh" => IbhhMethodClass::Ibhh(method::IbhhMethod::new(
                    cli.threshold,
                    cli.reset_interval,
                    cli.ibhh_k,
                    allowlist,
                )),
                _ => unreachable!(),
            };
            start = Instant::now();
            let alerted_domains = domain_detection_run(
                &mut method,
                &dataset,
                iter,
                counter,
                bad_domain_cnt,
                alert_cnt,
                domain_parser,
                progress_bar,
                |alert| {
                    if !cli.quiet {
                        println!("{alert}")
                    }
                },
                is_bench,
                &tune,
            )?;
            if is_peacetime {
                // Write peace time allow list
                dataset.generate_peace_time_domain_allowlist(&alerted_domains, true)?;
                eprintln!("Wrote peace time domain allowlist.")
            } else if !is_tune && !is_bench {
                let res = dataset.evaluate_domains(&alerted_domains);
                res.report();
                println!("--- Client evaluation report via client oracle ---");
                let client_res = dataset.evaluate_clients_via_oracle(&alerted_domains);
                client_res.report();
            }
        }
        "bfcms" => {
            let mut method = method::BfcmsMethod::<SimulatedTimeToIdleCache<Bfcms>>::new(
                cli.threshold as u32,
                cli.reset_interval,
                allowlist,
                cli.trust_rdns,
                !cli.ablation_rdns_special,
            );
            start = Instant::now();
            let mut all_clients = HashSet::new();
            let mut alerted_clients: HashMap<IpAddr, u32> = HashMap::new();
            let mut related_domains: HashSet<String> = HashSet::new();
            for record in iter.progress_with(progress_bar) {
                *counter += 1;
                let record = record?;
                let datum: Result<Datum, _> = domain_parser.parse(&record);
                let datum = match datum {
                    Ok(v) => v,
                    Err(dataset::Error::BadDomain) => {
                        *bad_domain_cnt += 1;
                        continue;
                    }
                    Err(e) => return Err(eyre!(e)),
                };
                if !is_bench {
                    all_clients.insert(datum.client);
                }
                let client = datum.client;
                let (alert, value) = method.process_single(datum)?;
                if let Some(alert) = alert
                    && !is_bench
                {
                    let (client, cnt) = alert.clients().next().unwrap();
                    if is_peacetime {
                        for (domain, _) in alert.domains() {
                            related_domains.insert(domain.to_owned());
                        }
                    }
                    if let Some(prev_cnt) = alerted_clients.get_mut(&client) {
                        *prev_cnt = (*prev_cnt).max(cnt);
                        // alert_cnt += 1;
                        if !cli.quiet {
                            println!("Throttled alert! {alert}");
                        }
                    } else {
                        alerted_clients.insert(client, cnt);

                        *alert_cnt += 1;
                        if !cli.quiet {
                            println!("Alert! {alert}");
                        }
                    }
                }
                if !tune.is_empty() {
                    let entry = client_values.entry(client).or_default();
                    if *entry < value {
                        *entry = value;
                    }
                }
            }

            for acceptable_fpr in tune {
                threshold_tuning(
                    &client_values,
                    acceptable_fpr,
                    method.reset_interval() as f64,
                );
            }

            let detection = alerted_clients.keys().cloned().collect();
            if is_peacetime {
                dataset.generate_peace_time_domain_allowlist(&related_domains, false)?;
                eprintln!("Wrote peace time domain allowlist.")
            } else if !is_tune && !is_bench {
                let res = dataset.evaluate_clients(&detection);
                res.report();
            }
        }
        "uniqd" => {
            let mut method = UniqdMethod::<SimulatedTimeToIdleCache<Uniqd>>::new(
                cli.threshold as u32,
                cli.reset_interval,
                allowlist,
                cli.trust_rdns,
            );
            start = Instant::now();
            let mut all_clients = HashSet::new();
            let mut alerted_clients: HashMap<IpAddr, u32> = HashMap::new();
            for record in iter.progress_with(progress_bar) {
                *counter += 1;
                let record = record?;
                let datum: Result<Datum, _> = domain_parser.parse(&record);
                let datum = match datum {
                    Ok(v) => v,
                    Err(dataset::Error::BadDomain) => {
                        *bad_domain_cnt += 1;
                        continue;
                    }
                    Err(e) => return Err(eyre!(e)),
                };
                if !is_bench {
                    all_clients.insert(datum.client);
                }
                let client = datum.client;
                let (alert, value) = method.process_single(datum)?;
                if let Some(alert) = alert
                    && !is_bench
                {
                    let (client, cnt) = alert.clients().next().unwrap();
                    if let Some(prev_cnt) = alerted_clients.get_mut(&client) {
                        *prev_cnt = (*prev_cnt).max(cnt);
                        // alert_cnt += 1;
                        if !cli.quiet {
                            println!("Throttled alert! {alert}");
                        }
                    } else {
                        alerted_clients.insert(client, cnt);
                        *alert_cnt += 1;
                        if !cli.quiet {
                            println!("Alert! {alert}");
                        }
                    }
                }
                if !tune.is_empty() {
                    let entry = client_values.entry(client).or_default();
                    if *entry < value {
                        *entry = value;
                    }
                }
            }
            let detection = alerted_clients.keys().cloned().collect();

            for acceptable_fpr in tune {
                threshold_tuning(
                    &client_values,
                    acceptable_fpr,
                    method.reset_interval() as f64,
                )
            }

            if !is_tune && !is_bench {
                let res = dataset.evaluate_clients(&detection);
                res.report();
            }
        }
        other => bail!("{other} method not found"),
    }
    let elapsed = start.elapsed();
    Ok(elapsed)
}

fn client_oracle_threshold_tuning(
    dataset: &impl Dataset,
    values: &HashMap<String, f64>,
    acceptable_fpr: f64,
    reset_interval: f64,
) {
    let acceptable_fp_count = (acceptable_fpr * values.len() as f64).ceil() as usize;
    let values = values
        .into_iter()
        .sorted_by(|(_, a), (_, b)| f64::total_cmp(b, a));
    let mut alerted_clients = HashSet::new();
    let mut current_threshold = 0.;
    for (domain, value) in values {
        let clients = dataset.tuning_client_oracle(&domain.to_ascii_lowercase());
        alerted_clients |= clients;
        if alerted_clients.len() > acceptable_fp_count {
            // Time to stop
            current_threshold = value + 1.;
            break;
        }
    }
    let raw_threshold = current_threshold;
    println!(
        "ClientTunedRawThreshold for {}: {}",
        acceptable_fpr, raw_threshold
    );
    println!(
        "ClientTunedThreshold {}: {}",
        acceptable_fpr,
        (raw_threshold / (reset_interval / 1000.))
    );
}
