use dns_exf_detect::domain::{DomainParser, StaticDomainParser};
use hashbrown::HashSet;
use indicatif::{ProgressBar, ProgressStyle};
use parquet::{
    file::reader::{FileReader, SerializedFileReader},
    record::RowAccessor,
};

fn main() {
    let path = std::env::args().nth(1).expect("Missing path to data");
    let file = std::fs::File::open(path).expect("failed to open your data");
    let reader = SerializedFileReader::try_from(file).expect("failed to create parquet reader");
    let meta = reader.metadata();
    let num_rows: i64 = meta.row_groups().iter().map(|r| r.num_rows()).sum();
    let bar = ProgressBar::new(num_rows as u64).with_style(
        ProgressStyle::with_template("Elapsed: {elapsed} {wide_bar} Speed: {per_sec}").unwrap(),
    );

    let iter = reader.into_iter().with_batch_size(1_000_000);
    let suffixlists = ["psl/public_suffix_list.dat", "psl/old.dat", "psl/cdn.dat"];
    let domain_parser =
        StaticDomainParser::new(suffixlists.into_iter()).expect("Failed to create domain parser");
    let mut unique_domains = HashSet::new();

    for row in iter {
        let row = row.unwrap();
        let domain = row.get_string(3).unwrap();
        if domain.contains("..") || domain.contains("\\") || domain.is_empty() {
            continue;
        }
        let parsed = domain_parser
            .parse_domain(domain);
        let parsed = match parsed {
            Ok(parsed) => parsed,
            Err(e) => {
                eprintln!("Error while parsing {domain}: {e:?}");
                continue;
            }
        };
        let suffix = parsed.suffix().expect("No suffix?");
        let domain = parsed.root().unwrap_or(suffix);
        unique_domains.insert(domain.to_owned());
        bar.inc(1);
    }
    for domain in unique_domains {
        println!("{domain}")
    }
}
