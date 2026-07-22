//! Correctness and safety-oriented CCSDS tests for flight-adjacent use.
//!
//! Goals:
//! - Encode → decode stays within one quantum of the declared resolution (truncation).
//! - Representable values round-trip bit-identically where the format allows.
//! - Leap-second civil mapping matches Annex A ranges.
//! - Field capacity and malformed inputs fail with `Err`, never panic.
//! - Boundary day / year / ms values are explicit.

#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::civil_parts::Parts;
use deep_time::{Dt, DtErrKind, Scale};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Upper bound (exclusive) on truncation residual in attoseconds.
/// For large `n_frac`, binary ULP is < 1 atto; integer round-trip can still
/// leave a 1-atto residual on the library’s attosecond lattice.
fn cuc_max_residual_attos(n_frac: u8) -> u128 {
    if n_frac == 0 {
        return 1_000_000_000_000_000_000; // whole second
    }
    let bits = 8u32 * u32::from(n_frac);
    if bits >= 60 {
        return 2; // allow residual 0 or 1
    }
    let m = 1u128 << bits;
    // ceil(10¹⁸ / 2^(8n)) — exclusive bound for residual
    (1_000_000_000_000_000_000u128 + m - 1) / m
}

fn ccs_quantum_attos(n_subsec: u8) -> u128 {
    if n_subsec == 0 {
        return 1_000_000_000_000_000_000;
    }
    let places = 2u32 * u32::from(n_subsec);
    1_000_000_000_000_000_000u128 / 10u128.pow(places)
}

/// Decode CUC and return (coarse_secs_since_1958, frac_attos) without civil conversion.
fn cuc_raw_fields(buf: &[u8]) -> (u64, u64) {
    let p = Parts::from_ccsds_cuc(buf).unwrap();
    let dt = p.to_dt().unwrap();
    let since = dt
        .target(Scale::TAI)
        .to_scale_and_diff(Dt::CCSDS_EPOCH, false);
    (since.to_sec64_floor() as u64, since.to_sec_ufrac())
}

// ─────────────────────────────────────────────────────────────────────────────
// CUC: capacity safety (no silent truncation)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cuc_rejects_coarse_overflow() {
    // 2025 needs ≥ 4 coarse octets; 3 must fail.
    let dt = Dt::from_ymd(2025, 4, 17, Scale::TAI, 14, 30, 45, 0);
    assert!(
        matches!(
            dt.to_ccsds_cuc(3, 0, false),
            Err(e) if e.kind() == DtErrKind::OutOfRange
        ),
        "expected OutOfRange for undersized n_coarse"
    );
    // 4 octets must succeed
    assert!(dt.to_ccsds_cuc(4, 0, false).is_ok());
}

#[test]
fn cuc_max_coarse_field_boundary() {
    // n_coarse=1 holds 0..=255 seconds only
    let ok = Dt::CCSDS_EPOCH.add_sec(255);
    assert!(ok.to_ccsds_cuc(1, 0, false).is_ok());
    let bad = Dt::CCSDS_EPOCH.add_sec(256);
    assert!(matches!(
        bad.to_ccsds_cuc(1, 0, false),
        Err(e) if e.kind() == DtErrKind::OutOfRange
    ));
}

// ─────────────────────────────────────────────────────────────────────────────
// CUC: truncation quanta (encode → decode never moves forward in time)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cuc_encode_decode_within_quantum() {
    let samples: &[u64] = &[
        0,
        1,
        2,
        255,
        256,
        999,
        1_000,
        500_000_000_000_000_000,
        123_456_789_012_345_678,
        999_999_999_999_999_999,
        314_159_265_358_979_323,
        1,
    ];
    for &attos in samples {
        let dt = Dt::CCSDS_EPOCH.add_attos(attos as i128).add_sec(1_000_000);
        for n_frac in 0u8..=10 {
            let (buf, len) = dt.to_ccsds_cuc(4, n_frac, false).unwrap();
            let (secs, frac) = cuc_raw_fields(&buf[..len]);
            assert_eq!(secs, 1_000_000);
            assert!(
                frac <= attos,
                "n_frac={n_frac}: recovered {frac} > original {attos}"
            );
            let max_err = cuc_max_residual_attos(n_frac);
            let err = attos.saturating_sub(frac) as u128;
            assert!(
                err <= max_err,
                "n_frac={n_frac}: residual {err} > max {max_err} (attos={attos})"
            );
        }
    }
}

