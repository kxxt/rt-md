use std::{
    fs::File,
    io::{BufWriter, Lines, StdinLock, Write, stdin},
    net::{IpAddr, UdpSocket},
    sync::{
        Arc, LazyLock, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, get_service, post},
};
use chrono::{DateTime, FixedOffset, Local, Utc};
use clap::Parser;
use color_eyre::eyre::OptionExt;
use dns_exf_detect::{
    allowlist::{AllowList, AllowMode, DynamicAllowList, PlainAllowList},
    cache::SimulatedTimeToIdleCache,
    dataset::Datum,
    domain::{DomainParser, DynamicDomainParser},
    method::{
        self, AlertSummary, DetectionMethod,
        bfcms::{Bfcms, BfcmsAlertSummary},
    },
    syslog::SyslogHeader,
    threshold_tuning,
};
use hashbrown::{HashMap, HashSet};
use itertools::Itertools;
use nix::unistd::{Pid, close};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
    #[arg(short, long, help = "Detection threshold(per second)")]
    pub threshold: f64,
    #[arg(short = 'R', long, help = "Reset interval (ms)")]
    pub reset_interval: u64,
    #[arg(long, help = "trust RDNS queries")]
    pub trust_rdns: bool,
    #[arg(long, help = "Be quiet")]
    pub quiet: bool,
    #[arg(long, help = "Duration to run (seconds)")]
    pub duration: u32,
    #[arg(long, help = "Syslog udp address to listen for logs")]
    pub syslog: Option<String>,
    #[arg(long, hide = true, help = "Read logs from stdin")]
    pub broken_do_not_use_stdin: bool,
    #[arg(long, help = "Dashboard port", default_value_t = 6000)]
    pub port: u16,
    #[arg(long, help = "Debug mode")]
    pub debug: bool,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    Peacetime {},
    Eval {},
    Tune {
        #[arg(long, required = true, help = "the acceptable FPR for tuning")]
        acceptable_fpr: Vec<f64>,
    },
}

const PEACETIME_ALLOWLIST_PATH: &str = "peacetime.allowlist";

static DOMAIN_PARSER: LazyLock<DynamicDomainParser> = LazyLock::new(|| {
    DynamicDomainParser::new(
        [
            "../../psl/public_suffix_list.dat",
            "../../psl/old.dat",
            "../../psl/cdn.dat",
        ]
        .into_iter(),
    )
    .expect("Failed to initialize configurable suffix lists")
});

static ALLOWLISTS: LazyLock<DynamicAllowList<PlainAllowList, 3>> = LazyLock::new(|| {
    DynamicAllowList::from_fixed([
        PlainAllowList::load_tranco_csv("../../allowlist/popularity/top-1m.csv").unwrap(),
        if std::fs::exists(PEACETIME_ALLOWLIST_PATH).unwrap() {
            PlainAllowList::load(PEACETIME_ALLOWLIST_PATH, AllowMode::Domain).unwrap()
        } else {
            PlainAllowList::empty()
        },
        PlainAllowList::load("../../allowlist/local/local-tld.list", AllowMode::Suffix).unwrap(),
    ])
});

static QUERIES_PROCESSED: AtomicU64 = AtomicU64::new(0);
static DONE: AtomicBool = AtomicBool::new(false);

struct AllowListForwarder;

impl AllowList for AllowListForwarder {
    fn contains(&self, domain: &str, suffix: &str) -> bool {
        ALLOWLISTS.contains(domain, suffix)
    }

