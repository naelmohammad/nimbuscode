#[cfg(test)]
mod tests {
    use nimbuscode::{Config, extract_code_blocks};
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_config() -> PathBuf {
        let test_dir = PathBuf::from("./test_config");
        if !test_dir.exists() {
            fs::create_dir_all(&test_dir).unwrap();
        }
        test_dir
    }

    fn cleanup_test_config(test_dir: &PathBuf) {
        if test_dir.exists() {
            fs::remove_dir_all(test_dir).unwrap();
        }
    }

    #[test]
    fn test_extract_code_blocks() {
        let markdown_text = r#"
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
        "#;

        let code_blocks = extract_code_blocks(markdown_text);
        assert_eq!(code_blocks.len(), 2);
        assert_eq!(code_blocks[0], "def hello():\n    print(\"Hello, world!\")");
        assert_eq!(
            code_blocks[1],
            "function hello() {\n    console.log(\"Hello, world!\");\n}"
        );
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.api_key, "");
        assert_eq!(config.model, "openrouter/auto");
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_extract_code_blocks_with_language() {
        let markdown_text = r#"
        ```rust
        fn main() {
            println!("Hello, world!");
        }
        ```
        "#;

        let code_blocks = extract_code_blocks(markdown_text);
        assert_eq!(code_blocks.len(), 1);
        assert_eq!(code_blocks[0], "fn main() {\n    println!(\"Hello, world!\");\n}");
    }

    #[test]
    fn test_extract_code_blocks_empty() {
        let markdown_text = "No code blocks here.";
        let code_blocks = extract_code_blocks(markdown_text);
        assert_eq!(code_blocks.len(), 0);
    }
}
