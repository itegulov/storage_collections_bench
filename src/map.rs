use arbitrary::Arbitrary;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::store::LookupMap;
use std::hint::black_box;
use crate::types::{HeavyMock, LightDenseMock, LightSparseMock};

#[derive(Arbitrary, Debug, BorshDeserialize, BorshSerialize)]
pub enum MapAction<K, V> {
    Insert(K, V),
    Set(K, Option<V>),
    Remove(K),
    Flush,
    Get(K),
}

macro_rules! lookup_map_contract_gen {
    ($key:ty, $value:ty, $contract:ident, $function:ident) => {
        #[near_bindgen]
        #[derive(BorshDeserialize, BorshSerialize)]
        pub struct $contract {
            map: LookupMap<$key, $value>,
        }

        impl Default for $contract {
            fn default() -> Self {
                Self {
                    map: LookupMap::new(b"m"),
                }
            }
        }

        #[near_bindgen]
        impl $contract {
            pub fn $function(&mut self, #[serializer(borsh)] actions: Vec<MapAction<$key, $value>>) {
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
    }
}

lookup_map_contract_gen!(
    HeavyMock,
    HeavyMock,
    LookupMapHeavy,
    fuzz_map_heavy
);

lookup_map_contract_gen!(
    LightSparseMock,
    LightSparseMock,
    LookupMapLightSparse,
    fuzz_map_light_sparse
);

lookup_map_contract_gen!(
    LightDenseMock,
    LightDenseMock,
    LookupMapLightDense,
    fuzz_map_light_dense
);

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::{Arbitrary, Unstructured};
    use near_sdk::Gas;
    use rand::SeedableRng;
    use rand::{Rng, RngCore};
    use test_case::test_case;
    use crate::utils::test::run_contract_dual_function;

    const BUFFER_SIZE: usize = 4096;

    async fn fuzz_map_contract<K, V>(wasm_file: &str, function1: &str, function2: &str, elements: usize) -> (Gas, Gas)
        where for<'a> K: Arbitrary<'a> + BorshSerialize,
              for<'a> V: Arbitrary<'a> + BorshSerialize
    {
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];

        run_contract_dual_function(wasm_file, function1, function2, || {
            let mut result = Vec::new();
            while result.len() < elements {
                rng.fill_bytes(&mut buf);
                let mut u = Unstructured::new(&buf[0..(rng.gen::<usize>() % BUFFER_SIZE)]);
                result.append(&mut Vec::<MapAction<K, V>>::arbitrary(&mut u).unwrap());
            }
            result.truncate(elements);
            result
        }).await
    }

    #[test_case(64,   468203378609614,  409301808366658 ; "with 064 operations")]
    #[test_case(128,  838212905301421,  710631463045522 ; "with 128 operations")]
    #[test_case(256, 1592517606670836, 1318483224652866 ; "with 256 operations")]
    #[test_case(512, 3155354036410634, 2551572705088052 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_map_heavy(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_map_contract::<HeavyMock, HeavyMock>("./collections_bench.wasm", "fuzz_map_heavy", "fuzz_map_heavy_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   416849604447070,  380249695382278 ; "with 064 operations")]
    #[test_case(128,  723570553118064,  642674717935410 ; "with 128 operations")]
    #[test_case(256, 1361840141440078, 1180687096406980 ; "with 256 operations")]
    #[test_case(512, 2723008766411777, 2279832931258724 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_map_light_sparse(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_map_contract::<LightSparseMock, LightSparseMock>("./collections_bench.wasm", "fuzz_map_light_sparse", "fuzz_map_light_sparse_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,  219162627550378, 247495299299674 ; "with 064 operations")]
    #[test_case(128, 261080348130321, 338368821484623 ; "with 128 operations")]
    #[test_case(256, 343706649215428, 520416810294946 ; "with 256 operations")]
    #[test_case(512, 511428199178783, 888209489047712 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_map_light_dense(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_map_contract::<LightDenseMock, LightDenseMock>("./collections_bench.wasm", "fuzz_map_light_dense", "fuzz_map_light_dense_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }
}
