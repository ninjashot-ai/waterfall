use core::{state_key, CryptoHash, Instruction, State};

use anyhow::{anyhow, Result};
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

    fn parse_from(value: String) -> Self {
        Self {
            system_config_hash: CryptoHash::default(),
            memory: Vec::new(),
            new_message: value,
            new_message_index: 0,
        }
    }

    fn parse_into(&self) -> String {
        self.new_message.clone()
    }

    fn prepare(&mut self, state: &State<String>) -> Result<(), Self::Error> {
        let system_config_key = state_key!("system_config");
        let system_config_hash = state.storage.get(&system_config_key)
            .ok_or(anyhow!("system_config must exists"))?;
        self.system_config_hash = CryptoHash::from_string(&system_config_hash)?;

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

// pub struct LlmRuntime<IX, T> {
//     client: Client,
//     instructions: Vec<IX>,
// }

// impl RoleplayRuntimeClient {
//     pub async fn new(
//         executor: mpsc::Sender<(FunctionCall, oneshot::Sender<Result<String>>)>
//     ) -> Self {
//         let env = MongoDbEnv::load();
//         let db = get_db(&env.get_env_var("MONGODB_URI"), "voda_is").await;

//         let env = RoleplayEnv::load();
//         let config = OpenAIConfig::new()
//             .with_api_key(env.get_env_var("OPENAI_API_KEY"))
//             .with_api_base(env.get_env_var("OPENAI_BASE_URL"));

//         let client = Client::build(
//             reqwest::Client::new(),
//             config,
//             Default::default()
//         );

//         Self { client, db, executor }
//     }

//     pub async fn send_request(
//         &self, 
//         system_config: &SystemConfig,
//         messages: Vec<ChatCompletionRequestMessage>, 
//     ) -> Result<(
//         String, CompletionUsage, 
//         Vec<FunctionCall>, Vec<String>
//     )> {

//         let tools = system_config.functions.iter()
//             .map(|function| ChatCompletionToolArgs::default()
//                 .function(function.clone())
//                 .build()
//                 .expect("Message should build")
//             )
//             .collect::<Vec<_>>();

//         // Create chat completion request
//         let request = CreateChatCompletionRequestArgs::default()
//             .model(&system_config.openai_model)
//             .messages(messages)
//             .tools(tools)
//             .tool_choice(ChatCompletionToolChoiceOption::Auto)
//             .temperature(system_config.openai_temperature)
//             .max_tokens(system_config.openai_max_tokens)
//             .build()?;

//         // Send request to OpenAI
//         let response = self.client
//             .chat()
//             .create(request)
//             .await?;

//         let content = response
//             .choices
//             .first()
//             .ok_or_else(|| anyhow!("No response from AI inference server"))?
//             .message
//             .content
//             .clone()
//             .unwrap_or_default();

//         let maybe_function_call = response
//             .choices
//             .first()
//             .ok_or_else(|| anyhow!("No response from AI inference server"))?
//             .message
//             .clone()
//             .tool_calls
//             .unwrap_or_default()
//             .into_iter()
//             .map(|tool_call| tool_call.function)
//             .collect::<Vec<_>>();


//         let mut maybe_results = Vec::new();
//         for maybe_function in maybe_function_call.iter() {
//             let result = self.execute_function_call(maybe_function).await?;
//             maybe_results.push(result);
//         }

//         let usage = response.usage.ok_or(|| {
//             tracing::warn!("Model {} returned no usage", system_config.openai_model);
//         }).map_err(|_| anyhow!("Model {} returned no usage", system_config.openai_model))?;

//         Ok((content, usage, maybe_function_call, maybe_results))
//     }
// }

// #[async_trait::async_trait]
// impl RuntimeClient for RoleplayRuntimeClient {
//     fn get_price(&self) -> u64 { 1 }
//     fn get_db(&self) -> &Database { &self.db }

//     async fn run(
//         &self, 
//         character: &Character, user: &mut User, system_config: &SystemConfig,
//         memory: &mut ConversationMemory, message: &HistoryMessage
//     ) -> Result<HistoryMessage> {
//         let is_new_conversation = memory.history.is_empty();

//         // this will be used to send to OpenAI
//         let chat_messages = prepare_chat_messages(
//             system_config, 
//             character, user, 
//             &memory.history, message, 
//             is_new_conversation
//         )?;

//         let (
//             content, 
//             usage, 
//             maybe_function_call,
//             maybe_results
//         ) = self.send_request( system_config, chat_messages).await?;

//         let response_message = HistoryMessage {
//             owner: user.id.clone(),
//             character_id: character.id.clone(),
//             role: MessageRole::Assistant,
//             content,
//             content_type: MessageType::Text,
//             function_call_request: maybe_function_call,
//             function_call_response: maybe_results,
//             created_at: get_current_timestamp(),
//         };

//         memory.history.push((message.clone(), response_message.clone()));
//         user.add_usage(usage, system_config.openai_model.clone());

//         Ok(response_message)
//     }

//     async fn regenerate(
//         &self, 
//         character: &Character, user: &mut User, system_config: &SystemConfig,
//         memory: &mut ConversationMemory
//     ) -> Result<HistoryMessage> {
//         let last_message = memory
//             .history
//             .pop()
//             .ok_or(anyhow!("No history found"))?;
//         let (new_message, _) = last_message;
//         self.run(character, user, system_config, memory, &new_message).await
//     }

//     async fn find_system_config_by_character(
//         &self, character: &Character
//     ) -> Result<SystemConfig> {
//         let tags = character.tags.clone();
//         let config_name = {
//             if tags.contains(&"gitcoin".to_string()) {
//                 "gitcoin-screening".to_string()
//             } else {
//                 match tags[0].as_str() {
//                     "zh" => "roleplay-zh".to_string(),
//                     "kr" => "roleplay-kr".to_string(),
//                     _ => "roleplay".to_string(),
//                 }
//             }
//         };

//         let config_hash = blake3_hash(config_name.as_bytes());

//         let config = SystemConfig::select_one_by_index(&self.db, &config_hash).await?
//             .ok_or(anyhow!("System config not found"))?;

//         Ok(config)
//     }

//     async fn execute_function_call(
//         &self, call: &FunctionCall
//     ) -> Result<String> {
//         let (tx, rx) = oneshot::channel();
//         self.executor.send((call.clone(), tx)).await?;
//         let result = rx.await?;
//         result
//     }
// }
