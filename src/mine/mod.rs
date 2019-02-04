use crate::block;
use std::thread;
use std::sync::{mpsc, Arc, RwLock};

const NUM_THREADS: u32 = 4;
const PROPOSAL_THLD: block::hash::Hash 

pub fn mine(voter_hash: &block::hash::Hash, proposal_hash: &block::hash::Hash, transactions_hash: &block::hash::Hash) -> block::block_header::BlockHeader {
    let done = Arc::new(RwLock::new(false)); // to tell threads to stop
    let (tx, rx) = mpsc::channel(); // chan to collect computed nonce
    let mut thread_handles = Vec::new();
    for thread_idx in 0..NUM_THREADS {
        // each thread uses a step of four
        // e.g. the 3rd thread tries 3, 7, 11, ...
        let range = (thread_idx..std::u32::MAX).step_by(NUM_THREADS as usize);
        // create a copy of those variables for every thread
        let threshold = block::BlockHash(thld.0);
        let mut block_to_test = block::Block {
            parent: block::BlockHash(parent_hash.0),
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
                if hash < threshold {
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
    for handle in thread_handles { // wait for threads to stop
        handle.join().unwrap();
    }
    let mined = block::Block {
        parent: block::BlockHash(parent_hash.0),
        nonce: received,
    };
    return mined;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block;

    #[test]
    fn test_mining() {
        let parent_hash = block::BlockHash([10; 32]);
        let mut threshold = block::BlockHash([0; 32]);
        threshold.0[1] = 50;
        let mined = mine(&parent_hash, &threshold);
        assert_eq!(mined.hash() < threshold, true);
    }
}
