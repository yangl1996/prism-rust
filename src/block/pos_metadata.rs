use crate::crypto::vrf::{VrfPublicKey, VrfSecretKey, VrfInput, VrfProof, VrfValue};
use crate::utxodb::Utxo;


pub type RandomSource = [u8; 32];
pub type TimeStamp = u128;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Default)]
pub struct Metadata {
    pub vrf_proof: VrfProof,
    pub vrf_value: VrfValue,
    pub vrf_pubkey: VrfPublicKey,
    /// Coin which "mined" the block
    pub utxo: Utxo,
    /// Random source of the parent block
    pub parent_random_source: RandomSource,
    /// Block creation time in UNIX format
    pub timestamp: TimeStamp,
    /// Random source for child block
    pub random_source: RandomSource,
}