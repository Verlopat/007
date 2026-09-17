use std::time::Duration;

pub trait Vrf: Send + Sync {
    fn eval(&self, beacon: u64, node_id: u64) -> (u64, usize);
    fn name(&self) -> &'static str;
}

pub struct SimpleVrf;

impl Vrf for SimpleVrf {
    fn eval(&self, beacon: u64, node_id: u64) -> (u64, usize) {
        let mut h = blake3::Hasher::new();
        h.update(&beacon.to_le_bytes());
        h.update(&node_id.to_le_bytes());
        let bytes = h.finalize();
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes.as_bytes()[..8]);
        (u64::from_le_bytes(arr), 96)
    }

    fn name(&self) -> &'static str {
        "simple-hash-vrf"
    }
}
