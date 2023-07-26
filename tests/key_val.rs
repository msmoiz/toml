use toml::from_str;

#[test]
fn basic() -> toml::Result<()> {
    let text = r#"key = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["key"].as_str(), "value");
    Ok(())
}

#[test]
fn unspecified_val() -> toml::Result<()> {
    let text = r#"key = # INVALID"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn missing_trailing_newline() -> toml::Result<()> {
    let text = r#"first = "Tom" last = "Preston-Werner" # INVALID"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}
