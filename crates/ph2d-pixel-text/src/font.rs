//! A fonte de pixel e o texto que ela escreve — porte de `docs/Pixel Art/app/engine/font.js` (W27).
//!
//! Glifo na convenção da FreeType: caixa de tinta `w×h`, canto de cima-esquerdo `left` células à direita
//! da caneta e `top` ACIMA da linha de base, avanço `adv`. A âncora (0, 0) é o canto de cima-esquerdo da
//! caixa da primeira linha. NFC antes de tudo; letra ausente sai com o caractere-padrão (ou tofu) e entra
//! em `missing`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use unicode_normalization::UnicodeNormalization;

/// Um glifo: `bits` tem `w·h` células, 1 = tinta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Glyph {
    pub w: u32,
    pub h: u32,
    pub left: i32,
    pub top: i32,
    pub adv: i32,
    pub bits: Vec<u8>,
}

impl Glyph {
    pub fn new(w: u32, h: u32, left: i32, top: i32, adv: i32, bits: Vec<u8>) -> Self {
        debug_assert_eq!(bits.len(), (w * h) as usize);
        Self {
            w,
            h,
            left,
            top,
            adv,
            bits,
        }
    }
}

/// Uma fonte de pixel. `line_gap` é o `dfExternalLeading` do FNT (zero nos outros formatos).
#[derive(Debug)]
pub struct PixelFont {
    pub family: String,
    pub style: String,
    pub format: &'static str,
    pub ascent: i32,
    pub descent: i32,
    pub line_gap: i32,
    pub glyphs: BTreeMap<u32, Glyph>,
    pub default_cp: Option<u32>,
    pub max_adv: i32,
    pub notes: Vec<String>,
    /// .fnt: byte → código, na ordem do arquivo.
    pub codes: Vec<(u8, u32)>,
    fallback: OnceLock<Glyph>,
}

impl PixelFont {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        family: String,
        style: String,
        format: &'static str,
        ascent: i32,
        descent: i32,
        line_gap: i32,
        glyphs: BTreeMap<u32, Glyph>,
        default_cp: Option<u32>,
        notes: Vec<String>,
    ) -> Self {
        let max_adv = glyphs.values().map(|g| g.adv).fold(0, i32::max);
        let family = if family.is_empty() {
            "sem nome".to_owned()
        } else {
            family
        };
        Self {
            family,
            style,
            format,
            ascent,
            descent,
            line_gap,
            glyphs,
            default_cp,
            max_adv,
            notes,
            codes: Vec::new(),
            fallback: OnceLock::new(),
        }
    }

    pub fn line_height(&self) -> i32 {
        self.ascent + self.descent
    }

    /// O nome do seletor: família, estilo quando diz algo, altura da linha.
    pub fn label(&self) -> String {
        let plain = matches!(
            self.style.to_ascii_lowercase().as_str(),
            "" | "regular" | "medium" | "normal" | "book" | "roman" | "r"
        );
        let style = if plain {
            String::new()
        } else {
            format!(" {}", self.style)
        };
        format!("{}{} · {}px", self.family, style, self.line_height())
    }

    /// O glifo que entra no lugar de um ausente: o padrão da fonte, ou a caixa vazada.
    pub fn fallback_glyph(&self) -> &Glyph {
        self.fallback.get_or_init(|| {
            if let Some(g) = self.default_cp.and_then(|cp| self.glyphs.get(&cp)) {
                return g.clone();
            }
            let w = (self.max_adv - 1).max(3) as u32;
            let h = self.ascent.max(3) as u32;
            let mut bits = vec![0; (w * h) as usize];
            for y in 0..h {
                for x in 0..w {
                    if y == 0 || y == h - 1 || x == 0 || x == w - 1 {
                        bits[(y * w + x) as usize] = 1;
                    }
                }
            }
            Glyph::new(w, h, 0, self.ascent, w as i32 + 1, bits)
        })
    }

    /// O chão do espaçamento e da entrelinha.
    pub fn spacing_floor(&self) -> (i32, i32) {
        (-self.max_adv, 1 - self.line_height() - self.line_gap)
    }

    /// O passo entre os topos de duas linhas — nunca menos de uma célula.
    pub fn line_step(&self, leading: i32) -> i32 {
        (self.line_height() + self.line_gap + leading).max(1)
    }
}

/// Tab avança quatro espaços.
pub const TAB_SPACES: i32 = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutOpts {
    pub align: Align,
    pub tracking: i32,
    pub leading: i32,
}

/// O texto como o artista o quis: NFC, e qualquer fim de linha vira `\n`.
pub fn text_lines(text: &str) -> Vec<String> {
    let nfc: String = text.nfc().collect();
    nfc.replace("\r\n", "\n")
        .replace('\r', "\n")
        .split('\n')
        .map(str::to_owned)
        .collect()
}

struct Item<'a> {
    cp: u32,
    g: GlyphRef<'a>,
    x: i32,
}

enum GlyphRef<'a> {
    Tab(i32),
    Ink(&'a Glyph),
}

impl GlyphRef<'_> {
    fn adv(&self) -> i32 {
        match self {
            GlyphRef::Tab(a) => *a,
            GlyphRef::Ink(g) => g.adv,
        }
    }
}

