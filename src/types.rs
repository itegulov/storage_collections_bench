use arbitrary::Arbitrary;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(
    BorshDeserialize,
    BorshSerialize,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Arbitrary,
    Clone,
    Debug,
    Default,
)]
pub struct StackHeapMock {
    a: u128,
    b: Option<[u8; 32]>,
    c: Vec<u8>,
}
