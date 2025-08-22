use clap::Parser;
use std::time::Duration;
use website_monitor::{monitor_websites, MonitorConfig, Shutdown, WebsiteStatus};

/// Simple CLI to run a single monitoring pass.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Website URLs to check
    urls: Vec<String>,

    /// Number of worker threads
    #[arg(long, default_value_t = 50)]
    workers: usize,

    /// Request timeout in seconds
    #[arg(long, default_value_t = 5)]
    timeout: u64,

    /// Maximum retries per website
    #[arg(long, default_value_t = 0)]
    retries: u32,
}

fn print_result(ws: &WebsiteStatus) {
    let rt_ms = ws.response_time.as_millis();
    match &ws.status {
        Ok(code) => {
            println!(
                "[OK] {} | status={} | {} ms | {}",
                ws.url, code, rt_ms, ws.timestamp
            );
        }
        Err(err) => {
            println!(
                "[ERR] {} | {} | {} ms | {}",
                ws.url, err, rt_ms, ws.timestamp
            );
        }
    }
}

fn main() {
    let args = Args::parse();

    if args.urls.is_empty() {
        eprintln!("No URLs provided. Example: website-monitor https://example.com");
        std::process::exit(1);
    }

    let shutdown = Shutdown::new();
    // Graceful shutdown on Ctrl+C: stop accepting new work and finish in-flight requests
    {
        let s = shutdown.clone();
        ctrlc::set_handler(move || {
            eprintln!("\nCtrl+C received — finishing in-flight requests and shutting down...");
            s.cancel();
        })
        .expect("failed to set Ctrl+C handler");
    }

    let config = MonitorConfig {
        worker_threads: args.workers,
        request_timeout: Duration::from_secs(args.timeout),
        max_retries: args.retries,
    };

    let results = monitor_websites(args.urls, config, Some(shutdown));

    // Summarize
    let mut ok = 0usize;
    let mut err = 0usize;

    for ws in &results {
        print_result(ws);
        if ws.status.is_ok() {
            ok += 1;
        } else {
            err += 1;
        }
    }

    println!("\nSummary: {} OK, {} ERR", ok, err);
}

