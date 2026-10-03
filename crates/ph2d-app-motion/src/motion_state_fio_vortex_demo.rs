//! ⭐⭐⭐ **UM NUMBER NO STRENGTH DO VORTEX** (cena `=128`) — a cena do report do Enio de 2026-10-03:
//! *«ligar a saída de um Number à entrada Strength de um Vortex: o fio aparece, mas o valor não tem
//! efeito»* e *«Number não aceita valores negativos»*.
//!
//! # O que a medição achou (handoff 03/10 §6.3)
//!
//! O cozimento obedecia ao fio nas DUAS rotas (`drive_force_tests`); quem mentia era o CARTÃO do
//! Vortex, que desenhava o `Strength` AUTORADO enquanto a força usava o número do fio. E o Number
//! ligado veste a faixa do destino, que era `0..40`: o arrasto parava no zero.
//!
//! # A CADEIA
//!
//! A galáxia legível da `=127` (as estrelas com contorno, a MESMA simulação — reutilizada, não
//! copiada), sem o esticão, e um `value.number` ligado ao `strength` do `force.vortex`. O cartão do
//! Vortex fica com o `2` da galáxia; o Number começa noutro número, e é ELE que manda.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

use super::traco_esticado_demo::{LEGIVEL, params_da_forma};

/// O número com que o Number nasce — DIFERENTE do `2` do cartão do Vortex, para a row ligada do
/// Vortex ler o do fio à primeira vista.
pub(crate) const FIO_INICIAL: f32 = 4.0;

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|(sinks, _, _)| sinks)
}

/// O documento, e os dois nós que a cena liga: `(saídas, number, vortex)`.
pub(crate) fn monta(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<(Vec<NodeId>, NodeId, NodeId)> {
    for tipo in [
        "motion.grid",
        "motion.duplicator",
        "source.shape",
        "motion.output",
        "motion.integrate",
        "force.vortex",
        "value.number",
    ] {
        reg.manifests()
            .find(|m| m.id == ph2d_nodegraph::node::NodeTypeId::of(tipo))?;
    }
    let arranjo = LEGIVEL;
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };
    let grade = no(g, "motion.grid", 0.0, 0.0);
    #[expect(clippy::cast_precision_loss, reason = "um lado de grelha pequeno")]
    let lado = arranjo.lado as f32;
    g.set_param(grade, "rows", lado);
    g.set_param(grade, "cols", lado);
    g.set_param(grade, "gap_x", arranjo.vao());
    g.set_param(grade, "gap_y", arranjo.vao());
    let forma = no(g, "source.shape", 0.0, 220.0);
    for (chave, valor) in params_da_forma(arranjo) {
        g.set_param(forma, chave, valor);
    }
    let ig = super::carimbo_demo::simulacao_com(g, grade, &arranjo.galaxia())?;
    let vortex = g
        .nodes()
        .iter()
        .find(|n| n.type_name == "force.vortex")
        .map(|n| n.id)?;
    let dup = no(g, "motion.duplicator", 240.0, 110.0);
    for (de, porta) in [(forma, 0u16), (ig, 1)] {
        g.connect(Edge {
            from: (de, 0),
            to: (dup, porta),
            delayed: false,
        })
        .ok()?;
    }
    let saida = no(g, "motion.output", 460.0, 110.0);
    g.connect(Edge {
        from: (dup, 0),
        to: (saida, 0),
        delayed: false,
    })
    .ok()?;
    // ── O FIO: um Number por cima do cartão do Vortex, ligado ao `Strength`.
    let numero = no(g, "value.number", 160.0, -400.0);
    g.set_param(numero, "value", FIO_INICIAL);
    g.drive_param(vortex, "strength", (numero, 0)).ok()?;
    Some((vec![saida], numero, vortex))
}

/// **O roteiro que o dono segue.** Cada passo nomeia o que aparece NA TELA (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    let n = LEGIVEL.estrelas();
    let f = FIO_INICIAL;
    eprintln!(
        "\n[number no vortex] {n} ESTRELAS AMARELAS que GIRAM como uma galaxia: e' uma SIMULACAO com\n\
         campos de forca. O REDEMOINHO e' o cartao `Vortex`; por cima dele ha' um cartao `Number`\n\
         ligado por um FIO a' linha `Strength` do Vortex.\n\
         \n\
         (1) Olhe o cartao `Vortex`: a linha `Strength` mostra {f} (o numero que o fio traz, numa\n    \
         cor diferente), e nao o 2 que estava escrito no cartao.\n\
         (2) No cartao `Number`, arraste o numero para a DIREITA (ate' perto de 10): a galaxia gira\n    \
         MAIS DEPRESSA (e abre-se devagar: gira mais do que o ima segura), e o\n    \
         `Strength` do Vortex acompanha o numero.\n\
         (3) Arraste o Number ate' 0: o redemoinho PARA, e as estrelas ficam quase paradas.\n\
         (4) Arraste para a ESQUERDA, abaixo de zero (ate' perto de -4): a galaxia volta a girar,\n    \
         AO CONTRARIO.\n\
         (5) Clique no numero do Number, escreva -2 e carregue Enter: fica -2, e gira ao contrario.\n\
         \n\
         DEU ERRADO se: o `Strength` do Vortex ficar em 2 enquanto o Number muda; se a galaxia nao\n\
         mudar de velocidade em (2); se o arrasto parar no 0 em (4); ou se o -2 nao entrar em (5).\n"
    );
}

#[cfg(test)]
#[path = "motion_state_fio_vortex_demo_tests.rs"]
mod tests;
