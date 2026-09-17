use clap::Parser;
use metrics::{MetricsCollector, RoundMetrics};
use pq_mpc_core::{
    outcome_changed,
    select_committee,
    tdg_adjust_threshold,
    MockThresholdCrypto,
    MockVrf,      // now = SimpleVrf, via the alias
    ThresholdCrypto,
};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::time::Instant;
use types::{Ballot, NodeIdentity, RoundConfig, TallyResult};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 100)]
    nodes: usize,
    #[arg(long, default_value_t = 20)]
    committee_size: usize,
    #[arg(long, default_value_t = 12)]
    threshold: usize,
    #[arg(long, default_value_t = 0.30)]
    sybil_frac: f64,
    #[arg(long, default_value_t = 50)]
    rounds: usize,
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    tdg_enabled: bool,
    #[arg(long, default_value = "tdg")]
    mode: String, // "tdg", "fixed-threshold", "naive"
    #[arg(long, default_value = "output/logs/sim.jsonl")]
    out: String,
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

fn build_nodes(n: usize, sybil_frac: f64) -> Vec<NodeIdentity> {
    let sybils = ((n as f64) * sybil_frac).round() as usize;
    (0..n)
        .map(|i| NodeIdentity {
            id: i as u64,
            adversarial: i < sybils,
            credibility: if i < sybils { 0.2 } else { 1.0 },
            stake: 1.0,
        })
        .collect()
}

fn generate_honest_ballots(nodes: &[NodeIdentity], rng: &mut ChaCha8Rng) -> Vec<Ballot> {
    nodes
        .iter()
        .filter(|n| !n.adversarial)
        .map(|n| Ballot {
            voter: n.id,
            value: if rng.gen_bool(0.5) { 1 } else { 0 },
        })
        .collect()
}

fn generate_adversary_ballots(nodes: &[NodeIdentity]) -> Vec<Ballot> {
    nodes
        .iter()
        .filter(|n| n.adversarial)
        .map(|n| Ballot {
            voter: n.id,
            value: 1,
        })
        .collect()
}

fn snapshot_all_ballots(nodes: &[NodeIdentity], rng: &mut ChaCha8Rng) -> Vec<Ballot> {
    let honest = generate_honest_ballots(nodes, rng);
    let adversary = generate_adversary_ballots(nodes);
    let mut combined = honest;
    combined.extend(adversary);
    combined.shuffle(rng);
    combined
}

async fn run_sim_naive(
    rng: &mut ChaCha8Rng,
    nodes: &[NodeIdentity],
    config: &RoundConfig,
    out: &MetricsCollector,
) {
    let ballots = snapshot_all_ballots(nodes, rng);

    let honest_votes: Vec<Ballot> = ballots
        .iter()
        .filter(|b| !nodes[b.voter as usize].adversarial)
        .cloned()
        .collect();

    let all_votes = ballots.clone();

    let honest_tally_yes = honest_votes.iter().filter(|b| b.value == 1).count() as u64;
    let honest_tally_no = honest_votes.len() as u64 - honest_tally_yes;

    let adversary_tally_yes = all_votes.iter().filter(|b| b.value == 1).count() as u64;
    let adversary_tally_no = all_votes.len() as u64 - adversary_tally_yes;

    let e2e = Instant::now();

    let changed = outcome_changed(
        honest_tally_yes,
        honest_tally_no,
        adversary_tally_yes,
        adversary_tally_no,
    );

    let metric = RoundMetrics {
        round: config.round,
        num_sybils: nodes.iter().filter(|n| n.adversarial).count(),
        committee_size: 0,
        threshold: 0,
        mode: config.mode.clone(),
        adversarial: changed,
        adversary_controlled_seats: 0,
        outcome_changed: changed,
        success: changed,
        liveness_fail: false,
        tallied: true,
        honest_tally_yes,
        honest_tally_no,
        adversary_tally_yes,
        adversary_tally_no,
        tally_latency_ms: e2e.elapsed().as_millis(),
        encrypt_ms: 0,
        ct_bytes: 0,
        partial_dec_ms: 0,
        zk_prove_ms: 0,
        zk_proof_bytes: 0,
        combine_ms: 0,
        vrf_eval_us: 0,
        net_bytes_sent: 0,
        net_bytes_recv: 0,
        sybil_pressure: config.sybil_fraction,
        new_threshold: 0,
    };

    out.write_round(&metric);
}

