//! **PEÇAS QUE NÃO SE ATRAVESSAM** (`PH2D_GPU_COOK_DEMO=114`) — o `motion.collide`
//! **DENTRO de uma simulação a correr**.
//!
//! ## Porque esta cena existe, tendo o nó já duas
//!
//! ⛔ **Nenhuma das duas mostra isto.** A `=48` (grupo H da conferência) é um banco de
//! comparação **PARADO** — o próprio doc dela diz *«esta cena julga-se PARADA»* —, e a `=8` é
//! um brinquedo de desempenho da grelha de vizinhos. As duas provam que o nó **empacota**;
//! nenhuma mostra o que o dono perguntou: *peças a colidirem umas com as outras enquanto a
//! simulação corre*.
//!
//! ⭐ E a composição que ela encena está **medida e sem cena**: a folha 03 da conferência
//! rodou-a com o `motion.verlet_rope` (gate `rope_thickness.rs`) e com o `motion.soft_body`
//! (gate `soft_body_radius.rs`) e concluiu, nos dois, *«não falta capacidade, falta o
//! GESTO»*. Uma capacidade provada por gate e invisível no produto é exactamente o que uma
//! cena serve para fechar.
//!
//! ## O que se vê
//!
//! Duas taças. Em cada uma caem **as mesmas** peças, pela **mesma** lei — e a cadeia difere em
//! **UM nó**:
//!
//! ```text
//!   ESQUERDA   ... -> sim.step ------------------> sim.collide(Bowl)   um BORRÃO no fundo
//!   DIREITA    ... -> sim.step -> motion.collide -> sim.collide(Bowl)  uma PILHA
//! ```
//!
//! ⚠️ **A taça é o que torna a diferença visível.** Num chão plano as peças espalham-se e
//! quase não se sobrepõem sozinhas; uma taça junta-as todas no mesmo ponto baixo, que é onde
//! *não se atravessar* deixa de ser detalhe e passa a ser a imagem inteira.
//!
//! ⚠️ **A ORDEM dentro do laço é uma decisão:** o `motion.collide` corre **antes** do
//! `sim.collide`, para o recipiente ter a última palavra. Ao contrário, uma peça acabada de
//! empurrar por uma vizinha podia sair pela parede da taça.
//!
//! ⚠️ **Ela precisa de Play** — é uma simulação, como a `=99` e a `=113`.

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas peças caem em cada taça: `5 × 5`. Poucas o bastante para se **contar** as da
/// direita, muitas o bastante para as da esquerda serem um borrão.
const COLS: f32 = 5.0;
const ROWS: f32 = 5.0;
/// O vão de partida e o tamanho de cada peça.
const GAP: f32 = 0.26;
const PECA: f32 = 0.22;
/// De que altura elas partem — ⚠️ **DENTRO da taça**, e isso não é enquadramento, é
/// correcção.
///
/// ⛔⛔ A 1.ª redacção largava-as de `y = 1,9`, bem acima da borda, e o anúncio prometia uma
/// chuva a cair. Medido (a mutação que pôs a gravidade a ZERO e **sobreviveu** ao gate de
/// controlo): as peças acabavam todas a `y = 0,250`, que é **exactamente a borda da taça**. Uma
/// taça é um recipiente — o `sim.collide` projecta para a superfície interior tudo o que está
/// fora dela —, então elas não caíam: eram **puxadas para dentro no primeiro tique**, com ou
/// sem gravidade. *A cena mostrava uma queda que não existia.*
const ALTURA: f32 = -0.2;

/// A taça: onde fica o fundo dela e o raio.
///
/// ⚠️ **Ela cresceu com a correcção acima:** o arranjo de partida tem de caber INTEIRO dentro
/// dela, senão a peça de canto nasce fora e é projectada para a borda em vez de cair.
const TACA_Y: f32 = -1.2;
const TACA_R: f32 = 1.8;
/// Quanto as duas metades se afastam — o suficiente para as taças não se tocarem.
const VAO: f32 = 2.0;

/// A gravidade (`force.wind` apontado para baixo, sem rajada — o doc dele diz que assim ele
/// **é** gravidade).
const GRAVIDADE: f32 = 4.0;
const RAJADA: f32 = 0.0;

/// ⭐ **O raio do disco de cada peça, em unidades do TAMANHO dela.**
///
/// A lei do nó é `r_i = radius · max(|size.x|, |size.y|)`, e as peças são desenhadas como um
/// quadrado de lado [`PECA`] ⇒ **`0,5` é exactamente o círculo inscrito**: dois discos tocam-se
/// quando os dois quadrados se encostam. ⛔ Não é um número de gosto — é o que faz a promessa
/// do nó (*«pares acabam apenas a tocar-se»*) coincidir com o que o olho vê desenhado.
const RAIO: f32 = 0.5;

