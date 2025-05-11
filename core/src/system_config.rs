use async_openai::types::FunctionObject;
use serde::{Deserialize, Serialize};

use crate::crypto_hash::CryptoHash;

pub trait RuntimeSystemConfig {
    fn id(&self) -> CryptoHash;
    fn name(&self) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LLMConfig {
    pub id: CryptoHash,
    pub system_prompt: String,
    pub openai_model: String,
    pub openai_temperature: f32,
    pub openai_max_tokens: u16,
    pub functions: Vec<FunctionObject>,
}

impl RuntimeSystemConfig for LLMConfig {
    fn id(&self) -> CryptoHash {
        self.id.clone()
    }

    fn name(&self) -> String {
        "LLM".to_string()
    }
}
