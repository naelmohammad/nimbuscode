# NimbusCode - Rust Implementation

A lightweight, portable AI coding assistant powered by OpenRouter's free models, implemented in Rust for maximum performance and minimal resource usage.

## Features

- **Minimal Footprint**: Uses minimal disk space and memory
- **Portable**: Works across platforms with minimal dependencies
- **Code Generation**: Create code based on natural language descriptions
- **Code Improvement**: Get suggestions to improve existing code
- **Code Explanation**: Understand complex code with detailed explanations
- **Cloud Deployment**: Generate deployment instructions for AWS, Azure, and GCP
- **Mobile Development**: Get guidance for iOS, Android, and cross-platform development
- **Interactive Mode**: Have a conversation with the AI about your coding questions

## Installation

```bash
# Clone the repository
git clone https://github.com/naelmohammad/nimbuscode-rust.git
cd nimbuscode-rust

# Build the Rust version
cargo build --release

# Optional: Create a symlink to make it available system-wide
ln -s $(pwd)/target/release/nimbuscode /usr/local/bin/nimbuscode
```

## Configuration

Before using NimbusCode, you need to set up your OpenRouter API key:

```bash
# Set your API key
./target/release/nimbuscode config --api-key YOUR_API_KEY

# Alternatively, set it as an environment variable
export OPENROUTER_API_KEY=your_api_key
```

## Usage

### Ask a coding question

```bash
./target/release/nimbuscode ask "How do I implement a binary search tree in Rust?"
```

### Generate code

```bash
./target/release/nimbuscode generate "A REST API for a todo list application" --language rust --save todo_api.rs
```

### Improve existing code

```bash
./target/release/nimbuscode improve my_script.rs --save improved_script.rs
```

### Explain code

```bash
./target/release/nimbuscode explain complex_algorithm.rs
```

### Cloud deployment guidance

```bash
./target/release/nimbuscode cloud "Deploy a containerized Rust application with a PostgreSQL database" --provider aws
```

### Mobile development guidance

```bash
./target/release/nimbuscode mobile "A fitness tracking app with social features" --platform cross
```

### Interactive mode

```bash
./target/release/nimbuscode interactive
```

### List available free models

```bash
./target/release/nimbuscode models
```

## Why NimbusCode in Rust?

- **Performance**: Rust's zero-cost abstractions provide maximum performance
- **Memory Safety**: Rust's ownership model prevents memory-related bugs
- **Concurrency**: Safe and efficient concurrent operations
- **Cross-Platform**: Easily compile for different operating systems
- **Small Binary Size**: Minimal dependencies and efficient code result in small binaries

## License

MIT

## Acknowledgements

- [OpenRouter](https://openrouter.ai/) for providing access to AI models
- [Cline](https://github.com/cline/cline) for inspiration
