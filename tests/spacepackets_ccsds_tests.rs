//! Interop tests: deep-time CCSDS codecs vs the
//! [`spacepackets`](https://crates.io/crates/spacepackets) crate (v0.18).
//!
//! Focus is **CDS** (full segment support including µs/ps of ms) and **CUC**
//! where spacepackets' API allows (≤4 coarse octets, ≤3 fine octets).
//!
//! Notes on intentional differences:
//! - spacepackets CDS built from UNIX ignores leap seconds (fixed 86_400 s days).
//!   Our CDS is civil-UTC / leap-second aware. Comparisons use non-leap instants
//!   or construct both sides from the same segment counters.
//! - spacepackets CUC counters are `u32` (4 coarse octets max).

#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::civil_parts::Parts;
use deep_time::{Dt, Scale};
use spacepackets::time::cds::{CdsBase, CdsTime, SubmillisPrecision};
use spacepackets::time::cuc::{
    CucTime, FractionalPart, FractionalResolution, WidthCounterPair, fractional_part_from_subsec_ns,
};
use spacepackets::time::{TimeReader, TimeWriter, UnixTime};

// ─────────────────────────────────────────────────────────────────────────────
// CDS helpers
// ─────────────────────────────────────────────────────────────────────────────

fn pack_sp_cds(days: u16, ms_of_day: u32, prec: SubmillisPrecision, submillis: u32) -> Vec<u8> {
    let mut stamp = CdsTime::new_with_u16_days(days, ms_of_day);
    assert!(
        stamp.set_submillis(prec, submillis),
        "spacepackets set_submillis failed (value out of width?)"
    );
    let mut buf = [0u8; 16];
    let n = stamp
        .write_to_bytes(&mut buf)
        .expect("spacepackets CDS write");
    buf[..n].to_vec()
}

fn days_since_1958(y: i64, mo: u8, d: u8) -> u16 {
    let days = Dt::ymd_to_days_since_1958(y, mo, d);
    assert!(days >= 0, "before CDS epoch");
    u16::try_from(days).expect("day count exceeds 16-bit CDS field")
}

