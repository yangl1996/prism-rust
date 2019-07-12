use crate::crypto::vrf::{VrfPublicKey, VrfSecretKey, VrfInput, VrfProof, VrfOutput};
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Default)]
pub struct Proof {
    pub vrf_proof: VrfProof,
    pub vrf_output: VrfOutput,
    pub coin: Coin,
}


//TODO: Move this ds to a better place (may be wallet)w
#[derive(Serialize, Deserialize, Clone, Debug, Default, Hash)]
pub struct Coin{
    pubkey: VrfPublicKey,
    value: u64,
    //Todo: Phase 3: proof that the coin is utxo.
}