#[test]
fn cuc_exact_half_second_roundtrips() {
    // 0.5 s is exact for every binary n_frac ≥ 1
    let dt = Dt::CCSDS_EPOCH.add_attos(500_000_000_000_000_000);
    for n_frac in 1u8..=10 {
        let (buf, len) = dt.to_ccsds_cuc(4, n_frac, false).unwrap();
        let (_, frac) = cuc_raw_fields(&buf[..len]);
        assert_eq!(frac, 500_000_000_000_000_000, "n_frac={n_frac}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CDS: quanta, identity, leap seconds, day capacity
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cds_encode_decode_ms_us_ps_quanta() {
    let cases: &[(u64, u8)] = &[
        (0, 0),
        (1, 0),
        (999_000_000_000_000, 0),   // just under 1 ms
        (1_000_000_000_000_000, 0), // exact 1 ms
        (123_456_789_012_345_678, 0),
        (123_456_789_012_345_678, 1),
        (123_456_789_012_345_678, 2),
        (999_999_999_999_999_999, 1),
        (999_999_999_999_999_999, 2),
    ];
    for &(attos, sub_code) in cases {
        let dt = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, attos);
        let (buf, len) = dt.to_ccsds_cds(2, sub_code, false).unwrap();
        let p = Parts::from_ccsds_cds(&buf[..len]).unwrap();
        assert!(
            p.attos <= attos,
            "sub={sub_code}: {p_attos} > {attos}",
            p_attos = p.attos
        );
        let residual = attos - p.attos;
        let max_res = match sub_code {
            0 => 1_000_000_000_000_000u64, // < 1 ms
            1 => 1_000_000_000_000u64,     // < 1 µs
            2 => 1_000_000u64,             // < 1 ps
            _ => unreachable!(),
        };
        assert!(
            residual < max_res,
            "sub={sub_code}: residual {residual} ≥ {max_res}"
        );
    }
}

#[test]
fn cds_representable_values_identity() {
    // Exact ms + exact µs
    let dt = Dt::from_ymd(
        2020,
        6,
        15,
        Scale::UTC,
        12,
        0,
        0,
        123_456_000_000_000_000, // 123 ms + 456 µs
    );
    let (buf, len) = dt.to_ccsds_cds(2, 1, false).unwrap();
    let p = Parts::from_ccsds_cds(&buf[..len]).unwrap();
    assert_eq!(p.attos, 123_456_000_000_000_000);
    assert_eq!(p.hr, 12);
    assert_eq!(p.min, 0);
    assert_eq!(p.sec, 0);

    // Exact ps residual of ms
    let dt = Dt::from_ymd(
        2020,
        6,
        15,
        Scale::UTC,
        0,
        0,
        1,
        5_000_000_000_000_000 + 42_000_000, // 5 ms + 42 ps
    );
    let (buf, len) = dt.to_ccsds_cds(2, 2, false).unwrap();
    let p = Parts::from_ccsds_cds(&buf[..len]).unwrap();
    assert_eq!(p.attos, 5_000_000_000_000_000 + 42_000_000);
}

#[test]
fn cds_positive_leap_second_matrix() {
    // Known positive leap: 2016-12-31 23:59:60 UTC
    let leap_day = |attos: u64| Dt::from_ymd(2016, 12, 31, Scale::UTC, 23, 59, 60, attos);

    for (attos, sub_code, expect_ms, expect_sub) in [
        (0u64, 0u8, 86_400_000u32, 0u32),
        (250_000_000_000_000_000, 0, 86_400_250, 0),
        (250_000_000_000_000_000, 1, 86_400_250, 0), // exact 250 ms
        (250_123_000_000_000_000, 1, 86_400_250, 123), // +123 µs
        (1_000_000_000_000_000 - 1, 0, 86_400_000, 0), // truncate to 0 ms of leap second
        (999_000_000_000_000_000, 0, 86_400_999, 0),
    ] {
        let dt = leap_day(attos);
        assert_eq!(dt.to_ymd().sec(), 60, "civil leap second");
        let (buf, len) = dt.to_ccsds_cds(2, sub_code, false).unwrap();
        let ms = u32::from_be_bytes(buf[3..7].try_into().unwrap());
        assert_eq!(ms, expect_ms, "attos={attos} sub={sub_code}");
        if sub_code == 1 {
            let us = u16::from_be_bytes(buf[7..9].try_into().unwrap()) as u32;
            assert_eq!(us, expect_sub, "µs attos={attos}");
        }
        let p = Parts::from_ccsds_cds(&buf[..len]).unwrap();
        assert_eq!(p.sec, 60);
        assert_eq!(p.hr, 23);
        assert_eq!(p.min, 59);
        assert_eq!(p.yr, Some(2016));
        assert_eq!(p.mo, Some(12));
        assert_eq!(p.day, Some(31));
    }
}

#[test]
fn cds_day_after_leap_is_normal() {
    // 2017-01-01 00:00:00 — first second after the leap
    let dt = Dt::from_ymd(2017, 1, 1, Scale::UTC, 0, 0, 0, 0);
    let (buf, len) = dt.to_ccsds_cds(2, 0, false).unwrap();
    let ms = u32::from_be_bytes(buf[3..7].try_into().unwrap());
    assert_eq!(ms, 0);
    let p = Parts::from_ccsds_cds(&buf[..len]).unwrap();
    assert_eq!(p.sec, 0);
    assert_eq!(p.yr, Some(2017));
    assert_eq!(p.mo, Some(1));
    assert_eq!(p.day, Some(1));
}

#[test]
fn cds_day_field_capacity() {
    // 2-byte day max = 65535 days after 1958-01-01
    let (y, mo, d) = Dt::days_since_1958_to_ymd(65_535);
    let dt = Dt::from_ymd(y, mo, d, Scale::UTC, 0, 0, 0, 0);
    assert!(dt.to_ccsds_cds(2, 0, false).is_ok());

    let (y2, mo2, d2) = Dt::days_since_1958_to_ymd(65_536);
    let dt2 = Dt::from_ymd(y2, mo2, d2, Scale::UTC, 0, 0, 0, 0);
    assert!(matches!(
        dt2.to_ccsds_cds(2, 0, false),
        Err(e) if e.kind() == DtErrKind::OutOfRange
    ));
    // 3-byte day must accept it
    assert!(dt2.to_ccsds_cds(3, 0, false).is_ok());
}

#[test]
fn cds_end_of_normal_day_ms() {
    // 23:59:59.999 → 86_399_999
    let dt = Dt::from_ymd(2024, 6, 1, Scale::UTC, 23, 59, 59, 999_000_000_000_000_000);
    let (buf, _) = dt.to_ccsds_cds(2, 0, false).unwrap();
    let ms = u32::from_be_bytes(buf[3..7].try_into().unwrap());
    assert_eq!(ms, 86_399_999);
}

// ─────────────────────────────────────────────────────────────────────────────
// CCS: quanta, BCD bounds, leap second
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ccs_encode_decode_within_quantum() {
    let attos_samples = [
        0u64,
        1,
        5_000_000_000_000_000, // 0.005
        123_456_789_012_345_678,
        999_999_999_999_999_999,
    ];
    for attos in attos_samples {
        let dt = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, attos);
        for n in 0u8..=6 {
            let (buf, len) = dt.to_ccsds_ccs(false, n).unwrap();
            let p = Parts::from_ccsds_ccs(&buf[..len]).unwrap();
            assert!(p.attos <= attos, "n={n}");
            let residual = (attos - p.attos) as u128;
            assert!(
                residual < ccs_quantum_attos(n),
                "n={n}: residual {residual}"
            );
            assert_eq!(p.sec, 45);
            assert_eq!(p.yr, Some(2025));
        }
    }
}

#[test]
fn ccs_leap_second_and_year_bounds() {
    let leap = Dt::from_ymd(2016, 12, 31, Scale::UTC, 23, 59, 60, 0);
    let (buf, len) = leap.to_ccsds_ccs(false, 0).unwrap();
    assert_eq!(buf[7], 0x60); // BCD second 60
    let p = Parts::from_ccsds_ccs(&buf[..len]).unwrap();
    assert_eq!(p.sec, 60);

    let y1 = Dt::from_ymd(1, 1, 1, Scale::UTC, 0, 0, 0, 0);
    assert!(y1.to_ccsds_ccs(false, 0).is_ok());
    let y9999 = Dt::from_ymd(9999, 12, 31, Scale::UTC, 23, 59, 59, 0);
    assert!(y9999.to_ccsds_ccs(true, 2).is_ok());
}

#[test]
fn ccs_exact_decimal_identity() {
    // 0.123456 with n_subsec=3 (6 digits) is exact
    let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 0, 0, 0, 123_456_000_000_000_000);
    let (buf, len) = dt.to_ccsds_ccs(false, 3).unwrap();
    let p = Parts::from_ccsds_ccs(&buf[..len]).unwrap();
    assert_eq!(p.attos, 123_456_000_000_000_000);
}

