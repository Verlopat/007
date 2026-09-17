use serde::{Deserialize, Serialize};

pub type NodeId = u64;
pub type RoundId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub id: NodeId,
    pub adversarial: bool,
    pub credibility: f64,
    pub stake: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ballot {
    pub voter: NodeId,
    pub value: u8,            // 0 = no, 1 = yes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBallot {
    pub voter: NodeId,
    pub ciphertext_bytes: usize,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialDecryption {
    pub round: RoundId,
    pub member: NodeId,
    pub share_bytes: usize,
    pub zk_proof_bytes: usize,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TallyResult {
    pub round: RoundId,
    pub total_yes: u64,
    pub total_no: u64,
    pub decrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeSeat {
    pub node_id: NodeId,
    pub vrf_output: u64,
    pub proof_bytes: usize,
    pub adversarial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundConfig {
    pub round: RoundId,
    pub total_nodes: usize,
    pub sybil_fraction: f64,
    pub committee_size: usize,
    pub threshold: usize,
    pub mode: String,          // "tdg", "fixed-threshold", "naive"
    pub tdg_enabled: bool,     // only for tdg or fixed
    pub beacon: u64,
}
