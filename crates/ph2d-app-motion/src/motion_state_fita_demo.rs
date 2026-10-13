//! A cena `=139` — **A FITA QUE DOBRA** ([doc 132 §2](../../../docs/Motion%20Nodes/132_zerar_as_seis_pendencias.md), P2).
//!
//! A galáxia SIMULADA da `=134` (formas: estrelas; campos: núcleo, redemoinho, ímã), sacudida por
//! um gatilho `batida` que um quadrado lento dispara; e [`GATILHOS`] gatilhos `sinal_*` disparados
//! por um seno rápido — o jogo de quem ainda não tem jogo: muitas entradas a mudar em todo tique.
//! Com o tecto da fita pequeno ([`ph2d_eval_motion::razao::VAR_DO_TECTO_DA_FITA`]) a história
//! dobra em segundos: a faixa da Timeline começa depois do 0, e arrastar para antes dela faz o
//! cartão de cada `Input` dizer que mostra a base.

use super::jogo_demo::{entrada, forma, liga, no_em};
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_node_value_input::{FIRE, KIND_TRIGGER};
use ph2d_nodegraph::graph::NodeId;

/// Os gatilhos que o seno dispara em todo tique.
pub(crate) const GATILHOS: usize = 16;
/// O tecto que o roteiro põe na linha de comando: a dobra chega em ~2–3 s a 60 Hz (o gate mede).
pub(crate) const TECTO_DO_ROTEIRO: &str = "384k";
/// ⭐ **O segundo em que a cena PÁRA o relógio.** A ferramenta Motion toca o relógio do editor ao
/// entrar, e a fita grava os tiques DELE: sem paragem, 25 s depois de abrir a faixa está em
/// `[22, 25]` s, fora da régua — a cena ensinaria que ela não existe. Parado aqui, a faixa
/// `[~2,7 s, 5 s]` fica na régua para sempre (até o artista tocar de novo).
pub(crate) const PARA_EM_S: f64 = 5.0;

/// O nó que o roteador e o gate leem.
pub(crate) struct Cena {
    pub saida: NodeId,
}

/// O documento.
pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Cena> {
    let g = &mut doc.graph;
    let no = no_em(0.0);
    let (galaxia, redemoinho) = super::ritmo_demo::galaxia_e_redemoinho(g, 0.0)?;
    // A BATIDA: um quadrado de 1,5 s dispara o gatilho; o envelope acelera o redemoinho.
    let quadrado = no(g, "value.lfo", -1320.0, -330.0);
    g.set_param(quadrado, "wave", 2.0);
    g.set_param(quadrado, "period", 1.5);
    let batida = entrada(g, &no, "batida", KIND_TRIGGER, -1100.0, -330.0);
    g.drive_param(batida, FIRE, (quadrado, 0)).ok()?;
    let envelope = no(g, "pulse.adsr", -880.0, -330.0);
    for (p, v) in [
        ("attack", 0.03),
        ("decay", 0.15),
        ("sustain", 0.5),
        ("hold", 0.1),
        ("release", 0.5),
    ] {
        g.set_param(envelope, p, v);
    }
    // A porta `5` do gatilho é o PULSO (a da `=134`).
    liga(g, (batida, 5), envelope, 0, false)?;
    liga(g, (envelope, 0), envelope, 1, true)?;
    let forca = no(g, "value.map_range", -660.0, -330.0);
    let base = super::ritmo_demo::VORTEX;
    for (p, v) in [
        ("in_lo", 0.0),
        ("in_hi", 1.0),
        ("out_lo", base),
        ("out_hi", base * super::jogo_demo::EXPLOSAO),
        ("clamp", 1.0),
    ] {
        g.set_param(forca, p, v);
    }
    liga(g, (envelope, 0), forca, 0, false)?;
    g.drive_param(redemoinho, "strength", (forca, 0)).ok()?;
    // AS ESTRELAS sobre os pontos da galáxia.
    let estrela = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Star,
        0.13,
        [1.0, 0.85, 0.4],
        0.0,
        160.0,
    )?;
    let pecas = no(g, "motion.duplicator", 220.0, 100.0);
    g.set_param(pecas, "point_scale", 1.0);
    liga(g, (estrela, 0), pecas, 0, false)?;
    liga(g, (galaxia, 0), pecas, 1, false)?;
    let saida = no(g, "motion.output", 440.0, 100.0);
    liga(g, (pecas, 0), saida, 0, false)?;
    // AS MUITAS ENTRADAS: um seno de 0,5 s no `fire` de cada gatilho — um disparo por tique.
    let seno = no(g, "value.lfo", -1540.0, -660.0);
    g.set_param(seno, "period", 0.5);
    for i in 0..GATILHOS {
        #[expect(clippy::cast_precision_loss, reason = "dezasseis cartões")]
        let (col, lin) = ((i % 4) as f32, (i / 4) as f32);
        let e = entrada(
            g,
            &no,
            &format!("sinal_{i}"),
            KIND_TRIGGER,
            -1320.0 + 220.0 * col,
            -660.0 - 110.0 * lin,
        );
        g.drive_param(e, FIRE, (seno, 0)).ok()?;
    }
    g.validate(reg).ok()?;
    Some(Cena { saida })
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    crate::motion_demo_legend::pede_paragem(PARA_EM_S);
    // A régua que repete a fita é a da `Arrange` (o relógio do editor); a `Keys` não a pinta.
    crate::motion_demo_legend::pede_a_aba_arrange();
    monta(doc, reg).map(|c| vec![c.saida])
}

