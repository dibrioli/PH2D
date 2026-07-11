//! T1.1 — empacotamento do `FlipDrawing` (SoA do documento) para os **storage
//! buffers** que o vertex shader lê por índice.
//!
//! Layout (mesma convenção Pod/`cast_slice` do `instance_buffer` do ph2d-render):
//! - `points: [GpuPoint]` — todos os pontos de todos os traços do desenho, em
//!   sequência; cada traço ocupa `[first_point, first_point+point_count)`.
//! - `strokes: [GpuStroke]` — a tabela de traços (offset+contagem + atributos
//!   por-curva).
//!
//! **Sem vértices de padding de adjacência** (o GP 5.2 os usa por um limite de
//! attribute-count do OpenGL — `draw_grease_pencil_lib.glsl:458`). Aqui há storage
//! buffers WGSL: o vertex shader indexa livremente e faz clamp (aberto) / wrap
//! (fechado) nos vizinhos `id-1/id+1/id+2`. Fica pro T1.2.
//!
//! Puro (sem wgpu) — testável headless. O upload pra GPU envolve estas structs.

use ph2d_flip::{Cap, FlipDrawing};

/// `stroke.closed` (traço cíclico). Sem este bit = aberto.
pub const FLAG_CLOSED: u32 = 1 << 0;
/// Ponta INICIAL reta (`Cap::Flat`). Sem o bit = arredondada (`Cap::Round`).
pub const FLAG_START_FLAT: u32 = 1 << 1;
/// Ponta FINAL reta (`Cap::Flat`). Sem o bit = arredondada.
pub const FLAG_END_FLAT: u32 = 1 << 2;

/// Um ponto na GPU: posição (mundo, 2D), largura e opacidade por-ponto, cor RGBA
/// linear straight-alpha. `repr(C)` + `Pod` = 8×`f32` = 32 bytes, sem padding.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuPoint {
    /// Posição em unidades de mundo (a câmera ortográfica converte pra tela).
    pub pos: [f32; 2],
    /// Largura (raio×2, mundo) neste ponto.
    pub width: f32,
    /// Opacidade `[0,1]` neste ponto.
    pub opacity: f32,
    /// Cor RGBA linear straight-alpha.
    pub color: [f32; 4],
}

/// Uma entrada da tabela de traços: onde estão os pontos + atributos por-curva.
/// `repr(C)` + `Pod` = 8×4 bytes = 32 bytes, sem padding.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuStroke {
    /// Índice do primeiro ponto em `points`.
    pub first_point: u32,
    /// Quantos pontos este traço tem.
    pub point_count: u32,
    /// `FLAG_*` (closed / caps).
    pub flags: u32,
    /// Dureza da borda `[0,1]` (1 = dura).
    pub hardness: f32,
    /// Material (paleta) — chave de batching futuro.
    pub material: u32,
    /// Padding para alinhar a 32 bytes (storage buffer stride estável).
    pub _pad: [u32; 3],
}

/// O resultado do empacotamento de um desenho, pronto para upload.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlipGpuData {
    pub points: Vec<GpuPoint>,
    pub strokes: Vec<GpuStroke>,
    /// Paralelo a `points`: o índice do traço a que cada ponto pertence. Deixa o
    /// vertex shader achar `first_point`/`point_count`/`flags` do vizinho (clamp
    /// aberto / wrap fechado) sem varrer a tabela.
    pub point_stroke: Vec<u32>,
}

impl FlipGpuData {
    /// Sem geometria (desenho vazio → nada a rasterizar).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strokes.is_empty()
    }

    /// Total de pontos empacotados (para dimensionar o buffer / vertex count).
    #[must_use]
    pub fn point_count(&self) -> usize {
        self.points.len()
    }
}

/// Empacota `flags` a partir dos atributos por-curva.
fn stroke_flags(closed: bool, cap: (Cap, Cap)) -> u32 {
    let mut f = 0;
    if closed {
        f |= FLAG_CLOSED;
    }
    if cap.0 == Cap::Flat {
        f |= FLAG_START_FLAT;
    }
    if cap.1 == Cap::Flat {
        f |= FLAG_END_FLAT;
    }
    f
}

