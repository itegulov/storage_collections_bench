use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::near_bindgen;
use std::hint::black_box;
use crate::map::MapAction;
use crate::types::HeavyMock;

type KeyType = HeavyMock;
type ValueType = HeavyMock;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LookupMapBench {
    map: LookupMap<KeyType, ValueType>,
}

impl Default for LookupMapBench {
    fn default() -> Self {
        Self {
            map: LookupMap::new(b"m"),
        }
    }
}

#[near_bindgen]
impl LookupMapBench {
    pub fn fuzz_old_map(&mut self, #[serializer(borsh)] actions: Vec<MapAction<KeyType, ValueType>>) {
        let lm = &mut self.map;
        for op in actions {
            match op {
                MapAction::Insert(k, v) => {
                    let _r = black_box(lm.insert(&k, &v));
                }
                MapAction::Set(k, v) => {
                    if let Some(v) = v {
                        let _r = black_box(lm.insert(&k, &v));
                    } else {
                        black_box(lm.remove(&k));
                    }
                }
                MapAction::Remove(k) => {
                    let _r = black_box(lm.remove(&k));
                }
                MapAction::Flush => {
                    // lm.flush();
                }
                MapAction::Get(k) => {
                    let _r = black_box(lm.get(&k));
                }
            }
        }
    }
}
