use hdrhistogram::Histogram;
use serde::Serialize;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use types::RoundId;

#[derive(Debug, Clone, Serialize)]
pub struct RoundMetrics {
    pub round: RoundId,
    pub num_sybils: usize,
    pub committee_size: usize,
    pub threshold: usize,
    pub mode: String, // "tdg", "fixed-threshold", "naive"
    pub adversarial: bool,
    pub adversary_controlled_seats: usize,
    pub outcome_changed: bool,   // adversary changed outcome
    pub success: bool,           // committee captured
    pub liveness_fail: bool,
    pub tallied: bool,
    pub honest_tally_yes: u64,
    pub honest_tally_no: u64,
    pub adversary_tally_yes: u64,
    pub adversary_tally_no: u64,
    pub tally_latency_ms: u128,
    pub encrypt_ms: u128,
    pub ct_bytes: usize,
    pub partial_dec_ms: u128,
    pub zk_prove_ms: u128,
    pub zk_proof_bytes: usize,
    pub combine_ms: u128,
    pub vrf_eval_us: u128,
    pub net_bytes_sent: usize,
    pub net_bytes_recv: usize,
    pub sybil_pressure: f64,
    pub new_threshold: usize,
}

pub struct MetricsCollector {
    file: PathBuf,
    latency_hist: Arc<Mutex<Histogram<u64>>>,
}

impl MetricsCollector {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let file = path.into();
        if let Some(parent) = file.parent() {
            let _ = create_dir_all(parent);
        }
        Self {
            file,
            latency_hist: Arc::new(Mutex::new(Histogram::new(3).unwrap())),
        }
    }

    pub fn record_latency(&self, d: Duration) {
        let mut h = self.latency_hist.lock().unwrap();
        let _ = h.record(d.as_millis() as u64);
    }

    pub fn write_round(&self, m: &RoundMetrics) {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)
            .unwrap();
        let line = serde_json::to_string(m).unwrap();
        let _ = writeln!(f, "{line}");
    }
}
