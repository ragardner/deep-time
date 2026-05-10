/*
from astropy.time import Time

tai_time = Time('2020-01-01 00:00:00', scale='tai')
galexsec_value = tai_time.galexsec
gps_value = tai_time.gps

print(f"galexsec    : {galexsec_value}")
print(f"gps    : {gps_value}")

# galexsec  : 1261871963.0
# gps       : 1261871981.0
*/

use deep_time::{Dt, Scale};

#[cfg(test)]
mod astropy_verified_conversions_tests {
    use super::*;

    #[test]
    fn galexsec() {
        let galexsec =
            Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI).to_galexsec(Scale::TAI);
        // # galexsec  : 1261871963.0
        assert_eq!(galexsec, 1261871963.0);
    }

    #[test]
    fn gps() {
        let gps = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI).to_gps(Scale::TAI);
        // # gps       : 1261871981.0
        assert_eq!(gps, 1261871981.0);
    }
}
