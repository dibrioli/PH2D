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
            for part in crate::keymap::split_keys(rhs)
                .into_iter()
                .filter(|p| *p != "-")
            {
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

    /// As teclas da tabela `NAMES` de `chord.rs` (todas, na mesma ordem).
    const NAMED: &[K] = &[
        K::KeyA,
        K::KeyB,
        K::KeyC,
        K::KeyD,
        K::KeyE,
        K::KeyF,
        K::KeyG,
        K::KeyH,
        K::KeyI,
        K::KeyJ,
        K::KeyK,
        K::KeyL,
        K::KeyM,
        K::KeyN,
        K::KeyO,
        K::KeyP,
        K::KeyQ,
        K::KeyR,
        K::KeyS,
        K::KeyT,
        K::KeyU,
        K::KeyV,
        K::KeyW,
        K::KeyX,
        K::KeyY,
        K::KeyZ,
        K::Digit0,
        K::Digit1,
        K::Digit2,
        K::Digit3,
        K::Digit4,
        K::Digit5,
        K::Digit6,
        K::Digit7,
        K::Digit8,
        K::Digit9,
        K::F1,
        K::F2,
        K::F3,
        K::F4,
        K::F5,
        K::F6,
        K::F7,
        K::F8,
        K::F9,
        K::F10,
        K::F11,
        K::F12,
        K::Enter,
        K::NumpadEnter,
        K::Escape,
        K::Tab,
        K::Space,
        K::Delete,
        K::Backspace,
        K::Insert,
        K::Home,
        K::End,
        K::PageUp,
        K::PageDown,
        K::ArrowLeft,
        K::ArrowRight,
        K::ArrowUp,
        K::ArrowDown,
        K::Quote,
        K::Minus,
        K::Equal,
        K::BracketLeft,
        K::BracketRight,
        K::Comma,
        K::Period,
        K::Backquote,
        K::Slash,
        K::Semicolon,
        K::Backslash,
        K::NumpadSubtract,
        K::NumpadAdd,
    ];

    fn sorted_keys(k: &Keymap, a: PixelAction) -> Vec<String> {
        let mut v: Vec<String> = k
            .keys_of(a)
            .into_iter()
            .map(|(c, x)| format!("{c}|{}", x.id()))
            .collect();
        v.sort();
        v
    }

    fn only(a: PixelAction, keys: &[(Chord, Context)]) -> Option<Keymap> {
        let mut k = Keymap::factory();
        for (c, _) in k.keys_of(a) {
            k.remove(a, c);
        }
        for &(c, x) in keys {
            k.assign_replacing(a, c, x).ok()?;
        }
        Some(k)
    }

    #[test]
    fn every_named_key_survives_the_user_file_round_trip() {
        let a = PixelAction::Outline;
        let mut tested = 0;
        for &key in NAMED {
            assert!(crate::key_name(key).is_some(), "{key:?} sem nome");
            let bare = Chord::new(key);
            let all = bare.ctrl().alt().shift();
            for set in [
                vec![(bare, Context::Normal)],
                vec![(all, Context::Normal)],
                vec![(bare, Context::Selection)],
                vec![
                    (all, Context::Normal),
                    (bare, Context::Selection),
                    (Chord::new(K::KeyQ).ctrl().alt(), Context::Normal),
                ],
            ] {
                let Some(k) = only(a, &set) else {
                    continue;
                };
                let (lido, rep) = Keymap::from_text(&k.to_text());
                assert_eq!(rep, TextReport::default(), "{key:?}: {}", k.to_text());
                assert_eq!(sorted_keys(&lido, a), sorted_keys(&k, a), "{}", k.to_text());
                let mut imp = only(a, &[]).expect("vazio");
                assert_eq!(imp.import_text(&k.to_text()), TextReport::default());
                assert_eq!(sorted_keys(&imp, a), sorted_keys(&k, a), "import {key:?}");
                tested += 1;
            }
        }
        assert!(tested >= NAMED.len() * 3, "{tested}");
    }

    #[test]
    fn files_written_with_the_old_comma_format_still_read() {
        let a = PixelAction::Outline;
        let comma = Chord::new(K::Comma);
        let old = "edit.outline = A, ,, Ctrl+,, , @Selection, B\nedit.fill = -\n";
        let (k, rep) = Keymap::from_text(old);
        assert_eq!(rep, TextReport::default());
        let want = only(
            a,
            &[
                (Chord::new(K::KeyA), Context::Normal),
                (comma, Context::Normal),
                (comma.ctrl(), Context::Normal),
                (comma, Context::Selection),
                (Chord::new(K::KeyB), Context::Normal),
            ],
        )
        .expect("livres");
        assert_eq!(sorted_keys(&k, a), sorted_keys(&want, a));
        assert!(k.keys_of(PixelAction::Fill).is_empty(), "`-` = nenhuma");
        let (hand, rep) = Keymap::from_text("edit.outline = A,B,Ctrl+,");
        assert_eq!(rep, TextReport::default());
        assert_eq!(hand.keys_of(a).len(), 3);
    }

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
