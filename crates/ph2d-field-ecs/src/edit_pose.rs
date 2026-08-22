//! ⭐ **A POSE, escrita em MUNDO** — a regra-mãe do vetorial aplicada ao campo.
//!
//! ⚠️ *O que se vê e se aponta é **MUNDO**; o que o documento guarda é **LOCAL**.* O gizmo desenha e
//! agarra em mundo; o nó guarda a pose relativa ao pai. Escrever o deslocamento de mundo direto na
//! translação local funciona **exatamente** enquanto nenhum pai tiver rotação ou escala — o caso da
//! primeira cena de smoke e de nenhuma peça real. É essa conversão que estas funções existem para
//! fazer, num sítio só.

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use ph2d_field::xform::{quat_axis_angle, quat_conj, quat_mul, quat_normalize, quat_rotate};
use ph2d_field::{NodeShape, Xform};

use crate::{FieldNode, FieldPose};

/// ⭐ **Move um nó por um deslocamento de MUNDO**, escrevendo na pose **local** dele.
///
/// ⚠️ A conversão é a razão de esta função existir. O gizmo desenha e agarra em mundo — é a
/// regra-mãe do vetorial (*o que se vê e se aponta é MUNDO; o que o documento guarda é LOCAL*) — e
/// o que o nó guarda é a pose relativa ao pai. Somar o deslocamento de mundo direto na translação
/// local funcionaria **exatamente** enquanto nenhum pai tivesse rotação ou escala, que é o caso da
/// primeira cena de smoke e de nenhuma peça real.
///
/// A inversa de uma pose com rotação e escala uniforme é fechada: desfaz-se a rotação com o
/// conjugado e divide-se pela escala.
///
/// No-op silencioso se a entidade não tem pose — mover o que não tem onde guardar a posição não é
/// um erro a reportar, é um gesto sem alvo.
pub fn translate_world(world: &mut World, entity: Entity, delta: [f32; 3]) {
    if !delta.iter().all(|v| v.is_finite()) {
        return;
    }
    let parent = world
        .get::<bevy_ecs::hierarchy::ChildOf>(entity)
        .map(|c| c.0);
    let outer = parent.map_or(Xform::IDENTITY, |p| crate::world_xform(world, p));
    let inv_rot = [
        -outer.rotation[0],
        -outer.rotation[1],
        -outer.rotation[2],
        outer.rotation[3],
    ];
    let s = if outer.scale.abs() > f32::MIN_POSITIVE {
        outer.scale
    } else {
        1.0
    };
    let local = quat_rotate(inv_rot, [delta[0] / s, delta[1] / s, delta[2] / s]);
    if let Some(mut pose) = world.get_mut::<FieldPose>(entity) {
        for (t, d) in pose.xform.translation.iter_mut().zip(local) {
            *t += d;
        }
    }
}

/// ⭐ **Roda um nó em torno de um eixo do MUNDO**, pelo **próprio centro dele**.
///
/// ⚠️ O pivô é a origem do nó de propósito: é onde o gizmo desenha as argolas, e é a única escolha
/// que faz a peça girar debaixo do cursor em vez de descrever um arco à volta de outra coisa. Um
/// pivô diferente (o centro da seleção, o cursor 3D do Blender) é **produto**, e entra com a UI que
/// o escolhe — não por omissão.
///
/// A conta é a conjugação: `R_mundo = R_pai ⊗ R_local`, e querendo `R_mundo' = Q ⊗ R_mundo` sai
/// `R_local' = inv(R_pai) ⊗ Q ⊗ R_pai ⊗ R_local`. Sem o sanduíche, um giro em torno do X do mundo
/// aplicado a um filho de pai rodado giraria em torno do X **do pai** — o eixo errado, e ninguém
/// diria que o culpado é o gizmo.
///
/// No-op silencioso sem pose ou com ângulo não-finito.
pub fn rotate_world(world: &mut World, entity: Entity, axis: [f32; 3], angle: f32) {
    if !angle.is_finite() || angle == 0.0 {
        return;
    }
    let parent = world
        .get::<bevy_ecs::hierarchy::ChildOf>(entity)
        .map(|c| c.0);
    let outer = parent.map_or(Xform::IDENTITY, |p| crate::world_xform(world, p));
    let q = quat_axis_angle(axis, angle);
    let sandwich = quat_mul(quat_mul(quat_conj(outer.rotation), q), outer.rotation);
    if let Some(mut pose) = world.get_mut::<FieldPose>(entity) {
        pose.xform.rotation = quat_normalize(quat_mul(sandwich, pose.xform.rotation));
    }
}