/// A ficha do canvas.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    vec![crate::motion_demo_legend::Caption::new(
        [0.0, -2.6],
        "fita das entradas: toca 5 s e para; a faixa verde da Timeline comeca depois do 0"
            .to_string(),
    )]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    let var = ph2d_eval_motion::razao::VAR_DO_TECTO_DA_FITA;
    let tecto = ph2d_eval_motion::Razao::da_fita().tecto();
    eprintln!(
        "\n[a fita que dobra] UMA galaxia de estrelas a girar (uma SIMULACAO com campos), sacudida\n\
         de 1,5 em 1,5 s pelo gatilho `batida`. Os {GATILHOS} cartoes `sinal_*` disparam em TODO tique:\n\
         a fita das entradas guarda o que se publicou em cada tique, para o scrub o repetir.\n\
         Tecto da fita agora: {tecto} bytes (lance com {var}={TECTO_DO_ROTEIRO} para dobrar em segundos).\n\
         \n\
         (1) NAO carregue em nada: a cena TOCA sozinha 5 s e PARA, com a Timeline na aba\n    \
         `Arrange` (a regua do relogio que o Motion le). Por baixo dos segundos da regua cresce\n    \
         uma FAIXA VERDE: e' a historia gravada. Aos ~3 s ela deixa de comecar no 0 - o que\n    \
         ficou para tras foi DOBRADO. Parada, a faixa fica de ~2,7 s a 5 s.\n\
         (2) Arraste a regua para DENTRO da faixa: os cartoes `Input` mostram o numero de sempre.\n    \
         (Na aba `Keys` a faixa SOME: aquela regua mexe o relogio do clip, nao o do Motion.)\n\
         (3) Arraste a regua para ANTES do inicio da faixa: o cartao de cada `Input` diz\n    \
         `History before X s was folded - showing X s` - o replay mostra o segundo X, nao o\n    \
         que se publicou ali.\n\
         \n\
         DEU ERRADO se: a faixa nao aparecer ou comecar no 0 com o tecto pequeno; o relogio\n\
         nao parar aos 5 s; o cartao ficar calado antes da faixa; ou o aviso aparecer DENTRO dela.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_fita_demo_tests.rs"]
mod tests;