fn line_items<'a>(
    font: &'a PixelFont,
    line: &str,
    tracking: i32,
    missing: &mut BTreeSet<u32>,
) -> (Vec<Item<'a>>, i32) {
    let mut items: Vec<Item<'a>> = Vec::new();
    let (mut pen, mut prev_step) = (0, 0);
    for ch in line.chars() {
        let cp = ch as u32;
        let g = if cp == 9 {
            let space = font.glyphs.get(&32).map_or(font.max_adv, |s| s.adv);
            GlyphRef::Tab(TAB_SPACES * space)
        } else if cp < 32 || cp == 127 {
            continue;
        } else {
            match font.glyphs.get(&cp) {
                Some(g) => GlyphRef::Ink(g),
                None => {
                    missing.insert(cp);
                    GlyphRef::Ink(font.fallback_glyph())
                }
            }
        };
        if !items.is_empty() {
            pen += prev_step;
        }
        prev_step = (g.adv() + tracking).max(0);
        items.push(Item { cp, g, x: pen });
    }
    let width = items.last().map_or(0, |it| it.x + it.g.adv());
    (items, width)
}

/// Uma linha disposta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub offset: i32,
    pub width: i32,
    pub top: i32,
}

/// Um glifo com tinta, no canto de cima-esquerdo `(x, y)` a partir da âncora.
#[derive(Clone, Debug)]
pub struct Placed<'a> {
    pub cp: u32,
    pub g: &'a Glyph,
    pub x: i32,
    pub y: i32,
}

/// O texto disposto.
#[derive(Clone, Debug)]
pub struct Layout<'a> {
    pub width: i32,
    pub height: i32,
    pub step: i32,
    pub rows: Vec<Row>,
    pub glyphs: Vec<Placed<'a>>,
    /// `(x0, y0, x1, y1)` da tinta, ou `None`.
    pub ink: Option<(i32, i32, i32, i32)>,
    pub missing: Vec<u32>,
}

pub fn layout_text<'a>(font: &'a PixelFont, text: &str, opts: LayoutOpts) -> Layout<'a> {
    let mut missing = BTreeSet::new();
    let step = font.line_step(opts.leading);
    let raw: Vec<_> = text_lines(text)
        .iter()
        .map(|l| line_items(font, l, opts.tracking, &mut missing))
        .collect();
    let width = raw.iter().map(|r| r.1).fold(0, i32::max);
    let height = (raw.len() as i32 - 1) * step + font.line_height();
    let (mut rows, mut glyphs) = (Vec::new(), Vec::new());
    let mut ink: Option<(i32, i32, i32, i32)> = None;
    for (k, (items, w)) in raw.iter().enumerate() {
        let offset = match opts.align {
            Align::Left => 0,
            Align::Right => width - w,
            Align::Center => (width - w).div_euclid(2),
        };
        let top = k as i32 * step;
        rows.push(Row {
            offset,
            width: *w,
            top,
        });
        for it in items {
            let GlyphRef::Ink(g) = it.g else { continue };
            if g.w == 0 || g.h == 0 {
                continue;
            }
            let (x, y) = (offset + it.x + g.left, top + font.ascent - g.top);
            let (x1, y1) = (x + g.w as i32, y + g.h as i32);
            ink = Some(match ink {
                None => (x, y, x1, y1),
                Some((a, b, c, d)) => (a.min(x), b.min(y), c.max(x1), d.max(y1)),
            });
            glyphs.push(Placed { cp: it.cp, g, x, y });
        }
    }
    Layout {
        width,
        height,
        step,
        rows,
        glyphs,
        ink,
        missing: missing.into_iter().collect(),
    }
}

/// O texto como máscara (`mw × mh`, 1 = tinta) com o canto em (`mx`, `my`) a partir da âncora.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendered {
    pub mask: Vec<u8>,
    pub mx: i32,
    pub my: i32,
    pub mw: u32,
    pub mh: u32,
    pub missing: Vec<u32>,
}

pub fn render_text(font: &PixelFont, text: &str, opts: LayoutOpts) -> Rendered {
    let l = layout_text(font, text, opts);
    let Some((x0, y0, x1, y1)) = l.ink else {
        return Rendered {
            mask: Vec::new(),
            mx: 0,
            my: 0,
            mw: 0,
            mh: 0,
            missing: l.missing,
        };
    };
    let (mw, mh) = ((x1 - x0) as u32, (y1 - y0) as u32);
    let mut mask = vec![0u8; (mw * mh) as usize];
    for p in &l.glyphs {
        for gy in 0..p.g.h {
            let row = ((p.y - y0) as u32 + gy) * mw + (p.x - x0) as u32;
            for gx in 0..p.g.w {
                if p.g.bits[(gy * p.g.w + gx) as usize] != 0 {
                    mask[(row + gx) as usize] = 1;
                }
            }
        }
    }
    Rendered {
        mask,
        mx: x0,
        my: y0,
        mw,
        mh,
        missing: l.missing,
    }
}

/// O cursor depois de `index` caracteres do texto CRU: `(x, y, h)` a partir da âncora.
pub fn caret_at(font: &PixelFont, text: &str, index: usize, opts: LayoutOpts) -> (i32, i32, i32) {
    let full = layout_text(font, text, opts);
    let prefix: String = text.chars().take(index).collect();
    let before = text_lines(&prefix);
    let k = before.len().min(full.rows.len()) - 1;
    let (items, _) = line_items(
        font,
        before.last().map_or("", String::as_str),
        opts.tracking,
        &mut BTreeSet::new(),
    );
    let x = items
        .last()
        .map_or(0, |it| it.x + (it.g.adv() + opts.tracking).max(0));
    (
        full.rows[k].offset + x,
        full.rows[k].top,
        font.line_height(),
    )
}
