use crate::crypto::vrf::{VrfPublicKey, VrfSecretKey, VrfInput, VrfProof, VrfOutput};
use crate::utxodb::Utxo;
use super::header::RandomSource;

pub type TimeStamp = u128;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Default)]
pub struct Metadata {
    pub vrf_proof: VrfProof,
    pub vrf_output: VrfOutput,
    pub coin: Coin,
    /// Random source of the parent block
    pub parent_random_source: RandomSource,
    /// Block creation time in UNIX format.
    pub timestamp: TimeStamp,
    /// Random source for child block
    pub random_source: RandomSource,
}


//TODO: Move this ds to a better place (may be wallet)
#[derive(Serialize, Deserialize, Clone, Debug, Default, Hash)]
pub struct Coin{
    utxo: Utxo,
    pubkey: VrfPublicKey,
    //Todo: Phase 3: Short proof that the coin is utxo.
}
