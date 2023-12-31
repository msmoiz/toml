use toml::{from_str, Result};

#[test]
fn array_of_tables() -> Result<()> {
    let text = r#"
[[products]]
name = "Hammer"
sku = 738594937

[[products]]  # empty table within the array

[[products]]
name = "Nail"
sku = 284758393

color = "gray"
"#;
    let root = from_str(text)?;

    assert_eq!(root["products"][0]["name"].as_str(), "Hammer");
    assert_eq!(root["products"][0]["sku"].as_int(), 738594937);

    assert!(root["products"][1].as_table().is_empty());

    assert_eq!(root["products"][2]["name"].as_str(), "Nail");
    assert_eq!(root["products"][2]["sku"].as_int(), 284758393);
    assert_eq!(root["products"][2]["color"].as_str(), "gray");

    Ok(())
}

#[test]
fn array_of_tables_subtables() -> Result<()> {
    let text = r#"
[[fruits]]
name = "apple"

[fruits.physical]  # subtable
color = "red"
shape = "round"

[[fruits.varieties]]  # nested array of tables
name = "red delicious"

[[fruits.varieties]]
name = "granny smith"

[[fruits]]
name = "banana"

[[fruits.varieties]]
name = "plantain"
"#;
    let root = from_str(text)?;

    assert_eq!(root["fruits"][0]["name"].as_str(), "apple");
    assert_eq!(root["fruits"][0]["physical"]["color"].as_str(), "red");
    assert_eq!(root["fruits"][0]["physical"]["shape"].as_str(), "round");
    assert_eq!(
        root["fruits"][0]["varieties"][0]["name"].as_str(),
        "red delicious"
    );
    assert_eq!(
        root["fruits"][0]["varieties"][1]["name"].as_str(),
        "granny smith"
    );

    assert_eq!(root["fruits"][1]["name"].as_str(), "banana");
    assert_eq!(
        root["fruits"][1]["varieties"][0]["name"].as_str(),
        "plantain"
    );

    Ok(())
}

#[test]
fn redefine() -> Result<()> {
    let text = r#"
# INVALID TOML DOC
[fruit.physical]  # subtable, but to which parent element should it belong?
color = "red"
shape = "round"

[[fruit]]  # parser must throw an error upon discovering that "fruit" is
           # an array rather than a table
name = "apple"
"#;
    let root = from_str(text);

    assert!(root.is_err());

    Ok(())
}

#[test]
fn redefine_2() -> Result<()> {
    let text = r#"
# INVALID TOML DOC
fruits = []

[[fruits]] # Not allowed
"#;
    let root = from_str(text);

    assert!(root.is_err());

    Ok(())
}

#[test]
fn redefine_3() -> Result<()> {
    let text = r#"
# INVALID TOML DOC
[[fruits]]
name = "apple"

[[fruits.varieties]]
name = "red delicious"

# INVALID: This table conflicts with the previous array of tables
[fruits.varieties]
name = "granny smith"
"#;
    let root = from_str(text);

    assert!(root.is_err());

    Ok(())
}

#[test]
fn redefine_4() -> Result<()> {
    let text = r#"
# INVALID TOML DOC
[fruits.physical]
color = "red"
shape = "round"

# INVALID: This array of tables conflicts with the previous table
[[fruits.physical]]
color = "green"
"#;
    let root = from_str(text);

    assert!(root.is_err());

    Ok(())
}

#[test]
fn inline_tables() -> Result<()> {
    let text = r#"
points = [ { x = 1, y = 2, z = 3 },
           { x = 7, y = 8, z = 9 },
           { x = 2, y = 4, z = 8 } ]
"#;
    let root = from_str(text)?;

    assert_eq!(root["points"][0]["x"].as_int(), 1);
    assert_eq!(root["points"][0]["y"].as_int(), 2);
    assert_eq!(root["points"][0]["z"].as_int(), 3);

    assert_eq!(root["points"][1]["x"].as_int(), 7);
    assert_eq!(root["points"][1]["y"].as_int(), 8);
    assert_eq!(root["points"][1]["z"].as_int(), 9);

    assert_eq!(root["points"][2]["x"].as_int(), 2);
    assert_eq!(root["points"][2]["y"].as_int(), 4);
    assert_eq!(root["points"][2]["z"].as_int(), 8);

    Ok(())
}

#[test]
fn inline_tables_2() -> Result<()> {
    let text = r#"
[[a]]

[a.b.c]

[[a]]

[a.b.c]
"#;
    from_str(text)?;

    Ok(())
}
