/*!
# deep-time

A fully featured, high performance, no_std and no alloc Rust date and
time library with attosecond precision that provides astronomical and
civil timekeeping.

Functionality:
<https://github.com/ragardner/deep-time#overview>

Readme example:
<https://github.com/ragardner/deep-time#examples>

Additional examples:
<https://github.com/ragardner/deep-time#additional-examples>

The library's central time type is [`Dt`], a 32 byte struct that holds:

- An `i128` attoseconds count.
- A `scale` [`Scale`] field for its current time scale.
- A `target` [`Scale`] field for the time scale the object came from,
  and/or which time scale it should be converted to in various output
  functions.

[`Dt`] can act as an instant or duration.

While [`Dt`] can hold attoseconds counts from any epoch (start time),
the library's epoch for most functionality is 2000-01-01 noon on the
TAI time scale.

```
use deep_time::{Dt, Scale, from_ymd};

let dt = from_ymd!(2000, 1, 1; 12, on=Scale::TAI);
assert_eq!(dt, Dt::ZERO);

let dt = Dt::from_str_iso("2000-01-01 12:00 TAI").unwrap();
assert_eq!(dt, Dt::ZERO);
```

[`Dt`] handles massive datetimes and always with attosecond
resolution:

```
use deep_time::{Dt, Lang, Scale, from_ymd};

let mut dt = Dt::from_str_iso("292000000000-1-1").unwrap();
dt = dt.add_days(4);
let s = dt.to_str_lite("%Y-%m-%dT%H:%M:%S %L", Lang::En).unwrap();

assert_eq!(s.as_str(), "292000000000-01-05T00:00:00 UTC");

// negatives too
let dt = from_ymd!(-5000, 1, 1; 18, on=Scale::TAI);
assert_eq!(dt, Dt::parse("-5000-01-01T18:00:00 TAI").unwrap());

assert_eq!(dt.to_jd_f(), -105151.75);
assert_eq!(dt.to_mjd_f(), -2505152.25);
```

Once you have a [`Dt`] you can change its time scale:

```
use deep_time::{Dt, Scale, macros::from_jd_f};

let dt = from_jd_f!(2451545.0);
assert_eq!(dt.scale, Scale::TAI);

// leap seconds have been subtracted when going from TAI -> UTC
let utc = dt.to(Scale::UTC);
assert_eq!(utc.to_sec_f(), -32.0);
```

This crate has no default features.

The minimum Rust version is `1.90` and minimum Rust edition is `2024`.

This is mainly due to some `const` functionality that only became stable
recently.

To add deep-time to your Rust project with the parse and timezone
features, go to your project folder and run this terminal command:

```text
cargo add deep-time --features "parse,jiff-tz"
```

List of features you can enable:
<https://github.com/ragardner/deep-time#feature-flags>
*/

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::all))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

/*
uncomment this fn and run
cargo test --release --features "parse"

if it builds then std is being silently pulled in

update to use something that requires std if
.round() can work without std in the future
*/

// #[allow(dead_code)]
// fn check_if_std_silently_pulled_in() {
//     let x: f64 = 0.5;
//     let _: f64 = x.round();
// }

// ──────────────────────────────────────────────────────────────
// Optional panic handler (opt-in via feature)
// ──────────────────────────────────────────────────────────────
#[cfg(all(feature = "panic-handler", not(feature = "std"), not(test)))]
use core::panic::PanicInfo;

#[cfg(all(feature = "panic-handler", not(feature = "std"), not(test)))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Uses spin_loop() for better power characteristics than plain loop{}
    loop {
        core::hint::spin_loop();
    }
}

/// Alias for f64, maybe upgrade one day
pub type Real = f64;

/// Convert a number to the crates [`Real`] type (f64).
///
/// Equivalent to `n as f64`.
#[macro_export]
macro_rules! f {
    ($x:expr) => {
        $x as $crate::Real
    };
}

/// Safe Euclidean division.
/// Returns `default` if `rhs == 0` or if `lhs == i128::MIN && rhs == -1`.
macro_rules! safe_div_euc {
    ($lhs:expr, $rhs:expr, $default:expr) => {{
        match ($lhs).checked_div_euclid($rhs) {
            Some(q) => q,
            None => $default,
        }
    }};
}

/// Safe Euclidean remainder.
/// Returns `$default` if `rhs == 0` or if `lhs == Self::MIN && rhs == -1`.
macro_rules! safe_rem_euc {
    ($lhs:expr, $rhs:expr, $default:expr) => {{
        match ($lhs).checked_rem_euclid($rhs) {
            Some(r) => r,
            None => $default,
        }
    }};
}

/// Turns a list of string literals into an array of `&'static [u8]`.
macro_rules! byte_arrays {
    ( $( $s:literal ),+ $(,)? ) => {
        [ $( $s.as_bytes() ),+ ]
    };
}

// _________________________________________
// MOD
// _________________________________________
mod an_err;
mod dt;
mod error;
mod lite_str;
mod locale;
mod scale;
mod strtime;
mod time_range;
mod ymdhms;

#[cfg(feature = "parse")]
mod alloc_parse;

#[cfg(feature = "physics")]
mod physics;

// _________________________________________
// PUB MOD
// _________________________________________
pub mod civil_parts;
pub mod consts;
pub mod macros;
pub mod math;
pub mod tz;
pub mod utc;

#[cfg(feature = "eop")]
pub mod eop;

#[cfg(feature = "sidereal")]
pub mod sidereal;

// _________________________________________
// CRATE USE
// _________________________________________
pub(crate) use civil_parts::*;
pub(crate) use consts::*;
pub(crate) use locale::*;
#[allow(unused_imports)]
pub(crate) use math::{
    atan2::atan2,
    cos::cos,
    div::rem_euclid_f,
    floor::floor_f,
    log::log,
    sin::sin,
    sqrt::{hypot, sqrt},
};
pub(crate) use strtime::*;

#[cfg(feature = "parse")]
pub(crate) use alloc_parse::{
    alloc_consts::*, date::*, date_classification::*, duration::*, helpers::*, parse_date::*,
    types::*,
};

#[cfg(feature = "parse")]
pub(crate) use locale::{lang_data::*, lang_map::*};

// _________________________________________
// PUB USE
// _________________________________________
pub use an_err::AnErr;
pub use dt::Dt;
pub use dt::lunar;
pub use dt::numbers_traits::TraitsTime;
pub use error::{DtErr, DtErrKind};
pub use lite_str::LiteStr;
pub use locale::Lang;
pub use scale::Scale;
pub use strtime::StrPTimeFmt;
pub use time_range::{Every, TimeRange};
pub use ymdhms::YmdHms;

#[cfg(feature = "tdb-hi")]
pub use dt::tdb_hi;

#[cfg(feature = "parse")]
pub use alloc_parse::types::{Mode, Order, ParseCfg};

#[cfg(feature = "mars")]
pub use dt::mars;

#[cfg(feature = "sidereal")]
#[doc(hidden)]
pub use sidereal::Sidereal;

#[cfg(feature = "physics")]
pub use physics::{
    drift::Drift, observer::Observer, position::Position, spacetime::Spacetime, velocity::Velocity,
};
