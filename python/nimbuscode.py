#!/usr/bin/env python3
"""
NimbusCode - A lightweight, portable AI coding assistant powered by OpenRouter's free models.
"""

import argparse
import json
import os
import sys
import configparser
import textwrap
from pathlib import Path
import requests
from typing import Optional, Dict, Any, List

CONFIG_DIR = Path.home() / ".config" / "nimbuscode"
CONFIG_FILE = CONFIG_DIR / "config.ini"
DEFAULT_MODEL = "mistralai/mistral-7b-instruct:free"
API_URL = "https://openrouter.ai/api/v1/chat/completions"

class NimbusCode:
    def __init__(self):
        self.config = self._load_config()
        self.api_key = self._get_api_key()
        
    def _load_config(self) -> configparser.ConfigParser:
        """Load configuration from config file."""
        config = configparser.ConfigParser()
        if CONFIG_FILE.exists():
            config.read(CONFIG_FILE)
        if "DEFAULT" not in config:
            config["DEFAULT"] = {}
        if "API" not in config:
            config["API"] = {}
        return config
    
    def _save_config(self) -> None:
        """Save configuration to config file."""
        CONFIG_DIR.mkdir(parents=True, exist_ok=True)
        with open(CONFIG_FILE, "w") as f:
            self.config.write(f)
    
    def _get_api_key(self) -> Optional[str]:
        """Get API key from environment variable or config file."""
        api_key = os.environ.get("OPENROUTER_API_KEY")
        if not api_key and "api_key" in self.config["API"]:
            api_key = self.config["API"]["api_key"]
        return api_key
    
    def set_api_key(self, api_key: str) -> None:
        """Set API key in config file."""
        self.config["API"]["api_key"] = api_key
        self._save_config()
        self.api_key = api_key
        print("API key saved successfully.")
    
    def _make_request(self, messages: List[Dict[str, str]], model: str = None) -> Dict[str, Any]:
        """Make a request to the OpenRouter API."""
        if not self.api_key:
            print("Error: API key not set. Use 'nimbuscode config --api-key YOUR_API_KEY' or set the OPENROUTER_API_KEY environment variable.")
            sys.exit(1)
        
        if not model:
            model = self.config["API"].get("default_model", DEFAULT_MODEL)
        
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
            "HTTP-Referer": "https://github.com/naelmohammad/nimbuscode",
            "X-Title": "NimbusCode"
        }
        
        data = {
            "model": model,
            "messages": messages
        }
        
        try:
            response = requests.post(API_URL, headers=headers, json=data)
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            print(f"Error: Failed to communicate with OpenRouter API: {e}")
            sys.exit(1)
    
    def ask(self, question: str, model: str = None) -> str:
        """Ask a coding question."""
        messages = [
            {"role": "system", "content": "You are a helpful coding assistant. Provide concise, accurate answers to coding questions."},
            {"role": "user", "content": question}
        ]
        
        response = self._make_request(messages, model)
        return response["choices"][0]["message"]["content"]
    
    def generate(self, description: str, language: str = None, model: str = None) -> str:
        """Generate code based on a description."""
        content = f"Generate code for: {description}"
        if language:
            content += f"\nLanguage: {language}"
        
        messages = [
            {"role": "system", "content": "You are a code generator. Create clean, efficient, and well-documented code based on descriptions."},
            {"role": "user", "content": content}
        ]
        
        response = self._make_request(messages, model)
        return response["choices"][0]["message"]["content"]
    
    def improve(self, code: str, model: str = None) -> str:
        """Improve existing code."""
        messages = [
            {"role": "system", "content": "You are a code reviewer. Suggest improvements to make the code more efficient, readable, and maintainable."},
            {"role": "user", "content": f"Improve this code:\n\n```\n{code}\n```"}
        ]
        
        response = self._make_request(messages, model)
        return response["choices"][0]["message"]["content"]
    
    def explain(self, code: str, model: str = None) -> str:
        """Explain code."""
        messages = [
            {"role": "system", "content": "You are a code explainer. Break down complex code into understandable explanations."},
            {"role": "user", "content": f"Explain this code:\n\n```\n{code}\n```"}
        ]
        
        response = self._make_request(messages, model)
        return response["choices"][0]["message"]["content"]
    
    def cloud(self, description: str, provider: str = "aws", model: str = None) -> str:
        """Generate cloud deployment guidance."""
        messages = [
            {"role": "system", "content": "You are a cloud deployment expert. Provide clear instructions for deploying applications to cloud platforms."},
            {"role": "user", "content": f"Provide deployment instructions for {provider} for: {description}"}
        ]
        
        response = self._make_request(messages, model)
        return response["choices"][0]["message"]["content"]
    
    def mobile(self, description: str, platform: str = "cross", model: str = None) -> str:
        """Generate mobile development guidance."""
        messages = [
            {"role": "system", "content": "You are a mobile development expert. Provide guidance for building mobile applications."},
            {"role": "user", "content": f"Provide {platform} platform mobile development guidance for: {description}"}
        ]
        
        response = self._make_request(messages, model)
        return response["choices"][0]["message"]["content"]
    
    def interactive(self, model: str = None) -> None:
        """Start an interactive session."""
        print("NimbusCode Interactive Mode (type 'exit' to quit)")
        print("------------------------------------------------")
        
        messages = [
            {"role": "system", "content": "You are a helpful coding assistant. Provide concise, accurate answers to coding questions."}
        ]
        
        while True:
            try:
                user_input = input("\n> ")
                if user_input.lower() in ("exit", "quit"):
                    break
                
                messages.append({"role": "user", "content": user_input})
                response = self._make_request(messages, model)
                assistant_response = response["choices"][0]["message"]["content"]
                
                print("\n" + assistant_response)
                messages.append({"role": "assistant", "content": assistant_response})
                
            except KeyboardInterrupt:
                print("\nExiting interactive mode.")
                break
    
    def list_models(self) -> None:
        """List available free models from OpenRouter."""
        if not self.api_key:
            print("Error: API key not set. Use 'nimbuscode config --api-key YOUR_API_KEY' or set the OPENROUTER_API_KEY environment variable.")
            sys.exit(1)
        
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        
        try:
            response = requests.get("https://openrouter.ai/api/v1/models", headers=headers)
            response.raise_for_status()
            models = response.json()["data"]
            
            print("Available Free Models:")
            print("---------------------")
            
            free_models = [model for model in models if model.get("pricing", {}).get("prompt") == 0 and model.get("pricing", {}).get("completion") == 0]
            
            if not free_models:
                print("No free models available.")
                return
            
            for model in free_models:
                print(f"ID: {model['id']}")
                print(f"Name: {model['name']}")
                print(f"Context Length: {model.get('context_length', 'Unknown')}")
                print("---------------------")
            
        except requests.exceptions.RequestException as e:
            print(f"Error: Failed to fetch models: {e}")
            sys.exit(1)

