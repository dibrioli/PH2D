//! `PH2D_MACRO_SMOKE=2` — **o Motion nó a nó** (`docs/Macros/01_plano_onda1.md` §6): um objecto Motion
//! sem nós no editor e com um `Macro` explicativo — a macro de tutorial ([`super::roteiro::CENA`]:
//! formas + simulação com vento num campo) gravada por script pelas portas do editor, um gesto a cada
//! [`ENTRE`] quadros com o relógio a correr. A shell monta o clipe (`add_macro_clip`, laço, Autoplay)
//! e põe a cena a correr: cada ligação muda o que se vê, e cada volta do laço recua a macro ao início.

use super::roteiro::{CENA, Gesto, intent_de};
use crate::motion_bridge::quadro::{MaoDoEditor, o_quadro};
use crate::motion_scene_ctx::MotionSceneCtx;
use crate::motion_state::MotionState;
use ph2d_ecs::{Macro, ModoDaMacro, MotionGenerator, Name, StableId, Transform};
use ph2d_macro::{MacroRecorder, MacroTime, RefRect};
use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_graph::GraphIntent;

const DT: f64 = 1.0 / 60.0;
/// O quadro do 1.º gesto.
const PRIMEIRO: u32 = 30;
/// Quadros entre dois gestos.
pub(crate) const ENTRE: u32 = 30;
/// Quadros depois do último gesto (o vento a soprar).
const CAUDA: u32 = 180;

/// O quadro em que cai o gesto `i`, com `antes[j]` quadros a mais antes do gesto `j`.
fn quadro_do_gesto(i: usize, antes: &[u32]) -> u32 {
    let mais: u32 = antes.iter().take(i + 1).sum();
    PRIMEIRO + ENTRE * u32::try_from(i).unwrap_or(u32::MAX) + mais
}

/// **A macro de tutorial gravada** como um artista a faria no editor do objecto `dono` (`StableId`):
/// os bytes do `Macro.doc` e a duração em segundos. `None` = a gravação recusou (já gravava).
#[must_use]
pub fn grava_o_tutorial(dono: u64) -> Option<(Vec<u8>, f64)> {
    grava_gestos(dono, CENA, &[])
}

/// Os `gestos` gravados como a cena, com `mais[i]` quadros a mais antes do gesto `i`.
pub(crate) fn grava_gestos(dono: u64, gestos: &[Gesto], mais: &[u32]) -> Option<(Vec<u8>, f64)> {
    grava(dono, gestos, mais, 0, None)
}

/// Os `gestos` com a gravação armada só a meio do intervalo antes do gesto `arma`: os anteriores são o
/// ESTADO INICIAL da trilha (o tutorial «fazer» parte de um grafo meio feito). `lugar` põe cada nó
/// novo pelo tipo.
pub(crate) fn grava_desde(
    dono: u64,
    gestos: &[Gesto],
    arma: usize,
    lugar: Option<fn(&str) -> (f32, f32)>,
) -> Option<(Vec<u8>, f64)> {
    grava(dono, gestos, &[], arma, lugar)
}

