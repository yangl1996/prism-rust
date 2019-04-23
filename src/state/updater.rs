use std::sync::mpsc::{Receiver};
use crate::crypto::hash::H256;
use crate::blockchain::transaction::UpdateMessage as LedgerUpdateMessage;
use crate::blockdb::BlockDatabase;
use std::sync::{Mutex, Arc};
use crate::state::UTXODatabase;
use crate::wallet::Wallet;
use crate::handler;
use std::thread;

// do we need a handler?
pub struct Context {
    blockdb: Arc<BlockDatabase>,
    utxodb: Arc<UTXODatabase>, //do we need a mutex here?
    wallets: Arc<Vec<Mutex<Wallet>>>,
    state_update_source: Receiver<(LedgerUpdateMessage, Vec<H256>)>,
}

pub fn new(
    blockdb: &Arc<BlockDatabase>,
    utxodb: &Arc<UTXODatabase>, //do we need a mutex here?
    wallets: &Arc<Vec<Mutex<Wallet>>>,
    state_update_source: Receiver<(LedgerUpdateMessage, Vec<H256>)>,
) -> (Context) {
    let ctx = Context {
        blockdb: Arc::clone(blockdb),
        utxodb: Arc::clone(utxodb),
        wallets: Arc::clone(wallets),
        state_update_source: state_update_source,
    };

    return (ctx);
}

impl Context {
    pub fn start(mut self) {
        println!("State updater started");
        thread::Builder::new().name("state updater".to_string()).spawn(move || {
            self.updater_loop();
        }).unwrap();// do we need this unwrap?
    }

    fn updater_loop(&self) {
        loop {
            if let Ok((signal, hashes)) = self.state_update_source.recv() {
                match signal {
                    LedgerUpdateMessage::Add => {
                        handler::confirm_new_tx_block_hashes(hashes, &self.blockdb, &self.utxodb, &self.wallets);
                    },
                    LedgerUpdateMessage::Rollback => {
                        handler::unconfirm_old_tx_block_hashes(hashes, &self.blockdb, &self.utxodb, &self.wallets);
                    },
                }
            }
        }
    }
}
