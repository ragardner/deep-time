use deep_time::{Dt, Scale};

#[test]
fn to_gps_roundtrips() {
    let x = Dt::from_ymd(2015, 5, 20);

    let roundtrip = Dt::from_gps(x.to_gps(Scale::TAI));
    assert_eq!(
        x, roundtrip,
        "Round trip gps was not equal: {}, {}",
        x, roundtrip
    );

    let roundtrip = Dt::from_cxcsec(x.to_cxcsec(Scale::TAI));
    assert_eq!(
        x, roundtrip,
        "Round trip gps was not equal: {}, {}",
        x, roundtrip
    );

    let (w, tow) = x.to_gps_wk_and_tow(Scale::TAI);
    let roundtrip = Dt::from_gps_wk_and_tow(w, tow);
    assert_eq!(
        x, roundtrip,
        "Round trip gps was not equal: {}, {}",
        x, roundtrip
    );
}
