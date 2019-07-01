use rocksdb::{DB, Options, SliceTransform, BlockBasedOptions, BlockBasedIndexType};
use rand::Rng;
use std::time::Instant;

const SIZE: usize = 1000000;
fn main() {
    let mut rng = rand::thread_rng();
    let mut random_bytes: Vec<Vec<u8>> = vec![];
    for _ in 0..SIZE {
        let r: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        random_bytes.push(r);
    }
    /*
    let path = "/tmp/compare_direct_db";
    {
        DB::destroy(&Options::default(), &path).unwrap();
        let db = DB::open_default(&path).unwrap();
        for bytes in random_bytes.iter() {
            db.put(bytes, bytes).unwrap();
        }
        let start = Instant::now();
        {
            for bytes in random_bytes.iter() {
                db.get(bytes).unwrap();
            }
        }
        let end = Instant::now();
        let time = end.duration_since(start).as_micros();
        println!("Get time {}", time);
    }
    */
    let path = "/tmp/direct_db_1";
    {
        DB::destroy(&Options::default(), &path).unwrap();
        let mut opts = Options::default();
        opts.set_prefix_extractor( SliceTransform::create_fixed_prefix(32));
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_block_size(1<< 22);
        block_opts.set_index_type(BlockBasedIndexType::HashSearch);
        opts.set_block_based_table_factory(&block_opts);
        
        opts.create_if_missing(true);
        opts.optimize_for_point_lookup(256);
        let db = DB::open(&opts, &path).unwrap();
        for bytes in random_bytes.iter() {
            db.put(bytes, bytes).unwrap();
        }
        let start = Instant::now();
        {
            for bytes in random_bytes.iter() {
                db.get(bytes).unwrap();
            }
        }
        let end = Instant::now();
        let time = end.duration_since(start).as_micros();
        println!("Get time {}", time);
    }
    let path = "/tmp/direct_db_2";
    {
        DB::destroy(&Options::default(), &path).unwrap();
        let mut opts = Options::default();
        opts.set_prefix_extractor( SliceTransform::create_fixed_prefix(32));
        opts.create_if_missing(true);
        opts.optimize_for_point_lookup(256);
        let db = DB::open(&opts, &path).unwrap();
        for bytes in random_bytes.iter() {
            db.put(bytes, bytes).unwrap();
        }
        let start = Instant::now();
        {
            for bytes in random_bytes.iter() {
                db.get(bytes).unwrap();
            }
        }
        let end = Instant::now();
        let time = end.duration_since(start).as_micros();
        println!("Get time {}", time);
    }
}
