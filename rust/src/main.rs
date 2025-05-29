use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use dirs::home_dir;
use dotenv::dotenv;
use pulldown_cmark::{Event, Parser as MarkdownParser, Tag};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

// Constants
const DEFAULT_MODEL: &str = "openrouter/auto";
const DEFAULT_MAX_TOKENS: u32 = 1024;
const DEFAULT_TEMPERATURE: f32 = 0.7;

// Types
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: DEFAULT_TEMPERATURE,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenRouterChoice {
    message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenRouterModel {
    id: String,
    context_length: Option<u32>,
    description: Option<String>,
    pricing: Option<OpenRouterPricing>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenRouterPricing {
    prompt: f32,
    completion: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

// CLI Arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Ask the AI a coding question
    Ask {
        /// The prompt to send to the AI
        #[arg(required = true)]
        prompt: Vec<String>,

        /// File to use as context
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// System prompt to use
        #[arg(short, long)]
        system: Option<String>,

        /// Save the response to a file
        #[arg(long)]
        save: Option<PathBuf>,

        /// Extract and save code blocks
        #[arg(short, long)]
        extract: bool,
    },

    /// Configure NimbusCode settings
    Config {
        /// Set the OpenRouter API key
        #[arg(long)]
        api_key: Option<String>,

        /// Set the default model
        #[arg(long)]
        model: Option<String>,

        /// Set the maximum tokens
        #[arg(long)]
        max_tokens: Option<u32>,

        /// Set the temperature
        #[arg(long)]
        temperature: Option<f32>,

        /// Show the current configuration
        #[arg(long)]
        show: bool,
    },

    /// List available models from OpenRouter
    Models,

    /// Improve existing code with AI suggestions
    Improve {
        /// File containing code to improve
        file: PathBuf,

        /// Save the improved code to a file
        #[arg(long)]
        save: Option<PathBuf>,
    },

    /// Explain code with detailed comments and documentation
    Explain {
        /// File containing code to explain
        file: PathBuf,
    },

    /// Generate code based on a description
    Generate {
        /// Description of the code to generate
        #[arg(required = true)]
        prompt: Vec<String>,

        /// Programming language
        #[arg(short, long, default_value = "python")]
        language: String,

        /// Save the generated code to a file
        #[arg(long)]
        save: Option<PathBuf>,
    },

    /// Generate cloud deployment code or instructions
    Cloud {
        /// Description of the cloud deployment
        #[arg(required = true)]
        prompt: Vec<String>,

        /// Cloud provider (aws, azure, gcp)
        #[arg(short, long, default_value = "aws")]
        provider: String,

        /// Save the deployment code to a file
        #[arg(long)]
        save: Option<PathBuf>,
    },

    /// Generate mobile app development code or guidance
    Mobile {
        /// Description of the mobile app
        #[arg(required = true)]
        prompt: Vec<String>,

        /// Mobile platform (ios, android, cross)
        #[arg(short, long, default_value = "cross")]
        platform: String,

        /// Save the generated code to a file
        #[arg(long)]
        save: Option<PathBuf>,
    },

    /// Start an interactive coding session with the AI
    Interactive,
}

fn main() -> Result<()> {
    // Load environment variables from .env file if present
    dotenv().ok();

    // Parse command line arguments
    let cli = Cli::parse();

    // Ensure config directory exists
    ensure_config_dir()?;

    // Handle commands
    match cli.command {
        Commands::Ask {
            prompt,
            file,
            system,
            save,
            extract,
        } => {
            cmd_ask(prompt, file, system, save, extract)?;
        }
        Commands::Config {
            api_key,
            model,
            max_tokens,
            temperature,
            show,
        } => {
            cmd_config(api_key, model, max_tokens, temperature, show)?;
        }
        Commands::Models => {
            cmd_models()?;
        }
        Commands::Improve { file, save } => {
            cmd_improve(file, save)?;
        }
        Commands::Explain { file } => {
            cmd_explain(file)?;
        }
        Commands::Generate {
            prompt,
            language,
            save,
        } => {
            cmd_generate(prompt, language, save)?;
        }
        Commands::Cloud {
            prompt,
            provider,
            save,
        } => {
            cmd_cloud(prompt, provider, save)?;
        }
        Commands::Mobile {
            prompt,
            platform,
            save,
        } => {
            cmd_mobile(prompt, platform, save)?;
        }
        Commands::Interactive => {
            cmd_interactive()?;
        }
    }

    Ok(())
}

// Helper functions
fn get_config_dir() -> Result<PathBuf> {
    let home = home_dir().context("Could not determine home directory")?;
    Ok(home.join(".nimbuscode"))
}

fn get_config_file() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.json"))
}

fn ensure_config_dir() -> Result<()> {
    let config_dir = get_config_dir()?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }

    let config_file = get_config_file()?;
    if !config_file.exists() {
        let default_config = Config::default();
        save_config(&default_config)?;
        println!(
            "{}",
            "Config file created at ~/.nimbuscode/config.json".yellow()
        );
        println!(
            "{}",
            "Please set your OpenRouter API key with 'nimbuscode config --api-key YOUR_API_KEY'"
                .yellow()
        );
    }

    Ok(())
}

