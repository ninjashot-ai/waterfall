use waterfall_core::{state_key, CryptoHash, Instruction, LLMConfig, Runtime, State, StateDiff};
use std::env;

use anyhow::{anyhow, Result};
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionToolArgs, ChatCompletionToolChoiceOption, CompletionUsage, FunctionCall
};
use async_openai::{
    config::OpenAIConfig, types::CreateChatCompletionRequestArgs, Client
};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use super::LlmInstruction;

#[derive(Clone)]
pub struct LlmRuntime {
    client: Client<OpenAIConfig>,
    instructions: Vec<LlmInstruction>,

    pub state: State<String>,
}

#[async_trait::async_trait]
impl Runtime<LlmInstruction, String> for LlmRuntime {
    fn push_instruction(&mut self, instruction: LlmInstruction) {
        let mut ix = instruction;
        ix.prepare(&self.state).unwrap();
        self.instructions.push(ix);
    }

    async fn execute_one(&mut self, instruction: &LlmInstruction) -> Result<()> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner} {msg}").unwrap());
        spinner.set_message("Processing instruction...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));
        
        let (state_diff, usage) = self.send_request(&instruction).await?;
        state_diff.apply(&mut self.state);
        
        spinner.finish_with_message("✅ Done".green().to_string());
        
        // Print usage with fixed formatting
        println!("   Tokens: prompt={}, completion={}, total={}", 
            usage.prompt_tokens, 
            usage.completion_tokens,
            usage.total_tokens
        );

        Ok(())
    }

    async fn execute(&mut self) -> Result<()> {
        if self.instructions.is_empty() {
            return Ok(());
        }
        
        println!("{}", "Processing requests...".bright_black().italic());
        while let Some(instruction) = self.instructions.pop() {
            self.execute_one(&instruction).await?;
        }

        Ok(())
    }
}

impl LlmRuntime {
    pub fn new() -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(env::var("OPENAI_BASE_URL").unwrap())
            .with_api_key(env::var("OPENAI_API_KEY").unwrap());

        let client = Client::build(
            reqwest::Client::new(),
            config,
            Default::default()
        );

