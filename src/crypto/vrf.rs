use super::hash::{Hashable, H256};
use ed25519_dalek::{PublicKey,SecretKey};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct VrfPublicKey([u8; 32]); //TODO: We are using a fake public key for now

pub type VrfSecretKey = SecretKey;

#[derive(Serialize, Deserialize, Clone)]
pub struct VrfInput{
    random_source: [u8; 32],
    time: [u8; 16],
} //Random source and time


#[derive(Serialize, Deserialize, Clone)]
pub struct VrfProof ([u8; 32]);

impl VrfProof{
    fn default() -> Self {
        return VrfProof([0u8; 32]);
    }
}

pub type VrfOutput =  H256;


pub fn vrf_evaluate(pubkey: &VrfPublicKey, secret_key: &VrfSecretKey, input: &VrfInput) -> (VrfOutput, VrfProof) {
    //TODO: Replace it with a real vrf function
    //generating the hash using the public key for now
    let raw_input_1 = bincode::serialize(&input.random_source).unwrap();
    let raw_input_2 = bincode::serialize(&input.time).unwrap();
    let raw_public_key = bincode::serialize(&pubkey.0).unwrap();
    let raw_combined = [&raw_input_1[..], &raw_input_2[..], &raw_public_key[..]].concat();
    let output: VrfOutput =
        ring::digest::digest(&ring::digest::SHA256, &raw_combined).into();
    return (output, VrfProof::default());
}

pub fn vrf_check(pubkey: &VrfPublicKey, input: VrfInput, output: VrfOutput, proof: VrfProof) -> bool {

    let raw_input_1 = bincode::serialize(&input.random_source).unwrap();
    let raw_input_2 = bincode::serialize(&input.time).unwrap();
    let raw_public_key = bincode::serialize(&pubkey.0).unwrap();
    let raw_combined = [&raw_input_1[..], &raw_input_2[..], &raw_public_key[..]].concat();
    let output1: VrfOutput =
        ring::digest::digest(&ring::digest::SHA256, &raw_combined).into();
    return output1 == output;
}


