use crate::types::{HeavyMock, LightDenseMock, LightSparseMock};
use arbitrary::{Arbitrary, Unstructured};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::store::key::{Identity, Sha256};
use near_sdk::store::LookupSet;
use std::hint::black_box;

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
                        SetAction::Remove(v) => {
                            let _r = black_box(ls.remove(&v));
                        }
                        SetAction::Contains(v) => {
                            let _r = black_box(ls.contains(&v));
                        }
                    }
                }
            }
        }
    };
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
    Remove(T),
    Contains(T),
}

impl<'a, T: Arbitrary<'a>> Arbitrary<'a> for SetAction<T> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        match (u64::from(<u32 as arbitrary::Arbitrary>::arbitrary(u)?) * 100) >> 32 {
            0..=40 => Ok(SetAction::Insert(T::arbitrary(u)?)),
            41..=60 => Ok(SetAction::Remove(T::arbitrary(u)?)),
            _ => Ok(SetAction::Contains(T::arbitrary(u)?)),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::run_contract_dual_function;
    use arbitrary::{Arbitrary, Unstructured};
    use near_sdk::Gas;
    use rand::prelude::SliceRandom;
    use rand::SeedableRng;
    use rand::{Rng, RngCore};
    use rand_xorshift::XorShiftRng;
    use test_case::test_case;

    const BUFFER_SIZE: usize = 4096;

    fn generate_n_elements<T>(n: usize, rng: &mut XorShiftRng, mut buf: &mut Vec<u8>) -> Vec<T>
    where
        for<'a> T: Arbitrary<'a> + BorshSerialize,
    {
        let mut result = Vec::new();
        while result.len() < n {
            rng.fill_bytes(&mut buf);
            let mut u = Unstructured::new(&buf[0..(rng.gen::<usize>() % BUFFER_SIZE)]);
            result.append(&mut Vec::<T>::arbitrary(&mut u).unwrap());
        }
        result.truncate(n);
        result
    }

    async fn fuzz_random<T>(
        wasm_file: &str,
        function1: &str,
        function2: &str,
        elements: usize,
    ) -> (Gas, Gas)
    where
        for<'a> T: Arbitrary<'a> + BorshSerialize,
    {
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];

        run_contract_dual_function(wasm_file, function1, function2, || {
            generate_n_elements::<SetAction<T>>(elements, &mut rng, &mut buf)
        })
        .await
    }

    async fn fuzz_percentage<T: Clone + Default>(
        wasm_file: &str,
        function1: &str,
        function2: &str,
        n: usize,
        percentage: usize,
    ) -> (Gas, Gas)
    where
        for<'a> T: Arbitrary<'a> + BorshSerialize,
    {
        assert!(percentage <= 100);
        let hits = (n / 2) * percentage / 100;
        let misses = (n / 2) - hits;
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];
        rng.fill_bytes(&mut buf);

        run_contract_dual_function(wasm_file, function1, function2, || {
            let domain = generate_n_elements::<T>(n / 2, &mut rng, &mut buf);
            let mut elements = domain.to_vec();
            elements.shuffle(&mut rng);
            let mut result = Vec::new();
            for element in domain {
                result.push(SetAction::Insert(element));
            }
            for element in elements.drain(0..hits) {
                result.push(SetAction::Contains(element));
            }
            for _ in 0..misses {
                rng.fill_bytes(&mut buf);
                let mut u = Unstructured::new(&buf[0..(rng.gen::<usize>() % BUFFER_SIZE)]);
                result.push(SetAction::Contains(T::arbitrary(&mut u).unwrap()));
            }
            result
        })
        .await
    }

    async fn fuzz_insert_remove<T: Clone + Default>(
        wasm_file: &str,
        function1: &str,
        function2: &str,
        n: usize,
    ) -> (Gas, Gas)
    where
        for<'a> T: Arbitrary<'a> + BorshSerialize,
    {
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];
        rng.fill_bytes(&mut buf);

        run_contract_dual_function(wasm_file, function1, function2, || {
            let domain = generate_n_elements::<T>(n / 2, &mut rng, &mut buf);
            let mut elements = domain.to_vec();
            elements.shuffle(&mut rng);
            let mut result = Vec::new();
            for element in domain {
                result.push(SetAction::Insert(element));
            }
            for element in elements {
                result.push(SetAction::Remove(element));
            }
            result
        })
        .await
    }

    async fn fuzz_insert_same<T: Clone + Default>(
        wasm_file: &str,
        function1: &str,
        function2: &str,
        n: usize,
    ) -> (Gas, Gas)
    where
        for<'a> T: Arbitrary<'a> + BorshSerialize,
    {
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];
        rng.fill_bytes(&mut buf);

        run_contract_dual_function(wasm_file, function1, function2, || {
            let mut result = Vec::new();
            for _ in 0..n {
                result.push(SetAction::Insert(T::default()));
            }
            result
        })
        .await
    }

    #[test_case(64,   338656540883086,  284728179315679 ; "with 064 operations")]
    #[test_case(128, 1015988673146376,  751540735299189 ; "with 128 operations")]
    #[test_case(256, 1948464312994994, 1396324704823499 ; "with 256 operations")]
    #[test_case(512, 3853002269712426, 2709166018572981 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_heavy_identity(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_random::<HeavyMock>(
                "./collections_bench.wasm",
                "fuzz_set_heavy_identity",
                "fuzz_set_heavy_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   531603546470419,  434967916129132 ; "with 064 operations")]
    #[test_case(128,  951989467257261,  751540735299189 ; "with 128 operations")]
    #[test_case(256, 1817077668246152, 1396324704823499 ; "with 256 operations")]
    #[test_case(512, 3584698193412096, 2709166018572981 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_heavy_sha256(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_random::<HeavyMock>(
                "./collections_bench.wasm",
                "fuzz_set_heavy_sha256",
                "fuzz_set_heavy_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   269663886213498,  269790559185522 ; "with 064 operations")]
    #[test_case(128,  922348331930297,  710825273663933 ; "with 128 operations")]
    #[test_case(256, 1754717431422900, 1314366025217946 ; "with 256 operations")]
    #[test_case(512, 3452097186584782, 2539055896396954 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_light_sparse_identity(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_random::<LightSparseMock>(
                "./collections_bench.wasm",
                "fuzz_set_light_sparse_identity",
                "fuzz_set_light_sparse_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   490775326182185,  414538832808827 ; "with 064 operations")]
    #[test_case(128,  863466608670839,  710825273663933 ; "with 128 operations")]
    #[test_case(256, 1633520396826222, 1314366025217946 ; "with 256 operations")]
    #[test_case(512, 3206785072868194, 2539055896396954 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_light_sparse_sha256(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_random::<LightSparseMock>(
                "./collections_bench.wasm",
                "fuzz_set_light_sparse_sha256",
                "fuzz_set_light_sparse_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,  196253267091761, 268936788557945 ; "with 064 operations")]
    #[test_case(128, 206308226893745, 374132036987705 ; "with 128 operations")]
    #[test_case(256, 227188651845975, 580337210592258 ; "with 256 operations")]
    #[test_case(512, 272927212229992, 998590551375124 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_light_dense_identity(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_random::<LightDenseMock>(
                "./collections_bench.wasm",
                "fuzz_set_light_dense_identity",
                "fuzz_set_light_dense_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,  192522673407956, 268936788557945 ; "with 064 operations")]
    #[test_case(128, 203951771422580, 374132036987705 ; "with 128 operations")]
    #[test_case(256, 227266290516903, 580337210592258 ; "with 256 operations")]
    #[test_case(512, 280621303043965, 998590551375124 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_light_dense_sha256(ops: usize, new_gas_usage: u64, old_gas_usage: u64) {
        assert_eq!(
            fuzz_random::<LightDenseMock>(
                "./collections_bench.wasm",
                "fuzz_set_light_dense_sha256",
                "fuzz_set_light_dense_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   496744231620213,  375648112787400 ; "with 064 operations")]
    #[test_case(128,  878262208754025,  629461172694120 ; "with 128 operations")]
    #[test_case(256, 1655916649737927, 1144992894197910 ; "with 256 operations")]
    #[test_case(512, 3244494042851619, 2195843685141948 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_50_percent_light_sparse_identity(
        ops: usize,
        new_gas_usage: u64,
        old_gas_usage: u64,
    ) {
        assert_eq!(
            fuzz_percentage::<LightSparseMock>(
                "./collections_bench.wasm",
                "fuzz_set_light_sparse_identity",
                "fuzz_set_light_sparse_old",
                ops,
                50
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   460978429649124,  352527184924128 ; "with 064 operations")]
    #[test_case(128,  805257406158810,  581576917463124 ; "with 128 operations")]
    #[test_case(256, 1506473111380050, 1046242656367122 ; "with 256 operations")]
    #[test_case(512, 2939700087599658, 1991859852287166 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_75_percent_light_sparse_identity(
        ops: usize,
        new_gas_usage: u64,
        old_gas_usage: u64,
    ) {
        assert_eq!(
            fuzz_percentage::<LightSparseMock>(
                "./collections_bench.wasm",
                "fuzz_set_light_sparse_identity",
                "fuzz_set_light_sparse_old",
                ops,
                75
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   425504898504174,  329277441413448 ; "with 064 operations")]
    #[test_case(128,  732480732655629,  533660458320276 ; "with 128 operations")]
    #[test_case(256, 1358195486324472,  947720703729042 ; "with 256 operations")]
    #[test_case(512, 2636144750200017, 1788552301581276 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_100_percent_light_sparse_identity(
        ops: usize,
        new_gas_usage: u64,
        old_gas_usage: u64,
    ) {
        assert_eq!(
            fuzz_percentage::<LightSparseMock>(
                "./collections_bench.wasm",
                "fuzz_set_light_sparse_identity",
                "fuzz_set_light_sparse_old",
                ops,
                100
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   300514136623806,  273078017068860 ; "with 064 operations")]
    #[test_case(128,  472773627515589,  417139508914044 ; "with 128 operations")]
    #[test_case(256,  818460607665780,  705259160699868 ; "with 256 operations")]
    #[test_case(512, 1510945438052793, 1281505128080604 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_insert_remove_light_sparse_identity(
        ops: usize,
        new_gas_usage: u64,
        old_gas_usage: u64,
    ) {
        assert_eq!(
            fuzz_insert_remove::<LightSparseMock>(
                "./collections_bench.wasm",
                "fuzz_set_light_sparse_identity",
                "fuzz_set_light_sparse_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }

    #[test_case(64,   136128598780890,  252860062647504 ; "with 064 operations")]
    #[test_case(128,  138189263446746,  372970819531728 ; "with 128 operations")]
    #[test_case(256,  142307260873914,  613189001395632 ; "with 256 operations")]
    #[test_case(512,  150549919537338, 1093632028932528 ; "with 512 operations")]
    #[tokio::test]
    async fn fuzz_insert_same_light_sparse_identity(
        ops: usize,
        new_gas_usage: u64,
        old_gas_usage: u64,
    ) {
        assert_eq!(
            fuzz_insert_same::<LightSparseMock>(
                "./collections_bench.wasm",
                "fuzz_set_light_sparse_identity",
                "fuzz_set_light_sparse_old",
                ops
            )
            .await,
            (Gas(new_gas_usage), Gas(old_gas_usage))
        );
    }
}
