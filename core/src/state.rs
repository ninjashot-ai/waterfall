use std::collections::HashMap;

use crate::crypto_hash::CryptoHash;

pub struct State<T> {
    pub id: CryptoHash,
    pub storage: HashMap<
        CryptoHash, 
        (T, Option<CryptoHash>)
    >, // (value, ref_sub_state_id)
    pub sub_states: HashMap<CryptoHash, State<T>>,
}

pub struct StateDiff<T> {
    pub storage_insert: HashMap<CryptoHash, (T, Option<CryptoHash>)>,
    pub storage_update: HashMap<CryptoHash, (T, Option<CryptoHash>)>,
    pub storage_delete: Vec<CryptoHash>,
}
