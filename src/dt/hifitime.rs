use crate::{Dt, Scale};
use hifitime::{Duration, Epoch};

impl Dt {
    /// Converts this [`Dt`] to a [`hifitime::Epoch`] (TAI scale).
    ///
    /// Round-trips with [`Dt::from_hifitime`].
    pub fn to_hifitime_epoch(&self) -> Epoch {
        let nanos = self.to_ns();

        let j1900 = Epoch::from_gregorian_tai(1900, 1, 1, 12, 0, 0, 0);
        let j2000 = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);
        let offset_ns = j2000.to_tai_duration().total_nanoseconds()
            - j1900.to_tai_duration().total_nanoseconds();

        let ns_since_j1900 = nanos + offset_ns;

        let dur = Duration::from_total_nanoseconds(ns_since_j1900);
        let (centuries, nanos) = dur.to_parts();

        Epoch::from_tai_parts(centuries, nanos)
    }

    /// Creates a [`Dt`] from a [`hifitime::Epoch`].
    ///
    /// - The conversion is exact (within hifitime's nanosecond precision).
    /// - Uses a runtime-computed offset so it always matches whatever
    ///   calendar math hifitime uses (including negative years).
    pub fn from_hifitime_epoch(epoch: Epoch) -> Self {
        let ns_since_j1900 = epoch.to_tai_duration().total_nanoseconds();

        let j1900 = Epoch::from_gregorian_tai(1900, 1, 1, 12, 0, 0, 0);
        let j2000 = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);
        let offset_ns = j2000.to_tai_duration().total_nanoseconds()
            - j1900.to_tai_duration().total_nanoseconds();

        let ns_since_zero_tai = ns_since_j1900 - offset_ns;
        Self::from_ns(ns_since_zero_tai, Scale::TAI)
    }

    /// Converts this [`Dt`] to a [`hifitime::Duration`] (nanosecond precision).
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion is exact up to the nanosecond (128-bit integer arithmetic).
    /// - Internally uses [`hifitime::Duration::from_total_nanoseconds`], which
    ///   automatically normalizes centuries/nanoseconds and saturates at
    ///   [`Duration::MAX`] / [`Duration::MIN`] if outside hifitime's range
    ///   (±32,768 centuries).
    #[inline]
    pub fn to_hifitime_duration(&self) -> Duration {
        Duration::from_total_nanoseconds(self.to_attos() / 1_000_000_000i128)
    }

    /// Creates a [`Dt`] from a [`hifitime::Duration`] (nanosecond precision).
    ///
    /// Inverse of [`Dt::to_hifitime_duration`].
    #[inline]
    pub fn from_hifitime_duration(dur: Duration) -> Self {
        Self::from_ns(dur.total_nanoseconds(), Scale::TAI)
    }
}
