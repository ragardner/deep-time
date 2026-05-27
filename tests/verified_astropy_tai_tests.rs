#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

/*
from astropy.time import Time
import astropy.units as u

lib_zero = Time('2000-01-01 12:00:00', scale='tai', format='iso')

test_cases = [
    ('2000-01-01 12:00:00', 'UTC'),
    ('2025-04-16 00:00:00', 'UTC'),
    ('1970-01-01 00:00:00', 'UTC'),
    ('2015-06-30 23:59:59', 'UTC'),
    ('2015-06-30 23:59:60', 'UTC'),
    ('2015-07-01 00:00:00', 'UTC'),
    ('2100-01-01 00:00:00', 'UTC'),
    ('1900-01-01 00:00:00', 'UTC'),
]

for iso, scale in test_cases:
    t = Time(iso, scale=scale.lower(), format='iso')
    delta = (t.tai - lib_zero).to(u.s).value
    print(f"{iso} ({scale}) -> {format(delta, '.20f')}")

2000-01-01 12:00:00 (UTC) -> 31.99999999999860733624
2025-04-16 00:00:00 (UTC) -> 798033637.00000000000000000000
1970-01-01 00:00:00 (UTC) -> -946727991.99991798400878906250
2015-06-30 23:59:59 (UTC) -> 488980834.00000000000000000000
2015-06-30 23:59:60 (UTC) -> 488980835.00000000000000000000
2015-07-01 00:00:00 (UTC) -> 488980836.00000000000000000000
2100-01-01 00:00:00 (UTC) -> 3155716837.00000000000000000000
1900-01-01 00:00:00 (UTC) -> -3155716800.00000000000000000000
*/

use deep_time::{Dt, Scale};

mod astropy_verified_tai_sec_tests {
    use deep_time::constants::ATTOS_PER_SEC_I128;

    use super::*;

    #[test]
    fn tai_sec_at_unix_epoch() {
        let dt = Dt::from_ymdhms_on(1970, 1, 1, 0, 0, 0, 0, Scale::UTCSofa);

        let got = dt.to_sec_f();

        // 1970-01-01 00:00:00 (UTC) -> -946727991.99991798400878906250
        let diff = (got - -946727991.99991798400878906250).abs();

        assert!(
            diff < 1e-12,
            "UTCSofa seconds differ from Astropy ground truth\n\
         got      = {:.15}\n\
         expected = {:.15}\n\
         diff     = {:.3e}",
            got,
            -946727991.99991798400878906250,
            diff
        );
    }

    #[test]
    fn tai_sec_at_2000_01_01_12utc() {
        let dt = Dt::from_ymdhms(2000, 1, 1, 12, 0, 0, 0);
        assert_eq!(dt.to_sec(), 32);
        assert_eq!(dt.to_sec_frac(), 0);
    }

    #[test]
    fn tai_sec_at_2025_04_16() {
        let dt = Dt::from_ymdhms(2025, 4, 16, 0, 0, 0, 0);
        assert_eq!(dt.to_sec(), 798033637);
        assert_eq!(dt.to_sec_frac(), 0);
    }

    #[test]
    fn tai_sec_around_2015_leap_second() {
        let before = Dt::from_ymdhms(2015, 6, 30, 23, 59, 59, 0);
        assert_eq!(before.to_sec(), 488980834);
        assert_eq!(before.to_sec_frac(), 0);

        let leap = Dt::from_ymdhms(2015, 6, 30, 23, 59, 60, 0);
        assert_eq!(leap.to_sec(), 488980835);
        assert_eq!(leap.to_sec_frac(), 0);

        let after = Dt::from_ymdhms(2015, 7, 1, 0, 0, 0, 0);
        assert_eq!(after.to_sec(), 488980836);
        assert_eq!(after.to_sec_frac(), 0);
    }

    #[test]
    fn tai_sec_at_2100_01_01() {
        let dt = Dt::from_ymdhms(2100, 1, 1, 0, 0, 0, 0);
        assert_eq!(dt.to_sec(), 3155716837);
        assert_eq!(dt.to_sec_frac(), 0);
    }

    #[test]
    fn tai_sec_at_1900_01_01() {
        let dt = Dt::from_ymdhms(1900, 1, 1, 0, 0, 0, 0);
        assert_eq!(dt.to_sec(), -3155716800);
        assert_eq!(dt.to_sec_frac(), 0);
    }
}