        Self { client, instructions: Vec::new(), state: State::default() }
    }

    pub fn inject_system_config(&mut self, system_config: &LLMConfig) -> Result<()> {
        self.state.storage.insert(system_config.id.clone(), serde_json::to_string(system_config)?);
        Ok(())
    }

    fn prepare_messages(&self, ix: &LlmInstruction) -> Result<Vec<ChatCompletionRequestMessage>> {
        let mut messages = Vec::new();

        let system_config = self.state.storage.get(&ix.system_config_hash)
            .ok_or(anyhow!("LLM config not found"))?;
        let system_config = serde_json::from_str::<LLMConfig>(&system_config)?;

        messages.push(ChatCompletionRequestMessage::System(system_config.system_prompt.clone().into()));

        for (user, assistant, _tool_call) in ix.memory.iter() {
            messages.push(ChatCompletionRequestMessage::User(user.clone().into()));
            messages.push(ChatCompletionRequestMessage::Assistant(assistant.clone().into()));
        }

        messages.push(ChatCompletionRequestMessage::User(ix.new_message.clone().into()));

        Ok(messages)
    }

    pub fn print_state_pretty(&self) -> Result<()> {
        let width = 80;
        let border_h = "═".repeat(width - 2);
        
        println!("\n╔{}╗", border_h.bright_cyan());
        println!("║{:^width$}║", "🌊  WATERFALL STATE  🌊".bright_green(), width = width - 2);
        println!("╠{}╣", border_h.bright_cyan());
        
        // Display conversation history
        let mut index = 0;
        let mut has_messages = false;
        
        while let (Some(user_msg), Some(assistant_msg)) = (
            self.state.storage.get(&state_key!("user_message", index)),
            self.state.storage.get(&state_key!("assistant_message", index))
        ) {
            has_messages = true;
            
            // User message heading
            println!("║{:^width$}║", format!("👤 User ({})", index).bright_blue(), width = width - 2);
            println!("║ ╭{}╮ ║", "─".repeat(width - 8));
            
            // User message content
            for line in user_msg.lines() {
                if !line.is_empty() {
                    println!("║ │ {:<54} │ ║", line.blue());
                }
            }
            
            println!("║ ╰{}╯ ║", "─".repeat(width - 8));
            
            // Assistant message heading
            println!("║{:^width$}║", format!("🤖 Assistant ({})", index).bright_magenta(), width = width - 2);
            println!("║ ╭{}╮ ║", "─".repeat(width - 8));
            
            // Assistant message content
            for line in assistant_msg.lines() {
                if !line.is_empty() {
                    println!("║ │ {:<54} │ ║", line.magenta());
                }
            }
            
            println!("║ ╰{}╯ ║", "─".repeat(width - 8));
            println!("║{:^width$}║", " ".repeat(width - 2)); // Empty line for spacing
            
            index += 1;
        }
        
        if !has_messages {
            println!("║{:^width$}║", "📝 No conversation history yet".yellow().italic(), width = width - 2);
        }
        
        // Other state entries
        let message_keys: Vec<CryptoHash> = (0..index).flat_map(|i| {
            vec![state_key!("user_message", i), state_key!("assistant_message", i)]
        }).collect();
        
        let other_entries: Vec<_> = self.state.storage.iter()
            .filter(|(k, _)| !message_keys.contains(k))
            .collect();
        
        if !other_entries.is_empty() {
            println!("╠{}╣", border_h.bright_cyan());
            println!("║{:^width$}║", "📊 System State".bright_yellow(), width = width - 2);
            
            for (key, value) in other_entries {
                println!("║ ┌{}┐ ║", "─".repeat(width - 8));
                
                let key_str = key.to_string();
                let display_key = if key_str.len() > 20 {
                    format!("{}...{}", &key_str[..4], &key_str[key_str.len() - 4..])
                } else {
                    key_str
                };
                println!("║ │ {:<54} │ ║", display_key.bright_yellow().bold());
                
                let display_value = if value.len() > 100 {
                    format!("{}... [+{} bytes]", &value[..97], value.len() - 100)
                } else {
                    value.clone()
                };
                
                for line in display_value.lines() {
                    println!("║ │ {:<54} │ ║", line.white());
                }
                
                println!("║ └{}┘ ║", "─".repeat(width - 8));
            }
        }
        
        println!("╚{}╝", border_h.bright_cyan());
        
        Ok(())
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

    fn execute_function_call(&self, call: &FunctionCall) -> Result<()> {
        let width = 80;
        let border_h = "═".repeat(width - 2);
        
        println!("\n╔{}╗", border_h.bright_yellow());
        println!("║{:^width$}║", "🛠️  FUNCTION CALL  🛠️".bright_white().bold().on_yellow(), width = width - 2);
        println!("╠{}╣", border_h.bright_yellow());
        
        // Function name
        println!("║ 📛 Function:                                              ║");
        println!("║ ┌──────────────────────────────────────────────────┐     ║");
        println!("║ │ {:<54}│     ║", call.name.bright_white().bold());
        println!("║ └──────────────────────────────────────────────────┘     ║");
        
        // Arguments heading
        println!("║ 🔠 Arguments:                                             ║");
        println!("║ ┌──────────────────────────────────────────────────┐     ║");
        
        // Pretty print the JSON arguments
        let args_value: serde_json::Value = serde_json::from_str(&call.arguments).unwrap_or_default();
        let pretty_args = serde_json::to_string_pretty(&args_value).unwrap_or_default();
        
        for line in pretty_args.lines() {
            println!("║ │ {:<54}│     ║", line.bright_green());
        }
        
        println!("║ └──────────────────────────────────────────────────┘     ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        
        Ok(())
    }

    fn state_diff_from_response(&self, index: usize, request: &str, response: &str) -> Result<StateDiff<String>> {
        let mut state_diff = StateDiff::new();

        let user_message_key = state_key!("user_message", index);
        let assistant_message_key = state_key!("assistant_message", index);
        
        state_diff.storage_insert.insert(user_message_key, request.to_string());
        state_diff.storage_insert.insert(assistant_message_key, response.to_string());
        
        Ok(state_diff)
    }
}