def main():
    nimbus = NimbusCode()
    
    parser = argparse.ArgumentParser(description="NimbusCode - A lightweight AI coding assistant")
    subparsers = parser.add_subparsers(dest="command", help="Command to execute")
    
    # Config command
    config_parser = subparsers.add_parser("config", help="Configure NimbusCode")
    config_parser.add_argument("--api-key", help="Set OpenRouter API key")
    
    # Ask command
    ask_parser = subparsers.add_parser("ask", help="Ask a coding question")
    ask_parser.add_argument("question", help="The question to ask")
    ask_parser.add_argument("--model", help="Specify the model to use")
    
    # Generate command
    generate_parser = subparsers.add_parser("generate", help="Generate code")
    generate_parser.add_argument("description", help="Description of the code to generate")
    generate_parser.add_argument("--language", help="Programming language")
    generate_parser.add_argument("--model", help="Specify the model to use")
    generate_parser.add_argument("--save", help="Save output to file")
    
    # Improve command
    improve_parser = subparsers.add_parser("improve", help="Improve existing code")
    improve_parser.add_argument("file", help="File containing code to improve")
    improve_parser.add_argument("--model", help="Specify the model to use")
    improve_parser.add_argument("--save", help="Save output to file")
    
    # Explain command
    explain_parser = subparsers.add_parser("explain", help="Explain code")
    explain_parser.add_argument("file", help="File containing code to explain")
    explain_parser.add_argument("--model", help="Specify the model to use")
    
    # Cloud command
    cloud_parser = subparsers.add_parser("cloud", help="Get cloud deployment guidance")
    cloud_parser.add_argument("description", help="Description of the deployment")
    cloud_parser.add_argument("--provider", choices=["aws", "azure", "gcp"], default="aws", help="Cloud provider")
    cloud_parser.add_argument("--model", help="Specify the model to use")
    
    # Mobile command
    mobile_parser = subparsers.add_parser("mobile", help="Get mobile development guidance")
    mobile_parser.add_argument("description", help="Description of the mobile app")
    mobile_parser.add_argument("--platform", choices=["ios", "android", "cross"], default="cross", help="Mobile platform")
    mobile_parser.add_argument("--model", help="Specify the model to use")
    
    # Interactive command
    interactive_parser = subparsers.add_parser("interactive", help="Start interactive mode")
    interactive_parser.add_argument("--model", help="Specify the model to use")
    
    # Models command
    models_parser = subparsers.add_parser("models", help="List available free models")
    
    args = parser.parse_args()
    
    if args.command == "config":
        if args.api_key:
            nimbus.set_api_key(args.api_key)
        else:
            parser.print_help()
    
    elif args.command == "ask":
        response = nimbus.ask(args.question, args.model)
        print(textwrap.fill(response, width=80))
    
    elif args.command == "generate":
        response = nimbus.generate(args.description, args.language, args.model)
        if args.save:
            with open(args.save, "w") as f:
                f.write(response)
            print(f"Code saved to {args.save}")
        else:
            print(response)
    
    elif args.command == "improve":
        with open(args.file, "r") as f:
            code = f.read()
        response = nimbus.improve(code, args.model)
        if args.save:
            with open(args.save, "w") as f:
                f.write(response)
            print(f"Improved code saved to {args.save}")
        else:
            print(response)
    
    elif args.command == "explain":
        with open(args.file, "r") as f:
            code = f.read()
        response = nimbus.explain(code, args.model)
        print(textwrap.fill(response, width=80))
    
    elif args.command == "cloud":
        response = nimbus.cloud(args.description, args.provider, args.model)
        print(textwrap.fill(response, width=80))
    
    elif args.command == "mobile":
        response = nimbus.mobile(args.description, args.platform, args.model)
        print(textwrap.fill(response, width=80))
    
    elif args.command == "interactive":
        nimbus.interactive(args.model)
    
    elif args.command == "models":
        nimbus.list_models()
    
    else:
        parser.print_help()

if __name__ == "__main__":
    main()
