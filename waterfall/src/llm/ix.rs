use waterfall_core::{state_key, CryptoHash, Instruction, State};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct LlmInstruction {
    pub system_config_hash: CryptoHash,
    pub memory: Vec<(String, String, Option<String>)>, // user | assistant | function call
    pub new_message: String,
    pub new_message_index: usize,
}

impl Instruction<String> for LlmInstruction {
    const INSTRUCTION_NAME: &'static str = "llm_instruction";
    const FALLIBLE: bool = false;

    type Error = anyhow::Error;

    fn parse_from(value: String, system_config_hash: CryptoHash) -> Self {
        Self {
            system_config_hash,
            memory: Vec::new(),
            new_message: value,
            new_message_index: 0,
        }
    }

    fn parse_into(&self) -> String {
        self.new_message.clone()
    }

    fn prepare(&mut self, state: &State<String>) -> Result<(), Self::Error> {
        loop {
            let user_message_key = state_key!("user_message", self.new_message_index);
            let assistant_message_key = state_key!("assistant_message", self.new_message_index);
            let toolcall_message_key = state_key!("tool_call", self.new_message_index);

            let maybe_user_message = state.storage.get(&user_message_key);
            let maybe_assistant_message = state.storage.get(&assistant_message_key);
            if maybe_user_message.is_none() || maybe_assistant_message.is_none() { break; }
            
            let user_message = maybe_user_message.unwrap().clone();
            let assistant_message = maybe_assistant_message.unwrap().clone();
            let tool_call = state.storage.get(&toolcall_message_key);
            
            self.memory.push((user_message, assistant_message, tool_call.cloned()));
            
            self.new_message_index += 1;
        }

        Ok(())
    }
}
