use crate::{ATTOS_PER_WEEK, Dt, Real, Scale};

impl Dt {
    /// Creates a `Dt` in GPS Time (GPS) from a GPS week number and
    /// Time of Week (TOW).
    ///
    /// This is the exact inverse of [`Self::to_gps_week_and_tow`].
    ///
    /// - `week`: Full GPS week number (can be negative for dates before 1980).
    /// - `tow`: Time of Week as a [`Dt`]. Values ≥ 604800 seconds are
    ///   automatically carried into the week number.
    ///
    /// The resulting `Dt` is always in `Scale::GPS`.
    #[inline]
    pub const fn from_gps_wk_and_tow(wk: i64, tow: Dt) -> Self {
        let total_attos = (wk as i128) * ATTOS_PER_WEEK + tow.to_attos();
        Self::GPS_EPOCH.add(Dt::from_attos(total_attos, Scale::TAI))
    }

    /// Creates a `Dt` in GPS Time from a GPS week number and
    /// floating-point Time of Week.
    ///
    /// This is the floating-point counterpart to [`Self::from_gps_wk_and_tow`].
    #[inline]
    pub const fn from_gps_wk_and_tow_f(week: i64, tow: Real) -> Self {
        let tow_span = Dt::from_sec_f(tow);
        Self::from_gps_wk_and_tow(week, tow_span)
    }
}
