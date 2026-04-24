use crate::{TimeSpan, TimePoint};
use jiff::Timestamp;

impl TimePoint {
    /// Creates a `TimePoint` from a `jiff::Timestamp`.
    ///
    /// This is the exact reverse of [`TimePoint::to_jiff_timestamp`].
    ///
    /// `jiff::Timestamp` is the primary absolute instant type in the Jiff
    /// ecosystem (broadly convertible to `Zoned`, civil datetimes, etc.).
    ///
    /// - The resulting `TimePoint` is expressed in the TAI clock type
    ///   (the library's canonical internal scale).
    /// - Sub-nanosecond attoseconds are set to zero.
    /// - If the timestamp is outside the range representable as an `i64`
    ///   number of nanoseconds since the Unix epoch, it is clamped to
    ///   exactly `i64::MAX` / `i64::MIN` nanoseconds.
    pub fn from_jiff_timestamp(ts: Timestamp) -> Self {
        let nanos = ts.as_nanosecond();

        let ns = if nanos > i64::MAX as i128 {
            i64::MAX
        } else if nanos < i64::MIN as i128 {
            i64::MIN
        } else {
            nanos as i64
        };

        TimePoint::UNIX_EPOCH_TAI.add(TimeSpan::from_ns(ns))
    }
}
