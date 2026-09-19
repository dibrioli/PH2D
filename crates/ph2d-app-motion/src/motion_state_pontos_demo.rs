//! ⭐⭐⭐ **A CENA DAS POSIÇÕES E DA MARCA** (cena `=124`) — a resposta ao report do dono de
//! 2026-09-19: *«Grid está por padrão com 360x360 objetos. deveria ser 20x20»* e *«coloque gizmos
//! de pequenos pontos visíveis para as posições dos nós»*.
//!
//! # Porque ela existe, e porque a `=2` não servia
//!
//! ⛔⛔ **Eu mandei-lhe a cena errada.** A `=2` é o demo de PERFORMANCE do dispositivo — uma
//! `motion.grid` de `360 × 360 = 129 600` elementos, escolhida para provar que o device aguenta —,
//! e eu chamei-lhe *«o cartão do Grid»* num report. Ele abriu-a, contou os objectos e leu o número
//! como sendo o padrão do nó. ⚠️ **O padrão do nó é `3 × 3`**, e o `360` é daquela cena e só dela.
//!
//! ⇒ *baixar a `=2` para `20 × 20` apagaria a razão de ela existir.* Esta cena é o que o report
//! pedia: um grid pequeno, sozinho, onde as posições e o cartão se vêem.
//!
//! # As DUAS metades, lado a lado
//!
//! | fileira | o que tem | o que se vê |
//! |---|---|---|
//! | de cima | `motion.grid` → `motion.output` | **marcas** — cruzes, uma por posição, e mais nada |
//! | de baixo | o MESMO grid → `motion.duplicator` ← `source.shape` (**Bone**) | as **peças** |
//!
//! ⚠️⚠️ **A metade de baixo é o CONTROLO, e sem ela a de cima não ensina nada:** *«não desenha»* e
//! *«está partido»* têm exactamente o mesmo aspecto no ecrã, e o que os separa é ver a mesma nuvem
//! de posições a virar coisas assim que uma forma chega.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

use ph2d_motion_doc::MotionDoc;

/// ⭐ **`20 × 20`, o número que o dono pediu** — e ele é confortável nas duas pontas: `400`
/// posições cabem com folga no tecto do gizmo (`ph2d_app_motion::ponto_gizmo::MAX_PONTOS`, que é
/// `4 096` e foi MEDIDO contra o orçamento de um décimo de quadro), logo a amostra **não engata** e
/// o que se vê é a grelha inteira, posição a posição.
pub(super) const LADO: f32 = 20.0;

/// O vão entre posições. ⚠️ **Maior que o `1,0` de fábrica de propósito:** a marca é chrome e
/// mede-se em píxeis de ECRÃ, logo com a grelha muito apertada as cruzes encostam-se umas às
/// outras e a nuvem lê-se como uma mancha. Aqui elas ficam separadas em qualquer zoom razoável.
const VAO: f32 = 1.6;

/// O tamanho da peça na fileira de baixo — a mesma ordem de grandeza do vão, para as cópias
/// quase se tocarem e a fileira ler-se como uma superfície.
const TAMANHO: f32 = 1.4;

/// Quanto a fileira de baixo desce. ⚠️ Derivado do próprio grid (`LADO × VAO`) mais uma folga,
/// senão as duas fileiras sobrepõem-se no dia em que alguém mexer no vão.
const DESCIDA: f32 = -(LADO * VAO + 4.0 * VAO);

