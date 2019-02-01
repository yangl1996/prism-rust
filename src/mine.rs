use crate::block;

pub fn mine(block: &mut block::Block, thld: &block::BlockHash) {
    for nonce in 0..std::u32::MAX {
        block.nonce = nonce;
        let hash = block.hash();
        if hash < *thld {
            return;
        }
    }
    // TODO: we should not arrive here
    return;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block;

    #[test]
    fn test_mining() {
        let mut block = block::Block {
            parent: block::BlockHash([10; 32]),
            nonce: 12345,
        };
        let mut threshold = block::BlockHash([0; 32]);
        threshold.0[1] = 50;
        mine(&mut block, &threshold);
        assert_eq!(block.hash() < threshold, true);
    }
}
