//! ⭐⭐ **As tintas do editor de scripts LEEM-SE e DISTINGUEM-SE em todo tema** (CodeEditor, 15/10).
//!
//! Desenho por REGRA, não por gosto: o benefício do realce de sintaxe é fraco na literatura, e o que
//! sobra dele é legibilidade e distinção. Então a régua é:
//!
//! - **ler-se** — toda tinta de texto (as dez `code-*`, mais `text-1` das identificadoras e `text-2`
//!   da pontuação) a ≥ 4,5:1 (WCAG 2.2 AA 1.4.3) sobre o `code-bg` **e** sobre a
//!   `code-line-current`, que é onde o olho está a ler; o sublinhado de erro (`danger`) a ≥ 3:1
//!   (1.4.11, um gráfico); os números de linha (`text-3`) a ≥ 3:1 sobre a calha;
//! - **distinguir-se** — distância euclidiana em OKLab sobre as cores JÁ em 8 bits. O limiar de
//!   percepção em OKLab é ≈ `0,02`; um glifo é um campo pequeno e fino, onde a discriminação de cor
//!   cai várias vezes, logo os pares que se encostam no código e carregam o sentido
//!   (keyword × function × string × number × comment × type) pedem `0,10` (5×); qualquer outro par
//!   de tintas `0,07`; e cada tinta contra o texto simples `0,05` (senão ela não realça nada);
//! - **não ser um buraco** — o `code-bg` é o fundo do painel do tema (ΔE ≤ `0,02`), e a linha do
//!   cursor e a calha separam-se dele.
//!
//! Medido no commit que criou os tokens: pior contraste `4,60:1` (`code-number` na linha do cursor,
//! Candy); pior par importante `0,102` (function × comment, escuro); pior par qualquer `0,073`
//! (type × property).

use ph2d_tokens::contrast::CONTRAST_PAIRS;
use ph2d_tokens::{Color, ColorToken as C, Theme, srgb_to_oklch};

const AA_TEXT: f64 = 4.5;
const AA_GRAPHIC: f64 = 3.0;
const MIN_DE_MEANING: f64 = 0.10;
const MIN_DE_ANY: f64 = 0.07;
const MIN_DE_PLAIN: f64 = 0.05;
const MAX_DE_FROM_PANEL: f64 = 0.02;
const MIN_DE_SURFACE: f64 = 0.01;

/// Os pares que carregam o sentido do código e se encostam numa linha.
const MEANING: [C; 6] = [
    C::CodeKeyword,
    C::CodeFunction,
    C::CodeString,
    C::CodeNumber,
    C::CodeComment,
    C::CodeType,
];

/// A população sai da TABELA de contraste (o que o painel *Tokens* também mede), nunca de um prefixo.
fn inks() -> Vec<C> {
    CONTRAST_PAIRS
        .iter()
        .filter(|p| p.bg == C::CodeBg)
        .map(|p| p.fg)
        .collect()
}

fn lab(c: Color) -> [f64; 3] {
    let (l, ch, h) = srgb_to_oklch(c.r, c.g, c.b);
    let h = h.to_radians();
    [l, ch * h.cos(), ch * h.sin()]
}

fn de(a: Color, b: Color) -> f64 {
    let (x, y) = (lab(a), lab(b));
    ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2) + (x[2] - y[2]).powi(2)).sqrt()
}

fn opaque(c: Color) -> Color {
    Color { a: 0xFF, ..c }
}

#[test]
fn the_code_inks_read_and_differ() {
    let inks = inks();
    assert_eq!(inks.len(), 10, "a tabela perdeu tintas do editor: {inks:?}");
    for t in MEANING {
        assert!(inks.contains(&t), "{} saiu da tabela de contraste", t.key());
    }
    let mut falhas = Vec::new();
    for theme in Theme::ALL {
        let r = |t: C| t.resolve(theme);
        let (bg, line, gutter) = (r(C::CodeBg), r(C::CodeLineCurrent), r(C::CodeGutter));
        for &ink in inks.iter().chain(&[C::Text1, C::Text2]) {
            for (sobre, fundo) in [("code-bg", bg), ("code-line-current", line)] {
                let k = r(ink).contrast_ratio(&fundo);
                if k < AA_TEXT {
                    falhas.push(format!("{theme:?}: {} sobre {sobre} = {k:.2}", ink.key()));
                }
            }
        }
        for (sobre, fundo) in [("code-bg", bg), ("code-line-current", line)] {
            let k = r(C::Danger).contrast_ratio(&fundo);
            if k < AA_GRAPHIC {
                falhas.push(format!("{theme:?}: danger sobre {sobre} = {k:.2}"));
            }
        }
        let k = r(C::Text3).contrast_ratio(&gutter);
        if k < AA_GRAPHIC {
            falhas.push(format!("{theme:?}: text-3 sobre code-gutter = {k:.2}"));
        }
        let d = de(bg, opaque(r(C::PanelBg)));
        if d > MAX_DE_FROM_PANEL {
            falhas.push(format!("{theme:?}: code-bg a {d:.3} do painel — um buraco"));
        }
        for (nome, s) in [("code-line-current", line), ("code-gutter", gutter)] {
            let d = de(s, bg);
            if d < MIN_DE_SURFACE {
                falhas.push(format!("{theme:?}: {nome} a {d:.3} do code-bg — invisível"));
            }
        }
        for (i, &a) in inks.iter().enumerate() {
            let d = de(r(a), r(C::Text1));
            if d < MIN_DE_PLAIN {
                falhas.push(format!("{theme:?}: {} a {d:.3} do text-1", a.key()));
            }
            for &b in &inks[i + 1..] {
                let min = if MEANING.contains(&a) && MEANING.contains(&b) {
                    MIN_DE_MEANING
                } else {
                    MIN_DE_ANY
                };
                let d = de(r(a), r(b));
                if d < min {
                    falhas.push(format!(
                        "{theme:?}: {} × {} = {d:.3} < {min}",
                        a.key(),
                        b.key()
                    ));
                }
            }
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
