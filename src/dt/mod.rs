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

use crate::{ATTOS_PER_SEC, Scale};
use core::fmt;

/// **The library's central time type.** A high-precision instant/duration with attosecond
/// resolution.
///
/// **Fields:**
///
/// - pub attos: [`i128`] - total time in attoseconds since the reference epoch
///   (2000-01-01 noon), as a signed integer. Negative values represent times
///   before the epoch.
/// - pub tag: [`Scale`] - a timescale tag, for example when a `Dt` is created by
///   [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd) despite being TAI
///   internally, the timescale tag will be whatever [`Scale`] was used for
///   [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd). This timescale tag
///   is then used by the many functions such as
///   [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd) which **expect** a TAI
///   internal representation, but use the tag to determine which timescale to
///   output the date as.
///
/// **Notes:**
///
/// - In theory it supports a range of roughly ±5.39 trillion years but many of the to and
///   from functions cap at i64 seconds, which can mean a range of ±292 billion years in practice.
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
/// - [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd)
/// - [`Dt::to_ymd_rich`](../struct.Dt.html#method.to_ymd_rich)
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
/// - [`Dt::tag`](../struct.Dt.html#method.tag)
/// - [`Dt::from_sec`](../struct.Dt.html#method.from_sec)
/// - [`Dt::to_sec64`](../struct.Dt.html#method.to_sec64)
/// - [`Dt::from_attos`](../struct.Dt.html#method.from_attos)
/// - [`Dt::convert_internal`](../struct.Dt.html#method.convert_internal)
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
///  .to_str_with_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York")
///  .unwrap();
/// let b = x
///  .to_str_bin_with_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York")
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
/// let dt = Dt::from_ymd(2000, 1, 1, 12, 0, 0, 0, Scale::UTC);
///
/// // dt is internally TAI but has a UTC tag
/// let unix_ms = dt.to_unix().to_ms();
///
/// // unix timestamp in ms for 2000-01-01 noon UTC
/// assert_eq!(unix_ms, 946728000000);
/// ```
///
/// ### Converting time scales
///
/// There are two ways to mess about with time scales. **Most output
/// functions such as [`Dt::to_unix`](../struct.Dt.html#method.to_unix)
/// and [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd) require the
/// internal `attos` of the [`Dt`] to be TAI.** This rule is dependent
/// on the function.
///
/// So to simply make some output functions output on a particular timescale
/// you merely need to change the tag to a different scale.
///
/// ```
/// use deep_time::{Dt, Scale};
///
/// // Leap seconds were added to the secounds count
/// // This Dt has attos that are now on the TAI timescale
/// let dt = Dt::from_ymd(2025, 1, 1, 0, 0, 0, 0, Scale::UTC);
///
/// // The internal tag is currently UTC so we don't need to do
/// // anything to output back to UTC and round trip
/// let bytes = dt.to_str_bin("%d %m %Y").unwrap();
///
/// assert_eq!(bytes.as_str().unwrap(), "01 01 2025");
///
/// // But if we wanted to change the tag to effectively convert
/// // to something like TT before output:
/// let tt = dt.tag(Scale::TT);
///
///
///
/// ```
///
/// ```
/// use deep_time::{Dt, Scale};
///
/// // this fn converts from UTC and creates a TAI Dt
/// let dt = Dt::from_ymd(2000, 1, 1, 12, 0, 0, 0, Scale::UTC);
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
///
/// # #[cfg(feature = "alloc")]
/// assert_eq!(s, "01 01 2000 12:00:00", "expected: {}, got: {}", "01 01 2000 12:00:00", s);
/// ```
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct Dt {
    pub attos: i128,
    pub scale: Scale,
    pub target: Scale,
}

impl Dt {
    #[inline(always)]
    pub const fn target(&self, t: Scale) -> Dt {
        Dt::new(self.attos, self.scale, t)
    }

    #[inline(always)]
    pub(crate) const fn with(&self, s: Scale) -> Dt {
        Dt::new(self.attos, s, self.target)
    }
}

impl Default for Dt {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.to_attos();
        let precision = f.precision().unwrap_or(9).min(18);

        let is_negative = total < 0;
        let abs_attos = if is_negative {
            total.wrapping_neg() as u128
        } else {
            total as u128
        };

        if is_negative {
            f.write_str("-")?;
        } else if f.sign_plus() {
            f.write_str("+")?;
        }

        let attos_per_sec = ATTOS_PER_SEC as u128;
        let whole_seconds = abs_attos / attos_per_sec;
        let fractional_attos = abs_attos % attos_per_sec;

        // Integer seconds
        write!(f, "{}", whole_seconds)?;

        // Fractional part (only when requested *and* non-zero after truncation)
        if precision > 0 && fractional_attos > 0 {
            let scale = 10u128.pow(18 - precision as u32);
            let frac_value = fractional_attos / scale;

            if frac_value > 0 {
                write!(f, ".{:0>width$}", frac_value, width = precision)?;
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dt")
            .field("attos", &self.to_attos())
            .finish()
    }
}
