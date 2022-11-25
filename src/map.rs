use crate::types::{HeavyMock, LightDenseMock, LightSparseMock};
use arbitrary::Arbitrary;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::store::TreeMap;
use std::hint::black_box;

#[derive(Arbitrary, Debug, BorshDeserialize, BorshSerialize)]
pub enum MapAction<K, V> {
    Insert(K, V),
    Iter,
}

macro_rules! tree_map_contract_gen {
    ($key:ty, $value:ty, $contract:ident, $function:ident) => {
        #[near_bindgen]
        #[derive(BorshDeserialize, BorshSerialize)]
        pub struct $contract {
            map: TreeMap<$key, $value>,
        }

        impl Default for $contract {
            fn default() -> Self {
                Self {
                    map: TreeMap::new(b"m"),
                }
            }
        }

        #[near_bindgen]
        impl $contract {
            pub fn $function(
                &mut self,
                #[serializer(borsh)] actions: Vec<MapAction<$key, $value>>,
            ) {
                let lm = &mut self.map;
                for op in actions {
                    match op {
                        MapAction::Insert(k, v) => {
                            let _r = black_box(lm.insert(k, v));
                            black_box(lm.flush());
                        }
                        MapAction::Iter => black_box(for (_k, _v) in lm.iter() {}),
                    }
                }
            }
        }
    };
}

tree_map_contract_gen!(HeavyMock, HeavyMock, TreeMapHeavy, fuzz_map_heavy);

tree_map_contract_gen!(
    LightSparseMock,
    LightSparseMock,
    TreeMapLightSparse,
    fuzz_map_light_sparse
);

tree_map_contract_gen!(
    LightDenseMock,
    LightDenseMock,
    TreeMapLightDense,
    fuzz_map_light_dense
);

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::run_contract_dual_function;
    use arbitrary::{Arbitrary, Unstructured};
    use near_sdk::Gas;
    use rand::SeedableRng;
    use rand::{Rng, RngCore};
    use test_case::test_case;

    const BUFFER_SIZE: usize = 4096;

    async fn fuzz_map_contract<K, V>(
        wasm_file: &str,
        function1: &str,
        function2: &str,
        elements: usize,
    ) -> (Gas, Gas)
    where
        for<'a> K: Arbitrary<'a> + BorshSerialize,
        for<'a> V: Arbitrary<'a> + BorshSerialize,
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
        })
        .await
    }

    #[test_case(16,  861250031808949,  902135726849905 ; "with 16 operations")]
    #[test_case(32, 1765021166438443, 1958150495613199 ; "with 32 operations")]
    #[test_case(48, 2867495607839629, 3384915632845729 ; "with 48 operations")]
    #[tokio::test]
    async fn hash_fuzz_map_heavy(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_map_contract::<HeavyMock, HeavyMock>(
                "./collections_bench.wasm",
                "fuzz_map_heavy",
                "fuzz_map_heavy_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(16,  862180924828843,  892878218405239 ; "with 16 operations")]
    #[test_case(32, 1704863262307304, 1856373819020288 ; "with 32 operations")]
    #[test_case(48, 2677047045139060, 3111809051510848 ; "with 48 operations")]
    #[test_case(64, 3684779400644183, 4438571771124572 ; "with 64 operations")]
    #[tokio::test]
    async fn hash_fuzz_map_light_sparse(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_map_contract::<LightSparseMock, LightSparseMock>(
                "./collections_bench.wasm",
                "fuzz_map_light_sparse",
                "fuzz_map_light_sparse_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(16,  189926515837336, 190140448673152 ; "with 16 operations")]
    #[test_case(32,  208238520424964, 208692311317460 ; "with 32 operations")]
    #[test_case(48, 223638855796465, 224368060146181 ; "with 48 operations")]
    #[test_case(64, 238748262288980, 239751954495416 ; "with 64 operations")]
    #[tokio::test]
    async fn hash_fuzz_map_light_dense(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_map_contract::<LightDenseMock, LightDenseMock>(
                "./collections_bench.wasm",
                "fuzz_map_light_dense",
                "fuzz_map_light_dense_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }
}
