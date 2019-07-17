use super::hash::{Hashable, H256};
use ed25519_dalek::{PublicKey,SecretKey};
use serde::{Serialize, Deserialize};
use crate::block::pos_metadata::{TimeStamp, RandomSource};
use crate::transaction::CoinId;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Hash)]
pub struct VrfPublicKey([u8; 32]); //TODO: We are using a fake public key for now

impl std::convert::From<&PublicKey> for VrfPublicKey {
    fn from(other: &PublicKey) -> Self {
        VrfPublicKey(other.to_bytes())
    }
}

pub type VrfSecretKey = SecretKey;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Hash)]
pub struct VrfInput{
    pub random_source: RandomSource,
    pub time: TimeStamp,
    pub coin: CoinId,
} //Random source and time


#[derive(Serialize, Deserialize, Clone, Debug, Default, Hash    )]
pub struct VrfProof ([u8; 32]);

impl VrfProof{
    fn default() -> Self {
        return VrfProof([0u8; 32]);
    }
}

pub type VrfOutput =  H256;


//TODO: Replace it with a real vrf functionalities
/// This produces a random output along with a proof
pub fn vrf_evaluate(pubkey: &VrfPublicKey, secret_key: &VrfSecretKey, input: &VrfInput) -> (VrfOutput, VrfProof) {
    //generating the hash using the public key for now. We need to use private key instead
    let raw_input = bincode::serialize(&input).unwrap();
    let raw_public_key = bincode::serialize(&pubkey.0).unwrap();
    let raw_combined = [&raw_input[..], &raw_public_key[..]].concat();
    let output: VrfOutput =
        ring::digest::digest(&ring::digest::SHA256, &raw_combined).into();
    return (output, VrfProof::default());
}

/// This checks if the random output produced by pubkey is valid.
pub fn vrf_check(pubkey: &VrfPublicKey, input: VrfInput, output: VrfOutput, proof: VrfProof) -> bool {

    let raw_input = bincode::serialize(&input).unwrap();
    let raw_public_key = bincode::serialize(&pubkey.0).unwrap();
    let raw_combined = [&raw_input[..], &raw_public_key[..]].concat();
    let output1: VrfOutput =
        ring::digest::digest(&ring::digest::SHA256, &raw_combined).into();
    return output1 == output;
}


