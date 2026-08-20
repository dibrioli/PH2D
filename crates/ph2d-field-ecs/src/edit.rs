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
    Bound, FieldError, NodeShape, Op, Primitive, Xform, characteristic_size, round_limit,
    set_shape_radius,
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
/// - Numa **primitiva** é uma parede ([`Bound::Hard`]): acima dela a forma deixa de existir e
///   o campo deixa de ser uma distância.
/// - Numa **operação** não há limite de validade nenhum ([`Bound::Soft`]) — o campo continua
///   correto com qualquer raio. O que existe é *escala*: um filete maior do que a menor peça que ele
///   junta engole-a. O número vem daí.
#[must_use]
pub fn radius_bound(world: &World, entity: Entity) -> Option<Bound> {
    match &world.get::<FieldNode>(entity)?.shape {
        NodeShape::Leaf(p) => round_limit(p).map(Bound::Hard),
        NodeShape::Combine(_) => Some(Bound::Soft(subtree_scale(world, entity))),
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

/// ⭐ **O que este nó mede** — vazio para uma operação, que não tem forma própria.
#[must_use]
pub fn dims_of(world: &World, entity: Entity) -> Vec<ph2d_field::Dim> {
    match &world.get::<FieldNode>(entity).map(|n| &n.shape) {
        Some(NodeShape::Leaf(p)) => ph2d_field::dims(p),
        _ => Vec::new(),
    }
}

/// ⭐ **Escreve uma dimensão de um nó**, ou recusa — e uma recusa deixa-o **como estava**.
///
/// ⚠️ **Encolher uma forma encolhe o filete dela**, em silêncio mas à vista (ver
/// [`ph2d_field::set_dim`]). Aqui isso é feito **depois** da escrita e **antes** de devolver: um
/// filete que ficasse por limitar deixaria o nó inválido, e a invariante do módulo é *um nó que
/// existe está válido*.
///
/// ⚠️ Numa **operação** o índice 0 é o raio da mistura — é a única dimensão que ela tem, e é por
/// aqui que o painel a edita, com a mesma porta das outras.
///
/// # Errors
/// Ver [`ph2d_field::set_dim`]. [`FieldError::BadRoot`] se a entidade não é um nó.
pub fn set_dim(
    world: &mut World,
    entity: Entity,
    index: usize,
    value: f32,
) -> Result<(), FieldError> {
    let Some(mut node) = world.get_mut::<FieldNode>(entity) else {
        return Err(FieldError::BadRoot);
    };
    let id = entity.to_bits() as u32;
    match &mut node.shape {
        // Uma operação tem uma dimensão só: o raio da mistura.
        NodeShape::Combine(_) if index == 0 => {
            let mut shape = node.shape.clone();
            set_shape_radius(&mut shape, id, value)?;
            node.shape = shape;
            Ok(())
        }
        NodeShape::Combine(_) => Err(FieldError::BadRoot),
        NodeShape::Leaf(p) => {
            let previous = p.clone();
            match ph2d_field::set_dim(p, id, index, value) {
                Ok(()) => {
                    ph2d_field::clamp_round(p);
                    Ok(())
                }
                Err(e) => {
                    *p = previous;
                    Err(e)
                }
            }
        }
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

/// ⭐ **Acrescenta uma forma à peça** e devolve a entidade dela.
///
/// `parent` é onde ela entra — uma operação, ou a raiz. `world_pos` é onde ela nasce, no **mundo**;
/// a pose guardada é a **local**, convertida pela cadeia do pai (a mesma conversão do
/// [`translate_world`], e pelo mesmo motivo).
///
/// ⚠️ **O nome é único entre irmãos**, e isso não é cosmética: a Hierarquia é a única superfície em
/// que estes objetos têm identidade legível, e três linhas «Cylinder» tornam-na inútil exatamente
/// quando a peça começa a ficar interessante.
///
/// # Errors
/// [`FieldError::BadRoot`] se `parent` não é um nó de modelagem — uma forma pendurada fora da peça
/// seria um objeto que a Hierarquia mostra e o traçado ignora.
pub fn add_leaf(
    world: &mut World,
    parent: Entity,
    primitive: Primitive,
    world_pos: [f32; 3],
) -> Result<Entity, FieldError> {
    if world.get::<FieldNode>(parent).is_none() {
        return Err(FieldError::BadRoot);
    }
    let shape = NodeShape::Leaf(primitive);
    let name = unique_sibling_name(world, parent, crate::shape_name(&shape));
    let child = world
        .spawn((
            ph2d_ecs::Name::new(name),
            FieldNode { shape },
            FieldPose::default(),
        ))
        .id();
    world.entity_mut(parent).add_child(child);
    // ⚠️ A pose depois de ter pai: a conversão mundo→local precisa da cadeia, e antes do
    // `add_child` a cadeia é outra (a identidade).
    let here = crate::world_xform(world, child).translation;
    translate_world(
        world,
        child,
        [
            world_pos[0] - here[0],
            world_pos[1] - here[1],
            world_pos[2] - here[2],
        ],
    );
    Ok(child)
}

/// ⭐ **Troca a operação de um nó de combinação** — união vira subtração, e a peça muda de forma sem
/// se desmontar.
///
/// ⚠️ **O raio da mistura sobrevive à troca.** Ele é do nó, não da operação: um filete de 0,12 que
/// se perdesse ao trocar de união para subtração obrigaria a re-encontrá-lo, e o gesto passaria a
/// custar dois.
///
/// # Errors
/// [`FieldError::BadRoot`] quando o nó não é uma combinação — uma folha não tem operação, e
/// inventar uma seria mudar o que a forma é.
pub fn set_op(world: &mut World, entity: Entity, op: Op) -> Result<(), FieldError> {
    let Some(mut node) = world.get_mut::<FieldNode>(entity) else {
        return Err(FieldError::BadRoot);
    };
    let NodeShape::Combine(current) = node.shape else {
        return Err(FieldError::BadRoot);
    };
    // Reconstrói a operação nova **com a mistura da antiga**.
    let blend = current.blend();
    node.shape = NodeShape::Combine(match op {
        Op::Union(_) => Op::Union(blend),
        Op::Intersection(_) => Op::Intersection(blend),
        Op::Difference(_) => Op::Difference(blend),
    });
    Ok(())
}

/// ⭐ **Embrulha os nós dados numa operação nova**, que fica no lugar deles.
///
/// É a autoria da booleana: escolhem-se duas formas e diz-se *"tira esta daquela"*.
///
/// ⚠️ **A ORDEM é a que entra, e ela é o significado** na subtração: `children[0]` menos todos os
/// seguintes. Ordenar por qualquer outra coisa — pelos bits da entidade, pela ordem da consulta —
/// faria o gesto tirar a peça errada, de forma que parece aleatória entre sessões.
///
/// Devolve `None` quando não há o que embrulhar (menos de dois nós, ou eles não partilham pai).
///
/// ⚠️ **Pai comum é EXIGIDO**, e não uma conveniência: mover um nó para debaixo de outra operação
/// muda o que ele é subtraído de — um segundo gesto, com o seu próprio desfazer. Um «embrulhar» que
/// o fizesse em silêncio seria dois gestos com um nome só.
pub fn wrap_in_op(world: &mut World, nodes: &[Entity], op: Op) -> Option<Entity> {
    if nodes.len() < 2 {
        return None;
    }
    let parent = world.get::<bevy_ecs::hierarchy::ChildOf>(nodes[0])?.0;
    for n in nodes {
        if world.get::<FieldNode>(*n).is_none()
            || world.get::<bevy_ecs::hierarchy::ChildOf>(*n).map(|c| c.0) != Some(parent)
        {
            return None;
        }
    }
    let shape = NodeShape::Combine(op);
    let name = unique_sibling_name(world, parent, crate::shape_name(&shape));
    let group = world
        .spawn((
            ph2d_ecs::Name::new(name),
            FieldNode { shape },
            FieldPose::default(),
        ))
        .id();
    world.entity_mut(parent).add_child(group);
    for n in nodes {
        world.entity_mut(group).add_child(*n);
    }
    Some(group)
}

/// Um nome que nenhum irmão já tem: `Cylinder`, `Cylinder 2`, `Cylinder 3`…
fn unique_sibling_name(world: &World, parent: Entity, base: &str) -> String {
    let taken: Vec<String> = world
        .get::<Children>(parent)
        .map(|c| {
            c.iter()
                .filter_map(|e| {
                    world
                        .get::<ph2d_ecs::Name>(*e)
                        .map(|n| n.as_str().to_string())
                })
                .collect()
        })
        .unwrap_or_default();
    if !taken.iter().any(|n| n == base) {
        return base.to_string();
    }
    // ⚠️ Sem teto: a busca acaba porque cada volta consome um nome que já existe, e a lista é
    // finita. Um `MAX` aqui seria um limite sem recurso por trás.
    (2..)
        .map(|k| format!("{base} {k}"))
        .find(|c| !taken.iter().any(|n| n == c))
        .unwrap_or_else(|| base.to_string())
}
