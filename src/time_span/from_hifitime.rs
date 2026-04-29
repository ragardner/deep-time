use crate::TimeSpan;
use hifitime::{Duration, Epoch};

impl TimeSpan {
    /// Creates a `TimeSpan` from a `hifitime::Duration` (nanosecond precision).
    ///
    /// This is the **exact reverse** of [`TimeSpan::to_hifitime_duration`].
    #[inline(always)]
    pub fn from_hifitime_duration(dur: Duration) -> Self {
        Self::from_ns(dur.total_nanoseconds())
    }

    /// Creates a `TimeSpan` from a `hifitime::Epoch` by taking its **raw internal `duration`**.
    ///
    /// This **respects whatever reference epoch the `hifitime::Epoch` has attached to it**
    /// (i.e. the duration is relative to the reference point of the Epoch's `time_scale`
    /// — J1900 for TAI/UTC, 1980-01-06 for GPST, J2000 for TT/TDB, etc.).
    #[inline]
    pub fn from_hifitime_epoch(epoch: Epoch) -> Self {
        Self::from_hifitime_duration(epoch.duration)
    }
}
