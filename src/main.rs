use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use configparser::ini::Ini;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use textwrap::fill;

const API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const DEFAULT_MODEL: &str = "mistralai/mistral-7b-instruct:free";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure NimbusCode
    Config {
        /// Set OpenRouter API key
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Ask a coding question
    Ask {
        /// The question to ask
        question: String,
        /// Specify the model to use
        #[arg(long)]
        model: Option<String>,
    },
    /// Generate code
    Generate {
        /// Description of the code to generate
        description: String,
        /// Programming language
        #[arg(long)]
        language: Option<String>,
        /// Specify the model to use
        #[arg(long)]
        model: Option<String>,
        /// Save output to file
        #[arg(long)]
        save: Option<String>,
    },
    /// Improve existing code
    Improve {
        /// File containing code to improve
        file: String,
        /// Specify the model to use
        #[arg(long)]
        model: Option<String>,
        /// Save output to file
        #[arg(long)]
        save: Option<String>,
    },
    /// Explain code
    Explain {
        /// File containing code to explain
        file: String,
        /// Specify the model to use
        #[arg(long)]
        model: Option<String>,
    },
    /// Get cloud deployment guidance
    Cloud {
        /// Description of the deployment
        description: String,
        /// Cloud provider
        #[arg(long, default_value = "aws")]
        provider: String,
        /// Specify the model to use
        #[arg(long)]
        model: Option<String>,
    },
    /// Get mobile development guidance
    Mobile {
        /// Description of the mobile app
        description: String,
        /// Mobile platform
        #[arg(long, default_value = "cross")]
        platform: String,
        /// Specify the model to use
        #[arg(long)]
        model: Option<String>,
    },
    /// Start interactive mode
    Interactive {
        /// Specify the model to use
        #[arg(long)]
        model: Option<String>,
    },
    /// List available free models
    Models,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

struct NimbusCode {
    config: Ini,
    api_key: Option<String>,
}

impl NimbusCode {
    fn new() -> Result<Self> {
        let mut config = Ini::new();
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("nimbuscode");
        let config_file = config_dir.join("config.ini");

        if config_file.exists() {
            config.load(&config_file).context("Failed to load config file")?;
        }

        let api_key = env::var("OPENROUTER_API_KEY").ok().or_else(|| {
            config
                .get("API", "api_key")
                .filter(|s| !s.is_empty())
        });

        Ok(Self { config, api_key })
    }

    fn save_config(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("nimbuscode");
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        let config_file = config_dir.join("config.ini");
        self.config
            .write(&config_file)
            .context("Failed to write config file")?;
        Ok(())
    }

    fn set_api_key(&mut self, api_key: &str) -> Result<()> {
        self.config.set("API", "api_key", Some(api_key.to_string()));
        self.save_config()?;
        self.api_key = Some(api_key.to_string());
        println!("API key saved successfully.");
        Ok(())
    }

    async fn make_request(
        &self,
        messages: Vec<Message>,
        model: Option<&str>,
    ) -> Result<ChatResponse> {
        let api_key = self.api_key.as_ref().context("API key not set. Use 'nimbuscode config --api-key YOUR_API_KEY' or set the OPENROUTER_API_KEY environment variable.")?;
        
        let model = model.unwrap_or_else(|| {
            self.config
                .get("API", "default_model")
                .unwrap_or_else(|| DEFAULT_MODEL.to_string())
                .as_str()
        });

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "HTTP-Referer",
            HeaderValue::from_static("https://github.com/naelmohammad/nimbuscode"),
        );
        headers.insert("X-Title", HeaderValue::from_static("NimbusCode"));

        let client = reqwest::Client::new();
        let response = client
            .post(API_URL)
            .headers(headers)
            .json(&ChatRequest {
                model: model.to_string(),
                messages,
            })
            .send()
            .await
            .context("Failed to send request to OpenRouter API")?;

        let response_status = response.status();
        if !response_status.is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("API request failed with status {}: {}", response_status, error_text);
        }

        let chat_response = response
            .json::<ChatResponse>()
            .await
            .context("Failed to parse API response")?;

        Ok(chat_response)
    }

    async fn ask(&self, question: &str, model: Option<&str>) -> Result<String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful coding assistant. Provide concise, accurate answers to coding questions.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: question.to_string(),
            },
        ];

        let response = self.make_request(messages, model).await?;
        Ok(response.choices[0].message.content.clone())
    }

    async fn generate(
        &self,
        description: &str,
        language: Option<&str>,
        model: Option<&str>,
    ) -> Result<String> {
        let mut content = format!("Generate code for: {}", description);
        if let Some(lang) = language {
            content = format!("{}\nLanguage: {}", content, lang);
        }

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a code generator. Create clean, efficient, and well-documented code based on descriptions.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content,
            },
        ];

        let response = self.make_request(messages, model).await?;
        Ok(response.choices[0].message.content.clone())
    }

    async fn improve(&self, code: &str, model: Option<&str>) -> Result<String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a code reviewer. Suggest improvements to make the code more efficient, readable, and maintainable.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!("Improve this code:\n\n```\n{}\n```", code),
            },
        ];

        let response = self.make_request(messages, model).await?;
        Ok(response.choices[0].message.content.clone())
    }

    async fn explain(&self, code: &str, model: Option<&str>) -> Result<String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a code explainer. Break down complex code into understandable explanations.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!("Explain this code:\n\n```\n{}\n```", code),
            },
        ];

        let response = self.make_request(messages, model).await?;
        Ok(response.choices[0].message.content.clone())
    }

    async fn cloud(
        &self,
        description: &str,
        provider: &str,
        model: Option<&str>,
    ) -> Result<String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a cloud deployment expert. Provide clear instructions for deploying applications to cloud platforms.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!("Provide deployment instructions for {} for: {}", provider, description),
            },
        ];

        let response = self.make_request(messages, model).await?;
        Ok(response.choices[0].message.content.clone())
    }

    async fn mobile(
        &self,
        description: &str,
        platform: &str,
        model: Option<&str>,
    ) -> Result<String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a mobile development expert. Provide guidance for building mobile applications.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!(
                    "Provide {} platform mobile development guidance for: {}",
                    platform, description
                ),
            },
        ];

        let response = self.make_request(messages, model).await?;
        Ok(response.choices[0].message.content.clone())
    }

    async fn interactive(&self, model: Option<&str>) -> Result<()> {
        println!("NimbusCode Interactive Mode (type 'exit' to quit)");
        println!("------------------------------------------------");

        let mut messages = vec![Message {
            role: "system".to_string(),
            content: "You are a helpful coding assistant. Provide concise, accurate answers to coding questions.".to_string(),
        }];

        loop {
            print!("\n> ");
            io::stdout().flush()?;

            let mut user_input = String::new();
            io::stdin().read_line(&mut user_input)?;
            let user_input = user_input.trim();

            if user_input.to_lowercase() == "exit" || user_input.to_lowercase() == "quit" {
                break;
            }

            messages.push(Message {
                role: "user".to_string(),
                content: user_input.to_string(),
            });

            let response = self.make_request(messages.clone(), model).await?;
            let assistant_response = &response.choices[0].message.content;

            println!("\n{}", assistant_response);

            messages.push(Message {
                role: "assistant".to_string(),
                content: assistant_response.clone(),
            });
        }

        Ok(())
    }

    async fn list_models(&self) -> Result<()> {
        let api_key = self.api_key.as_ref().context("API key not set. Use 'nimbuscode config --api-key YOUR_API_KEY' or set the OPENROUTER_API_KEY environment variable.")?;
        
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::new();
        let response = client
            .get("https://openrouter.ai/api/v1/models")
            .headers(headers)
            .send()
            .await
            .context("Failed to fetch models")?;

        let response_status = response.status();
        if !response_status.is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("API request failed with status {}: {}", response_status, error_text);
        }

        let models: Value = response.json().await?;
        let models = models["data"].as_array().context("Invalid response format")?;

        println!("Available Free Models:");
        println!("---------------------");

        let mut found_free_models = false;

        for model in models {
            let pricing = model.get("pricing").and_then(|p| p.as_object());
            
            if let Some(pricing) = pricing {
                let prompt_price = pricing.get("prompt").and_then(|p| p.as_f64()).unwrap_or(1.0);
                let completion_price = pricing.get("completion").and_then(|p| p.as_f64()).unwrap_or(1.0);
                
                if prompt_price == 0.0 && completion_price == 0.0 {
                    found_free_models = true;
                    
                    println!("ID: {}", model["id"].as_str().unwrap_or("Unknown"));
                    println!("Name: {}", model["name"].as_str().unwrap_or("Unknown"));
                    println!(
                        "Context Length: {}",
                        model.get("context_length")
                            .and_then(|c| c.as_u64())
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "Unknown".to_string())
                    );
                    println!("---------------------");
                }
            }
        }

        if !found_free_models {
            println!("No free models available.");
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut nimbus = NimbusCode::new()?;

    match cli.command {
        Commands::Config { api_key } => {
            if let Some(key) = api_key {
                nimbus.set_api_key(&key)?;
            } else {
                println!("Please provide an API key with --api-key");
            }
        }
        Commands::Ask { question, model } => {
            let response = nimbus.ask(&question, model.as_deref()).await?;
            println!("{}", fill(&response, 80));
        }
        Commands::Generate {
            description,
            language,
            model,
            save,
        } => {
            let response = nimbus
                .generate(&description, language.as_deref(), model.as_deref())
                .await?;
            if let Some(file_path) = save {
                let mut file = File::create(&file_path)?;
                file.write_all(response.as_bytes())?;
                println!("Code saved to {}", file_path);
            } else {
                println!("{}", response);
            }
        }
        Commands::Improve {
            file: file_path,
            model,
            save,
        } => {
            let mut file = File::open(&file_path)?;
            let mut code = String::new();
            file.read_to_string(&mut code)?;

            let response = nimbus.improve(&code, model.as_deref()).await?;
            if let Some(save_path) = save {
                let mut file = File::create(&save_path)?;
                file.write_all(response.as_bytes())?;
                println!("Improved code saved to {}", save_path);
            } else {
                println!("{}", response);
            }
        }
        Commands::Explain { file: file_path, model } => {
            let mut file = File::open(&file_path)?;
            let mut code = String::new();
            file.read_to_string(&mut code)?;

            let response = nimbus.explain(&code, model.as_deref()).await?;
            println!("{}", fill(&response, 80));
        }
        Commands::Cloud {
            description,
            provider,
            model,
        } => {
            let response = nimbus.cloud(&description, &provider, model.as_deref()).await?;
            println!("{}", fill(&response, 80));
        }
        Commands::Mobile {
            description,
            platform,
            model,
        } => {
            let response = nimbus.mobile(&description, &platform, model.as_deref()).await?;
            println!("{}", fill(&response, 80));
        }
        Commands::Interactive { model } => {
            nimbus.interactive(model.as_deref()).await?;
        }
        Commands::Models => {
            nimbus.list_models().await?;
        }
    }

    Ok(())
}
