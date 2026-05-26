mod arithmetic;
mod constructors;
mod conveniences;
mod conversions;
mod decimal_year;
mod from_ccsds;
mod from_str;
mod gregorian;
mod julian_date;
mod ops;
mod tdb;
mod to_bin_ccsds;
mod to_str;

pub mod lunar;
pub mod numbers_traits;
pub mod trajectory;

#[cfg(feature = "alloc")]
mod to_str_ccsds;

#[cfg(feature = "mars")]
pub mod mars;

#[cfg(feature = "hifitime")]
mod hifitime;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "jiff")]
mod jiff;

use crate::ATTOS_PER_SEC;
use core::fmt;

/// **The library's central time type.** A high-precision instant/duration with attosecond
/// resolution.
///
/// **Fields:**
///
/// - `pub attos: i128` — total time in attoseconds since the reference epoch (2000-01-01 noon), as a signed integer. Negative values represent times before the epoch.
///
/// **Notes:**
///
/// - Supports a range of roughly ±5.39 trillion years.
/// - Implements `Copy` and `Clone`. Optional derives for `serde` and `tsify` are available
///   behind the corresponding features.
/// - Does **not** store a time scale internally. The scale is always an explicit parameter
///   of conversion and construction methods.
/// - A wide range of math is available for this type, but it's not calendar aware, for basic
///   calendar aware math use the [`YmdHms`] type.
///
/// ## Reference epoch and scales
///
/// - The librarys epoch for nearly all functionality such as the conversion functions is
///   **2000-01-01 noon**. See also: [`Scale`](../enum.Scale.html).
/// - When using the conversion functions
///   and [`Dt::from`](../struct.Dt.html#method.from) etc. the epoch is
/// - Leap-second handling follows the chosen `Scale` (UTC, UTCSpice, UTCSofa).
///
/// ## See also (non-exhaustive list)
///
/// ### From and to calendar dates
///
/// - [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd)
/// - [`Dt::from_ymdhms`](../struct.Dt.html#method.from_ymdhms)
/// - [`Dt::from_ymdhms_on`](../struct.Dt.html#method.from_ymdhms_on)
/// - [`Dt::to_ymdhms`](../struct.Dt.html#method.to_ymdhms)
/// - [`Dt::to_ymdhms_on`](../struct.Dt.html#method.to_ymdhms_on)
/// - [`Dt::to_ymdhms_rich_on`](../struct.Dt.html#method.to_ymdhms_rich_on)
///
/// ### From and to str and bytes
///
/// Some of these require the alloc feature, they're marked with *
///
/// - [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse)*
/// - [`Dt::from_str_ccsds`](../struct.Dt.html#method.from_str_ccsds)
/// - [`Dt::parse`](../struct.Dt.html#method.parse)
/// - [`Dt::from_str`](../struct.Dt.html#method.from_str)
/// - [`Dt::to_str`](../struct.Dt.html#method.to_str)*
/// - [`Dt::to_str_with_offset`](../struct.Dt.html#method.to_str_with_offset)*
/// - [`Dt::to_str_with_tz`](../struct.Dt.html#method.to_str_with_tz)*
/// - [`Dt::to_str_iso8601`](../struct.Dt.html#method.to_str_iso8601)*
/// - [`Dt::to_str_bin`](../struct.Dt.html#method.to_str_bin)
/// - [`Dt::to_str_bin_with_offset`](../struct.Dt.html#method.to_str_bin_with_offset)
/// - [`Dt::to_str_bin_with_tz`](../struct.Dt.html#method.to_str_bin_with_tz)
///
/// ### From and to julian dates
///
/// - [`Dt::from_jd_f`](../struct.Dt.html#method.from_jd_f)
/// - [`Dt::from_mjd_f`](../struct.Dt.html#method.from_mjd_f)
/// - [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f)
/// - [`Dt::to_mjd_f`](../struct.Dt.html#method.to_mjd_f)
/// - [`Dt::ymd_to_jd`](../struct.Dt.html#method.ymd_to_jd)
/// - [`Dt::jd_to_ymd`](../struct.Dt.html#method.jd_to_ymd)
///
/// ### Conversions, time scales etc.
///
/// - [`Dt::from`](../struct.Dt.html#method.from)
/// - [`Dt::to`](../struct.Dt.html#method.to)
/// - [`Dt::to_unix`](../struct.Dt.html#method.to_unix)
/// - [`Dt::to_ntp`](../struct.Dt.html#method.to_ntp)
/// - [`Dt::to_gps_wk_and_tow`](../struct.Dt.html#method.to_gps_wk_and_tow)
///
/// ### Conversions from and to types from other libraries
///
/// - [`Dt::to_hifitime_epoch`](../struct.Dt.html#method.to_hifitime_epoch)
/// - [`Dt::to_jiff_timestamp`](../struct.Dt.html#method.to_jiff_timestamp)
/// - [`Dt::to_chrono_datetime_utc`](../struct.Dt.html#method.to_chrono_datetime_utc)
/// - [`Dt::from_hifitime_epoch`](../struct.Dt.html#method.from_hifitime_epoch)
/// - [`Dt::from_jiff_timestamp`](../struct.Dt.html#method.from_jiff_timestamp)
/// - [`Dt::from_chrono_datetime_utc`](../struct.Dt.html#method.from_chrono_datetime_utc)
///
/// ## Examples
///
/// ### Parsing a date
///
/// ```
/// use deep_time::{Dt, Scale};
///
/// // uses impl FromStr but Dt::parse provides the same functionality
/// let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
///
/// let ymd = x.to_ymdhms(Scale::TAI);
/// assert_eq!(ymd.yr(), 2000);
/// assert_eq!(ymd.mo(), 1);
/// assert_eq!(ymd.day(), 1);
/// assert_eq!(ymd.hr(), 12);
/// assert_eq!(ymd.min(), 0);
/// assert_eq!(ymd.sec(), 0);
/// assert_eq!(ymd.attos(), 0);
/// ```
///
/// ### Outputting a date to string / bytes
///
/// ```
/// # #[cfg(all(feature = "tz", feature = "parse"))]
/// # {
/// use deep_time::{Dt, Scale};
///
/// let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
///
/// let s = x
///  .to_str_with_tz(Scale::TAI, "%A, %B %d, %Y %H:%M:%S %Q", "America/New_York")
///  .unwrap();
/// let b = x
///  .to_str_bin_with_tz(Scale::TAI, "%A, %B %d, %Y %H:%M:%S %Q", "America/New_York")
///  .unwrap();
///
/// assert_eq!(s, "Saturday, January 01, 2000 07:00:00 America/New_York");
/// assert_eq!(b.as_str().unwrap(), "Saturday, January 01, 2000 07:00:00 America/New_York");
/// # }
/// ```
///
/// ### Creating a unix timestamp in milliseconds
///
/// ```
/// use deep_time::{Dt, Scale};
///
/// // this fn converts from UTC and creates a TAI Dt
/// let dt = Dt::from_ymdhms(2000, 1, 1, 12, 0, 0, 0);
///
/// // dt is now TAI so the current Scale is TAI, it was originally UTC though
/// let unix_ms = dt.to_unix(Scale::TAI, Scale::UTC).to_ms();
///
/// // unix timestamp in ms for 2000-01-01 noon UTC
/// assert_eq!(unix_ms, 946728000000);
/// ```
///
/// ### Converting time scales
///
/// ```
/// use deep_time::{Dt, Scale};
///
/// // this fn converts from UTC and creates a TAI Dt
/// let dt = Dt::from_ymdhms(2000, 1, 1, 12, 0, 0, 0);
///
/// // to tdb
/// let tdb = dt.to(Scale::TAI, Scale::TDB);
///
/// // then to tt, the current scale is TDB
/// let tt = tdb.to(Scale::TDB, Scale::TT);
///
/// // then back to TAI
/// let tai = tt.to(Scale::TT, Scale::TAI);
///
/// // round trip equality
/// assert_eq!(dt, tai);
/// ```
///
/// ### Performing some basic calendar aware math
///
/// ```
/// use deep_time::{Dt, Scale};
///
/// let x = Dt::from_ymd(2000, 2, 29).to_ymdhms(Scale::TAI);
/// let x = x.add_yr(1);
///
/// assert_eq!(x.day(), 28);
/// ```
///
/// ### Changing a dates format
///
/// ```
/// use deep_time::{Dt, StrPTimeFmt};
///
/// let fmt = Dt::parse_fmt("%Y-%m-%dT%H:%M:%S").unwrap();
///
/// # #[cfg(feature = "alloc")]
/// let s = fmt.to_str("2000-01-01T12:00:00", "%d %m %Y %H:%M:%S", false, false, false).unwrap();
/// ```
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct Dt {
    pub attos: i128,
}

