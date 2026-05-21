use std::{
    borrow::Cow,
    io::{BufWriter, Write},
    net::IpAddr,
    path::Path,
};

use dns_exf_detect::{
    dataset::{
        ours::{OurReader, OurZstdReader},
        ziza::CsvDatasetReader,
    },
    domain::{DomainParser, StaticDomainParser},
};
use hashbrown::{HashMap, HashSet};

fn main() {
    let format = std::env::args().nth(2);
    let format = match format.as_deref() {
        None | Some("ours") => "ours",
        Some("ziza") => "ziza",
        _ => panic!("unrecognized format {}", format.unwrap()),
    };
    let path = std::env::args().nth(1).expect("Missing path to data");
    let file = std::fs::File::open(&path).expect("failed to open your data");
    let iter = if format == "ours" {
        OurZstdReader
            .iter(&file)
            .expect("Failed to create data iter")
    } else {
        CsvDatasetReader::iter(file)
    };
    let prefix = std::env::args().nth(3).unwrap_or_default();

    let suffixlists = ["psl/public_suffix_list.dat", "psl/old.dat", "psl/cdn.dat"];
    let domain_parser =
        StaticDomainParser::new(suffixlists.into_iter()).expect("Failed to create domain parser");
    let mut oracle = HashMap::<String, HashSet<IpAddr>>::new();
    for datum in iter {
        if let Err(e) = &datum {
            eprintln!("corrupted datum: {datum:?}: {e}");
        }
        let datum = datum.unwrap();
        let domain = datum.full;
        let parsed = domain_parser.parse_domain(&domain);
        let parsed = match parsed {
            Ok(parsed) => parsed,
            Err(e) => {
                eprintln!("Error while parsing {domain}: {e:?}");
                continue;
            }
        };
        let suffix = parsed.suffix().expect("No suffix?");
        let domain = parsed.root().unwrap_or(suffix);
        let clients = oracle
            .entry(domain.to_string().to_ascii_lowercase())
            .or_default();
        clients.insert(datum.client);
    }
    let out = std::fs::File::create(
        Path::new(&path).parent().unwrap().join(
            if !prefix.is_empty() {
                Cow::Owned(prefix + ".client.oracle")
            } else {
                Cow::Borrowed("client.oracle")
            }
            .as_ref(),
        ),
    )
    .unwrap();
    if std::env::var("SAVE_UNIQUE_DOMAINS").is_ok() {
        let mut writer = BufWriter::new(
            std::fs::File::create(
                Path::new(&path)
                    .parent()
                    .unwrap()
                    .join("val_all_domains.list"),
            )
            .expect("failed to open domains output file"),
        );
        for unique_domain in oracle.keys() {
            writeln!(writer, "{unique_domain}").unwrap();
        }
        drop(writer);
    }
    let writer = BufWriter::new(out);
    serde_json::to_writer_pretty(writer, &oracle).expect("Failed to save oracle");
}
