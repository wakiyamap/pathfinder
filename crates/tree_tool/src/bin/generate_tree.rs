use pathfinder_lib::state::merkle_tree::MerkleTree;
use rusqlite::Connection;
use stark_hash::{stark_hash, StarkHash};
use std::io::BufRead;
use std::sync::Arc;
use web3::types::U256;

const ZERO_HASH: StarkHash = StarkHash::ZERO;

fn main() {
    let mut args = std::env::args().fuse();

    let name = args.next().expect("unsupported environment");
    let choice = args.next();
    let mut choice = choice.as_deref();
    let extra = args.next();

    if extra.is_some() {
        choice = None;
    }

    let parse = if choice == Some("global") {
        parse_global
    } else if choice == Some("storage") {
        parse_storage
    } else {
        if let Some(other) = choice {
            eprintln!(
                r#"Argument needs to be "storage" or "global", not {:?}"#,
                other
            );
        } else if extra.is_some() {
            eprintln!(r"Too many arguments");
        } else {
            eprintln!(
                r#"USAGE:
- echo "1 2 3" | cargo run -p tree_tool --bin {0} global
- echo "1 2" | cargo run -p tree_tool --bin {0} storage"#,
                name
            );
        }
        std::process::exit(1);
    };

    #[derive(Clone)]
    enum Message {
        Insert(Arc<(StarkHash, StarkHash)>),
        Fin(Arc<StarkHash>),
    }

    use Message::*;

    // uncomment the fibonacci to to have commits every nth row
    let txs = [/*1, 2, 3, 5, 7, 11*/]
        .into_iter()
        .map(|every: usize| {
            let (tx, rx) = std::sync::mpsc::channel();
            let jh = std::thread::spawn(move || {
                let name = format!("every_{}", every);
                let mut conn = Connection::open_in_memory().unwrap();
                let transaction = conn.transaction().unwrap();
                let mut root = ZERO_HASH;
                let mut uut = MerkleTree::load(name.clone(), &transaction, root).unwrap();

                let mut batch = 0;
                let mut total_commits = 0;

                for msg in rx {
                    match msg {
                        Insert(tuple) => {
                            uut.set(tuple.0, tuple.1).unwrap();
                            batch += 1;
                            if batch == every {
                                root = uut.commit().unwrap();
                                uut = MerkleTree::load(name.clone(), &transaction, root).unwrap();
                                batch = 0;
                                total_commits += 1;
                            }
                        }
                        Fin(root_hash) => {
                            root = uut.commit().unwrap();
                            if root != *root_hash {
                                panic!("Committing every {} produced a different hash (total commits: {})\nExpected: {:?}, Got: {:?}", every, total_commits, root_hash, root);
                            }
                            break;
                        }
                    }
                }
            });
            (tx, jh)
        })
        .collect::<Vec<_>>();

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
                // allow comments and empty lines for clearer examples
                continue;
            }

            let (key, value) = parse(buffer);

            uut.set(key, value).expect("how could this fail");

            let msg = Insert(Arc::new((key, value)));

            txs.iter().for_each(|(tx, _)| {
                let _ = tx.send(msg.clone());
            });
        }

        let root = uut.commit().unwrap();
        transaction.commit().unwrap();

        let msg = Fin(Arc::new(root));

        txs.iter().for_each(|(tx, _)| {
            let _ = tx.send(msg.clone());
        });

        if !txs.into_iter().map(|(_, jh)| jh.join().is_ok()).all(|x| x) {
            eprintln!("some threads failed, we got: {:?}", root);
            std::process::exit(1);
        }

        root
    };

    println!("{:?}", root);

    if std::env::var_os("TREE_TOOL_SUPPRESS_NODES").is_none() {
        dump(&mut conn, "test");
    }
}

fn dump(conn: &mut Connection, name: &str) {
    let tx = conn.transaction().unwrap();
    let mut stmt = tx
        .prepare(format!("select hash, data from {}", name).as_str())
        .unwrap();
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

fn parse_global(buffer: &str) -> (StarkHash, StarkHash) {
    let (contract_address, buffer) = buffer
        .split_once(' ')
        .expect("expected 3 values, whitespace separated; couldn't find first space");

    let contract_address = parse(contract_address)
        .unwrap_or_else(|| panic!("invalid contract_address: {:?}", contract_address));

    let buffer = buffer.trim();
    let (contract_hash, buffer) = buffer
        .split_once(' ')
        .expect("expected 3 values, whitespace separated; couldn't find second space");

    let contract_hash = parse(contract_hash)
        .unwrap_or_else(|| panic!("invalid contract_hash: {:?}", contract_hash));

    let contract_commitment_root = buffer.trim();
    let contract_commitment_root =
        parse(contract_commitment_root).unwrap_or_else(|| panic!("invalid value: {:?}", buffer));

    let value = stark_hash(contract_hash, contract_commitment_root);
    let value = stark_hash(value, ZERO_HASH);
    let value = stark_hash(value, ZERO_HASH);

    // python side does make sure every key is unique before asking the tree code to
    // process it
    (contract_address, value)
}

fn parse_storage(buffer: &str) -> (StarkHash, StarkHash) {
    // here we read just address = value
    // but there's no such thing as splitting whitespace \s+ which I think is what the
    // python side is doing so lets do it like this for a close approximation

    let (address, buffer) = buffer.split_once(' ').expect("expected 2 values per line");

    let address = parse(address).unwrap_or_else(|| panic!("invalid address: {:?}", address));

    let buffer = buffer.trim();
    let value = parse(buffer).unwrap_or_else(|| panic!("invalid value: {:?}", buffer));

    (address, value)
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
