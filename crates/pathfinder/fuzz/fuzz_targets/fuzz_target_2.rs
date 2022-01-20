#![no_main]
use libfuzzer_sys::fuzz_target;
use std::convert::TryInto;

use pedersen::{pedersen_hash, StarkHash};

fuzz_target!(|data: &[u8]| {
    if data.len() != 64 {
        return;
    }

    let first = StarkHash::from_be_bytes(data[..32].try_into().unwrap());
    let second = StarkHash::from_be_bytes(data[32..].try_into().unwrap());

    if let Ok(first) = first {
        if let Ok(second) = second {
            pedersen_hash(first, second);
        }
    }
});
