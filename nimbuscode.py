#!/usr/bin/env python3
"""
NimbusCode - Lightweight AI coding assistant using OpenRouter's free models.
"""

import os
import sys
import json
import textwrap
from pathlib import Path
from typing import Optional, List, Dict, Any

import typer
import requests
from rich.console import Console
from rich.markdown import Markdown
from rich.panel import Panel
from rich.syntax import Syntax
from rich.prompt import Prompt
from dotenv import load_dotenv

# Initialize Typer app
app = typer.Typer(help="NimbusCode - AI coding assistant")
console = Console()

# Load environment variables
load_dotenv()

# Constants
CONFIG_DIR = Path.home() / ".nimbuscode"
CONFIG_FILE = CONFIG_DIR / "config.json"
DEFAULT_MODEL = "openrouter/auto"  # Will select the best free model automatically
DEFAULT_CONTEXT_WINDOW = 8192  # Default context window size

def ensure_config_dir():
    """Ensure the configuration directory exists."""
    CONFIG_DIR.mkdir(exist_ok=True)
    
    if not CONFIG_FILE.exists():
        default_config = {
            "api_key": "",
            "model": DEFAULT_MODEL,
            "max_tokens": 1024,
            "temperature": 0.7,
        }
        with open(CONFIG_FILE, "w") as f:
            json.dump(default_config, f, indent=2)
        console.print("[yellow]Config file created at ~/.nimbuscode/config.json[/yellow]")
        console.print("[yellow]Please set your OpenRouter API key with 'nimbuscode config --api-key YOUR_API_KEY'[/yellow]")

def load_config() -> Dict[str, Any]:
    """Load configuration from file."""
    ensure_config_dir()
    try:
        with open(CONFIG_FILE, "r") as f:
            return json.load(f)
    except Exception as e:
        console.print(f"[red]Error loading config: {e}[/red]")
        return {
            "api_key": os.environ.get("OPENROUTER_API_KEY", ""),
            "model": DEFAULT_MODEL,
            "max_tokens": 1024,
            "temperature": 0.7,
        }

def save_config(config: Dict[str, Any]):
    """Save configuration to file."""
    ensure_config_dir()
    with open(CONFIG_FILE, "w") as f:
        json.dump(config, f, indent=2)

def get_api_key() -> str:
    """Get the API key from config or environment."""
    config = load_config()
    api_key = config.get("api_key") or os.environ.get("OPENROUTER_API_KEY", "")
    if not api_key:
        console.print("[red]API key not found. Please set it with 'nimbuscode config --api-key YOUR_API_KEY'[/red]")
        sys.exit(1)
    return api_key

def query_openrouter(prompt: str, system_prompt: str = None) -> str:
    """Query the OpenRouter API with the given prompt."""
    config = load_config()
    api_key = get_api_key()
    
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
        "HTTP-Referer": "https://github.com/cline/cline",  # Required for OpenRouter
    }
    
    messages = []
    if system_prompt:
        messages.append({"role": "system", "content": system_prompt})
    messages.append({"role": "user", "content": prompt})
    
    data = {
        "model": config.get("model", DEFAULT_MODEL),
        "messages": messages,
        "max_tokens": config.get("max_tokens", 1024),
        "temperature": config.get("temperature", 0.7),
    }
    
    try:
        response = requests.post(
            "https://openrouter.ai/api/v1/chat/completions",
            headers=headers,
            json=data,
        )
        response.raise_for_status()
        result = response.json()
        return result["choices"][0]["message"]["content"]
    except Exception as e:
        console.print(f"[red]Error querying OpenRouter API: {e}[/red]")
        if hasattr(e, "response") and hasattr(e.response, "text"):
            console.print(f"[red]Response: {e.response.text}[/red]")
        return f"Error: {str(e)}"

def extract_code_blocks(markdown_text: str) -> List[str]:
    """Extract code blocks from markdown text."""
    code_blocks = []
    lines = markdown_text.split('\n')
    in_code_block = False
    current_block = []
    
    for line in lines:
        if line.startswith('```'):
            if in_code_block:
                code_blocks.append('\n'.join(current_block))
                current_block = []
                in_code_block = False
            else:
                in_code_block = True
                # Skip the language identifier line
                continue
        elif in_code_block:
            current_block.append(line)
    
    return code_blocks