/// Constrói o documento. `None` se algum tipo de nó não estiver registado.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    // ⭐ **O OSSO e não um círculo** — ordem do dono (2026-09-19): *«no caso dos ossos e
    // segmentos de corda, criaremos no nó shape formas similares para isso»*. A fileira de baixo
    // é a mesma nuvem da de cima, vestida com a forma que substituiu o gizmo retirado ⇒ *a cena
    // mostra as duas metades da ordem dele de uma vez*: em cima as posições nuas, em baixo o que
    // um `Duplicator` faz com elas quando há uma forma.
    let osso = super::sim_demo::indice_de(reg, "source.shape", "kind", "Bone")?;
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };
    let grade = |g: &mut ph2d_nodegraph::graph::Graph, n: NodeId, dy: f32| {
        g.set_param(n, "rows", LADO);
        g.set_param(n, "cols", LADO);
        g.set_param(n, "gap_x", VAO);
        g.set_param(n, "gap_y", VAO);
        // O deslocamento vertical da fileira, autorado no nó que o produz.
        if dy != 0.0 {
            let m = g.add_node("motion.move".to_string());
            g.set_pos(m, Pos { x: 200.0, y: 260.0 });
            g.set_param(m, "dy", dy);
            let _ = g.connect(Edge {
                from: (n, 0),
                to: (m, 0),
                delayed: false,
            });
            return m;
        }
        n
    };

    // ── A fileira DE CIMA: só posições.
    let so_posicoes = no(g, "motion.grid", 0.0, 0.0);
    let cabeca = grade(g, so_posicoes, 0.0);
    let saida_a = no(g, "motion.output", 420.0, 0.0);
    g.connect(Edge {
        from: (cabeca, 0),
        to: (saida_a, 0),
        delayed: false,
    })
    .ok()?;

    // ── A fileira DE BAIXO: o MESMO grid, vestido.
    let com_forma = no(g, "motion.grid", 0.0, 260.0);
    let corpo = grade(g, com_forma, DESCIDA);
    let forma = no(g, "source.shape", 0.0, 380.0);
    g.set_param(forma, ph2d_node_motion_shape::param::KIND, osso);
    g.set_param(forma, ph2d_node_motion_shape::param::SIZE, TAMANHO);
    let dup = no(g, "motion.duplicator", 220.0, 320.0);
    let saida_b = no(g, "motion.output", 420.0, 320.0);
    // ⚠️ A forma na porta `0`, os pontos na `1` — a ordem que o manifesto do duplicador declara.
    for (de, porta) in [(forma, 0u16), (corpo, 1)] {
        g.connect(Edge {
            from: (de, 0),
            to: (dup, porta),
            delayed: false,
        })
        .ok()?;
    }
    g.connect(Edge {
        from: (dup, 0),
        to: (saida_b, 0),
        delayed: false,
    })
    .ok()?;
    Some(vec![saida_a, saida_b])
}

/// O roteiro que o dono segue. ⚠️ **Cada passo nomeia o que aparece NA TELA** (§0.8).
pub(super) fn announce() {
    let n = (LADO * LADO) as u32;
    eprintln!(
        "\n[pontos] DUAS fileiras do MESMO grid de {LADO:.0}x{LADO:.0} ({n} posicoes cada).\n\
         \n\
         EM CIMA  = so' posicoes: o grid vai direito ao Output. Nao ha' forma nenhuma.\n\
         EM BAIXO = as MESMAS posicoes com uma forma, por um Duplicator.\n\
         \n\
         (1) Olhe a fileira DE CIMA: sao CRUZINHAS, uma por posicao — nao ha' quadrado nenhum.\n    \
         Elas sao do EDITOR: nao entram no que o app entrega.\n\
         (2) Olhe a de BAIXO: as mesmas posicoes, agora com OSSOS. E' o que um Duplicator faz.\n\
         (3) Clique no cartao `Grid` de cima. Ele tem um (!) no canto — carregue nele e leia.\n\
         (4) No mesmo cartao, mexa em `Gap X` / `Gap Y`: as cruzes AFASTAM-SE, e o centro da\n    \
         nuvem fica parado. (Era isto que estava quebrado no report do `gap y`.)\n\
         (5) Carregue no cartao `Duplicator` de baixo e no `Shape`: sao eles que fazem pixels.\n\
         \n\
         DEU ERRADO se: a fileira de cima tiver QUADRADOS em vez de cruzes; se as cruzes nao\n    \
         aparecerem de todo; ou se mexer no `Gap` fizer a nuvem VIAJAR em vez de espacar.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_pontos_demo_tests.rs"]
mod tests;
