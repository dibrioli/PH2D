//! **O ficheiro do utilizador** (`~/.ph2d/pixel-keys.txt`, o delta de `Keymap::to_text`) e o Import
//! que SOMA (Aseprite R6). ⚠️ Ponte: quando a `ph2d-pixel-keys` tiver `user_path`/`load_user`/
//! `save_user`/`Keymap::import_text` (P12a), estas chamam-nas e este ficheiro sai.

use std::str::FromStr;

use ph2d_pixel_keys::{Chord, Context, Keymap, PixelAction, TextReport};

pub fn user_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".ph2d/pixel-keys.txt"))
}

pub fn load_user() -> (Keymap, TextReport) {
    let t = user_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    Keymap::from_text(&t)
}

/// Atómico: um irmão temporário e `rename`.
pub fn save_to(p: &std::path::Path, k: &Keymap) -> std::io::Result<()> {
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let tmp = p.with_extension("txt.tmp");
    std::fs::write(&tmp, k.to_text())?;
    std::fs::rename(tmp, p)
}

pub fn save_user(k: &Keymap) -> std::io::Result<()> {
    let p = user_path().ok_or_else(|| std::io::Error::other("HOME is not set"))?;
    save_to(&p, k)
}

/// Soma as teclas do texto às que há: a linha não tira as da acção (ao contrário do ficheiro do
/// utilizador); a tecla que ela dá sai de quem a tinha.
pub fn import_text(k: &mut Keymap, text: &str) -> TextReport {
    let mut rep = TextReport::default();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((id, rhs)) = line.split_once('=') else {
            rep.rejected.push(line.to_owned());
            continue;
        };
        let Some(a) = PixelAction::from_id(id.trim()) else {
            rep.unknown_ids.push(id.trim().to_owned());
            continue;
        };
        for part in rhs.split(',').map(str::trim).filter(|p| *p != "-") {
            let (tecla, ctx) = match part.split_once('@') {
                Some((t, c)) => (t, Context::from_id(c.trim())),
                None => (part, Some(Context::Normal)),
            };
            let ok = match (Chord::from_str(tecla.trim()), ctx) {
                (Ok(t), Some(c)) => k.assign_replacing(a, t, c).is_ok(),
                _ => false,
            };
            if !ok {
                rep.rejected.push(part.to_owned());
            }
        }
    }
    rep
}
