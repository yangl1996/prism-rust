use crate::blockchain::transaction::UpdateMessage as LedgerUpdateMessage;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::H256;
use crate::handler;
use crate::state::UTXODatabase;
use crate::wallet::Wallet;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

/// A state updater data struct holding blockdb, utxodb, wallets and a receiver.
/// When receiving state update message from ledger, which is the tx block hashes to be updated, it reads these blocks
/// from blockdb, extract the transactions, do the sanity check for them, then update the utxodb and wallets.
pub struct Context {
    blockdb: Arc<BlockDatabase>,
    utxodb: Arc<UTXODatabase>,
    wallets: Arc<Vec<Mutex<Wallet>>>,
    state_update_source: Receiver<(LedgerUpdateMessage, Vec<H256>)>,
}

pub fn new(
    blockdb: &Arc<BlockDatabase>,
    utxodb: &Arc<UTXODatabase>,
    wallets: &Arc<Vec<Mutex<Wallet>>>,
    state_update_source: Receiver<(LedgerUpdateMessage, Vec<H256>)>,
) -> (Context) {
    let ctx = Context {
        blockdb: Arc::clone(blockdb),
        utxodb: Arc::clone(utxodb),
        wallets: Arc::clone(wallets),
        state_update_source: state_update_source,
    };

    return ctx;
}

impl Context {
    /// Start a new thread that runs the loop keeping receiving state update message from ledger.
    pub fn start(self) {
        println!("State updater started");
        thread::Builder::new()
            .name("state updater".to_string())
            .spawn(move || {
                self.updater_loop();
            })
            .unwrap();
    }

    /// The loop keeping receiving state update message from ledger.
    fn updater_loop(&self) {
        loop {
            if let Ok((signal, hashes)) = self.state_update_source.recv() {
                match signal {
                    LedgerUpdateMessage::Add => {
                        handler::confirm_new_tx_block_hashes(
                            hashes,
                            &self.blockdb,
                            &self.utxodb,
                            &self.wallets,
                        );
                    }
                    LedgerUpdateMessage::Rollback => {
                        handler::unconfirm_old_tx_block_hashes(
                            hashes,
                            &self.blockdb,
                            &self.utxodb,
                            &self.wallets,
                        );
                    }
                }
            }
        }
    }
}
