# NimbusCode (Rust Implementation)

A lightweight, portable AI coding assistant powered by OpenRouter's free models, implemented in Rust for maximum performance and minimal resource usage.

## Features

- **Minimal Footprint**: Uses minimal disk space and memory
- **High Performance**: Rust implementation for speed and efficiency
- **Portable**: Works across platforms with minimal dependencies
- **Code Generation**: Create code based on natural language descriptions
- **Code Improvement**: Get suggestions to improve existing code
- **Code Explanation**: Understand complex code with detailed explanations
- **Cloud Deployment**: Generate deployment instructions for AWS, Azure, and GCP
- **Mobile Development**: Get guidance for iOS, Android, and cross-platform development
- **Interactive Mode**: Have a conversation with the AI about your coding questions

## Building

```bash
# Navigate to the Rust implementation directory
cd rust

# Build in release mode for optimal performance
cargo build --release

# The binary will be available at target/release/nimbuscode
```

## Installation

```bash
# After building, you can copy the binary to a location in your PATH
cp target/release/nimbuscode /usr/local/bin/
```

## Configuration

Before using NimbusCode, you need to set up your OpenRouter API key:

```bash
# Set your API key
nimbuscode config --api-key YOUR_API_KEY

# Alternatively, set it as an environment variable
export OPENROUTER_API_KEY=your_api_key
```

## Usage

### Ask a coding question

```bash
nimbuscode ask "How do I implement a binary search tree in Rust?"
```

### Generate code

```bash
nimbuscode generate "A REST API for a todo list application" --language rust --save todo_api.rs
```

### Improve existing code

```bash
nimbuscode improve my_script.rs --save improved_script.rs
```

### Explain code

```bash
nimbuscode explain complex_algorithm.rs
```

### Cloud deployment guidance

```bash
nimbuscode cloud "Deploy a containerized Rust application with a PostgreSQL database" --provider aws
```

### Mobile development guidance

```bash
nimbuscode mobile "A fitness tracking app with social features" --platform cross
```

### Interactive mode

```bash
nimbuscode interactive
```

### List available free models

```bash
nimbuscode models
```

## Why the Rust Implementation?

- **Performance**: Significantly faster execution and lower memory usage
- **Binary Size**: Smaller executable with fewer dependencies
- **Resource Efficiency**: Lower CPU and memory footprint
- **Cross-Platform**: Easy to compile for different operating systems
- **Security**: Rust's memory safety guarantees prevent common vulnerabilities
