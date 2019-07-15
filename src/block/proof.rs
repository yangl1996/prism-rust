use crate::crypto::vrf::{VrfPublicKey, VrfSecretKey, VrfInput, VrfProof, VrfOutput};
use super::header::RandomSource;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Default)]
pub struct Proof {
    pub vrf_proof: VrfProof,
    pub vrf_output: VrfOutput,
    pub coin: Coin,
    /// The random source of the parent block
    pub parent_random_source: RandomSource,
    /// Block creation time in UNIX format.
    pub timestamp: u128,
}


//TODO: Move this ds to a better place (may be wallet)w
#[derive(Serialize, Deserialize, Clone, Debug, Default, Hash)]
pub struct Coin{
    pub pubkey: VrfPublicKey,
    pub value: u64,
    //Todo: Phase 3: proof that the coin is utxo.
}
