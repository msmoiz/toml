use toml::{from_str, Result};

#[test]
fn inline_table() -> Result<()> {
    let text = r#"
name = { first = "Tom", last = "Preston-Werner" }
point = { x = 1, y = 2 }
animal = { type.name = "pug" }
"#;
    let root = from_str(text)?;
    assert_eq!(root["name"]["first"].as_str(), "Tom");
    assert_eq!(root["name"]["last"].as_str(), "Preston-Werner");
    assert_eq!(root["point"]["x"].as_int(), 1);
    assert_eq!(root["point"]["y"].as_int(), 2);
    assert_eq!(root["animal"]["type"]["name"].as_str(), "pug");
    Ok(())
}

#[test]
fn inline_table_2() -> Result<()> {
    let text = r#"
[product]
type = { name = "Nail" }
type.edible = false  # INVALID
"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn inline_table_3() -> Result<()> {
    let text = r#"
[product]
type.name = "Nail"
type = { edible = false }  # INVALID
"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}
