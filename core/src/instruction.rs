use crate::{crypto_hash::CryptoHash, state::State};

pub trait Instruction<T>: Clone + Send + Sync + 'static {
    const INSTRUCTION_NAME: &'static str;
    const FALLIBLE: bool;

    type Error;

    fn parse_from(value: T, system_config_hash: CryptoHash) -> Self;
    fn parse_into(&self) -> T;

    fn prepare(&mut self, state: &State<T>) -> Result<(), Self::Error>;
}

/// A macro that creates a state key by hashing the provided string and optional index.
///
/// This macro uses blake3_hash to create a CryptoHash from the combined input.
///
/// # Examples
///
/// ```
/// let key = state_key!("user_message");
/// let indexed_key = state_key!("user_message", 5);
/// ```
#[macro_export]
macro_rules! state_key {
    ($key:expr) => {
        $crate::blake3_hash($key.as_bytes())
    };
    ($key:expr, $index:expr) => {
        $crate::blake3_hash(format!("{}:{}", $key, $index).as_bytes())
    }
}
