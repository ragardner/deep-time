//! Ergonomic timestamp constructors for common epochs (optional import).
//!
//! ```
//! use deep_time_core::{
//!     GPSTimestamp, J2000Timestamp, TAITimestamp, UTCTimestamp, UnixTimestamp,
//! };
//!
//! let p1 = 1_741_234_567_i128.unix_seconds();
//! let p2 = 123_456_789_i128.j2000_seconds();
//! let p3 = 1_741_234_567_i128.utc_seconds();
//! let p4 = 1_234_567_890_i128.gps_seconds();
//! ```

use crate::{C_SQUARED, ClockDrift, ClockType, Delta, Position, Timestamp, Velocity};

// ──────────────────────────────────────────────────────────────
// Traits
// ──────────────────────────────────────────────────────────────

pub trait UnixTimestamp: Copy + Sized {
    fn unix_seconds(self) -> Timestamp;
    fn unix_milliseconds(self) -> Timestamp;
    fn unix_microseconds(self) -> Timestamp;
    fn unix_nanoseconds(self) -> Timestamp;
}

pub trait TAITimestamp: Copy + Sized {
    fn tai_seconds(self) -> Timestamp;
    fn tai_milliseconds(self) -> Timestamp;
    fn tai_microseconds(self) -> Timestamp;
    fn tai_nanoseconds(self) -> Timestamp;
}

pub trait J2000Timestamp: Copy + Sized {
    fn j2000_seconds(self) -> Timestamp;
    fn j2000_milliseconds(self) -> Timestamp;
    fn j2000_microseconds(self) -> Timestamp;
    fn j2000_nanoseconds(self) -> Timestamp;
}

pub trait UTCTimestamp: Copy + Sized {
    fn utc_seconds(self) -> Timestamp;
    fn utc_milliseconds(self) -> Timestamp;
    fn utc_microseconds(self) -> Timestamp;
    fn utc_nanoseconds(self) -> Timestamp;
}

pub trait GPSTimestamp: Copy + Sized {
    fn gps_seconds(self) -> Timestamp;
    fn gps_milliseconds(self) -> Timestamp;
    fn gps_microseconds(self) -> Timestamp;
    fn gps_nanoseconds(self) -> Timestamp;
}

// ──────────────────────────────────────────────────────────────
// Macro for traits that use existing Timestamp::from_* helpers
// ──────────────────────────────────────────────────────────────

macro_rules! impl_timestamp_trait {
    ($trait:ident,
     $sec:ident, $ms:ident, $us:ident, $ns:ident,
     $ctor_sec:ident, $ctor_ms:ident, $ctor_us:ident, $ctor_ns:ident) => {
        impl $trait for i8 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for i16 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for i32 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for i64 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for i128 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self)
            }
        }
        impl $trait for u8 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for u16 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for u32 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for u64 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for u128 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
    };
}

impl_timestamp_trait!(
    UnixTimestamp,
    unix_seconds,
    unix_milliseconds,
    unix_microseconds,
    unix_nanoseconds,
    from_unix_seconds,
    from_unix_milliseconds,
    from_unix_microseconds,
    from_unix_nanoseconds
);

impl_timestamp_trait!(
    TAITimestamp,
    tai_seconds,
    tai_milliseconds,
    tai_microseconds,
    tai_nanoseconds,
    from_tai_sec,
    from_tai_ms,
    from_tai_us,
    from_tai_ns
);

impl_timestamp_trait!(
    J2000Timestamp,
    j2000_seconds,
    j2000_milliseconds,
    j2000_microseconds,
    j2000_nanoseconds,
    from_tai_sec,
    from_tai_ms,
    from_tai_us,
    from_tai_ns
);

impl_timestamp_trait!(
    UTCTimestamp,
    utc_seconds,
    utc_milliseconds,
    utc_microseconds,
    utc_nanoseconds,
    from_utc_sec,
    from_utc_ms,
    from_utc_ns,
    from_utc_ns
);

// ──────────────────────────────────────────────────────────────
// GPS (uses its own clock_type constructor)
// ──────────────────────────────────────────────────────────────

