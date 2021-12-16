use arbitrary::{Arbitrary, Unstructured};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::store::LookupSet;
use std::hint::black_box;
use crate::types::{LightMock, HeavyMock};

macro_rules! lookup_set_contract_gen {
    ($ty:ty, $contract:ident, $function:ident) => {
        #[near_bindgen]
        #[derive(BorshDeserialize, BorshSerialize)]
        pub struct $contract {
            set: LookupSet<$ty>,
        }

        impl Default for $contract {
            fn default() -> Self {
                Self {
                    set: LookupSet::new(b"m"),
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
    LookupSetBenchHeavyElement,
    fuzz_set_heavy
);

lookup_set_contract_gen!(
    LightMock,
    LookupSetBenchLightElement,
    fuzz_set_light
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
            61..=65 => Ok(SetAction::Flush),
            _ => Ok(SetAction::Contains(T::arbitrary(u)?)),
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

    const BUFFER_SIZE: usize = 1024;

    async fn fuzz_set_contract<T>(wasm_file: &str, function: &str, elements: usize) -> Gas
    where for<'a> T: Arbitrary<'a> + BorshSerialize
    {
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        let mut buf = vec![0; BUFFER_SIZE];

        run_contract(wasm_file, function, || {
            let mut result = Vec::new();
            while result.len() < elements {
                rng.fill_bytes(&mut buf);
                let u = Unstructured::new(&buf[0..(rng.gen::<usize>() % BUFFER_SIZE)]);
                result.append(&mut Vec::<SetAction<T>>::arbitrary_take_rest(u).unwrap());
            }
            result.truncate(elements);
            result
        }).await
    }

    #[tokio::test]
    async fn hash_fuzz_set_heavy_064() {
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench-HASH.wasm", "fuzz_set_heavy", 64).await,
            Gas(226268326877842)
        );
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench-HASH.wasm", "fuzz_old_set_heavy", 64).await,
            Gas(403085638651657)
        );
    }

    #[tokio::test]
    async fn hash_fuzz_set_heavy_128() {
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench-HASH.wasm", "fuzz_set_heavy", 128).await,
            Gas(249470017800888)
        );
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench-HASH.wasm", "fuzz_old_set_heavy", 128).await,
            Gas(676830230516808)
        );
    }

    #[tokio::test]
    async fn hash_fuzz_set_heavy_256() {
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench-HASH.wasm", "fuzz_set_heavy", 256).await,
            Gas(297976042585104)
        );
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench-HASH.wasm", "fuzz_old_set_heavy", 256).await,
            Gas(1236507566538105)
        );
    }

    #[tokio::test]
    async fn hash_fuzz_set_heavy_512() {
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench-HASH.wasm", "fuzz_set_heavy", 512).await,
            Gas(396350595132001)
        );
        assert_eq!(
            fuzz_set_contract::<HeavyMock>("./collections_bench-HASH.wasm", "fuzz_old_set_heavy", 512).await,
            Gas(2374677071872957)
        );
    }

    #[tokio::test]
    async fn hash_fuzz_set_light_064() {
        assert_eq!(
            fuzz_set_contract::<LightMock>("./collections_bench-HASH.wasm", "fuzz_set_light", 64).await,
            Gas(403134795944472)
        );
        assert_eq!(
            fuzz_set_contract::<LightMock>("./collections_bench-HASH.wasm", "fuzz_old_set_light", 64).await,
            Gas(404547708273117)
        );
    }

    #[tokio::test]
    async fn hash_fuzz_set_light_128() {
        assert_eq!(
            fuzz_set_contract::<LightMock>("./collections_bench-HASH.wasm", "fuzz_set_light", 128).await,
            Gas(514535454332574)
        );
        assert_eq!(
            fuzz_set_contract::<LightMock>("./collections_bench-HASH.wasm", "fuzz_old_set_light", 128).await,
            Gas(672624188418087)
        );
    }

    #[tokio::test]
    async fn hash_fuzz_set_light_256() {
        assert_eq!(
            fuzz_set_contract::<LightMock>("./collections_bench-HASH.wasm", "fuzz_set_light", 256).await,
            Gas(575817396769886)
        );
        assert_eq!(
            fuzz_set_contract::<LightMock>("./collections_bench-HASH.wasm", "fuzz_old_set_light", 256).await,
            Gas(1204274314120067)
        );
    }

    #[tokio::test]
    async fn hash_fuzz_set_light_512() {
        assert_eq!(
            fuzz_set_contract::<LightMock>("./collections_bench-HASH.wasm", "fuzz_set_light", 512).await,
            Gas(752160390438430)
        );
        assert_eq!(
            fuzz_set_contract::<LightMock>("./collections_bench-HASH.wasm", "fuzz_old_set_light", 512).await,
            Gas(2232711006043945)
        );
    }
}
