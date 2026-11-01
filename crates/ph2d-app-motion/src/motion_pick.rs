//! ⭐ GEF 19 — **o clique nas formas e imagens do Motion** (dono 01/11: o 1.º clique é do que se vê
//! por cima, em TODA família). Que objectos Motion têm uma cópia sob um ponto do mundo — a de cima
//! primeiro; a shell junta-os aos acertos das outras famílias e ordena tudo pela pilha do desenho.
//!
//! As cópias vêm de onde o quadro as cozinhou: com o cozimento vivo, do DISPOSITIVO (uma leitura
//! limitada, `n × 64 B`, uma vez por quadro — o realce do ponteiro e o clique partilham-na pelo
//! [`CacheDoClique`]); senão do `pump` da CPU. O dono de cada cópia é o objecto Motion do sink dela
//! (`faixas_dos_sinks` no dispositivo, `FaixaDoSink` na CPU). O acerto é a CAIXA da geometria no
//! espaço local da cópia (`path_visual_bounds`; o quad unitário para uma imagem), pela inversa da
//! pose `P + basis · (anchor + q · size)`.

use std::collections::BTreeSet;
use std::ops::Range;

use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::NodeId;
use ph2d_vector::{Affine, Point, Rect};

use crate::motion_state::MotionState;

/// Uma cópia desenhada, com o objecto dono e a geometria (`0` = imagem).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Copia {
    pub dono: u64,
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub basis: [f32; 4],
    pub anchor: [f32; 2],
    pub geometry: u32,
}

/// As cópias do quadro com dono (lidas do dispositivo ou juntadas da CPU) — esquecidas a cada
/// quadro desenhado: o realce do ponteiro pergunta a cada movimento.
#[derive(Default)]
pub struct CacheDoClique {
    lidas: Option<Vec<Copia>>,
}

impl CacheDoClique {
    /// Um quadro novo: a próxima pergunta volta a ler.
    pub fn esquece(&mut self) {
        self.lidas = None;
    }
}

/// O que o clique da shell precisa do Motion.
pub struct Alvo<'a> {
    pub motion: &'a MotionState,
    pub gpu: &'a GpuContext,
    pub cache: &'a mut CacheDoClique,
}

impl<'a> Alvo<'a> {
    #[must_use]
    pub fn new(motion: &'a MotionState, gpu: &'a GpuContext, cache: &'a mut CacheDoClique) -> Self {
        Self { motion, gpu, cache }
    }

    /// [`donos_em`] neste alvo.
    #[must_use]
    pub fn donos_em(&mut self, mundo: [f32; 2]) -> Vec<u64> {
        donos_em(self.motion, self.gpu, self.cache, mundo)
    }
}

/// Os objectos Motion com uma cópia sob `mundo`, a de cima primeiro.
#[must_use]
pub fn donos_em(
    motion: &MotionState,
    gpu: &GpuContext,
    cache: &mut CacheDoClique,
    mundo: [f32; 2],
) -> Vec<u64> {
    if motion.objeto.donos.is_empty() {
        return Vec::new();
    }
    let copias = cache.lidas.get_or_insert_with(|| {
        // A sonda (`PH2D_SONDA`, bit 1) mede a leitura da placa contra a junta da CPU no mesmo processo.
        let na_cpu = ph2d_gpu::sonda::variante().is_some_and(|v| v & 2 == 2);
        let t = std::time::Instant::now();
        let placa = motion.gpu_live && !na_cpu;
        let lidas = if placa {
            do_dispositivo(motion, gpu)
        } else {
            da_cpu(motion)
        };
        let ns = u64::try_from(t.elapsed().as_nanos()).unwrap_or(u64::MAX);
        ph2d_gpu::sonda::nota_cpu(if placa { "realce.placa" } else { "realce.cpu" }, ns);
        lidas
    });
    acertos(copias, mundo, |h| {
        motion
            .shape_store
            .get(h)
            .and_then(|p| ph2d_vec_render::path_visual_bounds(p, Affine::IDENTITY))
    })
}

/// Os donos das cópias sob `mundo`, da última desenhada (a de cima) para a primeira, sem repetir.
/// `caixa(h)` = a caixa local da geometria `h`; uma imagem é o quad unitário centrado.
pub(crate) fn acertos(
    copias: &[Copia],
    mundo: [f32; 2],
    caixa: impl Fn(u32) -> Option<Rect>,
) -> Vec<u64> {
    let mut vistos = BTreeSet::new();
    let mut fora = Vec::new();
    for c in copias.iter().rev() {
        if vistos.contains(&c.dono) {
            continue;
        }
        let local = if c.geometry == 0 {
            Some(Rect::new(-0.5, -0.5, 0.5, 0.5))
        } else {
            caixa(c.geometry)
        };
        let Some(r) = local else { continue };
        let pose = pose(c);
        if pose.determinant().abs() < 1e-12 {
            continue;
        }
        let q = pose.inverse() * Point::new(f64::from(mundo[0]), f64::from(mundo[1]));
        if r.x0 <= q.x && q.x <= r.x1 && r.y0 <= q.y && q.y <= r.y1 && vistos.insert(c.dono) {
            fora.push(c.dono);
        }
    }
    fora
}