macro_rules! impl_gps_timestamp {
    ($ty:ty) => {
        impl GPSTimestamp for $ty {
            #[inline(always)]
            fn gps_seconds(self) -> Timestamp {
                Timestamp::new(self as i128, 0, ClockType::GPST)
            }
            #[inline(always)]
            fn gps_milliseconds(self) -> Timestamp {
                Timestamp::new(0, 0, ClockType::GPST).add(Delta::from_ms(self as i128))
            }
            #[inline(always)]
            fn gps_microseconds(self) -> Timestamp {
                Timestamp::new(0, 0, ClockType::GPST).add(Delta::from_us(self as i128))
            }
            #[inline(always)]
            fn gps_nanoseconds(self) -> Timestamp {
                Timestamp::new(0, 0, ClockType::GPST).add(Delta::from_ns(self as i128))
            }
        }
    };
}

impl_gps_timestamp!(i8);
impl_gps_timestamp!(i16);
impl_gps_timestamp!(i32);
impl_gps_timestamp!(i64);
impl_gps_timestamp!(i128);
impl_gps_timestamp!(u8);
impl_gps_timestamp!(u16);
impl_gps_timestamp!(u32);
impl_gps_timestamp!(u64);
impl_gps_timestamp!(u128);

// ──────────────────────────────────────────────────────────────
// RelativisticTrajectory trait
// ──────────────────────────────────────────────────────────────

/// A trajectory or ephemeris capable of computing the accumulated **proper time** (τ)
/// along a coordinate-time path (typically TT or TDB).
///
/// Proper time is the time your spacecraft’s *onboard clock* actually experiences.
/// Because of Einstein’s relativity (speed + gravity), it slowly drifts from Earth clocks.
/// Implement this trait to tell the library “here’s my spacecraft’s path” — it will
/// calculate exactly how much time passed on the spacecraft’s clock.
///
/// The default implementation uses the same weak-field post-Newtonian approximation
/// already used everywhere else in the library (`ClockDrift::from_weak_field_approximation`).
pub trait RelativisticTrajectory {
    /// Returns the kinematic and gravitational state at coordinate time `t`.
    ///
    /// Tell the library where the spacecraft is, how fast it is moving,
    /// and how deep it is in the Sun’s gravity well at any given moment.
    ///
    /// The returned tuple contains:
    /// - `Position`: Cartesian position (meters, usually Sun-centered)
    /// - `Velocity`: Cartesian velocity (m/s)
    /// - `f64`: Newtonian gravitational potential `Φ` (**positive** convention: `Φ = +GM/r`)
    fn state_and_potential_at(&self, t: Timestamp) -> (Position, Velocity, f64);

    /// Returns the instantaneous proper-time rate `dτ/dt` at time `t`.
    ///
    /// This tells you how fast (or slow) the onboard clock is currently ticking
    /// compared to Earth coordinate time. A value of `0.999999999` means the clock
    /// is running slightly slow.
    fn proper_time_rate_at(&self, t: Timestamp) -> f64 {
        let (_, vel, phi) = self.state_and_potential_at(t);
        let drift = ClockDrift::from_weak_field_approximation(
            vel.norm_squared() / (2.0 * C_SQUARED),
            phi / C_SQUARED,
        );
        1.0 + drift.evaluate(Delta::ZERO).as_sec_f64()
    }

