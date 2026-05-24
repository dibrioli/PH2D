//! `ph2d-loc-trend` — record per-file LOC over time and flag
//! emergent god-files.
//!
//! Wave 10 / Etapa 6.3. The CRITICAL_FILES list captures the
//! high-leverage files where uncontrolled growth has historically
//! resulted in unmaintainable code (the post-Wave-10 cleanup of
//! `editor-core/src/lib.rs`, `panel-color-equalization/src/paint.rs`
//! etc.). The tool reads `metrics/loc-trend.json`, appends today's
//! sample, and warns when a critical file grew > 10% in the last 30
//! days without an accompanying ADR-NNNN entry in `docs/architecture/`.
//!
//! Wire format (loc-trend.json):
//!
//! ```json
//! { "samples": [
//!     { "date": "2026-05-24", "files": { "crates/.../paint.rs": 318, ... } }
//! ]}
//! ```
//!
//! Dependency-free: minimal JSON serializer (we control the schema).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Files whose unbounded growth has historically required intervention.
/// Adding to this list requires a single-line justification.
pub const CRITICAL_FILES: &[&str] = &[
    // Editor-core orchestrators
    "crates/ph2d-editor-core/src/lib.rs",
    "crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs",
    "shells/desktop/src/render_loop/mod.rs",
    // Panel paint orchestrators (the ones that were split in Etapa 5.2
    // or are still at risk).
    "crates/ph2d-panel-color-equalization/src/paint.rs",
    "crates/ph2d-panel-color-equalization/src/paint_sections.rs",
    "crates/ph2d-panel-grid-snap/src/paint.rs",
    "crates/ph2d-panel-grid-snap/src/paint_helpers.rs",
    "crates/ph2d-panel-bgremoval/src/paint.rs",
    "crates/ph2d-panel-equalize-sizes/src/paint.rs",
    "crates/ph2d-panel-inspector/src/paint.rs",
    "crates/ph2d-panel-inspector/src/sections.rs",
    "crates/ph2d-panel-hierarchy/src/paint.rs",
    // Tool-runtime: contract gateway, must stay slim.
    "crates/ph2d-tool-runtime/src/lib.rs",
];

/// Maximum allowed growth ratio over `WINDOW_DAYS` without an ADR.
pub const MAX_GROWTH_PCT: f64 = 0.10;
pub const WINDOW_DAYS: i64 = 30;

/// A single sample recorded by `record_sample`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample {
    pub date: String, // YYYY-MM-DD
    pub files: BTreeMap<String, usize>,
}

/// In-memory representation of `loc-trend.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrendHistory {
    pub samples: Vec<Sample>,
}

/// Read and parse `metrics/loc-trend.json` (or return empty if absent).
pub fn read_history(path: &Path) -> TrendHistory {
    let Ok(src) = fs::read_to_string(path) else {
        return TrendHistory::default();
    };
    parse_json(&src).unwrap_or_default()
}

/// Compute today's sample by walking the workspace and counting LOC
/// for every entry in `CRITICAL_FILES` that exists.
pub fn snapshot_today(workspace_root: &Path, today: &str) -> Sample {
    let mut files = BTreeMap::new();
    for path in CRITICAL_FILES {
        let abs = workspace_root.join(path);
        if let Ok(src) = fs::read_to_string(&abs) {
            files.insert((*path).to_string(), src.lines().count());
        }
    }
    Sample {
        date: today.to_string(),
        files,
    }
}

/// Append `sample` to `history`, replacing any prior entry for the
/// same date.
pub fn append_sample(history: &mut TrendHistory, sample: Sample) {
    history.samples.retain(|s| s.date != sample.date);
    history.samples.push(sample);
    history.samples.sort_by(|a, b| a.date.cmp(&b.date));
}

/// Write `history` back to `path` (pretty-printed, one sample per
/// line for human-friendly diff).
pub fn write_history(path: &Path, history: &TrendHistory) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serialize_json(history))
}

