mod arithmetic;
mod arithmetic_calendar;
mod constructors;
mod conveniences;
mod et;
mod from_ccsds;
mod from_str;
mod gregorian;
mod helpers;
mod julian_date;
mod ops;
mod to_bin_ccsds;
mod to_scale;
mod to_str;
mod to_unit;

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

#[cfg(feature = "time")]
mod time;

#[cfg(feature = "icu")]
mod icu;

#[cfg(feature = "mars")]
pub mod mars;

#[cfg(feature = "tdb-hi")]
pub mod tdb_hi;

#[cfg(not(feature = "tdb-hi"))]
mod tdb;

use crate::{ATTOS_PER_SEC, Scale};
use core::fmt;

/// **The library's central time type.** A high-precision instant/duration with attosecond
/// resolution.
///
/// **Fields:**
///
/// - pub attos: [`i128`] - signed attosecond count. As a duration this is an
///   elapsed span; as an instant it is an offset from some epoch (any epoch may
///   apply depending on construction and use; calendar/conversion APIs commonly
///   use the library epoch of 2000-01-01 noon).
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
/// ## See also
///
/// Functionality:
/// <https://github.com/ragardner/deep-time#overview>
///
/// Feature flags:
/// <https://github.com/ragardner/deep-time#feature-flags>
///
/// Non-exhaustive list of functions:
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
/// - [`Dt::from_str`](../struct.Dt.html#method.from_str)
/// - [`Dt::parse`](../struct.Dt.html#method.parse)
/// - [`Dt::from_strptime`](../struct.Dt.html#method.from_strptime)
/// - [`Dt::to_str`](../struct.Dt.html#method.to_str)*
/// - [`Dt::to_str_in_offset`](../struct.Dt.html#method.to_str_in_offset)*
/// - [`Dt::to_str_in_tz`](../struct.Dt.html#method.to_str_in_tz)*
/// - [`Dt::to_str_iso8601`](../struct.Dt.html#method.to_str_iso8601)*
/// - [`Dt::to_str_b`](../struct.Dt.html#method.to_str_b)
/// - [`Dt::to_str_b_in_offset`](../struct.Dt.html#method.to_str_b_in_offset)
/// - [`Dt::to_str_b_in_tz`](../struct.Dt.html#method.to_str_b_in_tz)
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
/// - [`Dt::to_tai`](../struct.Dt.html#method.to_tai)
/// - [`Dt::convert`](../struct.Dt.html#method.convert)
/// - [`Dt::from_sec`](../struct.Dt.html#method.from_sec)
/// - [`Dt::to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor)
/// - [`Dt::new`](../struct.Dt.html#method.new)
/// - [`Dt::to_unix`](../struct.Dt.html#method.to_unix)
/// - [`Dt::to_ntp`](../struct.Dt.html#method.to_ntp)
/// - [`Dt::to_gps_wk_and_tow`](../struct.Dt.html#method.to_gps_wk_and_tow)
///
/// ### Conversions from and to types from other libraries
///
/// - [`Dt::to_hifitime_epoch`](../struct.Dt.html#method.to_hifitime_epoch)
/// - [`Dt::to_jiff_timestamp`](../struct.Dt.html#method.to_jiff_timestamp)
/// - [`Dt::to_chrono_datetime_utc`](../struct.Dt.html#method.to_chrono_datetime_utc)
/// - [`Dt::to_time_timestamp`](../struct.Dt.html#method.to_time_timestamp)
/// - [`Dt::from_hifitime_epoch`](../struct.Dt.html#method.from_hifitime_epoch)
/// - [`Dt::from_jiff_timestamp`](../struct.Dt.html#method.from_jiff_timestamp)
/// - [`Dt::from_chrono_datetime_utc`](../struct.Dt.html#method.from_chrono_datetime_utc)
/// - [`Dt::from_time_timestamp`](../struct.Dt.html#method.from_time_timestamp)
///
/// ## Examples
///
/// ### Parsing a date
///
/// Without alloc
///
/// ```rust
/// use deep_time::{Dt, Scale};
///
/// // uses impl FromStr but Dt::parse provides the same functionality
/// let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
/// let x = Dt::from_str("2000-01-01 12:00:00").unwrap();
/// let x = Dt::from_strptime(
///     "2000-01-01 12:00:00",
///     "%Y-%m-%d %H:%M:%S",
///     false,
///     false,
///     false,
/// )
/// .unwrap();
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
/// With the lenient, auto-parser (`parse` feature; IANA zones need `jiff-tz`):
///
/// ```rust
/// # #[cfg(all(feature = "parse", any(feature = "jiff-tz", feature = "jiff-tz-bundle")))]
/// # {
/// use deep_time::{Dt, ParseCfg, Scale};
///
/// let cfg = ParseCfg::default();
///
/// // leading junk, dotted date, 12-hour clock, IANA zone in brackets, trailing junk
/// let dt = Dt::from_str_parse(
///     "log >>> 15-Aug-2024 2:30pm [America/New_York] done",
///     &cfg,
/// )
/// .unwrap();
///
/// // same instant as 18:30 UTC (EDT is UTC−4)
/// let expected = Dt::from_ymd(2024, 8, 15, Scale::UTC, 18, 30, 0, 0);
/// assert_eq!(dt, expected);
/// # }
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
///  .to_str_b_in_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York", Lang::En)
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
/// let unix_ms = dt.to_unix().to_ms().0;
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
/// // Leap seconds were added to the seconds count
/// // This Dt has attos that are now on the TAI timescale
/// let dt = Dt::from_ymd(2025, 1, 1, Scale::UTC, 0, 0, 0, 0);
///
/// // The internal target is currently UTC so we don't need to do
/// // anything to output back to UTC and round trip
/// let bytes = dt.to_str_b("%d %m %Y %H:%M:%S", Lang::En).unwrap();
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
/// let x = x.add_years(1);
///
/// assert_eq!(x.day(), 28);
/// ```
///
/// ### Comparisons
///
/// ```rust
/// use deep_time::macros::from_ymd;
/// use deep_time::{Dt, Scale};
///
/// let a = from_ymd!(2000, 1, 1; 12, on=Scale::TAI);
/// let mut b = Dt::from_str("2000-01-01T12 TAI").unwrap();
///
/// // same instant but on the TT time scale
/// b = b.to(Scale::TT);
///
/// // comparisons only use the attos field
/// // changing b to TT has changed its attos
/// assert_ne!(a, b);
///
/// // to check if two Dt's are the same instant
/// // they must be on the same time scale and
/// // from the same epoch
/// b = b.to(Scale::TAI);
/// assert_eq!(a, b);
///
/// // Dt also allows various mathematical operations
/// b = b.to(Scale::UTC);
/// let diff = (a - b).to_sec();
/// assert_eq!(diff, 32);
/// ```
///
/// #### Sorting
///
/// ```rust
/// # #[cfg(feature = "alloc")]
/// # {
/// use deep_time::macros::from_ymd;
/// use deep_time::{Dt, Scale};
///
/// let mut times = vec![
///     from_ymd!(2000, 1, 3),
///     from_ymd!(2000, 1, 1),
///     from_ymd!(2000, 1, 2),
/// ];
///
/// // sort uses Ord, which only looks at attos (not scale / target)
/// times.sort();
///
/// assert_eq!(times[0], from_ymd!(2000, 1, 1));
/// assert_eq!(times[1], from_ymd!(2000, 1, 2));
/// assert_eq!(times[2], from_ymd!(2000, 1, 3));
/// # }
/// ```
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Dt {
    /// Signed attosecond count.
    ///
    /// [`Dt`] can represent a duration or an instant. As a duration, `attos` is
    /// simply an elapsed span. As an instant, it is an offset from some epoch —
    /// any epoch may apply depending on how the value is constructed and used.
    ///
    /// Calendar and conversion APIs commonly interpret it relative to the library
    /// epoch (2000-01-01 noon), but the field itself is only a count of attoseconds.
    pub attos: i128,
    /// The current time scale of this object.
    pub scale: Scale,
    /// Target time scale used by many output functions such as
    /// [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd)
    /// and
    /// [`Dt::to_unix`](../struct.Dt.html#method.to_unix).
    ///
    /// These functions convert to this scale before producing output.
    pub target: Scale,
}

