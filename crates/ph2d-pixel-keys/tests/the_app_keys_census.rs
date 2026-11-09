//! **A lei dos conflitos de teclas, por CENSO da shell** (`docs/PixelArt/P12_teclas.md` §4; dono
//! 08/11: «cuide de não criar conflitos entre atalhos»). O conjunto de teclas do app NÃO é escrito
//! à mão: lê-se dos braços `KeyCode::…` da cadeia de teclado da shell que corre na aba do Pixel, com
//! os modificadores que o braço e os `if` que o cercam exigem. Cada tecla de fábrica que casa uma
//! tecla do app tem de estar declarada (`RESERVED_GLOBAL`, `SAME_MEANING`, `SHADOWED` ou
//! `OWNER_EXCEPTIONS`), e cada linha declarada de `RESERVED_GLOBAL`/`SHADOWED` tem de ainda existir.
//!
//! Fora do censo (e porquê): `keyboard_cadeia.rs` (o vetor e o Motion, só com a tool deles viva,
//! nunca na aba do Pixel), `keymap.rs` (as teclas do grafo, só com o cursor sobre o grafo) e os
//! `SAME_MEANING` (o dono é o widget focado do `ph2d-editor-core`, não um braço da shell).

use std::path::PathBuf;

use ph2d_pixel_keys::{Chord, FACTORY, OWNER_EXCEPTIONS, RESERVED_GLOBAL, SAME_MEANING, SHADOWED};

/// `(ficheiro, marca a partir da qual as letras soltas já não levam acorde)`.
const CADEIA: &[(&str, Option<&str>)] = &[
    ("input_dispatch/keyboard.rs", None),
    ("input_dispatch/posse_do_jogo.rs", None),
    ("input_dispatch/keyboard_files.rs", None),
    ("input_dispatch/keyboard_timeline.rs", None),
    ("input_dispatch/keyboard_hierarchy.rs", None),
    (
        "input_handlers.rs",
        Some("letra_solta::acorde_em_letra_solta"),
    ),
    ("input_dispatch/handlers_teclas_editor.rs", Some("")),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Need {
    Any,
    On,
    Off,
}

impl Need {
    fn admits(self, b: bool) -> bool {
        match self {
            Self::Any => true,
            Self::On => b,
            Self::Off => !b,
        }
    }
}

#[derive(Debug, Clone)]
struct AppKey {
    key: String,
    ctrl: Need,
    shift: Need,
    alt: Need,
    at: String,
}

impl AppKey {
    fn hit(&self, c: Chord) -> bool {
        format!("{:?}", c.key) == self.key
            && self.ctrl.admits(c.ctrl)
            && self.shift.admits(c.shift)
            && self.alt.admits(c.alt)
    }
}

fn shell() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../shells/desktop/src")
}

fn need(t: &str, off: &[&str], on: &[&str]) -> Need {
    if off.iter().any(|p| t.contains(p)) {
        Need::Off
    } else if on.iter().any(|p| t.contains(p)) {
        Need::On
    } else {
        Need::Any
    }
}

/// As letras com braço solto, lidas de `letra_solta.rs` (a mesma tabela que o despacho usa).
fn letras_soltas() -> Vec<char> {
    let src = std::fs::read_to_string(shell().join("input_dispatch/letra_solta.rs")).unwrap();
    let l = src.lines().find(|l| l.contains("LETRAS_SOLTAS:")).unwrap();
    let corpo = &l[l.find("= [").unwrap()..];
    corpo
        .split('\'')
        .skip(1)
        .step_by(2)
        .filter_map(|s| s.chars().next())
        .collect()
}

