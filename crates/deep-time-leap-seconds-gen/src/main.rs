//! Generate `src/utc/leap_seconds_list.rs` from an IANA `leap-seconds.list` file.

use deep_time::Dt;
use deep_time::utc::LeapSec;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_INPUT: &str = "tests/assets/leap-seconds.list.txt";
const DEFAULT_OUTPUT: &str = "src/utc/leap_seconds_list.rs";

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let check_only = args.iter().any(|a| a == "--check");
    let positional: Vec<&String> = args
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();

    let root = workspace_root();
    let input = root.join(
        positional
            .first()
            .map(|s| s.as_str())
            .unwrap_or(DEFAULT_INPUT),
    );
    let output = root.join(
        positional
            .get(1)
            .map(|s| s.as_str())
            .unwrap_or(DEFAULT_OUTPUT),
    );

    let source = fs::read_to_string(&input)?;
    let list = Dt::leap_sec_list_from_str(&source);
    if list.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("no leap second entries found in {}", input.display()),
        ));
    }

    let row_comments = row_comments_from_source(&source);
    if row_comments.len() != list.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "expected {} row comments, found {} in {}",
                list.len(),
                row_comments.len(),
                input.display()
            ),
        ));
    }

    let expires = file_expires_from_source(&source);
    let generated = format_output(&list, &row_comments, expires.as_deref());

    if check_only {
        let existing = fs::read_to_string(&output)?;
        if existing == generated {
            println!("{} is up to date", output.display());
            return Ok(());
        }
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} is out of date", output.display()),
        ));
    }

    fs::write(&output, &generated)?;
    rustfmt(&output);
    println!("wrote {}", output.display());
    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn row_comments_from_source(source: &str) -> Vec<String> {
    let mut comments = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        if parts.next().is_none() || parts.next().is_none() {
            continue;
        }

        let comment = trimmed
            .split_once('#')
            .map(|(_, c)| c.trim().to_string())
            .unwrap_or_default();
        comments.push(comment);
    }

    comments
}

fn file_expires_from_source(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim_start_matches('#').trim();
        if let Some(rest) = trimmed.strip_prefix("File expires on ") {
            return Some(rest.to_string());
        }
    }
    None
}

fn last_leap_second_label(comment: &str, leap_sec_after: i64) -> String {
    format!(
        "{} (TAI-UTC = {leap_sec_after} s)",
        iso_date_from_comment(comment)
    )
}

fn iso_date_from_comment(comment: &str) -> String {
    let mut parts = comment.split_whitespace();
    let day: u8 = parts.next().and_then(|d| d.parse().ok()).unwrap_or(1);
    let month = parts.next().unwrap_or("Jan");
    let year = parts.next().unwrap_or("1972");

    let month_num = match month {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => 1,
    };

    format!("{year}-{month_num:02}-{day:02}")
}

fn format_output(list: &[LeapSec], row_comments: &[String], expires: Option<&str>) -> String {
    let last = list.last().expect("non-empty list");
    let last_comment = row_comments.last().expect("matching comments");
    let last_label = last_leap_second_label(last_comment, last.leap_sec_after);

    let mut out = String::new();
    out.push_str("//! Leap seconds table from the official IANA\n");
    out.push_str(
        "//! [leap-seconds.list](https://data.iana.org/time-zones/data/leap-seconds.list)\n",
    );
    out.push_str(&format!("//! Last leap second: {last_label}\n"));
    if let Some(expires) = expires {
        out.push_str(&format!("//! File expires: {expires}\n"));
    }
    out.push('\n');
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]\n");
    out.push_str("pub struct LeapSec {\n");
    out.push_str("    pub ntp_timestamp: i64,\n");
    out.push_str("    pub leap_sec_after: i64,\n");
    out.push_str("    pub utc_sec: i64,\n");
    out.push_str("    pub tai_sec: i64,\n");
    out.push_str("}\n\n");
    out.push_str("pub const LEAP_SECS: &[LeapSec] = &[\n");

    for (entry, comment) in list.iter().zip(row_comments) {
        out.push_str("    LeapSec {\n");
        out.push_str(&format!(
            "        ntp_timestamp: {},\n",
            entry.ntp_timestamp
        ));
        out.push_str(&format!(
            "        leap_sec_after: {},\n",
            entry.leap_sec_after
        ));
        out.push_str(&format!("        utc_sec: {},\n", entry.utc_sec));
        out.push_str(&format!("        tai_sec: {},\n", entry.tai_sec));
        if comment.is_empty() {
            out.push_str("    },\n");
        } else {
            out.push_str(&format!("    }}, // {comment}\n"));
        }
    }

    out.push_str("];\n");
    out
}

fn rustfmt(path: &Path) {
    match Command::new("rustfmt").arg(path).status() {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!(
            "warning: rustfmt exited with status {} for {}",
            status,
            path.display(),
        ),
        Err(err) => eprintln!(
            "warning: could not run rustfmt on {}: {err}",
            path.display()
        ),
    }
}
