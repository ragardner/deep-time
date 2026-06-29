mod arithmetic;
mod arithmetic_calendar;
mod constructors;
mod conveniences;
mod conversions;
mod et;
mod from_ccsds;
mod from_str;
mod gregorian;
mod julian_date;
mod ops;
mod to_bin_ccsds;
mod to_str;

pub mod lunar;
pub mod numbers_traits;

#[cfg(feature = "alloc")]
mod to_str_ccsds;

#[cfg(feature = "hifitime")]
mod hifitime;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "jiff")]
mod jiff;

#[cfg(feature = "mars")]
pub mod mars;

#[cfg(feature = "tdb_fairhead1990")]
pub mod tdb_fairhead1990;

#[cfg(not(feature = "tdb_fairhead1990"))]
mod tdb;

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
/// - pub scale: [`Scale`] - the current time scale of the object.
/// - pub target: [`Scale`] - a target time scale used by many output functions such as
///   [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd) and
///   [`Dt::to_unix`](../struct.Dt.html#method.to_unix). The functions convert to the
///   `target` time scale before producing an output.
///
/// **Notes:**
///
/// - In theory it supports a range of roughly ±5.39 trillion years but many of the to and
///   from functions cap at i64 seconds, which can mean a range of ±292 billion years in practice.
///   Additionally, when parsing dates with a timezone the Rust library `jiff` is used which has
///   a limit of `-9999 - 9999` years.
/// - Implements `Copy` and `Clone`. Optional derives for `serde` and `tsify` are available
///   behind the corresponding features.
/// - A wide range of math is available for this type, including basic calendar aware math and,
///   with the `jiff-tz` feature enabled, timezone and DST aware math. **Behavior greatly
///   differs between functions.**
/// - **Comparison** (`==`, `Ord`, and [`Dt::cmp`](../struct.Dt.html#method.cmp)) uses only the
///   `attos` field. `scale` and `target` are not consulted and no time-scale conversion is
///   performed. To test whether two values denote the same physical instant, convert both to a
///   common scale (e.g. with [`Dt::to`](../struct.Dt.html#method.to)) before comparing.
///
/// ```rust
/// use deep_time::{Dt, Scale};
///
/// let tai = Dt::ZERO;
/// let relabeled = tai.with(Scale::TT); // relabels scale only — attos unchanged
///
/// assert_eq!(tai, relabeled);
/// assert_ne!(tai, tai.to(Scale::TT)); // .to() converts attos — no longer equal
/// ```
///
/// ## Reference epoch and scales
///
/// - The librarys epoch for nearly all functionality such as the conversion functions is
///   **2000-01-01 noon**. See also: [`Scale`](../enum.Scale.html).
/// - Leap-second handling follows the chosen `Scale` (UTC, UtcSpice, UtcHist).
///
/// ## See also (non-exhaustive list)
///
/// ### From and to calendar dates
///
/// - [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd)
/// - [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd)
///
/// ### From and to str and bytes
///
/// Some of these require the alloc feature, they're marked with *
///
/// - [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse)*
/// - [`Dt::from_str_iso`](../struct.Dt.html#method.from_str_iso)
/// - [`Dt::parse`](../struct.Dt.html#method.parse)
/// - [`Dt::from_str`](../struct.Dt.html#method.from_str)
/// - [`Dt::to_str`](../struct.Dt.html#method.to_str)*
/// - [`Dt::to_str_in_offset`](../struct.Dt.html#method.to_str_in_offset)*
/// - [`Dt::to_str_in_tz`](../struct.Dt.html#method.to_str_in_tz)*
/// - [`Dt::to_str_iso8601`](../struct.Dt.html#method.to_str_iso8601)*
/// - [`Dt::to_str_lite`](../struct.Dt.html#method.to_str_lite)
/// - [`Dt::to_str_lite_in_offset`](../struct.Dt.html#method.to_str_lite_in_offset)
/// - [`Dt::to_str_lite_in_tz`](../struct.Dt.html#method.to_str_lite_in_tz)
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
/// - [`Dt::target`](../struct.Dt.html#method.target)
/// - [`Dt::to`](../struct.Dt.html#method.to)
/// - [`Dt::convert`](../struct.Dt.html#method.convert)
/// - [`Dt::to_tai`](../struct.Dt.html#method.to_tai)
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
/// ```rust
/// use deep_time::{Dt, Scale};
///
/// // uses impl FromStr but Dt::parse provides the same functionality
/// let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
///
/// let ymd = x.to_ymd();
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
/// ```rust
/// # #[cfg(all(any(feature = "jiff-tz", feature = "jiff-tz-bundle"), feature = "parse"))]
/// # {
/// use deep_time::{Dt, Lang, Scale};
///
/// let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
///
/// let s = x
///  .to_str_in_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York", Lang::En)
///  .unwrap();
/// let b = x
///  .to_str_lite_in_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York", Lang::En)
///  .unwrap();
///
/// assert_eq!(s, "Saturday, January 01, 2000 07:00:00 America/New_York");
/// assert_eq!(b.as_str(), "Saturday, January 01, 2000 07:00:00 America/New_York");
/// # }
/// ```
///
/// ### Creating a unix timestamp in milliseconds
///
/// ```rust
/// use deep_time::{Dt, Scale};
///
/// // this fn converts from UTC and creates a TAI Dt
/// let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
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
/// Many functions such as
/// [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd) will convert to
/// `TAI` from the [`Dt`]s current `scale` then to the [`Dt`]s `target`
/// [`Scale`] prior to producing an output.
///
/// So you don't necessarily have to convert time scales prior to using
/// many of the output functions. You just have to change the `target`
/// time scale.
///
/// #### Using the target field
///
/// ```rust
/// use deep_time::{Dt, Lang, Scale};
///
/// // Leap seconds were added to the secounds count
/// // This Dt has attos that are now on the TAI timescale
/// let dt = Dt::from_ymd(2025, 1, 1, Scale::UTC, 0, 0, 0, 0);
///
/// // The internal target is currently UTC so we don't need to do
/// // anything to output back to UTC and round trip
/// let bytes = dt.to_str_lite("%d %m %Y %H:%M:%S", Lang::En).unwrap();
///
/// assert_eq!(bytes.as_str(), "01 01 2025 00:00:00");
///
/// // Perhaps we want to make a GPS timestamp out of our Dt
/// // If we want it to be on the GPS time scale we have to set the
/// // target prior to calling to_gps()
/// let gps = dt.target(Scale::GPS).to_gps().to_sec_f();
/// ```
///
/// #### Converting the internal attos to a new time scale
///
/// ```rust
/// use deep_time::{Dt, Scale};
///
/// // this fn converts from UTC and creates a TAI Dt
/// let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
///
/// // to tdb
/// let tdb = dt.to(Scale::TDB);
///
/// // then to tt, the current scale is TDB
/// let tt = tdb.to(Scale::TT);
///
/// // then back to TAI
/// let tai = tt.to(Scale::TAI);
///
/// // round trip equality
/// assert_eq!(dt, tai);
/// ```
///
/// ### Performing some basic calendar aware math
///
/// ```rust
/// use deep_time::{Dt, Scale};
///
/// let x = Dt::from_ymd(2000, 2, 29, Scale::UTC, 0, 0, 0, 0).to_ymd();
/// let x = x.add_yr(1);
///
/// assert_eq!(x.day(), 28);
/// ```
///
/// ### Changing a dates format
///
/// ```rust
/// use deep_time::{Dt, Lang, StrPTimeFmt};
///
/// let fmt = Dt::parse_fmt("%Y-%m-%dT%H:%M:%S").unwrap();
///
/// # #[cfg(feature = "alloc")]
/// let s = fmt.to_str("2000-01-01T12:00:00", "%d %m %Y %H:%M:%S", false, false, false, Lang::En).unwrap();
///
/// # #[cfg(feature = "alloc")]
/// assert_eq!(s, "01 01 2000 12:00:00", "expected: {}, got: {}", "01 01 2000 12:00:00", s);
/// ```
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Dt {
    pub attos: i128,
    pub scale: Scale,
    pub target: Scale,
}

