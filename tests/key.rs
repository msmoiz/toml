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
fn dotted_key() -> toml::Result<()> {
    let text = r#"name = "Orange""#;
    let root = from_str(text)?;
    assert_eq!(root["name"].as_str(), "Orange");
    Ok(())
}

#[test]
fn dotted_key_2() -> toml::Result<()> {
    let text = r#"physical.color = "orange""#;
    let root = from_str(text)?;
    assert_eq!(root["physical"]["color"].as_str(), "orange");
    Ok(())
}

#[test]
fn dotted_key_3() -> toml::Result<()> {
    let text = r#"physical.shape = "round""#;
    let root = from_str(text)?;
    assert_eq!(root["physical"]["shape"].as_str(), "round");
    Ok(())
}

#[test]
fn dotted_key_4() -> toml::Result<()> {
    let text = r#"site."google.com" = true"#;
    let root = from_str(text)?;
    assert_eq!(root["site"]["google.com"].as_bool(), true);
    Ok(())
}

#[test]
fn dotted_whitespace() -> toml::Result<()> {
    let text = r#"fruit.name = "banana"     # this is best practice"#;
    let root = from_str(text)?;
    assert_eq!(root["fruit"]["name"].as_str(), "banana");
    Ok(())
}

#[test]
fn dotted_whitespace_2() -> toml::Result<()> {
    let text = r#"fruit. color = "yellow"    # same as fruit.color"#;
    let root = from_str(text)?;
    assert_eq!(root["fruit"]["color"].as_str(), "yellow");
    Ok(())
}

#[test]
fn dotted_whitespace_3() -> toml::Result<()> {
    let text = r#"fruit . flavor = "banana"   # same as fruit.flavor"#;
    let root = from_str(text)?;
    assert_eq!(root["fruit"]["flavor"].as_str(), "banana");
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

#[test]
fn redefined_key_3() -> toml::Result<()> {
    let text = r#"
# This makes the key "fruit" into a table.
fruit.apple.smooth = true

# So then you can add to the table "fruit" like so:
fruit.orange = 2
"#;
    let root = from_str(text)?;
    assert_eq!(root["fruit"]["apple"]["smooth"].as_bool(), true);
    assert_eq!(root["fruit"]["orange"].as_int(), 2);
    Ok(())
}

#[test]
fn redefined_key_4() -> toml::Result<()> {
    let text = r#"
# THE FOLLOWING IS INVALID

# This defines the value of fruit.apple to be an integer.
fruit.apple = 1

# But then this treats fruit.apple like it's a table.
# You can't turn an integer into a table.
fruit.apple.smooth = true
"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn key_order() -> toml::Result<()> {
    let text = r#"
# VALID BUT DISCOURAGED

apple.type = "fruit"
orange.type = "fruit"

apple.skin = "thin"
orange.skin = "thick"

apple.color = "red"
orange.color = "orange"
"#;
    let root = from_str(text)?;
    assert_eq!(root["apple"]["type"].as_str(), "fruit");
    assert_eq!(root["apple"]["skin"].as_str(), "thin");
    assert_eq!(root["apple"]["color"].as_str(), "red");
    assert_eq!(root["orange"]["type"].as_str(), "fruit");
    assert_eq!(root["orange"]["skin"].as_str(), "thick");
    assert_eq!(root["orange"]["color"].as_str(), "orange");
    Ok(())
}

#[test]
fn key_order_2() -> toml::Result<()> {
    let text = r#"
# RECOMMENDED

apple.type = "fruit"
apple.skin = "thin"
apple.color = "red"

orange.type = "fruit"
orange.skin = "thick"
orange.color = "orange"
"#;
    let root = from_str(text)?;
    assert_eq!(root["apple"]["type"].as_str(), "fruit");
    assert_eq!(root["apple"]["skin"].as_str(), "thin");
    assert_eq!(root["apple"]["color"].as_str(), "red");
    assert_eq!(root["orange"]["type"].as_str(), "fruit");
    assert_eq!(root["orange"]["skin"].as_str(), "thick");
    assert_eq!(root["orange"]["color"].as_str(), "orange");
    Ok(())
}

#[test]
fn misleading_float_dotted_key() -> toml::Result<()> {
    let text = r#"3.14159 = "pi""#;
    let root = from_str(text)?;
    assert_eq!(root["3"]["14159"].as_str(), "pi");
    Ok(())
}
