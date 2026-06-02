use super::{OffsetInfo, abbrev_from_str};
use jiff::{
    Span, Timestamp,
    civil::{DateTime, date},
    tz::{AmbiguousOffset, TimeZone},
};

const EPOCH: DateTime = date(1970, 1, 1).at(0, 0, 0, 0);

#[inline(always)]
fn static_abbrev(jiff_abbrev: &str) -> &'static str {
    abbrev_from_str(jiff_abbrev).unwrap_or("???")
}

pub fn jiff_offset_info_at_local(name: &str, local_unix: i64) -> Option<OffsetInfo> {
    let tz = TimeZone::get(name).ok()?;

    let civil = EPOCH.checked_add(Span::new().seconds(local_unix)).ok()?;

    let amb = tz.to_ambiguous_zoned(civil);

    match amb.offset() {
        AmbiguousOffset::Gap { before, after } => {
            let gap_seconds = after.seconds() - before.seconds();

            let zdt = amb.compatible().ok()?;
            // let info = tz.to_offset_info(zdt.timestamp());

            Some(OffsetInfo {
                offset: zdt.offset().seconds(),
                abbrev: "???", // static_abbrev(info.abbreviation())
                is_gap: true,
                gap_size: gap_seconds as i64,
            })
        }
        _ => {
            // let info = tz.to_offset_info(zdt.timestamp());

            Some(OffsetInfo {
                offset: amb.compatible().ok()?.offset().seconds(),
                abbrev: "???", // static_abbrev(info.abbreviation())
                is_gap: false,
                gap_size: 0,
            })
        }
    }
}

pub fn jiff_offset_info_at_utc(name: &str, utc_unix: i64) -> Option<OffsetInfo> {
    let tz = TimeZone::get(name).ok()?;
    let ts = Timestamp::from_second(utc_unix).ok()?;

    let zdt = ts.in_tz(name).ok()?;
    let info = tz.to_offset_info(ts);

    Some(OffsetInfo {
        offset: zdt.offset().seconds(),
        abbrev: static_abbrev(info.abbreviation()),
        is_gap: false,
        gap_size: 0,
    })
}
