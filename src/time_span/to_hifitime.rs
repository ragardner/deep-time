use crate::TimeSpan;
use hifitime::{Duration, Epoch, TimeScale};

impl TimeSpan {
    /// Converts this `TimeSpan` to a [`hifitime::Duration`] (nanosecond precision).
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion is fully exact up to the nanosecond (128-bit integer arithmetic).
    /// - Internally uses [`hifitime::Duration::from_total_nanoseconds`], which
    ///   automatically normalizes centuries/nanoseconds and saturates at
    ///   [`Duration::MAX`] / [`Duration::MIN`] if outside hifitime's range
    ///   (±32,768 centuries).
    #[inline(always)]
    pub fn to_hifitime_duration(self) -> Duration {
        Duration::from_total_nanoseconds(self.total_attos() / 1_000_000_000i128)
    }

    /// Converts this `TimeSpan` to a [`hifitime::Epoch`] using the given `TimeScale`.
    ///
    /// The `TimeSpan` is interpreted as the **offset from the reference epoch**
    /// of the supplied `time_scale` (TAI reference for `TimeScale::TAI`,
    /// J2000 for `TT`/`TDB`/`ET`, GPS epoch for `GPS`, Unix for `UTC`, etc.).
    ///
    /// Uses the generic `hifitime::Epoch::from_duration` constructor — the most
    /// direct and flexible way to respect any attached epoch/timescale.
    #[inline(always)]
    pub fn to_hifitime_epoch(self, time_scale: TimeScale) -> Epoch {
        Epoch::from_duration(self.to_hifitime_duration(), time_scale)
    }
}
