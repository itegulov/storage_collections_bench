use arbitrary::{Arbitrary, Unstructured};
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
pub struct HeavyMock {
    a: u128,
    b: Option<[u8; 32]>,
    c: Vec<u8>,
}

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
pub struct LightSparseMock {
    a: u32
}

#[derive(
    BorshDeserialize,
    BorshSerialize,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Clone,
    Debug,
    Default,
)]
pub struct LightDenseMock {
    a: u32
}

impl <'a> Arbitrary<'a> for LightDenseMock {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let a = u32::arbitrary(u)?;
        Ok(LightDenseMock { a: a & 7 })
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        return (32, Some(32))
    }
}
