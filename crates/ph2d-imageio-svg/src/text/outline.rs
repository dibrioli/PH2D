//! O contorno de cada glifo (`skrifa`, sem hinting — o SVG escala o texto como geometria) escrito
//! como dados de `<path d>` no espaço do `<text>`: Y para baixo, a origem na linha de base.

use std::fmt::Write as _;

use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider as _};

use super::layout::Run;

/// Acrescenta a `d[dono]` o contorno de cada glifo da corrida.
pub(crate) fn append(run: &Run, d: &mut [String]) {
    if run.size <= 0.0 {
        return;
    }
    let Ok(font) = FontRef::from_index(run.font.data.as_ref(), run.font.index) else {
        return;
    };
    let coords: Vec<NormalizedCoord> = run
        .coords
        .iter()
        .map(|c| NormalizedCoord::from_bits(*c))
        .collect();
    let outlines = font.outline_glyphs();
    for &(id, at, owner) in &run.glyphs {
        let settings = DrawSettings::unhinted(Size::new(run.size), LocationRef::new(&coords));
        let Some(g) = outlines.get(GlyphId::new(id)) else {
            continue;
        };
        let mut pen = Pen {
            d: &mut d[owner],
            at,
            skew: run.skew,
        };
        let _ = g.draw(settings, &mut pen);
    }
}

struct Pen<'a> {
    d: &'a mut String,
    at: [f32; 2],
    skew: f32,
}

impl Pen<'_> {
    fn pt(&mut self, x: f32, y: f32) {
        let _ = write!(
            self.d,
            "{} {} ",
            self.at[0] + x + self.skew * y,
            self.at[1] - y
        );
    }
}

impl OutlinePen for Pen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.d.push('M');
        self.pt(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.d.push('L');
        self.pt(x, y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.d.push('Q');
        self.pt(cx, cy);
        self.pt(x, y);
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.d.push('C');
        self.pt(cx0, cy0);
        self.pt(cx1, cy1);
        self.pt(x, y);
    }
    fn close(&mut self) {
        self.d.push('Z');
    }
}
