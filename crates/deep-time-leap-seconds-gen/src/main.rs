//! Generate `src/utc/leap_seconds_list.rs` from an IANA `leap-seconds.list` file.
//!
//! Maintainer-only code generator for the `deep-time` workspace. It reads the
//! an IANA leap-second table, builds a Rust module (`LeapSec`, `LEAP_SECS`),
//! formats it with `rustfmt`, and writes the result or verifies it is already current.
//!
//! Exposed as the Cargo alias `gen-leap-seconds`.
//!
//! ## Usage
//!
//! ```text
//! cargo gen-leap-seconds
//! cargo gen-leap-seconds --check
//! cargo gen-leap-seconds [INPUT] [OUTPUT]
//! ```
//!
//! Default paths:
//!
//! - Input:  `tests/assets/leap-seconds.list.txt`
//! - Output: `src/utc/leap_seconds_list.rs`
//!
//! Run from the repository root. `[INPUT]` and `[OUTPUT]` are resolved relative
//! to the workspace root (from `CARGO_MANIFEST_DIR`), not the shell's current
//! directory. Absolute paths and paths that escape the workspace are rejected.
//!
//! ## Check mode (`--check`)
//!
//! Runs the same generation pipeline but does not write anything. Exits successfully
//! only when the output file already matches the freshly generated, `rustfmt`-formatted
//! content. Line endings are normalized before comparison so Git `autocrlf` on
//! Windows does not cause false failures.
//!
//! ## Requirements
//!
//! The `rustfmt` component must be installed and available on `PATH` (e.g. via
//! `rustup component add rustfmt`). Formatting is done in memory through
//! `rustfmt --emit stdout`; no temporary formatting files are created.

use deep_time::Dt;
use deep_time::utc::LeapSec;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

const DEFAULT_INPUT: &str = "tests/assets/leap-seconds.list.txt";
const DEFAULT_OUTPUT: &str = "src/utc/leap_seconds_list.rs";

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

/// Main entry point. Parses arguments, generates the output, and either writes
/// it or performs a check.
fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let check_only = args.iter().any(|a| a == "--check");

    let positional: Vec<&str> = args
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .collect();

    let root = workspace_root()?;

    let input = resolve_path(&root, positional.first().copied().unwrap_or(DEFAULT_INPUT))?;
    let output = resolve_path(&root, positional.get(1).copied().unwrap_or(DEFAULT_OUTPUT))?;

    let source = fs::read_to_string(&input)?;
    let list = Dt::leap_sec_list_from_str(&source);

    if list.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("no leap second entries found in {}", input.display()),
        ));
    }

    // Preserve inline comments from the IANA file so they appear in the generated code.
    let row_comments = row_comments_from_source(&source);
    if row_comments.len() != list.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "expected {} row comments, found {}",
                list.len(),
                row_comments.len()
            ),
        ));
    }

    let expires = file_expires_from_source(&source);

    // Build the entire output in memory first.
    let generated = format_output(&list, &row_comments, expires.as_deref());
    let formatted = format_with_rustfmt(&generated)?;

    if check_only {
        let existing = fs::read(&output)?;

        // Normalize line endings for comparison. This makes --check reliable on
        // Windows when Git has converted the file to CRLF due to core.autocrlf=true.
        let existing_norm = normalize_line_endings(&existing);
        let formatted_norm = normalize_line_endings(&formatted);

        if existing_norm == formatted_norm {
            println!("{} is up to date", output.display());
            return Ok(());
        }
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} is out of date", output.display()),
        ));
    }

    // Final write — no temporary files are created.
    fs::write(&output, &formatted)?;
    println!("wrote {}", output.display());
    Ok(())
}

/// Returns the canonical workspace root directory.
///
/// Canonicalizing here keeps containment checks consistent with resolved input
/// and output paths, which are also canonicalized before comparison.
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
        })?;

    canonicalize_safely(root).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("workspace root {} is unavailable: {err}", root.display()),
        )
    })
}

// -----------------------------------------------------------------------------
// Path handling
// -----------------------------------------------------------------------------

