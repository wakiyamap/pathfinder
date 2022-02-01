#![no_main]
use libfuzzer_sys::fuzz_target;

use pedersen::{
    hash::{pedersen_hash, pedersen_hash_preprocessed},
    StarkHash,
};

fuzz_target!(|data: &[u8]| {
    if data.len() != 64 {
        return;
    }

    let a = StarkHash::from_be_slice(&data[..32]);
    let b = StarkHash::from_be_slice(&data[32..]);

    if let Ok(a) = a {
        if let Ok(b) = b {
            // the other impl was removed
            // assert_eq!(pedersen_hash(a, b), pedersen_hash_preprocessed(a, b));
        }
    }
});