/// Local → mundo: `P + basis · (anchor + q · size)` (o mesmo afim do `instance_pose`).
fn pose(c: &Copia) -> Affine {
    let [b0, b1, b2, b3] = c.basis;
    let [sx, sy] = c.size;
    let [ax, ay] = c.anchor;
    Affine::new([
        f64::from(b0 * sx),
        f64::from(b1 * sx),
        f64::from(b2 * sy),
        f64::from(b3 * sy),
        f64::from(c.pos[0] + b0 * ax + b2 * ay),
        f64::from(c.pos[1] + b1 * ax + b3 * ay),
    ])
}

/// O dono vivo e visível do sink.
fn dono(motion: &MotionState, sink: NodeId) -> Option<u64> {
    crate::motion_objeto::origem::dono_do_no(motion, sink)
        .filter(|d| d.visible)
        .map(|d| d.entity)
}

/// Junta as cópias `todas` às faixas `(sink, cópias)`, com o dono de cada uma; sem tinta, fora.
fn com_donos<T>(
    motion: &MotionState,
    faixas: impl Iterator<Item = (NodeId, Range<usize>)>,
    todas: &[T],
    copia: impl Fn(&T, u64) -> Option<Copia>,
    fora: &mut Vec<Copia>,
) {
    for (sink, r) in faixas {
        let Some(d) = dono(motion, sink) else {
            continue;
        };
        let r = r.start.min(todas.len())..r.end.min(todas.len());
        fora.extend(todas[r].iter().filter_map(|t| copia(t, d)));
    }
}

fn da_imagem(i: &ph2d_render::RenderInstance, dono: u64) -> Option<Copia> {
    (i.tint[3] > 0.0).then_some(Copia {
        dono,
        pos: i.world_pos,
        size: i.size,
        basis: i.basis,
        anchor: i.anchor,
        geometry: 0,
    })
}

/// As cópias da CPU: as imagens (sprites e ladrilhos do LOD) por baixo, as formas por cima — a
/// ordem em que o quadro as desenha.
fn da_cpu(motion: &MotionState) -> Vec<Copia> {
    let p = &motion.pump;
    let mut fora = Vec::new();
    let imagens = p.faixas.iter().flat_map(|f| {
        [
            (f.sink, f.instancias.clone()),
            (f.sink, f.ladrilhos.clone()),
        ]
    });
    com_donos(motion, imagens, &p.instances, da_imagem, &mut fora);
    let formas = p.faixas.iter().map(|f| (f.sink, f.vetores.clone()));
    com_donos(
        motion,
        formas,
        &p.vector_instances,
        |v, dono| {
            (v.tint[3] > 0.0).then_some(Copia {
                dono,
                pos: v.world_pos,
                size: v.size,
                basis: v.basis,
                anchor: v.anchor,
                geometry: v.geometry_id,
            })
        },
        &mut fora,
    );
    fora
}

/// As cópias do dispositivo: as linhas de sprite, as imagens de grupo e as formas do último cozimento.
fn do_dispositivo(motion: &MotionState, gpu: &GpuContext) -> Vec<Copia> {
    let (faixas_img, faixas_formas) = motion.gpu_cook.faixas_dos_sinks();
    let usize_de = |r: &Range<u32>| r.start as usize..r.end as usize;
    let mut fora = Vec::new();
    if let Some(i) = motion.gpu_cook.instances().filter(|i| i.len() > 0) {
        let linhas = ph2d_gpu_cook::read_instances(gpu, i);
        let faixas = faixas_img.iter().map(|(s, r)| (*s, usize_de(r)));
        com_donos(motion, faixas, &linhas, da_imagem, &mut fora);
    }
    // As imagens das saídas em grupo saíram do buffer das sprites para o da placa (P4).
    if let Some(g) = motion.gpu_cook.imagens_do_grupo() {
        let linhas = ph2d_gpu_cook::read_imagens_do_grupo(gpu, g);
        let faixas = g.faixas().iter().map(|(s, r)| (*s, usize_de(r)));
        com_donos(motion, faixas, &linhas, da_imagem, &mut fora);
    }
    if let Some(f) = motion.gpu_cook.formas() {
        let palavras = ph2d_gpu_cook::read_formas(gpu, f);
        let formas: &[ph2d_shape_gpu::ShapeInstance] = bytemuck::cast_slice(&palavras);
        let faixas = faixas_formas.iter().map(|(s, r)| (*s, usize_de(r)));
        com_donos(
            motion,
            faixas,
            formas,
            |s, dono| {
                (s.tint[3] > 0.0).then_some(Copia {
                    dono,
                    pos: s.pos,
                    size: s.size,
                    basis: s.basis,
                    anchor: s.anchor,
                    geometry: s.geometry,
                })
            },
            &mut fora,
        );
    }
    fora
}

#[cfg(test)]
#[path = "motion_pick_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_pick_gpu_tests.rs"]
mod gpu_tests;