/// ⭐ **Roda um nó em torno de um PIVÔ que não é o dele** — o que uma seleção de vários exige.
///
/// ⚠️ **Ela CONTÉM a lei antiga em vez de a duplicar**, e essa é a propriedade que a torna segura:
/// com o pivô em cima da origem do nó, a translação sai exactamente zero e o resultado é
/// byte-a-byte o de [`rotate_world`]. Não há um caso especial para "um só nó" — há uma lei mais
/// geral cujo caso particular é o antigo.
///
/// ⚠️ **Orbitar é TRANSLADAR**, e é por isso que isto se escreve com as duas portas que já existem:
/// a orientação por [`rotate_world`], a posição por [`translate_world`]. Uma terceira conta de pose
/// aqui divergiria das outras duas no dia em que a hierarquia mudasse de forma.
pub fn rotate_world_about(
    world: &mut World,
    entity: Entity,
    axis: [f32; 3],
    angle: f32,
    pivot: [f32; 3],
) {
    if !angle.is_finite() || angle == 0.0 || !pivot.iter().all(|v| v.is_finite()) {
        return;
    }
    let before = crate::world_xform(world, entity).translation;
    rotate_world(world, entity, axis, angle);
    let arm = [
        before[0] - pivot[0],
        before[1] - pivot[1],
        before[2] - pivot[2],
    ];
    let spun = quat_rotate(quat_axis_angle(axis, angle), arm);
    translate_world(
        world,
        entity,
        [
            pivot[0] + spun[0] - before[0],
            pivot[1] + spun[1] - before[1],
            pivot[2] + spun[2] - before[2],
        ],
    );
}

/// ⭐ **Escala um nó em torno de um PIVÔ que não é o dele** — a irmã de [`rotate_world_about`], com
/// a mesma propriedade: pivô na origem do nó ⇒ byte-a-byte o [`scale_by`] de sempre.
pub fn scale_about(world: &mut World, entity: Entity, factor: f32, pivot: [f32; 3]) {
    if !factor.is_finite() || factor <= 0.0 || !pivot.iter().all(|v| v.is_finite()) {
        return;
    }
    let before = crate::world_xform(world, entity).translation;
    scale_by(world, entity, factor);
    translate_world(
        world,
        entity,
        [
            (before[0] - pivot[0]) * (factor - 1.0),
            (before[1] - pivot[1]) * (factor - 1.0),
            (before[2] - pivot[2]) * (factor - 1.0),
        ],
    );
}

/// ⭐ **Quem, de uma seleção, é o TOPO do seu ramo** — a lista que um gesto pode mover sem aplicar
/// duas vezes ao mesmo objeto.
///
/// ⚠️ **É o defeito clássico de mover uma seleção**: com um pai e um filho ambos escolhidos, o
/// filho recebe o gesto **e** herda o do pai pela hierarquia — ele anda o dobro, e só ele. Um
/// artista que escolhe um grupo e uma peça dentro dele não está a pedir isso.
///
/// A ordem da entrada é preservada — quem chama depende dela para saber quem é o principal.
#[must_use]
pub fn top_level(world: &World, selection: &[Entity]) -> Vec<Entity> {
    selection
        .iter()
        .copied()
        .filter(|e| {
            let mut up = world.get::<ChildOf>(*e).map(|c| c.0);
            while let Some(p) = up {
                if selection.contains(&p) {
                    return false;
                }
                up = world.get::<ChildOf>(p).map(|c| c.0);
            }
            true
        })
        .collect()
}

/// ⭐ **Escala um nó por um fator UNIFORME.**
///
/// ⛔ Uniforme porque o documento é uniforme ([ADR-0161 §6]): escala não-uniforme destrói a
/// propriedade de distância de que o módulo inteiro depende. Não há aqui uma função por eixo à
/// espera de ser escrita — há uma decisão medida.
///
/// ⚠️ Um fator não-positivo ou não-finito é **recusado em silêncio** e não aplicado pela metade: a
/// invariante é *um nó que existe está válido*, e uma escala nula faria o campo deixar de ser uma
/// distância.
///
/// [ADR-0161 §6]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md
pub fn scale_by(world: &mut World, entity: Entity, factor: f32) {
    if !factor.is_finite() || factor <= 0.0 {
        return;
    }
    // ⭐ **Numa FOLHA, crescer é crescer as DIMENSÕES** — e não o fator da pose. As duas dão a mesma
    // forma, mas só uma delas é o número que o painel mostra: escalar a pose deixaria o artista com
    // uma caixa que mede 2 na tela e diz «1» no painel. Ver `ph2d_field::scale_primitive`.
    if let Some(mut node) = world.get_mut::<FieldNode>(entity)
        && let NodeShape::Leaf(p) = &mut node.shape
    {
        if ph2d_field::scale_primitive(p, factor) {
            ph2d_field::clamp_round(p);
        }
        return;
    }
    if let Some(mut pose) = world.get_mut::<FieldPose>(entity) {
        let next = pose.xform.scale * factor;
        if next.is_finite() && next > 0.0 {
            pose.xform.scale = next;
        }
    }
}