fn load_config() -> Result<Config> {
    let config_file = get_config_file()?;
    if !config_file.exists() {
        return Ok(Config::default());
    }

    let mut file = File::open(config_file).context("Failed to open config file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read config file")?;

    let config: Config = serde_json::from_str(&contents).context("Failed to parse config file")?;
    Ok(config)
}

fn save_config(config: &Config) -> Result<()> {
    let config_file = get_config_file()?;
    let json = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    let mut file = File::create(config_file).context("Failed to create config file")?;
    file.write_all(json.as_bytes())
        .context("Failed to write config file")?;
    Ok(())
}

fn get_api_key() -> Result<String> {
    // Try to get API key from config
    let config = load_config()?;
    let api_key = if !config.api_key.is_empty() {
        config.api_key
    } else {
        // Try to get API key from environment variable
        std::env::var("OPENROUTER_API_KEY").unwrap_or_default()
    };

    if api_key.is_empty() {
        eprintln!(
            "{}",
            "API key not found. Please set it with 'nimbuscode config --api-key YOUR_API_KEY'"
                .red()
        );
        process::exit(1);
    }

    Ok(api_key)
}

fn query_openrouter(prompt: &str, system_prompt: Option<&str>) -> Result<String> {
    let config = load_config()?;
    let api_key = get_api_key()?;

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .context("Failed to create Authorization header")?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "HTTP-Referer",
        HeaderValue::from_static("https://github.com/cline/cline"),
    );

    let mut messages = Vec::new();
    if let Some(system) = system_prompt {
        messages.push(Message {
            role: "system".to_string(),
            content: system.to_string(),
        });
    }
    messages.push(Message {
        role: "user".to_string(),
        content: prompt.to_string(),
    });

    let request = OpenRouterRequest {
        model: config.model,
        messages,
        max_tokens: config.max_tokens,
        temperature: config.temperature,
    };

    let client = Client::new();
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .headers(headers)
        .json(&request)
        .send()
        .context("Failed to send request to OpenRouter API")?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .context("Failed to read error response from OpenRouter API")?;
        return Err(anyhow::anyhow!(
            "OpenRouter API returned error: {}",
            error_text
        ));
    }

    let response_data: OpenRouterResponse = response
        .json()
        .context("Failed to parse response from OpenRouter API")?;

    if response_data.choices.is_empty() {
        return Err(anyhow::anyhow!("OpenRouter API returned no choices"));
    }

    Ok(response_data.choices[0].message.content.clone())
}

fn extract_code_blocks(markdown_text: &str) -> Vec<String> {
    let mut code_blocks = Vec::new();
    let mut in_code_block = false;
    let mut current_block = String::new();

    let parser = MarkdownParser::new(markdown_text);
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
            }
            Event::End(Tag::CodeBlock(_)) => {
                if !current_block.is_empty() {
                    code_blocks.push(current_block.clone());
                    current_block.clear();
                }
                in_code_block = false;
            }
            Event::Text(text) => {
                if in_code_block {
                    current_block.push_str(&text);
                }
            }
            _ => {}
        }
    }

    code_blocks
}

fn print_markdown(text: &str) {
    // Simple markdown rendering for terminal
    // In a real implementation, you might want to use a more sophisticated renderer
    println!("\n{}", text);
}

