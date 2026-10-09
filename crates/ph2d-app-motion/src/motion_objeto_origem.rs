//! ⭐⭐ **A ORIGEM de um objecto Motion** — mover, girar ou escalar o objecto leva tudo o que os sinks
//! dele geram (escolha do dono, 08/10: *«vai junto»*, como no Cavalry). Doc ObjectsAndComponents 01
//! §13, F5.
//!
//! O grafo pensa no espaço do OBJECTO; a afim local → mundo do dono aplica-se à faixa de cada sink
//! DEPOIS do cozido (`world_pos' = L·p + t`, `basis' = L·B`), nas três saídas:
//!
//! | saída | onde |
//! |---|---|
//! | cópias de sprite (CPU) | [`aplica_cpu`], sobre `pump.instances` |
//! | cópias vectoriais (CPU) | [`aplica_cpu`], sobre `pump.vector_instances` |
//! | cópias na placa (sprites e formas) | [`crate::motion_objeto::placa`] |
//!
//! ⚠️ **Nunca duas vezes:** com a cena pausada a bomba NÃO recompõe (reusa as listas), e recozinhar
//! o mesmo tick avança a simulação de novo — por isso a CPU guarda a cópia LOCAL da última
//! composição e reaplica sobre ela quando só a afim mudou. ⭐ **Identidade = o caminho de sempre,
//! byte a byte**: sem objecto deslocado nada é copiado nem tocado.

use std::collections::BTreeMap;

use ph2d_eval_motion::VectorInstance;
use ph2d_nodegraph::graph::NodeId;
use ph2d_render::RenderInstance;

use super::{Afim, IDENTIDADE};
use crate::motion_state::MotionState;

/// O que a origem guarda entre quadros.
#[derive(Debug, Default)]
pub struct Origem {
    /// O passe da placa, compilado no 1.º quadro que o pede.
    passe: Option<ph2d_gpu_cook::instances::dono::PasseDoDono>,
    /// As listas como a bomba as compôs (espaço LOCAL), desde que alguma afim não é identidade.
    local: Option<(Vec<RenderInstance>, Vec<VectorInstance>)>,
    /// A composição da bomba a que [`Self::local`] corresponde.
    composicao: u64,
    /// A afim aplicada a cada sink nas listas da bomba.
    aplicada: BTreeMap<NodeId, Afim>,
}

/// O objecto Motion vivo dono de `no` (`MotionDoc::owner_of`), ou `None` sem dono vivo.
#[must_use]
pub fn dono_do_no(motion: &MotionState, no: NodeId) -> Option<&super::Dono> {
    motion
        .doc
        .owner_of(no)
        .and_then(|o| motion.objeto.donos.get(&o))
}

/// ⭐ A afim local → mundo do que `no` gera: a do dono dele, identidade sem dono vivo. Porta única —
/// os sinks (aqui) e as legendas das demos (`motion_demo_legend`) leem a mesma.
#[must_use]
pub fn afim_do_no(motion: &MotionState, no: NodeId) -> Afim {
    dono_do_no(motion, no).map_or(IDENTIDADE, |d| d.afim)
}

/// A afim de cada sink: a do dono dele, identidade sem dono vivo.
#[must_use]
pub fn afins_dos_sinks(motion: &MotionState) -> BTreeMap<NodeId, Afim> {
    motion
        .pump
        .faixas
        .iter()
        .map(|f| (f.sink, afim_do_no(motion, f.sink)))
        .collect()
}

/// `L·v` da parte linear de `a`.
#[must_use]
pub fn linear(a: &Afim, v: [f32; 2]) -> [f32; 2] {
    [a[0] * v[0] + a[2] * v[1], a[1] * v[0] + a[3] * v[1]]
}

/// `L·p + t`.
#[must_use]
pub fn ponto(a: &Afim, p: [f32; 2]) -> [f32; 2] {
    let l = linear(a, p);
    [l[0] + a[4], l[1] + a[5]]
}

/// `L·B` (base 2×2 por colunas).
#[must_use]
pub fn base(a: &Afim, b: [f32; 4]) -> [f32; 4] {
    let c0 = linear(a, [b[0], b[1]]);
    let c1 = linear(a, [b[2], b[3]]);
    [c0[0], c0[1], c1[0], c1[1]]
}

/// ⭐ **A origem na CPU** — depois da marcha do quadro e ANTES do LOD (que move cópias vectoriais
/// para o fim das sprites e partiria as faixas).
pub fn aplica_cpu(motion: &mut MotionState) {
    // ⭐ O caminho de sempre não aloca: nenhum objecto deslocado e nada aplicado antes ⇒ nada a fazer.
    if motion.objeto.origem.local.is_none()
        && motion.objeto.donos.values().all(|d| d.afim == IDENTIDADE)
    {
        motion.objeto.origem.composicao = motion.pump.composicoes;
        return;
    }
    let afins = afins_dos_sinks(motion);
    let todas_identidade = afins.values().all(|a| *a == IDENTIDADE);
    let nova = motion.pump.composicoes != motion.objeto.origem.composicao;
    let o = &mut motion.objeto.origem;
    if todas_identidade {
        // Volta ao espaço local se alguma afim tinha sido aplicada a estas listas.
        if !nova && let Some((i, v)) = o.local.take() {
            motion.pump.instances = i;
            motion.pump.vector_instances = v;
        }
        o.local = None;
        o.aplicada.clear();
        o.composicao = motion.pump.composicoes;
        return;
    }
    if nova {
        o.local = Some((
            motion.pump.instances.clone(),
            motion.pump.vector_instances.clone(),
        ));
        o.composicao = motion.pump.composicoes;
    } else if o.aplicada == afins {
        return;
    } else if let Some((i, v)) = &o.local {
        motion.pump.instances.clone_from(i);
        motion.pump.vector_instances.clone_from(v);
    }
    for f in &motion.pump.faixas {
        let Some(a) = afins.get(&f.sink).filter(|a| **a != IDENTIDADE) else {
            continue;
        };
        if let Some(lista) = motion.pump.instances.get_mut(f.instancias.clone()) {
            for inst in lista {
                inst.world_pos = ponto(a, inst.world_pos);
                inst.basis = base(a, inst.basis);
            }
        }
        if let Some(lista) = motion.pump.vector_instances.get_mut(f.vetores.clone()) {
            for vi in lista {
                vi.world_pos = ponto(a, vi.world_pos);
                vi.basis = base(a, vi.basis);
            }
        }
    }
    motion.objeto.origem.aplicada = afins;
}

/// ⭐ **A origem na placa** — depois do cozido na placa, que reescreve os buffers a cada quadro (por
/// isso no sítio, sem cópia local). Sem objecto deslocado, nada.
pub fn aplica_gpu(motion: &mut MotionState, gpu: &ph2d_gpu::GpuContext) {
    if motion.objeto.donos.values().all(|d| d.afim == IDENTIDADE) {
        return;
    }
    let (sprites, formas) = motion.gpu_cook.faixas_dos_sinks();
    let afins: BTreeMap<NodeId, Afim> = sprites
        .iter()
        .chain(formas)
        .filter_map(|(sink, _)| {
            let dono = motion.doc.owner_of(*sink)?;
            let a = motion.objeto.donos.get(&dono)?.afim;
            (a != IDENTIDADE).then_some((*sink, a))
        })
        .collect();
    if afins.is_empty() {
        return;
    }
    let passe = motion
        .objeto
        .origem
        .passe
        .get_or_insert_with(|| ph2d_gpu_cook::instances::dono::PasseDoDono::new(gpu));
    motion.gpu_cook.aplica_donos(gpu, passe, &afins);
}
