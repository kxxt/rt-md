use dns_exf_detect::{
    dataset::{
        ours::{OurReader, OurZstdReader},
        ziza::CsvDatasetReader,
    },
    domain::{DomainParser, StaticDomainParser},
};
use hashbrown::HashSet;

fn main() {
    let format = std::env::args().nth(1);
    let format = match format.as_deref() {
        None | Some("ours") => "ours",
        Some("ziza") => "ziza",
        _ => panic!("unrecognized format {}", format.unwrap()),
    };
    let mut hosts = HashSet::new();
    let mut domains = HashSet::new();
    let mut queries = 0;
    for path in std::env::args().skip(2) {
        let file = std::fs::File::open(path).expect("failed to open your data");
        let iter = if format == "ours" {
            OurZstdReader
                .iter(&file)
                .expect("Failed to create data iter")
        } else {
            CsvDatasetReader::iter(file)
        };

        let suffixlists = ["psl/public_suffix_list.dat", "psl/old.dat", "psl/cdn.dat"];
        let domain_parser = StaticDomainParser::new(suffixlists.into_iter())
            .expect("Failed to create domain parser");
        for datum in iter {
            queries += 1;
            let datum = datum.unwrap();
            hosts.insert(datum.client);
            let domain = datum.full;
            if domain.contains("..") || domain.contains("\\") || domain.is_empty() {
                continue;
            }
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
            domains.insert(domain.to_owned());
        }
    }
    println!("#Domains: {}", domains.len());
    println!("#Hosts: {}", hosts.len());
    println!("#Queries: {}", queries);
}