impl Dt {
    /// Returns a new [`Dt`] with the `target` field set to the given
    /// `t` arg.
    #[inline(always)]
    pub const fn target(&self, t: Scale) -> Dt {
        Dt::new(self.attos, self.scale, t)
    }

    /// Returns a new [`Dt`] with the `scale` field sr to the given
    /// `s` arg.
    ///
    /// **Does NOT perform any time scale conversions**.
    #[inline(always)]
    pub const fn with(&self, s: Scale) -> Dt {
        Dt::new(self.attos, s, self.target)
    }
}

impl Default for Dt {
    fn default() -> Dt {
        Self::ZERO
    }
}

impl fmt::Display for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.to_attos();
        let precision = f.precision().unwrap_or(18).min(18);

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

        write!(f, "{}", whole_seconds)?;

        if precision > 0 && fractional_attos > 0 {
            let scale = 10u128.pow(18 - precision as u32);
            let frac_value = fractional_attos / scale;

            if frac_value > 0 {
                f.write_str(".")?;

                let mut digits = [0u8; 18];
                let mut n = frac_value;

                for i in (0..precision).rev() {
                    digits[i] = (n % 10) as u8;
                    n /= 10;
                }

                let last = digits[..precision]
                    .iter()
                    .rposition(|&d| d != 0)
                    .unwrap_or(0);

                for &d in &digits[..=last] {
                    write!(f, "{}", d)?;
                }
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dt")
            .field("attos", &self.to_attos())
            .field("scale", &self.scale)
            .field("target", &self.target)
            .finish()
    }
}

#[cfg(feature = "wire")]
impl Dt {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize = 19;

    /// Serializes this `Dt` into a fixed 18-byte little-endian buffer using the
    /// `attos: i128` + `scale: Scale` representation.
    ///
    /// ## Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `[1..17]`: total attoseconds as little-endian `i128`
    /// - Byte `17`: scale as `u8` (enum discriminant)
    /// - Byte `18`: target as `u8` (enum discriminant)
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;
        buf[1..17].copy_from_slice(&self.attos.to_le_bytes());
        buf[17] = self.target as u8;
        buf
    }

    /// Deserializes a [`Dt`] from exactly 18 bytes of wire data.
    ///
    /// Returns `None` if the version byte is unknown, the length is wrong,
    /// or the scale byte is not a valid `Scale` variant.
    ///
    /// ## Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `[1..17]`: total attoseconds as little-endian `i128`
    /// - Byte `17`: scale as `u8` (enum discriminant)
    /// - Byte `18`: target as `u8` (enum discriminant)
    ///
    /// ## Security
    ///
    /// Safe to call with completely untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let attos = i128::from_le_bytes([
            bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16],
        ]);

        let scale = Scale::from_u8(bytes[17]);
        let target = Scale::from_u8(bytes[18]);

        Some(Dt::new(attos, scale, target))
    }
}
