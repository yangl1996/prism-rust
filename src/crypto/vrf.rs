use super::hash::{Hashable, H256};
use ed25519_dalek::{PublicKey,SecretKey};
use serde::{Serialize, Deserialize};
use crate::block::pos_metadata::{Metadata, TimeStamp, RandomSource};
use crate::transaction::CoinId;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Hash)]
pub struct VrfPublicKey([u8; 32]); //TODO: We are using a fake public key for now

impl Hashable for VrfPublicKey {
    fn hash(&self) -> H256 {
        return ring::digest::digest(&ring::digest::SHA256, &self.0).into();
    }
}

// Now VrfPublicKey and PublicKey are essentially the same, so we can convert
impl std::convert::From<&PublicKey> for VrfPublicKey {
    fn from(other: &PublicKey) -> Self {
        VrfPublicKey(other.to_bytes())
    }
}

// Now VrfPublicKey and PublicKey are essentially the same, so we can convert
impl std::convert::From<&VrfPublicKey> for PublicKey {
    fn from(other: &VrfPublicKey) -> Self {
        PublicKey::from_bytes(&other.0).unwrap()
    }
}

pub type VrfSecretKey = SecretKey;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Hash)]
pub struct VrfInput {
    pub random_source: RandomSource,
    pub time: TimeStamp,
    pub coin: CoinId,
}

impl std::convert::From<&Metadata> for VrfInput {
    fn from(other: &Metadata) -> VrfInput {
        VrfInput {
            random_source: other.parent_random_source.clone(),
            time: other.timestamp,
            coin: other.utxo.coin.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Hash    )]
pub struct VrfProof ([u8; 32]);

pub type VrfValue =  H256;


//TODO: Replace it with a real vrf functionalities
/// This produces a random output along with a proof
pub fn vrf_evaluate(public_key: &VrfPublicKey, secret_key: &VrfSecretKey, input: &VrfInput) -> (VrfValue, VrfProof) {
    //generating the hash using the public key for now. We need to use private key instead
    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
    ctx.update(&input.random_source);
    ctx.update(&input.time.to_be_bytes());
    let raw: [u8;32] = input.coin.hash.into();
    ctx.update(&raw);
    ctx.update(&input.coin.index.to_be_bytes());
    ctx.update(&public_key.0);
    let digest = ctx.finish();
    let output: VrfValue = digest.into();
    return (output, VrfProof::default());
}

/// This checks if the random output produced by public_key is valid.
pub fn vrf_verify(public_key: &VrfPublicKey, input: &VrfInput, output: &VrfValue, proof: &VrfProof) -> bool {
    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
    ctx.update(&input.random_source);
    ctx.update(&input.time.to_be_bytes());
    let raw: [u8;32] = input.coin.hash.into();
    ctx.update(&raw);
    ctx.update(&input.coin.index.to_be_bytes());
    ctx.update(&public_key.0);
    let digest = ctx.finish();
    *output == digest.into()
}

