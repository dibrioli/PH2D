//! ⭐⭐ **A pilha mono resolve para a JetBrains Mono EMBUTIDA e ela é monoespaçada** (CodeEditor T5).
//!
//! A régua é o ficheiro no disco (`include_bytes!` daqui, não a porta do produto): cada run moldado
//! pela pilha [`ph2d_tokens::FONT_MONO`] tem de trazer EXACTAMENTE esses bytes. O controlo negativo
//! (a pilha da interface não os traz) prova que a comparação discrimina.

use parley::{FontFamily, Layout, PositionedLayoutItem, StyleProperty};
use ph2d_text::{MONO_FAMILY, TextSystem};

const EMBEDDED: &[u8] = include_bytes!("../../fonts/JetBrainsMonoVariable.ttf");
const SIZE_PX: f32 = 14.0;

fn shape(sys: &mut TextSystem, text: &str, mono: bool) -> Layout<()> {
    let (fcx, lcx, stack) = if mono {
        sys.mono_contexts()
    } else {
        sys.contexts()
    };
    let stack = stack.to_string();
    let mut b = lcx.ranged_builder(fcx, text, 1.0, true);
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(stack.into())));
    b.push_default(StyleProperty::FontSize(SIZE_PX));
    let mut layout = b.build(text);
    layout.break_all_lines(None);
    layout
}

/// `(é a embutida, id do glifo, avanço)` de cada glifo, pela ordem.
fn glyphs(layout: &Layout<()>) -> Vec<(bool, u32, f32)> {
    let mut out = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(gr) = item {
                let ours = gr.run().font().data.data() == EMBEDDED;
                out.extend(gr.glyphs().map(|g| (ours, g.id, g.advance)));
            }
        }
    }
    out
}

#[test]
fn the_mono_stack_resolves_to_the_embedded_jetbrains_mono() {
    assert!(
        ph2d_tokens::FONT_MONO.starts_with(&format!("'{MONO_FAMILY}',")),
        "FONT_MONO tem de abrir pela família forçada {MONO_FAMILY:?}: {:?}",
        ph2d_tokens::FONT_MONO
    );
    let mut sys = TextSystem::without_system_fonts();
    let mono = glyphs(&shape(&mut sys, "fn main() {}", true));
    assert!(!mono.is_empty(), "a pilha mono não moldou nada");
    assert!(
        mono.iter().all(|g| g.0),
        "um glifo da pilha mono não veio da JetBrains Mono embutida: {mono:?}"
    );
    let ui = glyphs(&shape(&mut sys, "fn main() {}", false));
    assert!(
        !ui.is_empty() && ui.iter().all(|g| !g.0),
        "controlo: a pilha da interface leu a mono"
    );
}

#[test]
fn every_mono_glyph_has_the_same_advance() {
    let mut sys = TextSystem::without_system_fonts();
    let probes = ["i", "W", "0", "—", "m", " ", "ç"];
    let advances: Vec<(&str, f32)> = probes
        .iter()
        .map(|p| {
            let g = glyphs(&shape(&mut sys, p, true));
            assert_eq!(g.len(), 1, "{p:?} deu {} glifos", g.len());
            assert!(
                g[0].0 && g[0].1 != 0,
                "{p:?} não veio da mono embutida: {g:?}"
            );
            (*p, g[0].2)
        })
        .collect();
    let first = advances[0].1;
    assert!(first > 0.0);
    assert!(
        advances.iter().all(|a| a.1 == first),
        "avanços diferentes na mono: {advances:?}"
    );
}

#[test]
fn portuguese_and_latin1_have_real_mono_glyphs() {
    let mut sys = TextSystem::without_system_fonts();
    let text = "ação é ü ñ ç ÁÉÍÓÚ ãõ àâêô ÇÑÜ ¿¡ «» ° ß ø æ";
    let g = glyphs(&shape(&mut sys, text, true));
    assert_eq!(
        g.len(),
        text.chars().count(),
        "um carácter não virou um glifo"
    );
    let bad: Vec<_> = text
        .chars()
        .zip(&g)
        .filter(|(_, g)| !g.0 || g.1 == 0)
        .collect();
    assert!(bad.is_empty(), ".notdef ou fonte alheia em: {bad:?}");
}
