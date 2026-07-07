# deep-time

A fully featured and high performance **Rust date and time library** with attosecond precision that provides **astronomical** and **civil** timekeeping.

[![docs.rs](https://img.shields.io/docsrs/deep-time)](https://docs.rs/deep-time)
[![Crates.io](https://img.shields.io/crates/v/deep-time)](https://crates.io/crates/deep-time)
[![Crates.io MSRV](https://img.shields.io/crates/msrv/deep-time)](https://crates.io/crates/deep-time)
[![Edition](https://img.shields.io/badge/edition-2024-blueviolet)](https://doc.rust-lang.org/edition-guide/editions/)
[![License](https://img.shields.io/crates/l/deep-time)](https://github.com/ragardner/deep-time/blob/main/LICENSE)

### Overview

- Auto-parsers for [datetimes](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.from_str_parse) and [durations](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.from_str_duration) that handle thousands of formats, relative dates and multiple languages, requires the `parse` feature
- No std, no alloc, and wide-spread [const fn](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.from_ymd)
- [Extensively validated](https://github.com/ragardner/deep-time/tree/main/tests) against outputs from **Astropy**, **Jiff**, and other libraries and sources
- Fast [ISO](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.from_str_iso) parser
- [Time scales](https://docs.rs/deep-time/latest/deep_time/enum.Scale.html) e.g. UTC with leap seconds support, including historical, TT, TAI, TDB, NAIF ET, LTC, GPS, etc. An optional feature `tdb_hi` can be enabled which provides the ERFA TDB model
- [Strptime](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.from_str)
- [Strftime](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_str) (multi-language day and month names available)
- First class [timezone](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_str_in_tz) support provided by the Rust library [jiff](https://github.com/BurntSushi/jiff) enabled with the `jiff-tz` feature
- To and from all kinds of inputs and outputs, functions mostly prefixed with `to` and `from`, available on the library's types, see the main time types functions: [Dt](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html). Including [JD](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_jd_f), [MJD](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_mjd_f), [Unix](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_unix), [NTP](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_ntp), etc.
- [Calendar aware](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.add_days) and, with the `jiff-tz` feature, [timezone aware](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.add_days_tz) math
- To and from [jiff](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_jiff_timestamp), [chrono](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_chrono_datetime_utc), and [hifitime](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_hifitime_epoch) types
- No-alloc [string return type](https://docs.rs/deep-time/latest/deep_time/struct.LiteStr.html)
- Const fn [libm math](https://docs.rs/deep-time/latest/deep_time/math/index.html) functions
- Safe, saturating arithmetic throughout
- **No** `unsafe` in the library - [`#![forbid(unsafe_code)]`](https://github.com/ragardner/deep-time/blob/main/src/lib.rs)
- [Lunar](https://docs.rs/deep-time/latest/deep_time/lunar/index.html) and [Mars](https://docs.rs/deep-time/latest/deep_time/mars/index.html) modules
- [Sidereal time](https://docs.rs/deep-time/latest/deep_time/sidereal/struct.Sidereal.html) with a const fn implementation of ERFA Equation of the Origins / Equinoxes
- [UT1 and EOP](https://docs.rs/deep-time/latest/deep_time/eop/index.html)
- [Light-time (Shapiro delay, etc.)](https://docs.rs/deep-time/latest/deep_time/struct.Observer.html), requires the `physics` feature
- [Proper time along trajectories](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.proper_time_from_states), requires the `physics` feature
- Relativity: [Drift](https://docs.rs/deep-time/latest/deep_time/struct.Drift.html), [Spacetime](https://docs.rs/deep-time/latest/deep_time/struct.Spacetime.html), [Position](https://docs.rs/deep-time/latest/deep_time/struct.Position.html), and [Velocity](https://docs.rs/deep-time/latest/deep_time/struct.Velocity.html) — see [docs/relativity.md](docs/relativity.md) for the underlying model. Requires the `physics` feature.
- CCSDS [CUC](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_ccsds_cuc), [CDS](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_ccsds_cds), and [CCS](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_ccsds_ccs)
- Binary size is mainly controlled through feature gating

### Examples

```rust
use deep_time::{Dt, DtErr, Lang, LiteStr, ParseCfg, Scale, YmdHms};

fn main() -> Result<(), DtErr> {
    // ============================================
    // Parsing
    // ============================================

    // Smart auto-parsing (multi-language + timezone)
    let cfg = ParseCfg {
        lang: Lang::Fr,
        ..Default::default()
    };
    let dt = Dt::from_str_parse("15 août 2024 à 14:30 [Europe/Paris]", &cfg)?;
    let s = dt.to_str_rfc9557("Europe/Paris")?;
    assert_eq!("2024-08-15T14:30:00+02:00[Europe/Paris]", s);

    // or with .parse
    let dt: Dt = "1 jan 2000 07:00 [America/New_York] TAI".parse()?; // noon
    assert_eq!(Dt::ZERO, dt);

    // Relative dates are also supported
    let ref_time = Dt::from_ymd(2026, 6, 16, Scale::UTC, 12, 0, 0, 0);
    let en_cfg = ParseCfg {
        ref_time: Some(ref_time),
        ..Default::default()
    };

    let dt = Dt::from_str_parse("2 days from now at 9am", &en_cfg)?;
    assert_eq!(dt, Dt::from_ymd(2026, 6, 18, Scale::UTC, 9, 0, 0, 0));

    let dt = Dt::from_str_parse("next Monday at 14:00", &en_cfg)?;
    assert_eq!(dt, Dt::from_ymd(2026, 6, 22, Scale::UTC, 14, 0, 0, 0));

    // Relative dates use Dt::now if the `std` feature is enabled and no
    // ref_time is provided in the ParseCfg
    let _ = Dt::from_str_parse("next Monday at 14:00", &ParseCfg::DEFAULT)?;

    // Fast ISO parsing with time scale and no alloc output
    let dt = Dt::from_str_iso("2000-01-01T12:00:00 TAI")?;
    let lite_str: LiteStr<512> = dt.to_str_lite_iso8601()?;
    assert_eq!("2000-01-01T12:00:00+00:00", lite_str.as_str());

    // ============================================
    // Formatting
    // ============================================

    let s = dt.to_str_in_tz("%A, %d %B %Y %I:%M%P", "America/New_York", Lang::En)?;
    assert_eq!("Saturday, 01 January 2000 07:00am", s);

    let s = dt.to_str_in_tz("%A, %-d de %B de %Y %H:%M", "America/New_York", Lang::Es)?;
    assert_eq!("Sábado, 1 de enero de 2000 07:00", s);

    // ============================================
    // Duration parsing
    // ============================================

    let span: Dt = Dt::from_str_duration("3 days 12 hours", Lang::En)?;
    let dur = span.to_str_lite_media_duration();
    assert_eq!("3:12:00:00", dur.to_string());

    // ============================================
    // Time scale conversions + round-tripping
    // ============================================

    let dt = Dt::from_ymd(2000, 1, 1, Scale::TAI, 0, 0, 0, 123456789);
    let tt = dt.to(Scale::TT);
    let tdb = tt.to(Scale::TDB);
    let ltc = tdb.to(Scale::LTC);
    let utc = ltc.to(Scale::UTC);
    let tcl = utc.to(Scale::TCL);
    let tcg = tcl.to(Scale::TCG);
    let tai = tcg.to_tai();

    // round trips work for pretty much everything except UTCHist
    assert_eq!(dt, tai);
    let ymd: YmdHms = tai.to_ymd();
    assert_eq!(ymd.attos(), 123456789);

    // ============================================
    // Other conversions
    // ============================================

    // unix
    let dt = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);
    let unix = dt.to_unix().to_sec_f();
    assert_eq!(unix, 0.0);

    // or to milliseconds
    let unix: i128 = dt.add_ms(1000).to_unix().to_ms();
    assert_eq!(unix, 1000);

    // to and from jd
    let jd = Dt::ZERO.to_jd_f_raw();
    assert_eq!(2451545.0, jd);
    let dt = Dt::from_jd_f(jd, Scale::TAI);
    assert_eq!(0, dt.attos);

    // ============================================
    // Calendar math
    // ============================================

    // calendar math and negative year
    let dt = Dt::from_ymd(-2000, 1, 31, Scale::TAI, 12, 0, 0, 0);
    let ymd = dt.add_mo(1).to_ymd();
    assert_eq!(ymd.day(), 29);

    // Timezone-aware calendar math (respects DST transitions, requires jiff-tz feature)
    let dt = Dt::from_str_iso("2025-03-30T00:30:00Z")?; // Just before London DST start

    // Normal (naive) addition — ignores DST rules
    let normal = dt.add_hr(1);

    // Timezone-aware addition — correctly handles the transition
    let aware = dt.add_hr_tz(1, "Europe/London")?;

    println!("Normal: {}", normal.to_str_rfc9557("Europe/London")?);
    println!("Aware:  {}", aware.to_str_rfc9557("Europe/London")?);

    // ============================================
    // Leap seconds
    // ============================================

    // genuine leap second input round trips
    let dt: Dt = "2015-06-30T23:59:60".parse()?;
    let s = dt.to_str_iso8601();
    assert_eq!("2015-06-30T23:59:60+00:00", s);

    Ok(())
}
```

### Documentation

- [Library's main documentation page](https://docs.rs/deep-time/latest)
- [Changelog](https://github.com/ragardner/deep-time/blob/main/CHANGELOG.md)
- [The main time type](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html)
- [Time scales](https://docs.rs/deep-time/latest/deep_time/enum.Scale.html)

### Installation

- **This crate has no default features.**
- The minimum Rust version is `1.90` and minimum Rust edition is `2024`. This is mainly due to some
  `const` functionality that only became stable recently.
- Enable `parse` for the auto-parsers and `jiff-tz` for timezone support and DST-aware calendar math.

For example, add this to your `Cargo.toml` in the `dependencies` section:

```toml
[dependencies]
deep-time = { version = "0.1", features = ["parse", "jiff-tz"] }
```

### Feature Flags

| Feature              | Description                                                                 | Requires     |
|----------------------|-----------------------------------------------------------------------------|--------------|
| `parse`              | Enables the auto-parsers (`from_str_parse`, `from_str_duration`, etc.)      | `alloc`      |
| `jiff-tz`            | Enables timezone-aware calendar math (`add_days_tz`, `add_hr_tz`, etc.) and `to_str_in_tz` | `std`       |
| `jiff-tz-bundle`     | Same as `jiff-tz` but bundles the full timezone database                  | `std`       |
| `jiff`               | Enables basic Jiff interop                                                | `alloc`     |
| `chrono`             | Enables Chrono interop                                                    | `alloc`     |
| `hifitime`           | Enables Hifitime interop                                                  | —           |
| `serde`              | Enables `Serialize` / `Deserialize` for `Dt` and other types              | `alloc`     |
| `js`                 | WebAssembly support (includes `serde` and JS bindings)                    | `std`       |
| `tsify`              | TypeScript definitions via `tsify` (for WASM)                             | `js`        |
| `std`                | Enables `std` functionality including `Dt::now()` and file handling       | —           |
| `alloc`              | Enables allocation (required for parsing and some conversions)            | —           |
| `es` / `de` / `fr`   | Language support, parsing different languages requires alloc, formatting does not | —           |
| `euro`               | Enables all European languages                                            |             |
| `lang`               | Enables all languages                                                     | `euro`      |
| `panic-handler`      | Provides an optional simple `#[panic_handler]` for `no_std` environments  | `no_std`    |
| `defmt`              | Enables `defmt::Format` trait implementations the main types. Intended for use with the `defmt` logging framework on embedded systems. | — |
| `wire`               | Enables wire format (serialization) support                               | —           |
| `tdb_hi`             | Replaces the fast TDB and TCB conversions with the full ERFA TDB model    | —           |
| `physics`            | Enables relativistic physics support (`Drift`, `Spacetime`, `Position`, `Velocity`, `Observer`, light-time, etc.) | —           |
| `mars`               | Enables Mars time support (`to_msd`, `to_mars_ls`, etc.)                  | —           |
| `sidereal`           | Enables sidereal time support                                             | —           |
| `eop`                | Enables Earth Orientation Parameters (UT1, etc.)                          | `alloc`     |
| `locale`             | Enables system locale detection                                           | `std`       |

#### Optional No-Alloc Panic Handler

`deep-time` supports `no_std` + `no_alloc` environments. When targeting bare-metal or embedded systems, you can enable a minimal panic handler:

```toml
[dependencies]
deep-time = { version = "0.1", features = ["panic-handler"] }
```

This provides a simple `#[panic_handler]` that uses `core::hint::spin_loop()` (more power-efficient than a plain `loop {}`).

You only need this if you are building a binary crate in a `no_std` environment without your own panic handler.

#### Notes

- The fast ISO 8601 parser (`from_str_iso`) works **without** the `parse` feature.
- Multi-language **parsing** requires the `parse` feature, but multi-language **formatting** works without it.
- The `.parse()` implementation on `Dt` automatically chooses between the full parser and the ISO parser depending on enabled features.

### Performance

Benchmarks were measured on an AMD Ryzen 7 7800X3D.

#### Parsing and Formatting

| deep-time vs jiff                                        | Time      | vs Jiff 0.2.31 |
|----------------------------------------------------------|-----------|----------------|
| `Parts::from_str_iso` vs `DateTime::parse`               | 19.4 ns   | 29.2% faster   |
| `Parts::from_str` vs `BrokenDownTime::parse`             | 33.4 ns   | 13.9% faster   |
| `Dt::from_str` vs `BrokenDownTime::parse`+`to_zoned`     | 185 ns    | 15.8% slower   |
| `Dt::to_str_lite` vs `DateTime::strftime`+`.to_string`   | 77.4 ns   | 28.2% slower   |
| `Dt::to_str` vs `DateTime::strftime`+`.to_string`        | 88.7 ns   | 46.8% slower   |
| `Dt::from_str_parse`                                     | 553 ns    | —              |

#### Time Scale Conversions

| Conversion       | deep-time     | hifitime 4.3  | Relative Performance      |
|------------------|---------------|---------------|---------------------------|
| TAI → UTC        | 9.6 ns        | 33.8 ns       | 3.5× faster               |
| UTC → TAI        | 12.7 ns       | 32.9 ns       | 2.6× faster               |
| TAI → TDB        | 134 ns        | 90.7 ns       | 1.5× slower               |
| TDB → TAI        | 598 ns        | 26.8 ns       | 22.3× slower              |
| GPS conversion   | 20.7 ns       | 6.4 ns        | 3.2× slower               |
| GPS week + TOW   | 28.2 ns       | 7.0 ns        | 4.0× slower               |

The tests were run with:

```sh
cargo test --release --features "parse hifitime std jiff-tz perf-tests" -- --nocapture perf_tests
```

### Bundled Files

This library bundles some data relevant to, for example, time scale conversions. While every effort will be made to keep the library up to date, perhaps some users will want to know how to re-generate or update certain files and then re-compile.

#### Leap Seconds

The latest leap seconds table is bundled as a `.rs` file. A runtime file can be parsed and loaded for time scale conversions, e.g.

- [Dt::leap_sec_list_from_file](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.leap_sec_list_from_file)
- [Dt::to_utc_from_tai_using_list](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_utc_from_tai_using_list)
- [Dt::to_tai_from_utc_using_list](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.to_tai_from_utc_using_list)

If for whatever reason you need to update the library's bundled leap seconds file and re-compile, follow these steps:

1. Download the desired leap seconds file, for example from [https://data.iana.org/time-zones/data/leap-seconds.list](https://data.iana.org/time-zones/data/leap-seconds.list)
2. Place the downloaded file in the library, with the following location and filename: `deep-time/tests/assets/leap-seconds.list.txt`
3. Then with a terminal open in the library run the command: `cargo gen-leap-seconds`
4. This should overwrite the file `src/utc/leap_seconds_list.rs` using the data
5. Re-compile the library