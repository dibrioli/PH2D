//! **Quem são as molduras, e como se chamam** — a lista que a shell publica para o
//! [`ph2d_editor::frame_label`] desenhar.
//!
//! Irmão do [`crate::vec_frame_spans`] (que responde *que INTERVALO da pilha de z cada moldura
//! ocupa*), e as duas perguntas partilham o mesmo sujeito de propósito: uma moldura é uma entidade
//! com `VecFrame`, e é o `VecPathRef` dela que diz qual caminho desenha a silhueta.
//!
//! ⚠️ **O nome vem do `Name` da entidade** — o mesmo que a Hierarquia mostra e que o `wire_id` da
//! timeline resolve. Um rótulo próprio seria um segundo nome para o mesmo objeto, e o dia em que
//! um deles fosse renomeado a tela e a árvore discordariam.

use ph2d_ecs::{Entity, Name, SimWorld, VecFrame};
use ph2d_editor::frame_label::FrameLabel;
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};

use crate::vec_entities::VecEntityMap;

/// As etiquetas de todas as molduras da cena, **em ordem de z**.
///
/// A varredura é a da CENA (e não uma query do ECS) por dois motivos que se somam: ela é
/// só-leitura sobre o mundo, e a ordem sai estável de graça — uma query itera por arquétipo, e uma
/// lista que se reordena entre frames não muda o desenho mas torna todo gate de ordem uma moeda.
///
/// O canto é o **topo-esquerdo em MUNDO** (`min_x`, `max_y`) da geometria já transformada — a
/// mesma `path_world_curve_bbox` que o alinhamento e os campos X/Y/W/H do painel usam. Derivar a
/// caixa aqui por conta própria daria uma etiqueta que flutua ao lado da moldura assim que ela
/// ganha uma pose.
#[must_use]
pub(crate) fn frame_labels(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    selected: &[VecPathId],
) -> Vec<FrameLabel> {
    let w = sim.world();
    let mut out = Vec::new();
    for path in scene.paths() {
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        if w.get_entity(e).is_err() || w.get::<VecFrame>(e).is_none() {
            continue;
        }
        let Some((lo, hi)) = scene.path_world_curve_bbox(xforms, path.id) else {
            continue;
        };
        out.push(FrameLabel {
            world_top_left: [lo[0], hi[1]],
            // ⚠️ Sem `Name` a etiqueta ainda existe: ela serve para dizer *"isto é uma moldura"*,
            // e escondê-la justamente na que o artista não reconhece seria o contrário do pedido.
            name: w
                .get::<Name>(e)
                .map_or_else(|| String::from("Frame"), |n| n.as_str().to_string()),
            selected: selected.contains(&path.id),
        });
    }
    out
}

#[cfg(test)]
#[path = "vec_frame_labels_tests.rs"]
mod tests;