/// O censo: cada `KeyCode::X` da cadeia, com o que o braço e os `if` abertos exigem.
fn app_keys() -> Vec<AppKey> {
    let soltas = letras_soltas();
    let mut out = Vec::new();
    for &(file, marca) in CADEIA {
        let src = std::fs::read_to_string(shell().join(file)).unwrap();
        let lines: Vec<&str> = src
            .lines()
            .take_while(|l| !l.trim_start().starts_with("#[cfg(test)]"))
            .map(|l| l.split("//").next().unwrap_or("").trim())
            .collect();
        let mut solta = marca == Some("");
        let (mut stack, mut stmt) = (Vec::<String>::new(), String::new());
        for (i, line) in lines.iter().enumerate() {
            if let Some(m) = marca
                && !m.is_empty()
                && line.contains(m)
            {
                solta = true;
            }
            let mut rest = *line;
            while let Some(p) = rest.find("KeyCode::") {
                let name: String = rest[p + 9..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric())
                    .collect();
                rest = &rest[p + 9..];
                let guarda: String = lines[i..lines.len().min(i + 4)]
                    .iter()
                    .scan(false, |fim, l| {
                        (!*fim).then(|| {
                            *fim = l.contains("=>");
                            l.split("=>").next().unwrap_or("").to_string()
                        })
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let ifs: Vec<&str> = stack
                    .iter()
                    .filter(|h| h.starts_with("if ") || h.contains("=>"))
                    .map(String::as_str)
                    .collect();
                let t = format!("{} {stmt} {guarda}", ifs.join(" "));
                let mut ctrl = need(
                    &t,
                    &[
                        "!cmd_chord",
                        "!self.modifiers.control_key()",
                        "!f.cmd",
                        "!ctrl",
                    ],
                    &["cmd_chord", "control_key()", "f.cmd", "ctrl"],
                );
                let mut alt = need(&t, &["!self.modifiers.alt_key()"], &["alt_key()"]);
                let letra = name.strip_prefix("Key").filter(|s| s.len() == 1);
                if solta
                    && ctrl == Need::Any
                    && letra.is_some_and(|l| soltas.contains(&l.chars().next().unwrap()))
                {
                    (ctrl, alt) = (Need::Off, Need::Off);
                }
                out.push(AppKey {
                    key: name,
                    ctrl,
                    shift: need(&t, &["!self.modifiers.shift_key()"], &["shift_key()"]),
                    alt,
                    at: format!("{file}:{}", i + 1),
                });
            }
            for ch in line.chars() {
                match ch {
                    '{' => stack.push(std::mem::take(&mut stmt).trim().to_string()),
                    '}' => {
                        stack.pop();
                        stmt.clear();
                    }
                    ';' | ',' => stmt.clear(),
                    c => stmt.push(c),
                }
            }
            stmt.push(' ');
        }
    }
    out
}

fn declared(c: Chord) -> bool {
    RESERVED_GLOBAL.iter().any(|(r, _)| *r == c)
        || SAME_MEANING.iter().any(|(r, ..)| *r == c)
        || SHADOWED.iter().any(|(r, ..)| *r == c)
        || OWNER_EXCEPTIONS.iter().any(|(_, r, _)| *r == c)
}

#[test]
fn every_factory_key_that_meets_an_app_key_is_declared() {
    let app = app_keys();
    assert!(
        app.len() > 30,
        "o censo leu {} teclas: a régua partiu",
        app.len()
    );
    let mut falta = Vec::new();
    for &(a, c, x) in FACTORY {
        if declared(c) {
            continue;
        }
        for k in app.iter().filter(|k| k.hit(c)) {
            falta.push(format!("{c} ({a:?}, {x:?}) × {} ({:?})", k.at, k));
        }
    }
    assert!(
        falta.is_empty(),
        "teclas do Pixel que casam uma tecla do app sem declaração:\n{}",
        falta.join("\n")
    );
}

#[test]
fn every_declared_app_key_still_exists_in_the_shell() {
    let app = app_keys();
    let mortas: Vec<String> = RESERVED_GLOBAL
        .iter()
        .map(|(c, _)| *c)
        .chain(SHADOWED.iter().map(|(c, ..)| *c))
        .filter(|&c| !app.iter().any(|k| k.hit(c)))
        .map(|c| c.to_string())
        .collect();
    assert!(mortas.is_empty(), "declaradas sem tecla no app: {mortas:?}");
}

#[test]
fn no_factory_key_is_a_reserved_one() {
    for &(a, c, _) in FACTORY {
        assert!(
            !RESERVED_GLOBAL.iter().any(|(r, _)| *r == c),
            "{c} ({a:?}) é do app"
        );
    }
}