// Command implementations
fn cmd_ask(
    prompt: Vec<String>,
    file: Option<PathBuf>,
    system: Option<String>,
    save: Option<PathBuf>,
    extract: bool,
) -> Result<()> {
    let full_prompt = prompt.join(" ");

    // Add file content to prompt if specified
    let full_prompt = if let Some(file_path) = file {
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File {} does not exist", file_path.display()));
        }
        let file_content = fs::read_to_string(&file_path).context("Failed to read file")?;
        format!(
            "File content:\n```\n{}\n```\n\nPrompt: {}",
            file_content, full_prompt
        )
    } else {
        full_prompt
    };

    // Default system prompt for coding assistance
    let default_system_prompt = "
    You are NimbusCode, an expert programming assistant. Your goal is to help the user write high-quality, 
    efficient, and secure code. Provide clear, concise explanations and code examples.
    Focus on best practices, performance optimization, and security considerations.
    When appropriate, suggest improvements to the user's code or approach.
    ";

    let system_prompt = system.as_deref().unwrap_or(default_system_prompt);

    println!("{}", "Thinking...".green());
    let response = query_openrouter(&full_prompt, Some(system_prompt))?;

    // Display the response
    println!("\n{}", "NimbusCode:".blue().bold());
    print_markdown(&response);

    // Save response if requested
    if let Some(save_path) = save {
        fs::write(&save_path, &response).context("Failed to save response to file")?;
        println!("\n{}", format!("Response saved to {}", save_path.display()).green());
    }

    // Extract and save code blocks if requested
    if extract {
        let code_blocks = extract_code_blocks(&response);
        if !code_blocks.is_empty() {
            for (i, block) in code_blocks.iter().enumerate() {
                let filename = format!("code_block_{}.txt", i + 1);
                fs::write(&filename, block).context("Failed to save code block")?;
                println!(
                    "{}",
                    format!("Code block saved to {}", filename).green()
                );
            }
        } else {
            println!("{}", "No code blocks found in the response".yellow());
        }
    }

    Ok(())
}

fn cmd_config(
    api_key: Option<String>,
    model: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    show: bool,
) -> Result<()> {
    let mut config = load_config()?;

    if show {
        // Hide API key for security
        let mut display_config = config.clone();
        if !display_config.api_key.is_empty() {
            let len = display_config.api_key.len();
            if len > 4 {
                display_config.api_key = format!(
                    "********{}",
                    &display_config.api_key[len - 4..len]
                );
            } else {
                display_config.api_key = "********".to_string();
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&display_config).context("Failed to serialize config")?
        );
        return Ok(());
    }

    if let Some(key) = api_key {
        config.api_key = key;
    }

    if let Some(m) = model {
        config.model = m;
    }

    if let Some(tokens) = max_tokens {
        config.max_tokens = tokens;
    }

    if let Some(temp) = temperature {
        config.temperature = temp;
    }

    save_config(&config)?;
    println!("{}", "Configuration updated successfully".green());

    Ok(())
}

fn cmd_models() -> Result<()> {
    let api_key = get_api_key()?;

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .context("Failed to create Authorization header")?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    println!("{}", "Fetching available models...".green());

    let client = Client::new();
    let response = client
        .get("https://openrouter.ai/api/v1/models")
        .headers(headers)
        .send()
        .context("Failed to fetch models from OpenRouter API")?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .context("Failed to read error response from OpenRouter API")?;
        return Err(anyhow::anyhow!(
            "OpenRouter API returned error: {}",
            error_text
        ));
    }

    let models: OpenRouterModelsResponse = response
        .json()
        .context("Failed to parse models response from OpenRouter API")?;

    println!("{}", "Available Models:".bold());

    // Filter for free models
    let mut free_models = Vec::new();
    for model in models.data {
        if let Some(pricing) = &model.pricing {
            if pricing.prompt == 0.0 && pricing.completion == 0.0 {
                free_models.push(model);
            }
        }
    }

    if !free_models.is_empty() {
        println!("{}", "Free Models:".green().bold());
        for model in free_models {
            println!("  {}", model.id.green());
            if let Some(context) = model.context_length {
                println!("    Context: {}", context);
            }
            if let Some(desc) = model.description {
                println!("    Description: {}", desc);
            }
            println!();
        }
    } else {
        println!("{}", "No free models found".yellow());
    }

    Ok(())
}

