use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupSet;
use near_sdk::near_bindgen;
use std::hint::black_box;
use crate::set::SetAction;
use crate::types::{HeavyMock, LightDenseMock, LightSparseMock};

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
                            let _r = black_box(ls.insert(&v));
                        }
                        SetAction::Put(v) => {
                            black_box(ls.insert(&v));
                        }
                        SetAction::Remove(v) => {
                            let _r = black_box(ls.remove(&v));
                        }
                        SetAction::Flush => {
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
    fuzz_set_heavy_old
);

lookup_set_contract_gen!(
    LightSparseMock,
    LookupSetLightSparseOld,
    fuzz_set_light_sparse_old
);

lookup_set_contract_gen!(
    LightDenseMock,
    LookupSetLightDenseOld,
    fuzz_set_light_dense_old
);