impl Dt {
    /// Returns a new [`Dt`] with the `target` field set to the given
    /// `t` arg.
    #[inline(always)]
    pub const fn target(&self, t: Scale) -> Dt {
        Dt::new(self.attos, self.scale, t)
    }

    /// Returns a new [`Dt`] with the `scale` field set to the given
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

/// Formats a [`Dt`], used when `.to_string()` is called.
///
/// ## Behavior
///
/// - Outer `[` `]`.
/// - Signed decimal **seconds** from the raw `attos` count (up to 18 fractional
///   digits, trailing zeros trimmed), then `s`.
/// - Space, then `scale` and `target` abbreviations separated by `>`
///   ([`Scale::abbrev`]).
///
/// ## Examples
///
/// ```rust
/// use core::fmt::Write;
/// use deep_time::{BufStr, Dt, Scale};
/// use deep_time::macros::from_sec;
///
/// let mut s = BufStr::<64>::default();
/// write!(&mut s, "{}", Dt::ZERO).unwrap();
/// assert_eq!(s.as_str(), "[0s TAI>TAI]");
///
/// let dt = from_sec!(86400, on = Scale::TAI, target = Scale::UTC);
/// s = BufStr::<64>::default();
/// write!(&mut s, "{}", dt).unwrap();
/// assert_eq!(s.as_str(), "[86400s TAI>UTC]");
///
/// let dt = Dt::new(
///     -1_500_000_000_000_000_000,
///     Scale::TT,
///     Scale::GPS,
/// );
/// s = BufStr::<64>::default();
/// write!(&mut s, "{}", dt).unwrap();
/// assert_eq!(s.as_str(), "[-1.5s TT>GPS]");
/// ```
impl fmt::Display for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const PREC: usize = 18;

