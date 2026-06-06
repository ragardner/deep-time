use super::OffsetInfo;
use crate::LiteStr; // adjust path to wherever your LiteStr lives
use jiff::{
    Span, Timestamp,
    civil::{DateTime, date},
    tz::{AmbiguousOffset, TimeZone},
};

const EPOCH: DateTime = date(1970, 1, 1).at(0, 0, 0, 0);

pub fn jiff_offset_info_at_local(name: &str, local_unix: i64) -> Option<OffsetInfo> {
    let tz = TimeZone::get(name).ok()?;

    let civil = EPOCH.checked_add(Span::new().seconds(local_unix)).ok()?;
    let amb = tz.to_ambiguous_zoned(civil);

    match amb.offset() {
        AmbiguousOffset::Gap { before, after } => {
            let gap_seconds = (after.seconds() as i64) - (before.seconds() as i64);
            let zdt = amb.compatible().ok()?;

            // let info = tz.to_offset_info(zdt.timestamp());

            Some(OffsetInfo {
                offset: zdt.offset().seconds(),
                abbrev: LiteStr::new(""),
                // abbrev: LiteStr::new(info.abbreviation()),
                is_gap: true,
                gap_size: gap_seconds,
            })
        }
        _ => {
            let zdt = amb.compatible().ok()?;
            // let info = tz.to_offset_info(zdt.timestamp());

            Some(OffsetInfo {
                offset: zdt.offset().seconds(),
                abbrev: LiteStr::new(""),
                // abbrev: LiteStr::new(info.abbreviation()),
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
        abbrev: LiteStr::new(info.abbreviation()),
        is_gap: false,
        gap_size: 0,
    })
}
