use waterfall_core::{Instruction, Runtime, ConfigReader};
use waterfall::{LlmInstruction, LlmRuntime};
use indicatif::{ProgressBar, ProgressStyle};
use colored::*;
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    println!("{}", "🌊 Waterfall CLI Demo 🌊".bright_blue().bold());
    
    // Create a spinner for loading config
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{spinner} {msg}").unwrap());
    spinner.set_message("Loading configuration...");
    
    // Load configuration from yaml file
    let system_config = match ConfigReader::new("config.yaml") {
        Ok(config) => {
            spinner.finish_with_message("Configuration loaded successfully!".green().to_string());
            config
        },
        Err(e) => {
            spinner.finish_with_message("Failed to load configuration".red().to_string());
            eprintln!("{}: {}", "Error".red().bold(), e);
            return;
        }
    };
    
    // Get user input
    println!("\n{}", "What would you like me to help you with?".yellow());
    print!("{} ", ">".cyan().bold());
    io::stdout().flush().unwrap();
    
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).unwrap();
    
    // Create the runtime with fancy loading
    let mut runtime = LlmRuntime::new();
    spinner.set_message("Initializing runtime...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    
    match runtime.inject_system_config(&system_config) {
        Ok(_) => spinner.finish_with_message("Runtime initialized successfully!".green().to_string()),
        Err(e) => {
            spinner.finish_with_message("Failed to initialize runtime".red().to_string());
            eprintln!("{}: {}", "Error".red().bold(), e);
            return;
        }
    }

    // Parse and execute instruction
    spinner.set_message("Processing your request...");
    spinner.reset();
    
    let ix = LlmInstruction::parse_from(
        user_input.trim().to_string(),
        system_config.id.clone(),
    );

    runtime.push_instruction(ix);
    
    // Execute with spinner
    spinner.set_message("Executing instruction...");
    match runtime.execute().await {
        Ok(_) => {
            spinner.finish_with_message("Execution completed!".green().to_string());
            runtime.print_state_pretty().unwrap();
        },
        Err(e) => {
            spinner.finish_with_message("Execution failed".red().to_string());
            eprintln!("{}: {}", "Error".red().bold(), e);
        }
    }
    
    // Loop to allow for more interactions
    loop {
        println!("\n{}", "What else would you like to do? (Type 'exit' to quit)".yellow());
        print!("{} ", ">".cyan().bold());
        io::stdout().flush().unwrap();
        
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        
        let input = user_input.trim();
        if input.to_lowercase() == "exit" {
            println!("{}", "Goodbye! 👋".bright_blue());
            break;
        }
        
        let ix = LlmInstruction::parse_from(
            input.to_string(),
            system_config.id.clone(),
        );
        
        runtime.push_instruction(ix);
        
        // Execute with spinner
        spinner.set_message("Executing instruction...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        
        match runtime.execute().await {
            Ok(_) => {
                spinner.finish_with_message("Execution completed!".green().to_string());
                runtime.print_state_pretty().unwrap();
            },
            Err(e) => {
                spinner.finish_with_message("Execution failed".red().to_string());
                eprintln!("{}: {}", "Error".red().bold(), e);
            }
        }
    }
}