/// Quanto tempo dura cada queda, e a pausa antes da seguinte.
const DURACAO: f32 = 3.0;
const PAUSA: f32 = 0.6;

/// A legenda que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    vec![
        Caption::new([-VAO, TACA_Y - TACA_R - 0.45], "sem Collide: um borrao"),
        Caption::new([VAO, TACA_Y - TACA_R - 0.45], "com Collide: uma pilha"),
    ]
}

/// Monta as duas taças. Devolve os dois sinks.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};

    // ⚠️ O índice de `Bowl` é PERGUNTADO ao registo, nunca digitado — a mesma porta da cena
    // `=113`: um literal que envelhecesse montaria a forma errada sem erro nenhum.
    let taca = super::sim_demo::indice_de(reg, "sim.collide", "shape", "Bowl")?;

    let mut metade = |x: f32, com_collide: bool, y_linha: f32| -> Option<NodeId> {
        let g = &mut doc.graph;
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", ROWS);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", GAP);
        g.set_param(grid, "gap_y", GAP);

        let alto = g.add_node("motion.transform");
        g.set_param(alto, "offset_x", x);
        g.set_param(alto, "offset_y", ALTURA);

        let tamanho = g.add_node("motion.scale");
        g.set_param(tamanho, "amount", PECA);

        let zone = g.add_node("sim.zone");
        // `Loop` para a queda recomeçar sozinha — sem isso o monte assenta uma vez e a cena
        // deixa de ter o que mostrar depois do primeiro olhar.
        g.set_param(
            zone,
            "mode",
            super::sim_demo::indice_de(reg, "sim.zone", "mode", "Loop")?,
        );
        g.set_param(zone, "duration", DURACAO);
        g.set_param(zone, "loop_delay", PAUSA);

        let vento = g.add_node("force.wind");
        g.set_param(vento, "angle", 270.0);
        g.set_param(vento, "strength", GRAVIDADE);
        g.set_param(vento, "gust", RAJADA);

        let passo = g.add_node("sim.step");

        let bowl = g.add_node("sim.collide");
        g.set_param(bowl, "shape", taca);
        g.set_param(bowl, "center_x", x);
        g.set_param(bowl, "center_y", TACA_Y);
        g.set_param(bowl, "radius", TACA_R);
        g.set_param(bowl, "restitution", 0.05);
        g.set_param(bowl, "friction", 0.6);

        // ⭐ O nó da pergunta — só nesta metade.
        let empurra = com_collide.then(|| {
            let c = g.add_node("motion.collide");
            g.set_param(c, "radius", RAIO);
            c
        });

        let out = g.add_node("motion.output");

        let fila: Vec<NodeId> = [grid, alto, tamanho, zone].into_iter().collect();
        for (i, n) in fila.into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                    x: 40.0 + i as f32 * 176.0,
                    y: y_linha,
                },
            );
        }
        let laco: Vec<NodeId> = [Some(vento), Some(passo), empurra, Some(bowl), Some(out)]
            .into_iter()
            .flatten()
            .collect();
        for (i, n) in laco.into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                    x: 260.0 + i as f32 * 176.0,
                    y: y_linha + 250.0,
                },
            );
        }

        // ⚠️ A aresta `zone -> vento` é `delayed`: é a entrada de estado que fecha o laço.
        // ⚠️ E o `motion.collide` entra ENTRE o passo e a taça — ver o cabeçalho.
        let mut arestas: Vec<(NodeId, u16, NodeId, u16, bool)> = vec![
            (grid, 0, alto, 0, false),
            (alto, 0, tamanho, 0, false),
            (tamanho, 0, zone, 0, false),
            (zone, 0, vento, 0, true),
            (vento, 0, passo, 0, false),
        ];
        match empurra {
            Some(c) => {
                arestas.push((passo, 0, c, 0, false));
                arestas.push((c, 0, bowl, 0, false));
            }
            None => arestas.push((passo, 0, bowl, 0, false)),
        }
        arestas.push((bowl, 0, zone, 1, false));
        arestas.push((zone, 0, out, 0, false));

        for (a, ap, b, bp, delayed) in arestas {
            g.connect(Edge {
                from: (a, ap),
                to: (b, bp),
                delayed,
            })
            .ok()?;
        }
        Some(out)
    };

    let esquerda = metade(-VAO, false, 120.0)?;
    let direita = metade(VAO, true, 640.0)?;
    doc.graph.validate(reg).ok()?;
    Some(vec![esquerda, direita])
}

#[cfg(test)]
#[path = "motion_state_pilha_demo_tests.rs"]
mod tests;
