use crate::{Dt, Scale, TSpan};

impl Dt {
    #[inline]
    pub const fn to_tai_attos_since(self, reference: Dt) -> i128 {
        self.to_tai_since(reference).to_attos()
    }

    #[inline]
    pub const fn from_tai_attos_since(attos: i128, reference: Dt) -> Self {
        reference.add(TSpan::from_attos(attos))
    }

    #[inline]
    pub const fn to_epoch(self, epoch: Dt, scale: Scale) -> TSpan {
        /*
        do not apply an offset using to() to the EPOCH because the offset is for TAI,
        the to() function assumes the epoch is TAI, the UTCSofa instant for 1970 is
        the same as the UTC instant UNIX_EPOCH should remain UTC and the offset should
        not be applied to the epoch
        */
        self.to(scale).to_diff_tp(epoch)
    }

    #[inline]
    pub const fn from_epoch(offset: TSpan, epoch: Dt, scale: Scale) -> Self {
        let total = epoch.to_span().add(offset);
        Dt::from(total.sec, total.attos, scale)
    }
}