// ─────────────────────────────────────────────────────────────────────────────
// Adversarial / safety: decode never panics
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn adversarial_short_and_random_buffers_do_not_panic() {
    // Empty and short
    for n in 0..20 {
        let buf = vec![0u8; n];
        let _ = Parts::from_ccsds_bin(&buf);
        let _ = Parts::from_ccsds_cuc(&buf);
        let _ = Parts::from_ccsds_cds(&buf);
        let _ = Parts::from_ccsds_ccs(&buf);
        let _ = Dt::from_ccsds_bin(&buf);
    }

    // Deterministic pseudo-random blobs (LCG) — all code IDs and lengths
    let mut state = 0xC0FFEEu64;
    for _ in 0..2000 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let len = (state % 24) as usize;
        let mut buf = vec![0u8; len.max(1)];
        for b in &mut buf {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            *b = (state >> 33) as u8;
        }
        // Force a mix of code IDs in the top of p-field
        buf[0] = match state % 8 {
            0 => (buf[0] & 0x0f) | 0x10, // CUC
            1 => (buf[0] & 0x0f) | 0x40, // CDS
            2 => (buf[0] & 0x0f) | 0x50, // CCS
            3 => buf[0] | 0x80,          // extension bit
            4 => 0x43,                   // CDS reserved subms
            5 => 0x48,                   // CDS agency epoch
            6 => 0x9c,                   // CUC extended
            _ => buf[0],
        };
        let _ = Parts::from_ccsds_bin(&buf);
        let _ = Parts::from_ccsds_cuc(&buf);
        let _ = Parts::from_ccsds_cds(&buf);
        let _ = Parts::from_ccsds_ccs(&buf);
    }
}

