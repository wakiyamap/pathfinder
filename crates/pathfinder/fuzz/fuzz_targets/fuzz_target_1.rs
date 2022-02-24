#![no_main]
use std::collections::HashMap;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use pathfinder_lib::state::merkle_tree::MerkleTree;
use pedersen::StarkHash;
use std::cell::RefCell;

#[derive(Arbitrary, Debug)]
enum Command {
    // Commit,
    Next(FuzzedStarkHash, FuzzedStarkHash),
    RemoveAny,
}

struct FuzzedStarkHash(StarkHash);

impl std::fmt::Debug for FuzzedStarkHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <StarkHash as std::fmt::Debug>::fmt(&self.0, f)
    }
}

impl<'a> arbitrary::Arbitrary<'a> for FuzzedStarkHash {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let needed = u.len().max(32).min(1);

        for needed in (1..=needed).rev() {
            let peeked = if let Some(p) = u.peek_bytes(needed) {
                p
            } else {
                continue;
            };

            if let Ok(s) = StarkHash::from_be_slice(peeked) {
                u.bytes(needed).unwrap();
                return Ok(FuzzedStarkHash(s));
            }
        }

        Err(arbitrary::Error::IncorrectFormat)
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        let _ = depth;
        (1, Some(32))
    }
}

impl From<FuzzedStarkHash> for StarkHash {
    fn from(f: FuzzedStarkHash) -> Self {
        f.0
    }
}

fuzz_target!(|cmds: Vec<Command>| {
    const ZERO_HASH: StarkHash = StarkHash::ZERO;

    let mut uut = MerkleTree::<RefCell<HashMap<_, _>>>::default();

    let mut h = HashMap::new();

    for cmd in cmds {
        match cmd {
            // Command::Commit => {
            //     let root = uut.commit().unwrap();
            //     uut = MerkleTree::load("test".to_string(), &transaction, root).unwrap();
            // }
            Command::Next(key, value) => {
                let key = StarkHash::from(key);
                let value = StarkHash::from(value);

                uut.set(key, value).unwrap();
                h.insert(key, value);
            }
            Command::RemoveAny => {
                if let Some(key) = h.keys().next().copied() {
                    h.remove(&key).unwrap();
                    uut.set(key, ZERO_HASH).unwrap();
                }
            }
        }
    }

    // the root is not needed, but we want to fully build the tree
    let _ = uut.commit_mut().unwrap();

    uut.visit_leaves(|key, value| {
        let found = h.remove(key);
        assert_eq!(found.as_ref(), Some(value));
        assert_ne!(value, &ZERO_HASH);
    })
    .unwrap();

    for (key, value) in h {
        assert_eq!(
            value, ZERO_HASH,
            "any remaining keys in hashmap should had been zero, for key: {key:x}",
        );
    }
});