    fn mutable(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
struct Query {
    qtime: DateTime<FixedOffset>,
    qname: String,
    src: IpAddr,
}

trait RealtimeDnsSource {
    fn try_next(&mut self) -> color_eyre::Result<Option<Query>>;
}

struct SyslogSocketSource {
    socket: UdpSocket,
    buf: Vec<u8>,
}

impl SyslogSocketSource {
    pub fn new(addr: &str) -> color_eyre::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        Ok(Self {
            socket,
            buf: vec![0; 16384],
        })
    }
}

impl RealtimeDnsSource for SyslogSocketSource {
    fn try_next(&mut self) -> color_eyre::Result<Option<Query>> {
        let len = self.socket.recv(&mut self.buf)?;
        if len == self.buf.len() {
            panic!("Buffer is not large enough!")
        }
        let message = &self.buf[..len];
        let timestamp = Local::now().into();
        let Some((_header, message)) = SyslogHeader::parse(message, Some(timestamp)) else {
            eprintln!(
                "Failed to parse syslog message: {}",
                String::from_utf8_lossy(message)
            );
            return Ok(None);
        };
        let message = String::from_utf8_lossy(message.strip_suffix(b"\n").unwrap_or(message));
        let Some((qtime, qname, src)) = parse_syslog(&message) else {
            // eprintln!("Malformed syslog message: {}", message);
            return Ok(None);
        };
        Ok(Some(Query {
            qtime,
            qname: qname.to_owned(), // well, we could optimize this if it turns out to be slow.
            src,
        }))
    }
}

struct StdinSource {
    iter: Lines<StdinLock<'static>>,
}

impl StdinSource {
    pub fn new() -> Self {
        Self {
            iter: stdin().lines(),
        }
    }
}

impl RealtimeDnsSource for StdinSource {
    fn try_next(&mut self) -> color_eyre::Result<Option<Query>> {
        let line = self.iter.next().transpose()?;
        line.map(|line| {
            let mut iter = line.split(',');
            _ = iter.next();
            Ok::<_, color_eyre::Report>(Query {
                qtime: Local::now().into(),
                src: iter
                    .next()
                    .ok_or_eyre("missing client")?
                    .trim_ascii()
                    .parse()?,
                qname: iter
                    .next()
                    .ok_or_eyre("missing request")?
                    .trim_ascii()
                    .to_string(),
            })
        })
        .transpose()
    }
}

type SharedState = Arc<RwLock<AppState>>;

struct AppState {
    firing_alerts: HashMap<IpAddr, BfcmsAlertSummary>,
    ignored_hosts: HashSet<IpAddr>,
    dismissed: HashMap<IpAddr, Instant>,
    threshold: f64,
}

impl AppState {
    pub fn new(threshold: f64) -> SharedState {
        Arc::new(RwLock::new(Self {
            threshold,
            firing_alerts: HashMap::new(),
            ignored_hosts: HashSet::new(),
            dismissed: HashMap::new(),
        }))
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let mut cli = Cli::parse();
    println!("{cli:?}");
    cli.threshold *= cli.reset_interval as f64 / 1000.;
    let port = cli.port;
    println!("RawThreshold: {}", cli.threshold);
    let shared_state = AppState::new(cli.threshold);

    // Load ignored hosts.
    // We need this to avoid processing queries generated by the resolver itself.
    if let Some(ignored_hosts) = std::fs::read_to_string("ignored.hosts")
        .inspect_err(|_| eprintln!("Failed to load ignored host list. Skipping"))
        .ok()
        .map(|v| {
            v.lines()
                .filter_map(|l| {
                    l.parse::<IpAddr>()
                        .inspect_err(|e| {
                            eprintln!("Failed to parse {l} as ip address: {e}, ignoring")
                        })
                        .ok()
                })
                .collect_vec()
        })
    {
        for v in ignored_hosts {
            shared_state.write().unwrap().ignored_hosts.insert(v);
        }
    }

    let mypid = Pid::this().to_string();

    if matches!(&cli.command, Command::Eval { .. }) {
        // Launch pidstat to measure resource consumption
        let pidstat_out = std::fs::File::create("pidstat.json")?;
        let _pidstat_handle = std::process::Command::new("pidstat")
            .args([
                "-o",
                "JSON",
                "-ur",
                "-p",
                mypid.as_str(),
                "1", // per minute collection
            ])
            .stdout(pidstat_out)
            .spawn()?;
        // Launch a script to collect alerts over time
        let _alert_collector_handle =
            std::process::Command::new("../../scripts/online-alerts-export.sh")
                .args([&cli.port.to_string(), "60"])
                .spawn()?;
    }

    // Spawn detector
    let _detector_handle = thread::spawn({
        let shared_state_ = shared_state.clone();
        || detector_main(cli, shared_state_).expect("Detector thread failed with err")
    });

    let app = Router::new()
        .route("/alerts", get(alerts_get))
        .route("/allowlist", post(allowlist_post))
        .route("/ignore-host", post(ignore_host_post))
        .route("/dismiss", post(dismiss_post))
        .route_service("/", ServeFile::new("../../frontend/index.htm"))
        .fallback_service(get_service(ServeDir::new("../../frontend")))
        .with_state(shared_state.clone());

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;

    axum::serve(listener, app)
        // .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

// async fn shutdown_signal() {
//     #[cfg(unix)]
//     let terminate = async {
//         signal::unix::signal(signal::unix::SignalKind::terminate())
//             .expect("failed to install signal handler")
//             .recv()
//             .await;
//     };

//     tokio::select! {
//         _ = ctrl_c => {},
//         _ = terminate => {},
//     }
// }

async fn dismiss_post(State(state): State<SharedState>, host: String) -> Result<(), StatusCode> {
    let host: IpAddr = host.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let now = chrono::Utc::now().timestamp_millis();
    let instant = Instant::now();
    eprintln!("[{now}] Dismissed alert from host {host}");
    let mut state = state.write().unwrap();
    state.dismissed.insert(host, instant);
    if let Some(a) = state.firing_alerts.remove(&host) {
        eprintln!("[{now}] Dismissed alert: {a}");
    }
    Ok(())
}

async fn allowlist_post(State(state): State<SharedState>, d: String) -> Result<(), StatusCode> {
    if ALLOWLISTS.contains(&d, "") {
        return Err(StatusCode::NOT_MODIFIED);
    }
    ALLOWLISTS.allowlist(&d);
    let now = chrono::Utc::now().timestamp_millis();
    eprintln!("[{}] Allowlisted {d}", now,);
    // Scan alerts to deduct this domain
    let mut state = state.write().unwrap();
    let mut to_resolve = vec![];
    let threshold = state.threshold as u32;
    for (&k, alert) in state.firing_alerts.iter_mut() {
        for (domain, &value) in &alert.top_domains {
            if domain == &d {
                eprintln!("[{}] Deducted {} from alert {}", now, value, alert);
                alert.top_domains.change_priority(&d, 0);
                alert.total -= value;
                if alert.total <= threshold {
                    to_resolve.push(k);
                }
                break;
            }
        }
    }
    for k in to_resolve {
        let alert = state.firing_alerts.remove(&k).unwrap();
        eprintln!("[{}] Resolved alert {}", now, alert);
    }
    Ok(())
}

async fn ignore_host_post(
    State(state): State<SharedState>,
    host: String,
) -> Result<(), StatusCode> {
    let now = chrono::Utc::now().timestamp_millis();
    let host: IpAddr = host.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    eprintln!("[{now}] Ignored host {host}");
    let mut state = state.write().unwrap();
    state.ignored_hosts.insert(host);
    if let Some(a) = state.firing_alerts.remove(&host) {
        eprintln!("[{now}] Dropped alert from ignored host: {a}");
    }
    Ok(())
}

async fn alerts_get(
    State(state): State<SharedState>,
) -> Result<Json<HashMap<IpAddr, BfcmsAlertSummary>>, StatusCode> {
    let alerts = state.read().unwrap().firing_alerts.clone();
    Ok(Json(alerts))
}

fn detector_main(cli: Cli, state: SharedState) -> color_eyre::Result<()> {
    let source: &mut dyn RealtimeDnsSource = match (cli.broken_do_not_use_stdin, &cli.syslog) {
        (true, None) => &mut StdinSource::new(),
        (false, Some(addr)) => &mut SyslogSocketSource::new(addr)?,
        _ => panic!("You must specify exactly one of --syslog or --stdin"),
    };
    let mut method = method::BfcmsMethod::<SimulatedTimeToIdleCache<Bfcms>>::new(
        cli.threshold as u32,
        cli.reset_interval,
        AllowListForwarder,
        cli.trust_rdns,
        true,
    );
    let start = Instant::now();
    let mut related_domains = HashSet::new();
    let mut host_values = HashMap::new();
    let mut clients: HashMap<IpAddr, bool> = HashMap::new();
    // Metric collector
    std::thread::spawn(move || {
        let measure_duration = Duration::from_secs(60);
        loop {
            let last_queries_processed = QUERIES_PROCESSED.load(Ordering::Relaxed);
            std::thread::sleep(measure_duration);
            eprintln!(
                "[{}] Throughput: {}",
                chrono::Utc::now().timestamp_millis(),
                (QUERIES_PROCESSED.load(Ordering::Relaxed) - last_queries_processed) as f64
                    / measure_duration.as_secs_f64()
            );

            if start.elapsed().as_secs() >= cli.duration as u64 {
                DONE.store(true, Ordering::Relaxed);
                break;
            }
        }
    });
    loop {
        QUERIES_PROCESSED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let Some(Query { qtime, qname, src }) = source.try_next()? else {
            continue;
        };
        if qname.is_empty() {
            // Sometimes the log generator went crazy.
            continue;
        }
        let Ok(parsed) = DOMAIN_PARSER
            .parse_domain(&qname)
            .inspect_err(|e| eprintln!("Malformed domain: {}, err: {e}", &qname))
        else {
            continue;
        };
        let Some(suffix) = parsed.suffix() else {
            continue;
        };
        let domain = parsed.root().unwrap_or(suffix);
        let subdomain = parsed.prefix().unwrap_or_default();
        let (alert, value) = method.process_single(Datum {
            timestamp_ms: qtime.timestamp_millis() as u64,
            suffix,
            full: &qname,
            domain,
            subdomain,
            client: src,
        })?;
        if matches!(&cli.command, Command::Eval { .. }) {
            let _ = clients.entry(src).or_default();
        }
        if matches!(&cli.command, Command::Tune { .. }) {
            let entry = host_values.entry(src).or_default();
            if *entry < value {
                *entry = value;
            }
        }
        if let Some(alert) = alert {
            match &cli.command {
                Command::Peacetime {} => {
                    for (domain, _) in alert.domains() {
                        related_domains.insert(domain.to_owned());
                    }
                }
                Command::Eval {} => {
                    // Update the alert if it is not already firing
                    let mut state = state.write().unwrap();
                    let dismissed = if let Some(instant) = state.dismissed.get(&src) {
                        if instant.elapsed() >= Duration::from_millis(cli.reset_interval) {
                            state.dismissed.remove(&src);
                            false
                        } else {
                            true
                        }
                    } else {
                        false
                    };
                    if !dismissed && !state.ignored_hosts.contains(&src) {
                        clients.insert(src, true);
                        println!("{alert}"); // Print the fresh alert instead of the highest one //
                        if let Some(old_alert) = state.firing_alerts.get_mut(&src) {
                            if old_alert.total <= alert.total {
                                *old_alert = alert;
                            }
                        } else {
                            state.firing_alerts.insert(src, alert);
                        }
                    }
                }
                Command::Tune { .. } => {}
            }
        }
        if DONE.load(Ordering::Relaxed) {
            break;
        }
    }
    match &cli.command {
        Command::Peacetime {} => {
            // Write peacetime allowlist
            let mut w = BufWriter::new(File::create(PEACETIME_ALLOWLIST_PATH)?);
            for line in related_domains {
                let lower = line.to_ascii_lowercase();
                writeln!(w, "{}", lower)?;
            }
            w.flush()?;
            drop(w);
            println!("Wrote peacetime allowlist");
        }
        Command::Eval {} => {
            // calc. FPR TPR
            let total = clients.len();
            let alerted: usize = clients.values().map(|&v| v as usize).sum();
            println!(
                "Alerted clients: {:?}",
                clients
                    .iter()
                    .filter(|&(_, &v)| v)
                    .map(|(k, _)| k)
                    .collect_vec()
            );
            println!("Number of alerted clients: {alerted} / {total}");
        }
        Command::Tune { acceptable_fpr } => {
            for acceptable_fpr in acceptable_fpr {
                threshold_tuning(
                    &host_values,
                    *acceptable_fpr,
                    method.reset_interval() as f64,
                );
            }
        }
    }
    println!(
        "Detector Done! Number of queries processed: {}",
        QUERIES_PROCESSED.load(Ordering::Relaxed)
    );
    close(0).unwrap();

    Ok(())
}

/// Parse fields
/// log format:
/// q_time=2025-11-12T10:59:36.205934 a_time=2025-11-12T10:59:36.227074 src=xxx sport=54942 dst=8.8.8.8 tid=35549 q_name=xxx q_type=AAAA a_ip= a_cname=aaa,xxx error=
/// We want: qname srcip qtime
fn parse_syslog(line: &str) -> Option<(DateTime<FixedOffset>, &str, IpAddr)> {
    let mut qtime = None;
    let mut qname = None;
    let mut srcip = None;
    let fields = line.split_ascii_whitespace();
    for field in fields {
        let Some((k, v)) = field.split_once('=') else {
            continue;
        };
        match k {
            "q_time" => {
                qtime = Some(Utc::now().into());
            }
            "q_name" => qname = Some(v),
            "src" => srcip = v.parse().ok(),
            _ => {}
        }
    }
    let qname = qname?;
    if !qname
        .chars()
        .all(|v| v.is_ascii_alphanumeric() || v == '.' || v == '-' || v == '_' || v == '+'
             /* Unfortunately, iodine uses '+' which is not an allowed char for domain name. We allow it solely for the purpose of this experiment. */)
    {
        // Malformed query containing escape sequence or invalid characters for qname.
        // Our threat model assumed blocking such queries.
        return None;
    }
    Some((qtime?, qname, srcip?))
}
