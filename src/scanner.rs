use futures::StreamExt;
use indicatif::ProgressBar;
use serde::Serialize;
use std::{collections::HashMap, net::IpAddr, sync::Arc};
use tabled::Tabled;
use tokio::{
    net::TcpStream,
    sync::mpsc,
    time::{timeout, Duration},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

use crate::services::{get_services, ServiceEntry};

const MAX_COMMIT_LEN: usize = 60;

#[derive(Debug, Tabled, Serialize)]
pub struct ScanPair {
    pub ip: IpAddr,
    pub port: u16,
    pub service_name: String,
    pub comment: String,
}

async fn scan_one(
    ip: IpAddr,
    port: u16,
    per_port: Duration,
    services: Arc<HashMap<u16, ServiceEntry>>,
    token: CancellationToken,
) -> Option<ScanPair> {
    if token.is_cancelled() {
        return None;
    }

    match timeout(per_port, TcpStream::connect((ip, port))).await {
        Ok(Ok(_)) => {
            let entry = services.get(&port);
            let name = entry.map(|s| s.name).unwrap_or("unknown");
            let raw = entry.map(|s| s.comment).unwrap_or("");
            let comment = if raw.len() > MAX_COMMIT_LEN {
                format!("{}…", &raw[..MAX_COMMIT_LEN].trim_end())
            } else {
                raw.to_string()
            };

            Some(ScanPair {
                ip,
                port,
                service_name: name.to_string(),
                comment,
            })
        }
        _ => None,
    }
}

pub async fn scan_ips(
    ips: Vec<IpAddr>,
    range: (u16, u16),
    per_port: Duration,
    max_concurrency: usize,
    pb: ProgressBar,
    token: CancellationToken,
) -> Vec<ScanPair> {
    let services = Arc::new(get_services());

    let (tx, rx) = mpsc::channel::<(IpAddr, u16)>(max_concurrency * 2);
    tokio::spawn({
        let token = token.clone();
        async move {
            for ip in ips {
                for port in range.0..=range.1 {
                    if token.is_cancelled() {
                        return;
                    }
                    if tx.send((ip, port)).await.is_err() {
                        return;
                    }
                }
            }
        }
    });

    let token_for_workers = token.clone();
    let mut stream = ReceiverStream::new(rx)
        .map(move |(ip, port)| {
            scan_one(
                ip,
                port,
                per_port,
                services.clone(),
                token_for_workers.clone(),
            )
        })
        .buffer_unordered(max_concurrency);

    let mut out = Vec::new();
    loop {
        tokio::select! {
            _ = token.cancelled() => break,

            res = stream.next() => match res {
                Some(Some(p)) => {
                    out.push(p);
                    pb.inc(1);
                }
                Some(None) => {
                    pb.inc(1);
                }
                None => break,
            },
        }
    }

    pb.finish_and_clear();
    out
}