/// Resolves a relative path against the workspace root and verifies it does
/// not escape the workspace.
///
/// Paths are canonicalized before the containment check. Symlinks are followed
/// to their target; if the target lies outside the workspace, the path is
/// rejected.
fn resolve_path(root: &Path, rel: &str) -> io::Result<PathBuf> {
    let rel_path = Path::new(rel);

    if rel_path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("absolute paths are not allowed: {}", rel),
        ));
    }

    reject_unsafe_path_components(rel_path)?;

    let full_path = root.join(rel_path);

    // Canonicalize for the containment check. For non-existing files (the
    // common case for output), we canonicalize the parent and re-append the filename.
    let resolved = if full_path.exists() {
        canonicalize_safely(&full_path)?
    } else {
        let parent = full_path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid output path"))?;
        if !parent.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("parent directory does not exist: {}", parent.display()),
            ));
        }
        canonicalize_safely(parent)?.join(full_path.file_name().unwrap())
    };

    if !is_path_within(&resolved, root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("path escapes workspace root: {}", rel),
        ));
    }

    Ok(resolved)
}

/// Rejects paths containing drive letters, UNC prefixes, or absolute root components.
fn reject_unsafe_path_components(path: &Path) -> io::Result<()> {
    for component in path.components() {
        if matches!(component, Component::Prefix(_) | Component::RootDir) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "path must be relative to the workspace root: {}",
                    path.display()
                ),
            ));
        }
    }
    Ok(())
}

/// Canonicalize the path, then strip the Windows \\?\ prefix so containment
/// checks work reliably across platforms.
fn canonicalize_safely(path: &Path) -> io::Result<PathBuf> {
    let canonical = path.canonicalize()?;
    Ok(strip_verbatim_prefix(canonical))
}

/// Removes the Windows extended-length path prefix (`\\?\` and `\\?\UNC\`).
/// This is required because `canonicalize()` returns these prefixes on Windows,
/// which breaks naive `starts_with` checks.
fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let rendered = path.as_os_str().to_string_lossy();
        if let Some(stripped) = rendered.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{}", stripped));
        }
        if let Some(stripped) = rendered.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }
    path
}

/// Checks whether `path` is inside `root` after cleaning any Windows verbatim prefixes.
fn is_path_within(path: &Path, root: &Path) -> bool {
    let path = strip_verbatim_prefix(path.to_path_buf());
    let root = strip_verbatim_prefix(root.to_path_buf());
    path.starts_with(&root)
}

// -----------------------------------------------------------------------------
// Rest of the file
// -----------------------------------------------------------------------------

/// Runs rustfmt on the generated source in memory.
fn format_with_rustfmt(source: &str) -> io::Result<Vec<u8>> {
    let mut child = Command::new("rustfmt")
        .args(["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| io::Error::new(e.kind(), format!("could not run rustfmt: {e}")))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(source.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(io::Error::other(format!("rustfmt failed: {stderr}")))
    }
}

/// Normalizes CRLF line endings to LF.
///
/// This is used only during `--check` to make comparisons reliable on Windows
/// when Git converts files to CRLF due to `core.autocrlf = true`.
fn normalize_line_endings(bytes: &[u8]) -> Vec<u8> {
    if !bytes.contains(&b'\r') {
        return bytes.to_vec();
    }

    let mut result = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            result.push(b'\n');
            i += 2;
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }
    result
}

// -----------------------------------------------------------------------------
// IANA File Parsing
// -----------------------------------------------------------------------------

/// Extracts trailing `#` comments from data rows in the IANA file.
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

/// Extracts the "File expires on ..." line from the IANA header.
fn file_expires_from_source(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim_start_matches('#').trim();
        if let Some(rest) = trimmed.strip_prefix("File expires on ") {
            return Some(rest.to_string());
        }
    }
    None
}

// -----------------------------------------------------------------------------
// Code Generation
// -----------------------------------------------------------------------------

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

/// Renders the complete generated Rust module (including struct definition).
fn format_output(list: &[LeapSec], row_comments: &[String], expires: Option<&str>) -> String {
    let last = list.last().expect("list should be non-empty");
    let last_comment = row_comments
        .last()
        .expect("comments should match list length");
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

    // Struct definition (kept for compatibility with how the module is included)
    out.push_str("/// Holds info about a leap-second transition. Used by [LEAP_SECS](constant.LEAP_SECS.html).\n");
    out.push_str("#[derive(Debug, Clone, PartialEq, PartialOrd)]\n");
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
