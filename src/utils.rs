use near_sdk::borsh::BorshSerialize;
use near_sdk::Gas;
use workspaces::prelude::*;

pub async fn run_contract<T: BorshSerialize, F: FnMut() -> Vec<T>>(wasm_file: &str, function: &str, mut f: F) -> Gas {
    let worker = workspaces::sandbox();
    let contract = worker.dev_deploy(std::fs::read(wasm_file).unwrap()).await.unwrap();

    let mut total_gas = Gas(0);

    for _ in 0..24 {
        // Randomize the amount of elements generated with unstructured.
        // Uses a slice of a random length from 0 to randomness buffer size
        let ops = f();
        // Call method with data
        let outcome = contract
            .call(&worker, function.to_string())
            .with_args(ops.try_to_vec().unwrap())
            .with_gas(10_000_000_000_000_0) // Default * 10
            .transact()
            .await
            .unwrap();

        let gas_burnt = outcome.total_gas_burnt;
        total_gas += Gas(gas_burnt);
        println!("outcome: {:?} {}", outcome.status, gas_burnt);
    }

    total_gas
}
