//! TZDB code generator for deep-time
//!
//! This is a development tool used to regenerate `tzdb.rs` from the
//! official IANA Time Zone Database.
//!
//! ## Setup
//!
//! 1. Download the latest tzdata release from:
//!    https://www.iana.org/time-zones
//! 2. Extract it into a `tzdata/` folder in the repository root.
//!
//!    Example structure:
//!    ```text
//!    deep-time/
//!    ├── tzdata/
//!    │   └── tzdata2026b/
//!    │       ├── africa
//!    │       ├── europe
//!    │       └── ...
//!    └── crates/
//!        └── tz-generator/
//!    ```
//!
//! ## Usage
//!
//! From the `deep-time` repository root, run:
//!
//! ```bash
//! cargo run -p tz-generator -- tzdata/tzdata2026b
//! ```
//!
//! The generator will write the updated data to `src/tz/tzdb.rs`.

use parse_zoneinfo::{
    line::{Line, Year},
    table::{Saving, Table, TableBuilder},
    transitions::TableTransitions,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Transition {
    pub utc_timestamp: i64,
    pub local_timestamp: i64,
    pub offset: i32,
    pub abbrev_idx: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeating {
    None,
    /// The transition pattern repeats indefinitely starting at `cycle_start`.
    /// One full period consists of `cycle_len` consecutive entries.
    /// The period is measured in local wall-clock seconds.
    Cycle {
        cycle_start: usize,
        cycle_len: usize,
        period: i64,
    },
}

// === Helper functions for the generator ===

fn build_internal_transitions(
    set: &parse_zoneinfo::transitions::FixedTimespanSet,
    abbrev_to_idx: &HashMap<String, u16>,
) -> Vec<Transition> {
    let mut transitions = Vec::new();

    let first = &set.first;
    let first_idx = *abbrev_to_idx.get(&first.name).unwrap();
    transitions.push(Transition {
        utc_timestamp: i64::MIN,
        local_timestamp: i64::MIN,
        offset: first.total_offset() as i32,
        abbrev_idx: first_idx,
    });

    let mut prev_offset = first.total_offset() as i64;

    for (ts, ft) in &set.rest {
        let local_ts = *ts + prev_offset;
        let idx = *abbrev_to_idx.get(&ft.name).unwrap();

        transitions.push(Transition {
            utc_timestamp: *ts,
            local_timestamp: local_ts,
            offset: ft.total_offset() as i32,
            abbrev_idx: idx,
        });

        prev_offset = ft.total_offset() as i64;
    }

    transitions
}

fn validate_transitions(transitions: &[Transition], name: &str) {
    for i in 1..transitions.len() {
        if transitions[i].local_timestamp < transitions[i - 1].local_timestamp {
            eprintln!("Warning: non-monotonic local_timestamp for {}", name);
        }
        if transitions[i].utc_timestamp < transitions[i - 1].utc_timestamp {
            eprintln!("Warning: non-monotonic utc_timestamp for {}", name);
        }
    }
}

fn check_perpetual_metadata(name: &str, table: &Table) -> Option<bool> {
    let zoneset = table.get_zoneset(name)?;
    let last_zone = zoneset.last()?;

    if last_zone.end_time.is_some() {
        return Some(false);
    }

    if let Saving::Multiple(ref rules_name) = last_zone.saving {
        if let Some(rules) = table.rulesets.get(rules_name) {
            let has_perpetual = rules
                .iter()
                .any(|r| matches!(r.to_year, Some(Year::Maximum) | None));
            return Some(has_perpetual);
        }
    }

    Some(false)
}

fn try_detect_stable_cycle(transitions: &[Transition]) -> Option<Repeating> {
    const VALIDATION_WINDOW: usize = 8;
    const MIN_REASONABLE_PERIOD: i64 = 2_592_000;
    const MAX_REASONABLE_PERIOD: i64 = 34_560_000;

    if transitions.len() <= VALIDATION_WINDOW {
        return None;
    }

    let validation_start = transitions.len().saturating_sub(VALIDATION_WINDOW);

    if validation_start == 0 || transitions[validation_start].local_timestamp == i64::MIN {
        return None;
    }

    let p0 = transitions[validation_start + 2].local_timestamp
        - transitions[validation_start].local_timestamp;
    let p1 = transitions[validation_start + 4].local_timestamp
        - transitions[validation_start + 2].local_timestamp;
    let p2 = transitions[validation_start + 6].local_timestamp
        - transitions[validation_start + 4].local_timestamp;

    if !(p0 > 0
        && p0 == p1
        && p1 == p2
        && p0 >= MIN_REASONABLE_PERIOD
        && p0 <= MAX_REASONABLE_PERIOD)
    {
        return None;
    }

    let window: Vec<(i32, u16)> = (0..VALIDATION_WINDOW)
        .map(|i| {
            let t = &transitions[validation_start + i];
            (t.offset, t.abbrev_idx)
        })
        .collect();

    let mut best_cycle_len = VALIDATION_WINDOW;

    for candidate in 2..=VALIDATION_WINDOW {
        let mut repeats = true;
        for pos in 0..candidate {
            let first = window[pos];
            for j in (pos + candidate..VALIDATION_WINDOW).step_by(candidate) {
                if window[j] != first {
                    repeats = false;
                    break;
                }
            }
            if !repeats {
                break;
            }
        }
        if repeats {
            best_cycle_len = candidate;
            break;
        }
    }

    Some(Repeating::Cycle {
        cycle_start: validation_start,
        cycle_len: best_cycle_len,
        period: p0,
    })
}

fn detect_repeating(name: &str, transitions: &[Transition], table: &Table) -> Repeating {
    // Prefer clean metadata from parse-zoneinfo when available
    if let Some(true) = check_perpetual_metadata(name, table) {
        if let Some(cycle) = try_detect_stable_cycle(transitions) {
            return cycle;
        }
    }

    // Empirical fallback / validation on the generated tail
    if let Some(cycle) = try_detect_stable_cycle(transitions) {
        return cycle;
    }

    Repeating::None
}

fn truncate_for_repeating(
    mut transitions: Vec<Transition>,
    repeating: Repeating,
) -> Vec<Transition> {
    if let Repeating::Cycle {
        cycle_start,
        cycle_len,
        ..
    } = repeating
    {
        let keep_up_to = cycle_start + cycle_len;
        if keep_up_to < transitions.len() {
            transitions.truncate(keep_up_to);
        }
    }
    transitions
}

// === main ===

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <path-to-tzdata2026b>");
        std::process::exit(1);
    }
    let tzdata_dir = Path::new(&args[1]);

    let dir_name = tzdata_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let version = dir_name
        .find(|c: char| c.is_ascii_digit())
        .map(|idx| &dir_name[idx..])
        .unwrap_or("unknown")
        .to_string();

    let zone_files = [
        "africa",
        "antarctica",
        "asia",
        "australasia",
        "backward",
        "backzone",
        "etcetera",
        "europe",
        "northamerica",
        "southamerica",
    ];

    let mut builder = TableBuilder::new();

    for entry in fs::read_dir(tzdata_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_str().unwrap();
            if !zone_files.contains(&filename) {
                continue;
            }
            let content = fs::read_to_string(&path).unwrap();
            for raw_line in content.lines() {
                let content = raw_line.split('#').next().unwrap().trim_end();
                if content.trim().is_empty() {
                    continue;
                }
                if let Ok(parsed) = Line::new(content) {
                    if let Err(e) = builder.add_line(parsed) {
                        eprintln!("Warning: failed to add line from {}: {}", filename, e);
                    }
                }
            }
        }
    }

    let table = builder.build();

    // Collect abbreviations
    let mut abbrev_set: HashSet<String> = HashSet::new();
    for name in table
        .zonesets
        .keys()
        .chain(table.links.keys())
        .map(|s| s.as_str())
    {
        if let Some(set) = table.timespans(name) {
            abbrev_set.insert(set.first.name.clone());
            for (_, ft) in &set.rest {
                abbrev_set.insert(ft.name.clone());
            }
        }
    }
    let mut abbrevs: Vec<String> = abbrev_set.into_iter().collect();
    abbrevs.sort();
    let abbrev_to_idx: HashMap<String, u16> = abbrevs
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i as u16))
        .collect();

    println!("Unique abbreviations found: {}", abbrevs.len());

    let mut name_to_data: HashMap<String, (Vec<Transition>, Repeating)> = HashMap::new();

    for name in table
        .zonesets
        .keys()
        .chain(table.links.keys())
        .map(|s| s.as_str())
    {
        if let Some(set) = table.timespans(name) {
            let transitions = build_internal_transitions(&set, &abbrev_to_idx);
            validate_transitions(&transitions, name);

            let repeating = detect_repeating(name, &transitions, &table);
            let transitions = truncate_for_repeating(transitions, repeating);

            name_to_data.insert(name.to_string(), (transitions, repeating));
        }
    }

    // Deduplication
    let mut unique: HashMap<Vec<Transition>, usize> = HashMap::new();
    let mut data_counter = 0usize;
    let mut data_names: Vec<String> = Vec::new();

    for (_, (trans, _)) in &name_to_data {
        if !unique.contains_key(trans) {
            unique.insert(trans.clone(), data_counter);
            data_names.push(format!("DATA_{}", data_counter));
            data_counter += 1;
        }
    }

    // Build entries
    let mut entries: Vec<(String, String, Repeating)> = Vec::new();
    for (name, (trans, repeating)) in &name_to_data {
        if let Some(&id) = unique.get(trans) {
            entries.push((name.clone(), data_names[id].clone(), *repeating));
        }
    }
    entries.sort_by_key(|(name, _, _)| name.clone());

    // Find UTC data for minimal mode
    let utc_data_name = entries
        .iter()
        .find(|(name, _, _)| name == "UTC")
        .map(|(_, dn, _)| dn.clone())
        .or_else(|| {
            entries
                .iter()
                .find(|(name, _, _)| name == "Etc/UTC")
                .map(|(_, dn, _)| dn.clone())
        })
        .unwrap_or_else(|| {
            data_names
                .first()
                .cloned()
                .unwrap_or_else(|| "DATA_0".to_string())
        });

    let minimal_entries: Vec<(String, String, Repeating)> = entries
        .iter()
        .filter(|(_, data_name, _)| data_name == &utc_data_name)
        .cloned()
        .collect();

    println!(
        "UTC-equivalent zones (minimal mode): {} zones share {}",
        minimal_entries.len(),
        utc_data_name
    );

    // === Generate tzdb.rs ===
    let mut output = String::new();

    output.push_str("#![allow(clippy::large_enum_variant)]\n");
    output.push_str("#![allow(clippy::too_many_lines)]\n");
    output.push_str("#![cfg_attr(rustfmt, rustfmt::skip)]\n\n");

    output.push_str(&format!(
        "//! This module is auto-generated from the IANA Time Zone Database\n\
//! found at: https://www.iana.org/time-zones\n\
//! Source directory: {}\n\
//! It provides both a minimal mode (UTC + identical zones only) and a full\n\
//! mode (`tz` feature) which has full historical transitions.\n\
//! Generator source: https://github.com/ragardner/deep-time\n\n",
        dir_name
    ));

    output.push_str(&format!("pub static VERSION: &str = \"{}\";\n\n", version));

    // ABBREVS
    output.push_str(&format!(
        "pub static ABBREVS: [&str; {}] = [\n",
        abbrevs.len()
    ));
    for abbr in &abbrevs {
        output.push_str(&format!("    \"{}\",\n", abbr));
    }
    output.push_str("];\n\n");

    output.push_str(
        r#"#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    /// Local wall-clock Unix timestamp of the transition instant
    /// (computed using the *previous* offset). Primary key for
    /// local-time binary search, gap/fold detection, and repeating
    /// cycle positioning.
    ///
    /// The corresponding UTC instant can be derived with
    /// `transition_utc(transitions, idx)`.
    pub local_timestamp: i64,
    pub offset: i32,
    pub abbrev_idx: u16,
}

