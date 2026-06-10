use crate::{DtErr, DtErrKind, Offset, Parser, TimeParts, an_err};

impl TimeParts {
    /// Low-level parser equivalent to `strptime` with a provided format string.
    ///
    /// This is the core entry point for format-string based parsing in the library.
    /// It supports a large range of `%` directives (similar to `jiff` / `chrono`).
    ///
    /// The parser populates a [`TimeParts`] struct. After successful parsing,
    /// [`Self::finish`] is called automatically to apply defaults and validation.
    ///
    /// ## Parameters
    ///
    /// - `fmt`: The format string containing `%` directives.
    /// - `input`: The string to parse.
    /// - `inp_can_end_before_fmt`: If `true`, the input may end before the format
    ///   string is fully consumed (extra format specifiers are ignored).
    /// - `fmt_can_end_before_inp`: If `true`, the format may end before the input
    ///   is fully consumed (trailing characters in the input are allowed).
    /// - `allow_partial_date`: If `true`, a missing month/day will be defaulted
    ///   to `1` instead of returning an [`Incomplete`] error.
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] containing one of the following [`DtErrKind`] variants:
    ///
    /// ### Format string errors
    ///
    /// - [`DtErrKind::TruncatedDirective`] — The format string ended immediately
    ///   after a `%` or after a `.` in a fractional directive (e.g. `%.`).
    /// - [`DtErrKind::UnknownItem`] — Unknown `%` directive character.
    /// - [`DtErrKind::UnsupportedItem`] — Known but unsupported directive
    ///   (e.g. `%c`, `%r`, `%x`, `%X`, `%Z`).
    /// - [`DtErrKind::BadFractional`] — Malformed fractional directive
    ///   (e.g. `%.x` where `x` is not `f` or `N`).
    ///
    /// ### Input parsing errors
    ///
    /// - [`DtErrKind::UnexpectedInputEnd`] — Input ended before a required value
    ///   could be parsed.
    /// - `Expected*` variants:
    ///   - [`DtErrKind::ExpectedYear`]
    ///   - [`DtErrKind::ExpectedMonth`]
    ///   - [`DtErrKind::ExpectedDay`]
    ///   - [`DtErrKind::ExpectedDayOfYear`]
    ///   - [`DtErrKind::ExpectedHour`]
    ///   - [`DtErrKind::ExpectedMinute`]
    ///   - [`DtErrKind::ExpectedSecond`]
    ///   - [`DtErrKind::ExpectedFractionalSeconds`]
    ///   - [`DtErrKind::ExpectedTimestamp`]
    ///   - [`DtErrKind::ExpectedWeekNumber`]
    ///   - [`DtErrKind::ExpectedWeekdayNumber`]
    /// - [`DtErrKind::MismatchedLiteral`] — A literal character from the format
    ///   string did not match the input.
    /// - [`DtErrKind::OutOfRange`] — A numeric value was parsed but is outside
    ///   the valid range for that component (e.g. month 13, hour 25, day 32).
    /// - [`DtErrKind::InvalidName`] — Unrecognized month name, weekday name,
    ///   or `am`/`pm` value.
    /// - [`DtErrKind::InvalidTimezoneOffset`] — Invalid or malformed timezone
    ///   offset / IANA name.
    /// - [`DtErrKind::MustStartWith`] — Timezone offset did not start with
    ///   `+` or `-`.
    ///
    /// ### Post-processing / validation errors
    ///
    /// - [`DtErrKind::Incomplete`] — Required date components (month/day) were
    ///   missing and `allow_partial_date` was `false`.
    /// - [`DtErrKind::TrailingCharacters`] — The input contained trailing
    ///   characters after parsing and `fmt_can_end_before_inp` was `false`.
    ///
    /// Because [`DtErrKind`] is `#[non_exhaustive]`, additional variants may
    /// appear in the future. You can match on the variants you care about and
    /// use a wildcard arm for the rest.
    ///
    /// The concrete error kind is available via [`DtErr::kind()`] (or by
    /// iterating [`DtErr::trace()`] if the error was chained with context
    /// higher up the call stack).
    pub fn from_str(
        fmt: &str,
        input: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<TimeParts, DtErr> {
        let mut parts = TimeParts::new_utc();
        let mut parser = Parser::new(
            fmt.as_bytes(),
            input.as_bytes(),
            &mut parts,
            inp_can_end_before_fmt,
        );
        parser.parse()?;
        if parser.inp.is_empty() || fmt_can_end_before_inp {
            // All input consumed → finalize
            parts.finish(allow_partial_date)?;
            Ok(parts)
        } else {
            // Trailing characters remain
            Err(an_err!(DtErrKind::TrailingCharacters))
        }
    }

    /// Finalizes a [`TimeParts`] after parsing by applying sensible defaults and
    /// performing validation.
    ///
    /// This is called automatically by the various parsing paths (`from_str`,
    /// CCSDS parsers, etc.). It ensures the struct is in a consistent state
    /// before being turned into a full [`Dt`] or passed to other converters.
    ///
    /// ## Behavior
    ///
    /// - If a Unix timestamp is present, it takes precedence and the time
    ///   components are defaulted to `00:00:00.000000000` with a UTC offset.
    /// - Otherwise:
    ///   - Hour/minute/second/attoseconds/offset are defaulted to `0` / `Utc`.
    ///   - Leap seconds (`second == 60`) are detected and flagged.
    /// - Date completeness is checked in this priority order:
    ///   1. Calendar date (`year`, `month`, `day`)
    ///   2. Ordinal date (`year`, `day_of_year`)
    ///   3. ISO week date (`iso_week_year`, `iso_week`)
    /// - If `allow_partial_date` is `true`, missing month/day are defaulted to `1`.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::Incomplete`] if no valid date representation is present.
    /// - [`DtErrKind::OutOfRange`] for seconds outside `0..=60`.
    #[inline(always)]
    pub(crate) fn finish(&mut self, allow_partial_date: bool) -> Result<(), DtErr> {
        if self.offset.is_none() {
            self.offset = Some(Offset::Utc);
        }
        if self.unix_timestamp_seconds.is_none() {
            let has_calendar_date = if allow_partial_date {
                if self.day.is_none() {
                    self.day = Some(1);
                }
                if self.mo.is_none() {
                    self.mo = Some(1);
                }
                self.yr.is_some()
            } else {
                self.yr.is_some() && self.mo.is_some() && self.day.is_some()
            };
            let has_ordinal_date = self.yr.is_some() && self.day_of_yr.is_some();
            let has_iso_week_date = self.iso_wk_yr.is_some() && self.iso_wk.is_some();

            if !has_calendar_date && !has_ordinal_date && !has_iso_week_date {
                return Err(an_err!(DtErrKind::Incomplete));
            }
        }

        Ok(())
    }
}