/// Empacota um desenho inteiro em `points` + `strokes`. A ordem dos traços é
/// preservada (é a ordem de z do desenho, fundo → topo).
#[must_use]
pub fn pack_drawing(drawing: &FlipDrawing) -> FlipGpuData {
    let mut points = Vec::new();
    let mut strokes = Vec::with_capacity(drawing.strokes.len());
    let mut point_stroke = Vec::new();
    for (sid, s) in drawing.strokes.iter().enumerate() {
        let first_point = points.len() as u32;
        let pos = s.positions();
        let w = s.widths();
        let op = s.opacities();
        let col = s.colors();
        for i in 0..pos.len() {
            points.push(GpuPoint {
                pos: [pos[i].x, pos[i].y],
                width: w[i],
                opacity: op[i],
                color: col[i].0,
            });
            point_stroke.push(sid as u32);
        }
        strokes.push(GpuStroke {
            first_point,
            point_count: pos.len() as u32,
            flags: stroke_flags(s.closed, s.cap),
            hardness: s.hardness,
            material: s.material.0,
            _pad: [0; 3],
        });
    }
    FlipGpuData {
        points,
        strokes,
        point_stroke,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_flip::{FlipStroke, Point, Rgba};

    #[test]
    fn pod_structs_are_32_bytes() {
        assert_eq!(std::mem::size_of::<GpuPoint>(), 32);
        assert_eq!(std::mem::size_of::<GpuStroke>(), 32);
    }

    #[test]
    fn pack_lays_out_points_and_stroke_table() {
        let mut d = FlipDrawing::new();
        // Traço 0: 2 pontos, aberto, cap default (round), hardness default.
        let mut s0 = FlipStroke::new();
        s0.push_point(Point {
            pos: Vec2::new(0.0, 0.0),
            width: 1.0,
            opacity: 1.0,
            color: Rgba::WHITE,
        });
        s0.push_point(Point {
            pos: Vec2::new(10.0, 0.0),
            width: 2.0,
            opacity: 0.5,
            color: Rgba::BLACK,
        });
        d.strokes.push(s0);
        // Traço 1: 3 pontos, fechado, ponta final reta, hardness 0.3, material 4.
        let mut s1 = FlipStroke::new();
        for k in 0..3 {
            s1.push_default(Vec2::new(k as f32, 5.0));
        }
        s1.closed = true;
        s1.cap = (Cap::Round, Cap::Flat);
        s1.hardness = 0.3;
        s1.material = ph2d_flip::MaterialId(4);
        d.strokes.push(s1);

        let g = pack_drawing(&d);
        assert_eq!(g.point_count(), 5, "2 + 3 pontos");
        assert_eq!(g.strokes.len(), 2);
        // point_stroke é paralelo a points: 2 pontos do traço 0, 3 do traço 1.
        assert_eq!(g.point_stroke, vec![0, 0, 1, 1, 1]);

        // Tabela do traço 0.
        assert_eq!(g.strokes[0].first_point, 0);
        assert_eq!(g.strokes[0].point_count, 2);
        assert_eq!(g.strokes[0].flags, 0, "aberto, caps round");
        assert_eq!(g.strokes[0].hardness, ph2d_flip::DEFAULT_HARDNESS);
        // Pontos do traço 0 (atributos por-ponto preservados).
        assert_eq!(g.points[0].pos, [0.0, 0.0]);
        assert_eq!(g.points[1].width, 2.0);
        assert_eq!(g.points[1].opacity, 0.5);
        assert_eq!(g.points[1].color, Rgba::BLACK.0);

        // Tabela do traço 1: offset após os 2 pontos do traço 0.
        assert_eq!(g.strokes[1].first_point, 2);
        assert_eq!(g.strokes[1].point_count, 3);
        assert_eq!(
            g.strokes[1].flags,
            FLAG_CLOSED | FLAG_END_FLAT,
            "fechado + ponta final reta"
        );
        assert_eq!(g.strokes[1].hardness, 0.3);
        assert_eq!(g.strokes[1].material, 4);
    }

    #[test]
    fn empty_drawing_packs_to_nothing() {
        let g = pack_drawing(&FlipDrawing::new());
        assert!(g.is_empty());
        assert_eq!(g.point_count(), 0);
    }

    #[test]
    fn cast_slice_round_trips_the_bytes() {
        // O upload usa `bytemuck::cast_slice`; prova que os bytes voltam idênticos.
        let mut d = FlipDrawing::new();
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(1.5, -2.5));
        d.strokes.push(s);
        let g = pack_drawing(&d);
        let bytes: &[u8] = bytemuck::cast_slice(&g.points);
        assert_eq!(bytes.len(), 32);
        let back: &[GpuPoint] = bytemuck::cast_slice(bytes);
        assert_eq!(back, g.points.as_slice());
    }
}
