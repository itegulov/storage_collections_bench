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

    #[test_case(64,   563825064761917,  462390383587738 ; "with 064 operations")]
    #[test_case(128, 1015988673146376,  806678573994387 ; "with 128 operations")]
    #[test_case(256, 1948464312994994, 1509637982363051 ; "with 256 operations")]
    #[test_case(512, 3853002269712426, 2937221293243821 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_heavy_identity(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench.wasm", "fuzz_set_heavy_identity", "fuzz_set_heavy_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   531877279721161,  439088085730312 ; "with 064 operations")]
    #[test_case(128,  952456423979115,  761103366003357 ; "with 128 operations")]
    #[test_case(256, 1817995479733934, 1414501881948077 ; "with 256 operations")]
    #[test_case(512, 3586952467241736, 2744507880714675 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_heavy_sha256(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench.wasm", "fuzz_set_heavy_sha256", "fuzz_set_heavy_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   519416888087732,  442802014341911 ; "with 064 operations")]
    #[test_case(128,  922348331930297,  766682493036443 ; "with 128 operations")]
    #[test_case(256, 1754717431422900, 1428585562111122 ; "with 256 operations")]
    #[test_case(512, 3452097186584782, 2769629841731656 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_light_sparse_identity(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<LightSparseMock>("./collections_bench.wasm", "fuzz_set_light_sparse_identity", "fuzz_set_light_sparse_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   490984651609223,  419109857175935 ; "with 064 operations")]
    #[test_case(128,  863949667348619,  718616689216241 ; "with 128 operations")]
    #[test_case(256, 1634921266991784, 1328356693801764 ; "with 256 operations")]
    #[test_case(512, 3209313079948576, 2565557784735274 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_light_sparse_sha256(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<LightSparseMock>("./collections_bench.wasm", "fuzz_set_light_sparse_sha256", "fuzz_set_light_sparse_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,  196253267091761, 269950410408107 ; "with 064 operations")]
    #[test_case(128, 206308226893745, 375410880361313 ; "with 128 operations")]
    #[test_case(256, 227188651845975, 581538386570496 ; "with 256 operations")]
    #[test_case(512, 272927212229992, 999485620510444 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_light_dense_identity(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<LightDenseMock>("./collections_bench.wasm", "fuzz_set_light_dense_identity", "fuzz_set_light_dense_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,  192522673407956, 269417916119849 ; "with 064 operations")]
    #[test_case(128, 203951771422580, 374677572373313 ; "with 128 operations")]
    #[test_case(256, 227266290516903, 580882745977866 ; "with 256 operations")]
    #[test_case(512, 280621303043965, 999136086760732 ; "with 512 operations")]
    #[tokio::test]
    async fn hash_fuzz_set_light_dense_sha256(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_set_contract::<LightDenseMock>("./collections_bench.wasm", "fuzz_set_light_dense_sha256", "fuzz_set_light_dense_old", ops).await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }
}
