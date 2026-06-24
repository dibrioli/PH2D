//! Text palette formats: GIMP `.gpl` and a plain hex list (coolors.co / CSS).

use super::{PaletteData, PaletteError};
use std::fmt::Write as _;

/// Parse a GIMP `.gpl` palette: the `GIMP Palette` magic, an optional `Name:` / `Columns:` header,
/// then `R G B   [name]` rows (three 0–255 decimals). `#` comments and unknown header lines are
/// skipped; a row missing/over-ranged ints is dropped rather than failing the whole file.
///
/// # Errors
/// [`PaletteError`] when the `GIMP Palette` magic is absent.
pub fn parse_gpl(bytes: &[u8]) -> Result<PaletteData, PaletteError> {
    let text = String::from_utf8_lossy(bytes);
    let mut name = String::new();
    let mut colors = Vec::new();
    let mut saw_magic = false;
    for line in text.lines() {
        let line = line.trim_end_matches(['\r', ' ', '\t']);
        if line.is_empty() {
            continue;
        }
        if !saw_magic {
            // Some exporters prepend a UTF-8 BOM; match on the trimmed start.
            saw_magic = line.trim_start().starts_with("GIMP Palette");
            continue;
        }
        if let Some(rest) = line.strip_prefix("Name:") {
            name = rest.trim().to_string();
            continue;
        }
        if line.starts_with("Columns:") || line.starts_with('#') {
            continue;
        }
        let mut it = line.split_whitespace();
        if let (Some(r), Some(g), Some(b)) = (it.next(), it.next(), it.next())
            && let (Ok(r), Ok(g), Ok(b)) = (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>())
        {
            colors.push([r, g, b, 255]);
        }
    }
    if !saw_magic {
        return Err(PaletteError("not a GIMP palette (missing magic)"));
    }
    Ok(PaletteData { name, colors })
}

/// Serialize a GIMP `.gpl`: the magic + `Name:` + `Columns: 0`, then `R G B<TAB>hex` rows (the hex is
/// the per-swatch name GIMP/Inkscape show). Alpha is dropped — `.gpl` is RGB only.
#[must_use]
pub fn write_gpl(p: &PaletteData) -> Vec<u8> {
    let name = if p.name.trim().is_empty() {
        "Untitled"
    } else {
        p.name.trim()
    };
    let mut s = format!("GIMP Palette\nName: {name}\nColumns: 0\n#\n");
    for c in &p.colors {
        let _ = writeln!(
            s,
            "{:3} {:3} {:3}\t{:02x}{:02x}{:02x}",
            c[0], c[1], c[2], c[0], c[1], c[2]
        );
    }
    s.into_bytes()
}

/// Parse a hex colour list: the first `#RGB` / `#RRGGBB` / `#RRGGBBAA` token on each line (leading
/// `#` optional), so `#FF0000 red`, a bare `ff0000`, and `;`/`//` comment lines all behave. This is
/// what coolors.co, Adobe Color web, Paletton and CSS dumps export.
///
/// # Errors
/// [`PaletteError`] when no hex colour is found anywhere.
pub fn parse_hex_list(bytes: &[u8]) -> Result<PaletteData, PaletteError> {
    let text = String::from_utf8_lossy(bytes);
    let mut colors = Vec::new();
    for line in text.lines() {
        if let Some(c) = line.split_whitespace().find_map(parse_hex_token) {
            colors.push(c);
        }
    }
    if colors.is_empty() {
        return Err(PaletteError("no hex colours found"));
    }
    Ok(PaletteData {
        name: String::new(),
        colors,
    })
}

/// Serialize a hex list: one upper-case `#RRGGBB` per line (`#RRGGBBAA` when a swatch is translucent).
#[must_use]
pub fn write_hex_list(p: &PaletteData) -> Vec<u8> {
    let mut s = String::new();
    for c in &p.colors {
        if c[3] == 255 {
            let _ = writeln!(s, "#{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
        } else {
            let _ = writeln!(s, "#{:02X}{:02X}{:02X}{:02X}", c[0], c[1], c[2], c[3]);
        }
    }
    s.into_bytes()
}

/// A `#RGB` / `#RRGGBB` / `#RRGGBBAA` token → RGBA8 (`#RGB` doubles each nibble, Alpha defaults full).
fn parse_hex_token(tok: &str) -> Option<[u8; 4]> {
    let h = tok.trim().trim_start_matches('#');
    if !matches!(h.len(), 3 | 6 | 8) || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some(match h.len() {
        3 => {
            let nib = |i: usize| u8::from_str_radix(&h[i..i + 1], 16).map(|v| v * 17);
            [nib(0).ok()?, nib(1).ok()?, nib(2).ok()?, 255]
        }
        6 => [byte(0)?, byte(2)?, byte(4)?, 255],
        _ => [byte(0)?, byte(2)?, byte(4)?, byte(6)?],
    })
}