    /// Computes the proper-time interval Δτ between two coordinate times.
    ///
    /// Given any two moments on an Earth clock (start → end), this returns how much time
    /// actually passed on the spacecraft’s clock. It automatically accounts for relativistic
    /// effects using the same math NASA and ESA use for deep-space navigation.
    ///
    /// # Mathematics
    ///
    /// \[
    /// \Delta\tau \approx \Delta t + \int_{t_0}^{t_1} \delta(t)\,\mathrm{d}t
    /// \]
    ///
    /// where \(\delta(t) = -\frac{v^2}{2c^2} - \frac{\Phi}{c^2}\).
    ///
    /// Uses composite Simpson’s rule (high accuracy) when `num_samples > 2`
    /// (**even** values recommended: 10, 20, 50, …) or the trapezoidal rule
    /// for `num_samples ≤ 2`. Negative intervals are handled correctly.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time_core::{Timestamp, Delta, ClockType, Position, Velocity, RelativisticTrajectory};
    ///
    /// struct EarthOrbit; // example trajectory
    ///
    /// impl RelativisticTrajectory for EarthOrbit {
    ///     fn state_and_potential_at(&self, _t: Timestamp) -> (Position, Velocity, f64) {
    ///         let pos = Position::from_au(1.0, 0.0, 0.0);
    ///         let vel = Velocity::new(0.0, 29_780.0, 0.0); // ~29.78 km/s
    ///         let phi = 1.3271244e20 / 1.495978707e11;     // Φ☉ at 1 AU (positive!)
    ///         (pos, vel, phi)
    ///     }
    /// }
    ///
    /// let traj = EarthOrbit;
    /// let start = Timestamp::ZERO.to_clock_type(ClockType::TDB);
    /// let end   = start.add(Delta::from_sec_f64(365.25 * 86_400.0)); // 1 year later
    ///
    /// let dtau = traj.proper_time_interval(start, end, 50);
    /// let dt   = end.duration_since(start);
    ///
    /// // Spacecraft clock runs ~0.5 s slower per year in Earth orbit
    /// let difference = (dt - dtau).as_sec_f64();
    /// ```
    fn proper_time_interval(&self, start: Timestamp, end: Timestamp, num_samples: usize) -> Delta {
        let mut dt = end.duration_since(start);
        if dt.is_zero() {
            return Delta::ZERO;
        }

        // Forward interval for quadrature; sign restored at the end
        let sign = if dt.sec < 0 { -1.0 } else { 1.0 };
        if sign < 0.0 {
            dt = dt.neg();
        }

        let dt_sec = dt.as_sec_f64();

        if num_samples <= 2 {
            // Fast trapezoidal path (matches existing one-way relativistic delay code)
            let (_, vel0, phi0) = self.state_and_potential_at(start);
            let rate0 = ClockDrift::from_weak_field_approximation(
                vel0.norm_squared() / (2.0 * C_SQUARED),
                phi0 / C_SQUARED,
            )
            .evaluate(Delta::ZERO)
            .as_sec_f64();

            let (_, vel1, phi1) = self.state_and_potential_at(end);
            let rate1 = ClockDrift::from_weak_field_approximation(
                vel1.norm_squared() / (2.0 * C_SQUARED),
                phi1 / C_SQUARED,
            )
            .evaluate(Delta::ZERO)
            .as_sec_f64();

            let integral = 0.5 * (rate0 + rate1) * dt_sec;
            return Delta::from_sec_f64(sign * (dt_sec + integral));
        }

        // Simpson’s rule quadrature (high-order accuracy)
        let n = num_samples as f64;
        let h = dt_sec / n;
        let mut s = 0.0;

        for i in 0..=num_samples {
            let lambda = (i as f64) / n;
            let t_i = start.add(Delta::from_sec_f64(lambda * dt_sec));
            let (_, vel, phi) = self.state_and_potential_at(t_i);

            let rate = ClockDrift::from_weak_field_approximation(
                vel.norm_squared() / (2.0 * C_SQUARED),
                phi / C_SQUARED,
            )
            .evaluate(Delta::ZERO)
            .as_sec_f64();

            let coeff = if i == 0 || i == num_samples {
                1.0
            } else if i % 2 == 0 {
                2.0
            } else {
                4.0
            };
            s += coeff * rate;
        }

        let integral = (h / 3.0) * s;
        Delta::from_sec_f64(sign * (dt_sec + integral))
    }

    /// Returns the relativistic correction Δτ − Δt (how much the onboard clock
    /// has gained or lost relative to coordinate time).
    ///
    /// This is the number you usually care about for clock steering or navigation:
    /// “How many seconds has my spacecraft clock drifted behind/ahead of Earth time?”
    fn relativistic_correction(
        &self,
        start: Timestamp,
        end: Timestamp,
        num_samples: usize,
    ) -> Delta {
        let dtau = self.proper_time_interval(start, end, num_samples);
        let dt = end.duration_since(start);
        dtau.sub(dt)
    }
}
