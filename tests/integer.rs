use toml::{from_str, Result};

#[test]
fn basic() -> Result<()> {
    let text = "
int1 = +99
int2 = 42
int3 = 0
int4 = -17
";
    let root = from_str(text)?;
    assert_eq!(root["int1"].as_int(), 99);
    assert_eq!(root["int2"].as_int(), 42);
    assert_eq!(root["int3"].as_int(), 0);
    assert_eq!(root["int4"].as_int(), -17);
    Ok(())
}

#[test]
fn underscores() -> Result<()> {
    let text = "
int5 = 1_000
int6 = 5_349_221
int7 = 53_49_221  # Indian number system grouping
int8 = 1_2_3_4_5  # VALID but discouraged
";
    let root = from_str(text)?;
    assert_eq!(root["int5"].as_int(), 1000);
    assert_eq!(root["int6"].as_int(), 5349221);
    assert_eq!(root["int7"].as_int(), 5349221);
    assert_eq!(root["int8"].as_int(), 12345);
    Ok(())
}

#[test]
fn hex() -> Result<()> {
    let text = "
hex1 = 0xDEADBEEF
hex2 = 0xdeadbeef
hex3 = 0xdead_beef
";
    let root = from_str(text)?;
    assert_eq!(root["hex1"].as_int(), 3735928559);
    assert_eq!(root["hex2"].as_int(), 3735928559);
    assert_eq!(root["hex3"].as_int(), 3735928559);
    Ok(())
}

#[test]
fn octal() -> Result<()> {
    let text = "
oct1 = 0o01234567
oct2 = 0o755 # useful for Unix file permissions
";
    let root = from_str(text)?;
    assert_eq!(root["oct1"].as_int(), 342391);
    assert_eq!(root["oct2"].as_int(), 493);
    Ok(())
}

#[test]
fn binary() -> Result<()> {
    let text = "
bin1 = 0b11010110
";
    let root = from_str(text)?;
    assert_eq!(root["bin1"].as_int(), 214);
    Ok(())
}
