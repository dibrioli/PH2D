//! **O que se edita num nó da cena**, e as perguntas que o painel faz sobre ele.
//!
//! ⚠️ Nenhuma regra é inventada aqui: o raio muda por [`ph2d_field::set_shape_radius`] e o teto sai
//! de [`ph2d_field::round_limit`] / [`ph2d_field::characteristic_size`] — as **mesmas** funções que
//! a validação do documento cozido usa. Um painel que calculasse o próprio teto ofereceria valores
//! que a peça recusa, e o artista veria o controle parar sem explicação.

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::world::World;
use ph2d_field::xform::{quat_axis_angle, quat_conj, quat_mul, quat_normalize, quat_rotate};
use ph2d_field::{
    FieldError, NodeShape, RadiusBound, Xform, characteristic_size, round_limit, set_shape_radius,
};

use crate::{FieldNode, FieldPose};

/// **A árvore em pré-ordem**, com a profundidade de cada nó — a mesma ordem e o mesmo aninhamento
/// que a Hierarquia mostra.
///
/// ⭐ É a ordem certa para o painel: uma lista que discordasse da Hierarquia obrigaria o artista a
/// manter dois mapas na cabeça da mesma peça.
#[must_use]
pub fn walk(world: &World, root: Entity) -> Vec<(Entity, u8)> {
    let mut out = Vec::new();
    let mut stack = vec![(root, 0u8)];
    while let Some((e, depth)) = stack.pop() {
        if world.get::<FieldNode>(e).is_none() {
            continue;
        }
        out.push((e, depth));
        if let Some(children) = world.get::<Children>(e) {
            // Invertido para o `pop` sair na ordem de `Children`.
            for c in children
                .iter()
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                stack.push((c, depth.saturating_add(1)));
            }
        }
    }
    out
}

/// O raio editável deste nó, ou `None` quando ele não tem nenhum.
#[must_use]
pub fn radius_of(world: &World, entity: Entity) -> Option<f32> {
    world.get::<FieldNode>(entity)?.shape.radius()
}

/// Até onde esse raio pode ir, e **de que natureza é o limite**.
///
/// - Numa **primitiva** é uma parede ([`RadiusBound::Hard`]): acima dela a forma deixa de existir e
///   o campo deixa de ser uma distância.
/// - Numa **operação** não há limite de validade nenhum ([`RadiusBound::Soft`]) — o campo continua
///   correto com qualquer raio. O que existe é *escala*: um filete maior do que a menor peça que ele
///   junta engole-a. O número vem daí.
#[must_use]
pub fn radius_bound(world: &World, entity: Entity) -> Option<RadiusBound> {
    match &world.get::<FieldNode>(entity)?.shape {
        NodeShape::Leaf(p) => round_limit(p).map(RadiusBound::Hard),
        NodeShape::Combine(_) => Some(RadiusBound::Soft(subtree_scale(world, entity))),
    }
}

/// A menor peça sob um nó, **com a escala da cadeia acumulada**.
///
/// ⚠️ A escala acumula de propósito: um cilindro de 0,1 dentro de um grupo escalado 3× mede 0,3 na
/// peça, e é esse o número que dá sentido a um raio de mistura. Usar só a escala do próprio nó
/// (como a versão de arena fazia, onde não havia cadeia) subestimaria a peça em cada nível de
/// agrupamento.
fn subtree_scale(world: &World, root: Entity) -> f32 {
    let mut best = f32::INFINITY;
    let mut stack = vec![(root, 1.0f32)];
    while let Some((e, acc)) = stack.pop() {
        let Some(node) = world.get::<FieldNode>(e) else {
            continue;
        };
        let acc = acc * world.get::<FieldPose>(e).map_or(1.0, |p| p.xform.scale);
        match &node.shape {
            NodeShape::Leaf(p) => best = best.min(characteristic_size(p) * acc),
            NodeShape::Combine(_) => {
                if let Some(children) = world.get::<Children>(e) {
                    for c in children.iter().copied().collect::<Vec<_>>() {
                        stack.push((c, acc));
                    }
                }
            }
        }
    }
    if best.is_finite() && best > 0.0 {
        best
    } else {
        1.0
    }
}

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
    if let Some(mut pose) = world.get_mut::<FieldPose>(entity) {
        let next = pose.xform.scale * factor;
        if next.is_finite() && next > 0.0 {
            pose.xform.scale = next;
        }
    }
}

/// **Muda o raio de um nó da cena**, ou recusa — e uma recusa deixa o nó **como estava**.
///
/// ⚠️ É a única porta. A invariante do módulo é *uma peça que existe está válida*, e um `set` que a
/// quebrasse produziria a forma errada em silêncio em vez de um erro.
///
/// # Errors
/// Ver [`ph2d_field::set_shape_radius`]. [`FieldError::BadRoot`] se a entidade não é um nó.
pub fn set_radius(world: &mut World, entity: Entity, radius: f32) -> Result<(), FieldError> {
    let Some(mut node) = world.get_mut::<FieldNode>(entity) else {
        return Err(FieldError::BadRoot);
    };
    let mut shape = node.shape.clone();
    // O índice na mensagem vem da entidade: não há arena aqui, e um número que identifique o nó
    // vale mais do que um zero constante.
    set_shape_radius(&mut shape, entity.to_bits() as u32, radius)?;
    node.shape = shape;
    Ok(())
}
