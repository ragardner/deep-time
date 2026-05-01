use crate::{TimePoint, TimeSpan};
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
    #[inline]
    pub fn from_jiff_timestamp(ts: Timestamp) -> Self {
        TimePoint::UNIX_EPOCH_TAI.add(TimeSpan::from_ns(ts.as_nanosecond()))
    }
}
