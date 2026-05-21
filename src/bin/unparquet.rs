use std::{io::BufWriter, io::Write, net::IpAddr, path::Path};

// Convert Parquet to Zstd compressed CSV
use indicatif::{ProgressBar, ProgressStyle};
use parquet::{
    file::reader::{FileReader, SerializedFileReader},
    record::RowAccessor,
};
use zstd::Encoder;

fn main() {
    let path = std::env::args().nth(1).expect("Missing path to data");
    let file = std::fs::File::open(&path).expect("failed to open your data");
    let reader = SerializedFileReader::try_from(file).expect("failed to create parquet reader");
    let meta = reader.metadata();
    let num_rows: i64 = meta.row_groups().iter().map(|r| r.num_rows()).sum();
    let bar = ProgressBar::new(num_rows as u64).with_style(
        ProgressStyle::with_template("Elapsed: {elapsed} {wide_bar} Speed: {per_sec}").unwrap(),
    );

    let output_path = Path::new(&path).with_extension("csv.zst");
    let file = std::fs::File::create(&output_path).expect("Failed to open output file");
    let encoder = Encoder::new(file, 3)
        .expect("failed to init zstd encoder")
        .auto_finish();
    let mut writer = BufWriter::new(encoder);

    let iter = reader.into_iter().with_batch_size(1_000_000);

    writeln!(writer, "datetime, client, qname").unwrap();

    for row in bar.wrap_iter(iter) {
        let row = row.unwrap();
        let datetime = row.get_timestamp_millis(0).unwrap();
        let _resolver: IpAddr = row.get_string(1).unwrap().parse().unwrap();
        let client: IpAddr = row.get_string(2).unwrap().parse().unwrap();
        let qname = row.get_string(3).unwrap();
        let _qtype = row.get_string(4).unwrap();
        writeln!(writer, "{datetime}, {client}, {qname}").unwrap();
    }
}
