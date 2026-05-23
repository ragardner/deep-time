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
mod formatting;
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

/// ## [`Dt`] A high-precision instant or duration with attosecond resolution.
///
/// This is the core time type of the library. It represents both absolute
/// instants and durations using the same compact representation, making it
/// convenient anywhere precise time measurement or arithmetic is needed.
///
/// ## Representation
///
/// It stores:
///
/// - `sec: i64` — whole seconds (signed).
/// - `attos: u64` — fractional seconds in attoseconds (`0 ≤ attos < 10¹⁸`).
///     - These always push the `Dt` towards the positive.
///
/// This gives a resolution of one attosecond while supporting a range of
/// roughly ±292 billion years. An [`i128`] was considered but decided against
/// due to the difficulty of math without overflow.
///
/// There are many different ways to go to and from a `Dt` see the [`documentation`](../struct.Dt.html)
/// for the full list of methods.
///
/// It implements `Copy` and `Clone`. Optional derives for `serde` and
/// `tsify` are available behind the corresponding features.
///
/// ## Reference epoch and scales
///
/// When using the conversion functions [`Dt::to`] and [`Dt::from`] the
/// epoch for **all** time scales is [`Dt::ZERO`] 2000-01-01 noon.
///
/// Many convenience constructors and accessors exist for common epochs
/// (UNIX, GPS, Galileo, BeiDou, CXC, 1977 TT/TCG/TCB, etc.).
///
/// See the [`Scale`] documentation for the complete list of supported scales,
/// leap-second handling, historical UTC models, relativistic coordinate times
/// (TCG, TCB), and the lunar scales LTC / TCL (based on the LTE440 model).
///
/// ## Arithmetic and manipulation
///
/// `Dt` provides rich const-friendly arithmetic:
///
/// - Addition and subtraction of durations
/// - Multiplication and division by integers or `Real` (f64)
/// - `floor`, `ceil`, `round` to an arbitrary unit
/// - Many convenience increment/decrement methods (`add_1ns`, `sub_ms`, …)
/// - Signed difference via [`to_diff_raw`](Self::to_diff_raw)
///
/// Relativistic proper-time corrections and clock-drift models are supported
/// via [`convert_using_drift`](Self::convert_using_drift) and related methods.
///
/// ## Notes
///
/// - `Dt` does **not** store a time scale internally. The scale is always
///   an explicit parameter of conversion and construction methods.
/// - Leap-second handling follows the chosen `Scale` (UTC, UTCSpice, UTCSofa).
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct Dt {
    pub sec: i64,
    pub attos: u64,
}

impl Dt {
    /// Normalizes the representation so that the attosecond part lies in the range `[0, ATTOS_PER_SEC)`.
    #[inline]
    pub const fn carry_attos_mut(&mut self) -> &mut Self {
        if self.attos >= ATTOS_PER_SEC {
            self.sec = self.sec.saturating_add((self.attos / ATTOS_PER_SEC) as i64);
            self.attos %= ATTOS_PER_SEC;
        }
        self
    }

    /// Normalizes the representation so that the attosecond part lies in the range `[0, ATTOS_PER_SEC)`.
    #[inline]
    pub const fn carry_attos(&self) -> Self {
        if self.attos < ATTOS_PER_SEC {
            return *self;
        }
        Self {
            sec: self.sec.saturating_add((self.attos / ATTOS_PER_SEC) as i64),
            attos: self.attos % ATTOS_PER_SEC,
        }
    }
}

impl Default for Dt {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sec = self.sec;
        let attos = self.attos;

        // Default to nanosecond precision (9 digits) — most useful for everyday use
        let precision = f.precision().unwrap_or(9);

        // Respect the `+` sign when the user writes {:+}
        if f.sign_plus() && sec >= 0 {
            write!(f, "+")?;
        }

        write!(f, "{}", sec)?;

        if precision > 0 {
            let prec = precision.min(18);
            let scale = 10u64.pow(18 - prec as u32);
            let value = attos / scale;
            write!(f, ".{:0>width$}", value, width = prec)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dt")
            .field("sec", &self.sec)
            .field("attos", &self.attos)
            .finish()
    }
}
