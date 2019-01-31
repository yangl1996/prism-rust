mod block;
use block::BlockHash;

fn main() {
    let test = BlockHash([0; 32]);
    println!("{}", test);
}
