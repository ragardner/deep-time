#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, Scale};

#[test]
fn to_gps_roundtrips() {
    let x = Dt::from_ymd(2015, 5, 20, Scale::UTC, 0, 0, 0, 0);

    let roundtrip = Dt::from_gps(x.to_gps());
    assert_eq!(
        x, roundtrip,
        "Round trip gps was not equal: {}, {}",
        x, roundtrip
    );

    let roundtrip = Dt::from_cxcsec(x.to_cxcsec());
    assert_eq!(
        x, roundtrip,
        "Round trip cxcsec was not equal: {}, {}",
        x, roundtrip
    );

    let (w, tow) = x.to_gps_wk_and_tow();
    let roundtrip = Dt::from_gps_wk_and_tow(w, tow);
    assert_eq!(
        x, roundtrip,
        "Round trip gps wk and tow was not equal: {}, {}",
        x, roundtrip
    );
}

#[test]
fn to_gps_wk_and_tow_expected_values() {
    let x = Dt::from_ymd(2015, 5, 20, Scale::GPS, 0, 0, 0, 0);
    let (wk, tow) = x.to_gps_wk_and_tow();
    assert_eq!(wk, 1845);
    assert_eq!(tow.to_sec(), 259200); // 3*86400 → Wednesday
    assert_eq!(x.to_gps_day_of_wk(), 3);

    // Epoch: 1980-01-06 00:00:00 GPS → week 0, TOW 0, Sunday
    let x = Dt::from_ymd(1980, 1, 6, Scale::GPS, 0, 0, 0, 0);
    let (wk, tow) = x.to_gps_wk_and_tow();
    assert_eq!(wk, 0, "epoch week");
    assert_eq!(tow.to_sec(), 0, "epoch TOW");
    assert_eq!(x.to_gps_day_of_wk(), 0, "epoch DOW (Sunday)");

    // Epoch + 12 hours
    let x = Dt::from_ymd(1980, 1, 6, Scale::GPS, 12, 0, 0, 0);
    let (wk, tow) = x.to_gps_wk_and_tow();
    assert_eq!(wk, 0);
    assert_eq!(tow.to_sec(), 43200); // 12*3600
    assert_eq!(x.to_gps_day_of_wk(), 0);

    // Exactly one week later
    let x = Dt::from_ymd(1980, 1, 13, Scale::GPS, 0, 0, 0, 0);
    let (wk, tow) = x.to_gps_wk_and_tow();
    assert_eq!(wk, 1);
    assert_eq!(tow.to_sec(), 0);
    assert_eq!(x.to_gps_day_of_wk(), 0);

    // 2000-01-01 00:00:00 GPS (matches IGS week ~1042 period)
    let x = Dt::from_ymd(2000, 1, 1, Scale::GPS, 0, 0, 0, 0);
    let (wk, tow) = x.to_gps_wk_and_tow();
    assert_eq!(wk, 1042);
    assert_eq!(tow.to_sec(), 518400); // 6*86400 → Saturday
    assert_eq!(x.to_gps_day_of_wk(), 6);

    // 2000-01-01 12:00:00 GPS (close to common J2000-ish test points)
    let x = Dt::from_ymd(2000, 1, 1, Scale::GPS, 12, 0, 0, 0);
    let (wk, tow) = x.to_gps_wk_and_tow();
    assert_eq!(wk, 1042);
    assert_eq!(tow.to_sec(), 561600);

    // With sub-day time
    let x = Dt::from_ymd(2015, 5, 20, Scale::GPS, 15, 30, 0, 0);
    let (wk, tow) = x.to_gps_wk_and_tow();
    assert_eq!(wk, 1845);
    assert_eq!(tow.to_sec(), 315000);
}
