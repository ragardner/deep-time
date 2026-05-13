#[cfg(feature = "hifitime")]
mod tests {
    use deep_time::{Dt, Scale, leap_seconds};
    use hifitime::{Duration, Epoch, TimeScale};
    /// Seconds between hifitime's TAI reference epoch (1900-01-01 00:00:00 TAI)
    /// and our library's J2000.0 TAI.
    const HIFITIME_TAI_EPOCH_TO_OUR_J2000: i64 = 3_155_716_800;

    /// Returns hifitime's TAI representation as `(sec, attos)` using integer nanoseconds.
    /// hifitime is only ns-precise, so we truncate our attos to ns for comparison.
    fn hifitime_tai_parts(hi: Epoch) -> (i64, u64) {
        let tai = hi.to_time_scale(TimeScale::TAI);
        let ref_tai = Epoch::from_tai_seconds(HIFITIME_TAI_EPOCH_TO_OUR_J2000 as f64);
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

    fn assert_tai_matches_hifitime(our_tai: Dt, hi: Epoch, context: &str) {
        let (our_sec, our_attos) = (our_tai.sec(), our_tai.attos());
        let (hi_sec, hi_attos) = hifitime_tai_parts(hi);

        assert_eq!(our_sec, hi_sec, "{} — TAI seconds differ", context);
        // Truncate to ns precision (hifitime only guarantees ns)
        assert_eq!(
            our_attos / 1_000_000_000 * 1_000_000_000,
            hi_attos,
            "{} — TAI subseconds differ (ns precision)",
            context
        );
    }

    #[test]
    fn tt_matches_hifitime_latest() {
        use hifitime::{Epoch, TimeScale};

        // Same test instant as your TCL test (2038-01-01 00:00 TAI)
        let tai_sec: f64 = 4_354_905_600.0;

        let epoch_tai = Epoch::from_tai_seconds(tai_sec);
        let epoch_tt = epoch_tai.to_time_scale(TimeScale::TT);
        let hifi_tt_sec = epoch_tt.duration.to_seconds();

        let my_2038_tai = Dt::from_ymd_on(2038, 1, 1, Scale::TAI);

        // Our library gives seconds since J2000.0 TT.
        // hifitime gives seconds since 1900-01-01 TT.
        // Add the known constant offset so both numbers count from the same point.
        let my_tt_sec = my_2038_tai.to(Scale::TAI, Scale::TT).to_sec_f()
            + (HIFITIME_TAI_EPOCH_TO_OUR_J2000 as f64);

        let diff = (my_tt_sec - hifi_tt_sec).abs();

        assert!(
            diff < 1e-12,
            "TT mismatch with hifitime: our (adjusted) = {:.12}, hifitime = {:.12}, diff = {:.12} s",
            my_tt_sec,
            hifi_tt_sec,
            diff
        );
    }

    #[test]
    fn tdb_matches_hifitime_latest() {
        use hifitime::{Epoch, TimeScale};

        // Same test instant as your TCL and TT tests (2038-01-01 00:00 TAI)
        let tai_sec: f64 = 4_354_905_600.0;

        let epoch_tai = Epoch::from_tai_seconds(tai_sec);
        let epoch_tdb = epoch_tai.to_time_scale(TimeScale::TDB);
        let hifi_tdb_sec = epoch_tdb.duration.to_seconds();

        let my_2038_tai = Dt::from_ymd_on(2038, 1, 1, Scale::TAI);

        // Both libraries give TDB seconds since J2000.0 → direct comparison, no offset
        let my_tdb_sec = my_2038_tai.to(Scale::TAI, Scale::TDB).to_sec_f();

        let diff = (my_tdb_sec - hifi_tdb_sec).abs();

        assert!(
            diff < 1e-4,
            "TDB mismatch with hifitime: our = {:.9}, hifitime = {:.9}, diff = {:.9} s",
            my_tdb_sec,
            hifi_tdb_sec,
            diff
        );
    }

    #[test]
    fn utc_matches_hifitime_latest() {
        use hifitime::{Epoch, TimeScale};

        // Same test instant as your TCL / TT / TDB tests (2038-01-01 00:00 TAI)
        let tai_sec: f64 = 4_354_905_600.0;

        let epoch_tai = Epoch::from_tai_seconds(tai_sec);
        let epoch_utc = epoch_tai.to_time_scale(TimeScale::UTC);
        let hifi_utc_sec = epoch_utc.duration.to_seconds();

        let my_2038_tai = Dt::from_ymd_on(2038, 1, 1, Scale::TAI);

        // Our library gives seconds since J2000.0 UTC.
        // hifitime gives seconds since 1900-01-01 00:00 UTC.
        // Add the known constant offset so both numbers count from the same point.
        let my_utc_sec = my_2038_tai.to(Scale::TAI, Scale::UTC).to_sec_f()
            + (HIFITIME_TAI_EPOCH_TO_OUR_J2000 as f64);

        let diff = (my_utc_sec - hifi_utc_sec).abs();

        assert!(
            diff < 1e-12,
            "UTC mismatch with hifitime: our (adjusted) = {:.12}, hifitime = {:.12}, diff = {:.12} s",
            my_utc_sec,
            hifi_utc_sec,
            diff
        );
    }

    #[test]
    fn gps_matches_hifitime_latest() {
        use hifitime::{Epoch, TimeScale};

        // Same test instant as your TCL / TT / TDB / UTC tests (2038-01-01 00:00 TAI)
        let tai_sec: f64 = 4_354_905_600.0;

        let epoch_tai = Epoch::from_tai_seconds(tai_sec);
        let epoch_gps = epoch_tai.to_time_scale(TimeScale::GPST);
        let hifi_gps_sec = epoch_gps.duration.to_seconds();

        let my_2038_tai = Dt::from_ymd_on(2038, 1, 1, Scale::TAI);

        // hifitime GPST: seconds since 1980-01-06 00:00:00 UTC (standard GPS epoch)
        // Our library: seconds since J2000.0 on the GPS scale
        // Offset between GPS epoch and J2000.0 = exactly 630763200 s
        let my_gps_sec = my_2038_tai.to(Scale::TAI, Scale::GPS).to_sec_f() + 630_763_200.0;

        let diff = (my_gps_sec - hifi_gps_sec).abs();

        assert!(
            diff < 1e-12,
            "GPS mismatch with hifitime: our (adjusted) = {:.12}, hifitime = {:.12}, diff = {:.12} s",
            my_gps_sec,
            hifi_gps_sec,
            diff
        );
    }

    #[test]
    fn bdt_matches_hifitime_latest() {
        use hifitime::{Epoch, TimeScale};

        // Same test instant as all your other cross-validation tests (2038-01-01 00:00 TAI)
        let tai_sec: f64 = 4_354_905_600.0;

        let epoch_tai = Epoch::from_tai_seconds(tai_sec);
        let epoch_bdt = epoch_tai.to_time_scale(TimeScale::BDT);
        let hifi_bdt_sec = epoch_bdt.duration.to_seconds();

        let my_2038_tai = Dt::from_ymd_on(2038, 1, 1, Scale::TAI);

        // hifitime BDT: seconds since 2006-01-01 00:00:00 UTC
        // Our library: seconds since J2000.0 on the BDT scale
        // → subtract the offset (BDT reference is later than J2000)
        let my_bdt_sec = my_2038_tai.to(Scale::TAI, Scale::BDT).to_sec_f() - 189_345_600.0;

        let diff = (my_bdt_sec - hifi_bdt_sec).abs();

        assert!(
            diff < 1e-12,
            "BDT mismatch with hifitime: our (adjusted) = {:.12}, hifitime = {:.12}, diff = {:.12} s",
            my_bdt_sec,
            hifi_bdt_sec,
            diff
        );
    }

    #[test]
    fn test_utc_leap_second() {
        let hi_utc = Epoch::from_gregorian(2016, 12, 31, 23, 59, 60, 0, TimeScale::UTC);
        let hi_tai = hi_utc.to_time_scale(TimeScale::TAI);
        let (hi_tai_sec, hi_tai_subsec) = hifitime_tai_parts(hi_tai);
        let our_tai = Dt::new(hi_tai_sec, hi_tai_subsec);
        let our_utc = our_tai.to(Scale::TAI, Scale::UTC).to_tai(Scale::UTC);

        assert_tai_matches_hifitime(our_utc, hi_tai, "UTC leap second 2016-12-31");
    }

    #[test]
    fn test_j2000_zero_points() {
        let our = Dt::from(0, 0, Scale::TAI);
        let hi = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);
        assert_tai_matches_hifitime(our, hi, "J2000 TAI zero");

        let our = Dt::from(0, 0, Scale::TT);
        let hi = Epoch::from_gregorian_tai(2000, 1, 1, 11, 59, 27, 816_000_000);
        assert_tai_matches_hifitime(our, hi, "J2000 TT zero");

        let our = Dt::from(0, 0, Scale::GPS);
        let hi = Epoch::from_gregorian(2000, 1, 1, 12, 0, 0, 0, TimeScale::GPST);
        assert_tai_matches_hifitime(our, hi, "J2000 GPST zero");

        let our = Dt::from(0, 0, Scale::BDT);
        let hi = Epoch::from_gregorian(2000, 1, 1, 12, 0, 0, 0, TimeScale::BDT);
        assert_tai_matches_hifitime(our, hi, "J2000 BDT zero");
    }