/// Check growth for every file in the latest sample against the
/// nearest sample older than `WINDOW_DAYS`. Returns the list of
/// (file, old_loc, new_loc, growth_pct) tuples exceeding
/// `MAX_GROWTH_PCT`.
pub fn detect_growth(history: &TrendHistory) -> Vec<(String, usize, usize, f64)> {
    let mut out = Vec::new();
    let Some(latest) = history.samples.last() else {
        return out;
    };
    let cutoff = subtract_days(&latest.date, WINDOW_DAYS);
    // Find the most recent sample at or before `cutoff`.
    let baseline = history
        .samples
        .iter()
        .rev()
        .find(|s| s.date <= cutoff)
        .cloned();
    let Some(baseline) = baseline else {
        return out; // not enough history yet
    };
    for (file, &new_loc) in &latest.files {
        let Some(&old_loc) = baseline.files.get(file) else {
            continue;
        };
        if old_loc == 0 {
            continue;
        }
        let growth = (new_loc as f64 - old_loc as f64) / old_loc as f64;
        if growth > MAX_GROWTH_PCT {
            out.push((file.clone(), old_loc, new_loc, growth));
        }
    }
    out.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Subtract `days` from a YYYY-MM-DD date string. Uses Howard
/// Hinnant's proleptic Gregorian algorithm (correct for any date in
/// the proleptic range — same algorithm `epoch_secs_to_ymd` uses).
pub fn subtract_days(date: &str, days: i64) -> String {
    let Some(z) = ymd_to_serial(date) else {
        return date.to_string();
    };
    serial_to_ymd(z - days)
}

/// YYYY-MM-DD → days since proleptic Gregorian 0000-03-01 (the same
/// "serial" form `epoch_secs_to_ymd` produces internally).
fn ymd_to_serial(date: &str) -> Option<i64> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y / 400 } else { (y - 399) / 400 };
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// Inverse of `ymd_to_serial` — same algorithm as
/// `epoch_secs_to_ymd` but takes the "serial" form (days since
/// 1970-01-01) directly.
fn serial_to_ymd(serial: i64) -> String {
    let z = serial + 719_468;
    let era = if z >= 0 {
        z / 146_097
    } else {
        (z - 146_096) / 146_097
    };
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y_final = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y_final, m, d)
}

// ---------- ad-hoc JSON ----------
//
// Schema is fixed; we don't need serde_json. Serializer pretty-prints
// each sample on its own line so diff stays readable.

fn serialize_json(history: &TrendHistory) -> String {
    let mut out = String::from("{\n  \"samples\": [\n");
    for (i, s) in history.samples.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"date\": \"{}\",\n", s.date));
        out.push_str("      \"files\": {\n");
        let entries: Vec<String> = s
            .files
            .iter()
            .map(|(k, v)| format!("        \"{k}\": {v}"))
            .collect();
        out.push_str(&entries.join(",\n"));
        out.push_str("\n      }\n");
        out.push_str(if i + 1 == history.samples.len() {
            "    }\n"
        } else {
            "    },\n"
        });
    }
    out.push_str("  ]\n}\n");
    out
}

fn parse_json(src: &str) -> Option<TrendHistory> {
    let mut history = TrendHistory::default();
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Find next `"date": "<...>"`
        let Some(rel) = src[i..].find("\"date\":") else {
            break;
        };
        let date_start = i + rel + "\"date\":".len();
        let date_str = read_string(&src[date_start..])?;
        // Find the next `"files": { ... }` block.
        let Some(files_rel) = src[date_start..].find("\"files\":") else {
            break;
        };
        let files_start = date_start + files_rel + "\"files\":".len();
        let block_start = src[files_start..].find('{')?;
        let abs_block_start = files_start + block_start;
        let block_end = find_matching_brace(&src[abs_block_start..])?;
        let block = &src[abs_block_start + 1..abs_block_start + block_end];
        let files = parse_files_object(block);
        history.samples.push(Sample {
            date: date_str,
            files,
        });
        i = abs_block_start + block_end + 1;
    }
    history.samples.sort_by(|a, b| a.date.cmp(&b.date));
    Some(history)
}

fn read_string(src: &str) -> Option<String> {
    let start = src.find('"')?;
    let tail = &src[start + 1..];
    let end = tail.find('"')?;
    Some(tail[..end].to_string())
}

