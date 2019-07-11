use crate::crypto::hash::{Hashable, H256};
use crate::crypto::vrf::{VrfPublicKey, VrfSecretKey, VrfInput, VrfProof};
use crate::crypto::vrf::VrfOutput;

// TODO: Add the address of the miner

/// The proof which certifies leader election and content integrity.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Proof {
    ///The three fields are used to check for pos leader election
    vrf_proof: VrfProof,
    vrf_output: VrfOutput,
    coin: Coin,
    ///This is a signature on the content of the block
    signature: Vec<u8>,
    ///This is the random source for child block
    random_source: [u8; 32]
}

impl Proof {
    pub fn len(&self) -> usize {
        return 1024; //TODO:: Calculate this properly
    }
}


//TODO: Move this ds to a better place
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Coin{
    pubkey: VrfPublicKey,
    value: u64
}