    #[test]
    fn test_negative_and_subsecond() {
        // Use a smaller negative value that hifitime handles cleanly
        let our = Dt::from(-1_000_000_000i64, 123_456_789_012_345_678, Scale::GPS);

        let delta = Duration::from_seconds(-1_000_000_000f64)
            + Duration::from_nanoseconds(123_456_789_012_345_678u64 as f64 / 1_000_000_000.0);
        let gpst_zero = Epoch::from_gregorian(2000, 1, 1, 12, 0, 0, 0, TimeScale::GPST);
        let hi = gpst_zero + delta;

        assert_tai_matches_hifitime(our, hi, "negative GPST with sub-second");
    }

    #[test]
    fn test_all_leap_second_epochs() {
        let cases: &[(i64, &str)] = &[
            (489_024_000, "after 1998-12-31 leap"),
            (536_544_000, "after 2016-12-31 leap"),
            (599_616_000, "2019-01-01 (no leap, but near epoch)"),
        ];

        for &(tai_sec, label) in cases {
            let our = Dt::new(tai_sec, 0);
            let hi = Epoch::from_tai_seconds((HIFITIME_TAI_EPOCH_TO_OUR_J2000 + tai_sec) as f64);
            assert_tai_matches_hifitime(our, hi, label);
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

            let my_utc = Dt::from_ymd(year as i64, month, day);
            let my_offset = leap_seconds::get_leap_seconds(&my_utc, true).offset as i32;

            let offset = utc
                .leap_seconds(true) // IERS-only (post-1972)
                .expect("Leap second date should be after 1960")
                .round() as i32;

            assert_eq!(
                offset, expected_offset,
                "hifitime leap second offset mismatch for {}-{:02}-{:02}",
                year, month, day
            );

            assert_eq!(
                my_offset, expected_offset,
                "deep_time leap second offset mismatch for {}-{:02}-{:02}",
                year, month, day
            );

            assert_eq!(
                offset, my_offset,
                "hifitime and deep_time disagree on leap second offset for {}-{:02}-{:02}",
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
        let tp = Dt::ZERO;
        let h = tp.to_hifitime(Scale::TAI);
        let tp2 = Dt::from_hifitime_epoch(h);
        assert_eq!(tp, tp2);
    }

    #[test]
    fn roundtrip_unix_epoch() {
        let tp = Dt::UNIX_EPOCH;
        let h = tp.to_hifitime(Scale::TAI);
        let tp2 = Dt::from_hifitime_epoch(h);
        assert_eq!(tp, tp2);
    }

    #[test]
    fn roundtrip_traditional_gps_epoch() {
        let tp = Dt::GPS_EPOCH;
        let h = tp.to_hifitime(Scale::TAI);
        let tp2 = Dt::from_hifitime_epoch(h);
        assert_eq!(tp, tp2);
    }

    #[test]
    fn hifitime_different_scales() {
        let h_utc = Epoch::from_gregorian_utc(2024, 4, 26, 3, 28, 0, 0);
        let tp = Dt::from_hifitime_epoch(h_utc);
        let h_tai = tp.to_hifitime(Scale::TAI);
        assert_eq!(
            h_utc.to_tai_duration().total_nanoseconds(),
            h_tai.to_tai_duration().total_nanoseconds()
        );
    }

    #[test]
    fn large_positive_time() {
        let h = Epoch::from_gregorian_tai(3000, 1, 1, 12, 0, 0, 0);
        let tp = Dt::from_hifitime_epoch(h);
        let h2 = tp.to_hifitime(Scale::TAI);
        assert_eq!(
            h.to_tai_duration().total_nanoseconds(),
            h2.to_tai_duration().total_nanoseconds()
        );
    }

    #[test]
    fn leap_second_boundary() {
        let h = Epoch::from_gregorian_str("2016-12-31T23:59:60 UTC").unwrap();
        let tp = Dt::from_hifitime_epoch(h);
        let h2 = tp.to_hifitime(Scale::TAI);
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
        let tp = Dt::from_hifitime_epoch(h);
        assert_eq!(tp.attos() % 1_000_000_000, 0);
    }

    // #[test]
    // fn large_negative_time() {
    //     let h = Epoch::from_gregorian_tai(-1000, 1, 1, 12, 0, 0, 0);
    //     let tp = Dt::from_hifitime(h);
    //     let h2 = tp.to_hifitime();
    //     assert_eq!(
    //         h.to_tai_duration().total_nanoseconds(),
    //         h2.to_tai_duration().total_nanoseconds()
    //     );
    // }
}
