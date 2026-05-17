#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

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

print("")

for year in [
    1000,
    1500,
    2000,
    2001,
    2002,
    2003,
    2004,
    2005,
    2006,
    2007,
    2009,
    2010,
    2011,
    2012,
    2013,
    2014,
    2015,
    2016,
    2017,
    2018,
    2020,
    2021,
    2027,
    2030,
    2035,
    2099,
    4000,
    8000,
]:
    for month in ["01", "04"]:
        x = Time(f"{year}-{month}-01 00:00:00", scale="tai")
        print(f"{x.tdb.jd:.10f},")

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

2086302.5003725048,
2086392.5003725193,
2268923.5003725016,
2269013.5003725193,
2451544.5003724988,
2451635.5003725188,
2451910.5003724992,
2452000.5003725188,
2452275.5003724992,
2452365.5003725188,
2452640.5003724992,
2452730.5003725188,
2453005.5003724992,
2453096.5003725193,
2453371.5003724997,
2453461.5003725193,
2453736.5003724992,
2453826.5003725193,
2454101.5003724992,
2454191.5003725193,
2454832.5003724992,
2454922.5003725193,
2455197.5003724988,
2455287.5003725193,
2455562.5003724988,
2455652.5003725193,
2455927.5003724988,
2456018.5003725193,
2456293.5003724992,
2456383.5003725188,
2456658.5003724992,
2456748.5003725188,
2457023.5003724992,
2457113.5003725193,
2457388.5003724992,
2457479.5003725193,
2457754.5003724992,
2457844.5003725193,
2458119.5003724992,
2458209.5003725193,
2458849.5003724988,
2458940.5003725193,
2459215.5003724988,
2459305.5003725193,
2461406.5003724992,
2461496.5003725188,
2462502.5003724992,
2462592.5003725193,
2464328.5003724988,
2464418.5003725188,
2487704.5003724992,
2487794.5003725193,
3182029.5003724890,
3182120.5003725146,
4642999.5003724843,
4643090.5003724955,
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
    fn decimal_year() {
        let x = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI);
        let y = Dt::from_ymdhms_on(-2000, 1, 1, 0, 0, 0, 0, Scale::TAI);
        // jyear 2019.9986310746065
        assert_eq!(x.to_jyear(), 2019.9986310746065);
        // byear 2020.000335739628
        assert!((x.to_byear() - 2020.000335739628).abs() < 1e-12);
        // decimalyear 2020.0
        assert_eq!(x.to_decimalyear(Scale::TAI), 2020.0);
        // Negative decimal year
        assert_eq!(y.to_decimalyear(Scale::TAI), -2000.0);
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
    fn tcg_jd() {
        let jd = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to(Scale::TAI, Scale::TCG)
            .to_jd_f();
        // jd_tcg 2458849.500383445
        assert_eq!(jd, 2458849.500383445);
    }

    #[test]
    fn tcb_jd() {
        let jd = Dt::from_ymdhms_on(2020, 1, 1, 0, 0, 0, 0, Scale::TAI)
            .to(Scale::TAI, Scale::TCB)
            .to_jd_f();
        // jd_tcb 2458849.500616009
        assert_eq!(jd, 2458849.500616009);
    }

    #[test]
    fn tdb_jd_multi() {
        let results = [
            2086302.5003725048,
            2086392.5003725193,
            2268923.5003725016,
            2269013.5003725193,
            2451544.5003724988,
            2451635.5003725188,
            2451910.5003724992,
            2452000.5003725188,
            2452275.5003724992,
            2452365.5003725188,
            2452640.5003724992,
            2452730.5003725188,
            2453005.5003724992,
            2453096.5003725193,
            2453371.5003724997,
            2453461.5003725193,
            2453736.5003724992,
            2453826.5003725193,
            2454101.5003724992,
            2454191.5003725193,
            2454832.5003724992,
            2454922.5003725193,
            2455197.5003724988,
            2455287.5003725193,
            2455562.5003724988,
            2455652.5003725193,
            2455927.5003724988,
            2456018.5003725193,
            2456293.5003724992,
            2456383.5003725188,
            2456658.5003724992,
            2456748.5003725188,
            2457023.5003724992,
            2457113.5003725193,
            2457388.5003724992,
            2457479.5003725193,
            2457754.5003724992,
            2457844.5003725193,
            2458119.5003724992,
            2458209.5003725193,
            2458849.5003724988,
            2458940.5003725193,
            2459215.5003724988,
            2459305.5003725193,
            2461406.5003724992,
            2461496.5003725188,
            2462502.5003724992,
            2462592.5003725193,
            2464328.5003724988,
            2464418.5003725188,
            2487704.5003724992,
            2487794.5003725193,
            3182029.5003724890,
            3182120.5003725146,
            4642999.5003724843,
            4643090.5003724955,
        ];
        let mut results_idx: usize = 0;
        const TOLERANCE: f64 = 2.0e-9;
        for yr in [
            1000, 1500, 2000, 2001, 2002, 2003, 2004, 2005, 2006, 2007, 2009, 2010, 2011, 2012,
            2013, 2014, 2015, 2016, 2017, 2018, 2020, 2021, 2027, 2030, 2035, 2099, 4000, 8000,
        ]
        .iter()
        {
            for mo in [1, 4] {
                let jd = Dt::from_ymdhms_on(*yr as i64, mo, 1, 0, 0, 0, 0, Scale::TAI)
                    .to(Scale::TAI, Scale::TDB)
                    .to_jd_f();
                let expected = results[results_idx];
                let diff = (jd - expected).abs();

                assert!(
                    diff < TOLERANCE,
                    "{yr}-{mo:02}-01: diff = {diff:.2e} JD (expected {expected}, got {jd})"
                );
                // eprintln!(
                //     "{}-{}-01 diff: {:.2e}",
                //     yr,
                //     mo,
                //     (jd - results[results_idx]).abs()
                // );
                results_idx += 1;
            }
        }
    }
}