fn find_matching_brace(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_files_object(block: &str) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for line in block.lines() {
        let l = line.trim().trim_end_matches(',');
        if let Some((k, v)) = l.split_once(':') {
            let k = k.trim().trim_matches('"').to_string();
            let v: usize = v.trim().parse().unwrap_or(0);
            if !k.is_empty() {
                out.insert(k, v);
            }
        }
    }
    out
}

/// Today's date in YYYY-MM-DD, using `SystemTime::now()` and a
/// hand-rolled Y/M/D from epoch (zero deps).
pub fn today_utc() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    epoch_secs_to_ymd(secs)
}

fn epoch_secs_to_ymd(secs: i64) -> String {
    let days = secs / 86_400;
    serial_to_ymd(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_serialize_parse() {
        let mut h = TrendHistory::default();
        let mut files = BTreeMap::new();
        files.insert("crates/foo.rs".to_string(), 100);
        files.insert("crates/bar.rs".to_string(), 200);
        h.samples.push(Sample {
            date: "2026-05-24".to_string(),
            files,
        });
        let json = serialize_json(&h);
        let parsed = parse_json(&json).expect("parses");
        assert_eq!(parsed.samples.len(), 1);
        assert_eq!(parsed.samples[0].date, "2026-05-24");
        assert_eq!(parsed.samples[0].files.get("crates/foo.rs"), Some(&100));
    }

    #[test]
    fn append_sample_replaces_same_date() {
        let mut h = TrendHistory::default();
        let mut files = BTreeMap::new();
        files.insert("x".to_string(), 1);
        append_sample(
            &mut h,
            Sample {
                date: "2026-05-24".to_string(),
                files: files.clone(),
            },
        );
        files.insert("x".to_string(), 2);
        append_sample(
            &mut h,
            Sample {
                date: "2026-05-24".to_string(),
                files,
            },
        );
        assert_eq!(h.samples.len(), 1);
        assert_eq!(h.samples[0].files.get("x"), Some(&2));
    }

    #[test]
    fn growth_above_threshold_is_flagged() {
        let mut h = TrendHistory::default();
        let mut old_files = BTreeMap::new();
        old_files.insert("crates/p.rs".to_string(), 100);
        h.samples.push(Sample {
            date: "2026-04-15".to_string(),
            files: old_files,
        });
        let mut new_files = BTreeMap::new();
        new_files.insert("crates/p.rs".to_string(), 120);
        h.samples.push(Sample {
            date: "2026-05-24".to_string(),
            files: new_files,
        });
        let flagged = detect_growth(&h);
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].0, "crates/p.rs");
        assert!((flagged[0].3 - 0.20).abs() < 1e-6);
    }

    #[test]
    fn growth_below_threshold_is_silent() {
        let mut h = TrendHistory::default();
        let mut old = BTreeMap::new();
        old.insert("crates/p.rs".to_string(), 100);
        h.samples.push(Sample {
            date: "2026-04-15".to_string(),
            files: old,
        });
        let mut new = BTreeMap::new();
        new.insert("crates/p.rs".to_string(), 105);
        h.samples.push(Sample {
            date: "2026-05-24".to_string(),
            files: new,
        });
        assert!(detect_growth(&h).is_empty());
    }

    #[test]
    fn epoch_zero_is_unix_epoch() {
        assert_eq!(epoch_secs_to_ymd(0), "1970-01-01");
    }

    #[test]
    fn subtract_days_round_trips() {
        // Known: 2026-05-24 - 30 days = 2026-04-24
        assert_eq!(subtract_days("2026-05-24", 30), "2026-04-24");
        // Cross-month: 2026-03-05 - 10 days = 2026-02-23
        assert_eq!(subtract_days("2026-03-05", 10), "2026-02-23");
        // Cross-year: 2026-01-15 - 30 days = 2025-12-16
        assert_eq!(subtract_days("2026-01-15", 30), "2025-12-16");
        // Leap-day handling: 2024 was a leap year.
        assert_eq!(subtract_days("2024-03-01", 1), "2024-02-29");
    }
}
