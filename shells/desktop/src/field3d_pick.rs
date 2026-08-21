//! ⭐ **Que objeto está sob este pixel?** — a seleção por clique na janela 3D.
//!
//! # Como se pergunta a um CAMPO quem ele é
//!
//! Uma malha traz consigo a resposta (cada triângulo sabe de que objeto é). Um campo implícito não:
//! o que se avalia é **um número**, e o número da peça inteira já é a união de todos os nós. Então a
//! pergunta é feita em dois passos, e nenhum deles é uma estrutura nova:
//!
//! 1. **Onde** a superfície está sob o cursor — uma marcha de **um** raio, pela mesma função que
//!    desenha o quadro ([`ph2d_field_render::surface_under`]);
//! 2. **De quem** é aquele ponto — pergunta-se a cada folha o valor do campo *dela* ali, e ganha a
//!    de menor módulo. Numa superfície de união o vencedor vale ~0 e os outros valem a distância a
//!    que estão, então a resposta não é apertada: ela é a diferença entre tocar e não tocar.
//!
//! ⛔ **A alternativa recusada** era dar um identificador a cada nó e fazer a marcha devolvê-lo
//! junto com a distância — o *id-buffer* que um renderizador de malha usa. Ela obrigaria a árvore de
//! avaliação a carregar um segundo canal por todo o operador (`min` de dois números passaria a ser
//! `min` de dois pares), que é o custo espalhado por **cada pixel de cada quadro** para responder a
//! uma pergunta que só se faz **num clique**. O preço aqui é uma compilação por folha, uma vez, e
//! ele está medido ([`measure_pick_cost`](#) e o doc 07).

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, NodeShape};
use ph2d_field_ecs::FieldNode;
use ph2d_field_render::{Orbit, Screen, surface_under};

/// ⭐ **O nó sob o pixel** — `None` quando o clique caiu no fundo.
///
/// Devolve sempre uma **folha** (um cilindro, uma caixa), nunca a operação que a contém: é o objeto
/// que o artista apontou. Quem quer o grupo inteiro clica nele na Hierarquia, que é onde um grupo
/// tem nome e linha própria.
pub(crate) fn node_under(
    world: &World,
    root: Entity,
    doc: &FieldDoc,
    cam: &Orbit,
    screen: Screen,
    px: [f32; 2],
) -> Option<Entity> {
    let p = surface_under(doc, cam, screen, px)?;
    let mut best: Option<(f32, Entity)> = None;
    for (e, _) in ph2d_field_ecs::walk(world, root) {
        let Some(FieldNode {
            shape: NodeShape::Leaf(prim),
        }) = world.get::<FieldNode>(e)
        else {
            continue;
        };
        // ⚠️ A pose de MUNDO, e não a local: o ponto veio do mundo. Uma folha avaliada com a pose
        // local responderia sobre um sítio onde ela não está — e o erro cresce com o aninhamento,
        // então ele passaria despercebido numa peça plana e escolheria o objeto errado numa peça
        // agrupada.
        let placed = FieldDoc::new(
            vec![Node {
                xform: ph2d_field_ecs::world_xform(world, e),
                kind: NodeKind::Leaf(prim.clone()),
                mods: Vec::new(),
            }],
            NodeId(0),
        );
        let Ok(placed) = placed else {
            continue;
        };
        let v = ph2d_field_eval::Field::new(&placed)
            .at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
            .abs() as f32;
        if best.is_none_or(|(b, _)| v < b) {
            best = Some((v, e));
        }
    }
    best.map(|(_, e)| e)
}

#[cfg(test)]
#[path = "field3d_pick_tests.rs"]
mod tests;
