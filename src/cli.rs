use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
    #[arg(long, help = "Path to dataset config")]
    pub dataset: PathBuf,
    #[arg(long, help = "Method to use")]
    pub method: String,
    #[arg(short, long, help = "Detection threshold(per second)")]
    pub threshold: f64,
    #[arg(short = 'R', long, help = "Reset interval")]
    pub reset_interval: u64,
    #[arg(short = 'k', long, default_value = "1000", help = "ibHH cache size")]
    pub ibhh_k: usize,
    #[arg(long, help = "trust RDNS queries")]
    pub trust_rdns: bool,
    #[arg(long, help = "skip peacetime allowlist")]
    pub skip_peacetime_allowlist: bool,
    #[arg(long, help = "skip popularity allowlist")]
    pub skip_popularity_allowlist: bool,
    #[arg(long, help = "Be quiet")]
    pub quiet: bool,
    #[arg(long)]
    pub skip_internal_allowlist: bool,
    #[arg(long)]
    pub ablation_rdns_special: bool,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    Peacetime {},
    Eval {},
    Tune {
        #[arg(long, required = true, help = "the acceptable FPR for tuning")]
        acceptable_fpr: Vec<f64>,
    },
    Bench {},
}
