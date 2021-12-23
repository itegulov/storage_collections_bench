#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
pub mod test {
    use std::future::join;
    use near_sdk::borsh::BorshSerialize;
    use near_sdk::Gas;
    use workspaces::prelude::*;
    use workspaces::{Contract, Network, Worker};

    async fn run_contract<T: BorshSerialize, F: FnMut() -> Vec<T>, I: Network>(
        contract: &Contract,
        worker: &Worker<I>,
        function: &str,
        ops: &Vec<T>
    ) -> Gas {
        let outcome = contract
            .call(&worker, function)
            .args(ops.try_to_vec().unwrap())
            .gas(300_000_000_000_000) // 300 TGas
            .transact()
            .await
            .unwrap();
        println!("outcome: {:?}", outcome);
        return Gas(outcome.total_gas_burnt);
    }

    pub async fn run_contract_dual_function<T: BorshSerialize, F: FnMut() -> Vec<T>>(
        wasm_file: &str,
        function1: &str,
        function2: &str,
        mut generate_ops: F
    ) -> (Gas, Gas) {
        let worker1 = workspaces::sandbox();
        let contract1 = worker1.dev_deploy(std::fs::read(wasm_file).unwrap()).await.unwrap();
        let worker2 = workspaces::sandbox();
        let contract2 = worker2.dev_deploy(std::fs::read(wasm_file).unwrap()).await.unwrap();

        let mut total_gas_1 = Gas(0);
        let mut total_gas_2 = Gas(0);

        for _ in 0..24 {
            // Randomize the amount of elements generated with unstructured.
            // Uses a slice of a random length from 0 to randomness buffer size
            let ops = generate_ops();
            let gas_fut_1 = run_contract::<T, F, _>(&contract1, &worker1, function1, &ops);
            let gas_fut_2 = run_contract::<T, F, _>(&contract2, &worker2, function2, &ops);
            let (gas_1, gas_2) = join!(gas_fut_1, gas_fut_2).await;
            total_gas_1 += gas_1;
            total_gas_2 += gas_2;
        }

        (total_gas_1, total_gas_2)
    }
}
