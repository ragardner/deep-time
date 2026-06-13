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

// TODO: add expected value tests
