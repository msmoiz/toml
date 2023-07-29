use toml::{from_str, Result};

#[test]
fn array() -> Result<()> {
    let text = r#"
integers = [ 1, 2, 3 ]
colors = [ "red", "yellow", "green" ]
nested_arrays_of_ints = [ [ 1, 2 ], [3, 4, 5] ]
nested_mixed_array = [ [ 1, 2 ], ["a", "b", "c"] ]
string_array = [ "all", 'strings', """are the same""", '''type''' ]

# Mixed-type arrays are allowed
numbers = [ 0.1, 0.2, 0.5, 1, 2, 5 ]
contributors = [
   "Foo Bar <foo@example.com>",
   { name = "Baz Qux", email = "bazqux@example.com", url = "https://example.com/bazqux" }
]
"#;
    let root = from_str(text)?;
    assert_eq!(root["integers"][0].as_int(), 1);
    assert_eq!(root["integers"][1].as_int(), 2);
    assert_eq!(root["integers"][2].as_int(), 3);

    assert_eq!(root["colors"][0].as_str(), "red");
    assert_eq!(root["colors"][1].as_str(), "yellow");
    assert_eq!(root["colors"][2].as_str(), "green");

    assert_eq!(root["nested_arrays_of_ints"][0][0].as_int(), 1);
    assert_eq!(root["nested_arrays_of_ints"][0][1].as_int(), 2);
    assert_eq!(root["nested_arrays_of_ints"][1][0].as_int(), 3);
    assert_eq!(root["nested_arrays_of_ints"][1][1].as_int(), 4);
    assert_eq!(root["nested_arrays_of_ints"][1][2].as_int(), 5);

    assert_eq!(root["nested_mixed_array"][0][0].as_int(), 1);
    assert_eq!(root["nested_mixed_array"][0][1].as_int(), 2);
    assert_eq!(root["nested_mixed_array"][1][0].as_str(), "a");
    assert_eq!(root["nested_mixed_array"][1][1].as_str(), "b");
    assert_eq!(root["nested_mixed_array"][1][2].as_str(), "c");

    assert_eq!(root["string_array"][0].as_str(), "all");
    assert_eq!(root["string_array"][1].as_str(), "strings");
    assert_eq!(root["string_array"][2].as_str(), "are the same");
    assert_eq!(root["string_array"][3].as_str(), "type");

    assert_eq!(root["numbers"][0].as_float(), 0.1);
    assert_eq!(root["numbers"][1].as_float(), 0.2);
    assert_eq!(root["numbers"][2].as_float(), 0.5);
    assert_eq!(root["numbers"][3].as_int(), 1);
    assert_eq!(root["numbers"][4].as_int(), 2);
    assert_eq!(root["numbers"][5].as_int(), 5);

    assert_eq!(
        root["contributors"][0].as_str(),
        "Foo Bar <foo@example.com>"
    );

    assert_eq!(root["contributors"][1]["name"].as_str(), "Baz Qux");
    assert_eq!(
        root["contributors"][1]["email"].as_str(),
        "bazqux@example.com"
    );
    assert_eq!(
        root["contributors"][1]["url"].as_str(),
        "https://example.com/bazqux"
    );

    Ok(())
}

#[test]
fn multiline_array() -> Result<()> {
    let text = r#"
integers2 = [
  1, 2, 3
]

integers3 = [
  1,
  2, # this is ok
]
"#;
    let root = from_str(text)?;

    assert_eq!(root["integers2"][0].as_int(), 1);
    assert_eq!(root["integers2"][1].as_int(), 2);
    assert_eq!(root["integers2"][2].as_int(), 3);

    assert_eq!(root["integers3"][0].as_int(), 1);
    assert_eq!(root["integers3"][1].as_int(), 2);

    Ok(())
}
