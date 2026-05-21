use dns_exf_detect::domain::{DomainParser, StaticDomainParser};
use std::io::stdin;

// Scans for invalid data in lines
fn main() {
    let parser = StaticDomainParser::new(
        ["psl/public_suffix_list.dat", "psl/old.dat", "psl/cdn.dat"].into_iter(),
    )
    .expect("Failed to create domain parser");
    for (i, line) in stdin().lines().enumerate() {
        let line = line.expect("Failed to read line");
        // hhmmssdotms = fields[0]
        // src_ip = fields[1]
        // query_name = fields[-2]
        // qtype = fields[-1]
        let fields: Vec<_> = line.trim().split(' ').collect();
        let client = fields[1];
        let [qname, qtype] = fields.last_chunk::<2>().expect("More fields expected");
        if qname.is_empty() {
            println!("Got empty query name from {client}, {qtype}");
            continue;
        }
        if qname.contains('\\') {
            // invalid data
            continue;
        }
        let _ = parser
            .parse_domain(qname)
            .inspect_err(|e| eprintln!("Failed to parse {qname} {qtype} on line {i}, error: {e}"));
    }
}