impl Default for Dt {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.attos;
        let precision = f.precision().unwrap_or(9);

        if f.sign_plus() && total >= 0 {
            write!(f, "+")?;
        }

        if total >= 0 {
            let secs = total / ATTOS_PER_SEC as i128;
            let attos_frac = total % ATTOS_PER_SEC as i128;
            write!(f, "{}", secs)?;

            if precision > 0 && attos_frac > 0 {
                let prec = precision.min(18);
                let scale = 10u128.pow(18 - prec as u32);
                let value = (attos_frac as u128) / scale;
                write!(f, ".{:0>width$}", value, width = prec)?;
            }
        } else {
            let abs_total = (-total) as u128;
            let int_secs = (abs_total / ATTOS_PER_SEC as u128) as u64;
            let frac_attos = (abs_total % ATTOS_PER_SEC as u128) as u64;

            if frac_attos == 0 {
                write!(f, "-{}", int_secs)?;
            } else {
                write!(f, "-{}", int_secs)?;
                if precision > 0 {
                    let prec = precision.min(18);
                    let scale = 10u128.pow(18 - prec as u32);
                    let value = (frac_attos as u128) / scale;
                    write!(f, ".{:0>width$}", value, width = prec)?;
                }
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dt").field("attos", &self.attos).finish()
    }
}
