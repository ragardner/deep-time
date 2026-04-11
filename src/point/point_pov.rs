use crate::{MICROQUECTOS_PER_MILLISEC, MICROQUECTOS_PER_SEC, Point, TimePov};

impl Point {
    /// J2000.0 reference epoch: 2000-01-01 12:00:00 TT  
    /// **This is now the zero point for TAI** (your new standard count)
    pub const J2000_TAI: Self = Self::ZERO;

    /// J1900 reference epoch: 1900-01-01 12:00:00 TAI (noon)
    /// Exactly 36,525 days before J2000.0 (integer seconds)
    pub const J1900_TAI: Self = Self::from_tai_sec(-3_155_760_000);

    /// UNIX epoch expressed in TAI: 1970-01-01 00:00:00 TAI
    /// (exact fractional handling for the 0.184 s borrow)
    pub const UNIX_EPOCH_TAI: Self = Self {
        sec: -946_728_000,
        subsec: MICROQUECTOS_PER_SEC - 184 * MICROQUECTOS_PER_MILLISEC,
        pov: TimePov::TAI,
    };

    /// GPS Time reference epoch: 1980-01-06 00:00:00 GPST
    pub const GPS_EPOCH: Self = Self::new(0, 0, TimePov::GPST);

    /// Galileo Time reference epoch: 1999-08-22 00:00:00 GST
    pub const GALILEO_EPOCH: Self = Self::new(0, 0, TimePov::GST);

    /// BeiDou Time reference epoch: 2006-01-01 00:00:00 BDT
    pub const BEIDOU_EPOCH: Self = Self::new(0, 0, TimePov::BDT);

    /// QZSS Time reference epoch (identical reference to GPST)
    pub const QZSS_EPOCH: Self = Self::new(0, 0, TimePov::QZSST);
}
