#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
pub mod test {
    use near_sdk::borsh::BorshSerialize;
    use near_sdk::Gas;
    use workspaces::prelude::*;

    pub async fn run_contract_dual_function<T: BorshSerialize, F: FnMut() -> Vec<T>>(
        wasm_file: &str,
        function1: &str,
        function2: &str,
        mut generate_ops: F
    ) -> (Gas, Gas) {
        let worker = workspaces::sandbox();
        let contract = worker.dev_deploy(std::fs::read(wasm_file).unwrap()).await.unwrap();

        let mut total_gas_1 = Gas(0);
        let mut total_gas_2 = Gas(0);

        for _ in 0..24 {
            // Randomize the amount of elements generated with unstructured.
            // Uses a slice of a random length from 0 to randomness buffer size
            let ops = generate_ops();
            // Call method with data
            let outcome1 = contract
                .call(&worker, function1.to_string())
                .with_args(ops.try_to_vec().unwrap())
                .with_gas(300_000_000_000_000) // 300 TGas
                .transact()
                .await
                .unwrap();

            total_gas_1 += Gas(outcome1.total_gas_burnt);
            println!("outcome: {:?}", outcome1);

            let outcome2 = contract
                .call(&worker, function2.to_string())
                .with_args(ops.try_to_vec().unwrap())
                .with_gas(300_000_000_000_000) // 300 TGas
                .transact()
                .await
                .unwrap();

            total_gas_2 += Gas(outcome2.total_gas_burnt);
            println!("outcome: {:?}", outcome2);
        }

        (total_gas_1, total_gas_2)
    }
}
