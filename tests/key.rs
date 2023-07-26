use toml::from_str;

#[test]
fn bare_key_1() -> toml::Result<()> {
    let text = r#"key = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["key"].as_str(), "value");
    Ok(())
}

#[test]
fn bare_key_2() -> toml::Result<()> {
    let text = r#"bare_key = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["bare_key"].as_str(), "value");
    Ok(())
}

#[test]
fn bare_key_3() -> toml::Result<()> {
    let text = r#"bare-key = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["bare-key"].as_str(), "value");
    Ok(())
}

#[test]
fn bare_key_4() -> toml::Result<()> {
    let text = r#"1234 = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["1234"].as_str(), "value");
    Ok(())
}

#[test]
fn quoted_key_1() -> toml::Result<()> {
    let text = r#""127.0.0.1" = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["127.0.0.1"].as_str(), "value");
    Ok(())
}

#[test]
fn quoted_key_2() -> toml::Result<()> {
    let text = r#""character encoding" = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["character encoding"].as_str(), "value");
    Ok(())
}

#[test]
fn quoted_key_3() -> toml::Result<()> {
    let text = r#""ʎǝʞ" = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["ʎǝʞ"].as_str(), "value");
    Ok(())
}

#[test]
fn quoted_key_4() -> toml::Result<()> {
    let text = r#"'key2' = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["key2"].as_str(), "value");
    Ok(())
}

#[test]
fn quoted_key_5() -> toml::Result<()> {
    let text = r#"'quoted "value"' = "value""#;
    let root = from_str(text)?;
    assert_eq!(root["quoted \"value\""].as_str(), "value");
    Ok(())
}

#[test]
fn empty_key_1() -> toml::Result<()> {
    let text = r#"= "no key name"  # INVALID"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn empty_key_2() -> toml::Result<()> {
    let text = r#""" = "blank"     # VALID but discouraged"#;
    let root = from_str(text)?;
    assert_eq!(root[""].as_str(), "blank");
    Ok(())
}

#[test]
fn empty_key_3() -> toml::Result<()> {
    let text = r#"'' = 'blank'     # VALID but discouraged"#;
    let root = from_str(text)?;
    assert_eq!(root[""].as_str(), "blank");
    Ok(())
}

#[test]
fn redefined_key() -> toml::Result<()> {
    let text = r#"
# DO NOT DO THIS
name = "Tom"
name = "Pradyun"
"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn redefined_key_2() -> toml::Result<()> {
    let text = r#"
# THIS WILL NOT WORK
spelling = "favorite"
"spelling" = "favourite"
"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}
