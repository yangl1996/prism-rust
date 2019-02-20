use crate::block::block_header;
use crate::hash::{self, Hashable};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

const NUM_THREADS: u32 = 4;
const PROPOSAL_THLD: hash::Hash = hash::Hash(hex!(
    "0001000000000000000000000000000000000000000000000000000000000000"
));

pub fn mine(
    voter_hash: &hash::Hash,
    proposal_hash: &hash::Hash,
    transactions_hash: &hash::Hash,
) -> u32 {
    let done = Arc::new(RwLock::new(false)); // to tell threads to stop
    let (tx, rx) = mpsc::channel(); // chan to collect computed nonce
    let mut thread_handles = Vec::new();
    for thread_idx in 0..NUM_THREADS {
        // each thread uses a step of four
        // e.g. the 3rd thread tries 3, 7, 11, ...
        let range = (thread_idx..std::u32::MAX).step_by(NUM_THREADS as usize);
        let mut block_to_test = block_header::BlockHeader {
            voter_hash: voter_hash.clone(),
            proposal_hash: proposal_hash.clone(),
            transactions_hash: transactions_hash.clone(),
            nonce: 0,
        };
        let tx = mpsc::Sender::clone(&tx);
        let done = done.clone();
        let handle = thread::spawn(move || {
            for nonce in range {
                {
                    let done = done.read().unwrap();
                    if *done == true {
                        return;
                    }
                }
                block_to_test.nonce = nonce;
                let hash = block_to_test.hash();
                if hash < PROPOSAL_THLD {
                    match tx.send(nonce) {
                        Ok(()) => return,
                        Err(_) => return, // just ignore senderror
                    }
                }
            }
            return;
        });
        thread_handles.push(handle);
    }
    let received = rx.recv().unwrap(); // if error, just panic here
    {
        let mut done = done.write().unwrap(); // tell threads to stop
        *done = true;
    }
    for handle in thread_handles {
        // wait for threads to stop
        handle.join().unwrap();
    }
    return received;
}

#[cfg(test)]
mod tests {
    use super::mine;
    use super::PROPOSAL_THLD;
    use crate::block::block_header;
    use crate::hash::{self, Hashable};

    #[test]
    fn mining() {
        let voter_hash = hash::Hash(hex!(
            "0101010101010101010101010101010101010101010101010101010101010101"
        ));
        let proposal_hash = hash::Hash(hex!(
            "0101010101010101010101010101010101010101010101010101010101010101"
        ));
        let transactions_hash = hash::Hash(hex!(
            "0101010101010101010101010101010101010101010101010101010101010101"
        ));
        let mined_nonce = mine(&voter_hash, &proposal_hash, &transactions_hash);
        let mined = block_header::BlockHeader {
            voter_hash: voter_hash,
            proposal_hash: proposal_hash,
            transactions_hash: transactions_hash,
            nonce: mined_nonce,
        };
        assert_eq!(mined.hash() < PROPOSAL_THLD, true);
    }
}
