#![feature(asm)]
#![feature(bench_black_box)]

use arbitrary::Arbitrary;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::store::LookupMap;
use std::hint::black_box;

#[derive(
    BorshDeserialize,
    BorshSerialize,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Arbitrary,
    Clone,
    Debug,
    Default,
)]
pub struct StackHeapMock {
    a: u128,
    b: Option<[u8; 32]>,
    c: Vec<u8>,
}

type KeyType = StackHeapMock;
type ValueType = StackHeapMock;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LookupMapBench {
    map: LookupMap<KeyType, ValueType>,
}

impl Default for LookupMapBench {
    fn default() -> Self {
        Self {
            map: LookupMap::new(b"m"),
        }
    }
}

#[derive(Arbitrary, Debug, BorshDeserialize, BorshSerialize)]
pub enum MapAction<K, V> {
    Insert(K, V),
    Set(K, Option<V>),
    Remove(K),
    Flush,
    Get(K),
}

#[near_bindgen]
impl LookupMapBench {
    pub fn fuzz(&mut self, #[serializer(borsh)] actions: Vec<MapAction<KeyType, ValueType>>) {
        let lm = &mut self.map;
        for op in actions {
            match op {
                MapAction::Insert(k, v) => {
                    let _r = black_box(lm.insert(k, v));
                }
                MapAction::Set(k, v) => {
                    lm.set(k, v);
                }
                MapAction::Remove(k) => {
                    let _r = black_box(lm.remove(&k));
                }
                MapAction::Flush => {
                    lm.flush();
                }
                MapAction::Get(k) => {
                    let _r = black_box(lm.get(&k));
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
    use near_sdk::Gas;
    use rand::SeedableRng;
    use rand::{Rng, RngCore};
    use workspaces::prelude::*;

    const BUFFER_SIZE: usize = 4096;

    async fn fuzz_contract(wasm_file: &str) -> u64 {
        let worker = workspaces::sandbox();
        let contract = worker.dev_deploy(std::fs::read(wasm_file).unwrap()).await.unwrap();

        let mut total_gas: Gas = Gas(0);

        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];
        for _ in 0..24 {
            rng.fill_bytes(&mut buf);

            // Randomize the amount of elements generated with unstructured.
            // Uses a slice of a random length from 0 to randomness buffer size
            let u = Unstructured::new(&buf[0..(rng.gen::<usize>() % BUFFER_SIZE)]);
            if let Ok(ops) = Vec::<MapAction<KeyType, ValueType>>::arbitrary_take_rest(u) {
                // Call method with data
                let outcome = contract
                    .call(&worker, "fuzz".to_string())
                    .with_args(ops.try_to_vec().unwrap())
                    .with_gas(10_000_000_000_000_0) // Default * 10
                    .transact()
                    .await
                    .unwrap();

                let gas_burnt = outcome.total_gas_burnt;
                total_gas += Gas(gas_burnt);
                println!("outcome: {:?} {}", outcome.status, gas_burnt);
                // println!("logs: {:?}", outcome.receipts_outcome.iter().map(|o| &o.outcome.logs));
            }
        }

        total_gas.0
    }

    #[tokio::test]
    async fn hashing_fuzz() {
        assert_eq!(
            fuzz_contract("./collections_bench-HASH.wasm").await,
            635133853758771
        );
    }

    #[tokio::test]
    async fn serialize_fuzz() {
        assert_eq!(
            fuzz_contract("./collections_bench-SERIALIZE.wasm").await,
            502773485446719
        );
    }

    #[tokio::test]
    async fn old_fuzz() {
        assert_eq!(
            fuzz_contract("./old_structure.wasm").await,
            912612606379644
        );
    }

    #[tokio::test]
    #[ignore]
    async fn curr_fuzz() {
        assert_eq!(
            fuzz_contract("./target/wasm32-unknown-unknown/release/collections_bench.wasm").await,
            635133853758771
        );
    }
}
