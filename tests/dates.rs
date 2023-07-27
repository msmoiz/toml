use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone, Timelike};
use toml::{from_str, Result};

#[test]
fn offset_date_time() -> Result<()> {
    let text = "
odt1 = 1979-05-27T07:32:00Z
odt2 = 1979-05-27T00:32:00-07:00
odt3 = 1979-05-27T00:32:00.999999-07:00
odt4 = 1979-05-27 07:32:00Z
";
    let root = from_str(text)?;

    assert_eq!(
        root["odt1"].as_offset_date_time(),
        FixedOffset::west_opt(0)
            .unwrap()
            .with_ymd_and_hms(1979, 05, 27, 07, 32, 00)
            .unwrap()
    );

    assert_eq!(
        root["odt2"].as_offset_date_time(),
        FixedOffset::west_opt(7 * 60 * 60)
            .unwrap()
            .with_ymd_and_hms(1979, 05, 27, 00, 32, 00)
            .unwrap()
    );

    assert_eq!(
        root["odt3"].as_offset_date_time(),
        FixedOffset::west_opt(7 * 60 * 60)
            .unwrap()
            .with_ymd_and_hms(1979, 05, 27, 00, 32, 00)
            .unwrap()
            .with_nanosecond(999999000)
            .unwrap()
    );

    assert_eq!(
        root["odt4"].as_offset_date_time(),
        FixedOffset::west_opt(0)
            .unwrap()
            .with_ymd_and_hms(1979, 05, 27, 07, 32, 00)
            .unwrap()
    );

    Ok(())
}

#[test]
fn local_date_time() -> Result<()> {
    let text = "
ldt1 = 1979-05-27T07:32:00
ldt2 = 1979-05-27T00:32:00.999999
";
    let root = from_str(text)?;

    assert_eq!(
        root["ldt1"].as_local_date_time(),
        NaiveDate::from_ymd_opt(1979, 05, 27)
            .unwrap()
            .and_hms_opt(07, 32, 00)
            .unwrap()
    );

    assert_eq!(
        root["ldt2"].as_local_date_time(),
        NaiveDate::from_ymd_opt(1979, 05, 27)
            .unwrap()
            .and_hms_nano_opt(00, 32, 00, 999999000)
            .unwrap()
    );

    Ok(())
}

#[test]
fn local_date() -> Result<()> {
    let text = "ld1 = 1979-05-27";
    let root = from_str(text)?;

    assert_eq!(
        root["ld1"].as_local_date(),
        NaiveDate::from_ymd_opt(1979, 05, 27).unwrap()
    );

    Ok(())
}

#[test]
fn local_time() -> Result<()> {
    let text = "
lt1 = 07:32:00
lt2 = 00:32:00.999999
";
    let root = from_str(text)?;

    assert_eq!(
        root["lt1"].as_local_time(),
        NaiveTime::from_hms_opt(07, 32, 00).unwrap()
    );

    assert_eq!(
        root["lt2"].as_local_time(),
        NaiveTime::from_hms_nano_opt(00, 32, 00, 999999000).unwrap()
    );

    Ok(())
}
