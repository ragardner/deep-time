use crate::{Dt, TSpan};
use jiff::Timestamp;

impl Dt {
    /// Creates a `Dt` from a `jiff::Timestamp`.
    ///
    /// This is the exact reverse of [`Dt::to_jiff_timestamp`].
    ///
    /// `jiff::Timestamp` is the primary absolute instant type in the Jiff
    /// ecosystem (broadly convertible to `Zoned`, civil datetimes, etc.).
    ///
    /// - The resulting `Dt` is expressed in the TAI scale
    ///   (the library's canonical internal scale).
    /// - Sub-nanosecond attoseconds are set to zero.
    #[inline]
    pub fn from_jiff_timestamp(ts: Timestamp) -> Self {
        Dt::UNIX_EPOCH.add(TSpan::from_ns(ts.as_nanosecond()))
    }
}
