use arbitrary::Arbitrary;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::store::LookupSet;
use std::hint::black_box;
use crate::types::StackHeapMock;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LookupSetBench {
    set: LookupSet<StackHeapMock>,
}

impl Default for LookupSetBench {
    fn default() -> Self {
        Self {
            set: LookupSet::new(b"m"),
        }
    }
}

#[derive(Arbitrary, Debug, BorshDeserialize, BorshSerialize)]
pub enum SetAction<T> {
    Insert(T),
    Put(T),
    Remove(T),
    Flush,
    Contains(T),
}

#[near_bindgen]
impl LookupSetBench {
    pub fn fuzz_set(&mut self, #[serializer(borsh)] actions: Vec<SetAction<StackHeapMock>>) {
        let ls = &mut self.set;
        for op in actions {
            match op {
                SetAction::Insert(v) => {
                    let _r = black_box(ls.insert(v));
                }
                SetAction::Put(v) => {
                    black_box(ls.put(v));
                }
                SetAction::Remove(v) => {
                    let _r = black_box(ls.remove(&v));
                }
                SetAction::Flush => {
                    black_box(ls.flush())
                }
                SetAction::Contains(v) => {
                    let _r = black_box(ls.contains(&v));
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
    use crate::utils::run_contract;

    const BUFFER_SIZE: usize = 4096;

    async fn fuzz_set_contract(wasm_file: &str) -> Gas {
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];

        run_contract(wasm_file, "fuzz_set", || {
            rng.fill_bytes(&mut buf);
            let u = Unstructured::new(&buf[0..(rng.gen::<usize>() % BUFFER_SIZE)]);
            Vec::<SetAction<StackHeapMock>>::arbitrary_take_rest(u).unwrap()
        }).await
    }

    #[tokio::test]
    async fn serialize_fuzz_set() {
        assert_eq!(
            fuzz_set_contract("./collections_bench-SERIALIZE.wasm").await,
            Gas(502192654310629)
        );
    }

    #[tokio::test]
    async fn old_fuzz_set() {
        assert_eq!(
            fuzz_set_contract("./old_structure.wasm").await,
            Gas(808241101138249)
        );
    }
}