@app.command("ask")
def ask(
    prompt: List[str] = typer.Argument(..., help="The prompt to send to the AI"),
    file: Optional[Path] = typer.Option(None, "--file", "-f", help="File to use as context"),
    system: Optional[str] = typer.Option(None, "--system", "-s", help="System prompt to use"),
    save: Optional[Path] = typer.Option(None, "--save", help="Save the response to a file"),
    extract: bool = typer.Option(False, "--extract", "-e", help="Extract and save code blocks"),
):
    """Ask the AI a coding question."""
    full_prompt = " ".join(prompt)
    
    # Add file content to prompt if specified
    if file and file.exists():
        with open(file, "r") as f:
            file_content = f.read()
        full_prompt = f"File content:\n```\n{file_content}\n```\n\nPrompt: {full_prompt}"
    
    # Default system prompt for coding assistance
    default_system_prompt = """
    You are NimbusCode, an expert programming assistant. Your goal is to help the user write high-quality, 
    efficient, and secure code. Provide clear, concise explanations and code examples.
    Focus on best practices, performance optimization, and security considerations.
    When appropriate, suggest improvements to the user's code or approach.
    """
    
    system_prompt = system or default_system_prompt
    
    with console.status("[bold green]Thinking..."):
        response = query_openrouter(full_prompt, system_prompt)
    
    # Display the response
    console.print(Panel(Markdown(response), title="NimbusCode", border_style="blue"))
    
    # Save response if requested
    if save:
        with open(save, "w") as f:
            f.write(response)
        console.print(f"[green]Response saved to {save}[/green]")
    
    # Extract and save code blocks if requested
    if extract:
        code_blocks = extract_code_blocks(response)
        if code_blocks:
            for i, block in enumerate(code_blocks):
                filename = f"code_block_{i+1}.txt"
                with open(filename, "w") as f:
                    f.write(block)
                console.print(f"[green]Code block saved to {filename}[/green]")
        else:
            console.print("[yellow]No code blocks found in the response[/yellow]")

@app.command("config")
def config(
    api_key: Optional[str] = typer.Option(None, "--api-key", help="Set the OpenRouter API key"),
    model: Optional[str] = typer.Option(None, "--model", help="Set the default model"),
    max_tokens: Optional[int] = typer.Option(None, "--max-tokens", help="Set the maximum tokens"),
    temperature: Optional[float] = typer.Option(None, "--temperature", help="Set the temperature"),
    show: bool = typer.Option(False, "--show", help="Show the current configuration"),
):
    """Configure NimbusCode settings."""
    current_config = load_config()
    
    if show:
        # Hide API key for security
        display_config = current_config.copy()
        if "api_key" in display_config and display_config["api_key"]:
            display_config["api_key"] = "********" + display_config["api_key"][-4:]
        console.print(json.dumps(display_config, indent=2))
        return
    
    if api_key:
        current_config["api_key"] = api_key
    
    if model:
        current_config["model"] = model
    
    if max_tokens is not None:
        current_config["max_tokens"] = max_tokens
    
    if temperature is not None:
        current_config["temperature"] = temperature
    
    save_config(current_config)
    console.print("[green]Configuration updated successfully[/green]")

@app.command("models")
def list_models():
    """List available models from OpenRouter."""
    api_key = get_api_key()
    
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
    }
    
    try:
        with console.status("[bold green]Fetching available models..."):
            response = requests.get(
                "https://openrouter.ai/api/v1/models",
                headers=headers,
            )
            response.raise_for_status()
            models = response.json()
        
        console.print("[bold]Available Models:[/bold]")
        
        # Filter for free models
        free_models = []
        for model in models["data"]:
            if "pricing" in model and model["pricing"].get("prompt", 0) == 0 and model["pricing"].get("completion", 0) == 0:
                free_models.append(model)
        
        if free_models:
            console.print("[bold green]Free Models:[/bold green]")
            for model in free_models:
                console.print(f"  [green]{model['id']}[/green]")
                console.print(f"    Context: {model.get('context_length', 'Unknown')}")
                console.print(f"    Description: {model.get('description', 'No description')}")
                console.print("")
        else:
            console.print("[yellow]No free models found[/yellow]")
            
    except Exception as e:
        console.print(f"[red]Error fetching models: {e}[/red]")