#[test]
fn valid_encodes_always_decode() {
    let instants = [
        Dt::CCSDS_EPOCH,
        Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0),
        Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0),
        Dt::from_ymd(
            2016,
            12,
            31,
            Scale::UTC,
            23,
            59,
            60,
            100_000_000_000_000_000,
        ),
        Dt::from_ymd(2025, 4, 17, Scale::TAI, 14, 30, 45, 123_456_789_012_345_678),
        Dt::from_ymd(
            9999,
            12,
            31,
            Scale::UTC,
            23,
            59,
            59,
            999_999_999_999_999_999,
        ),
    ];
    for dt in instants {
        for n_frac in [0u8, 1, 3, 4, 8] {
            if let Ok((buf, len)) = dt.to_ccsds_cuc(4, n_frac, false) {
                assert!(Parts::from_ccsds_cuc(&buf[..len]).is_ok());
                assert!(Dt::from_ccsds_cuc(&buf[..len]).is_ok());
            }
        }
        for sub in 0u8..=2 {
            if let Ok((buf, len)) = dt.to_ccsds_cds(2, sub, false) {
                assert!(Parts::from_ccsds_cds(&buf[..len]).is_ok());
                assert!(Dt::from_ccsds_cds(&buf[..len]).is_ok());
            }
            if let Ok((buf, len)) = dt.to_ccsds_cds(3, sub, true) {
                assert!(Parts::from_ccsds_cds(&buf[..len]).is_ok());
            }
        }
        for n in 0u8..=6 {
            for doy in [false, true] {
                if let Ok((buf, len)) = dt.to_ccsds_ccs(doy, n) {
                    assert!(Parts::from_ccsds_ccs(&buf[..len]).is_ok());
                    assert!(Dt::from_ccsds_ccs(&buf[..len]).is_ok());
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Dual-path: Parts and Dt APIs agree
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parts_and_dt_encode_decode_agree() {
    let dt = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, 123_456_789_000_000_000);
    let parts = Parts {
        yr: Some(2025),
        mo: Some(4),
        day: Some(17),
        hr: 14,
        min: 30,
        sec: 45,
        attos: 123_456_789_000_000_000,
        scale: Scale::UTC,
        ..Parts::default()
    };

    let (a, la) = dt.to_ccsds_cds(2, 1, false).unwrap();
    let (b, lb) = parts.to_ccsds_cds(2, 1, false).unwrap();
    assert_eq!(&a[..la], &b[..lb]);

    let (a, la) = dt.to_ccsds_ccs(false, 3).unwrap();
    let (b, lb) = parts.to_ccsds_ccs(false, 3).unwrap();
    assert_eq!(&a[..la], &b[..lb]);

    let tai = Dt::from_ymd(2025, 4, 17, Scale::TAI, 14, 30, 45, 0);
    let (a, la) = tai.to_ccsds_cuc(4, 2, false).unwrap();
    assert!(Parts::from_ccsds_cuc(&a[..la]).is_ok());
    assert_eq!(Dt::from_ccsds_cuc(&a[..la]).unwrap().to_ymd().sec(), 45);
}

// ─────────────────────────────────────────────────────────────────────────────
// Monotonicity of CUC coarse field for consecutive seconds
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cuc_coarse_seconds_monotonic() {
    let mut prev = 0u64;
    for s in 0..1000 {
        let dt = Dt::CCSDS_EPOCH.add_sec(s);
        let (buf, len) = dt.to_ccsds_cuc(4, 0, false).unwrap();
        // coarse is last 4 bytes after 1-byte p-field
        let coarse = u32::from_be_bytes(buf[1..5].try_into().unwrap()) as u64;
        assert_eq!(coarse, s as u64);
        assert!(coarse >= prev);
        prev = coarse;
        let _ = len;
    }
}
