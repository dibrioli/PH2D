//! **Nota adesiva ↔ cartão** (o Miro): o título é a 1.ª linha da nota e o resto vira a descrição em
//! markdown (negrito, itálico e riscado passam; sublinhado e cor de letra o markdown não tem); a cor
//! da nota vira a etiqueta de cor mais próxima (OKLab, sobre as cores do tema claro), e o inverso.

use ph2d_board_model::{Marks, Rgba, RichText, STICKY_COLORS, STICKY_DEFAULT_COLOR, Span};
use ph2d_kanban_model::{Hue, LabelColor, Shade};
use ph2d_richdoc::{Block, BlockKind, Inline, Mark, RichDoc};
use ph2d_tokens::Theme;

/// O cartão que uma nota dá: título e descrição (markdown).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardText {
    pub title: String,
    pub description: String,
}

/// O troço `[a, b)` de `rich` como `Inline`, com as marcas que o markdown tem.
fn inline_of(rich: &RichText, a: usize, b: usize) -> Inline {
    let mut out = Inline::plain(&rich.as_str()[a..b]);
    for s in rich.spans() {
        let (s0, s1) = (s.range().start.max(a), s.range().end.min(b));
        if s0 >= s1 {
            continue;
        }
        let r = s0 - a..s1 - a;
        let m = s.marks;
        for (on, mark) in [
            (m.bold, Mark::Bold),
            (m.italic, Mark::Italic),
            (m.strike, Mark::Strike),
        ] {
            if on {
                out.add_mark(r.clone(), mark);
            }
        }
    }
    out
}

/// ⭐ Nota → cartão. A 1.ª linha com texto é o título; as seguintes, parágrafos da descrição.
#[must_use]
pub fn card_from_note(rich: &RichText) -> CardText {
    let text = rich.as_str();
    let mut lines = Vec::new();
    let mut at = 0;
    for l in text.split('\n') {
        lines.push((at, at + l.len()));
        at += l.len() + 1;
    }
    let first = lines
        .iter()
        .position(|&(a, b)| !text[a..b].trim().is_empty());
    let Some(first) = first else {
        return CardText {
            title: String::new(),
            description: String::new(),
        };
    };
    let (a, b) = lines[first];
    let blocks: Vec<Block> = lines[first + 1..]
        .iter()
        .filter(|&&(a, b)| !text[a..b].trim().is_empty())
        .map(|&(a, b)| Block::new(BlockKind::Paragraph, inline_of(rich, a, b)))
        .collect();
    let description = if blocks.is_empty() {
        String::new()
    } else {
        ph2d_kanban_md::render(&RichDoc::from_blocks(blocks))
    };
    CardText {
        title: text[a..b].trim().to_owned(),
        description,
    }
}

/// ⭐ Cartão → nota: o título e, uma linha por bloco, a descrição (as marcas que a nota tem).
#[must_use]
pub fn note_from_card(title: &str, description: &str) -> RichText {
    let mut text = title.trim().to_owned();
    let mut spans = Vec::new();
    if !description.trim().is_empty() {
        for b in ph2d_kanban_md::parse(description).into_blocks() {
            text.push('\n');
            let base = text.len();
            text.push_str(b.inline.text());
            for run in b.inline.runs() {
                let marks = Marks {
                    bold: run.marks.contains(&Mark::Bold),
                    italic: run.marks.contains(&Mark::Italic),
                    strike: run.marks.contains(&Mark::Strike),
                    ..Marks::default()
                };
                if marks != Marks::default() {
                    spans.push(Span {
                        start: u32::try_from(base + run.range.start).unwrap_or(u32::MAX),
                        end: u32::try_from(base + run.range.end).unwrap_or(u32::MAX),
                        marks,
                    });
                }
            }
        }
    }
    RichText::with_spans(text, &spans)
}

fn lab([r, g, b]: [u8; 3]) -> [f32; 3] {
    let lin = [r, g, b].map(ph2d_color::srgb::srgb_to_linear_byte);
    ph2d_color::perceptual::to_oklab(lin)
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// A cor de uma etiqueta no tema claro (o do papel das notas).
fn label_rgb(l: LabelColor) -> [u8; 3] {
    let s = Shade::ALL.iter().position(|x| *x == l.shade).unwrap_or(0);
    let h = Hue::ALL.iter().position(|x| *x == l.hue).unwrap_or(0);
    let c = ph2d_tokens::kanban::LABELS[s][h].resolve(Theme::Light);
    [c.r, c.g, c.b]
}

/// A etiqueta de cor mais próxima da cor da nota (`None` = a nota branca/cinza de nascença do Miro,
/// que não pede etiqueta).
#[must_use]
pub fn label_for_note(fill: Rgba) -> Option<LabelColor> {
    if fill.0 == STICKY_COLORS[0] {
        return None;
    }
    let target = lab([fill.0[0], fill.0[1], fill.0[2]]);
    LabelColor::all().into_iter().min_by(|a, b| {
        dist(lab(label_rgb(*a)), target).total_cmp(&dist(lab(label_rgb(*b)), target))
    })
}

/// A cor de nota mais próxima da etiqueta (`None` = a cor de nascença das notas).
#[must_use]
pub fn note_color_for(label: Option<LabelColor>) -> Rgba {
    let Some(l) = label else {
        return Rgba(STICKY_COLORS[STICKY_DEFAULT_COLOR]);
    };
    let target = lab(label_rgb(l));
    let best = STICKY_COLORS[1..]
        .iter()
        .min_by(|a, b| {
            dist(lab([a[0], a[1], a[2]]), target).total_cmp(&dist(lab([b[0], b[1], b[2]]), target))
        })
        .copied()
        .unwrap_or(STICKY_COLORS[STICKY_DEFAULT_COLOR]);
    Rgba(best)
}

#[cfg(test)]
#[path = "notes_tests.rs"]
mod tests;