@app.command("improve")
def improve_code(
    file: Path = typer.Argument(..., help="File containing code to improve"),
    save: Optional[Path] = typer.Option(None, "--save", help="Save the improved code to a file"),
):
    """Improve existing code with AI suggestions."""
    if not file.exists():
        console.print(f"[red]File {file} does not exist[/red]")
        return
    
    with open(file, "r") as f:
        code = f.read()
    
    prompt = f"""
    Please improve the following code. Focus on:
    1. Code quality and readability
    2. Performance optimizations
    3. Security best practices
    4. Error handling
    5. Documentation
    
    Provide the improved code and explain your changes.
    
    ```
    {code}
    ```
    """
    
    system_prompt = """
    You are NimbusCode, an expert code reviewer and optimizer. Analyze the provided code and suggest
    improvements. Return the improved code in a markdown code block with the same language as the original.
    Explain your changes clearly but concisely.
    """
    
    with console.status("[bold green]Analyzing and improving code..."):
        response = query_openrouter(prompt, system_prompt)
    
    console.print(Panel(Markdown(response), title=f"Code Improvements for {file.name}", border_style="blue"))
    
    # Extract and save the improved code if requested
    if save:
        code_blocks = extract_code_blocks(response)
        if code_blocks:
            with open(save, "w") as f:
                f.write(code_blocks[0])  # Save the first code block
            console.print(f"[green]Improved code saved to {save}[/green]")
        else:
            console.print("[yellow]No code blocks found in the response[/yellow]")

@app.command("explain")
def explain_code(
    file: Path = typer.Argument(..., help="File containing code to explain"),
):
    """Explain code with detailed comments and documentation."""
    if not file.exists():
        console.print(f"[red]File {file} does not exist[/red]")
        return
    
    with open(file, "r") as f:
        code = f.read()
    
    prompt = f"""
    Please explain the following code in detail:
    
    ```
    {code}
    ```
    
    Include:
    1. Overall purpose and functionality
    2. Breakdown of key components
    3. How the different parts work together
    4. Any potential issues or considerations
    """
    
    system_prompt = """
    You are NimbusCode, an expert code analyst. Provide a clear, educational explanation of the code.
    Break down complex concepts and use examples where helpful. Your goal is to help the user fully
    understand how the code works.
    """
    
    with console.status("[bold green]Analyzing code..."):
        response = query_openrouter(prompt, system_prompt)
    
    console.print(Panel(Markdown(response), title=f"Code Explanation for {file.name}", border_style="blue"))

@app.command("generate")
def generate_code(
    prompt: List[str] = typer.Argument(..., help="Description of the code to generate"),
    language: str = typer.Option("python", "--language", "-l", help="Programming language"),
    save: Optional[Path] = typer.Option(None, "--save", help="Save the generated code to a file"),
):
    """Generate code based on a description."""
    full_prompt = " ".join(prompt)
    
    prompt = f"""
    Generate {language} code for the following:
    
    {full_prompt}
    
    Provide complete, working code with appropriate comments and documentation.
    """
    
    system_prompt = f"""
    You are NimbusCode, an expert {language} developer. Generate high-quality, efficient, and secure code
    based on the user's requirements. Include helpful comments and documentation. Focus on best practices
    and maintainability.
    """
    
    with console.status("[bold green]Generating code..."):
        response = query_openrouter(prompt, system_prompt)
    
    console.print(Panel(Markdown(response), title=f"Generated {language.capitalize()} Code", border_style="blue"))
    
    # Extract and save the generated code if requested
    if save:
        code_blocks = extract_code_blocks(response)
        if code_blocks:
            with open(save, "w") as f:
                f.write(code_blocks[0])  # Save the first code block
            console.print(f"[green]Generated code saved to {save}[/green]")
        else:
            console.print("[yellow]No code blocks found in the response[/yellow]")

