//! `ph2d-pixel-text` — «que células acende um texto?» (Pixel Art, P8 item 8; `docs/PixelArt/P8_plano.md`).
//!
//! Duas famílias de fonte:
//! - **de pixel** (`Builtin`, e as que o artista importa por [`read_font_file`]): BDF/HEX/FNT/FON, porte
//!   dos leitores do MVP W27; cada letra JÁ é uma grade, nada se inventa;
//! - **do sistema** (`System(path, index)`): contorno do `skrifa` e o rasterizador de [`outline`].

#![forbid(unsafe_code)]

mod bdf;
pub mod carimbo;
mod charsets;
pub mod font;
mod hex;
mod io;
pub mod outline;
mod win;

pub use bdf::parse_bdf;
pub use charsets::charset_to_unicode;
pub use font::{
    Align, Glyph, LayoutOpts, PixelFont, Rendered, caret_at, layout_text, render_text, text_lines,
};
pub use hex::parse_hex;
pub use io::{FONT_EXTENSIONS, decode_text, read_font_file};
pub use win::{parse_fnt, parse_fon};

use std::path::PathBuf;

/// As fontes de pixel que vêm de fábrica (`fonts/`, sem um byte mudado; licenças em `fonts/README.md`).
pub const BUILTIN: [(&str, &str, &[u8]); 4] = [
    (
        "cozette",
        "cozette.bdf",
        include_bytes!("../fonts/cozette.bdf"),
    ),
    (
        "tom-thumb",
        "tom-thumb.bdf",
        include_bytes!("../fonts/tom-thumb.bdf"),
    ),
    (
        "unscii-8",
        "unscii-8.hex",
        include_bytes!("../fonts/unscii-8.hex"),
    ),
    (
        "unscii-16",
        "unscii-16.hex",
        include_bytes!("../fonts/unscii-16.hex"),
    ),
];

