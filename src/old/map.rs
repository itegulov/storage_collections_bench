use crate::map::MapAction;
use crate::types::{HeavyMock, LightDenseMock, LightSparseMock};
use near_sdk_old::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk_old::near_bindgen;
use near_sdk_old::store::TreeMap;
use std::hint::black_box;

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

tree_map_contract_gen!(HeavyMock, HeavyMock, TreeMapHeavyOld, fuzz_map_heavy_old);

tree_map_contract_gen!(
    LightSparseMock,
    LightSparseMock,
    TreeMapLightSparseOld,
    fuzz_map_light_sparse_old
);

tree_map_contract_gen!(
    LightDenseMock,
    LightDenseMock,
    TreeMapLightDenseOld,
    fuzz_map_light_dense_old
);
