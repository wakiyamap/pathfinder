#![no_main]
use libfuzzer_sys::fuzz_target;
use std::convert::TryInto;

use pathfinder_lib::merkle_tree::MerkleTree;
use pedersen::StarkHash;
use rusqlite::Connection;

fuzz_target!(|data: &[u8]| {
    if data.len() == 0 {
        return;
    }
    if data.len() % 64 != 0 {
        return;
    }

    let kvs = data
        .chunks(32)
        .map(|chunk| StarkHash::from_be_bytes(chunk.try_into().unwrap()))
        .collect::<Result<Vec<_>, _>>();

    if kvs.is_err() {
        return;
    }

    let mut kvs = kvs.unwrap();

    const ZERO_HASH: StarkHash = StarkHash::zero();

    let mut conn = Connection::open_in_memory().unwrap();
    let transaction = conn.transaction().unwrap();

    let mut uut = MerkleTree::load("test".to_string(), &transaction, ZERO_HASH).unwrap();

    assert_eq!(kvs.len() % 2, 0);
    assert!(kvs.len() >= 2);

    {
        let value = kvs.pop().unwrap();
        let address = kvs.pop().unwrap();
        uut.set(address, value).expect("how could this fail?");
        let root = uut.commit().unwrap();
        uut = MerkleTree::load("test".to_string(), &transaction, root).unwrap();
    }

    while !kvs.is_empty() {
        let value = kvs.pop().unwrap();
        let address = kvs.pop().unwrap();
        uut.set(address, value).expect("how could this fail?");
    }

    uut.commit().unwrap();
});