/// Lê uma fonte de fábrica pelo nome.
pub fn builtin(name: &str) -> Result<PixelFont, String> {
    let (_, file, bytes) = BUILTIN
        .iter()
        .find(|b| b.0 == name)
        .ok_or_else(|| format!("fonte de fábrica desconhecida: {name}"))?;
    read_font_file(file, bytes)?
        .pop()
        .ok_or_else(|| format!("{file}: sem fonte"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontChoice {
    /// Uma fonte de [`BUILTIN`], pelo nome.
    Builtin(String),
    /// Um ficheiro TTF/OTF/TTC e o índice da face.
    System(PathBuf, u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextStyle {
    pub font: FontChoice,
    /// Fonte de pixel: múltiplo inteiro da altura da linha (≥ 1×). Fonte do sistema: píxeis por em.
    pub px: u32,
    pub bold: bool,
    pub italic: bool,
    pub antialias: bool,
    /// Fonte do sistema: hinting TrueType (o Aseprite liga-o por omissão). Fonte de pixel: ignora.
    pub hinting: bool,
}

impl TextStyle {
    /// As omissões do Aseprite: sem negrito/itálico, Antialias desligado, Hinting ligado.
    pub fn new(font: FontChoice, px: u32) -> Self {
        Self {
            font,
            px,
            bold: false,
            italic: false,
            antialias: false,
            hinting: true,
        }
    }
}

/// As células que o texto acende: `cover` tem `w·h` valores 0..=255, linha a linha.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextMask {
    pub w: u32,
    pub h: u32,
    pub cover: Vec<u8>,
    /// Linha da máscara onde fica a linha de base da primeira linha de texto.
    pub baseline: i32,
    /// Coluna da máscara onde a caneta começa (a tinta pode recuar para a esquerda dela).
    pub origin_x: i32,
    /// Caracteres que a fonte não tem.
    pub missing: Vec<u32>,
    /// O que não se cumpriu do estilo (variante ausente, …), em texto.
    pub notes: Vec<String>,
}

/// Que células acende `text` com `style`. Erro só quando a fonte não abre.
pub fn rasterize(text: &str, style: &TextStyle) -> Result<TextMask, String> {
    match &style.font {
        FontChoice::Builtin(name) => Ok(rasterize_pixel(&builtin(name)?, text, style)),
        FontChoice::System(path, index) => {
            let data = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
            rasterize_outline(&data, *index, text, style)
        }
    }
}

fn variant_notes(style: &TextStyle, what: &str) -> Vec<String> {
    let mut notes = Vec::new();
    if style.bold {
        notes.push(format!("{what}: sem variante Bold — saiu Regular"));
    }
    if style.italic {
        notes.push(format!("{what}: sem variante Italic — saiu Regular"));
    }
    notes
}

/// Fonte de pixel: a máscara do MVP, ampliada por um inteiro.
pub fn rasterize_pixel(font: &PixelFont, text: &str, style: &TextStyle) -> TextMask {
    let r = render_text(font, text, LayoutOpts::default());
    let scale = (style.px / font.line_height().max(1) as u32).max(1);
    let (w, h) = (r.mw * scale, r.mh * scale);
    let mut cover = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            if r.mask[((y / scale) * r.mw + x / scale) as usize] != 0 {
                cover[(y * w + x) as usize] = 255;
            }
        }
    }
    let s = scale as i32;
    TextMask {
        w,
        h,
        cover,
        baseline: (font.ascent - r.my) * s,
        origin_x: -r.mx * s,
        missing: r.missing,
        notes: variant_notes(style, &font.family),
    }
}

/// Fonte de contorno: glifo a glifo pela caneta (avanço arredondado, sem kerning), uma linha por `\n`.
pub fn rasterize_outline(
    data: &[u8],
    index: u32,
    text: &str,
    style: &TextStyle,
) -> Result<TextMask, String> {
    let face = outline::face(data, index)?;
    let m = outline::face_metrics(&face, style.px);
    let hint = style
        .hinting
        .then(|| outline::hinter(&face, style.px, style.antialias))
        .flatten();
    let step = m.ascent + m.descent;
    let mut placed = Vec::new();
    let mut missing = std::collections::BTreeSet::new();
    for (k, line) in text_lines(text).iter().enumerate() {
        let mut pen = 0;
        for ch in line.chars() {
            if (ch as u32) < 32 {
                continue;
            }
            let g = outline::rasterize_glyph(&face, ch, style.px, style.antialias, hint.as_ref())
                .or_else(|| {
                    missing.insert(ch as u32);
                    let fffd = '\u{fffd}';
                    outline::rasterize_glyph(&face, fffd, style.px, style.antialias, hint.as_ref())
                });
            let Some(g) = g else { continue };
            let base = k as i32 * step + m.ascent;
            placed.push((pen + g.left, base - g.top, g.clone()));
            pen += g.adv;
        }
    }
    let lines = text_lines(text).len() as i32;
    let (mut x0, mut y0, mut x1, mut ink) = (0, 0, 0, i32::MIN);
    for (x, y, g) in &placed {
        if g.w > 0 {
            x0 = x0.min(*x);
            y0 = y0.min(*y);
            x1 = x1.max(x + g.w as i32);
            ink = ink.max(y + g.h as i32);
        }
    }
    // A caixa do Aseprite corta a última linha da descida quando o ascent arredonda para cima.
    let base = (lines - 1) * step + m.ascent;
    let ink = (ink - outline::box_bottom_cut(&face, style.px)).max(base.min(ink));
    let y1 = ink.max(lines * step).max(y0);
    let (w, h) = ((x1 - x0) as u32, (y1 - y0) as u32);
    let mut cover = vec![0u8; (w * h) as usize];
    for (x, y, g) in &placed {
        for gy in 0..g.h {
            if y + gy as i32 >= ink {
                break;
            }
            for gx in 0..g.w {
                let mut v = g.cover[(gy * g.w + gx) as usize];
                if style.antialias {
                    v = outline::TRIAL_TEXT_TONE[usize::from(v)];
                }
                let at = (((y - y0) as u32 + gy) * w + (x - x0) as u32 + gx) as usize;
                cover[at] = cover[at].max(v);
            }
        }
    }
    Ok(TextMask {
        w,
        h,
        cover,
        baseline: m.ascent - y0,
        origin_x: -x0,
        missing: missing.into_iter().collect(),
        notes: {
            // A variante é a FACE escolhida (o ficheiro Bold/Italic): só se regista o que ela não é.
            let attrs = skrifa::MetadataProvider::attributes(&face);
            let wants = TextStyle {
                bold: style.bold && attrs.weight.value() < 600.0,
                italic: style.italic && attrs.style == skrifa::attribute::Style::Normal,
                ..style.clone()
            };
            variant_notes(&wants, "a face escolhida")
        },
    })
}
