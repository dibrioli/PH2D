//! **O ficheiro do utilizador** (`~/.ph2d/pixel-keys.txt`, o delta de [`Keymap::to_text`]) e o Import
//! que SOMA (Aseprite R6, `docs/PixelArt/P12_teclas.md` §5). O texto é puro; a escrita é atómica.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::{Chord, Context, Keymap, PixelAction, TextReport};

/// `~/.ph2d/pixel-keys.txt` (`None` sem `HOME`).
#[must_use]
pub fn user_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".ph2d/pixel-keys.txt"))
}

/// O mapa do ficheiro `p` (ausente ou ilegível = a fábrica).
#[must_use]
pub fn load_from(p: &Path) -> (Keymap, TextReport) {
    Keymap::from_text(&std::fs::read_to_string(p).unwrap_or_default())
}

/// O mapa do ficheiro do utilizador.
#[must_use]
pub fn load_user() -> (Keymap, TextReport) {
    user_path().map_or_else(|| Keymap::from_text(""), |p| load_from(&p))
}

/// Grava `text` em `p` atomicamente: um irmão temporário e `rename` (um corte a meio deixa o antigo).
pub fn write_atomic(p: &Path, text: &str) -> std::io::Result<()> {
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut tmp = p.as_os_str().to_owned();
    tmp.push(".tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(tmp, p)
}

/// Grava o delta de `k` em `p`.
pub fn save_to(p: &Path, k: &Keymap) -> std::io::Result<()> {
    write_atomic(p, &k.to_text())
}

/// Grava o delta de `k` no ficheiro do utilizador.
pub fn save_user(k: &Keymap) -> std::io::Result<()> {
    let p = user_path().ok_or_else(|| std::io::Error::other("HOME is not set"))?;
    save_to(&p, k)
}

impl Keymap {
    /// **Import**: SOMA as teclas do texto às que há (a linha não tira as da acção, ao contrário do
    /// ficheiro do utilizador); a tecla que ela dá sai de quem a tinha. `-` não tira nada.
    pub fn import_text(&mut self, text: &str) -> TextReport {
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
                    (Ok(t), Some(c)) => self.assign_replacing(a, t, c).is_ok(),
                    _ => false,
                };
                if !ok {
                    rep.rejected.push(part.to_owned());
                }
            }
        }
        rep
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::KeyCode as K;

    #[test]
    fn import_adds_and_the_atomic_write_round_trips() {
        let mut k = Keymap::factory();
        let antes = k.keys_of(PixelAction::Outline).len();
        let rep = k.import_text("edit.outline = F12\nnope.x = F11\nedit.fill = Ctrl+K");
        assert_eq!(k.keys_of(PixelAction::Outline).len(), antes + 1);
        assert_eq!(rep.unknown_ids, ["nope.x"]);
        assert_eq!(rep.rejected, ["Ctrl+K"]);
        let dir = std::env::temp_dir().join(format!("ph2d-pixel-keys-{}", std::process::id()));
        let p = dir.join("keys.txt");
        save_to(&p, &k).expect("grava");
        assert!(!dir.join("keys.txt.tmp").exists());
        let (lido, _) = load_from(&p);
        assert_eq!(
            lido.resolve(Chord::new(K::F12), Context::Normal),
            Some(PixelAction::Outline)
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
