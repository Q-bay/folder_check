use std::path::Path;
use super::*;

#[test]
fn test_should_ignore() {
    let ignore_patterns = IgnorePatterns {
        paths: vec![Regex::new(r"(^|/)target/").unwrap()],
        extensions: vec![".txt".to_string(), ".log".to_string()],
    };

    assert!(should_ignore(Path::new("target/debug/build"), &ignore_patterns));
    assert!(should_ignore(Path::new("src/file.txt"), &ignore_patterns));
    assert!(!should_ignore(Path::new("src/main.rs"), &ignore_patterns));
}

#[test]
fn test_load_ignore_patterns() {
    let result = load_ignore_patterns(".foldercheckignore");
    assert!(result.is_ok());
    let patterns = result.unwrap();
    assert!(!patterns.paths.is_empty());
    assert!(!patterns.extensions.is_empty());
}

#[test]
fn test_arg_check() {
    let valid_args = vec![
        "program".to_string(),
        "command".to_string(),
        "/path/to/folder".to_string(),
        "10".to_string(),
    ];
    assert!(std::panic::catch_unwind(|| arg_check(&valid_args)).is_ok());

    let invalid_args = vec!["program".to_string()];
    assert!(std::panic::catch_unwind(|| arg_check(&invalid_args)).is_err());
}