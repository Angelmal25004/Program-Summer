use httpmock::prelude::*;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};
use website_monitor::{monitor_websites, MonitorConfig, Shutdown};

pub fn monitor_websites(
    urls: Vec<String>,
    mut config: MonitorConfig,
    shutdown: Option<Shutdown>,
) -> Vec<WebsiteStatus> {
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::{Duration, Instant};

    if urls.is_empty() {
        return Vec::new();
    }

    // Normalize worker count
    if config.worker_threads == 0 {
        config.worker_threads = 1;
    }
    config.worker_threads = config.worker_threads.min(urls.len());

    let shutdown = shutdown.unwrap_or_else(Shutdown::new);

    // Channels: jobs in, results out
    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let (res_tx, res_rx) = mpsc::channel::<WebsiteStatus>();

    // Queue initial jobs
    for url in &urls {
        let _ = job_tx.send(Job {
            url: url.clone(),
            attempt: 0,
        });
    }

    // Share receiver among workers (Receiver isn't cloneable)
    let job_rx = Arc::new(Mutex::new(job_rx));

    // Spawn workers
    let mut workers = Vec::with_capacity(config.worker_threads);
    for _ in 0..config.worker_threads {
        let jobs = Arc::clone(&job_rx);
        let results = res_tx.clone();
        let job_tx_retry = job_tx.clone(); // used only when retrying
        let shutdown_clone = shutdown.clone();
        let timeout = config.request_timeout;
        let max_retries = config.max_retries;

        // Build a per-thread blocking client with timeout & limited redirects
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .expect("failed to build reqwest client");

        workers.push(std::thread::spawn(move || {
            use std::sync::mpsc::RecvTimeoutError;

            loop {
                if shutdown_clone.is_cancelled() {
                    break;
                }

                // Try to receive a job with a short timeout so we can re-check shutdown regularly.
                let job = {
                    let rx = jobs.lock().unwrap();
                    rx.recv_timeout(Duration::from_millis(100))
                };

                let job = match job {
                    Ok(j) => j,
                    Err(RecvTimeoutError::Timeout) => {
                        // No job right now — loop to check shutdown or await more work
                        continue
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        // All senders dropped; no more jobs will arrive
                        break
                    }
                };

                // Execute the job
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
                        // Retry with light backoff if allowed and not shutting down
                        if !shutdown_clone.is_cancelled() && job.attempt < max_retries {
                            let backoff = Duration::from_millis(100 * (job.attempt as u64 + 1));
                            std::thread::sleep(backoff);
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
            // Worker exits: drops its Sender clones, helping receivers detect closure if needed
        }));
    }

    // Drop main clones we no longer need; workers retain their own clones.
    drop(job_tx);
    drop(res_tx);

    // Collect exactly one result per unique URL.
    use std::collections::HashSet;
    let mut seen = HashSet::with_capacity(urls.len());
    let mut out = Vec::with_capacity(urls.len());

    while seen.len() < urls.len() {
        match res_rx.recv() {
            Ok(ws) => {
                if seen.insert(ws.url.clone()) {
                    out.push(ws);
                }
            }
            Err(_) => {
                // All result senders dropped; workers done
                break;
            }
        }
    }

    // We’ve gathered all results we need — trigger graceful shutdown so workers exit
    shutdown.cancel();

    // Join workers
    for w in workers {
        let _ = w.join();
    }

    out
}
