use pathfinder_lib::state::merkle_tree::MerkleTree;
use rusqlite::Connection;
use stark_hash::StarkHash;
use std::io::BufRead;
use web3::types::U256;

fn main() {
    if std::env::args().skip(1).count() != 0 {
        let first = std::env::args().nth(0);
        eprintln!(
            r#"USAGE: echo "1 2" | cargo run --example {}"#,
            first.as_deref().unwrap_or("merkle_storage_tree")
        );
        return;
    }

    const ZERO_HASH: StarkHash = StarkHash::ZERO;

    let mut conn = Connection::open_in_memory().unwrap();

    let root = {
        let transaction = conn.transaction().unwrap();

        let mut uut = MerkleTree::load("test".to_string(), &transaction, ZERO_HASH).unwrap();

        let mut buffer = String::new();
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();

        loop {
            buffer.clear();
            let read = stdin.read_line(&mut buffer).unwrap();

            if read == 0 {
                break;
            }

            let buffer = buffer.trim();
            if buffer.is_empty() || buffer.chars().next() == Some('#') {
                // TODO: impl this to python side
                // allow comments and empty lines for clearer examples
                continue;
            }

            // here we read just address = value
            // but there's no such thing as splitting whitespace \s+ which I think is what the
            // python side is doing so lets do it like this for a close approximation

            let (address, buffer) = buffer.split_once(' ').expect("expected 2 values per line");

            let address =
                parse(address).unwrap_or_else(|| panic!("invalid address: {:?}", address));

            let buffer = buffer.trim();
            let value = parse(buffer).unwrap_or_else(|| panic!("invalid value: {:?}", buffer));

            uut.set(address, value).expect("how could this fail?");
        }

        let root = uut.commit().unwrap();

        transaction.commit().unwrap();
        root
    };

    println!("{:?}", Hex(root.as_ref()));

    let tx = conn.transaction().unwrap();
    let mut stmt = tx.prepare("select hash, data from test").unwrap();
    let mut res = stmt.query([]).unwrap();

    while let Some(row) = res.next().unwrap() {
        let hash = row.get_ref(0).unwrap().as_blob().unwrap();
        let data = row.get_ref(1).unwrap().as_blob().unwrap();

        if data.is_empty() {
            // this is a starknet_storage_leaf, and currently we don't have the contract state
            continue;
        }

        eprintln!("patricia_node:{:?} => {:?}", Hex(hash), Hex(data));
    }
}

fn parse(s: &str) -> Option<StarkHash> {
    if let Some(suffix) = s.strip_prefix("0x") {
        StarkHash::from_hex_str(suffix).ok()
    } else {
        let u = U256::from_dec_str(s).ok()?;
        let mut bytes = [0u8; 32];
        u.to_big_endian(&mut bytes);
        StarkHash::from_be_bytes(bytes).ok()
    }
}

struct Hex<'a>(&'a [u8]);

use std::fmt;

impl fmt::Debug for Hex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|&b| write!(f, "{:02x}", b))
    }
}
