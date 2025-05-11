use core::{Instruction, Runtime, ConfigReader};

use waterfall::{LlmInstruction, LlmRuntime};

#[tokio::main]
async fn main() {
    // Load configuration from yaml file
    let system_config = ConfigReader::new("config.yaml")
        .expect("Failed to load config");
    
    let mut runtime = LlmRuntime::new();
    runtime.inject_system_config(&system_config).unwrap();

    let ix = LlmInstruction::parse_from(
        "Generate me a list of things I should do when I'm trying to learn Rust. List multiple steps if needed.".to_string(),
        system_config.id,
    );

    runtime.push_instruction(ix);
    runtime.execute().await.unwrap();

    println!("State: {:?}", runtime.state);
}