fn cmd_improve(file: PathBuf, save: Option<PathBuf>) -> Result<()> {
    if !file.exists() {
        return Err(anyhow::anyhow!("File {} does not exist", file.display()));
    }

    let code = fs::read_to_string(&file).context("Failed to read file")?;

    let prompt = format!(
        "
    Please improve the following code. Focus on:
    1. Code quality and readability
    2. Performance optimizations
    3. Security best practices
    4. Error handling
    5. Documentation
    
    Provide the improved code and explain your changes.
    
    ```
    {}
    ```
    ",
        code
    );

    let system_prompt = "
    You are NimbusCode, an expert code reviewer and optimizer. Analyze the provided code and suggest
    improvements. Return the improved code in a markdown code block with the same language as the original.
    Explain your changes clearly but concisely.
    ";

    println!("{}", "Analyzing and improving code...".green());
    let response = query_openrouter(&prompt, Some(system_prompt))?;

    println!(
        "\n{}",
        format!("Code Improvements for {}", file.display()).blue().bold()
    );
    print_markdown(&response);

    // Extract and save the improved code if requested
    if let Some(save_path) = save {
        let code_blocks = extract_code_blocks(&response);
        if !code_blocks.is_empty() {
            fs::write(&save_path, &code_blocks[0]).context("Failed to save improved code")?;
            println!(
                "\n{}",
                format!("Improved code saved to {}", save_path.display()).green()
            );
        } else {
            println!("{}", "No code blocks found in the response".yellow());
        }
    }

    Ok(())
}

fn cmd_explain(file: PathBuf) -> Result<()> {
    if !file.exists() {
        return Err(anyhow::anyhow!("File {} does not exist", file.display()));
    }

    let code = fs::read_to_string(&file).context("Failed to read file")?;

    let prompt = format!(
        "
    Please explain the following code in detail:
    
    ```
    {}
    ```
    
    Include:
    1. Overall purpose and functionality
    2. Breakdown of key components
    3. How the different parts work together
    4. Any potential issues or considerations
    ",
        code
    );

    let system_prompt = "
    You are NimbusCode, an expert code analyst. Provide a clear, educational explanation of the code.
    Break down complex concepts and use examples where helpful. Your goal is to help the user fully
    understand how the code works.
    ";

    println!("{}", "Analyzing code...".green());
    let response = query_openrouter(&prompt, Some(system_prompt))?;

    println!(
        "\n{}",
        format!("Code Explanation for {}", file.display()).blue().bold()
    );
    print_markdown(&response);

    Ok(())
}

fn cmd_generate(prompt: Vec<String>, language: String, save: Option<PathBuf>) -> Result<()> {
    let full_prompt = prompt.join(" ");

    let prompt = format!(
        "
    Generate {} code for the following:
    
    {}
    
    Provide complete, working code with appropriate comments and documentation.
    ",
        language, full_prompt
    );

    let system_prompt = format!(
        "
    You are NimbusCode, an expert {} developer. Generate high-quality, efficient, and secure code
    based on the user's requirements. Include helpful comments and documentation. Focus on best practices
    and maintainability.
    ",
        language
    );

    println!("{}", "Generating code...".green());
    let response = query_openrouter(&prompt, Some(&system_prompt))?;

    println!(
        "\n{}",
        format!("Generated {} Code", language.to_uppercase()).blue().bold()
    );
    print_markdown(&response);

    // Extract and save the generated code if requested
    if let Some(save_path) = save {
        let code_blocks = extract_code_blocks(&response);
        if !code_blocks.is_empty() {
            fs::write(&save_path, &code_blocks[0]).context("Failed to save generated code")?;
            println!(
                "\n{}",
                format!("Generated code saved to {}", save_path.display()).green()
            );
        } else {
            println!("{}", "No code blocks found in the response".yellow());
        }
    }

    Ok(())
}

