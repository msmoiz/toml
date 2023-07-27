use toml::{from_str, Result};

#[test]
fn bool() -> Result<()> {
    let text = "
bool1 = true
bool2 = false
";
    let root = from_str(text)?;
    assert_eq!(root["bool1"].as_bool(), true);
    assert_eq!(root["bool2"].as_bool(), false);
    Ok(())
}
