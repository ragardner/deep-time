//! Generate `src/utc/leap_seconds_list.rs` from an IANA `leap-seconds.list` file.
//!
//! This is a maintainer-only code generator for the `deep-time` workspace. It
//! reads the official leap-second table, emits a Rust source file containing
//! `LEAP_SECS`, and is exposed as the Cargo alias `gen-leap-seconds`.
//!
//! # Usage
//!
//! ```text
//! cargo gen-leap-seconds
//! cargo gen-leap-seconds --check
//! cargo gen-leap-seconds [INPUT] [OUTPUT]
//! ```
//!
//! Paths are resolved relative to the workspace root. Absolute paths and paths
//! that escape the workspace (including via `..` or symlinks) are rejected.
//!
//! # Check mode (`--check`)
//!
//! Without `--check`, the tool overwrites `src/utc/leap_seconds_list.rs` (or a
//! custom output path) with newly generated code. With `--check`, it does **not**
//! overwrite anything. Instead it answers one question: *would running the
//! generator change that file?*
//!
//! It reads the leap-seconds input, builds the Rust source, runs `rustfmt`, and
//! compares that result byte-for-byte to the current contents of
//! `src/utc/leap_seconds_list.rs`. If they are identical, it prints that the
//! file is up to date and exits 0. If they differ, it exits 1.
//!
//! Typical use: CI runs `cargo gen-leap-seconds --check` to fail the build when
//! someone updated the input data but forgot to regenerate the Rust file.
//!
//! # Requirements
//!
//! `rustfmt` must be available on `PATH`. Formatting is performed in memory
//! via `rustfmt --emit stdout` so no intermediate files are left behind for
//! that step.

use deep_time::Dt;
use deep_time::utc::LeapSec;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

/// Default IANA leap-seconds list location within the workspace.
const DEFAULT_INPUT: &str = "tests/assets/leap-seconds.list.txt";

/// Default generated Rust module written into the main crate.
const DEFAULT_OUTPUT: &str = "src/utc/leap_seconds_list.rs";

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

/// Parses CLI arguments, generates output, and either writes or checks the result.
fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let check_only = args.iter().any(|a| a == "--check");
    let positional: Vec<&String> = args
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();

    let root = workspace_root()?;
    let input = resolve_workspace_path(
        &root,
        positional
            .first()
            .map(|s| s.as_str())
            .unwrap_or(DEFAULT_INPUT),
    )?;
    let output = resolve_workspace_path(
        &root,
        positional
            .get(1)
            .map(|s| s.as_str())
            .unwrap_or(DEFAULT_OUTPUT),
    )?;

    let source = fs::read_to_string(&input)?;
    let list = Dt::leap_sec_list_from_str(&source);
    if list.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("no leap second entries found in {}", input.display()),
        ));
    }

    // Keep inline comments from the IANA file aligned with parsed rows.
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
    let formatted = format_with_rustfmt(&generated)?;

    if check_only {
        let existing = fs::read(&output)?;
        if existing == formatted {
            println!("{} is up to date", output.display());
            return Ok(());
        }
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} is out of date", output.display()),
        ));
    }

    write_atomically(&output, &formatted)?;
    println!("wrote {}", output.display());
    Ok(())
}

/// Returns the canonical path to the `deep-time` workspace root.
///
/// The generator crate lives at `crates/deep-time-leap-seconds-gen/`, so the
/// workspace root is two levels above `CARGO_MANIFEST_DIR`.
fn workspace_root() -> io::Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "could not locate workspace root from CARGO_MANIFEST_DIR",
            )
        })?
        .to_path_buf();
    root.canonicalize().map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("workspace root {} is unavailable: {err}", root.display()),
        )
    })
}

/// Resolves a workspace-relative path and verifies it stays inside the root.
///
/// Absolute paths, symlink targets, and normalized paths that leave the
/// workspace are rejected before any file I/O occurs.
fn resolve_workspace_path(root: &Path, rel: &str) -> io::Result<PathBuf> {
    let path = Path::new(rel);
    if path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("path must be relative to the workspace root: {rel}"),
        ));
    }

    let joined = normalize_path(&root.join(path));
    if joined.is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("refusing to use symlink path: {}", joined.display()),
        ));
    }
    resolve_within_root(root, &joined)
}

