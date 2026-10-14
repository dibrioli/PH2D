//! `PH2D_MACRO_SMOKE=2` — **o Motion nó a nó** (`docs/Macros/01_plano_onda1.md` §6): um objecto Motion
//! sem nós no editor e com um `Macro` explicativo — a macro de tutorial ([`super::roteiro::CENA`]:
//! formas + simulação com vento num campo) gravada por script pelas portas do editor, um gesto a cada
//! [`ENTRE`] quadros com o relógio a correr. A shell monta o clipe (`add_macro_clip`, laço, Autoplay)
//! e põe a cena a correr: cada ligação muda o que se vê, e cada volta do laço recua a macro ao início.

use super::roteiro::{CENA, intent_de};
use crate::motion_bridge::quadro::{MaoDoEditor, o_quadro};
use crate::motion_scene_ctx::MotionSceneCtx;
use crate::motion_state::MotionState;
use ph2d_ecs::{Macro, ModoDaMacro, MotionGenerator, Name, StableId, Transform};
use ph2d_macro::{MacroRecorder, MacroTime, RefRect};
use ph2d_nodegraph::graph::NodeId;

const DT: f64 = 1.0 / 60.0;
/// O quadro do 1.º gesto.
const PRIMEIRO: u32 = 30;
/// Quadros entre dois gestos.
const ENTRE: u32 = 30;
/// Quadros depois do último gesto (o vento a soprar).
const CAUDA: u32 = 180;

/// O quadro em que cai o gesto `i`.
fn quadro_do_gesto(i: usize) -> u32 {
    PRIMEIRO + ENTRE * u32::try_from(i).unwrap_or(u32::MAX)
}

/// **A macro de tutorial gravada** como um artista a faria no editor do objecto `dono` (`StableId`):
/// os bytes do `Macro.doc` e a duração em segundos. `None` = a gravação recusou (já gravava).
#[must_use]
pub fn grava_o_tutorial(dono: u64) -> Option<(Vec<u8>, f64)> {
    let mut m = MotionState::new();
    m.doc = ph2d_motion_doc::MotionDoc::new();
    m.sinks.clear();
    m.gpu_enabled = false;
    m.objeto.editing = Some(dono);
    let mut rec = MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, 0);
    let trilha = m.gravacao_abre(&mut rec).ok()?;
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let mut split = ph2d_editor_core::screens::layout::CenterSplit::None;
    let mut ph = ph2d_core::Playhead::new(DT);
    ph.play();
    let fim = quadro_do_gesto(CENA.len() - 1) + CAUDA;
    let mut slots: Vec<NodeId> = Vec::new();
    let mut agora = 0;
    for q in 0..=fim {
        let antes: Vec<NodeId> = m.doc.graph.nodes().iter().map(|n| n.id).collect();
        let lote: Vec<_> = (0..CENA.len())
            .filter(|&i| quadro_do_gesto(i) == q)
            .map(|i| intent_de(CENA[i], &slots, i))
            .collect();
        ph.seek(f64::from(q) * DT);
        crate::motion_bridge::aplica_e_grava(&mut m, &mut ph, &mut toasts, &mut split, lote);
        // As edições de param que o grafo deixou ao painel: no editor fazem a volta pela fila
        // global e chegam ao dreno dos params; aqui vão direto à MESMA porta.
        m.ao_painel.selecao = None;
        let params = std::mem::take(&mut m.ao_painel.params);
        if !params.is_empty() {
            crate::motion_bridge::params::aplica_lote_de_params(
                &mut m,
                super::Destino::No,
                params,
                true,
                &mut toasts,
            );
        }
        m.macro_fecha();
        o_quadro(&mut m, None, &ph, DT, MaoDoEditor::default());
        agora = MacroTime::from_secs_f64(f64::from(q) * DT).0;
        for c in m.gravacao_passos() {
            rec.push(trilha, agora, c).ok()?;
        }
        let mut novos: Vec<NodeId> = m
            .doc
            .graph
            .nodes()
            .iter()
            .map(|n| n.id)
            .filter(|n| !antes.contains(n))
            .collect();
        novos.sort_by_key(|n| n.0);
        slots.extend(novos);
    }
    for c in m.gravacao_para() {
        rec.push(trilha, agora, c).ok()?;
    }
    Some((ph2d_macro::codec::write(&rec.finish()), f64::from(fim) * DT))
}

/// **A cena**: o objecto «Wind tutorial» com o `Macro` gravado. Devolve os bits da entidade e a
/// duração da macro, para a shell montar o clipe.
pub fn monta(cx: &mut MotionSceneCtx<'_>) -> Option<(u64, f64)> {
    let world = cx.sim.world_mut();
    let e = world
        .spawn((
            Name::new("Wind tutorial"),
            Transform::IDENTITY,
            MotionGenerator,
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(world);
    let dono = world.get::<StableId>(e).map_or(0, |s| s.0);
    let (doc, duracao) = grava_o_tutorial(dono)?;
    world.entity_mut(e).insert(Macro {
        doc,
        modo: ModoDaMacro::Explicativo,
    });
    eprintln!(
        "[macro smoke =2] 'Wind tutorial' is a Motion object with NO nodes of its own: a recorded macro \
         builds its graph node by node ({} gestures, {duracao:.1} s) — shapes in a grid, then a \
         simulation with wind blowing only inside a box field. The scene runs (top-bar Play) and the macro loops: every \
         loop goes back to the empty graph.",
        CENA.len()
    );
    Some((e.to_bits(), duracao))
}
