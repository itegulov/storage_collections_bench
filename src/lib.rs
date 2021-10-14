use arbitrary::Arbitrary;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::store::LookupMap;

#[derive(BorshDeserialize, BorshSerialize, Ord, PartialOrd, Eq, PartialEq, Arbitrary, Clone)]
pub struct StackHeapMock {
    a: Option<u128>,
    b: Option<[u8; 32]>,
    c: Vec<u8>,
}

type KeyType = StackHeapMock;
type ValueType = StackHeapMock;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LookupBench {
    map: LookupMap<KeyType, ValueType>,
}

impl Default for LookupBench {
    fn default() -> Self {
        Self {
            map: LookupMap::new(b"m"),
        }
    }
}

#[derive(Arbitrary, Debug, BorshDeserialize, BorshSerialize)]
pub enum Action<K, V> {
    Insert(K, V),
    Set(K, Option<V>),
    Remove(K),
    Flush,
    Get(K),
}

#[near_bindgen]
impl LookupBench {
    pub fn fuzz(&mut self, #[serializer(borsh)] actions: Vec<Action<KeyType, ValueType>>) {
        let lm = &mut self.map;
        for op in actions {
            match op {
                Action::Insert(k, v) => {
                    let _r1 = lm.insert(k, v);
                }
                Action::Set(k, v) => {
                    lm.set(k, v);
                }
                Action::Remove(k) => {
                    let _r1 = lm.remove(&k);
                }
                Action::Flush => {
                    lm.flush();
                }
                Action::Get(k) => {
                    let _r1 = lm.get(&k);
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::{Arbitrary, Unstructured};
    use rand::SeedableRng;
    use rand::{Rng, RngCore};
    use runner;

    const BUFFER_SIZE: usize = 4096;

    async fn fuzz_contract(wasm_file: &str) -> u128 {
        let (contract_id, signer) = runner::dev_deploy(wasm_file).await.unwrap();

        let mut total_gas: u128 = 0;

        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];
        for _ in 0..24 {
            rng.fill_bytes(&mut buf);

            // Randomize the amount of elements generated with unstructured.
            // Uses a slice of a random length from 0 to randomness buffer size
            let u = Unstructured::new(&buf[0..(rng.gen::<usize>() % BUFFER_SIZE)]);
            if let Ok(ops) = Vec::<Action<KeyType, ValueType>>::arbitrary_take_rest(u) {
                // Call method with data
                let outcome = runner::call(
                    &signer,
                    contract_id.clone(),
                    contract_id.clone(),
                    "fuzz".to_string(),
                    ops.try_to_vec().unwrap(),
                    None,
                    // Default * 10
                    Some(10_000_000_000_000_0),
                )
                .await
                .unwrap();

                let gas_burnt = outcome.transaction_outcome.outcome.gas_burnt as u128;
                total_gas += gas_burnt;
                println!("{:?} {}", outcome.status, gas_burnt);
            }
        }

        total_gas
    }
    
    #[runner::test]
    async fn hashing_fuzz() {
        assert_eq!(fuzz_contract("./collections_bench-HASHING.wasm").await, 58524904330838);
    }
}
