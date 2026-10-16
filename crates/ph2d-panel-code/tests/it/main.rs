//! Um binário de teste para a crate (convenção do repo: teste novo = ficheiro aqui + linha `mod`).
//! Sem placa: pinta-se numa `VectorScene` e afirma-se pelos instrumentos do `CodeView`.

mod a11y;
mod clique;
mod cronometro;
mod estado;
mod ligaduras;
mod medicoes_do_fecho;
mod sublinhado;
mod virtualizacao;

use ph2d_code_buffer::Buffer;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::{Badge, CodeFrame, CodeView, FoldMark, Ink, Problem};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::ops::Range;

pub const TAB: usize = 4;

/// `n` linhas DISTINTAS de código com ~`largura` caracteres.
pub fn texto(n: usize, largura: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        let mut l = format!("local v{i} = math.sin({i}) * {i} -- linha {i}");
        while l.len() < largura {
            l.push_str(" + x");
        }
        s.push_str(&l);
        if i + 1 < n {
            s.push('\n');
        }
    }
    s
}

/// Realce de brincar: palavras, números e pontuação em tintas diferentes (bytes do buffer).
pub fn spans_de(buf: &Buffer, r: Range<usize>) -> Vec<(Range<usize>, Ink)> {
    let s = buf.text().slice(r.clone());
    let mut out = Vec::new();
    let mut cur: Option<(usize, Ink)> = None;
    let class = |c: char| {
        if c.is_alphabetic() || c == '_' {
            Some(Ink::Keyword)
        } else if c.is_ascii_digit() {
            Some(Ink::Number)
        } else if c.is_whitespace() {
            None
        } else {
            Some(Ink::Operator)
        }
    };
    for (i, c) in s.char_indices().chain(std::iter::once((s.len(), ' '))) {
        let k = class(c);
        if cur.map(|c| c.1) != k {
            if let Some((a, ink)) = cur.take() {
                out.push((r.start + a..r.start + i, ink));
            }
            cur = k.map(|ink| (i, ink));
        }
    }
    out
}

#[derive(Default)]
pub struct Deco {
    pub problems: Vec<Problem>,
    pub folds: Vec<FoldMark>,
    pub matches: Vec<Range<usize>>,
    pub current: Option<usize>,
    pub bracket: Option<(usize, usize)>,
    pub preedit: Option<String>,
    pub badge: Option<Badge>,
}

pub fn ts() -> TextSystem {
    TextSystem::without_system_fonts()
}

/// As dobras de `d` que começam nas linhas pedidas (o que a família responde à pintura).
pub fn marcas(d: &Deco) -> impl FnMut(Range<usize>) -> Vec<FoldMark> + '_ {
    |lines| {
        d.folds
            .iter()
            .filter(|f| lines.contains(&f.start))
            .copied()
            .collect()
    }
}

pub fn frame<'a>(
    buf: &'a Buffer,
    d: &'a Deco,
    spans: &'a mut dyn FnMut(Range<usize>) -> Vec<(Range<usize>, Ink)>,
    folds: &'a mut dyn FnMut(Range<usize>) -> Vec<FoldMark>,
) -> CodeFrame<'a> {
    CodeFrame {
        buffer: buf,
        spans,
        problems: &d.problems,
        folds,
        closed: &d.folds,
        bracket: d.bracket,
        matches: &d.matches,
        current_match: d.current,
        preedit: d.preedit.as_deref(),
        caret_on: true,
        tab: TAB,
        badge: d.badge.as_ref(),
        project_unsaved: false,
    }
}

pub fn pinta(
    v: &mut CodeView,
    buf: &Buffer,
    ts: &mut TextSystem,
    area: Rect,
    d: &Deco,
) -> VectorScene {
    let mut scene = VectorScene::new();
    pinta_em(v, buf, ts, area, d, &mut scene);
    scene
}

pub fn pinta_em(
    v: &mut CodeView,
    buf: &Buffer,
    ts: &mut TextSystem,
    area: Rect,
    d: &Deco,
    scene: &mut VectorScene,
) {
    let mut f = |r: Range<usize>| spans_de(buf, r);
    let mut m = marcas(d);
    v.paint(
        frame(buf, d, &mut f, &mut m),
        area,
        scene,
        ts,
        &Theme::default(),
    );
}

/// A área onde cabem exatamente `rows` linhas de texto (mede-se com uma pintura de aquecimento).
pub fn area_para(v: &mut CodeView, buf: &Buffer, ts: &mut TextSystem, rows: usize) -> Rect {
    pinta(
        v,
        buf,
        ts,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 1200.0,
            h: 400.0,
        },
        &Deco::default(),
    );
    let m = *v.metrics();
    Rect {
        x: 0.0,
        y: 0.0,
        w: 1200.0,
        h: rows as f32 * m.line_h + m.status_h,
    }
}
