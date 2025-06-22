use clap::{value_parser, ArgGroup, Parser, ValueEnum};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use ipnet::IpNet;
use k0fiscan::{
    ports::{get_top_ports, parse_port},
    scanner::scan_ips,
    services::get_services,
};
use std::{net::IpAddr, time::Duration, sync::Arc, io};
use std::io::Write;
use tabled::{settings::Style, Table};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum OutputFormat {
    Table,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "☕ k0fiscan — fast, async TCP port scanner brewed with Tokio."
)]
#[command(group(
    ArgGroup::new("scope").required(true).args(
        &["target", "network", "start_ip", "ip_list"]
    )
))]
struct Args {
    #[arg(group = "scope", short, long)]
    target: Option<IpAddr>,

    #[arg(group = "scope", short, long)]
    network: Option<String>,

    #[arg(short = 's', long, requires = "end_ip")]
    start_ip: Option<IpAddr>,

    #[arg(short = 'e', long, requires = "start_ip")]
    end_ip: Option<IpAddr>,

    #[arg(short, long)]
    port_range: Option<String>,

    #[arg(short = 'x', long, default_value_t = 10, value_parser = value_parser!(u8).range(0..100),
        help = "percentage 0-100")]
    top_ports: u8,

    #[arg(short = 'l', long = "list", value_parser = value_parser!(IpAddr), value_delimiter = ',')]
    ip_list: Option<Vec<IpAddr>>,

    #[arg(short = 'm', long, default_value_t = 500)]
    max_tasks: usize,

    #[arg(short = 'o', long, default_value = "table")]
    output: OutputFormat,
}

fn get_ips_from_cidr(network: &str) -> Vec<IpAddr> {
    let net: IpNet = network.parse().expect("Invalid CIDR");
    net.hosts().collect()
}

fn ips_between(start: IpAddr, end: IpAddr) -> Vec<IpAddr> {
    match (start, end) {
        (IpAddr::V4(s), IpAddr::V4(e)) => {
            let (s, e) = (u32::from(s), u32::from(e));
            if s > e {
                return Vec::new();
            }
            (s..=e).map(|n| IpAddr::V4(n.into())).collect()
        }
        (IpAddr::V6(s), IpAddr::V6(e)) => {
            let (s, e) = (u128::from(s), u128::from(e));
            if s > e {
                return Vec::new();
            }
            (s..=e).map(|n| IpAddr::V6(n.into())).collect()
        }
        _ => Vec::new(),
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let srv_map = get_services();
    let ports: Vec<u16> = if let Some(port_range) = args.port_range {
        parse_port(&port_range).expect("msg")
    } else {
        get_top_ports(&srv_map, args.top_ports as f32)
    };

    let ips: Vec<IpAddr> = if let Some(target) = args.target {
        vec![target]
    } else if let Some(network) = args.network {
        get_ips_from_cidr(&network)
    } else if let (Some(start), Some(end)) = (args.start_ip, args.end_ip) {
        ips_between(start, end)
    } else if let Some(ip_list) = args.ip_list {
        ip_list
    } else {
        panic!("No valid IP")
    };

    let cancel = CancellationToken::new();

    // Ctrl-C handler:
    {
        let cancel = cancel.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            eprintln!("\n⏹  Ctrl-C: stopping scan …");
            cancel.cancel();
        });
    }

    let multi = MultiProgress::new();
    let spinner = multi.add(ProgressBar::new_spinner());
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );
    spinner.set_message("☕ Scanning started... brewing ports");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let total = ips.len() * ports.len();
    let pb = multi.add(ProgressBar::new(total as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% ETA: {eta}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let open_ports = scan_ips(
        ips,
        ports,
        Duration::from_millis(300),
        args.max_tasks,
        pb,
        Arc::new(srv_map),
        cancel.clone(),
    )
    .await;

    spinner.finish_with_message("☕ Done — your coffee is ready!");

    match args.output {
        OutputFormat::Table => {
            let mut table = Table::new(open_ports);
            table.with(Style::modern());
            if let Err(e) = writeln!(io::stdout(), "{table}") {
                if e.kind() != io::ErrorKind::BrokenPipe {
                    eprintln!("Error writing table: {e}");
                }
                std::process::exit(0);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&open_ports).unwrap();
            if let Err(e) = writeln!(io::stdout(), "{}", json) {
                if e.kind() != io::ErrorKind::BrokenPipe {
                    eprintln!("Error writing JSON: {e}");
                }
                std::process::exit(0);
            }
        }
    }

}
