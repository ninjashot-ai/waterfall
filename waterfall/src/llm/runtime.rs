use core::{state_key, LLMConfig, Runtime, State, StateDiff};
use std::env;

use anyhow::{anyhow, Result};
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionToolArgs, ChatCompletionToolChoiceOption, CompletionUsage, FunctionCall
};
use async_openai::{
    config::OpenAIConfig, types::CreateChatCompletionRequestArgs, Client
};

use super::LlmInstruction;

#[derive(Clone)]
pub struct LlmRuntime {
    client: Client<OpenAIConfig>,
    instructions: Vec<LlmInstruction>,

    state: State<String>,
}

impl LlmRuntime {
    pub fn new() -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(env::var("OPENAI_API_BASE").unwrap())
            .with_api_key(env::var("OPENAI_API_KEY").unwrap());

        let client = Client::build(
            reqwest::Client::new(),
            config,
            Default::default()
        );

        Self { client, instructions: Vec::new(), state: State::default() }
    }

    fn prepare_messages(&self, ix: &LlmInstruction) -> Result<Vec<ChatCompletionRequestMessage>> {
        let mut messages = Vec::new();

        for (user, assistant, _tool_call) in ix.memory.iter() {
            messages.push(ChatCompletionRequestMessage::User(user.clone().into()));
            messages.push(ChatCompletionRequestMessage::Assistant(assistant.clone().into()));
        }

        Ok(messages)
    }

    fn execute_function_call(&self, call: &FunctionCall) -> Result<()> {
        // TOOD: push it into the queue
        Ok(())
    }

    fn state_diff_from_response(&self, index: usize, request: &str, response: &str) -> Result<StateDiff<String>> {
        let mut state_diff = StateDiff::new();

        let user_message_key = state_key!("user_message", index);
        let assistant_message_key = state_key!("assistant_message", index);
        
        // TODO: record toolcall
        // let toolcall_message_key = state_key!("tool_call", index);

        state_diff.storage_insert.insert(user_message_key, request.to_string());
        state_diff.storage_insert.insert(assistant_message_key, response.to_string());
        
        Ok(state_diff)
    }

    pub async fn send_request(&self, ix: &LlmInstruction) -> Result<(
        StateDiff<String>, CompletionUsage
    )> {
        let llm_config = self.state.storage.get(&ix.system_config_hash)
            .ok_or(anyhow!("LLM config not found"))?;
        let llm_config = serde_json::from_str::<LLMConfig>(&llm_config)?;

        let tools = llm_config.functions.iter()
            .map(|function| ChatCompletionToolArgs::default()
                .function(function.clone())
                .build()
                .expect("Message should build")
            )
            .collect::<Vec<_>>();

        let messages = self.prepare_messages(ix)?;
        let request = CreateChatCompletionRequestArgs::default()
            .model(&llm_config.openai_model)
            .messages(messages)
            .tools(tools)
            .tool_choice(ChatCompletionToolChoiceOption::Auto)
            .temperature(llm_config.openai_temperature)
            .max_tokens(llm_config.openai_max_tokens)
            .build()?;

        //  Send request to OpenAI
        let response = self.client
            .chat()
            .create(request)
            .await?;

        let content = response
            .choices
            .first()
            .ok_or_else(|| anyhow!("No response from AI inference server"))?
            .message
            .content
            .clone()
            .unwrap_or_default();

        let maybe_function_call = response
            .choices
            .first()
            .ok_or_else(|| anyhow!("No response from AI inference server"))?
            .message
            .clone()
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tool_call| tool_call.function)
            .collect::<Vec<_>>();

        for maybe_function in maybe_function_call.iter() {
            self.execute_function_call(maybe_function)?;
        }

        let usage = response.usage.ok_or(|| {
            tracing::warn!("Model {} returned no usage", llm_config.openai_model);
        }).map_err(|_| anyhow!("Model {} returned no usage", llm_config.openai_model))?;

        let state_diff = self.state_diff_from_response(
            ix.new_message_index, 
            &ix.new_message, 
            &content
        )?;

        Ok((state_diff, usage))
    }
}

impl Runtime<LlmInstruction, String> for LlmRuntime {
    type Error = anyhow::Error;

    fn push_instruction(&mut self, instruction: LlmInstruction) {
        self.instructions.push(instruction);
    }

    async fn execute_one(&mut self, instruction: &LlmInstruction) -> Result<(), Self::Error> {
        let (state_diff, usage) = self.send_request(&instruction).await?;
        state_diff.apply(&mut self.state);

        Ok(())
    }

    async fn execute(&mut self) -> Result<(), Self::Error> {
        while let Some(instruction) = self.instructions.pop() {
            self.execute_one(&instruction).await?;
        }

        Ok(())
    }
}
