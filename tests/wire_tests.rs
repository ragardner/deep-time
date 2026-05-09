#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "wire")]
mod tests {
    use alloc::vec::Vec;
    use core::fmt::Debug;
    use deep_time::{
        ClockDrift, ClockModel, Dt, GregorianTime, Meridiem, Offset, Scale, TimeParts, TimeRange,
        Weekday,
    };

    /// Helper function to test round-trip serialization/deserialization.

    fn assert_roundtrip<T>(
        original: &T,
        to_bytes: impl Fn(&T) -> Vec<u8>,
        from_bytes: impl Fn(&[u8]) -> Option<T>,
    ) where
        T: PartialEq + Debug,
    {
        let bytes = to_bytes(original);
        let recovered = from_bytes(&bytes).expect("deserialization failed");
        assert_eq!(original, &recovered, "Round-trip failed");
    }

    #[test]
    fn test_dt_roundtrip() {
        let span = Dt::from_sec(123456789, Scale::TAI) + Dt::from_ns(987654321, Scale::TAI);
        assert_roundtrip(&span, |d| d.to_wire_bytes().to_vec(), Dt::from_wire_bytes);
    }

    #[test]
    fn test_timepoint_roundtrip() {
        let tp = Dt::new(9876543210, 123456789012345678);
        assert_roundtrip(&tp, |t| t.to_wire_bytes().to_vec(), Dt::from_wire_bytes);
    }

    #[test]
    fn test_clockdrift_roundtrip() {
        let drift = ClockDrift::new(
            Dt::from_sec(5, Scale::TAI),
            Dt::from_ns(1, Scale::TAI),
            Dt::from_attos(2, Scale::TAI),
        );
        assert_roundtrip(
            &drift,
            |d| d.to_wire_bytes().to_vec(),
            ClockDrift::from_wire_bytes,
        );
    }

    #[test]
    fn test_clockmodel_roundtrip() {
        let model = ClockModel::new(
            Scale::Custom,
            Dt::new(0, 0),
            ClockDrift::from_offset_and_rate(
                Dt::from_sec(42, Scale::TAI),
                Dt::from_ns(1, Scale::TAI),
            ),
        );
        assert_roundtrip(
            &model,
            |m| m.to_wire_bytes().to_vec(),
            ClockModel::from_wire_bytes,
        );
    }

    #[test]
    fn test_timerange_roundtrip() {
        let start = Dt::new(1000000000, 0);
        let end = start + Dt::from_hr(24, Scale::TAI);
        let step = Dt::from_hr(1, Scale::TAI);
        let range = start.range_to(end, step);

        assert_roundtrip(
            &range,
            |r| r.to_wire_bytes().to_vec(),
            TimeRange::from_wire_bytes,
        );
    }

    #[test]
    fn test_gregorian_time_roundtrip() {
        let gp = GregorianTime::new(
            1_700_000_000_000_000_000_000_000_000, // unix_attosec
            2024,                                  // yr
            12,                                    // mo
            25,                                    // day
            12,                                    // hr
            0,                                     // min
            0,                                     // sec
            123456789012345678,                    // attos
            2024,                                  // iso_yr
            52,                                    // iso_wk
            Weekday::Wednesday,                    // iso_wkday
            360,                                   // day_of_yr
            3,                                     // wkday
            51,                                    // wk_of_yr_sun
            52,                                    // wk_of_yr_mon
        );

        assert_roundtrip(
            &gp,
            |g| g.to_wire_bytes().to_vec(),
            GregorianTime::from_wire_bytes,
        );
    }

    #[test]
    fn test_time_parts_roundtrip() {
        let mut dc = TimeParts::default();
        dc.year = Some(2025);
        dc.month = Some(6);
        dc.day = Some(15);
        dc.hour = Some(14);
        dc.minute = Some(30);
        dc.second = Some(0);
        dc.attos = Some(0);
        dc.scale = Scale::TAI;
        dc.offset = Some(Offset::Utc);

        assert_roundtrip(
            &dc,
            |d| d.to_wire_bytes().to_vec(),
            TimeParts::from_wire_bytes,
        );
    }

    #[test]
    fn test_small_enums_roundtrip() {
        // Meridiem
        assert_roundtrip(
            &Meridiem::AM,
            |m| vec![m.to_wire_byte()],
            |b| Meridiem::from_wire_byte(b[0]),
        );
        assert_roundtrip(
            &Meridiem::PM,
            |m| vec![m.to_wire_byte()],
            |b| Meridiem::from_wire_byte(b[0]),
        );

        // Weekday
        for wd in [Weekday::Sunday, Weekday::Wednesday, Weekday::Saturday] {
            assert_roundtrip(
                &wd,
                |w| vec![w.to_wire_byte()],
                |b| Weekday::from_wire_byte(b[0]),
            );
        }

        // Offset
        let offsets = [Offset::Utc, Offset::None, Offset::Fixed(3600)];
        for offset in offsets {
            assert_roundtrip(
                &offset,
                |t| t.to_wire_bytes().to_vec(),
                Offset::from_wire_bytes,
            );
        }
    }
}
