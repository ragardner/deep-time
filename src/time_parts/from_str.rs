use crate::{DtErrKind, DtError, TimeParts, TimeZone, parser::Parser};

impl TimeParts {
    pub fn from_str(
        fmt: &str,
        input: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<TimeParts, DtError> {
        let mut tm = TimeParts::new_utc();
        let mut parser = Parser::new(
            fmt.as_bytes(),
            input.as_bytes(),
            &mut tm,
            inp_can_end_before_fmt,
        );
        if let Err(e) = parser.parse() {
            return Err(e);
        }
        if parser.inp.is_empty() || fmt_can_end_before_inp {
            // All input consumed → finalize
            tm.finish(allow_partial_date)?;
            Ok(tm)
        } else {
            // Trailing characters remain
            Err(DtError::new(DtErrKind::TrailingCharacters))
        }
    }

    pub fn finish(&mut self, allow_partial_date: bool) -> core::result::Result<&mut Self, DtError> {
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
        if let Some(sec) = self.second {
            if sec == 60 {
                self.is_leap_second = true;
            } else if sec > 60 {
                return Err(DtError::new(DtErrKind::SecondOutOfRange));
            }
        } else {
            self.second = Some(0);
        }
        if self.attos.is_none() {
            self.attos = Some(0);
        }
        if self.tz.is_none() {
            self.tz = Some(TimeZone::Utc);
        }

        let has_calendar_date = if allow_partial_date {
            if self.day.is_none() {
                self.day = Some(1);
            }
            if self.month.is_none() {
                self.month = Some(1);
            }
            self.year.is_some()
        } else {
            self.year.is_some() && self.month.is_some() && self.day.is_some()
        };
        let has_ordinal_date = self.year.is_some() && self.day_of_year.is_some();
        let has_iso_week_date = self.iso_week_year.is_some() && self.iso_week.is_some();

        if !has_calendar_date && !has_ordinal_date && !has_iso_week_date {
            return Err(DtError::new(DtErrKind::IncompleteDate));
        }

        Ok(self)
    }
}
