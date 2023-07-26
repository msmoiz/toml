use toml::from_str;

#[test]
fn comment() -> toml::Result<()> {
    let text = r###"# This is a full-line comment
key = "value"  # This is a comment at the end of a line
another = "# This is not a comment""###;
    let root = from_str(text)?;
    assert_eq!(root["key"].as_str(), "value");
    Ok(())
}
