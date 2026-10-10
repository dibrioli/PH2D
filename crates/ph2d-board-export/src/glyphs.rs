//! As letras como CONTORNOS (E5): os mesmos glifos, da mesma fonte, com as mesmas coordenadas de
//! variação que o Vello desenhou — a imagem exacta do ecrã, sem embeber a fonte (a Inter da casa é
//! VARIÁVEL: embebê-la pedia instanciá-la, estudo §3).

use std::collections::BTreeMap;

use ph2d_vector::{Affine, BezPath, GlyphRun, PathEl, Point};
use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider as _};

/// Um contorno de glifo no espaço da fonte (y para CIMA), à escala do tamanho da corrida.
#[derive(Default)]
struct Pen(BezPath);

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to((f64::from(x), f64::from(y)));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to((f64::from(x), f64::from(y)));
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.0
            .quad_to((f64::from(cx), f64::from(cy)), (f64::from(x), f64::from(y)));
    }
    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        self.0.curve_to(
            (f64::from(c0x), f64::from(c0y)),
            (f64::from(c1x), f64::from(c1y)),
            (f64::from(x), f64::from(y)),
        );
    }
    fn close(&mut self) {
        self.0.close_path();
    }
}

/// A chave de um contorno: a fonte (o endereço dos bytes + o índice), o glifo, o tamanho e a
/// variação.
type Key = (usize, u32, u32, u32, Vec<i16>);

/// Os contornos já tirados (um quadro grande repete as mesmas letras milhares de vezes).
#[derive(Default)]
pub(crate) struct Glyphs {
    cache: BTreeMap<Key, BezPath>,
}

impl Glyphs {
    /// A corrida inteira num caminho, no espaço da CORRIDA (falta só o `run.transform`): cada glifo na
    /// sua posição, com a inclinação sintética e o y virado (a fonte sobe, o ecrã desce) — o que o
    /// Vello faz ao desenhar a corrida.
    pub(crate) fn run_path(&mut self, run: &GlyphRun) -> BezPath {
        let data = run.font.data.data();
        let font = FontRef::from_index(data, run.font.index).ok();
        let coords: Vec<NormalizedCoord> = run
            .coords
            .iter()
            .map(|&c| NormalizedCoord::from_bits(c))
            .collect();
        let mut out = BezPath::new();
        for g in &run.glyphs {
            let key = (
                data.as_ptr() as usize,
                run.font.index,
                g.id,
                run.size.to_bits(),
                run.coords.clone(),
            );
            let path = self.cache.entry(key).or_insert_with(|| {
                let mut pen = Pen::default();
                if let Some(glyph) = font
                    .as_ref()
                    .and_then(|f| f.outline_glyphs().get(GlyphId::new(g.id)))
                {
                    let at = DrawSettings::unhinted(Size::new(run.size), LocationRef::new(&coords));
                    let _ = glyph.draw(at, &mut pen);
                }
                pen.0
            });
            let place = Affine::translate((f64::from(g.x), f64::from(g.y)))
                * run.skew.unwrap_or(Affine::IDENTITY)
                * Affine::FLIP_Y;
            out.extend(path.elements().iter().map(|&e| map(place, e)));
        }
        out
    }
}

fn map(t: Affine, e: PathEl) -> PathEl {
    let p = |q: Point| t * q;
    match e {
        PathEl::MoveTo(a) => PathEl::MoveTo(p(a)),
        PathEl::LineTo(a) => PathEl::LineTo(p(a)),
        PathEl::QuadTo(a, b) => PathEl::QuadTo(p(a), p(b)),
        PathEl::CurveTo(a, b, c) => PathEl::CurveTo(p(a), p(b), p(c)),
        PathEl::ClosePath => PathEl::ClosePath,
    }
}
