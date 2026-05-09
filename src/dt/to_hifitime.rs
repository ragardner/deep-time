use crate::Dt;
use hifitime::{Duration, Epoch};

impl Dt {
    /// Converts this `Dt` to a `hifitime::Epoch` (TAI scale).
    ///
    /// Round-trips perfectly with `from_hifitime` thanks to the
    /// runtime-computed offset that matches hifitime's calendar math.
    pub fn to_hifitime(self) -> Epoch {
        let ns_since_zero = self.to_attos() / 1_000_000_000;

        let j1900 = Epoch::from_gregorian_tai(1900, 1, 1, 12, 0, 0, 0);
        let j2000 = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);
        let offset_ns = j2000.to_tai_duration().total_nanoseconds()
            - j1900.to_tai_duration().total_nanoseconds();

        let ns_since_j1900 = ns_since_zero + offset_ns;

        let dur = Duration::from_total_nanoseconds(ns_since_j1900);
        let (centuries, nanos) = dur.to_parts();

        Epoch::from_tai_parts(centuries, nanos)
    }

    /// Converts this `Span` to a [`hifitime::Duration`] (nanosecond precision).
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion is fully exact up to the nanosecond (128-bit integer arithmetic).
    /// - Internally uses [`hifitime::Duration::from_total_nanoseconds`], which
    ///   automatically normalizes centuries/nanoseconds and saturates at
    ///   [`Duration::MAX`] / [`Duration::MIN`] if outside hifitime's range
    ///   (±32,768 centuries).
    #[inline]
    pub fn to_hifitime_duration(self) -> Duration {
        Duration::from_total_nanoseconds(self.to_attos() / 1_000_000_000i128)
    }
}
