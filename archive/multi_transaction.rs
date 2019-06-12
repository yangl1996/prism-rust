//use prism::wallet::Wallet;
//use prism::transaction::Input;
//use prism::transaction::tests::generate_random_coinid;
//use std::time::Instant;
//use prism::crypto::hash::{H256, Hashable};
//use prism::crypto::sign::KeyPair;
//#[test]
//fn multi_transaction() {
//    let kp = KeyPair::random();
//    let addr = kp.public_key().hash();
//    // give the test address 10 x 10 coins
//    let mut ico: Vec<Input> = vec![];
//    for _ in 0..10000 {
//        ico.push(
//            Input {
//                value: 1,
//                owner: addr,
//                coin: generate_random_coinid(),
//            });
//    }
//    {
//        let w = Wallet::new(std::path::Path::new("/tmp/walletdb_test.rocksdb")).unwrap();
//        w.load_keypair(KeyPair::from_pkcs8(kp.pkcs8_bytes.clone())).unwrap();
//
//        w.apply_diff(&ico, &[]).unwrap();
//        // generate transactions
//        let recipient = H256::default();
//
//        let start = Instant::now();
//        for _ in 0..1000 {
//            let _tx = w.create_transaction(recipient, 9).unwrap();
//        }
//        let end = Instant::now();
//        let time = end.duration_since(start).as_millis() as f64;
//        println!("Time single create\t{}", time);
//    }
//    {
//        let w = Wallet::new(std::path::Path::new("/tmp/walletdb_test.rocksdb")).unwrap();
//        w.load_keypair(KeyPair::from_pkcs8(kp.pkcs8_bytes.clone())).unwrap();
//
//        w.apply_diff(&ico, &[]).unwrap();
//        // generate transactions
//        let recipient = H256::default();
//
//        let start = Instant::now();
//        for _ in 0..100 {
//            let _txs = w.create_transactions(&[(recipient, 9);10]).unwrap();
//        }
//        let end = Instant::now();
//        let time = end.duration_since(start).as_millis() as f64;
//        println!("Time multi create\t{}", time);
//    }
//    {
//        let w = Wallet::new(std::path::Path::new("/tmp/walletdb_test.rocksdb")).unwrap();
//        w.load_keypair(KeyPair::from_pkcs8(kp.pkcs8_bytes.clone())).unwrap();
//
//        w.apply_diff(&ico, &[]).unwrap();
//        // generate transactions
//        let recipient = H256::default();
//
//        let start = Instant::now();
//        for _ in 0..10 {
//            let _txs = w.create_transactions(&[(recipient, 9);100]).unwrap();
//        }
//        let end = Instant::now();
//        let time = end.duration_since(start).as_millis() as f64;
//        println!("Time multi create\t{}", time);
//    }
//    {
//        let w = Wallet::new(std::path::Path::new("/tmp/walletdb_test.rocksdb")).unwrap();
//        w.load_keypair(KeyPair::from_pkcs8(kp.pkcs8_bytes.clone())).unwrap();
//
//        w.apply_diff(&ico, &[]).unwrap();
//        // generate transactions
//        let recipient = H256::default();
//
//        let start = Instant::now();
//        for _ in 0..1 {
//            let _txs = w.create_transactions(&[(recipient, 9);1000]).unwrap();
//        }
//        let end = Instant::now();
//        let time = end.duration_since(start).as_millis() as f64;
//        println!("Time multi create\t{}", time);
//    }
//}