# deep-time

A fully featured and high performance attosecond precision **Rust** date and time library that aims to blend astronomical and civil timekeeping.

[![docs.rs](https://img.shields.io/docsrs/deep-time)](https://docs.rs/deep-time)
[![Crates.io](https://img.shields.io/crates/v/deep-time)](https://crates.io/crates/deep-time)
[![License](https://img.shields.io/crates/l/deep-time)](https://github.com/ragardner/deep-time/blob/main/LICENSE)

### Overview

A non-exhaustive list of functionality:

- No std, no alloc, and wide-spread [const fn](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_ymd).
- [Extensively validated](https://github.com/ragardner/deep-time/tree/main/tests) against outputs from **Astropy**, **Jiff**, and other libraries and sources
- Powerful and fast auto-parsers for [datetimes](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_str_parse) and [durations](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_str_duration) that handle thousands of formats and multiple languages, requires the `parse` feature
- Fast [ISO](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_str_iso) parser
- [Time scales](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/enum.Scale.html) e.g. UTC with full leap second support, including historical, TT, TAI, TDB, LTC, GPS, etc.
- [Strptime](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_str)
- [Strftime](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_str) (multi-language day and month names available)
- First class [timezone](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_str_in_tz) support provided by the Rust library [jiff](https://github.com/BurntSushi/jiff) enabled with the `jiff-tz` feature.
- To and from all kinds of inputs and outputs, functions mostly prefixed with `to` and `from`, available on the library's types, see the main time types functions: [Dt](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html). Including [JD](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_jd_f), [MJD](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_mjd_f), [Unix](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_unix), [NTP](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_ntp), etc.
- [Calendar aware](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.add_days) and, with the `jiff-tz` feature, [timezone aware](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.add_days_tz) math
- To and from [jiff](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_jiff_timestamp), [chrono](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_chrono_datetime_utc), and [hifitime](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_hifitime_epoch) types
- No-alloc [error type](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.AnErr.html) with rich tracing
- No-alloc [string return type](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.LiteStr.html)
- Const fn [libm math](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/math/index.html) functions
- Safe, saturating arithmetic throughout
- [Lunar](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/lunar/index.html) and [Mars](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/mars/index.html) modules
- [Sidereal time](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/sidereal/struct.Sidereal.html) with a rust implementation of ERFA Equation of the Origins / Equinoxes
- [UT1 and EOP](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/eop/index.html)
- [Light-time (Shapiro delay, etc.)](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.ObserverState.html)
- [Proper time along trajectories](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.proper_time_from_states)
- Relativity: [Drift](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Drift.html), [Spacetime](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Spacetime.html), [Position](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Position.html), and [Velocity](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Velocity.html)
- CCSDS [CUC](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_ccsds_cuc), [CDS](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_ccsds_cds), and [CCS](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_ccsds_ccs)