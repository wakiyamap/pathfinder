#![allow(dead_code)]
use goose::prelude::*;

async fn syncing(user: &mut GooseUser) -> GooseTaskResult {
    let json = &serde_json::json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": "starknet_syncing",
    });

    user.post_json("", json).await?;

    Ok(())
}

async fn transaction_hash(user: &mut GooseUser) -> GooseTaskResult {
    let json = &serde_json::json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": "starknet_getTransactionByHash",
        "params": {
            "transaction_hash": "0x6bc8a636965aabff8637eba5df9775bfe79858a51458dbcf8c6d55d584e90f1",
        }
    });

    user.post_json("", json).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_taskset(taskset!("pathfinder").register_task(task!(transaction_hash)))
        .execute()
        .await?
        .print();

    Ok(())
}
