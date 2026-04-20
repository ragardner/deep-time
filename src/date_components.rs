use crate::{ClockType, DtErrKind, DtError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Meridiem {
    #[default]
    AM,
    PM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weekday {
    #[default]
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Weekday {
    #[inline]
    pub fn from_sunday_zero_offset(n: i8) -> Result<Self, &'static str> {
        match n {
            0 => Ok(Weekday::Sunday),
            1 => Ok(Weekday::Monday),
            2 => Ok(Weekday::Tuesday),
            3 => Ok(Weekday::Wednesday),
            4 => Ok(Weekday::Thursday),
            5 => Ok(Weekday::Friday),
            6 => Ok(Weekday::Saturday),
            _ => Err("weekday number out of range (must be 0-6, Sunday=0)"),
        }
    }

    #[inline]
    pub fn from_monday_one_offset(n: i8) -> Result<Self, &'static str> {
        match n {
            1 => Ok(Weekday::Monday),
            2 => Ok(Weekday::Tuesday),
            3 => Ok(Weekday::Wednesday),
            4 => Ok(Weekday::Thursday),
            5 => Ok(Weekday::Friday),
            6 => Ok(Weekday::Saturday),
            7 => Ok(Weekday::Sunday),
            _ => Err("weekday number out of range (must be 1-7, Monday=1)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeZone {
    #[default]
    Utc,
    None,
    /// Fixed offset from UTC in seconds
    Fixed(i32),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DateComponents {
    pub year: Option<i64>,
    pub month: Option<u8>,  // 1-12
    pub day: Option<u8>,    // 1-31
    pub hour: Option<u8>,   // 0-23
    pub minute: Option<u8>, // 0-59
    pub second: Option<u8>, // 0-60
    pub attos: Option<u64>, // 0 ≤ value < 10¹⁸
    pub tz: Option<TimeZone>,
    pub iana_name: Option<[u8; 48]>,
    pub is_leap_second: bool,
    pub clock_type: ClockType,
    pub weekday: Option<Weekday>,
    pub day_of_year: Option<u16>,   // 1-366 (%j)
    pub iso_week_year: Option<i64>, // %G / %g
    pub iso_week: Option<u8>,       // 1-53 (%V)
    pub week_sun: Option<u8>,       // 0-53 (%U)
    pub week_mon: Option<u8>,       // 0-53 (%W)
    pub meridiem: Option<Meridiem>,
    pub unix_timestamp_seconds: Option<i64>, // %s
}

impl DateComponents {
    #[inline]
    pub fn finish(mut self) -> core::result::Result<Self, DtError> {
        if self.unix_timestamp_seconds.is_some() {
            if self.hour.is_none() {
                self.hour = Some(0);
            }
            if self.minute.is_none() {
                self.minute = Some(0);
            }
            if self.second.is_none() {
                self.second = Some(0);
            }
            if self.attos.is_none() {
                self.attos = Some(0);
            }
            if self.tz.is_none() {
                self.tz = Some(TimeZone::Utc);
            }
            return Ok(self);
        }

        // Sensible defaults for time components (most tests expect a full datetime)
        if self.hour.is_none() {
            self.hour = Some(0);
        }
        if self.minute.is_none() {
            self.minute = Some(0);
        }
        if self.second.is_none() {
            self.second = Some(0);
        }
        if self.attos.is_none() {
            self.attos = Some(0);
        }
        if self.tz.is_none() {
            self.tz = Some(TimeZone::Utc);
        }

        let has_calendar_date = self.year.is_some() && self.month.is_some() && self.day.is_some();
        let has_ordinal_date = self.year.is_some() && self.day_of_year.is_some();
        let has_iso_week_date = self.iso_week_year.is_some() && self.iso_week.is_some();

        if !has_calendar_date && !has_ordinal_date && !has_iso_week_date {
            return Err(DtError::new(DtErrKind::IncompleteDate));
        }

        let sec = self.second.unwrap();
        if sec > 60 {
            return Err(DtError::new(DtErrKind::SecondOutOfRange));
        }
        if sec == 60 {
            self.is_leap_second = true;
        }

        Ok(self)
    }
}
