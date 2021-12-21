use arbitrary::{Arbitrary, Unstructured};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::store::LookupSet;
use std::hint::black_box;
use near_sdk::store::key::{Identity, Sha256};
use crate::types::{LightDenseMock, LightSparseMock, HeavyMock};

macro_rules! lookup_set_contract_gen {
    ($ty:ty, $to_key:ty, $contract:ident, $function:ident) => {
        #[near_bindgen]
        #[derive(BorshDeserialize, BorshSerialize)]
        pub struct $contract {
            set: LookupSet<$ty, $to_key>,
        }

        impl Default for $contract {
            fn default() -> Self {
                Self {
                    set: LookupSet::with_hasher(b"m"),
                }
            }
        }

        #[near_bindgen]
        impl $contract {
            pub fn $function(&mut self, #[serializer(borsh)] actions: Vec<SetAction<$ty>>) {
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
    }
}

lookup_set_contract_gen!(
    HeavyMock,
    Identity,
    LookupSetHeavyIdentity,
    fuzz_set_heavy_identity
);

lookup_set_contract_gen!(
    HeavyMock,
    Sha256,
    LookupSetHeavySha256,
    fuzz_set_heavy_sha256
);

lookup_set_contract_gen!(
    LightSparseMock,
    Identity,
    LookupSetLightSparseIdentity,
    fuzz_set_light_sparse_identity
);

lookup_set_contract_gen!(
    LightSparseMock,
    Sha256,
    LookupSetLightSparseSha256,
    fuzz_set_light_sparse_sha256
);

lookup_set_contract_gen!(
    LightDenseMock,
    Identity,
    LookupSetLightDenseIdentity,
    fuzz_set_light_dense_identity
);

lookup_set_contract_gen!(
    LightDenseMock,
    Sha256,
    LookupSetLightDenseSha256,
    fuzz_set_light_dense_sha256
);

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub enum SetAction<T> {
    Insert(T),
    Put(T),
    Remove(T),
    Flush,
    Contains(T),
}

impl <'a, T: Arbitrary<'a>> Arbitrary<'a> for SetAction<T> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        match (u64::from(<u32 as arbitrary::Arbitrary>::arbitrary(u)?) * 100) >> 32 {
            1..=35 => Ok(SetAction::Insert(T::arbitrary(u)?)),
            36..=40 => Ok(SetAction::Put(T::arbitrary(u)?)),
            41..=60 => Ok(SetAction::Remove(T::arbitrary(u)?)),
            61..=99 => Ok(SetAction::Contains(T::arbitrary(u)?)),
            _ => Ok(SetAction::Flush),
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
    use test_case::test_case;
    use crate::utils::test::run_contract_dual_function;

    const BUFFER_SIZE: usize = 4096;

    async fn fuzz_set_contract<T>(wasm_file: &str, function1: &str, function2: &str, elements: usize) -> (Gas, Gas)
    where for<'a> T: Arbitrary<'a> + BorshSerialize
    {
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];

        run_contract_dual_function(wasm_file, function1, function2, || {
            let mut result = Vec::new();
            while result.len() < elements {
                rng.fill_bytes(&mut buf);
                let mut u = Unstructured::new(&buf[0..(rng.gen::<usize>() % BUFFER_SIZE)]);
                result.append(&mut Vec::<SetAction<T>>::arbitrary(&mut u).unwrap());
            }
            result.truncate(elements);
            result
        }).await
    }

    #[test_case(64,    563723433287917,  462288752113738 ; "with 064 operations")]
    #[test_case(128,  1015887041672376,  806576942520387 ; "with 128 operations")]
    #[test_case(256,  1948362681520994, 1509536350889051 ; "with 256 operations")]
    #[test_case(512,  3852900638238426, 2937119661769821 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_heavy_identity(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench.wasm", "fuzz_set_heavy_identity", "fuzz_set_heavy_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   531775648247161,  438986454256312 ; "with 064 operations")]
    #[test_case(128,  952354792505115,  761001734529357 ; "with 128 operations")]
    #[test_case(256, 1817893848259934, 1414400250474077 ; "with 256 operations")]
    #[test_case(512, 3586850835767736, 2744406249240675 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_heavy_sha256(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench.wasm", "fuzz_set_heavy_sha256", "fuzz_set_heavy_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   519315256613732,  442518790213892 ; "with 064 operations")]
    #[test_case(128,  922246700456297,  766217437159403 ; "with 128 operations")]
    #[test_case(256, 1754615799948900, 1427757320926044 ; "with 256 operations")]
    #[test_case(512, 3451995555110782, 2768077620880522 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_light_sparse_identity(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<LightSparseMock>("./collections_bench.wasm", "fuzz_set_light_sparse_identity", "fuzz_set_light_sparse_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   490883020135223,  418826633047916 ; "with 064 operations")]
    #[test_case(128,  863848035874619,  718151633339201 ; "with 128 operations")]
    #[test_case(256, 1634819635517784, 1327528452616686 ; "with 256 operations")]
    #[test_case(512, 3209211448474576, 2564005563884140 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_light_sparse_sha256(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<LightSparseMock>("./collections_bench.wasm", "fuzz_set_light_sparse_sha256", "fuzz_set_light_sparse_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,  196151635617761, 269667186280088 ; "with 064 operations")]
    #[test_case(128, 206206595419745, 374945824484273 ; "with 128 operations")]
    #[test_case(256, 227087020371975, 580710145385418 ; "with 256 operations")]
    #[test_case(512, 272825580755992, 997933399659310 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_light_dense_identity(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<LightDenseMock>("./collections_bench.wasm", "fuzz_set_light_dense_identity", "fuzz_set_light_dense_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,  192421041933956, 269134691991830 ; "with 064 operations")]
    #[test_case(128, 203850139948580, 374212516496273 ; "with 128 operations")]
    #[test_case(256, 227164659042903, 580054504792788 ; "with 256 operations")]
    #[test_case(512, 280519671569965, 997583865909598 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_light_dense_sha256(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<LightDenseMock>("./collections_bench.wasm", "fuzz_set_light_dense_sha256", "fuzz_set_light_dense_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }
}
