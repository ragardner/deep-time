use crate::{Dt, Scale};
use hifitime::{Duration, Epoch};

impl Dt {
    /// Creates a `Dt` from a `hifitime::Epoch`.
    ///
    /// The conversion is exact (within hifitime's nanosecond precision).
    /// Uses a runtime-computed offset so it always matches whatever
    /// calendar math hifitime uses (including negative years).
    pub fn from_hifitime_epoch(epoch: Epoch) -> Self {
        let ns_since_j1900 = epoch.to_tai_duration().total_nanoseconds();

        let j1900 = Epoch::from_gregorian_tai(1900, 1, 1, 12, 0, 0, 0);
        let j2000 = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);
        let offset_ns = j2000.to_tai_duration().total_nanoseconds()
            - j1900.to_tai_duration().total_nanoseconds();

        let ns_since_zero_tai = ns_since_j1900 - offset_ns;
        Self::from_ns(ns_since_zero_tai, Scale::TAI)
    }

    /// Creates a `Dt` from a `hifitime::Duration` (nanosecond precision).
    ///
    /// This is the **exact reverse** of [`Dt::to_hifitime_duration`].
    #[inline]
    pub fn from_hifitime_duration(dur: Duration) -> Self {
        Self::from_ns(dur.total_nanoseconds(), Scale::TAI)
    }
}
