/*
from astropy.time import Time

t = Time("2020-01-01 00:00:00", scale="tai")

print("galexsec", t.galexsec)
print("gps", t.gps)
print("cxcsec", t.cxcsec)
print("unix_utc", t.unix)
print("unix_tai", t.unix_tai)

print("jyear", t.jyear)
print("byear", t.byear)
print("decimalyear", t.decimalyear)
print("yday", t.yday)

scales = ["tai", "tt", "tdb", "tcg", "tcb", "utc", "ut1"]

for scale in scales:
    ts = getattr(t, scale)
    print(f"jd_{scale}", ts.jd)

"""
galexsec 1261871963.0
gps 1261871981.0
cxcsec 694224032.184
unix_utc 1577836763.0
unix_tai 1577836800.0
jyear 2019.9986310746065
byear 2020.000335739628
decimalyear 2020.0
yday 2020:001:00:00:00.000
jd_tai 2458849.5
jd_tt 2458849.5003725
jd_tdb 2458849.5003724988
jd_tcg 2458849.500383445
jd_tcb 2458849.500616009
jd_utc 2458849.4995717593
jd_ut1 2458849.499569709
"""
*/

use deep_time::{Dt, Scale};

#[cfg(test)]
mod astropy_verified_conversions_tests {
    use super::*;

    #[test]
    fn galexsec() {
        let galexsec = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to_galexsec(Scale::TAI)
            .to_sec_f();
        // galexsec  : 1261871963.0
        assert_eq!(galexsec, 1261871963.0);
    }

    #[test]
    fn gps() {
        let gps = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to_gps(Scale::TAI)
            .to_sec_f();
        // gps       : 1261871981.0
        assert_eq!(gps, 1261871981.0);
    }

    #[test]
    fn unix_tai() {
        let unix_tai = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to_unix(Scale::TAI, Scale::TAI)
            .to_sec_f();
        // unix_tai 1577836800.0
        assert_eq!(unix_tai, 1577836800.0);
    }

    #[test]
    fn unix_utc() {
        let unix = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to_unix(Scale::TAI, Scale::UTC)
            .to_sec_f();
        // unix_utc 1577836763.0
        assert_eq!(unix, 1577836763.0);
    }

    #[test]
    fn cxcsec() {
        let cxc = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to_cxcsec(Scale::TAI)
            .to_sec_f();
        // cxcsec 694224032.184
        assert_eq!(cxc, 694224032.184);
    }

    #[test]
    fn tai_jd() {
        let jd = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI).to_jd_f();
        // jd_tai 2458849.5
        assert_eq!(jd, 2458849.5);
    }

    #[test]
    fn tt_jd() {
        let jd = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to(Scale::TAI, Scale::TT)
            .to_jd_f();
        // jd_tt 2458849.5003725
        assert_eq!(jd, 2458849.5003725);
    }

    #[test]
    fn tdb_jd() {
        let jd = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to(Scale::TAI, Scale::TDB)
            .to_jd_f();
        // jd_tdb 2458849.5003724988
        assert_eq!(jd, 2458849.5003724988);
    }

    #[test]
    fn tcg_jd() {
        let jd = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to(Scale::TAI, Scale::TCG)
            .to_jd_f();
        // jd_tcg 2458849.500383445
        assert_eq!(jd, 2458849.500383445);
    }

    // #[test]
    // fn tcb_jd() {
    //     let jd = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
    //         .to(Scale::TAI, Scale::TCB)
    //         .to_jd_f();
    //     // jd_tcb 2458849.500616009
    //     assert_eq!(jd, 2458849.500616009);
    // }
}
