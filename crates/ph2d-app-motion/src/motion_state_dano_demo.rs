//! A cena `=136` — **VIDA E DANO NUM OBJECTO** ([doc 131 §3](../../../docs/Motion%20Nodes/131_zerar_o_ultimo_bloco.md), D.13).
//!
//! Um objecto tem UM `Motion Input` (vida OU dano), e a composição já diz o dano a partir da vida:
//! **`vida` → `On Change` (Fall) → `ADSR`** dispara só quando a vida DESCE — subir não dispara, e
//! nascer também não (o 1.º tique só regista), que é a lei do `Damage` do componente
//! (`health_publishes_its_ratio_and_damage_fires_on_each_drop`). O disparo sacode a galáxia
//! SIMULADA da `=134` (o redemoinho acelera, as estrelas incham).
//!
//! E a COR num gesto: o `Tint` das estrelas fica por ligar, e o cartão `cor` (Input Color) ao lado
//! — soltar o `r` dele no `r` do Tint liga os quatro canais (`grupo_de_cor`).

use super::jogo_demo::{entrada, forma, liga, no_em};
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// A vida de fábrica do cartão: cheia.
pub(crate) const VIDA: f32 = 1.0;
/// A cor de fábrica do cartão `cor` — a que o gesto leva às estrelas.
pub(crate) const COR: [f32; 4] = [0.35, 0.95, 0.55, 1.0];
/// Quantas vezes o redemoinho acelera e as estrelas incham no pico do dano (os da `=134`).
const EXPLOSAO: f32 = super::jogo_demo::EXPLOSAO;
const CLARAO: f32 = super::jogo_demo::CLARAO;

/// Os nós que os gates e o roteiro leem.
#[cfg_attr(not(test), expect(dead_code, reason = "lidos pelos gates da cena"))]
pub(crate) struct Cena {
    pub saida: NodeId,
    pub dano: NodeId,
}

/// O documento.
pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Cena> {
    let g = &mut doc.graph;
    let no = no_em(0.0);
    let (galaxia, redemoinho) = super::ritmo_demo::galaxia_e_redemoinho(g, 0.0)?;
    let vida = entrada(
        g,
        &no,
        "vida",
        ph2d_node_value_input::KIND_NUMBER,
        -1100.0,
        -330.0,
    );
    g.set_param(vida, ph2d_node_value_input::VALUE_PARAM, VIDA);
    // O DANO: a vida a descer (`direction` 1 = Fall); o `pre` guarda o valor do tique anterior.
    let dano = no(g, "pulse.on_change", -880.0, -330.0);
    g.set_param(dano, "direction", 1.0);
    liga(g, (vida, 0), dano, 0, false)?;
    liga(g, (dano, 0), dano, 1, true)?;
    let envelope = no(g, "pulse.adsr", -660.0, -330.0);
    for (p, v) in [
        ("attack", 0.03),
        ("decay", 0.15),
        ("sustain", 0.5),
        ("hold", 0.1),
        ("release", 0.5),
    ] {
        g.set_param(envelope, p, v);
    }
    liga(g, (dano, 0), envelope, 0, false)?;
    liga(g, (envelope, 0), envelope, 1, true)?;
    let mapa = |g: &mut ph2d_nodegraph::graph::Graph, dy: f32, lo: f32, hi: f32| {
        let m = no(g, "value.map_range", -440.0, dy);
        for (p, v) in [
            ("in_lo", 0.0),
            ("in_hi", 1.0),
            ("out_lo", lo),
            ("out_hi", hi),
            ("clamp", 1.0),
        ] {
            g.set_param(m, p, v);
        }
        m
    };
    let base = super::ritmo_demo::VORTEX;
    let forca = mapa(g, -330.0, base, base * EXPLOSAO);
    liga(g, (envelope, 0), forca, 0, false)?;
    g.drive_param(redemoinho, "strength", (forca, 0)).ok()?;
    let incha = mapa(g, -440.0, 1.0, CLARAO);
    liga(g, (envelope, 0), incha, 0, false)?;
    let clarao = no(g, "motion.drive", 0.0, 0.0);
    g.set_param(clarao, "channel", 3.0);
    g.set_param(clarao, "mode", 2.0);
    liga(g, (galaxia, 0), clarao, 0, false)?;
    liga(g, (incha, 0), clarao, 1, false)?;
    let estrela = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Star,
        0.13,
        [1.0, 1.0, 1.0],
        0.0,
        160.0,
    )?;
    let pecas = no(g, "motion.duplicator", 220.0, 100.0);
    g.set_param(pecas, "point_scale", 1.0);
    liga(g, (estrela, 0), pecas, 0, false)?;
    liga(g, (clarao, 0), pecas, 1, false)?;
    // A COR: o Tint por ligar e o cartão `cor` ao lado (o gesto é do roteiro).
    let cor = entrada(
        g,
        &no,
        "cor",
        ph2d_node_value_input::KIND_COLOR,
        220.0,
        -220.0,
    );
    for (canal, v) in ph2d_node_value_input::COLOR.iter().zip(COR) {
        g.set_param(cor, *canal, v);
    }
    let tint = no(g, "motion.tint", 440.0, 100.0);
    liga(g, (pecas, 0), tint, 0, false)?;
    let saida = no(g, "motion.output", 660.0, 100.0);
    liga(g, (tint, 0), saida, 0, false)?;
    g.validate(reg).ok()?;
    Some(Cena { saida, dano })
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|c| vec![c.saida])
}

/// A ficha do canvas.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    vec![crate::motion_demo_legend::Caption::new(
        [0.0, -2.6],
        "vida: baixar = dano (On Change Fall -> ADSR); cor: solte o r no r do Tint".to_string(),
    )]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[vida e dano] UMA galaxia de estrelas BRANCAS a girar (uma SIMULACAO com campos). Um\n\
         objecto so' tem UMA entrada do jogo - aqui a `vida` - e o DANO sai dela no grafo.\n\
         \n\
         (1) De PLAY. No cartao `Input` chamado `vida`, arraste `Value` para BAIXO (de 1 para\n    \
         0,7): as estrelas INCHAM e o redemoinho acelera; em dois segundos tudo acalma.\n\
         (2) Arraste `Value` para CIMA: nada acontece - curar nao e' dano.\n\
         (3) No grafo ha' DOIS cartoes `Input`: o da esquerda, ligado ao `On Change`, e' a `vida`; o\n    \
         `cor` e' o que esta' SOZINHO a' direita. Aproxime o grafo (roda do rato) ate' ler as\n    \
         saidas do `cor` (Out, R, G, B, A). Puxe um fio da saida `R` e solte-o\n    \
         em cima do cartao `Tint`: abre um menu; escolha `Tint: Color R`. Aparecem QUATRO fios\n    \
         e as estrelas ficam verdes. Um `Ctrl+Z` tira os quatro. (Um fio do `R` da `vida` nao\n    \
         liga: a `vida` e' um numero, e aparece um aviso a dize-lo.)\n\
         \n\
         DEU ERRADO se: subir a vida tambem sacudir a galaxia; abrir a cena ja' a sacudir;\n\
         ou o fio do `r` ligar so' um canal.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_dano_demo_tests.rs"]
mod tests;
