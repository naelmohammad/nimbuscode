# Updates

## Version 0.1.0 (Initial Release) - May 30, 2025

### Added
- Initial release of NimbusCode with both Python and Rust implementations
- Core functionality:
  - Ask coding questions
  - Generate code from descriptions
  - Improve existing code
  - Explain complex code
  - Get cloud deployment guidance
  - Get mobile development guidance
  - Interactive mode for conversations
  - List available free models from OpenRouter
- Configuration management for API keys
- Support for all free models from OpenRouter
- Comprehensive documentation and examples

### Technical Details
- Python implementation uses minimal dependencies (only requests)
- Rust implementation uses tokio for async runtime
- Both versions share the same command-line interface
- Configuration stored in user's config directory
- Support for environment variables for API keys