async fn run_sim_mpc(
    rng: &mut ChaCha8Rng,
    nodes: &[NodeIdentity],
    config: &RoundConfig,
    tdg_enabled: bool,
    crypto: &MockThresholdCrypto,
    vrf: &dyn pq_mpc_core::Vrf,   // this is now vrf::Vrf
    out: &MetricsCollector,
) {
    let base_threshold = config.threshold;
    let committee_size = config.committee_size;
    let sybil_pressure = config.sybil_fraction;

    let new_threshold = if tdg_enabled {
        tdg_adjust_threshold(base_threshold, committee_size, sybil_pressure)
    } else {
        base_threshold
    };

    let node_refs = nodes
        .iter()
        .map(|n| (n.id, n.adversarial))
        .collect::<Vec<_>>();

    let (committee, vrf_eval_us) = select_committee(config, &node_refs, vrf);
    let bad_seats = committee.iter().filter(|c| c.adversarial).count();

    let ballots = snapshot_all_ballots(nodes, rng);

    let honest_votes: Vec<Ballot> = ballots
        .iter()
        .filter(|b| !nodes[b.voter as usize].adversarial)
        .cloned()
        .collect();

    let honest_tally_yes = honest_votes.iter().filter(|b| b.value == 1).count() as u64;
    let honest_tally_no = honest_votes.len() as u64 - honest_tally_yes;

    let e2e = Instant::now();

    let mut encrypt_ms = 0u128;
    let mut ct_bytes = 0usize;
    for b in &ballots {
        let (ct, enc_ms) = crypto.encrypt_ballot(b).await.unwrap();
        encrypt_ms += enc_ms;
        ct_bytes += ct.ciphertext_bytes;
    }

    let honest_seats = committee.iter().filter(|c| !c.adversarial).count();
    let liveness_fail = honest_seats < new_threshold;

    let mut partial_dec_ms = 0u128;
    let mut zk_prove_ms = 0u128;
    let mut zk_proof_bytes = 0usize;
    let mut net_bytes_sent = 0usize;
    let mut net_bytes_recv = 0usize;
    let mut shares = Vec::new();

    if !liveness_fail {
        for seat in committee.iter().take(new_threshold) {
            let (share, dec_ms, zk_ms) = crypto
                .partial_decrypt(config.round, seat.node_id, true)
                .await
                .unwrap();
            partial_dec_ms += dec_ms;
            zk_prove_ms += zk_ms;
            zk_proof_bytes += share.zk_proof_bytes;
            net_bytes_sent += share.share_bytes + share.zk_proof_bytes;
            net_bytes_recv += share.share_bytes + share.zk_proof_bytes;
            shares.push(share);
        }
    }

    let (tally, combine_ms): (TallyResult, u128) = if liveness_fail {
        (
            TallyResult {
                round: config.round,
                total_yes: 0,
                total_no: 0,
                decrypted: false,
            },
            0,
        )
    } else {
        crypto.combine(config.round, &shares, &ballots).await.unwrap()
    };

    let adversary_tally_yes = tally.total_yes;
    let adversary_tally_no = tally.total_no;

    let changed = if tally.total_yes + tally.total_no == 0 {
        false
    } else {
        outcome_changed(
            honest_tally_yes,
            honest_tally_no,
            adversary_tally_yes,
            adversary_tally_no,
        )
    };

    let metric = RoundMetrics {
        round: config.round,
        num_sybils: nodes.iter().filter(|n| n.adversarial).count(),
        committee_size,
        threshold: base_threshold,
        mode: config.mode.clone(),
        adversarial: bad_seats >= new_threshold,
        adversary_controlled_seats: bad_seats,
        outcome_changed: changed,
        success: bad_seats >= new_threshold,
        liveness_fail,
        tallied: tally.decrypted,
        honest_tally_yes,
        honest_tally_no,
        adversary_tally_yes,
        adversary_tally_no,
        tally_latency_ms: e2e.elapsed().as_millis(),
        encrypt_ms,
        ct_bytes,
        partial_dec_ms,
        zk_prove_ms,
        zk_proof_bytes,
        combine_ms,
        vrf_eval_us,
        net_bytes_sent,
        net_bytes_recv,
        sybil_pressure,
        new_threshold,
    };

    out.record_latency(e2e.elapsed());
    out.write_round(&metric);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().init();

    let args = Args::parse();
    let metrics = MetricsCollector::new(&args.out);
    let mut rng = ChaCha8Rng::seed_from_u64(args.seed);
    let crypto = MockThresholdCrypto;
    let vrf = MockVrf;  // this is now vrf::SimpleVrf

    let nodes = build_nodes(args.nodes, args.sybil_frac);

    for round in 0..args.rounds {
        let beacon = rng.gen::<u64>();
        let cfg = RoundConfig {
            round: round as u64,
            total_nodes: args.nodes,
            sybil_fraction: args.sybil_frac,
            committee_size: args.committee_size,
            threshold: args.threshold,
            mode: args.mode.clone(),
            tdg_enabled: args.tdg_enabled,
            beacon,
        };

        match cfg.mode.as_str() {
            "naive" => {
                run_sim_naive(&mut rng, &nodes, &cfg, &metrics).await;
            }
            "fixed-threshold" | "tdg" => {
                let tdg_on = cfg.mode == "tdg" && args.tdg_enabled;
                run_sim_mpc(&mut rng, &nodes, &cfg, tdg_on, &crypto, &vrf, &metrics).await;
            }
            _ => {
                panic!("unknown mode: {}", cfg.mode);
            }
        }
    }

    Ok(())
}
