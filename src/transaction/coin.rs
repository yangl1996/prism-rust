use super::transaction::*;

pub struct Coin{
    pubkey: PubKey,
    nonce: Nonce,
    value: u32,
}