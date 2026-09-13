//! **A recolha dos filhos de uma moldura que flui** — irmão de `layout_live.rs` pelo tecto de 600
//! LOC da shell (HR-18): a função saiu do `LayoutLive::lay_out` pelo tecto de 200 por função, e
//! como função-irmã no mesmo ficheiro empurrava-o para 604.
//!
//! Corte mecânico: o corpo é o de lá, verbatim; abriu-se só a visibilidade para o módulo pai.

use super::*;

/// **Os filhos, em largura**, na ordem da hierarquia: um nó do motor e um `Collected` por filho
/// que entra no fluxo da moldura.
///
/// ⚠️ Saiu do [`LayoutLive::lay_out`] pelo tecto de 200 LOC por função (`fn_loc_caps`), verbatim;
/// corre no mesmo sítio, depois da raiz e antes do motor.
#[allow(clippy::too_many_arguments)]
pub(super) fn collect_flow_children(
    sim: &SimWorld,
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    frame: Entity,
    tok: crate::vec_bindings::TokenCtx,
    nodes: &mut Vec<Node>,
    collected: &mut Vec<Collected>,
) {
    let w = sim.world();
    // Os filhos, em largura, na ordem da HIERARQUIA — é ela que o artista vê e reordena.
    let mut queue: Vec<(Entity, usize)> = vec![(frame, 0)];
    while let Some((parent, parent_idx)) = queue.pop() {
        let Some(kids) = w.get::<ph2d_ecs::Children>(parent) else {
            continue;
        };
        for &kid in kids.iter() {
            // ⚠️ **O fora-do-fluxo sai da FATIA, e não do motor** — o *Absolute position* do
            // Figma. Um nó que o motor nunca vê fica exactamente com a pose que o artista lhe
            // deu, e continua a andar com o pai e a ser recortado por ele (ele não deixou de
            // ser filho na hierarquia). Dizê-lo ao motor em vez disto pediria um inset — quatro
            // números derivados que ninguém autorou, para reproduzir a posição que já existe.
            if w.get::<VecLayoutAbsolute>(kid).is_some() {
                continue;
            }
            let flows_here = w.get::<VecLayout>(kid).is_some();
            // ⚠️ **Uma moldura que FLUI mede-se e move-se por SI, nunca pela sub-árvore.**
            //
            // Os filhos dela viram nós próprios, com transformação própria; incluí-los aqui
            // aplicaria a deles DUAS vezes — a do pai a mover a sub-árvore inteira, e a
            // própria — e a cada frame os netos fugiriam mais para fora da moldura. Um nó que
            // NÃO flui é o oposto: nada lá dentro é nó, então ele carrega a sub-árvore toda.
            let paths = if flows_here {
                own_paths(sim, scene, kid).unwrap_or_default()
            } else {
                ph2d_vec_entities::entities::subtree_paths(sim, scene, kid)
            };
            if paths.is_empty() {
                continue;
            }
            let items = world_of_all(scene, xforms, live, &paths);
            let Some(bbox) = bbox_of(&items) else {
                continue;
            };
            let measured = [bbox.1[0] - bbox.0[0], bbox.1[1] - bbox.0[1]];
            let (size, min, max) = size_of(w.get::<VecLayoutSize>(kid), flows_here, measured);
            nodes.push(Node {
                parent: Some(parent_idx),
                frame: w
                    .get::<VecLayout>(kid)
                    .map(|l| frame_style(l, crate::vec_bindings::bound_gap(sim, kid, tok))),
                item: item_style(w.get::<VecLayoutItem>(kid)),
                size,
                min,
                max,
            });
            let idx = nodes.len() - 1;
            collected.push(Collected {
                paths,
                bbox,
                who: Some((kid, parent)),
            });
            if flows_here {
                queue.push((kid, idx));
            }
        }
    }
}
