use near_sdk_old::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk_old::store::LookupMap;
use near_sdk_old::near_bindgen;
use std::hint::black_box;
use crate::map::MapAction;
use crate::types::{HeavyMock, LightDenseMock, LightSparseMock};

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
                            if let Some(v) = v {
                                let _r = black_box(lm.insert(k, v));
                            } else {
                                black_box(lm.remove(&k));
                            }
                        }
                        MapAction::Remove(k) => {
                            let _r = black_box(lm.remove(&k));
                        }
                        MapAction::Flush => {
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
    LookupMapHeavyOld,
    fuzz_map_heavy_old
);

lookup_map_contract_gen!(
    LightSparseMock,
    LightSparseMock,
    LookupMapLightSparseOld,
    fuzz_map_light_sparse_old
);

lookup_map_contract_gen!(
    LightDenseMock,
    LightDenseMock,
    LookupMapLightDenseOld,
    fuzz_map_light_dense_old
);
