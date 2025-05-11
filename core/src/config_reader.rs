use anyhow::{anyhow, Result};
use async_openai::types::FunctionObject;
use serde::Deserialize;
use serde_yaml::{self, Value};
use std::fs;
use std::path::Path;

use crate::{state_key, LLMConfig};

#[derive(Debug, Deserialize)]
pub struct ConfigReader;

impl ConfigReader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<LLMConfig> {
        let config_str = fs::read_to_string(path)?;
        let config: Value = serde_yaml::from_str(&config_str)?;
        
        // Extract orchestrator config
        let orchestrator = config.get("orchestrator")
            .ok_or_else(|| anyhow!("Missing orchestrator section in config"))?;
        
        // Extract required fields
        let id = orchestrator.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid id field"))?;
        
        let system_prompt = orchestrator.get("system_prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid system_prompt field"))?;
        
        let model = orchestrator.get("model")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid model field"))?;
        
        let temperature = orchestrator.get("temperature")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing or invalid temperature field"))?;
        
        let max_tokens = orchestrator.get("max_tokens")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("Missing or invalid max_tokens field"))?;
        
        // Extract tools if they exist
        let functions = if let Some(tools) = orchestrator.get("tools") {
            if let Some(tools_array) = tools.as_sequence() {
                tools_array.iter()
                    .map(|tool| parse_tool(tool))
                    .collect::<Result<Vec<FunctionObject>>>()?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(LLMConfig {
            id: state_key!(id),
            system_prompt: system_prompt.to_string(),
            openai_model: model.to_string(),
            openai_temperature: temperature as f32,
            openai_max_tokens: max_tokens as u16,
            functions,
        })
    }
}

fn parse_tool(tool: &Value) -> Result<FunctionObject> {
    let name = tool.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Tool missing name"))?;
    
    let description = tool.get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    let strict = tool.get("strict")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // Convert parameters section to JSON for the FunctionObject
    let parameters = if let Some(params) = tool.get("parameters") {
        serde_json::to_value(params)?
    } else {
        serde_json::json!({})
    };
    
    Ok(FunctionObject {
        name: name.to_string(),
        description: Some(description),
        parameters: Some(parameters),
        strict: Some(strict),
    })
} 