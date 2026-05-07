use crate::{ClockType, TimePoint, TimeSpan};

impl TimePoint {
    #[inline]
    pub const fn to_tai_attos_since(self, reference: TimePoint) -> i128 {
        self.to_tai_since(reference).to_attos()
    }

    #[inline]
    pub const fn from_tai_attos_since(attos: i128, reference: TimePoint) -> Self {
        reference.add(TimeSpan::from_attos(attos))
    }

    #[inline]
    pub const fn to_epoch(self, epoch: TimePoint, clock_type: ClockType) -> TimeSpan {
        /*
        do not apply an offset using to() to the EPOCH because the offset is for TAI,
        the to() function assumes the epoch is TAI, the UTCSofa instant for 1970 is
        the same as the UTC instant UNIX_EPOCH should remain UTC and the offset should
        not be applied to the epoch
        */
        self.to(clock_type).to_diff_tp(epoch)
    }

    #[inline]
    pub const fn from_epoch(offset: TimeSpan, epoch: TimePoint, clock_type: ClockType) -> Self {
        let total = epoch.to_span().add(offset);
        TimePoint::from(total.sec, total.subsec, clock_type)
    }
}
