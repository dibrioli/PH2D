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

/// A face de uma corrida: os bytes da fonte (o endereço) e o índice, o tamanho, a variação e a
/// inclinação — tudo o que muda o contorno de um glifo além do id dele.
type Face = (usize, u32, u32, Vec<i16>, Option<[u64; 6]>);

/// ⭐ Cada glifo de cada face tirado UMA vez e reusado (o SVG por `<use>`, o PDF por um XObject) —
/// medido 09/10: com o contorno repetido em cada letra, 10 mil notas davam um SVG de 144 MB e um PDF
/// de 90 MB.
#[derive(Default)]
pub(crate) struct Glyphs {
    faces: BTreeMap<Face, u32>,
    index: BTreeMap<(u32, u32), usize>,
    paths: Vec<BezPath>,
}

impl Glyphs {
    /// Os glifos da corrida: o índice de cada contorno (já com a inclinação e o y virado, na origem) e
    /// a posição dele no espaço da corrida (falta só o `run.transform`).
    pub(crate) fn place(&mut self, run: &GlyphRun) -> Vec<(usize, [f64; 2])> {
        let data = run.font.data.data();
        let face = (
            data.as_ptr() as usize,
            run.font.index,
            run.size.to_bits(),
            run.coords.clone(),
            run.skew.map(|a| a.as_coeffs().map(f64::to_bits)),
        );
        let next = u32::try_from(self.faces.len()).unwrap_or(u32::MAX);
        let face = *self.faces.entry(face).or_insert(next);
        let mut font = None;
        let coords: Vec<NormalizedCoord> = run
            .coords
            .iter()
            .map(|&c| NormalizedCoord::from_bits(c))
            .collect();
        let mut out = Vec::with_capacity(run.glyphs.len());
        for g in &run.glyphs {
            let at = match self.index.get(&(face, g.id)) {
                Some(&i) => i,
                None => {
                    let f =
                        font.get_or_insert_with(|| FontRef::from_index(data, run.font.index).ok());
                    let mut pen = Pen::default();
                    if let Some(glyph) = f
                        .as_ref()
                        .and_then(|f| f.outline_glyphs().get(GlyphId::new(g.id)))
                    {
                        let at =
                            DrawSettings::unhinted(Size::new(run.size), LocationRef::new(&coords));
                        let _ = glyph.draw(at, &mut pen);
                    }
                    let local = run.skew.unwrap_or(Affine::IDENTITY) * Affine::FLIP_Y;
                    let path: BezPath = pen.0.elements().iter().map(|&e| map(local, e)).collect();
                    self.paths.push(path);
                    let i = self.paths.len() - 1;
                    self.index.insert((face, g.id), i);
                    i
                }
            };
            out.push((at, [f64::from(g.x), f64::from(g.y)]));
        }
        out
    }

    /// O contorno `i` (na origem do glifo, no espaço da corrida).
    pub(crate) fn path(&self, i: usize) -> &BezPath {
        &self.paths[i]
    }

    /// Quantos contornos há (o próximo índice novo).
    pub(crate) fn len(&self) -> usize {
        self.paths.len()
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