/// Repeating describes how (or if) the transition pattern continues
/// indefinitely after the last explicit entry (for zones with perpetual
/// DST rules). The generator detects stable cycles using both metadata
/// from parse-zoneinfo and empirical validation on the generated
/// transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeating {
    None,
    /// Pattern repeats every `period` local seconds starting at
    /// `cycle_start`. One repeating unit has `cycle_len` entries.
    Cycle {
        cycle_start: usize,
        cycle_len: usize,
        period: i64,
    },
}

#[inline]
pub fn get_tz_data(name: &str) -> Option<(&str, &'static [Transition], Repeating)> {
    let idx = TZ_ENTRIES.partition_point(|(n, _, _)| *n < name);
    if idx < TZ_ENTRIES.len() && TZ_ENTRIES[idx].0 == name {
        Some(TZ_ENTRIES[idx])
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OffsetInfo {
    pub offset: i32,
    pub abbrev: &'static str,
    /// Whether the requested local time falls in a gap (spring-forward).
    /// Add `gap_size` to the original local time and re-query to obtain
    /// a valid instant (yields the *later* instant).
    pub is_gap: bool,
    pub gap_size: i64,
}

#[inline]
pub fn abbrev(idx: u16) -> &'static str {
    ABBREVS[idx as usize]
}

#[inline]
pub fn abbrev_from_str(abbrev: &str) -> Option<&'static str> {
    match ABBREVS.binary_search(&abbrev) {
        Ok(i) => Some(ABBREVS[i]),
        Err(_) => None,
    }
}