fn grava(
    dono: u64,
    gestos: &[Gesto],
    mais: &[u32],
    arma: usize,
    lugar: Option<fn(&str) -> (f32, f32)>,
) -> Option<(Vec<u8>, f64)> {
    let mut m = MotionState::new();
    m.doc = ph2d_motion_doc::MotionDoc::new();
    m.sinks.clear();
    m.gpu_enabled = false;
    m.objeto.editing = Some(dono);
    let mut rec = MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, 0);
    let q0 = if arma == 0 {
        0
    } else {
        quadro_do_gesto(arma, mais) - ENTRE / 2
    };
    let mut trilha = None;
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let mut split = ph2d_editor_core::screens::layout::CenterSplit::None;
    let mut ph = ph2d_core::Playhead::new(DT);
    ph.play();
    let fim = quadro_do_gesto(gestos.len() - 1, mais) + CAUDA;
    let mut slots: Vec<NodeId> = Vec::new();
    let mut agora = 0;
    for q in 0..=fim {
        if q == q0 {
            trilha = Some(m.gravacao_abre(&mut rec).ok()?);
        }
        let antes: Vec<NodeId> = m.doc.graph.nodes().iter().map(|n| n.id).collect();
        let lote: Vec<_> = (0..gestos.len())
            .filter(|&i| quadro_do_gesto(i, mais) == q)
            .map(|i| match (intent_de(gestos[i], &slots, i), lugar) {
                (GraphIntent::AddNode { type_name, .. }, Some(f)) => {
                    let (x, y) = f(type_name);
                    GraphIntent::AddNode { type_name, x, y }
                }
                (g, _) => g,
            })
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
        agora = MacroTime::from_secs_f64(f64::from(q - q0.min(q)) * DT).0;
        if let Some(t) = trilha {
            for c in m.gravacao_passos() {
                rec.push(t, agora, c).ok()?;
            }
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
    let trilha = trilha?;
    for c in m.gravacao_para() {
        rec.push(trilha, agora, c).ok()?;
    }
    Some((
        ph2d_macro::codec::write(&rec.finish()),
        f64::from(fim - q0) * DT,
    ))
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

/// O 1.º gesto da [`CENA`] que a cena `=3` grava: as três ligações que fazem o vento soprar e a caixa a
/// crescer. Os anteriores são o grafo com que o aluno começa.
pub(crate) const FAZER_DESDE: usize = 19;

/// O lugar de cada nó da cena `=3` no editor: a cadeia que o aluno liga (caixa → vento → arrasto →
/// integrador) numa fila legível, à vista sem deslocar o editor.
pub(crate) fn lugar_do_fazer(tipo: &str) -> (f32, f32) {
    match tipo {
        "source.shape" => (0.0, 0.0),
        "motion.duplicator" => (240.0, 0.0),
        "motion.output" => (480.0, 0.0),
        "motion.grid" => (0.0, 170.0),
        "motion.integrate" => (480.0, 170.0),
        "field.box" => (0.0, 340.0),
        "force.wind" => (240.0, 340.0),
        _ => (480.0, 340.0),
    }
}

/// `PH2D_MACRO_SMOKE=3` — **o tutorial «fazer»** (`docs/Macros/03_plano_W6.md`): o objecto «Wind, your
/// turn» em Edit, com o grafo da `=2` MENOS as três ligações do vento, e um `Macro` em modo `Fazer` que
/// as grava. O aluno liga-as com o balão a apontar as portas. Devolve os bits da entidade.
pub fn monta_fazer(cx: &mut MotionSceneCtx<'_>) -> Option<u64> {
    let world = cx.sim.world_mut();
    let e = world
        .spawn((
            Name::new("Wind, your turn"),
            Transform::IDENTITY,
            MotionGenerator,
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(world);
    let dono = world.get::<StableId>(e).map_or(0, |s| s.0);
    let (doc, _) = grava_desde(dono, CENA, FAZER_DESDE, Some(lugar_do_fazer))?;
    let track = ph2d_macro::codec::read(&doc)
        .ok()?
        .tracks
        .into_iter()
        .next()?;
    super::tutorial::recomeca(cx.motion, dono, &track).ok()?;
    // O grafo inicial é a cena, não um passo que o Ctrl+Z do aluno desfaça.
    cx.motion.history = ph2d_motion_doc::MotionHistory::new();
    world.entity_mut(e).insert(Macro {
        doc,
        modo: ModoDaMacro::Fazer,
    });
    cx.motion.objeto.editing = Some(dono);
    let _ = cx
        .tools
        .set_active(&ph2d_editor_core::ToolId::new("motion"));
    cx.motion.pump.mark_dirty();
    cx.playhead.play();
    eprintln!(
        "[macro smoke =3] 'Wind, your turn' is a TUTORIAL: its grid of shapes and the wind nodes are \
         there, but three links are missing. A balloon says which one to make and rings the two \
         ports; link them in the node editor and the balloon moves on (the wind starts blowing at the \
         third). Ctrl+Z takes the balloon back. 'Show me' makes the step for you; 'Restart' empties \
         it again."
    );
    Some(e.to_bits())
}