// ─────────────────────────────────────────────────────────────────────────────
// CDS: wire identity from shared segment counters
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cds_wire_identity_from_shared_segments() {
    // (days, ms_of_day, prec, submillis)
    let cases: &[(u16, u32, SubmillisPrecision, u32)] = &[
        (0, 0, SubmillisPrecision::Absent, 0),
        (4383, 0, SubmillisPrecision::Absent, 0), // 1970-01-01
        (15340, 0, SubmillisPrecision::Absent, 0), // 2000-01-01
        (15340, 43_200_000, SubmillisPrecision::Absent, 0), // noon
        (24_578, 52_245_000, SubmillisPrecision::Absent, 0), // 2025-04-17 14:30:45
        (24_578, 52_245_123, SubmillisPrecision::Microseconds, 456),
        (
            24_578,
            52_245_123,
            SubmillisPrecision::Picoseconds,
            456_789_012,
        ),
        (0, 1, SubmillisPrecision::Microseconds, 500), // 1.5 ms
        (0, 86_399_999, SubmillisPrecision::Microseconds, 999),
    ];

    for &(days, ms, prec, sub) in cases {
        let sp = pack_sp_cds(days, ms, prec, sub);

        // Rebuild the same instant in deep-time from the segment meaning and encode.
        let (y, mo, d) = Dt::days_since_1958_to_ymd(days as i64);
        let total_ms = ms as u64;
        let is_leap = total_ms / 1000 == 86_400;
        let sec_of_day = if is_leap { 86_399 } else { total_ms / 1000 };
        let rem_ms = total_ms % 1000;
        let hr = (sec_of_day / 3600) as u8;
        let min = ((sec_of_day % 3600) / 60) as u8;
        let mut sec = (sec_of_day % 60) as u8;
        if is_leap {
            sec = 60;
        }

        let attos_in_sec = match prec {
            SubmillisPrecision::Absent => rem_ms * 1_000_000_000_000_000,
            SubmillisPrecision::Microseconds => {
                rem_ms * 1_000_000_000_000_000 + (sub as u64) * 1_000_000_000_000
            }
            SubmillisPrecision::Picoseconds => {
                rem_ms * 1_000_000_000_000_000 + (sub as u64) * 1_000_000
            }
            SubmillisPrecision::Reserved => unreachable!(),
        };

        let sub_ms_code = match prec {
            SubmillisPrecision::Absent => 0,
            SubmillisPrecision::Microseconds => 1,
            SubmillisPrecision::Picoseconds => 2,
            SubmillisPrecision::Reserved => unreachable!(),
        };

        let dt = Dt::from_ymd(y, mo, d, Scale::UTC, hr, min, sec, attos_in_sec);
        let (buf, len) = dt
            .to_ccsds_cds(2, sub_ms_code, false)
            .expect("deep-time CDS encode");

        assert_eq!(
            &buf[..len],
            sp.as_slice(),
            "wire mismatch days={days} ms={ms} prec={prec:?} sub={sub}\n\
             deep-time={:02x?}\nspacepackets={:02x?}",
            &buf[..len],
            &sp
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CDS: deep-time encode → spacepackets decode
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cds_deep_time_encode_spacepackets_decode() {
    let cases: &[(i64, u8, u8, u8, u8, u8, u64, u8)] = &[
        // y, mo, d, h, min, sec, attos, sub_ms_code
        (1958, 1, 1, 0, 0, 0, 0, 0),
        (1970, 1, 1, 0, 0, 0, 0, 0),
        (2000, 1, 1, 12, 0, 0, 0, 0),
        (2025, 4, 17, 14, 30, 45, 0, 0),
        (2025, 4, 17, 14, 30, 45, 123_456_789_000_000_000, 1),
        (2025, 4, 17, 14, 30, 45, 123_456_789_012_345_678, 2),
        (1958, 1, 1, 0, 0, 0, 1_500_000_000_000_000, 1),
    ];

    for &(y, mo, d, h, mi, s, attos, sub_code) in cases {
        let dt = Dt::from_ymd(y, mo, d, Scale::UTC, h, mi, s, attos);
        let (buf, len) = dt.to_ccsds_cds(2, sub_code, false).unwrap();

        let sp = CdsTime::from_bytes_with_u16_days(&buf[..len])
            .unwrap_or_else(|e| panic!("spacepackets decode failed for {y}-{mo}-{d}: {e:?}"));

        let expected_days = days_since_1958(y, mo, d);
        assert_eq!(sp.ccsds_days(), expected_days, "days {y}-{mo}-{d}");

        let sec_of_day = u32::from(h) * 3600 + u32::from(mi) * 60 + u32::from(s);
        let ms_in_sec = (attos / 1_000_000_000_000_000) as u32;
        let expected_ms = sec_of_day * 1000 + ms_in_sec;
        assert_eq!(sp.ms_of_day(), expected_ms, "ms {y}-{mo}-{d}");

        let attos_in_ms = attos % 1_000_000_000_000_000;
        match sub_code {
            0 => assert_eq!(sp.submillis_precision(), SubmillisPrecision::Absent),
            1 => {
                assert_eq!(sp.submillis_precision(), SubmillisPrecision::Microseconds);
                let us = (attos_in_ms / 1_000_000_000_000) as u32;
                assert_eq!(sp.submillis(), us, "µs {y}-{mo}-{d}");
            }
            2 => {
                assert_eq!(sp.submillis_precision(), SubmillisPrecision::Picoseconds);
                let ps = (attos_in_ms / 1_000_000) as u32;
                assert_eq!(sp.submillis(), ps, "ps {y}-{mo}-{d}");
            }
            _ => unreachable!(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CDS: spacepackets encode → deep-time decode
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cds_spacepackets_encode_deep_time_decode() {
    let cases: &[(u16, u32, SubmillisPrecision, u32)] = &[
        (0, 0, SubmillisPrecision::Absent, 0),
        (15_340, 43_200_000, SubmillisPrecision::Absent, 0),
        (24_578, 52_245_123, SubmillisPrecision::Microseconds, 456),
        (
            24_578,
            52_245_123,
            SubmillisPrecision::Picoseconds,
            456_000_000,
        ),
        (0, 1, SubmillisPrecision::Microseconds, 500),
    ];

    for &(days, ms, prec, sub) in cases {
        let sp_bytes = pack_sp_cds(days, ms, prec, sub);
        let parsed = Parts::from_ccsds_cds(&sp_bytes)
            .unwrap_or_else(|e| panic!("deep-time decode failed: {e:?}"));

        let (y, mo, d) = Dt::days_since_1958_to_ymd(days as i64);
        assert_eq!(parsed.yr, Some(y));
        assert_eq!(parsed.mo, Some(mo));
        assert_eq!(parsed.day, Some(d));

        let rem_ms = (ms % 1000) as u64;
        let expected_attos = match prec {
            SubmillisPrecision::Absent => rem_ms * 1_000_000_000_000_000,
            SubmillisPrecision::Microseconds => {
                rem_ms * 1_000_000_000_000_000 + (sub as u64) * 1_000_000_000_000
            }
            SubmillisPrecision::Picoseconds => {
                rem_ms * 1_000_000_000_000_000 + (sub as u64) * 1_000_000
            }
            SubmillisPrecision::Reserved => unreachable!(),
        };
        assert_eq!(parsed.attos, expected_attos, "attos days={days} ms={ms}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CDS: from_unix_time path (spacepackets) vs deep-time civil encode
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cds_from_unix_matches_deep_time_for_non_leap_instants() {
    // Unix times away from leap-second discontinuities. spacepackets uses fixed
    // 86_400 s days; deep-time uses civil UTC — they agree except at leap seconds.
    let cases: &[(i64, u32, SubmillisPrecision)] = &[
        // unix_secs, subsec_nanos, precision
        (0, 0, SubmillisPrecision::Absent),           // 1970-01-01
        (946_684_800, 0, SubmillisPrecision::Absent), // 2000-01-01
        (946_728_000, 0, SubmillisPrecision::Absent), // 2000-01-01 12:00
        (1_744_900_245, 0, SubmillisPrecision::Absent), // ~2025-04-17 14:30:45 UTC
        (1_744_900_245, 123_456_789, SubmillisPrecision::Microseconds),
        (1_744_900_245, 123_456_789, SubmillisPrecision::Picoseconds),
        (1_744_900_245, 123_000_000, SubmillisPrecision::Absent),
    ];

    for &(unix_secs, sub_ns, prec) in cases {
        let unix = UnixTime::new_checked(unix_secs, sub_ns).expect("unix");
        let sp = CdsTime::from_unix_time_with_u16_days(&unix, prec)
            .unwrap_or_else(|e| panic!("from_unix failed secs={unix_secs}: {e:?}"));

        let mut sp_buf = [0u8; 16];
        let sp_len = sp.write_to_bytes(&mut sp_buf).expect("sp write");

        // Same UNIX instant via deep-time: UTC attoseconds since 1970 → TAI, then CDS.
        let unix_dt = Dt::from_sec(unix_secs as i128, Scale::UTC, Scale::UTC)
            .add_attos((sub_ns as i128) * 1_000_000_000); // ns → attos
        let dt = Dt::from_unix(unix_dt).target(Scale::UTC);

        let sub_code = match prec {
            SubmillisPrecision::Absent => 0,
            SubmillisPrecision::Microseconds => 1,
            SubmillisPrecision::Picoseconds => 2,
            SubmillisPrecision::Reserved => unreachable!(),
        };
        let (dt_buf, dt_len) = dt.to_ccsds_cds(2, sub_code, false).unwrap();

        assert_eq!(
            &dt_buf[..dt_len],
            &sp_buf[..sp_len],
            "unix={unix_secs}.{sub_ns:09} prec={prec:?}\n\
             deep-time={:02x?}\nspacepackets={:02x?}",
            &dt_buf[..dt_len],
            &sp_buf[..sp_len]
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CUC: wire identity for 4 coarse + 0..3 fine (spacepackets' full CUC range)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cuc_wire_identity_coarse_and_fine() {
    // Whole seconds since 1958 TAI (spacepackets CucTime::new is a raw counter).
    for secs in [0u32, 1, 86_400, 1_000_000, 0x0102_0304] {
        let sp = CucTime::new(secs);
        let mut sp_buf = [0u8; 16];
        let sp_len = sp.write_to_bytes(&mut sp_buf).unwrap();

        let dt = Dt::CCSDS_EPOCH.add_sec(secs as i128);
        let (dt_buf, dt_len) = dt.to_ccsds_cuc(4, 0, false).unwrap();
        assert_eq!(&dt_buf[..dt_len], &sp_buf[..sp_len], "CUC secs={secs}");
    }

    // 0.5 s binary fraction with 1 fine octet → 0x80
    {
        let sp = CucTime::new_with_coarse_fractions(0, 0x80);
        let mut sp_buf = [0u8; 16];
        let sp_len = sp.write_to_bytes(&mut sp_buf).unwrap();

        let dt = Dt::CCSDS_EPOCH.add_attos(500_000_000_000_000_000);
        let (dt_buf, dt_len) = dt.to_ccsds_cuc(4, 1, false).unwrap();
        assert_eq!(&dt_buf[..dt_len], &sp_buf[..sp_len], "CUC 0.5s 1-frac");
    }

    // Match spacepackets' ns→fraction conversion for 1/2/3 fine octets.
    let ns_samples = [
        0u64,
        1,
        500_000_000,
        123_456_789,
        999_999_999,
        1_000_000,
        15_625_000, // exact 1/64 s
    ];
    for ns in ns_samples {
        for (res, n_frac) in [
            (FractionalResolution::FourMs, 1u8),
            (FractionalResolution::FifteenUs, 2),
            (FractionalResolution::SixtyNs, 3),
        ] {
            let frac = fractional_part_from_subsec_ns(res, ns);
            let sp = CucTime::new_with_fractions(42, frac).expect("cuc fractions");
            let mut sp_buf = [0u8; 16];
            let sp_len = sp.write_to_bytes(&mut sp_buf).unwrap();

            // deep-time: same coarse seconds + attos from ns
            let dt = Dt::CCSDS_EPOCH
                .add_sec(42)
                .add_attos((ns as i128) * 1_000_000_000);
            let (dt_buf, dt_len) = dt.to_ccsds_cuc(4, n_frac, false).unwrap();

            assert_eq!(
                &dt_buf[..dt_len],
                &sp_buf[..sp_len],
                "CUC ns={ns} n_frac={n_frac} res={res:?}\n\
                 deep-time={:02x?}\nspacepackets={:02x?}",
                &dt_buf[..dt_len],
                &sp_buf[..sp_len]
            );
        }
    }
}

#[test]
fn cuc_deep_time_encode_spacepackets_decode() {
    let dt = Dt::CCSDS_EPOCH
        .add_sec(1_000_000)
        .add_attos(500_000_000_000_000_000);
    let (buf, len) = dt.to_ccsds_cuc(4, 1, false).unwrap();

    let sp = CucTime::from_bytes(&buf[..len]).expect("spacepackets CUC decode");
    assert_eq!(sp.counter(), 1_000_000);
    assert_eq!(sp.counter_width(), 4);
    assert_eq!(sp.fractions().resolution(), FractionalResolution::FourMs);
    assert_eq!(sp.fractions().counter(), 0x80);
}

#[test]
fn cuc_spacepackets_encode_deep_time_decode() {
    let frac = FractionalPart::new_checked(FractionalResolution::FifteenUs, 0x1234).expect("frac");
    let sp = CucTime::new_generic(WidthCounterPair(4, 0x00AA_BBCC), frac).expect("cuc");
    let mut sp_buf = [0u8; 16];
    let sp_len = sp.write_to_bytes(&mut sp_buf).unwrap();

    let parsed = Parts::from_ccsds_cuc(&sp_buf[..sp_len]).expect("deep-time CUC decode");
    // Coarse seconds 0x00AABBCC since 1958-01-01 TAI
    let recovered = parsed.to_dt().unwrap();
    let since = recovered
        .target(Scale::TAI)
        .to_scale_and_diff(Dt::CCSDS_EPOCH, false);
    assert_eq!(since.to_sec64_floor(), 0x00AA_BBCC);
    // Fine: floor(0x1234 / 2^16 * 1e18) attos
    let expected_attos = (0x1234u128 * 1_000_000_000_000_000_000u128) / (1u128 << 16);
    assert_eq!(parsed.attos as u128, expected_attos);
}

// ─────────────────────────────────────────────────────────────────────────────
// 24-bit day CDS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cds_24bit_day_wire_identity() {
    // Day count that needs 3 octets ( > 65535 )
    let days: u32 = 100_000;
    let ms: u32 = 12_345_678;

    let stamp = CdsTime::new_with_u24_days(days, ms).expect("u24 days");
    let mut sp_buf = [0u8; 16];
    let sp_len = stamp.write_to_bytes(&mut sp_buf).expect("write");

    let (y, mo, d) = Dt::days_since_1958_to_ymd(days as i64);
    let sec_of_day = (ms / 1000) as u64;
    let rem_ms = (ms % 1000) as u64;
    let hr = (sec_of_day / 3600) as u8;
    let min = ((sec_of_day % 3600) / 60) as u8;
    let sec = (sec_of_day % 60) as u8;
    let attos = rem_ms * 1_000_000_000_000_000;

    let dt = Dt::from_ymd(y, mo, d, Scale::UTC, hr, min, sec, attos);
    let (dt_buf, dt_len) = dt.to_ccsds_cds(3, 0, false).unwrap();

    assert_eq!(
        &dt_buf[..dt_len],
        &sp_buf[..sp_len],
        "24-bit day CDS wire mismatch"
    );
}