#[inline]
fn last_transition(transitions: &[Transition]) -> Option<OffsetInfo> {
    transitions.last().map(|t| OffsetInfo {
        offset: t.offset,
        abbrev: abbrev(t.abbrev_idx),
        is_gap: false,
        gap_size: 0,
    })
}

/// Computes the UTC instant at which the transition at `idx` occurs.
#[inline]
fn transition_utc(transitions: &[Transition], idx: usize) -> i64 {
    if idx == 0 {
        i64::MIN
    } else {
        transitions[idx].local_timestamp - transitions[idx - 1].offset as i64
    }
}

/// Binary search for the last transition whose UTC time ≤ `utc_unix`.
#[inline]
fn find_transition_for_utc(transitions: &[Transition], utc_unix: i64) -> usize {
    let mut lo = 0usize;
    let mut hi = transitions.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if transition_utc(transitions, mid) <= utc_unix {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo == 0 { 0 } else { lo - 1 }
}

/// Resolve far-future local time for repeating zones.
/// The gap/fold logic here mirrors the historical path so that
/// `.earlier()` behavior on ambiguous (fold) times remains consistent
/// with jiff.
fn resolve_far_future_local(
    transitions: &[Transition],
    repeating: Repeating,
    local_unix: i64,
) -> Option<OffsetInfo> {
    let (cycle_start, cycle_len, period) = match repeating {
        Repeating::Cycle { cycle_start, cycle_len, period } => (cycle_start, cycle_len, period),
        Repeating::None => return last_transition(transitions),
    };

    if cycle_start + cycle_len > transitions.len() || cycle_len < 2 {
        return last_transition(transitions);
    }

    let cycle = &transitions[cycle_start..cycle_start + cycle_len];
    let first = &cycle[0];
    if first.local_timestamp == i64::MIN {
        return last_transition(transitions);
    }

    let elapsed = local_unix - first.local_timestamp;
    if elapsed < 0 {
        return last_transition(transitions);
    }
    let position_in_cycle = elapsed % period;

    let idx = cycle.partition_point(|t| {
        (t.local_timestamp - first.local_timestamp) <= position_in_cycle
    });

    if idx == 0 {
        let t = &cycle[0];
        return Some(OffsetInfo {
            offset: t.offset,
            abbrev: abbrev(t.abbrev_idx),
            is_gap: false,
            gap_size: 0,
        });
    }
    if idx >= cycle.len() {
        return last_transition(cycle);
    }

    let prev = &cycle[idx - 1];

    if idx >= 2 {
        let pprev = &cycle[idx - 2];
        let off_diff = (prev.offset - pprev.offset) as i64;
        if off_diff > 0 {
            let window_start = prev.local_timestamp;
            let window_size = off_diff;
            let window_end = window_start + window_size;
            let query_local = first.local_timestamp + position_in_cycle;
            if query_local >= window_start && query_local < window_end {
                return Some(OffsetInfo {
                    offset: prev.offset,
                    abbrev: abbrev(prev.abbrev_idx),
                    is_gap: true,
                    gap_size: off_diff,
                });
            }
        }
    }

    if idx < cycle.len() {
        let nxt = &cycle[idx];
        let off_diff = (nxt.offset - prev.offset) as i64;
        if off_diff != 0 {
            let window_start = prev.local_timestamp;
            let window_size = off_diff.saturating_abs();
            let window_end = window_start + window_size;
            let query_local = first.local_timestamp + position_in_cycle;
            if query_local >= window_start && query_local < window_end {
                if off_diff > 0 {
                    return Some(OffsetInfo {
                        offset: nxt.offset,
                        abbrev: abbrev(nxt.abbrev_idx),
                        is_gap: true,
                        gap_size: off_diff,
                    });
                } else {
                    // Fold → earlier instant (matches jiff .earlier())
                    return Some(OffsetInfo {
                        offset: prev.offset,
                        abbrev: abbrev(prev.abbrev_idx),
                        is_gap: false,
                        gap_size: 0,
                    });
                }
            }
        }
    }

    let t = &cycle[idx - 1];
    Some(OffsetInfo {
        offset: t.offset,
        abbrev: abbrev(t.abbrev_idx),
        is_gap: false,
        gap_size: 0,
    })
}

pub(crate) fn offset_info_at_local(name: &str, local_unix: i64) -> Option<OffsetInfo> {
    let (_, transitions, repeating) = get_tz_data(name)?;
    let idx = transitions.partition_point(|t| t.local_timestamp <= local_unix);
    if idx == 0 {
        let t = &transitions[0];
        return Some(OffsetInfo {
            offset: t.offset,
            abbrev: abbrev(t.abbrev_idx),
            is_gap: false,
            gap_size: 0,
        });
    }
    if idx >= transitions.len() {
        return resolve_far_future_local(transitions, repeating, local_unix);
    }

    let prev = &transitions[idx - 1];

    if idx >= 2 {
        let pprev = &transitions[idx - 2];
        let off_diff = (prev.offset - pprev.offset) as i64;
        if off_diff > 0 {
            let window_start = prev.local_timestamp;
            let window_size = off_diff;
            let window_end = window_start + window_size;
            if local_unix >= window_start && local_unix < window_end {
                return Some(OffsetInfo {
                    offset: prev.offset,
                    abbrev: abbrev(prev.abbrev_idx),
                    is_gap: true,
                    gap_size: off_diff,
                });
            }
        }
    }

    if idx < transitions.len() {
        let nxt = &transitions[idx];
        let off_diff = (nxt.offset - prev.offset) as i64;
        if off_diff != 0 {
            let window_start = prev.local_timestamp;
            let window_size = off_diff.saturating_abs();
            let window_end = window_start + window_size;
            if local_unix >= window_start && local_unix < window_end {
                if off_diff > 0 {
                    return Some(OffsetInfo {
                        offset: nxt.offset,
                        abbrev: abbrev(nxt.abbrev_idx),
                        is_gap: true,
                        gap_size: off_diff,
                    });
                } else {
                    // Fold → earlier instant (matches jiff .earlier())
                    return Some(OffsetInfo {
                        offset: prev.offset,
                        abbrev: abbrev(prev.abbrev_idx),
                        is_gap: false,
                        gap_size: 0,
                    });
                }
            }
        }
    }

    Some(OffsetInfo {
        offset: prev.offset,
        abbrev: abbrev(prev.abbrev_idx),
        is_gap: false,
        gap_size: 0,
    })
}

fn resolve_far_future_utc(
    transitions: &[Transition],
    repeating: Repeating,
    utc_unix: i64,
) -> Option<OffsetInfo> {
    let (cycle_start, cycle_len, period) = match repeating {
        Repeating::Cycle { cycle_start, cycle_len, period } => (cycle_start, cycle_len, period),
        Repeating::None => return last_transition(transitions),
    };

    if cycle_start + cycle_len > transitions.len() || cycle_len < 2 {
        return last_transition(transitions);
    }

    let cycle = &transitions[cycle_start..cycle_start + cycle_len];
    let first_t = transition_utc(transitions, cycle_start);
    if first_t == i64::MIN {
        return last_transition(transitions);
    }
    let elapsed = utc_unix - first_t;
    if elapsed < 0 {
        return last_transition(transitions);
    }
    let position_in_cycle = elapsed % period;

    let mut lo = 0usize;
    let mut hi = cycle.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let t_mid = transition_utc(transitions, cycle_start + mid);
        if (t_mid - first_t) <= position_in_cycle {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let best_j = if lo == 0 { 0 } else { lo - 1 };

    let t = &cycle[best_j];
    Some(OffsetInfo {
        offset: t.offset,
        abbrev: abbrev(t.abbrev_idx),
        is_gap: false,
        gap_size: 0,
    })
}

pub(crate) fn offset_info_at_utc(name: &str, utc_unix: i64) -> Option<OffsetInfo> {
    let (_, transitions, repeating) = get_tz_data(name)?;
    if transitions.is_empty() {
        return None;
    }

    let idx = find_transition_for_utc(transitions, utc_unix);

    let last_idx = transitions.len() - 1;
    let last_t_utc = transition_utc(transitions, last_idx);
    if utc_unix > last_t_utc {
        if let Repeating::Cycle { .. } = repeating {
            return resolve_far_future_utc(transitions, repeating, utc_unix);
        }
    }

    let t = &transitions[idx];
    Some(OffsetInfo {
        offset: t.offset,
        abbrev: abbrev(t.abbrev_idx),
        is_gap: false,
        gap_size: 0,
    })
}
"#,
    );

    // DATA_N arrays
    for (trans, &id) in &unique {
        let name = &data_names[id];
        output.push_str("#[cfg(feature = \"tz\")]\n");
        output.push_str(&format!("static {}: &[Transition] = &[\n", name));
        for t in trans {
            output.push_str(&format!(
                "    Transition {{ local_timestamp: {}, offset: {}, abbrev_idx: {} }},\n",
                t.local_timestamp, t.offset, t.abbrev_idx
            ));
        }
        output.push_str("];\n\n");
    }

    // DATA_0 for minimal mode
    let utc_trans = if let Some((trans, _)) = name_to_data.get("UTC") {
        trans.clone()
    } else if let Some((trans, _)) = name_to_data.get("Etc/UTC") {
        trans.clone()
    } else {
        vec![]
    };

    output.push_str("#[cfg(not(feature = \"tz\"))]\n");
    output.push_str("static DATA_0: &[Transition] = &[\n");
    for t in &utc_trans {
        output.push_str(&format!(
            "    Transition {{ local_timestamp: {}, offset: {}, abbrev_idx: {} }},\n",
            t.local_timestamp, t.offset, t.abbrev_idx
        ));
    }
    output.push_str("];\n\n");

    // TZ_ENTRIES (full)
    output.push_str("#[cfg(feature = \"tz\")]\n");
    output.push_str("pub static TZ_ENTRIES: &[(&str, &[Transition], Repeating)] = &[\n");
    for (name, data_name, repeating) in &entries {
        let repeating_str = match repeating {
            Repeating::None => "Repeating::None".to_string(),
            Repeating::Cycle {
                cycle_start,
                cycle_len,
                period,
            } => format!(
                "Repeating::Cycle {{ cycle_start: {}, cycle_len: {}, period: {} }}",
                cycle_start, cycle_len, period
            ),
        };
        output.push_str(&format!(
            "    (\"{}\", {}, {}),\n",
            name, data_name, repeating_str
        ));
    }
    output.push_str("];\n\n");

    // TZ_ENTRIES (minimal)
    output.push_str("#[cfg(not(feature = \"tz\"))]\n");
    output.push_str("pub static TZ_ENTRIES: &[(&str, &[Transition], Repeating)] = &[\n");
    for (name, _data_name, repeating) in &minimal_entries {
        let repeating_str = match repeating {
            Repeating::None => "Repeating::None".to_string(),
            Repeating::Cycle {
                cycle_start,
                cycle_len,
                period,
            } => format!(
                "Repeating::Cycle {{ cycle_start: {}, cycle_len: {}, period: {} }}",
                cycle_start, cycle_len, period
            ),
        };
        output.push_str(&format!("    (\"{}\", DATA_0, {}),\n", name, repeating_str));
    }
    output.push_str("];\n");

    fs::write("src/tz/tzdb.rs", output).unwrap();

    // Debug prints
    if let Some((trans, _)) = name_to_data.get("Africa/Accra") {
        println!("DEBUG: Africa/Accra has {} transitions", trans.len());
    }
    if let Some((trans, _)) = name_to_data.get("America/New_York") {
        println!("DEBUG: America/New_York has {} transitions", trans.len());
    }
    if let Some((trans, _)) = name_to_data.get("Europe/London") {
        println!("DEBUG: Europe/London has {} transitions", trans.len());
    }

    println!(
        "✅ Generated src/tz/tzdb.rs (version {}) with {} zones ({} unique tables, {} abbreviations)",
        version,
        entries.len(),
        unique.len(),
        abbrevs.len()
    );
    println!(
        "   Minimal mode: {} zones → all point to DATA_0",
        minimal_entries.len()
    );
}