        let total = self.to_attos();
        let is_negative = total < 0;
        let abs_attos = if is_negative {
            total.wrapping_neg() as u128
        } else {
            total as u128
        };

        f.write_str("[")?;

        if is_negative {
            f.write_str("-")?;
        } else if f.sign_plus() {
            f.write_str("+")?;
        }

        let attos_per_sec = ATTOS_PER_SEC as u128;
        let whole_seconds = abs_attos / attos_per_sec;
        let fractional_attos = abs_attos % attos_per_sec;

        write!(f, "{}", whole_seconds)?;

        if fractional_attos > 0 {
            f.write_str(".")?;

            let mut digits = [0u8; 18];
            let mut n = fractional_attos;

            for i in (0..PREC).rev() {
                digits[i] = (n % 10) as u8;
                n /= 10;
            }

            let last = digits[..PREC].iter().rposition(|&d| d != 0).unwrap_or(0);

            for &d in &digits[..=last] {
                write!(f, "{}", d)?;
            }
        }

        write!(f, "s {}>{}]", self.scale.abbrev(), self.target.abbrev())
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

    /// Serializes this `Dt` into a fixed 19-byte little-endian buffer using the
    /// `attos: i128` + `scale: Scale` + `target: Scale` representation.
    ///
    /// ## Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `[1..17]`: total attoseconds as little-endian `i128`
    /// - Byte `17`: scale as `u8`
    /// - Byte `18`: target as `u8`
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;
        buf[1..17].copy_from_slice(&self.attos.to_le_bytes());
        buf[17] = self.scale as u8;
        buf[18] = self.target as u8;
        buf
    }

    /// Deserializes a [`Dt`] from exactly 19 bytes of wire data.
    ///
    /// ## Errors
    ///
    /// Returns `None` only when:
    /// - `bytes` is not exactly [`WIRE_SIZE`](Self::WIRE_SIZE) long, or
    /// - the version byte is not [`WIRE_VERSION`](Self::WIRE_VERSION).
    ///
    /// Scale and target never cause failure: each is passed through
    /// [`Scale::from_u8`], which substitutes [`Scale::Custom`] for any
    /// unrecognized value.
    ///
    /// ## Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `[1..17]`: total attoseconds as little-endian `i128`
    /// - Byte `17`: scale as `u8`
    /// - Byte `18`: target as `u8`
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
