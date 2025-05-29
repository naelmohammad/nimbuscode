#!/usr/bin/env python3
"""
Unit tests for NimbusCode Python implementation.
"""

import os
import sys
import json
import unittest
from unittest.mock import patch, MagicMock
from pathlib import Path

# Add parent directory to path to import nimbuscode
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
import nimbuscode

class TestNimbusCode(unittest.TestCase):
    """Test cases for NimbusCode."""

    def setUp(self):
        """Set up test environment."""
        # Create a temporary config directory and file for testing
        self.test_config_dir = Path("./test_config")
        self.test_config_file = self.test_config_dir / "config.json"
        
        # Override the config paths for testing
        nimbuscode.CONFIG_DIR = self.test_config_dir
        nimbuscode.CONFIG_FILE = self.test_config_file
        
        # Create the test directory
        self.test_config_dir.mkdir(exist_ok=True)
        
        # Create a test config file
        test_config = {
            "api_key": "test_api_key",
            "model": "test_model",
            "max_tokens": 500,
            "temperature": 0.5,
        }
        with open(self.test_config_file, "w") as f:
            json.dump(test_config, f)

    def tearDown(self):
        """Clean up after tests."""
        # Remove the test config file and directory
        if self.test_config_file.exists():
            self.test_config_file.unlink()
        if self.test_config_dir.exists():
            self.test_config_dir.rmdir()

    def test_load_config(self):
        """Test loading configuration from file."""
        config = nimbuscode.load_config()
        self.assertEqual(config["api_key"], "test_api_key")
        self.assertEqual(config["model"], "test_model")
        self.assertEqual(config["max_tokens"], 500)
        self.assertEqual(config["temperature"], 0.5)

    def test_save_config(self):
        """Test saving configuration to file."""
        new_config = {
            "api_key": "new_api_key",
            "model": "new_model",
            "max_tokens": 1000,
            "temperature": 0.7,
        }
        nimbuscode.save_config(new_config)
        
        # Load the config again to verify it was saved
        loaded_config = nimbuscode.load_config()
        self.assertEqual(loaded_config["api_key"], "new_api_key")
        self.assertEqual(loaded_config["model"], "new_model")
        self.assertEqual(loaded_config["max_tokens"], 1000)
        self.assertEqual(loaded_config["temperature"], 0.7)

    def test_get_api_key(self):
        """Test getting API key from config."""
        with patch.dict(os.environ, {"OPENROUTER_API_KEY": ""}):
            api_key = nimbuscode.get_api_key()
            self.assertEqual(api_key, "test_api_key")

    def test_get_api_key_from_env(self):
        """Test getting API key from environment variable."""
        # Remove the api_key from config
        config = nimbuscode.load_config()
        config["api_key"] = ""
        nimbuscode.save_config(config)
        
        # Set environment variable
        with patch.dict(os.environ, {"OPENROUTER_API_KEY": "env_api_key"}):
            api_key = nimbuscode.get_api_key()
            self.assertEqual(api_key, "env_api_key")

    @patch('nimbuscode.requests.post')
    def test_query_openrouter(self, mock_post):
        """Test querying the OpenRouter API."""
        # Mock the response from requests.post
        mock_response = MagicMock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {
            "choices": [
                {
                    "message": {
                        "content": "Test response"
                    }
                }
            ]
        }
        mock_post.return_value = mock_response
        
        # Call the function
        response = nimbuscode.query_openrouter("Test prompt", "Test system prompt")
        
        # Verify the response
        self.assertEqual(response, "Test response")
        
        # Verify the API was called with the correct parameters
        mock_post.assert_called_once()
        args, kwargs = mock_post.call_args
        self.assertEqual(args[0], "https://openrouter.ai/api/v1/chat/completions")
        self.assertIn("headers", kwargs)
        self.assertIn("json", kwargs)
        self.assertEqual(kwargs["json"]["messages"][0]["role"], "system")
        self.assertEqual(kwargs["json"]["messages"][0]["content"], "Test system prompt")
        self.assertEqual(kwargs["json"]["messages"][1]["role"], "user")
        self.assertEqual(kwargs["json"]["messages"][1]["content"], "Test prompt")

    def test_extract_code_blocks(self):
        """Test extracting code blocks from markdown text."""
        markdown_text = """
        Here is some code:
        
        ```python
        def hello():
            print("Hello, world!")
        ```
        
        And another block:
        
        ```javascript
        function hello() {
            console.log("Hello, world!");
        }
        ```
        """
        
        code_blocks = nimbuscode.extract_code_blocks(markdown_text)
        self.assertEqual(len(code_blocks), 2)
        self.assertEqual(code_blocks[0], '        def hello():\n            print("Hello, world!")')
        self.assertEqual(code_blocks[1], '        function hello() {\n            console.log("Hello, world!");\n        }')

if __name__ == "__main__":
    unittest.main()
