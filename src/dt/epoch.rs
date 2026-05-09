use crate::{Dt, Scale};

impl Dt {
    #[inline]
    pub const fn to_tai_attos_since(self, reference: Dt) -> i128 {
        self.to_diff_raw(reference).to_attos()
    }

    #[inline]
    pub const fn from_tai_attos_since(attos: i128, reference: Dt) -> Self {
        reference.add(Dt::from_attos(attos, Scale::TAI))
    }

    #[inline]
    pub const fn to_epoch(self, epoch: Dt, scale: Scale) -> Dt {
        self.to(scale).to_diff_raw(epoch)
    }

    #[inline]
    pub const fn from_epoch(offset: Dt, epoch: Dt, scale: Scale) -> Self {
        Dt::from_dt(epoch.add(offset), scale)
    }
}
