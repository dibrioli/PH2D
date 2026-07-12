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

use crate::fill::{FillVertex, triangulate};
use crate::fill_holes;
use crate::neighbors::{self, Seg};
use ph2d_core::Vec2;
use ph2d_flip::{Cap, FlipDrawing};

/// Espaçamento de profundidade por-traço (GP §2, `2e-7`). Cada traço ocupa 2
/// slots: fill em `2·sid+1`, traço em `2·sid+2` (o traço ganha o GREATER sobre o
/// próprio fill; sids maiores ficam por cima).
pub(crate) const DEPTH_STEP: f32 = 2e-7;

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

/// Um segmento vizinho GEOMÉTRICO, resolvido em índices GLOBAIS de ponto — o par
/// `(a, b)` já com o wrap do traço fechado embutido, para o fragment não precisar
/// consultar a tabela de traços. 8 bytes = o stride de um `vec2<u32>` no WGSL.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSegRef {
    pub a: u32,
    pub b: u32,
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
    /// Vértices de fill (triângulos já triangulados), dos traços fechados com
    /// preenchimento. Rasterizados ABAIXO do traço (profundidade menor).
    pub fills: Vec<FillVertex>,
    /// Paralelo a `points`: `[offset, count]` em [`Self::seg_extras`] — os segmentos
    /// NÃO-adjacentes que podem influenciar os pixels do quad do segmento que
    /// começa NESTE ponto. É o que torna a cobertura do fragment a **UNIÃO GLOBAL**
    /// da polilinha num único passe: sem esta lista, um traço que volta sobre si
    /// mesmo (zigzag apertado, laço, letra) sofre a "mordida" de longo alcance — o
    /// quad do segmento de índice MENOR vence o núcleo do outro por depth
    /// first-wins. Vazia na esmagadora maioria dos traços (custo zero); ver
    /// `neighbors.rs` e `docs/Flip/03_traco_rasterizacao.md`.
    pub seg_extra_range: Vec<[u32; 2]>,
    /// A lista concatenada apontada por [`Self::seg_extra_range`].
    pub seg_extras: Vec<GpuSegRef>,
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

    /// Anexa `other`, deslocando os índices para o espaço GLOBAL deste buffer
    /// (`sid` via `point_stroke`/posição na tabela, `first_point`, a depth
    /// absoluta dos fills e os índices da janela de vizinhos). Usado para **dobrar
    /// o preview ao vivo na fatia da camada ativa** — o traço em curso passa a
    /// compor pelo blend/opacity DELA em tempo real, byte-idêntico ao que o bake
    /// produz. O apenso ganha `sid` MAIOR → depth maior → por cima dos traços já
    /// presentes (mesma regra GREATER do pack normal).
    pub fn append(&mut self, other: &FlipGpuData) {
        let point_base = self.points.len() as u32;
        let stroke_base = self.strokes.len() as u32;
        let extra_base = self.seg_extras.len() as u32;
        self.points.extend_from_slice(&other.points);
        self.point_stroke
            .extend(other.point_stroke.iter().map(|&s| s + stroke_base));
        self.strokes.extend(other.strokes.iter().map(|s| {
            let mut s = *s;
            s.first_point += point_base;
            s
        }));
        // A janela de vizinhos é indexada por PONTO e aponta PONTOS: os dois espaços
        // rebaseiam pelo mesmo deslocamento.
        self.seg_extra_range
            .extend(other.seg_extra_range.iter().map(|&[off, count]| {
                if count == 0 {
                    [0, 0]
                } else {
                    [off + extra_base, count]
                }
            }));
        self.seg_extras
            .extend(other.seg_extras.iter().map(|r| GpuSegRef {
                a: r.a + point_base,
                b: r.b + point_base,
            }));
        // Fills carregam depth ABSOLUTA = (2·sid+1)·DEPTH_STEP (o traço lê o sid no
        // shader; o fill não). Rebaseia pelo mesmo deslocamento de sid para casar.
        let depth_shift = 2.0 * stroke_base as f32 * DEPTH_STEP;
        self.fills.extend(other.fills.iter().map(|v| {
            let mut v = *v;
            v.depth += depth_shift;
            v
        }));
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

/// Os segmentos de UM traço, em índices GLOBAIS de ponto (o wrap do fechado já
/// resolvido) — a entrada do broadphase de vizinhos geométricos.
fn stroke_segments(g: &FlipGpuData, first_point: u32, count: u32, closed: bool) -> Vec<Seg> {
    if count < 2 {
        return Vec::new();
    }
    let last = first_point + count - 1;
    let mut segs = Vec::with_capacity(count as usize);
    let mut push = |a: u32, b: u32| {
        let (pa, pb) = (g.points[a as usize], g.points[b as usize]);
        segs.push(Seg {
            a,
            b,
            pa: Vec2::new(pa.pos[0], pa.pos[1]),
            pb: Vec2::new(pb.pos[0], pb.pos[1]),
            radius: (pa.width.max(pb.width)) * 0.5,
        });
    };
    for a in first_point..last {
        push(a, a + 1);
    }
    if closed {
        push(last, first_point); // a costura
    }
    segs
}

/// Anexa os traços de `drawing` aos acumuladores (o `sid` de cada traço é o
/// índice GLOBAL em `strokes`, então `point_stroke` casa entre desenhos).
fn append_drawing(g: &mut FlipGpuData, drawing: &FlipDrawing) {
    for s in &drawing.strokes {
        let sid = g.strokes.len() as u32;
        let first_point = g.points.len() as u32;
        let pos = s.positions();

        // **`hide_stroke`** (W4): o traço não é rasterizado — só o preenchimento dele.
        // É o que o balde produz (a cor entra POR BAIXO do line-art, sem desenhar um
        // segundo contorno em cima). A `GpuStroke` é empurrada mesmo assim, com ZERO
        // pontos: ela não gera quad nenhum, e o `sid` (que dá a profundidade do fill)
        // continua alinhado com a ordem de z dos traços.
        if s.hide_stroke {
            g.strokes.push(GpuStroke {
                first_point,
                point_count: 0,
                flags: stroke_flags(s.closed, s.cap),
                hardness: s.hardness,
                material: s.material.0,
                _pad: [0; 3],
            });
        } else {
            let w = s.widths();
            let op = s.opacities();
            let col = s.colors();
            for i in 0..pos.len() {
                g.points.push(GpuPoint {
                    pos: [pos[i].x, pos[i].y],
                    width: w[i],
                    opacity: op[i],
                    color: col[i].0,
                });
                g.point_stroke.push(sid);
                g.seg_extra_range.push([0, 0]); // preenchido abaixo
            }
            g.strokes.push(GpuStroke {
                first_point,
                point_count: pos.len() as u32,
                flags: stroke_flags(s.closed, s.cap),
                hardness: s.hardness,
                material: s.material.0,
                _pad: [0; 3],
            });

            // A janela de vizinhos GEOMÉTRICOS deste traço (a união global no fragment;
            // vazia para traços que não voltam sobre si mesmos — o caso comum).
            let segs = stroke_segments(g, first_point, pos.len() as u32, s.closed);
            for (i, extras) in neighbors::extras_for_stroke(&segs).into_iter().enumerate() {
                if extras.is_empty() {
                    continue;
                }
                let offset = g.seg_extras.len() as u32;
                for j in &extras {
                    let sj = segs[*j as usize];
                    g.seg_extras.push(GpuSegRef { a: sj.a, b: sj.b });
                }
                // O range é indexado pelo PONTO INICIAL do segmento (a identidade do
                // segmento no shader é o `gp` do seu primeiro ponto).
                let owner = segs[i].a as usize;
                g.seg_extra_range[owner] = [offset, extras.len() as u32];
            }
        }

        // Fill: só traço FECHADO com preenchimento. Sem buracos, o ear-clipping do W1
        // (byte-idêntico ao que já estava). COM buracos, a decomposição trapezoidal
        // even-odd (`fill_holes`) — ela devolve POSIÇÕES novas (os cantos dos
        // trapézios), não índices de entrada.
        if s.closed
            && let Some(f) = s.fill
        {
            let depth = (2 * sid + 1) as f32 * DEPTH_STEP;
            let a = f.color.a() * f.opacity;
            // Premultiplicado (o fragment de fill não multiplica de novo).
            let color = [f.color.r() * a, f.color.g() * a, f.color.b() * a, a];
            let mut emit = |p: Vec2| {
                g.fills.push(FillVertex {
                    pos: [p.x, p.y],
                    depth,
                    _pad: 0.0,
                    color,
                });
            };
            if s.holes.is_empty() {
                let pts: Vec<Vec2> = pos.to_vec();
                for &vi in &triangulate(&pts) {
                    emit(pts[vi as usize]);
                }
            } else {
                for p in fill_holes::triangulate_even_odd(pos, &s.holes) {
                    emit(p);
                }
            }
        }
    }
}

/// Empacota um desenho inteiro. A ordem dos traços é preservada (ordem de z do
/// desenho, fundo → topo).
#[must_use]
pub fn pack_drawing(drawing: &FlipDrawing) -> FlipGpuData {
    let mut g = FlipGpuData::default();
    append_drawing(&mut g, drawing);
    g
}

/// Achata VÁRIOS desenhos ativos (ex.: as camadas de um objeto no quadro atual)
/// num único buffer, na ordem dada (fundo → topo). O blend/opacity por-camada é
/// do T1.7 — aqui tudo compõe chapado no mesmo passe.
#[must_use]
pub fn pack_drawings(drawings: &[&FlipDrawing]) -> FlipGpuData {
    let mut g = FlipGpuData::default();
    for d in drawings {
        append_drawing(&mut g, d);
    }
    g
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
    fn pack_drawings_flattens_multiple_with_global_stroke_ids() {
        // Dois desenhos (ex.: camadas BG+FG ativas): os sids ficam GLOBAIS e
        // point_stroke casa entre eles.
        let mut d0 = FlipDrawing::new();
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(0.0, 0.0));
        s.push_default(Vec2::new(1.0, 0.0));
        d0.strokes.push(s);
        let mut d1 = FlipDrawing::new();
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(2.0, 0.0));
        d1.strokes.push(s);

        let g = pack_drawings(&[&d0, &d1]);
        assert_eq!(g.strokes.len(), 2, "1 traço de cada desenho");
        assert_eq!(g.point_count(), 3, "2 + 1 pontos");
        // Traço 1 (do d1) tem sid GLOBAL 1 e offset 2.
        assert_eq!(g.strokes[1].first_point, 2);
        assert_eq!(g.point_stroke, vec![0, 0, 1]);
        // Vazio no meio não quebra.
        let empty = FlipDrawing::new();
        let g2 = pack_drawings(&[&d0, &empty, &d1]);
        assert_eq!(g2.point_stroke, vec![0, 0, 1]);
    }

    #[test]
    fn append_rebases_indices_like_pack_drawings() {
        // Dobrar o preview na fatia da camada = anexar dois buffers já empacotados
        // deve dar o MESMO layout que empacotar os dois desenhos juntos.
        let mut d0 = FlipDrawing::new();
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(0.0, 0.0));
        s.push_default(Vec2::new(1.0, 0.0));
        d0.strokes.push(s);
        let mut d1 = FlipDrawing::new();
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(2.0, 0.0));
        s.push_default(Vec2::new(3.0, 0.0));
        d1.strokes.push(s);

        let mut merged = pack_drawing(&d0);
        merged.append(&pack_drawing(&d1));
        let joined = pack_drawings(&[&d0, &d1]);

        assert_eq!(merged.points, joined.points, "pontos concatenados");
        assert_eq!(merged.point_stroke, joined.point_stroke, "sid global");
        assert_eq!(merged.strokes, joined.strokes, "first_point deslocado");
        // O apenso ganha o sid MAIOR (por cima).
        assert_eq!(merged.point_stroke, vec![0, 0, 1, 1]);
    }

    #[test]
    fn append_empty_base_equals_the_other() {
        // Camada ativa vazia (1º traço) + preview = só o preview.
        let mut d = FlipDrawing::new();
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(0.0, 0.0));
        s.push_default(Vec2::new(1.0, 1.0));
        d.strokes.push(s);
        let only = pack_drawing(&d);
        let mut merged = FlipGpuData::default();
        merged.append(&only);
        assert_eq!(merged, only);
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
