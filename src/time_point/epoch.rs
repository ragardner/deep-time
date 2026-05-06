use crate::{ClockType, TimePoint, TimeSpan};

impl TimePoint {
    #[inline]
    pub const fn to_tai_attos_since(self, reference: TimePoint) -> i128 {
        self.to_tai_since(reference).to_attos()
    }

    #[inline]
    pub const fn from_tai_attos_since(attos: i128, reference: TimePoint) -> Self {
        reference.add(TimeSpan::from_attos(attos))
    }

    #[inline]
    pub const fn to_epoch(self, epoch: TimePoint, clock_type: ClockType) -> TimeSpan {
        /*
        do not apply an offset using to() to the EPOCH because the offset is for TAI,
        the to() function assumes the epoch is TAI, the UTCSofa instant for 1970 is
        the same as the UTC instant UNIX_EPOCH should remain UTC and the offset should
        not be applied to the epoch
        */
        self.to_type(clock_type).to().to_diff_tp(epoch)
    }

    #[inline]
    pub const fn from_epoch(offset: TimeSpan, epoch: TimePoint, clock_type: ClockType) -> Self {
        let total = epoch.to_span().add(offset);
        TimePoint::from(total.sec, total.subsec, clock_type)
    }
}

// TODO
// #[test]
// fn check_epoch_zeros() {
//     assert_eq!(
//         TimePoint::GPS_EPOCH.to_epoch_attos(ClockType::GPS),
//         0,
//         "GPS_EPOCH should be zero when measured from itself on GPS scale"
//     );

//     assert_eq!(
//         TimePoint::GALILEO_EPOCH.to_epoch_attos(ClockType::GST),
//         0,
//         "GALILEO_EPOCH should be zero on GST scale"
//     );

//     assert_eq!(
//         TimePoint::BDT_EPOCH.to_epoch_attos(ClockType::BDT),
//         0,
//         "BDT_EPOCH should be zero on BDT scale"
//     );

//     assert_eq!(
//         TimePoint::UNIX_EPOCH.to_epoch_attos(ClockType::UTC),
//         0,
//         "UNIX_EPOCH should be zero on UTC scale"
//     );
// }

// #[test]
// fn to_epoch_attos_basic_cases() {
//     // A normal date well after the last leap second
//     let t = TimePoint::from_ymdhms(2023, 6, 15, 12, 0, 0, 0, ClockType::UTC);
//     let unix_attos = t.to_epoch_attos(ClockType::UTC);
//     assert!(unix_attos > 1_600_000_000_000_000_000);

//     // Sub-second precision is preserved
//     let t2 = TimePoint::from_ymdhms(
//         2023,
//         6,
//         15,
//         12,
//         0,
//         0,
//         123_456_789_000_000_000,
//         ClockType::UTC,
//     );
//     let attos2 = t2.to_epoch_attos(ClockType::UTC);
//     assert_eq!(attos2 % ATTOS_PER_SEC_I128, 123_456_789_000_000_000);

//     // Roundtrip on GPS scale (non-epoch instant)
//     let t_gps = TimePoint::from_ymdhms(2020, 1, 1, 0, 0, 0, 0, ClockType::GPS);
//     let back = TimePoint::from_epoch_attos(t_gps.to_epoch_attos(ClockType::GPS), ClockType::GPS);
//     assert_eq!(t_gps, back);

//     let x = TimePoint::from_ymdhms(2016, 12, 31, 23, 59, 59, 0, ClockType::UTC);
//     eprintln!("internal tai sec after from_ymdhms 59: {}", x.sec());
//     let leap = TimePoint::from_ymdhms(2016, 12, 31, 23, 59, 60, 0, ClockType::UTC);
//     eprintln!("internal tai sec after from_ymdhms 60: {}", leap.sec());
//     let y = TimePoint::from_ymdhms(2017, 1, 1, 0, 0, 0, 0, ClockType::UTC);
//     eprintln!(
//         "internal tai sec after from_ymdhms 00 next day: {}",
//         y.sec()
//     );

//     eprintln!(
//         "is_leap_second(&leap): {:?}",
//         get_leap_seconds(&leap, false)
//     );
//     eprintln!(
//         "leap_seconds_before(&leap): {:?}",
//         get_leap_seconds(&leap, false)
//     );

//     // ------------------------------------------------------------
//     // Leap second case (put at the bottom as requested)
//     // 2016-12-31 23:59:60 UTC  →  civil unix timestamp of 2017-01-01 00:00:00
//     // ------------------------------------------------------------
//     let leap = TimePoint::from_ymdhms(2016, 12, 31, 23, 59, 60, 0, ClockType::UTC);
//     let leap_attos = leap.to_epoch_attos(ClockType::UTC);

//     let unix_sec_part = leap_attos.div_euclid(ATTOS_PER_SEC_I128);
//     assert_eq!(unix_sec_part, 1_483_228_799);

//     let after = TimePoint::from_ymdhms(2017, 1, 1, 0, 0, 0, 0, ClockType::UTC);
//     let after_attos = after.to_epoch_attos(ClockType::UTC);

//     let unix_sec_part = after_attos.div_euclid(ATTOS_PER_SEC_I128);
//     assert_eq!(unix_sec_part, 1_483_228_800); // 2017-01-01 00:00:00 UTC

//     // The fractional part should be zero for this instant
//     assert_eq!(after_attos % ATTOS_PER_SEC_I128, 0);
// }
