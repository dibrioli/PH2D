//! **ABRIR ESPAÇO PARA UM NÓ QUE ENTROU NUM FIO** — irmão do [`super`] (`motion_bridge::rewire`)
//! pelo tecto de LOC e por RESPONSABILIDADE. Ordem do dono (2026-09-19, depois do smoke): *«se o
//! espaço onde o nó entrou for muito apertado, crie espaço»*.

use super::{MotionState, NodeId};
use ph2d_nodegraph::graph::{Graph, Pos};

/// ⭐⭐⭐ **ABRE ESPAÇO PARA O NÓ QUE ACABOU DE ENTRAR NUM FIO** — ordem do dono (2026-09-19,
/// depois do smoke): *«se o espaço onde o nó entrou for muito apertado, crie espaço»*.
///
/// É o *auto-offset* que o Blender faz ao inserir um nó numa ligação: **os vizinhos é que se
/// afastam**, e afasta-se a jusante inteira — não só o `v` —, senão o `v` empurrado passa a
/// sobrepor-se a quem ele alimenta e o problema anda uma carta para a frente.
///
/// ⚠️⚠️ **O «apertado» é MEDIDO e não escolhido:** o vão de que três cartas em fila precisam é
/// **dois passos de coluna** ([`ph2d_nodegraph::layout::DX`], `u → nó` e `nó → v`), e esse é o
/// mesmo número que o *Arrange* do painel usa — ele é `> CARD_W` por declaração, logo duas colunas
/// nunca se tocam. *Um número escolhido aqui seria uma terceira resposta a «quanto mede uma
/// coluna», e a que o artista vê é sempre a que envelhece.*
///
/// ⛔ **Só quando é preciso.** Com espaço a sobrar nada se mexe: o artista pousou a carta onde
/// quis, e uma arrumação que corre sempre é uma que lhe tira o desenho das mãos.
///
/// ⚠️ **E o nó entra na coluna do MEIO, mantendo o `y` que a mão escolheu.** Sem isso, abrir o vão
/// deixa a carta onde ela caiu — que, num vão apertado, é **em cima do `u`**: o espaço estaria
/// aberto e a sobreposição continuava lá, que é o defeito que o dono reportou.
pub(super) fn abre_espaco(motion: &mut MotionState, u: NodeId, node: NodeId, v: NodeId) {
    use ph2d_nodegraph::layout::DX;
    let (Some(pu), Some(pv)) = (motion.doc.graph.pos(u), motion.doc.graph.pos(v)) else {
        return;
    };
    let preciso = 2.0 * DX;
    let vao = pv.x - pu.x;
    if vao >= preciso {
        return;
    }
    let empurrao = preciso - vao;
    for n in jusante(&motion.doc.graph, v) {
        if let Some(p) = motion.doc.graph.pos(n) {
            motion.doc.graph.set_pos(
                n,
                Pos {
                    x: p.x + empurrao,
                    y: p.y,
                },
            );
        }
    }
    if let Some(pn) = motion.doc.graph.pos(node) {
        motion.doc.graph.set_pos(
            node,
            Pos {
                x: pu.x + DX,
                y: pn.y,
            },
        );
    }
}

/// Todo nó alcançável a partir de `raiz` pelos fios ORDINÁRIOS, a raiz incluída.
///
/// ⚠️ **Os `delayed` ficam de fora**: eles são o `pre` que o motor mantém e fecham ciclo por
/// construção — segui-los arrastaria metade do grafo atrás de uma realimentação.
fn jusante(g: &Graph, raiz: NodeId) -> std::collections::BTreeSet<NodeId> {
    let mut vistos = std::collections::BTreeSet::new();
    let mut fila = vec![raiz];
    while let Some(n) = fila.pop() {
        if !vistos.insert(n) {
            continue;
        }
        for e in g.edges() {
            if !e.delayed && e.from.0 == n && !vistos.contains(&e.to.0) {
                fila.push(e.to.0);
            }
        }
    }
    vistos
}
