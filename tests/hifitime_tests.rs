#[cfg(feature = "hifitime")]
mod tests {
    use deep_time::{ClockType, TimePoint};
    use hifitime::{Duration, Epoch, TimeScale};
    /// Seconds between hifitime's TAI reference epoch (1900-01-01 00:00:00 TAI)
    /// and our library's `ZERO` (2000-01-01 12:00:00 TAI).
    const HIFITIME_TAI_EPOCH_TO_OUR_ZERO: i64 = 3_155_716_800;

    /// Map our `ClockType` to the equivalent `hifitime::TimeScale`.
    fn to_hifitime(ct: ClockType) -> Option<TimeScale> {
        match ct {
            ClockType::TAI => Some(TimeScale::TAI),
            ClockType::TT => Some(TimeScale::TT),
            ClockType::TDB => Some(TimeScale::TDB),
            ClockType::UTC => Some(TimeScale::UTC),
            ClockType::GPS | ClockType::QZSS => Some(TimeScale::GPST),
            ClockType::GST => Some(TimeScale::GST),
            ClockType::BDT => Some(TimeScale::BDT),
            _ => None,
        }
    }

    /// Returns hifitime's TAI representation as `(sec, attos)` using integer nanoseconds (no f64 loss).
    fn hifitime_tai_parts(hi: Epoch) -> (i64, u64) {
        let tai = hi.to_time_scale(TimeScale::TAI);
        let ref_tai = Epoch::from_tai_seconds(HIFITIME_TAI_EPOCH_TO_OUR_ZERO as f64);
        let dur: Duration = tai - ref_tai;

        let total_ns = dur.total_nanoseconds();
        let ns_per_sec = 1_000_000_000i128;

        if total_ns >= 0 {
            let sec = (total_ns / ns_per_sec) as i64;
            let ns_rem = (total_ns % ns_per_sec) as u64;
            let attos = ns_rem * 1_000_000_000;
            (sec, attos)
        } else {
            let attos_pos = (-total_ns) as u128;
            let sec_pos = (attos_pos / ns_per_sec as u128) as i64;
            let rem = (attos_pos % ns_per_sec as u128) as u64;
            if rem == 0 {
                (-sec_pos, 0)
            } else {
                (-sec_pos - 1, (ns_per_sec as u64 - rem) * 1_000_000_000)
            }
        }
    }

    fn assert_tp_matches_hifitime(tp: TimePoint, hi: Epoch, msg: &str) {
        let our_tai = tp.to_type(ClockType::TAI);

        let (our_sec, our_attos) = (our_tai.sec(), our_tai.subsec());
        let (hi_sec, hi_attos) = hifitime_tai_parts(hi);

        assert_eq!(our_sec, hi_sec, "{} — TAI seconds differ", msg);
        // Truncate to ns precision (hifitime is only ns-precise)
        assert_eq!(
            our_attos / 1_000_000_000 * 1_000_000_000,
            hi_attos,
            "{} — TAI attoseconds differ (to ns precision)",
            msg
        );
    }

    #[test]
    fn test_utc_leap_second() {
        let hi_utc = Epoch::from_gregorian(2016, 12, 31, 23, 59, 60, 0, TimeScale::UTC);
        let hi_tai = hi_utc.to_time_scale(TimeScale::TAI);
        let (hi_tai_sec, hi_tai_subsec) = hifitime_tai_parts(hi_tai);
        let our_tai = TimePoint::new(hi_tai_sec, hi_tai_subsec, ClockType::TAI);
        let our_utc = our_tai.to_type(ClockType::UTC);

        assert_tp_matches_hifitime(our_utc, hi_tai, "UTC leap second 2016-12-31");
    }

    #[test]
    fn test_j2000_zero_points() {
        let our = TimePoint::from(0, 0, ClockType::TAI);
        let hi = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);
        assert_tp_matches_hifitime(our, hi, "J2000 TAI zero");

        let our = TimePoint::from(0, 0, ClockType::TT);
        let hi = Epoch::from_gregorian_tai(2000, 1, 1, 11, 59, 27, 816_000_000);
        assert_tp_matches_hifitime(our, hi, "J2000 TT zero");

        let our = TimePoint::from(0, 0, ClockType::GPS);
        let hi = Epoch::from_gregorian(2000, 1, 1, 12, 0, 0, 0, TimeScale::GPST);
        assert_tp_matches_hifitime(our, hi, "J2000 GPST zero");

