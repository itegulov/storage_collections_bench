use arbitrary::Arbitrary;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::store::LookupMap;
use std::hint::black_box;
use crate::types::HeavyMock;

type KeyType = HeavyMock;
type ValueType = HeavyMock;

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
    use crate::utils::test::run_contract;

    const BUFFER_SIZE: usize = 4096;

    async fn fuzz_contract(wasm_file: &str) -> Gas {
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];

        run_contract(wasm_file, "fuzz", || {
            rng.fill_bytes(&mut buf);
            let u = Unstructured::new(&buf[0..(rng.gen::<usize>() % BUFFER_SIZE)]);
            Vec::<MapAction<KeyType, ValueType>>::arbitrary_take_rest(u).unwrap()
        }).await
    }

    #[tokio::test]
    async fn hashing_fuzz() {
        assert_eq!(
            fuzz_contract("./collections_bench-HASH.wasm").await,
            Gas(635133853758771)
        );
    }

    #[tokio::test]
    async fn serialize_fuzz() {
        assert_eq!(
            fuzz_contract("./collections_bench-SERIALIZE.wasm").await,
            Gas(502822617744153)
        );
    }

    #[tokio::test]
    async fn old_fuzz() {
        assert_eq!(
            fuzz_contract("./old_structure.wasm").await,
            Gas(912643592244114)
        );
    }

    #[tokio::test]
    #[ignore]
    async fn curr_fuzz() {
        assert_eq!(
            fuzz_contract("./target/wasm32-unknown-unknown/release/collections_bench.wasm").await,
            Gas(635133853758771)
        );
    }
}
