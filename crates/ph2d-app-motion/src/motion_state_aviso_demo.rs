//! A cena `=141` — **O AVISO QUE SE LÊ INTEIRO** ([doc 132 §5](../../../docs/Motion%20Nodes/132_zerar_as_seis_pendencias.md), P5).
//!
//! Uma galáxia SIMULADA (os campos da `=132`) de estrelas; um `Oscillator` ondula cada estrela (uma
//! corrente POR ELEMENTO) e um `Transform` leva-as ao `Output`. O gesto do roteiro — um fio da saída
//! do `Oscillator` solto no CORPO do `Transform` — não liga, não abre menu nenhum, e diz porquê com o
//! aviso mais longo do Motion (`can_t_drive_that_output_is_a_per_element_stream` + a cura). O `LFO`
//! ao lado é o contraste: um VALOR, e o mesmo fio dele abre o menu dos params do `Transform`.
//!
//! `PH2D_MOTION_AVISO_AUTO=<segundos>` faz o gesto sozinho UMA vez, pelo MESMO caminho (o
//! `GraphIntent::DropRefused` que o painel emite ao soltar o fio), para a foto.

use super::jogo_demo::{forma, liga, no_em};
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};
use std::time::Duration;

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
    // Abaixo da galáxia: a -3,2 a ponta de baixo dela tapava-se (régua `motion_demo_legend::tapa`, 15/10).
    vec![crate::motion_demo_legend::Caption::new(
        [0.0, -3.78],
        "Oscillator wire dropped on the Transform: no link, no menu, the warning says why"
            .to_string(),
    )]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8) — o gesto REAL (o fio solto no CORPO do
/// cartão), não o menu que o fio de uma corrente nunca abre.
pub(super) fn announce() {
    eprintln!(
        "\n[aviso inteiro] UMA galaxia de estrelas DOURADAS a girar (uma SIMULACAO com campos); o\n\
         cartao `Oscillator` faz cada estrela ondular - uma onda POR ESTRELA, nao um numero.\n\
         \n\
         (1) De PLAY: as estrelas giram e tremem de leve.\n\
         (2) Aproxime o grafo (roda do rato) ate' ler os cartoes. Puxe um fio da saida `Out` do\n    \
         cartao `Oscillator` e solte-o em cima do cartao `Transform`: NAO liga, NAO abre nenhum\n    \
         menu, e aparece no TOPO um aviso amarelo INTEIRO: diz que aquela saida e' uma corrente\n    \
         por estrela, nao um valor, e que no' usar para ler um numero dela (`Attribute`). Some\n    \
         sozinho em ~5 s.\n\
         (3) O controlo: puxe um fio da saida `Out` do cartao `LFO` (em cima) e solte-o em cima do\n    \
         cartao `Transform`: abre um menu com os parametros dele; escolha `Transform: Scale`.\n    \
         Este LIGA, e as estrelas passam a crescer e encolher juntas.\n\
         (Com `PH2D_MOTION_AVISO_AUTO=<segundos>` o passo (2) faz-se sozinho UMA vez, nesse\n    \
         instante depois de a cena abrir.)\n\
         \n\
         DEU ERRADO se: o passo (2) abrir a janela de escolher nos ou um menu; o aviso acabar em\n\
         `...` (cortado); sumir antes de o acabar de ler; NAO sumir (ou voltar); ou o fio do\n\
         passo (2) ficar desenhado.\n"
    );
}

/// O gesto do passo (2), como o painel o emite ao soltar o fio da saída `Out` do `Oscillator` no
/// CORPO do `Transform` (`interact_drop::resolve_loose_output_drop`: nenhuma entrada livre
/// compatível, nenhuma fileira no menu ⇒ `DropRefused`).
#[must_use]
pub(crate) fn o_gesto(g: &Graph) -> Option<ph2d_panel_motion_graph::GraphIntent> {
    let de = |tipo: &str| {
        g.nodes()
            .iter()
            .find(|n| n.type_name == tipo)
            .map(|n| n.id.0)
    };
    Some(ph2d_panel_motion_graph::GraphIntent::DropRefused {
        from_node: de("motion.oscillator")?,
        from_port: 0,
        to_node: de("motion.transform")?,
    })
}

/// O instante do gesto automático: `PH2D_MOTION_AVISO_AUTO=<segundos>` (`1` ≈ 1 s), só na `=141`.
#[must_use]
pub(crate) fn espera_do_ambiente(auto: Option<&str>, cena: Option<&str>) -> Option<Duration> {
    let s: f32 = auto?.trim().parse().ok()?;
    (cena == Some("141") && s.is_finite() && s > 0.0).then(|| Duration::from_secs_f32(s))
}

/// ⛔ **O gesto automático dispara UMA vez** (smoke do dono, 13/10: *«o aviso não some só»* — ele
/// repetia-se sempre que o aviso saía do ecrã, e o aviso nunca sumia). A lei, sem ambiente nem
/// relógio de parede.
pub(crate) struct GestoAuto {
    espera: Duration,
    feito: bool,
}

impl GestoAuto {
    pub(crate) fn novo(espera: Duration) -> Self {
        Self {
            espera,
            feito: false,
        }
    }

    /// O gesto, no primeiro passo em que `decorrido` alcança a espera — e nunca mais.
    pub(crate) fn passo(
        &mut self,
        g: &Graph,
        decorrido: Duration,
    ) -> Option<ph2d_panel_motion_graph::GraphIntent> {
        if self.feito || decorrido < self.espera {
            return None;
        }
        let gesto = o_gesto(g)?;
        self.feito = true;
        Some(gesto)
    }
}

/// Chamado pela ponte, com a ferramenta Motion em mãos, ANTES de drenar os gestos do quadro. O
/// relógio conta da primeira chamada (a ferramenta em mãos), em segundos de parede.
pub(crate) fn auto_gesto(g: &Graph) {
    use std::sync::{Mutex, OnceLock, PoisonError};
    use std::time::Instant;
    static ESPERA: OnceLock<Option<Duration>> = OnceLock::new();
    static ESTADO: Mutex<Option<(Instant, GestoAuto)>> = Mutex::new(None);
    let Some(espera) = *ESPERA.get_or_init(|| {
        espera_do_ambiente(
            std::env::var("PH2D_MOTION_AVISO_AUTO").ok().as_deref(),
            std::env::var("PH2D_GPU_COOK_DEMO").ok().as_deref(),
        )
    }) else {
        return;
    };
    let mut estado = ESTADO.lock().unwrap_or_else(PoisonError::into_inner);
    let (t0, auto) = estado.get_or_insert_with(|| (Instant::now(), GestoAuto::novo(espera)));
    if let Some(gesto) = auto.passo(g, t0.elapsed()) {
        ph2d_panel_motion_graph::push_intent(gesto);
    }
}
