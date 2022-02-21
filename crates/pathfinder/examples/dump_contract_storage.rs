use std::path::PathBuf;

use pedersen::StarkHash;

fn main() {
    let me = std::env::args()
        .nth(0)
        .unwrap_or_else(|| String::from("dump_contract_storage"));
    let args = std::env::args().count();
    if args < 3 || args > 4 {
        eprintln!("USAGE: {me} DB_FILE ROOT_HASH CONTRACT_ADDRESS?");
        eprintln!("ROOT_HASH and CONTRACT_ADDRESS are both in non-prefixed hex format.");
        eprintln!("If CONTRACT_ADDRESS is not given, the contract addresses of the global tree are instead dumped.");
        std::process::exit(1);
    }

    let path = std::env::args().nth(1).unwrap();

    let mut it = std::env::args()
        .skip(2)
        .map(|s| StarkHash::from_hex_str(&s))
        .fuse();

    let root_hash = it.next().unwrap().expect("Invalid root hash");
    let contract_address = it.next().map(|res| res.expect("Invalid contract address"));

    let storage =
        pathfinder_lib::storage::Storage::migrate(PathBuf::from(path)).expect("Migration failed");

    let mut conn = storage.connection().unwrap();
    let tx = conn.transaction().unwrap();

    let global = pathfinder_lib::state::merkle_tree::MerkleTree::load(
        "tree_global".to_owned(),
        &tx,
        root_hash,
    )
    .expect("Tree load failed");

    if let Some(contract_address) = contract_address {
        let contract_state = global.get(contract_address).unwrap();

        assert_ne!(contract_state, StarkHash::ZERO, "no such contract address");

        let contract_root = pathfinder_lib::storage::ContractsStateTable::get_root(
            &tx,
            pathfinder_lib::core::ContractStateHash(contract_state),
        )
        .unwrap()
        .expect("No such contract_root");

        let tree = pathfinder_lib::state::merkle_tree::MerkleTree::load(
            "tree_contracts".to_owned(),
            &tx,
            contract_root.0,
        )
        .unwrap();

        tree.visit_leaves(|k, v| println!("0x{k:x} 0x{v:x}"))
            .unwrap();
    } else {
        global
            .visit_leaves(|k, v| println!("0x{k:x} 0x{v:x}"))
            .unwrap();
    }
}
