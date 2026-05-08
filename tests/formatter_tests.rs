#[cfg(test)]
mod format_tests {
    use deep_time::{Dt, Scale, constants::STRFTIME_SIZE};

    #[test]
    fn test_leap_second_gotcha_2016_12_31() {
        // 2016-12-31 23:59:60 UTC — the last leap second in the current table
        // (TAI-UTC offset becomes 37 seconds at this instant)
        let leap = Dt::from_ymdhms(
            2016,
            12,
            31,
            23,
            59,
            60,
            123_456_789_000_000_000,
            Scale::UTC,
        );

        // === Gotcha 1: Civil time must show sec=60 (not roll over to next day) ===
        let g = leap.to_ymdhms();
        assert_eq!(g.yr, 2016);
        assert_eq!(g.mo, 12);
        assert_eq!(g.day, 31);
        assert_eq!(g.hr, 23);
        assert_eq!(g.min, 59);
        assert_eq!(
            g.sec, 60,
            "Leap second must be represented as 60, not rolled over"
        );
        assert_eq!(g.attos, 123_456_789_000_000_000);

        // === Gotcha 2: Round-trip must be exact ===
        let roundtrip = Dt::from_ymdhms(
            2016,
            12,
            31,
            23,
            59,
            60,
            123_456_789_000_000_000,
            Scale::UTC,
        );
        assert_eq!(leap, roundtrip);

        // === Gotcha 3: Formatting must output "60" correctly ===
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = leap
            .to_u8_with_offset("%Y-%m-%d %H:%M:%S.%f", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2016-12-31 23:59:60.123456789000000000");

        // === Gotcha 4: Trimmed fractional on leap second ===
        let n = leap
            .to_u8_with_offset("%Y-%m-%dT%H:%M:%S%.~fZ", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2016-12-31T23:59:60.123456789Z");

        // === Gotcha 5: leap second Unix timestamp (POSIX convention) ===
        let unix = leap.to_epoch(Dt::UNIX_EPOCH, Scale::UTC).to_sec();
        assert_eq!(unix, 1483228799); // same as 23:59:59 — the leap second "replays" the previous second
    }

    #[test]
    fn test_basic_formatting() {
        let t = Dt::from_ymdhms(2025, 4, 16, 14, 30, 45, 123_456_789_000_000_000, Scale::UTC);

        let mut buf = [0u8; STRFTIME_SIZE];

        // ISO date + time + fractional (now full attosecond precision)
        let n = t
            .to_u8_with_offset("%Y-%m-%d %H:%M:%S.%f", &mut buf, 0) // 0 = UTC
            .unwrap();
        assert_eq!(&buf[0..n], b"2025-04-16 14:30:45.123456789000000000");

        // Shortcuts
        let n = t.to_u8_with_offset("%F", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"2025-04-16");

        let n = t.to_u8_with_offset("%T", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"14:30:45");

        let n = t.to_u8_with_offset("%R", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"14:30");
    }

    #[test]
    fn test_fractional_seconds_fix() {
        let t = Dt::from_ymdhms(2025, 4, 16, 0, 0, 0, 123_456_789_000_000_000, Scale::UTC);

        let mut buf = [0u8; STRFTIME_SIZE];

        // %f and %N now default to 18 attosecond digits
        let n = t.to_u8_with_offset("%f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"123456789000000000");

        let n = t.to_u8_with_offset("%N", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"123456789000000000");

        // Custom width
        let n = t.to_u8_with_offset("%.3f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123");

        let n = t.to_u8_with_offset("%.6N", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"123456");
    }

    #[test]
    fn test_iso_week_fix() {
        let mut buf = [0u8; STRFTIME_SIZE];

        // 2000-01-01 was Saturday → belongs to 1999 week 52
        let t2000 = Dt::from_ymdhms(2000, 1, 1, 12, 0, 0, 0, Scale::UTC);
        let n = t2000.to_u8_with_offset("%G-W%V-%u", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"1999-W52-6");

        // 2000-01-03 is Monday of week 1 of 2000
        let t2000_monday = Dt::from_ymdhms(2000, 1, 3, 12, 0, 0, 0, Scale::UTC);
        let n = t2000_monday
            .to_u8_with_offset("%G-W%V-%u", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2000-W01-1");

        // Year with 53 weeks (2015-12-28 is Monday of week 53 of 2015)
        let t_week53 = Dt::from_ymdhms(2015, 12, 28, 12, 0, 0, 0, Scale::UTC);
        let n = t_week53.to_u8_with_offset("%G-W%V", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"2015-W53");
    }

    #[test]
    fn test_timezone_offset() {
        let t = Dt::from_ymdhms(2025, 4, 16, 14, 30, 45, 0, Scale::UTC);
        let mut buf = [0u8; STRFTIME_SIZE];

        // %z with different colon counts
        let n = t.to_u8_with_offset("%z", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"+0000");

        let n = t.to_u8_with_offset("%:z", &mut buf, -5 * 3600).unwrap();
        assert_eq!(&buf[0..n], b"-05:00");

        let n = t.to_u8_with_offset("%::z", &mut buf, -8 * 3600).unwrap();
        assert_eq!(&buf[0..n], b"-08:00:00");

        let n = t
            .to_u8_with_offset("%z", &mut buf, 2 * 3600 + 30 * 60)
            .unwrap();
        assert_eq!(&buf[0..n], b"+0230");

        // %Q
        let n = t.to_u8_with_offset("%Q", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"UTC");

        let n = t.to_u8_with_offset("%z", &mut buf, -5 * 3600).unwrap();
        assert_eq!(&buf[0..n], b"-0500");
    }

    #[test]
    fn test_padding_and_flags() {
        let t = Dt::from_ymdhms(2025, 4, 5, 3, 9, 7, 0, Scale::UTC);
        let mut buf = [0u8; STRFTIME_SIZE];

        // Default zero padding
        let n = t.to_u8_with_offset("%d %H %M %S", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"05 03 09 07");

        // Space padding
        let n = t.to_u8_with_offset("%_d %_H", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b" 5  3");

        // No padding
        let n = t.to_u8_with_offset("%-d %-H", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"5 3");

        // Zero padding explicit
        let n = t.to_u8_with_offset("%0d %0H", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"05 03");
    }

    #[test]
    fn test_weekday_and_month_names() {
        let t = Dt::from_ymdhms(2025, 4, 16, 0, 0, 0, 0, Scale::UTC); // Wednesday
        let mut buf = [0u8; STRFTIME_SIZE];

        let n = t.to_u8_with_offset("%A, %B %d, %Y", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"Wednesday, April 16, 2025");

        let n = t.to_u8_with_offset("%a %b %d", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"Wed Apr 16");
    }

    #[test]
    fn test_unix_timestamp_and_day_of_year() {
        let t = Dt::from_ymdhms(1970, 1, 1, 0, 0, 0, 0, Scale::UTC); // Unix epoch
        let mut buf = [0u8; STRFTIME_SIZE];

        let n = t.to_u8_with_offset("%s", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"0");

        let n = t.to_u8_with_offset("%j", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"001");
    }

    #[test]
    fn test_edge_cases_roundtrip_and_extreme_values() {
        let mut buf = [0u8; STRFTIME_SIZE];

        // ── Negative & zero years ─────────────────────────────────────
        let t_neg = Dt::from_ymdhms(-123, 6, 15, 9, 30, 45, 0, Scale::UTC);
        let n = t_neg.to_u8_with_offset("%Y-%m-%d", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"-0123-06-15");

        let n = t_neg.to_u8_with_offset("%C", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"-2"); // century

        let t_zero = Dt::from_ymdhms(0, 1, 1, 0, 0, 0, 0, Scale::UTC);
        let n = t_zero.to_u8_with_offset("%Y", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"0000");

        // ── ISO week year-boundary cases (now fixed correctly) ───────
        // 2024-12-30 (Mon) → belongs to 2025 week 1
        let t_2024_dec30 = Dt::from_ymdhms(2024, 12, 30, 12, 0, 0, 0, Scale::UTC);
        let n = t_2024_dec30
            .to_u8_with_offset("%G-W%V-%u", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2025-W01-1");

        // 2024-12-31 (Tue) → still 2025-W01-2
        let t_2024_dec31 = Dt::from_ymdhms(2024, 12, 31, 12, 0, 0, 0, Scale::UTC);
        let n = t_2024_dec31
            .to_u8_with_offset("%G-W%V-%u", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2025-W01-2");

        // 2025-01-01 (Wed) → 2025-W01-3
        let t_2025_jan1 = Dt::from_ymdhms(2025, 1, 1, 12, 0, 0, 0, Scale::UTC);
        let n = t_2025_jan1
            .to_u8_with_offset("%G-W%V-%u", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2025-W01-3");

        // Year with 53 weeks
        let t_2015_dec28 = Dt::from_ymdhms(2015, 12, 28, 12, 0, 0, 0, Scale::UTC);
        let n = t_2015_dec28
            .to_u8_with_offset("%G-W%V", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2015-W53");

        // ── Week numbers %U / %W edge cases ───────────────────────────
        // 2000-01-01 was Saturday → %U = 0 (first Sunday is Jan 2)
        let t2000 = Dt::from_ymdhms(2000, 1, 1, 12, 0, 0, 0, Scale::UTC);
        let n = t2000.to_u8_with_offset("%U", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"00");

        let n = t2000.to_u8_with_offset("%W", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"00");

        // 2023-12-31 was Sunday → %U = 53
        let t_sun = Dt::from_ymdhms(2023, 12, 31, 12, 0, 0, 0, Scale::UTC);
        let n = t_sun.to_u8_with_offset("%U", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"53");

        // ── Fractional seconds extremes ───────────────────────────────
        let t_frac = Dt::from_ymdhms(2025, 4, 16, 0, 0, 0, 0, Scale::UTC);
        let n = t_frac.to_u8_with_offset("%.0f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b""); // width 0 = nothing

        let n = t_frac.to_u8_with_offset("%.9N", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"000000000");

        let n = t_frac.to_u8_with_offset("%S.%f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"00.000000000000000000");

        // ── Timezone offsets with seconds & different colon counts ─────
        let ny = -5 * 3600;
        let la = -8 * 3600;
        let weird = 1 * 3600 + 23 * 60 + 45;

        let n = t_frac.to_u8_with_offset("%::z", &mut buf, ny).unwrap();
        assert_eq!(&buf[0..n], b"-05:00:00");

        let n = t_frac.to_u8_with_offset("%:z", &mut buf, la).unwrap();
        assert_eq!(&buf[0..n], b"-08:00");

        // %::z with seconds component (tests full +HH:MM:SS support)
        let n = t_frac.to_u8_with_offset("%::z", &mut buf, weird).unwrap();
        assert_eq!(&buf[0..n], b"+01:23:45");

        // ── Padding + explicit width + flags combined ─────────────────
        let t_small = Dt::from_ymdhms(2025, 4, 5, 3, 9, 7, 0, Scale::UTC);

        let n = t_small.to_u8_with_offset("%03d", &mut buf, 0).unwrap(); // explicit width 3, default zero
        assert_eq!(&buf[0..n], b"005");

        let n = t_small.to_u8_with_offset("%-5H", &mut buf, 0).unwrap(); // left-justify, width 5 → no pad
        assert_eq!(&buf[0..n], b"3");

        // space-pad to width 3 (correct behavior for flag '_')
        let n = t_small.to_u8_with_offset("%_3M", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"  9"); // two spaces + '9'

        // ── Negative Unix timestamp ───────────────────────────────────
        let t_neg_unix = Dt::from_ymdhms(1969, 12, 31, 23, 59, 59, 0, Scale::UTC);
        let n = t_neg_unix.to_u8_with_offset("%s", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"-1");

        // Large positive (well within i64 for %s)
        let t_large = Dt::from_ymdhms(2038, 1, 19, 3, 14, 7, 0, Scale::UTC);
        let n = t_large.to_u8_with_offset("%s", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"2147483647");
    }

    #[test]
    fn test_fractional_trim_flag() {
        let mut buf = [0u8; STRFTIME_SIZE];

        // Value with trailing zeros in fractional part
        let t = Dt::from_ymdhms(2025, 4, 16, 0, 0, 0, 123_456_789_000_000_000, Scale::UTC);

        // %.~f should trim all trailing zeros
        let n = t.to_u8_with_offset("%.~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123456789");

        // %.9~f should trim to 9 significant digits
        let n = t.to_u8_with_offset("%.9~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123456789");

        // %.18~f trims trailing zeros (this is the intended behavior of ~)
        let n = t.to_u8_with_offset("%.18~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123456789"); // trimmed to significant digits

        // Value that becomes all zeros after trimming
        let t_zero = Dt::from_ymdhms(2025, 4, 16, 0, 0, 0, 0, Scale::UTC);
        let n = t_zero.to_u8_with_offset("%.~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b""); // no dot, no "0"

        let n = t_zero.to_u8_with_offset("%.9~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b""); // still nothing

        // Without ~ it should NOT trim (keeps trailing zeros)
        let t_trailing = Dt::from_ymdhms(2025, 4, 16, 0, 0, 0, 123_000_000_000_000_000, Scale::UTC);
        let n = t_trailing.to_u8_with_offset("%.9f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123000000"); // keeps trailing zeros without ~

        let n = t_trailing.to_u8_with_offset("%.9~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123"); // trims with ~

        // %.0~f should always be empty
        let n = t.to_u8_with_offset("%.0~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"");

        // ── Negative years + fractional trim ─────────────────────────────
        let t_neg = Dt::from_ymdhms(-123, 6, 15, 9, 30, 45, 123_456_789_000_000_000, Scale::UTC);
        let n = t_neg
            .to_u8_with_offset("%Y-%m-%dT%H:%M:%S%.~fZ", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"-0123-06-15T09:30:45.123456789Z");

        // Negative year with all-zero fractional after trim
        let t_neg_zero = Dt::from_ymdhms(-1, 1, 1, 0, 0, 0, 0, Scale::UTC);
        let n = t_neg_zero
            .to_u8_with_offset("%Y-%.~f", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"-0001-"); // no fractional part at all

        // Year 0 with fractional
        let t_year0 = Dt::from_ymdhms(0, 1, 1, 0, 0, 0, 500_000_000_000_000_000, Scale::UTC);
        let n = t_year0.to_u8_with_offset("%Y%.~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"0000.5");

        // ── Long years (6 digits) + fractional ───────────────────────────
        let t_long_year = Dt::from_ymdhms(123456, 7, 4, 12, 0, 0, 987654321987654321, Scale::UTC);
        let n = t_long_year
            .to_u8_with_offset("%Y-%m-%dT%H:%M:%S%.~fZ", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"123456-07-04T12:00:00.987654321987654321Z");

        let t_long_neg_year =
            Dt::from_ymdhms(-100000, 12, 31, 23, 59, 59, 111111111111111111, Scale::UTC);
        let n = t_long_neg_year
            .to_u8_with_offset("%Y-%.~f", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"-100000-.111111111111111111");

        // ── 18-digit attos with NO trailing zeros (with and without ~) ───
        let t_full_attos = Dt::from_ymdhms(2025, 4, 16, 0, 0, 0, 123456789012345678, Scale::UTC);

        // With ~ (should still output all 18 digits since no trailing zeros)
        let n = t_full_attos
            .to_u8_with_offset("%.18~f", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b".123456789012345678");

        // Without ~ (same result)
        let n = t_full_attos
            .to_u8_with_offset("%.18f", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b".123456789012345678");
    }
}
