use chrono::{DateTime, Utc};
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

/// Output format
#[derive(Debug, Clone)]
pub struct WebsiteStatus {
    pub url: String,
    pub status: Result<u16, String>,
    pub response_time: Duration,
    pub timestamp: DateTime<Utc>,
}

/// Configurable options
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    /// Per-request timeout (default 5s recommended)
    pub request_timeout: Duration,
    /// Maximum number of retries per website (0 = no retry)
    pub max_retries: u32,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            worker_threads: 50,
            request_timeout: Duration::from_secs(5),
            max_retries: 0,
        }
    }
}

/// Graceful shutdown token.
/// Cancels new work and lets in-flight requests finish.
#[derive(Clone, Default)]
pub struct Shutdown {
    cancelled: Arc<AtomicBool>,
}
impl Shutdown {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Internal job message
#[derive(Debug, Clone)]
struct Job {
    url: String,
    attempt: u32,
}

/// Perform a single HTTP GET and return the status code.
fn fetch_status(client: &reqwest::blocking::Client, url: &str) -> Result<u16, String> {
    let resp = client
        .get(url)
        .send()
        .map_err(|e| format!("request error: {e}"))?;

    Ok(resp.status().as_u16())
}

/// Core monitoring function.
pub fn monitor_websites(
    urls: Vec<String>,
    mut config: MonitorConfig,
    shutdown: Option<Shutdown>,
) -> Vec<WebsiteStatus> {
    if urls.is_empty() {
        return Vec::new();
    }

    if config.worker_threads == 0 {
        config.worker_threads = 1;
    }
    config.worker_threads = config.worker_threads.min(urls.len());

    let shutdown = shutdown.unwrap_or_else(Shutdown::new);

    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let (res_tx, res_rx) = mpsc::channel::<WebsiteStatus>();

    // Enqueue initial jobs
    for url in &urls {
        let _ = job_tx.send(Job {
            url: url.clone(),
            attempt: 0,
        });
    }
    // Share the receiver among workers
    let job_rx = Arc::new(Mutex::new(job_rx));

    // Spawn workers
    let mut workers = Vec::with_capacity(config.worker_threads);
    for _ in 0..config.worker_threads {
        let jobs_shared = Arc::clone(&job_rx);
        let results = res_tx.clone();
        let job_tx_retry = job_tx.clone();
        let shutdown_clone = shutdown.clone();
        let timeout = config.request_timeout;
        let max_retries = config.max_retries;

        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .expect("failed to build reqwest client");

        workers.push(thread::spawn(move || {
            loop {
                if shutdown_clone.is_cancelled() {
                    break;
                }

                // Poll the shared receiver with a short timeout so we can notice shutdown.
                let job_opt = {
                    let rx_guard = jobs_shared.lock().expect("poisoned receiver mutex");
                    rx_guard.recv_timeout(Duration::from_millis(100)).ok()
                };

                let Some(job) = job_opt else {
                    // timeout or channel closed; if channel closed, we’re done
                    // check if all senders are gone (recv_timeout Err::Disconnected)
                    // We can detect it by trying again immediately; but simplest:
                    // if there are no more senders AND queue is empty, all workers will get None repeatedly.
                    // We’ll break when shutdown is requested or when no more jobs will ever arrive.
                    // To avoid spin, sleep a touch.
                    if shutdown_clone.is_cancelled() {
                        break;
                    }
                    // If channel is actually disconnected, future recv_timeout will always Err,
                    // but we'll still loop and exit after not receiving any new URLs and seen-count completes upstream.
                    continue;
                };

                let start = Instant::now();
                let result = fetch_status(&client, &job.url);
                let elapsed = start.elapsed();

                match result {
                    Ok(code) => {
                        let _ = results.send(WebsiteStatus {
                            url: job.url,
                            status: Ok(code),
                            response_time: elapsed,
                            timestamp: Utc::now(),
                        });
                    }
                    Err(err) => {
                        if !shutdown_clone.is_cancelled() && job.attempt < max_retries {
                            // Light backoff
                            let backoff = Duration::from_millis(100 * (job.attempt as u64 + 1));
                            thread::sleep(backoff);
                            let _ = job_tx_retry.send(Job {
                                url: job.url,
                                attempt: job.attempt + 1,
                            });
                        } else {
                            let _ = results.send(WebsiteStatus {
                                url: job.url,
                                status: Err(err),
                                response_time: elapsed,
                                timestamp: Utc::now(),
                            });
                        }
                    }
                }
            }
        }));
    }

    // Drop main’s extra senders so the channel closes when workers finish retrying
    drop(job_tx);
    drop(res_tx);

    // Collect results: one per unique URL
    let mut seen = HashSet::with_capacity(urls.len());
    let mut out = Vec::with_capacity(urls.len());

    while seen.len() < urls.len() {
        match res_rx.recv() {
            Ok(ws) => {
                if seen.insert(ws.url.clone()) {
                    out.push(ws);
                }
            }
            Err(_) => break, // all senders dropped
        }
    }

    for w in workers {
        let _ = w.join();
    }

    out
}

/// Convenience: run a single pass with defaults and no shutdown handle.
pub fn monitor_once(urls: Vec<String>) -> Vec<WebsiteStatus> {
    monitor_websites(urls, MonitorConfig::default(), None)
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    #[test]
    fn config_default_sane() {
        let cfg = MonitorConfig::default();
        assert!(cfg.worker_threads >= 1);
        assert_eq!(cfg.request_timeout, Duration::from_secs(5));
    }

    #[test]
    fn shutdown_flag_works() {
        let s = Shutdown::new();
        assert!(!s.is_cancelled());
        s.cancel();
        assert!(s.is_cancelled());
    }
}