        let our = TimePoint::from(0, 0, ClockType::BDT);
        let hi = Epoch::from_gregorian(2000, 1, 1, 12, 0, 0, 0, TimeScale::BDT);
        assert_tp_matches_hifitime(our, hi, "J2000 BDT zero");
    }

    #[test]
    fn test_scale_conversions_all_directions() {
        let base_tai_sec = 123_456_789_i64;
        let base_attos = 987_654_321_000_000_000u64;

        let our_base = TimePoint::new(base_tai_sec, base_attos, ClockType::TAI);

        let scales: &[ClockType] = &[
            ClockType::TAI,
            ClockType::TT,
            ClockType::TDB,
            ClockType::UTC,
            ClockType::GPS,
            ClockType::GST,
            ClockType::BDT,
            ClockType::QZSS,
        ];

        for &from in scales {
            if to_hifitime(from).is_none() {
                continue;
            }
            let our_from = our_base.to_type(from);

            for &to in scales {
                if to_hifitime(to).is_none() {
                    continue;
                }

                let our_to = our_from.to_type(to);

                let ns_since_our_zero = (base_tai_sec as i128) * 1_000_000_000i128
                    + (base_attos / 1_000_000_000) as i128;

                let hi_base =
                    Epoch::from_tai_seconds(
                        (HIFITIME_TAI_EPOCH_TO_OUR_ZERO as i128 + ns_since_our_zero / 1_000_000_000)
                            as f64,
                    ) + Duration::from_nanoseconds((ns_since_our_zero % 1_000_000_000) as f64);

                let hi_from = hi_base.to_time_scale(to_hifitime(from).unwrap());
                let hi_to = hi_from.to_time_scale(to_hifitime(to).unwrap());

                assert_tp_matches_hifitime(our_to, hi_to, &format!("{:?} → {:?}", from, to));
            }
        }
    }

    #[test]
    fn test_negative_and_subsecond() {
        // Use a smaller negative value that hifitime handles cleanly
        let our = TimePoint::from(-1_000_000_000i64, 123_456_789_012_345_678, ClockType::GPS);

        let delta = Duration::from_seconds(-1_000_000_000f64)
            + Duration::from_nanoseconds(123_456_789_012_345_678u64 as f64 / 1_000_000_000.0);
        let gpst_zero = Epoch::from_gregorian(2000, 1, 1, 12, 0, 0, 0, TimeScale::GPST);
        let hi = gpst_zero + delta;

        assert_tp_matches_hifitime(our, hi, "negative GPST with sub-second");
    }

    #[test]
    fn test_all_leap_second_epochs() {
        let cases: &[(i64, &str)] = &[
            (489_024_000, "after 1998-12-31 leap"),
            (536_544_000, "after 2016-12-31 leap"),
            (599_616_000, "2019-01-01 (no leap, but near epoch)"),
        ];

        for &(tai_sec, label) in cases {
            let our = TimePoint::new(tai_sec, 0, ClockType::TAI);
            let hi = Epoch::from_tai_seconds((HIFITIME_TAI_EPOCH_TO_OUR_ZERO + tai_sec) as f64);
            assert_tp_matches_hifitime(our, hi, label);
        }
    }

    #[test]
    fn test_multiple_leap_second_dates() {
        // Known leap second dates and the TAI-UTC offset *after* each insertion
        // (using IERS-only table, consistent with hifitime's default behavior)
        let cases: &[(i32, u8, u8, u8, u8, u8, i32)] = &[
            (1999, 1, 1, 0, 0, 0, 32), // After 1998-12-31 leap
            (2006, 1, 1, 0, 0, 0, 33), // After 2005-12-31 leap
            (2009, 1, 1, 0, 0, 0, 34), // After 2008-12-31 leap
            (2012, 7, 1, 0, 0, 0, 35), // After 2012-06-30 leap
            (2015, 7, 1, 0, 0, 0, 36), // After 2015-06-30 leap
            (2017, 1, 1, 0, 0, 0, 37), // After 2016-12-31 leap
        ];

        for &(year, month, day, hour, min, sec, expected_offset) in cases {
            let utc = Epoch::from_gregorian(year, month, day, hour, min, sec, 0, TimeScale::UTC);

            let offset = utc
                .leap_seconds(true) // IERS-only (post-1972)
                .expect("Leap second date should be after 1960")
                .round() as i32;

            assert_eq!(
                offset, expected_offset,
                "Leap second offset mismatch for {}-{:02}-{:02}",
                year, month, day
            );
        }
    }

    // #[cfg(all(test, feature = "hifitime"))]
    // mod hifitime_bug_tests {
    //     use hifitime::Epoch;

    //     #[test]
    //     fn hifitime_self_roundtrip_large_negative() {
    //         let h = Epoch::from_gregorian_tai(-1000, 1, 1, 12, 0, 0, 0);
    //         let original_ns = h.to_tai_duration().total_nanoseconds();

    //         // Try the main path
    //         let dur = hifitime::Duration::from_total_nanoseconds(original_ns);
    //         let (centuries, nanos) = dur.to_parts();
    //         let h2 = Epoch::from_tai_parts(centuries, nanos);
    //         let roundtrip_ns = h2.to_tai_duration().total_nanoseconds();

    //         assert_eq!(
    //             original_ns, roundtrip_ns,
    //             "hifitime Duration roundtrip failed for large negative value"
    //         );
    //     }

    //     #[test]
    //     fn hifitime_from_tai_seconds_also_broken() {
    //         let h = Epoch::from_gregorian_tai(-1000, 1, 1, 12, 0, 0, 0);
    //         let original_ns = h.to_tai_duration().total_nanoseconds();
    //         let seconds_f64 = original_ns as f64 / 1_000_000_000.0;

    //         let h2 = Epoch::from_tai_seconds(seconds_f64);
    //         let roundtrip_ns = h2.to_tai_duration().total_nanoseconds();

    //         assert_eq!(
    //             original_ns, roundtrip_ns,
    //             "hifitime from_tai_seconds also fails for large negative value"
    //         );
    //     }
    // }

    #[test]
    fn roundtrip_j2000() {
        let tp = TimePoint::ZERO;
        let h = tp.to_hifitime();
        let tp2 = TimePoint::from_hifitime_epoch(h);
        assert_eq!(tp, tp2);
    }

    #[test]
    fn roundtrip_unix_epoch() {
        let tp = TimePoint::UNIX_EPOCH;
        let h = tp.to_hifitime();
        let tp2 = TimePoint::from_hifitime_epoch(h);
        assert_eq!(tp, tp2);
    }

    #[test]
    fn roundtrip_traditional_gps_epoch() {
        let tp = TimePoint::GPS_EPOCH.to_type(ClockType::TAI);
        let h = tp.to_hifitime();
        let tp2 = TimePoint::from_hifitime_epoch(h);
        assert_eq!(tp, tp2);
    }

    #[test]
    fn hifitime_different_scales() {
        let h_utc = Epoch::from_gregorian_utc(2024, 4, 26, 3, 28, 0, 0);
        let tp = TimePoint::from_hifitime_epoch(h_utc);
        let h_tai = tp.to_hifitime();
        assert_eq!(
            h_utc.to_tai_duration().total_nanoseconds(),
            h_tai.to_tai_duration().total_nanoseconds()
        );
    }

    #[test]
    fn large_positive_time() {
        let h = Epoch::from_gregorian_tai(3000, 1, 1, 12, 0, 0, 0);
        let tp = TimePoint::from_hifitime_epoch(h);
        let h2 = tp.to_hifitime();
        assert_eq!(
            h.to_tai_duration().total_nanoseconds(),
            h2.to_tai_duration().total_nanoseconds()
        );
    }

    #[test]
    fn leap_second_boundary() {
        let h = Epoch::from_gregorian_str("2016-12-31T23:59:60 UTC").unwrap();
        let tp = TimePoint::from_hifitime_epoch(h);
        let h2 = tp.to_hifitime();
        assert_eq!(
            h.to_tai_duration().total_nanoseconds(),
            h2.to_tai_duration().total_nanoseconds()
        );
    }

    #[test]
    fn sub_nanosecond_is_zero() {
        let h = Epoch::from_tai_duration(hifitime::Duration::from_total_nanoseconds(
            1_234_567_890_123_456_789i128,
        ));
        let tp = TimePoint::from_hifitime_epoch(h);
        assert_eq!(tp.subsec() % 1_000_000_000, 0);
    }

    // #[test]
    // fn large_negative_time() {
    //     let h = Epoch::from_gregorian_tai(-1000, 1, 1, 12, 0, 0, 0);
    //     let tp = TimePoint::from_hifitime(h);
    //     let h2 = tp.to_hifitime();
    //     assert_eq!(
    //         h.to_tai_duration().total_nanoseconds(),
    //         h2.to_tai_duration().total_nanoseconds()
    //     );
    // }
}
