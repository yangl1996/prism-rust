mod block;

use block::{Block, BlockHash};

pub fn mine(&mut block::Block block, &block::BlockHash thld) {
    for nonce in 0..std::u32::MAX {
        block.nonce = nonce;
        let hash = block.hash();
        if hash < thld {
            return;
        }
    }
    // TODO: we should not arrive here
    return;
}