@app.command("cloud")
def cloud_deployment(
    prompt: List[str] = typer.Argument(..., help="Description of the cloud deployment"),
    provider: str = typer.Option("aws", "--provider", "-p", help="Cloud provider (aws, azure, gcp)"),
    save: Optional[Path] = typer.Option(None, "--save", help="Save the deployment code to a file"),
):
    """Generate cloud deployment code or instructions."""
    full_prompt = " ".join(prompt)
    
    prompt = f"""
    Generate {provider.upper()} cloud deployment code/instructions for:
    
    {full_prompt}
    
    Include:
    1. Required resources and services
    2. Infrastructure as Code (if applicable)
    3. Deployment steps
    4. Security considerations
    5. Cost optimization tips
    """
    
    system_prompt = f"""
    You are NimbusCode, an expert in cloud architecture and deployment on {provider.upper()}.
    Provide detailed, practical guidance for deploying applications to the cloud.
    Focus on security best practices, cost optimization, and maintainability.
    """
    
    with console.status("[bold green]Generating cloud deployment plan..."):
        response = query_openrouter(prompt, system_prompt)
    
    console.print(Panel(Markdown(response), title=f"{provider.upper()} Deployment Plan", border_style="blue"))
    
    # Save the response if requested
    if save:
        with open(save, "w") as f:
            f.write(response)
        console.print(f"[green]Deployment plan saved to {save}[/green]")

@app.command("mobile")
def mobile_development(
    prompt: List[str] = typer.Argument(..., help="Description of the mobile app"),
    platform: str = typer.Option("cross", "--platform", "-p", help="Mobile platform (ios, android, cross)"),
    save: Optional[Path] = typer.Option(None, "--save", help="Save the generated code to a file"),
):
    """Generate mobile app development code or guidance."""
    full_prompt = " ".join(prompt)
    
    platform_map = {
        "ios": "iOS (Swift/SwiftUI)",
        "android": "Android (Kotlin)",
        "cross": "cross-platform (React Native/Flutter)"
    }
    
    platform_display = platform_map.get(platform.lower(), platform)
    
    prompt = f"""
    Generate {platform_display} mobile app development code/guidance for:
    
    {full_prompt}
    
    Include:
    1. App architecture
    2. Key components/screens
    3. Implementation details
    4. Best practices
    5. Performance considerations
    """
    
    system_prompt = f"""
    You are NimbusCode, an expert in {platform_display} mobile app development.
    Provide detailed, practical guidance for building mobile applications.
    Focus on user experience, performance, and maintainable code architecture.
    """
    
    with console.status("[bold green]Generating mobile app guidance..."):
        response = query_openrouter(prompt, system_prompt)
    
    console.print(Panel(Markdown(response), title=f"{platform_display.capitalize()} App Development", border_style="blue"))
    
    # Save the response if requested
    if save:
        with open(save, "w") as f:
            f.write(response)
        console.print(f"[green]Mobile app guidance saved to {save}[/green]")

@app.command("interactive")
def interactive_mode():
    """Start an interactive coding session with the AI."""
    console.print("[bold blue]NimbusCode Interactive Mode[/bold blue]")
    console.print("Type your questions or 'exit' to quit.")
    
    history = []
    
    system_prompt = """
    You are NimbusCode, an expert programming assistant in an interactive session.
    Provide helpful, concise responses to the user's coding questions.
    Remember the context of the conversation and refer back to previous exchanges when relevant.
    """
    
    while True:
        try:
            user_input = Prompt.ask("\n[bold green]You[/bold green]")
            
            if user_input.lower() in ("exit", "quit", "q"):
                break
            
            # Add to conversation history
            history.append({"role": "user", "content": user_input})
            
            # Prepare the full conversation context
            full_prompt = "\n\n".join([
                f"{'User' if msg['role'] == 'user' else 'Assistant'}: {msg['content']}"
                for msg in history
            ])
            
            with console.status("[bold green]Thinking..."):
                response = query_openrouter(full_prompt, system_prompt)
            
            # Add response to history
            history.append({"role": "assistant", "content": response})
            
            # Display the response
            console.print("\n[bold blue]NimbusCode:[/bold blue]")
            console.print(Markdown(response))
            
        except KeyboardInterrupt:
            console.print("\n[yellow]Exiting interactive mode...[/yellow]")
            break
        except Exception as e:
            console.print(f"[red]Error: {e}[/red]")

if __name__ == "__main__":
    app()