fn cmd_cloud(prompt: Vec<String>, provider: String, save: Option<PathBuf>) -> Result<()> {
    let full_prompt = prompt.join(" ");

    let prompt = format!(
        "
    Generate {} cloud deployment code/instructions for:
    
    {}
    
    Include:
    1. Required resources and services
    2. Infrastructure as Code (if applicable)
    3. Deployment steps
    4. Security considerations
    5. Cost optimization tips
    ",
        provider.to_uppercase(),
        full_prompt
    );

    let system_prompt = format!(
        "
    You are NimbusCode, an expert in cloud architecture and deployment on {}.
    Provide detailed, practical guidance for deploying applications to the cloud.
    Focus on security best practices, cost optimization, and maintainability.
    ",
        provider.to_uppercase()
    );

    println!("{}", "Generating cloud deployment plan...".green());
    let response = query_openrouter(&prompt, Some(&system_prompt))?;

    println!(
        "\n{}",
        format!("{} Deployment Plan", provider.to_uppercase()).blue().bold()
    );
    print_markdown(&response);

    // Save the response if requested
    if let Some(save_path) = save {
        fs::write(&save_path, &response).context("Failed to save deployment plan")?;
        println!(
            "\n{}",
            format!("Deployment plan saved to {}", save_path.display()).green()
        );
    }

    Ok(())
}

fn cmd_mobile(prompt: Vec<String>, platform: String, save: Option<PathBuf>) -> Result<()> {
    let full_prompt = prompt.join(" ");

    let platform_map = HashMap::from([
        ("ios", "iOS (Swift/SwiftUI)"),
        ("android", "Android (Kotlin)"),
        ("cross", "cross-platform (React Native/Flutter)"),
    ]);

    let platform_display = platform_map
        .get(platform.as_str())
        .unwrap_or(&platform.as_str());

    let prompt = format!(
        "
    Generate {} mobile app development code/guidance for:
    
    {}
    
    Include:
    1. App architecture
    2. Key components/screens
    3. Implementation details
    4. Best practices
    5. Performance considerations
    ",
        platform_display, full_prompt
    );

    let system_prompt = format!(
        "
    You are NimbusCode, an expert in {} mobile app development.
    Provide detailed, practical guidance for building mobile applications.
    Focus on user experience, performance, and maintainable code architecture.
    ",
        platform_display
    );

    println!("{}", "Generating mobile app guidance...".green());
    let response = query_openrouter(&prompt, Some(&system_prompt))?;

    println!(
        "\n{}",
        format!("{} App Development", platform_display).blue().bold()
    );
    print_markdown(&response);

    // Save the response if requested
    if let Some(save_path) = save {
        fs::write(&save_path, &response).context("Failed to save mobile app guidance")?;
        println!(
            "\n{}",
            format!("Mobile app guidance saved to {}", save_path.display()).green()
        );
    }

    Ok(())
}

fn cmd_interactive() -> Result<()> {
    println!("{}", "NimbusCode Interactive Mode".blue().bold());
    println!("Type your questions or 'exit' to quit.");

    let mut history = Vec::new();

    let system_prompt = "
    You are NimbusCode, an expert programming assistant in an interactive session.
    Provide helpful, concise responses to the user's coding questions.
    Remember the context of the conversation and refer back to previous exchanges when relevant.
    ";

    loop {
        print!("\n{} ", "You:".green().bold());
        io::stdout().flush().context("Failed to flush stdout")?;

        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .context("Failed to read input")?;

        let user_input = user_input.trim();

        if user_input.to_lowercase() == "exit"
            || user_input.to_lowercase() == "quit"
            || user_input.to_lowercase() == "q"
        {
            break;
        }

        // Add to conversation history
        history.push(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        // Prepare the full conversation context
        let full_prompt = history
            .iter()
            .map(|msg| {
                format!(
                    "{}: {}",
                    if msg.role == "user" { "User" } else { "Assistant" },
                    msg.content
                )
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        println!("{}", "Thinking...".green());
        let response = query_openrouter(&full_prompt, Some(system_prompt))?;

        // Add response to history
        history.push(Message {
            role: "assistant".to_string(),
            content: response.clone(),
        });

        // Display the response
        println!("\n{}", "NimbusCode:".blue().bold());
        print_markdown(&response);
    }

    println!("{}", "Exiting interactive mode...".yellow());
    Ok(())
}
