use chrono::NaiveDate;
use toml::{from_str, Result};

#[test]
fn table() -> Result<()> {
    let text = "[table]";
    let root = from_str(text)?;
    assert!(root["table"].as_table().is_empty());
    Ok(())
}

#[test]
fn table_consec() -> Result<()> {
    let text = r#"
[table-1]
key1 = "some string"
key2 = 123

[table-2]
key1 = "another string"
key2 = 456
"#;
    let root = from_str(text)?;
    assert_eq!(root["table-1"]["key1"].as_str(), "some string");
    assert_eq!(root["table-1"]["key2"].as_int(), 123);
    assert_eq!(root["table-2"]["key1"].as_str(), "another string");
    assert_eq!(root["table-2"]["key2"].as_int(), 456);
    Ok(())
}

#[test]
fn dotted_table_name() -> Result<()> {
    let text = r#"
[dog."tater.man"]
type.name = "pug"
"#;
    let root = from_str(text)?;
    assert_eq!(root["dog"]["tater.man"]["type"]["name"].as_str(), "pug");
    Ok(())
}

#[test]
fn whitespace_around_key() -> Result<()> {
    let text = r#"
[a.b.c]            # this is best practice
[ d.e.f ]          # same as [d.e.f]
[ g .  h  . i ]    # same as [g.h.i]
[ j . "ʞ" . 'l' ]  # same as [j."ʞ".'l']
"#;
    let root = from_str(text)?;
    assert!(root["a"]["b"]["c"].as_table().is_empty());
    assert!(root["d"]["e"]["f"].as_table().is_empty());
    assert!(root["g"]["h"]["i"].as_table().is_empty());
    assert!(root["j"]["ʞ"]["l"].as_table().is_empty());
    Ok(())
}

#[test]
fn super_table() -> Result<()> {
    let text = r#"
# [x] you
# [x.y] don't
# [x.y.z] need these
[x.y.z.w] # for this to work

[x] # defining a super-table afterward is ok
"#;
    let root = from_str(text)?;
    assert!(root["x"]["y"]["z"]["w"].as_table().is_empty());
    Ok(())
}

#[test]
fn redefine() -> Result<()> {
    let text = r#"
# DO NOT DO THIS

[fruit]
apple = "red"

[fruit]
orange = "orange"
"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn redefine_2() -> Result<()> {
    let text = r#"
# DO NOT DO THIS EITHER

[fruit]
apple = "red"

[fruit.apple]
texture = "smooth"
"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn out_of_order() -> Result<()> {
    let text = r#"
# VALID BUT DISCOURAGED
[fruit.apple]
[animal]
[fruit.orange]
"#;
    let root = from_str(text)?;
    assert!(root["fruit"]["apple"].as_table().is_empty());
    assert!(root["fruit"]["orange"].as_table().is_empty());
    assert!(root["animal"].as_table().is_empty());
    Ok(())
}

#[test]
fn out_of_order_2() -> Result<()> {
    let text = r#"
# RECOMMENDED
[fruit.apple]
[fruit.orange]
[animal]
"#;
    let root = from_str(text)?;
    assert!(root["fruit"]["apple"].as_table().is_empty());
    assert!(root["fruit"]["orange"].as_table().is_empty());
    assert!(root["animal"].as_table().is_empty());
    Ok(())
}

#[test]
fn top_level_table() -> Result<()> {
    let text = r#"
# Top-level table begins.
name = "Fido"
breed = "pug"

# Top-level table ends.
[owner]
name = "Regina Dogman"
member_since = 1999-08-04
"#;
    let root = from_str(text)?;
    assert_eq!(root["name"].as_str(), "Fido");
    assert_eq!(root["breed"].as_str(), "pug");
    assert_eq!(root["owner"]["name"].as_str(), "Regina Dogman");
    assert_eq!(
        root["owner"]["member_since"].as_local_date(),
        NaiveDate::from_ymd_opt(1999, 08, 04).unwrap()
    );
    Ok(())
}

#[test]
fn dotted_key() -> Result<()> {
    let text = r#"
fruit.apple.color = "red"
# Defines a table named fruit
# Defines a table named fruit.apple

fruit.apple.taste.sweet = true
# Defines a table named fruit.apple.taste
# fruit and fruit.apple were already created
"#;
    let root = from_str(text)?;
    assert_eq!(root["fruit"]["apple"]["color"].as_str(), "red");
    assert_eq!(root["fruit"]["apple"]["taste"]["sweet"].as_bool(), true);
    Ok(())
}

#[test]
fn redefine_3() -> Result<()> {
    let text = r#"
[fruit]
apple.color = "red"
apple.taste.sweet = true
[fruit.apple.texture]  # you can add sub-tables
smooth = true
"#;
    let root = from_str(text)?;
    assert_eq!(root["fruit"]["apple"]["color"].as_str(), "red");
    assert_eq!(root["fruit"]["apple"]["taste"]["sweet"].as_bool(), true);
    assert_eq!(root["fruit"]["apple"]["texture"]["smooth"].as_bool(), true);
    Ok(())
}

#[test]
fn redefine_4() -> Result<()> {
    let text = r#"
[fruit]
apple.color = "red"
apple.taste.sweet = true
[fruit.apple]  # INVALID
[fruit.apple.taste]  # INVALID
"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn redefine_5() -> Result<()> {
    let text = r#"
[fruit]
[fruit]
"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}
