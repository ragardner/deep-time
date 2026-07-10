#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(feature = "alloc", feature = "physics", feature = "wire"))]
mod tests {
    use alloc::vec::Vec;
    use core::fmt::Debug;
    use deep_time::{
        Drift, Dt, Scale, TimeRange, YmdHms,
        civil_parts::{Meridiem, Offset, Parts, Weekday},
        from_sec,
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

    fn assert_dt_wire_roundtrip(original: &Dt) {
        let bytes = original.to_wire_bytes();
        let recovered = Dt::from_wire_bytes(&bytes).expect("deserialization failed");
        assert_eq!(original.attos, recovered.attos, "attos mismatch");
        assert_eq!(original.scale, recovered.scale, "scale mismatch");
        assert_eq!(original.target, recovered.target, "target mismatch");
    }

    #[test]
    fn test_dt_roundtrip() {
        let span = Dt::from_sec(123456789, Scale::TAI, Scale::TAI)
            + Dt::from_ns(987654321, 0, Scale::TAI, Scale::TAI);
        assert_dt_wire_roundtrip(&span);
    }

    #[test]
    fn test_dt_wire_preserves_scale_and_target() {
        let dt = Dt::from_ymd(2024, 6, 15, Scale::UTC, 12, 0, 0, 0)
            .with(Scale::TT)
            .target(Scale::GPS);
        assert_dt_wire_roundtrip(&dt);
        assert_eq!(dt.scale, Scale::TT);
        assert_eq!(dt.target, Scale::GPS);
    }

    #[test]
    fn test_clockdrift_roundtrip() {
        let drift = Drift::new(
            Dt::from_sec(5, Scale::TAI, Scale::TAI),
            Dt::from_ns(1, 0, Scale::TAI, Scale::TAI),
            deep_time::dt!(2),
        );
        assert_roundtrip(
            &drift,
            |d| d.to_wire_bytes().to_vec(),
            Drift::from_wire_bytes,
        );
    }

    #[test]
    fn test_timerange_roundtrip() {
        let start = from_sec!(1000000000);
        let end = start + Dt::from_hours(24, 0, Scale::TAI, Scale::TAI);
        let step = Dt::from_hours(1, 0, Scale::TAI, Scale::TAI);
        let range = start.range(end, step);

        assert_roundtrip(
            &range,
            |r| r.to_wire_bytes().to_vec(),
            TimeRange::from_wire_bytes,
        );
    }

    #[test]
    fn test_civil_parts_roundtrip() {
        let mut dc = Parts::default();
        dc.yr = Some(2025);
        dc.mo = Some(6);
        dc.day = Some(15);
        dc.hr = 14;
        dc.min = 30;
        dc.sec = 0;
        dc.attos = 0;
        dc.scale = Scale::TAI;
        dc.offset = Some(Offset::Fixed(3600));

        assert_roundtrip(&dc, |d| d.to_wire_bytes().to_vec(), Parts::from_wire_bytes);
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
        let offsets = [Offset::None, Offset::Fixed(3600)];
        for offset in offsets {
            assert_roundtrip(
                &offset,
                |t| t.to_wire_bytes().to_vec(),
                Offset::from_wire_bytes,
            );
        }
    }
}
