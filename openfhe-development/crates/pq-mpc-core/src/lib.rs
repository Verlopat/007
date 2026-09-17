use anyhow::Result;
use async_trait::async_trait;
use blake3::Hasher;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use types::{Ballot, CommitteeSeat, EncryptedBallot, PartialDecryption, RoundConfig, TallyResult};

#[async_trait]
pub trait ThresholdCrypto: Send + Sync {
    async fn encrypt_ballot(&self, b: &Ballot) -> Result<(EncryptedBallot, u128)>;
    async fn partial_decrypt(&self, round: u64, member: u64, valid: bool) -> Result<(PartialDecryption, u128, u128)>;
    async fn combine(&self, round: u64, shares: &[PartialDecryption], ballots: &[Ballot]) -> Result<(TallyResult, u128)>;
}

pub struct MockThresholdCrypto;

#[async_trait]
impl ThresholdCrypto for MockThresholdCrypto {
    async fn encrypt_ballot(&self, b: &Ballot) -> Result<(EncryptedBallot, u128)> {
        let start = Instant::now();
        sleep(Duration::from_millis(2)).await;
        let mut h = Hasher::new();
        h.update(&b.voter.to_le_bytes());
        h.update(&[b.value]);
        let out = h.finalize().as_bytes().to_vec();
        Ok((
            EncryptedBallot {
                voter: b.voter,
                ciphertext_bytes: 1280,
                payload: out,
            },
            start.elapsed().as_millis(),
        ))
    }

    async fn partial_decrypt(&self, round: u64, member: u64, valid: bool) -> Result<(PartialDecryption, u128, u128)> {
        let dec_start = Instant::now();
        sleep(Duration::from_millis(4)).await;
        let dec_ms = dec_start.elapsed().as_millis();

        let zk_start = Instant::now();
        sleep(Duration::from_millis(2)).await;
        let zk_ms = zk_start.elapsed().as_millis();

        Ok((
            PartialDecryption {
                round,
                member,
                share_bytes: 320,
                zk_proof_bytes: 192,
                valid,
            },
            dec_ms,
            zk_ms,
        ))
    }

    async fn combine(&self, round: u64, shares: &[PartialDecryption], ballots: &[Ballot]) -> Result<(TallyResult, u128)> {
        let start = Instant::now();
        sleep(Duration::from_millis(3)).await;
        let yes = ballots.iter().filter(|b| b.value == 1).count() as u64;
        let no = ballots.len() as u64 - yes;
        Ok((
            TallyResult {
                round,
                total_yes: yes,
                total_no: no,
                decrypted: !shares.is_empty(),
            },
            start.elapsed().as_millis(),
        ))
    }
}

// Import the new VRF trait module
pub mod vrf;
pub use vrf::Vrf;
pub use vrf::SimpleVrf as MockVrf; // keep the old name as alias for compatibility

pub fn select_committee(
    cfg: &RoundConfig,
    nodes: &[(u64, bool)],
    vrf: &dyn Vrf,
) -> (Vec<CommitteeSeat>, u128) {
    let start = Instant::now();
    let mut seats = nodes
        .iter()
        .map(|(id, adv)| {
            let (out, proof_bytes) = vrf.eval(cfg.beacon, *id);
            CommitteeSeat {
                node_id: *id,
                vrf_output: out,
                proof_bytes,
                adversarial: *adv,
            }
        })
        .collect::<Vec<_>>();
    seats.sort_by_key(|s| s.vrf_output);
    seats.truncate(cfg.committee_size);
    (seats, start.elapsed().as_micros())
}

pub fn tdg_adjust_threshold(base: usize, committee_size: usize, sybil_pressure: f64) -> usize {
    let mut t = base;
    if sybil_pressure > 0.2 {
        t += 1;
    }
    if sybil_pressure > 0.4 {
        t += 2;
    }
    t.min(committee_size.saturating_sub(1)).max(1)
}

pub fn outcome_changed(
    honest_tally_yes: u64,
    honest_tally_no: u64,
    adversary_tally_yes: u64,
    adversary_tally_no: u64,
) -> bool {
    let honest = honest_tally_yes + honest_tally_no;
    let adversary = adversary_tally_yes + adversary_tally_no;
    honest > 0
        && adversary > 0
        && (honest_tally_yes > honest_tally_no) != (adversary_tally_yes > adversary_tally_no)
}
