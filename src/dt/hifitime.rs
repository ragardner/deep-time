use crate::{Dt, Scale};
use core::convert::From;
use hifitime::{Duration, Epoch};

impl Dt {
    /// Converts this [`Dt`] to a [`hifitime::Epoch`].
    ///
    /// - The [`Dt`] is first converted to `Scale::TAI` before producing a result.
    /// - The returned [`hifitime::Epoch`] is on the TAI time scale.
    pub fn to_hifitime_epoch(&self) -> Epoch {
        let nanos = self.to_tai().to_ns().0;

        // [`Dt::ZERO`] is J2000.0 TAI noon; anchor on hifitime's matching instant.
        let j2000 = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);
        let ns_since_j1900 = nanos.saturating_add(j2000.to_tai_duration().total_nanoseconds());

        Epoch::from_tai_duration(Duration::from_total_nanoseconds(ns_since_j1900))
    }

    /// Creates a TAI [`Dt`] from a [`hifitime::Epoch`].
    pub fn from_hifitime_epoch(epoch: Epoch) -> Dt {
        let ns_since_j1900 = epoch.to_tai_duration().total_nanoseconds();

        let j2000 = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);
        let ns_since_zero_tai =
            ns_since_j1900.saturating_sub(j2000.to_tai_duration().total_nanoseconds());
        Self::from_ns(ns_since_zero_tai, 0, Scale::TAI, Scale::TAI)
    }

    /// Converts this [`Dt`] to a [`hifitime::Duration`] (nanosecond precision).
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion is exact up to the nanosecond (128-bit integer arithmetic).
    /// - Internally uses [`hifitime::Duration::from_total_nanoseconds`], which
    ///   automatically normalizes centuries/nanoseconds and saturates at
    ///   [`Duration::MAX`] / [`Duration::MIN`] if outside hifitime's range
    ///   (±32,768 centuries).
    #[inline(always)]
    pub fn to_hifitime_duration(&self) -> Duration {
        Duration::from_total_nanoseconds(self.to_ns().0)
    }

    /// Creates a [`Dt`] from a [`hifitime::Duration`] (nanosecond precision).
    ///
    /// Inverse of [`Dt::to_hifitime_duration`].
    #[inline(always)]
    pub fn from_hifitime_duration(dur: Duration) -> Dt {
        Self::from_ns(dur.total_nanoseconds(), 0, Scale::TAI, Scale::TAI)
    }
}

impl From<Epoch> for Dt {
    #[inline]
    fn from(epoch: Epoch) -> Self {
        Self::from_hifitime_epoch(epoch)
    }
}

impl From<Dt> for Epoch {
    #[inline]
    fn from(dt: Dt) -> Self {
        dt.to_hifitime_epoch()
    }
}

impl From<Duration> for Dt {
    #[inline]
    fn from(dur: Duration) -> Self {
        Self::from_hifitime_duration(dur)
    }
}

impl From<Dt> for Duration {
    #[inline]
    fn from(dt: Dt) -> Self {
        dt.to_hifitime_duration()
    }
}