/// Collapses `.` and `..` components without touching the filesystem.
fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

/// Canonicalizes `path` and ensures the result is contained in `root`.
///
/// For paths that do not yet exist (typically a new output file), the parent
/// directory is canonicalized and the final filename is appended afterward so
/// containment can still be checked without creating the file.
fn resolve_within_root(root: &Path, path: &Path) -> io::Result<PathBuf> {
    let resolved = if path.exists() {
        path.canonicalize()?
    } else {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid path: {}", path.display()),
            )
        })?;
        let file_name = path.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid path: {}", path.display()),
            )
        })?;
        if !parent.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("parent directory does not exist: {}", parent.display()),
            ));
        }
        parent.canonicalize()?.join(file_name)
    };

    if !resolved.starts_with(root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "path escapes workspace root ({}): {}",
                root.display(),
                path.display()
            ),
        ));
    }
    Ok(resolved)
}

/// Formats Rust source in memory by piping it through `rustfmt`.
///
/// Using stdin/stdout avoids writing a temporary formatting file to disk.
/// The returned bytes are exactly what a write or `--check` comparison should
/// use.
fn format_with_rustfmt(source: &str) -> io::Result<Vec<u8>> {
    let mut child = Command::new("rustfmt")
        .args(["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| io::Error::new(err.kind(), format!("could not run rustfmt: {err}")))?;

    // Closing stdin signals EOF to rustfmt after the source has been sent.
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(source.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        return Ok(output.stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("rustfmt failed: {stderr}"),
    ))
}

/// Writes `contents` to `path` atomically via a same-directory temporary file.
///
/// The destination is replaced only after the temporary file is fully written
/// and synced, so an interrupted run cannot leave a truncated output file.
fn write_atomically(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid output path: {}", path.display()),
        )
    })?;
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid output path: {}", path.display()),
        )
    })?;

    let tmp_path = parent.join(format!(
        ".{}.{}.tmp",
        file_name.to_string_lossy(),
        std::process::id()
    ));
    let mut tmp = fs::File::create(&tmp_path)?;
    tmp.write_all(contents)?;
    tmp.sync_all()?;

    match fs::rename(&tmp_path, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = fs::remove_file(&tmp_path);
            Err(err)
        }
    }
}

/// Extracts trailing inline comments from data rows in the IANA source file.
///
/// Each non-comment line with at least two whitespace-separated fields is
/// treated as a leap-second row; text after `#` becomes the row comment.
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

/// Reads the `File expires on …` metadata line from the IANA file header.
fn file_expires_from_source(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim_start_matches('#').trim();
        if let Some(rest) = trimmed.strip_prefix("File expires on ") {
            return Some(rest.to_string());
        }
    }
    None
}

/// Builds the human-readable label used in the generated module documentation.
fn last_leap_second_label(comment: &str, leap_sec_after: i64) -> String {
    format!(
        "{} (TAI-UTC = {leap_sec_after} s)",
        iso_date_from_comment(comment)
    )
}

/// Converts an IANA row comment such as `1 Jan 2017` into `2017-01-01`.
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

/// Renders the generated Rust module source before `rustfmt` is applied.
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
    out.push_str("/// Holds info about a leap-second transition. Used by [LEAP_SECS](constant.LEAP_SECS.html).\n");
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]\n");
    out.push_str("pub struct LeapSec {\n");
    out.push_str("    /// NTP timestamp of the transition (IANA file, column 1)\n");
    out.push_str("    pub ntp_timestamp: i64,\n");
    out.push_str(
        "    /// Cumulative TAI-UTC offset in seconds after this transition (IANA column 2)\n",
    );
    out.push_str("    pub leap_sec_after: i64,\n");
    out.push_str("    /// Library timestamp of the transition on the UTC scale\n");
    out.push_str("    pub utc_sec: i64,\n");
    out.push_str("    /// Library timestamp of the transition on the TAI scale\n");
    out.push_str("    pub tai_sec: i64,\n");
    out.push_str("}\n\n");
    out.push_str("/// Embedded leap-seconds list shipped with the library.\n");
    out.push_str("///\n");
    out.push_str("/// Each entry records the instant when the cumulative TAI-UTC offset\n");
    out.push_str("/// changes. Rows are sorted chronologically.\n");
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
