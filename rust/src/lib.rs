use anyhow::{Context, Result};
use dirs::home_dir;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;

// Constants
pub const DEFAULT_MODEL: &str = "openrouter/auto";
pub const DEFAULT_MAX_TOKENS: u32 = 1024;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

// Types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
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
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenRouterRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenRouterChoice {
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenRouterResponse {
    pub choices: Vec<OpenRouterChoice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenRouterModel {
    pub id: String,
    pub context_length: Option<u32>,
    pub description: Option<String>,
    pub pricing: Option<OpenRouterPricing>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenRouterPricing {
    pub prompt: f32,
    pub completion: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenRouterModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

// Helper functions
pub fn get_config_dir() -> Result<PathBuf> {
    let home = home_dir().context("Could not determine home directory")?;
    Ok(home.join(".nimbuscode"))
}

pub fn get_config_file() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.json"))
}

pub fn ensure_config_dir() -> Result<()> {
    let config_dir = get_config_dir()?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }

    let config_file = get_config_file()?;
    if !config_file.exists() {
        let default_config = Config::default();
        save_config(&default_config)?;
    }

    Ok(())
}

pub fn load_config() -> Result<Config> {
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

pub fn save_config(config: &Config) -> Result<()> {
    let config_file = get_config_file()?;
    let json = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    let mut file = File::create(config_file).context("Failed to create config file")?;
    file.write_all(json.as_bytes())
        .context("Failed to write config file")?;
    Ok(())
}

pub fn get_api_key() -> Result<String> {
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
            "API key not found. Please set it with 'nimbuscode config --api-key YOUR_API_KEY'"
        );
        process::exit(1);
    }

    Ok(api_key)
}

pub fn query_openrouter(prompt: &str, system_prompt: Option<&str>) -> Result<String> {
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

pub fn extract_code_blocks(markdown_text: &str) -> Vec<String> {
    let re = regex::Regex::new(r"```(?:\w+)?\n([\s\S]*?)\n```").unwrap();
    re.captures_iter(markdown_text)
        .map(|cap| cap[1].to_string())
        .collect()
}
