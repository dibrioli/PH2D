//! A cena `=141` — **O AVISO QUE SE LÊ INTEIRO** ([doc 132 §5](../../../docs/Motion%20Nodes/132_zerar_as_seis_pendencias.md), P5).
//!
//! Uma galáxia SIMULADA (os campos da `=132`) de estrelas; um `Oscillator` ondula cada estrela (uma
//! corrente POR ELEMENTO) e um `Transform` leva-as ao `Output`. O gesto do roteiro — um fio da saída
//! do `Oscillator` para o `Scale` do `Transform` — é recusado com o aviso mais longo do Motion
//! (`can_t_drive_that_output_is_a_per_element_stream` + a cura). O `LFO` ao lado é o contraste: um
//! VALOR, e o mesmo fio dele liga.
//!
//! `PH2D_MOTION_AVISO_AUTO=1` faz o gesto sozinho pelo MESMO caminho (um `GraphIntent::DriveParam`
//! na fila do painel), e repete-o sempre que o aviso sai do ecrã — para a foto.

use super::jogo_demo::{forma, liga, no_em};
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};

/// O param escalar que o gesto tenta conduzir.
pub(crate) const PARAM: &str = "scale";
/// Quadros com a ferramenta em mãos antes do 1.º gesto automático (~1 s a 60 Hz).
pub(crate) const ESPERA_QUADROS: u32 = 60;
const ESTRELA: f32 = 0.13;
const ONDA: f32 = 0.06;

/// Os nós que os gates leem (o produto só usa a `saida`).
#[cfg_attr(not(test), expect(dead_code, reason = "lidos pelos gates da cena"))]
pub(crate) struct Cena {
    pub saida: NodeId,
    pub oscilador: NodeId,
    pub transform: NodeId,
    pub lfo: NodeId,
}

/// O documento.
pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Cena> {
    let g = &mut doc.graph;
    let no = no_em(0.0);
    let galaxia = super::ritmo_demo::galaxia(g, 0.0)?;
    let oscilador = no(g, "motion.oscillator", -220.0, 0.0);
    g.set_param(oscilador, "amplitude", ONDA);
    g.set_param(oscilador, "frequency", 0.5);
    liga(g, (galaxia, 0), oscilador, 0, false)?;
    let estrela = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Star,
        ESTRELA,
        [0.98, 0.82, 0.35],
        -220.0,
        160.0,
    )?;
    let pecas = no(g, "motion.duplicator", 0.0, 0.0);
    g.set_param(pecas, "point_scale", 1.0);
    liga(g, (estrela, 0), pecas, 0, false)?;
    liga(g, (oscilador, 0), pecas, 1, false)?;
    let transform = no(g, "motion.transform", 220.0, 0.0);
    liga(g, (pecas, 0), transform, 0, false)?;
    let saida = no(g, "motion.output", 440.0, 0.0);
    liga(g, (transform, 0), saida, 0, false)?;
    let lfo = no(g, "value.lfo", 0.0, -160.0);
    g.set_param(lfo, "amplitude", 0.3);
    g.set_param(lfo, "offset", 1.0);
    liga(g, (galaxia, 0), lfo, 0, false)?;
    g.validate(reg).ok()?;
    Some(Cena {
        saida,
        oscilador,
        transform,
        lfo,
    })
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|c| vec![c.saida])
}

/// A ficha do canvas.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    vec![crate::motion_demo_legend::Caption::new(
        [0.0, -3.2],
        "fio do Oscillator no Scale do Transform: o aviso tem de se ler inteiro".to_string(),
    )]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[aviso inteiro] UMA galaxia de estrelas DOURADAS a girar (uma SIMULACAO com campos); o\n\
         cartao `Oscillator` faz cada estrela ondular - uma onda POR ESTRELA, nao um numero.\n\
         \n\
         (1) De PLAY: as estrelas giram e tremem de leve.\n\
         (2) Aproxime o grafo (roda do rato) ate' ler o cartao `Oscillator`. Puxe um fio da saida\n    \
         `Out` dele e solte-o em cima do cartao `Transform`: abre um menu; escolha\n    \
         `Transform: Scale`. NAO liga, e aparece no TOPO um aviso amarelo. Leia-o INTEIRO: ele\n    \
         diz que aquela saida e' uma corrente por estrela, nao um valor, e diz que no' inserir\n    \
         para ler um numero dela. Ele fica no ecra' o tempo de o ler.\n\
         (3) Faca o mesmo a partir da saida `Out` do cartao `LFO` (em cima): este LIGA, e as\n    \
         estrelas passam a crescer e encolher juntas.\n\
         (Com `PH2D_MOTION_AVISO_AUTO=1` o passo (2) faz-se sozinho ~1 s depois de a cena abrir,\n    \
         e repete-se sempre que o aviso sai do ecra'.)\n\
         \n\
         DEU ERRADO se: o aviso acabar em `...` (cortado); sumir antes de o acabar de ler; tapar\n\
         uma barra de progresso; ou o fio do passo (2) ficar desenhado.\n"
    );
}

/// O começo do aviso, sem a cura — o que o gesto automático procura na fila.
#[must_use]
pub(crate) fn cabeca_do_aviso() -> String {
    ph2d_i18n::tr_with(
        "app.motion.motion_bridge_subgraph.can_t_drive_that_output_is_a_per_element_stream",
        &[("cura", &"")],
    )
}

/// O gesto do passo (2), como o painel o emite ao escolher `Transform: Scale` no menu.
#[must_use]
pub(crate) fn o_gesto(g: &Graph) -> Option<ph2d_panel_motion_graph::GraphIntent> {
    let de = |tipo: &str| {
        g.nodes()
            .iter()
            .find(|n| n.type_name == tipo)
            .map(|n| n.id.0)
    };
    Some(ph2d_panel_motion_graph::GraphIntent::DriveParam {
        from_node: de("motion.oscillator")?,
        from_port: 0,
        to_node: de("motion.transform")?,
        param: PARAM,
    })
}

/// A lei do gesto automático, sem ambiente nem relógio: depois da espera, e só quando o aviso
/// não está no ecrã.
#[must_use]
pub(crate) fn gesto_se_mudo(
    g: &Graph,
    toasts: &ph2d_editor_core::ToastQueue,
    quadro: u32,
) -> Option<ph2d_panel_motion_graph::GraphIntent> {
    if quadro < ESPERA_QUADROS {
        return None;
    }
    let cabeca = cabeca_do_aviso();
    if toasts.iter().any(|t| t.message.starts_with(&cabeca)) {
        return None;
    }
    o_gesto(g)
}

fn auto_ligado() -> bool {
    static L: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *L.get_or_init(|| {
        std::env::var("PH2D_MOTION_AVISO_AUTO").is_ok_and(|v| v == "1")
            && std::env::var("PH2D_GPU_COOK_DEMO").is_ok_and(|v| v == "141")
    })
}

/// Chamado pela ponte, com a ferramenta Motion em mãos, ANTES de drenar os gestos do quadro.
pub(crate) fn auto_gesto(g: &Graph, toasts: &ph2d_editor_core::ToastQueue) {
    use std::sync::atomic::{AtomicU32, Ordering};
    static QUADRO: AtomicU32 = AtomicU32::new(0);
    if !auto_ligado() {
        return;
    }
    let q = QUADRO.fetch_add(1, Ordering::Relaxed);
    if let Some(gesto) = gesto_se_mudo(g, toasts, q) {
        ph2d_panel_motion_graph::push_intent(gesto);
    }
}
