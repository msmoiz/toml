use toml::{from_str, Result};

#[test]
fn basic_string() -> Result<()> {
    let text = r#"str = "I'm a string. \"You can quote me\". Name\tJose\nLocation\tSF.""#;
    let root = from_str(text)?;
    assert_eq!(
        root["str"].as_str(),
        "I'm a string. \"You can quote me\". Name\tJose\nLocation\tSF."
    );
    Ok(())
}

#[test]
fn basic_multiline_string() -> Result<()> {
    let text = r#"
str1 = """
Roses are red
Violets are blue"""
"#;
    let root = from_str(text)?;
    assert_eq!(root["str1"].as_str(), "Roses are red\nViolets are blue");
    Ok(())
}

#[test]
fn basic_multiline_string_2() -> Result<()> {
    let text = r#"str2 = "Roses are red\nViolets are blue""#;
    let root = from_str(text)?;
    assert_eq!(root["str2"].as_str(), "Roses are red\nViolets are blue");
    Ok(())
}

#[test]
#[ignore = "not adding windows support"]
fn basic_multiline_string_3() -> Result<()> {
    let text = r#"str3 = "Roses are red\nViolets are blue""#;
    let root = from_str(text)?;
    assert_eq!(root["str3"].as_str(), "Roses are red\r\nViolets are blue");
    Ok(())
}

#[test]
fn line_ending_slash() -> Result<()> {
    let text = r#"str1 = "The quick brown fox jumps over the lazy dog.""#;
    let root = from_str(text)?;
    assert_eq!(
        root["str1"].as_str(),
        "The quick brown fox jumps over the lazy dog."
    );
    Ok(())
}

#[test]
fn line_ending_slash_2() -> Result<()> {
    let text = r#"str2 = """
The quick brown \


  fox jumps over \
    the lazy dog."""
"#;
    let root = from_str(text)?;
    assert_eq!(
        root["str2"].as_str(),
        "The quick brown fox jumps over the lazy dog."
    );
    Ok(())
}

#[test]
fn line_ending_slash_3() -> Result<()> {
    let text = r#"
str3 = """\
    The quick brown \
    fox jumps over \
    the lazy dog.\
    """
"#;
    let root = from_str(text)?;
    assert_eq!(
        root["str3"].as_str(),
        "The quick brown fox jumps over the lazy dog."
    );
    Ok(())
}

#[test]
fn internal_quote() -> Result<()> {
    let text = r#"str4 = """Here are two quotation marks: "". Simple enough.""""#;
    let root = from_str(text)?;
    assert_eq!(
        root["str4"].as_str(),
        "Here are two quotation marks: \"\". Simple enough."
    );
    Ok(())
}

#[test]
fn internal_quote_2() -> Result<()> {
    let text = r#"str5 = """Here are three quotation marks: """."""  # INVALID"#;
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn internal_quote_3() -> Result<()> {
    let text = r#"str5 = """Here are three quotation marks: ""\".""""#;
    let root = from_str(text)?;
    assert_eq!(
        root["str5"].as_str(),
        r#"Here are three quotation marks: """."#
    );
    Ok(())
}

#[test]
fn internal_quote_4() -> Result<()> {
    let text = r#"str6 = """Here are fifteen quotation marks: ""\"""\"""\"""\"""\".""""#;
    let root = from_str(text)?;
    assert_eq!(
        root["str6"].as_str(),
        r#"Here are fifteen quotation marks: """""""""""""""."#
    );
    Ok(())
}

#[test]
fn internal_quote_5() -> Result<()> {
    let text = r#"
# "This," she said, "is just a pointless statement."
str7 = """"This," she said, "is just a pointless statement.""""
"#;
    let root = from_str(text)?;
    assert_eq!(
        root["str7"].as_str(),
        r#""This," she said, "is just a pointless statement.""#
    );
    Ok(())
}

#[test]
fn literal() -> Result<()> {
    let text = r#"
# What you see is what you get.
winpath  = 'C:\Users\nodejs\templates'
winpath2 = '\\ServerX\admin$\system32\'
quoted   = 'Tom "Dubs" Preston-Werner'
regex    = '<\i\c*\s*>'
"#;
    let root = from_str(text)?;
    assert_eq!(root["winpath"].as_str(), r#"C:\Users\nodejs\templates"#);
    assert_eq!(root["winpath2"].as_str(), r#"\\ServerX\admin$\system32\"#);
    assert_eq!(root["quoted"].as_str(), r#"Tom "Dubs" Preston-Werner"#);
    assert_eq!(root["regex"].as_str(), r#"<\i\c*\s*>"#);
    Ok(())
}

#[test]
fn literal_multiline() -> Result<()> {
    let text = r#"
regex2 = '''I [dw]on't need \d{2} apples'''
lines  = '''
The first newline is
trimmed in raw strings.
   All other whitespace
   is preserved.
'''
"#;
    let root = from_str(text)?;
    assert_eq!(root["regex2"].as_str(), r#"I [dw]on't need \d{2} apples"#);
    assert_eq!(
        root["lines"].as_str(),
        r#"The first newline is
trimmed in raw strings.
   All other whitespace
   is preserved.
"#
    );
    Ok(())
}

#[test]
fn literal_multiline_2() -> Result<()> {
    let text = r#"quot15 = '''Here are fifteen quotation marks: """""""""""""""'''"#;
    let root = from_str(text)?;
    assert_eq!(
        root["quot15"].as_str(),
        r#"Here are fifteen quotation marks: """"""""""""""""#
    );
    Ok(())
}

#[test]
fn literal_multiline_3() -> Result<()> {
    let text = r#"apos15 = "Here are fifteen apostrophes: '''''''''''''''""#;
    let root = from_str(text)?;
    assert_eq!(
        root["apos15"].as_str(),
        r#"Here are fifteen apostrophes: '''''''''''''''"#
    );
    Ok(())
}

#[test]
fn literal_multiline_4() -> Result<()> {
    let text = r#"str = ''''That,' she said, 'is still pointless.''''"#;
    let root = from_str(text)?;
    assert_eq!(
        root["str"].as_str(),
        r#"'That,' she said, 'is still pointless.'"#
    );
    Ok(())
}
