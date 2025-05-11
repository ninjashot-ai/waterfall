use crate::{crypto_hash::CryptoHash, state::{State, StateDiff}};

#[async_trait::async_trait]
pub trait Instruction<T> {
    const INSTRUCTION_NAME: &'static str;
    const FALLIBLE: bool;
    type Error: std::error::Error;

    fn id(&self) -> CryptoHash;

    fn parse_from(value: T) -> Self;
    fn parse_into(&self) -> T;

    async fn execute(&self, state: &State<T>) -> Result<StateDiff<T>, Self::Error>;
}
