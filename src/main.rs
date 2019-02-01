mod block;

fn main() {
    let block = block::Block {
        parent: block::BlockHash([10; 32]),
        nonce: 12345,
    };
    println!("{}", block);
    let serialized = block.serialized();
    for i in 0..36 {
        print!("{:>02x}", serialized[i]);
    }
}
