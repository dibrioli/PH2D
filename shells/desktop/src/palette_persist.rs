//! Cross-session persistence for the colour picker's named palettes — a small text file under the
//! user's home (`~/.ph2d/palettes.txt`), so palettes survive a restart. Format: a `# PH2D palettes`
//! header, then `[name]` starts a palette and each following `RRGGBBAA` hex line is one swatch.
//! Best-effort, std-only (no serde / config-dir dep).

use std::path::PathBuf;

/// `~/.ph2d/palettes.txt`, or `None` when `$HOME` is unset (persistence is then skipped).
fn palettes_file() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".ph2d").join("palettes.txt"))
}

/// Serialize the named palettes to the text format (`# PH2D palettes` + `[name]` / `RRGGBBAA` lines).
#[must_use]
pub fn serialize(palettes: &[(String, Vec<[u8; 4]>)]) -> String {
    use std::fmt::Write as _;
    let mut s = String::from("# PH2D palettes\n");
    for (name, colors) in palettes {
        let _ = writeln!(s, "[{name}]");
        for c in colors {
            let _ = writeln!(s, "{:02X}{:02X}{:02X}{:02X}", c[0], c[1], c[2], c[3]);
        }
    }
    s
}

/// Parse the text format; unknown / malformed lines are skipped. Inverse of [`serialize`].
#[must_use]
pub fn parse(text: &str) -> Vec<(String, Vec<[u8; 4]>)> {
    let mut out: Vec<(String, Vec<[u8; 4]>)> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            out.push((name.to_string(), Vec::new()));
        } else if line.len() == 8
            && line.bytes().all(|b| b.is_ascii_hexdigit())
            && let Some((_, colors)) = out.last_mut()
        {
            let h = |i: usize| u8::from_str_radix(&line[i..i + 2], 16).unwrap_or(0);
            colors.push([h(0), h(2), h(4), h(6)]);
        }
    }
    out
}

/// Serialize + write the named palettes. A write/IO error is logged, never fatal.
pub fn save(palettes: &[(String, Vec<[u8; 4]>)]) {
    let Some(path) = palettes_file() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, serialize(palettes)) {
        eprintln!("[ph2d] palette save: {e}");
    }
}

/// Read + parse the saved palettes. Empty when the file is absent / unreadable / malformed.
#[must_use]
pub fn load() -> Vec<(String, Vec<[u8; 4]>)> {
    palettes_file()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|t| parse(&t))
        .unwrap_or_default()
}

/// A cheap order-sensitive FNV-1a hash of the set, so the host saves only when it really changed.
#[must_use]
pub fn hash(palettes: &[(String, Vec<[u8; 4]>)]) -> u64 {
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let feed = |b: u8, h: &mut u64| {
        *h ^= u64::from(b);
        *h = h.wrapping_mul(PRIME);
    };
    for (name, colors) in palettes {
        for &b in name.as_bytes() {
            feed(b, &mut h);
        }
        feed(0, &mut h); // name/colour boundary
        for c in colors {
            for &b in c {
                feed(b, &mut h);
            }
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::{hash, parse, serialize};

    fn sample() -> Vec<(String, Vec<[u8; 4]>)> {
        vec![
            (
                "Sunset".to_string(),
                vec![[231, 76, 60, 255], [241, 196, 15, 128]],
            ),
            ("Cool".to_string(), vec![[52, 152, 219, 255]]),
            ("Empty".to_string(), vec![]),
        ]
    }

    #[test]
    fn serialize_parse_round_trips() {
        let p = sample();
        assert_eq!(
            parse(&serialize(&p)),
            p,
            "names, swatches and 8-bit alpha survive a round-trip"
        );
    }

    #[test]
    fn parse_tolerates_noise_and_keeps_a_real_gimp_irrelevant_lines() {
        let text = "# header\n\n[Warm]\nFF0000FF\n  garbage \n00FF00FF\n[Mono]\n808080FF\n";
        let got = parse(text);
        assert_eq!(got.len(), 2);
        assert_eq!(
            got[0],
            ("Warm".to_string(), vec![[255, 0, 0, 255], [0, 255, 0, 255]])
        );
        assert_eq!(got[1].0, "Mono");
    }

    #[test]
    fn hash_changes_with_content() {
        let a = sample();
        let mut b = a.clone();
        assert_eq!(hash(&a), hash(&b), "same content → same hash");
        b[0].1[0][0] = 0; // tweak one channel
        assert_ne!(
            hash(&a),
            hash(&b),
            "a colour change flips the hash → triggers a save"
        );
    }
}
