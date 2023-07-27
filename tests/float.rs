use toml::{from_str, Result};

#[test]
fn basic_float() -> Result<()> {
    let text = "
# fractional
flt1 = +1.0
flt2 = 3.1415
flt3 = -0.01

# exponent
flt4 = 5e+22
flt5 = 1e06
flt6 = -2E-2

# both
flt7 = 6.626e-34
";
    let root = from_str(text)?;
    assert_eq!(root["flt1"].as_float(), 1.0);
    assert_eq!(root["flt2"].as_float(), 3.1415);
    assert_eq!(root["flt3"].as_float(), -0.01);
    assert_eq!(root["flt4"].as_float(), 5e+22);
    assert_eq!(root["flt5"].as_float(), 1e06);
    assert_eq!(root["flt6"].as_float(), -2E-2);
    assert_eq!(root["flt7"].as_float(), 6.626e-34);
    Ok(())
}

#[test]
fn invalid() -> Result<()> {
    let text = "invalid_float_1 = .7";
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn invalid_2() -> Result<()> {
    let text = "invalid_float_2 = 7.";
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn invalid_3() -> Result<()> {
    let text = "invalid_float_3 = 3.e+20";
    let root = from_str(text);
    assert!(root.is_err());
    Ok(())
}

#[test]
fn underscore() -> Result<()> {
    let text = "flt8 = 224_617.445_991_228";
    let root = from_str(text)?;
    assert_eq!(root["flt8"].as_float(), 224617.445991228);
    Ok(())
}

#[test]
fn infinity() -> Result<()> {
    let text = "
sf1 = inf  # positive infinity
sf2 = +inf  # positive infinity
sf3 = -inf  # positive infinity
";
    let root = from_str(text)?;
    assert_eq!(root["sf1"].as_float(), f64::INFINITY);
    assert_eq!(root["sf2"].as_float(), f64::INFINITY);
    assert_eq!(root["sf3"].as_float(), f64::NEG_INFINITY);
    Ok(())
}

#[test]
#[ignore = "parsed nan does not match constant nan"]
fn nan() -> Result<()> {
    let text = "
sf4 = nan  # actual sNaN/qNaN encoding is implementation-specific
sf5 = +nan # same as `nan`
sf6 = -nan # valid, actual encoding is implementation-specific
";
    let root = from_str(text)?;
    assert_eq!(root["sf4"].as_float(), f64::NAN);
    assert_eq!(root["sf5"].as_float(), f64::NAN);
    assert_eq!(root["sf6"].as_float(), f64::NAN);
    Ok(())
}
