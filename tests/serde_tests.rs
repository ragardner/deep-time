#![cfg(feature = "serde")]

use deep_time::{Dt, DtErr, Scale, YmdHms};

#[test]
fn dt_roundtrip() {
    let original = Dt::from_ymd(2025, 6, 29, Scale::UTC, 14, 30, 45, 123456789012345678);

    let json = serde_json::to_string(&original).unwrap();

    let deserialized: Dt = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn dt_negative_year_roundtrip() {
    let original = Dt::from_ymd(-1234, 12, 31, Scale::TAI, 23, 59, 59, 0);

    let json = serde_json::to_string(&original).unwrap();

    let deserialized: Dt = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn ymdhms_roundtrip() {
    let dt = Dt::from_ymd(2030, 1, 15, Scale::GPS, 8, 45, 0, 987654321000000000);
    let original: YmdHms = dt.to_ymd();

    let json = serde_json::to_string(&original).unwrap();

    let deserialized: YmdHms = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn scale_roundtrip() {
    let scales = [Scale::TAI, Scale::UTC, Scale::TT, Scale::TDB, Scale::GPS];

    for scale in scales {
        let json = serde_json::to_string(&scale).unwrap();

        let deserialized: Scale = serde_json::from_str(&json).unwrap();
        assert_eq!(scale, deserialized);
    }
}

#[test]
fn dt_err_roundtrip() {
    let err: DtErr = deep_time::an_err!(deep_time::DtErrKind::InvalidInput, "test error");

    let json = serde_json::to_string(&err).unwrap();

    let deserialized: DtErr = serde_json::from_str(&json).unwrap();
    assert_eq!(err.to_string(), deserialized.to_string());
}
