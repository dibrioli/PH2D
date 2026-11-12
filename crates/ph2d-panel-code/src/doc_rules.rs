//! As regras puras da aba de scripts que a `ph2d-app-code` aplica (os acordes do teclado, o piscar do
//! cursor, o tempo de jogo do console, o tom do link): sem estado da aba, testáveis sozinhas.

use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_i18n::tr_with;
use std::ops::Range;

/// As letras dos acordes Ctrl/Cmd que o editor usa (no texto e nos campos da busca).
/// `/` comenta (Ctrl+/); `[`/`]` com Shift dobram/desdobram — a shell traduz a tecla FÍSICA
/// (`keyboard_board.rs`), porque com Ctrl o texto da tecla não chega.
const CHORDS: [char; 14] = [
    '/', '[', ']', 'a', 'c', 'd', 'f', 'g', 'h', 's', 'v', 'x', 'y', 'z',
];

/// O acorde é do editor? Os outros (Ctrl+K a paleta, Ctrl+O abrir…) NÃO são tomados: a shell segue
/// para o atalho dela.
pub fn foreign_chord(key: Key, mods: Modifiers) -> bool {
    (mods.ctrl || mods.meta)
        && !problems_chord(key, mods)
        && match key {
            Key::Char(c) => !CHORDS.contains(&c),
            Key::Other => true,
            _ => false,
        }
}

/// Ctrl+Shift+M: a lista de problemas abre/fecha (o atalho do VS Code; o mesmo que o clique no
/// contador da barra). Só com o Shift: o Ctrl+M segue para a shell.
pub fn problems_chord(key: Key, mods: Modifiers) -> bool {
    (mods.ctrl || mods.meta) && mods.shift && matches!(key, Key::Char('m' | 'M'))
}

/// Ctrl+Espaço: a lista de completação na hora — só com o TEXTO do script em foco (decisão do dono,
/// 17/10); num campo (busca, lista de scripts) segue para a shell, onde é o modo foco.
pub fn suggest_chord(key: Key, mods: Modifiers) -> bool {
    (mods.ctrl || mods.meta) && key == Key::Space
}

/// O meio-período do piscar do cursor: o token `Slow` (o do piscar do F4, 1,25 Hz — abaixo dos 3 Hz
/// da WCAG 2.3.1). Aceso um período, apagado outro; uma tecla recomeça-o aceso.
pub fn blink_on(now_ns: u128, last_input_ns: u128, steady: bool) -> bool {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let half = (f64::from(ph2d_tokens::Duration::Slow.ms()) * 1e6) as u128;
    steady || (now_ns.saturating_sub(last_input_ns) / half.max(1)).is_multiple_of(2)
}

/// O tempo de jogo como a linha o mostra (`m:ss`).
pub fn game_time(at_us: u64) -> String {
    let s = at_us / 1_000_000;
    let (min, sec) = (s / 60, format!("{:02}", s % 60));
    tr_with("code.console.time", &[("min", &min), ("sec", &sec)])
}

/// As cores `spans` (ordenadas, sem sobreposição) com o nome `link` na cor do link: os trechos que o
/// tocam partem-se à volta dele.
pub fn tint(
    spans: Vec<(Range<usize>, crate::Ink)>,
    link: &Range<usize>,
) -> Vec<(Range<usize>, crate::Ink)> {
    let mut out = Vec::with_capacity(spans.len() + 2);
    for (r, ink) in spans {
        if r.end <= link.start || r.start >= link.end {
            out.push((r, ink));
            continue;
        }
        if r.start < link.start {
            out.push((r.start..link.start, ink));
        }
        if r.end > link.end {
            out.push((link.end..r.end, ink));
        }
    }
    out.push((link.clone(), crate::Ink::Link));
    out.sort_by_key(|(r, _)| r.start);
    out
}
