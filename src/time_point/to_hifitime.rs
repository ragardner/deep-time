use crate::TimePoint;
use hifitime::{Duration, Epoch};

impl TimePoint {
    /// Converts this `TimePoint` to a `hifitime::Epoch` (TAI scale).
    ///
    /// Round-trips perfectly with `from_hifitime` thanks to the
    /// runtime-computed offset that matches hifitime's calendar math.
    pub fn to_hifitime(self) -> Epoch {
        let tai = self.to_tai();
        let ns_since_zero = tai.total_attos() / 1_000_000_000;

        let j1900 = Epoch::from_gregorian_tai(1900, 1, 1, 12, 0, 0, 0);
        let j2000 = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);
        let offset_ns = j2000.to_tai_duration().total_nanoseconds()
            - j1900.to_tai_duration().total_nanoseconds();

        let ns_since_j1900 = ns_since_zero + offset_ns;

        let dur = Duration::from_total_nanoseconds(ns_since_j1900);
        let (centuries, nanos) = dur.to_parts();

        Epoch::from_tai_parts(centuries, nanos)
    }
}
