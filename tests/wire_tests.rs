#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "wire")]
mod tests {
    use alloc::vec::Vec;
    use core::fmt::Debug;
    use deep_time::{
        Drift, Dt, Meridiem, Offset, Scale, TimeParts, TimeRange, Weekday, YmdHmsRich,
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
        let tp = Dt::new(
            Dt::sec_to_attos(9876543210) + 123456789012345678,
            Scale::TAI,
        );
        assert_roundtrip(&tp, |t| t.to_wire_bytes().to_vec(), Dt::from_wire_bytes);
    }

    #[test]
    fn test_clockdrift_roundtrip() {
        let drift = Drift::new(
            Dt::from_sec(5, Scale::TAI),
            Dt::from_ns(1, Scale::TAI),
            Dt::span(2),
        );
        assert_roundtrip(
            &drift,
            |d| d.to_wire_bytes().to_vec(),
            Drift::from_wire_bytes,
        );
    }

    #[test]
    fn test_timerange_roundtrip() {
        let start = Dt::from_tai_sec(1000000000);
        let end = start + Dt::from_hr(24, Scale::TAI);
        let step = Dt::from_hr(1, Scale::TAI);
        let range = start.range(end, step);

        assert_roundtrip(
            &range,
            |r| r.to_wire_bytes().to_vec(),
            TimeRange::from_wire_bytes,
        );
    }

    #[test]
    fn test_date_time_roundtrip() {
        let dt = Dt::from_ymd(2024, 12, 25, 12, 0, 0, 123456789012345678, Scale::UTC);
        let gp = dt.to_ymd_rich();

        assert_roundtrip(
            &gp,
            |g| g.to_wire_bytes().to_vec(),
            YmdHmsRich::from_wire_bytes,
        );
    }

    #[test]
    fn test_time_parts_roundtrip() {
        let mut dc = TimeParts::default();
        dc.yr = Some(2025);
        dc.mo = Some(6);
        dc.day = Some(15);
        dc.hr = Some(14);
        dc.min = Some(30);
        dc.sec = Some(0);
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
