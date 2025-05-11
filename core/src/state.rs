use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::crypto_hash::CryptoHash;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State<T> {
    pub id: CryptoHash,
    pub storage: HashMap<
        CryptoHash, T
        // (T, Option<CryptoHash>) (value, ref_sub_state_id)
    >,
    pub sub_states: HashMap<CryptoHash, State<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateDiff<T: Clone> {
    pub storage_insert: HashMap<CryptoHash, T>,
    pub storage_update: HashMap<CryptoHash, T>,
    pub storage_delete: Vec<CryptoHash>,
}

impl<T: Clone> StateDiff<T> {
    pub fn new() -> Self {
        Self {
            storage_insert: HashMap::new(),
            storage_update: HashMap::new(),
            storage_delete: Vec::new(),
        }
    }

    pub fn apply(&self, state: &mut State<T>) {
        for (key, value) in self.storage_insert.iter() {
            state.storage.insert(key.clone(), value.clone());
        }
        
        for (key, value) in self.storage_update.iter() {
            state.storage.insert(key.clone(), value.clone());
        }

        for key in self.storage_delete.iter() {
            state.storage.remove(key);
        }
    }
}