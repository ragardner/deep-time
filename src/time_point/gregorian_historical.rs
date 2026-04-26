use crate::{
    ClockType, TimePoint, TimeSpan, historical_sofa_offsets::historical_tai_minus_utc_offset,
};

impl TimePoint {
    /// Creates a `TimePoint` representing the **physical instant** corresponding
    /// to a civil date in the historical "rubber second" era (1960–1971),
    /// using the SOFA model for the TAI−UTC relationship.
    ///
    /// # When to use this function
    ///
    /// Use this constructor when you have a civil time from before 1972 and
    /// you want the library to automatically apply the historically correct
    /// physical offset. This is especially useful for:
    /// - Reconstructing old astronomical observations
    /// - Working with historical ephemerides or radio time-signal logs
    /// - Any situation where you need the exact physical moment that a
    ///   1960s UTC timestamp referred to.
    ///
    /// For dates on or after 1 January 1972, this function behaves
    /// identically to [`Self::from_gregorian_ymdhms`].
    ///
    /// # How it works
    ///
    /// 1. The civil date is first interpreted with ΔAT = 0 (modern rules).
    /// 2. If the date falls between 1960-01-01 and 1971-12-31, the historical
    ///    SOFA offset is added when converting to TAI.
    /// 3. The resulting physical instant is then converted to the requested
    ///    [`ClockType`].
    ///
    /// # Important notes
    ///
    /// - The returned `TimePoint` represents the **correct physical instant**.
    ///   All subsequent arithmetic and conversions (to TT, TCG, LTC, etc.)
    ///   will be accurate.
    /// - If you later convert the result back to UTC using the normal
    ///   [`Self::to_clock_type`] or [`Self::to_gregorian_time`], you may not
    ///   recover the exact original civil time. Use [`Self::to_historical_utc`]
    ///   instead if you need the historical civil representation.
    /// - Dates before 1960-01-01 fall back to normal (ΔAT = 0) behavior.
    pub const fn from_historical_gregorian_ymdhms(
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
        clock_type: ClockType,
    ) -> Self {
        let naive = Self::from_gregorian_ymdhms(yr, mo, day, hr, min, sec, attos, ClockType::UTC);

        if let Some(offset) = historical_tai_minus_utc_offset(yr as i32, mo, day) {
            let tai = naive.to_tai().add(TimeSpan::from_sec_f(offset));
            tai.to_clock_type(clock_type)
        } else {
            naive.to_clock_type(clock_type)
        }
    }

    /// Converts this instant to UTC using the **historical** TAI−UTC
    /// relationship for dates in the 1960–1971 period.
    ///
    /// # When to use this function
    ///
    /// Use this method when you have a `TimePoint` that was originally
    /// created with [`Self::from_historical_gregorian_ymdhms`] (or any
    /// pre-1972 instant) and you need to recover the **original civil time**
    /// that would have been displayed on clocks of that era.
    ///
    /// For dates on or after 1972-01-01, this function behaves identically
    /// to `to_clock_type(ClockType::UTC)`.
    ///
    /// # How it works
    ///
    /// 1. The instant is converted to TAI (always correct).
    /// 2. It is then converted to UTC using modern rules to obtain a
    ///    candidate civil date.
    /// 3. If that civil date falls in the historical period, the SOFA
    ///    offset is subtracted to recover the historically correct civil time.
    ///
    /// # Important notes
    ///
    /// - This is the recommended way to round-trip a historically created
    ///   `TimePoint` back to UTC.
    /// - Using the normal `to_clock_type(ClockType::UTC)` on a pre-1972
    ///   instant will generally produce a slightly different civil time.
    /// - The returned `TimePoint` has `clock_type == ClockType::UTC`.
    pub const fn to_historical_utc(&self) -> Self {
        let utc_naive = self.to_tai().to_clock_type(ClockType::UTC);
        let (yr, mo, day) = utc_naive.to_gregorian_ymd(None);

        if let Some(offset) = historical_tai_minus_utc_offset(yr as i32, mo, day) {
            utc_naive
                .sub(TimeSpan::from_sec_f(offset))
                .with_clock_type(ClockType::UTC)
        } else {
            utc_naive
        }
    }
}
