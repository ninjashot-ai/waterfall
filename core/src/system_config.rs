use async_openai::types::FunctionObject;

use crate::crypto_hash::CryptoHash;

pub trait RuntimeSystemConfig {
    fn id(&self) -> CryptoHash;
    fn name(&self) -> String;
}

pub trait LLMSystemConfig: RuntimeSystemConfig {
    fn get_system_prompt(&self) -> String;

    fn get_openai_base_url(&self) -> String;
    fn get_openai_model(&self) -> String;
    fn get_openai_temperature(&self) -> f32;
    fn get_openai_max_tokens(&self) -> u16;
    
    fn get_functions(&self) -> Vec<FunctionObject>;
}
