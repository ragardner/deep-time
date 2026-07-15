use deep_time::macros::from_ns;
use deep_time::{BufStr, Dt, DtErr, Lang, ParseCfg, Scale, YmdHms, from_ymd};

fn main() -> Result<(), DtErr> {
    // ============================================
    // Parsing
    // ============================================

    // Smart auto-parsing (multi-language + timezone)
    let cfg = ParseCfg {
        lang: Lang::Fr,
        ..Default::default()
    };
    let dt = Dt::from_str_parse("15 août 2024 à 14:30 [Europe/Paris]", &cfg)?;
    let s = dt.to_str_rfc9557("Europe/Paris")?;
    assert_eq!("2024-08-15T14:30:00+02:00[Europe/Paris]", s);

    // or with .parse
    let dt: Dt = "1 jan 2000 07:00 [America/New_York] TAI".parse()?; // noon
    assert_eq!(Dt::ZERO, dt);

    // Relative dates are also supported
    let ref_time = from_ymd!(2026, 6, 16; 12, on=Scale::UTC);
    let en_cfg = ParseCfg {
        ref_time: Some(ref_time),
        ..Default::default()
    };

    // from_ymd! macro defaults to Scale::UTC
    let dt = Dt::from_str_parse("2 days from now at 9am", &en_cfg)?;
    assert_eq!(dt, from_ymd!(2026, 6, 18; 9));

    let dt = Dt::from_str_parse("next Monday at 14:00", &en_cfg)?;
    assert_eq!(dt, from_ymd!(2026, 6, 22; 14));

    // Relative dates use Dt::now if the `std` feature is enabled and no
    // ref_time is provided in the ParseCfg
    let _ = Dt::from_str_parse("next Monday at 14:00", &ParseCfg::DEFAULT)?;

    // Fast ISO parsing with time scale and no alloc output
    let dt = Dt::from_str_iso("2000-01-01T12:00:00 TAI")?;
    let buf_str: BufStr<512> = dt.to_str_b_iso8601();
    assert_eq!("2000-01-01T12:00:00+00:00", buf_str.as_str());

    // ============================================
    // Formatting
    // ============================================

    let s = dt.to_str_in_tz("%A, %d %B %Y %I:%M%P", "America/New_York", Lang::En)?;
    assert_eq!("Saturday, 01 January 2000 07:00am", s);

    let s = dt.to_str_in_tz("%A, %-d de %B de %Y %H:%M", "America/New_York", Lang::Es)?;
    assert_eq!("Sábado, 1 de enero de 2000 07:00", s);

    // ============================================
    // Duration parsing
    // ============================================

    let span: Dt = Dt::from_str_duration("3 days 12 hours", Lang::En)?;
    let dur = span.to_str_b_media_duration();
    assert_eq!("3:12:00:00", dur.to_string());

    // ============================================
    // Time scale conversions + round-tripping
    // ============================================

    let dt = Dt::from_ymd(2000, 1, 1, Scale::TAI, 0, 0, 0, 123456789);
    let tt = dt.to(Scale::TT);
    let tdb = tt.to(Scale::TDB);
    let ltc = tdb.to(Scale::LTC);
    let utc = ltc.to(Scale::UTC);
    let tcl = utc.to(Scale::TCL);
    let tcg = tcl.to(Scale::TCG);
    let tai = tcg.to_tai();

    // round trips work for pretty much everything except UTCHist
    assert_eq!(dt, tai);
    let ymd: YmdHms = tai.to_ymd();
    assert_eq!(ymd.attos(), 123456789);

    // ============================================
    // Other conversions
    // ============================================

    // unix
    let dt = from_ymd!(1970);
    let unix = dt.to_unix().to_sec_f();
    assert_eq!(unix, 0.0);

    let dt = Dt::from_unix(from_ns!(0, on = Scale::UTC));
    assert_eq!(dt, Dt::UNIX_EPOCH);

    // or to milliseconds
    let unix: i128 = dt.add_ms(1000).to_unix().to_ms().0;
    assert_eq!(unix, 1000);

    // to and from jd
    let jd = Dt::ZERO.to_jd_f_raw();
    assert_eq!(2451545.0, jd);
    let dt = Dt::from_jd_f(jd, Scale::TAI);
    assert_eq!(0, dt.attos);

    // ============================================
    // Calendar math
    // ============================================

    // calendar math and negative year
    let dt = from_ymd!(-2000, 1, 31; 12, on=Scale::TAI);
    let ymd = dt.add_months(1).to_ymd();
    assert_eq!(ymd.day(), 29);

    // Timezone-aware calendar math (respects DST transitions, requires jiff-tz feature)
    let dt = Dt::from_str_iso("2025-03-30T00:30:00Z")?; // Just before London DST start

    // Normal (naive) addition — ignores DST rules
    let normal = dt.add_hours(1);

    // Timezone-aware addition — correctly handles the transition
    let aware = dt.add_hours_tz(1, "Europe/London")?;

    assert_eq!(
        normal.to_str_rfc9557("Europe/London")?,
        "2025-03-30T02:30:00+01:00[Europe/London]"
    );
    assert_eq!(
        aware.to_str_rfc9557("Europe/London")?,
        "2025-03-30T03:30:00+01:00[Europe/London]"
    );

    // ============================================
    // Leap seconds
    // ============================================

    // genuine leap second input round trips
    let dt: Dt = "2015-06-30T23:59:60".parse()?;
    let s = dt.to_str_iso8601();
    assert_eq!("2015-06-30T23:59:60+00:00", s);

    Ok(())
}
