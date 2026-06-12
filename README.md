# deep-time

A full featured and high performance attosecond precision **Rust** date and time library that aims to blend astronomical and civil time.

[![docs.rs](https://img.shields.io/docsrs/deep-time)](https://docs.rs/deep-time)
[![Crates.io](https://img.shields.io/crates/v/deep-time)](https://crates.io/crates/deep-time)
[![License](https://img.shields.io/crates/l/deep-time)](https://github.com/ragardner/deep-time/blob/main/LICENSE)

### Overview

A non-exhaustive list of functionality:

- No std, no alloc, and wide-spread [const fn](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_ymd).
- [Extensively validated](https://github.com/ragardner/deep-time/tree/main/tests) against outputs from **Astropy**, **Jiff**, and other libraries and sources.
- Powerful and fast multi-language [datetime](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_str_parse) and [duration](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_str_duration) auto-parsers
- Even faster [ISO parser](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_str_iso)
- Time scales, e.g. UTC with full leap second support, including historical, TT, TAI, TDB, etc.
- [Strptime](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.from_str)
- [Strftime](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_str) (multi-language day and month names available)
- First class timezone handling provided by the Rust library [jiff](https://github.com/BurntSushi/jiff) enabled with the `jiff-tz` feature.
- To and from all kinds of inputs and outputs, functions mostly prefixed with `to` and `from`, available on the library's types, see the main time types functions: [Dt](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html)
- [Calendar aware](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.add_days) and, with the `jiff-tz` feature, [timezone aware](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.add_days_tz) math
- To and from types from [jiff](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_jiff_timestamp), [chrono](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_chrono_datetime_utc), and [hifitime](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.to_hifitime_epoch)
- [Sidereal time](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/sidereal/struct.Sidereal.html)
- [UT1 and EOP](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/eop/index.html)
- [Light-time (Shapiro delay, etc.)](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.ObserverState.html)
- [Proper time along trajectories](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Dt.html#method.proper_time_from_states)
- No-alloc [error type](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.AnErr.html) with rich tracing and fixed width [string type](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.LiteStr.html)
- Relativity: [Drift](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Drift.html), [Spacetime](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Spacetime.html), [Position](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Position.html), and [Velocity](https://docs.rs/deep-time/0.1.0-beta.10/deep_time/struct.Velocity.html)