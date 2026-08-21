//! **O que se edita num nó da cena**, e as perguntas que o painel faz sobre ele.
//!
//! ⚠️ Nenhuma regra é inventada aqui: o raio muda por [`ph2d_field::set_shape_radius`] e o teto sai
//! de [`ph2d_field::round_limit`] / [`ph2d_field::characteristic_size`] — as **mesmas** funções que
//! a validação do documento cozido usa. Um painel que calculasse o próprio teto ofereceria valores
//! que a peça recusa, e o artista veria o controle parar sem explicação.

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::world::World;
use ph2d_field::xform::{
    quat_axis_angle, quat_conj, quat_mul, quat_normalize, quat_rotate, set_rotation_degree,
};
use ph2d_field::{
    Bound, Dim, FieldError, NodeShape, Op, Param, Primitive, Span, Xform, characteristic_size,
    round_limit, set_shape_radius,
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

/// ⭐ **Todos os números autorados de um nó**, na ordem em que o painel os mostra.
///
/// Posição · rotação · escala (só onde ela não compete com nada) · dimensões da forma.
///
/// ⚠️ **A ordem é a do Inspector de objeto que todo modelador tem** (posição, rotação, escala, e só
/// depois o que a forma mede). Ela não é decorativa: os três primeiros trios existem em **todo** nó,
/// então uma peça inteira lê-se com o olho no mesmo sítio de linha para linha.
///
/// ⚠️ **A escala aparece só numa OPERAÇÃO.** Numa folha, o tamanho visível são as dimensões — e
/// mostrar as duas coisas daria ao artista dois controles para a mesma coisa, sem forma de saber
/// qual o próximo gesto mexe. Ver [`ph2d_field::scale_primitive`], que é o outro lado da mesma
/// decisão.
#[must_use]
pub fn params_of(world: &World, entity: Entity) -> Vec<(Param, Dim)> {
    let Some(node) = world.get::<FieldNode>(entity) else {
        return Vec::new();
    };
    let pose = world
        .get::<FieldPose>(entity)
        .map_or(Xform::IDENTITY, |p| p.xform);
    let degrees = ph2d_field::xform::rotation_degrees(pose);
    let mut out: Vec<(Param, Dim)> = (0..3u8)
        .map(|k| {
            (
                Param::Pos(k),
                Dim {
                    key: POS_KEYS[k as usize],
                    value: pose.translation[k as usize],
                    // ⚠️ Uma posição não tem parede **nem piso**: a origem não é um canto do mundo.
                    // Quem fecha as duas pontas é a vista (ver `Span::Free`).
                    span: Span::Free,
                },
            )
        })
        .chain((0..3u8).map(|k| {
            (
                Param::Rot(k),
                Dim {
                    key: ROT_KEYS[k as usize],
                    value: degrees[k as usize],
                    span: Span::Turn(ROT_SPAN_DEG[k as usize]),
                },
            )
        }))
        .collect();
    match &node.shape {
        NodeShape::Combine(_) => {
            out.push((
                Param::Scale,
                Dim {
                    key: "field.dim.scale",
                    value: pose.scale,
                    span: Span::Positive,
                },
            ));
            // A única dimensão de uma operação é o raio da mistura, e ela entra pela porta de
            // sempre — `Dim(0)`, que o `set_param` reencaminha.
            if let Some(v) = node.shape.radius() {
                out.push((
                    Param::Dim(0),
                    Dim {
                        key: "field.dim.round",
                        value: v,
                        // ⚠️ **Uma mistura não tem parede**: o campo continua a ser uma distância com
                        // qualquer raio ([`radius_bound`] devolve sempre `Soft` aqui).
                        //
                        // ⏸️ O `radius_bound` sabe um alcance mais **apertado** do que o da vista — a
                        // menor peça sob o nó, que é o raio a partir do qual a mistura a engole.
                        // Trocá-lo pelo da vista muda o **tato** do arrasto desta linha e de mais
                        // nenhuma, e isso é número do Enio, com a peça à frente.
                        span: Span::Positive,
                    },
                ));
            }
        }
        NodeShape::Leaf(p) => out.extend(
            ph2d_field::dims(p)
                .into_iter()
                .enumerate()
                .map(|(i, d)| (Param::Dim(i as u16), d)),
        ),
    }
    out
}

/// As chaves i18n dos três eixos da posição.
const POS_KEYS: [&str; 3] = ["field.dim.pos_x", "field.dim.pos_y", "field.dim.pos_z"];

/// As chaves i18n dos três ângulos.
const ROT_KEYS: [&str; 3] = ["field.dim.rot_x", "field.dim.rot_y", "field.dim.rot_z"];

/// ⭐ **A faixa canónica de cada posição do trio, em graus** — e ela **não é a mesma nas três**.
///
/// ⚠️ Não é uma escolha de UI: é o alcance da própria representação. Num XYZ Euler o ângulo do
/// **meio** vive em `[−90°, 90°]` e os de fora em `(−180°, 180°]`, e é por isso que a linha do meio
/// tem metade do curso. Dar-lhe 180 foi o defeito da primeira versão: o slider oferecia sítios que a
/// leitura seguinte **renomeava**, e num arrasto isso vira um ciclo de dois (ver
/// [`ph2d_field::xform::set_rotation_degree`], que é onde a lei e a medição estão).
///
/// ⚠️ **Prender o do meio não perde orientação nenhuma** — toda orientação tem um trio canónico com
/// `|β| ≤ 90°`. Perde-se o *nome*: «Y = 120» escreve-se `X = 180 · Y = 60 · Z = 180`.
const ROT_SPAN_DEG: [f32; 3] = [180.0, 90.0, 180.0];

/// ⭐ **Escreve um número autorado de um nó**, ou recusa — a porta ÚNICA do painel.
///
/// # Errors
/// Ver [`set_dim`] e [`ph2d_field::set_shape_radius`]. [`FieldError::BadRoot`] se a entidade não é
/// um nó, e [`FieldError::NonPositive`] para uma escala não-positiva.
pub fn set_param(
    world: &mut World,
    entity: Entity,
    param: Param,
    value: f32,
) -> Result<(), FieldError> {
    if !value.is_finite() {
        return Err(FieldError::NonPositive {
            node: entity.to_bits() as u32,
            what: "param",
        });
    }
    match param {
        Param::Pos(k) if k < 3 => {
            let Some(mut pose) = world.get_mut::<FieldPose>(entity) else {
                return Err(FieldError::BadRoot);
            };
            pose.xform.translation[k as usize] = value;
            Ok(())
        }
        Param::Pos(_) => Err(FieldError::BadRoot),
        // ⚠️ **Escrever um ângulo lê os outros dois primeiro.** A pose guarda um quaternion, e três
        // ângulos são o nome canónico dele: pôr só um sem os companheiros seria construir uma
        // orientação a partir de um terço da informação. A lei — e o que ela renomeia — está em
        // [`set_rotation_degree`].
        Param::Rot(k) if k < 3 => {
            let Some(mut pose) = world.get_mut::<FieldPose>(entity) else {
                return Err(FieldError::BadRoot);
            };
            set_rotation_degree(&mut pose.xform, k, value);
            Ok(())
        }
        Param::Rot(_) => Err(FieldError::BadRoot),
        Param::Scale => {
            if value <= 0.0 {
                return Err(FieldError::NonPositive {
                    node: entity.to_bits() as u32,
                    what: "scale",
                });
            }
            let Some(mut pose) = world.get_mut::<FieldPose>(entity) else {
                return Err(FieldError::BadRoot);
            };
            pose.xform.scale = value;
            Ok(())
        }
        Param::Dim(i) => set_dim(world, entity, i as usize, value),
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

/// ⭐ **Duplica um nó e tudo o que está debaixo dele**, como irmão.
///
/// Devolve a cópia. `offset` é o deslocamento de **mundo** que a separa do original.
///
/// ⚠️ **A subárvore inteira**, e não só o nó: o caso útil é copiar um *furo* que já é ele próprio
/// uma subtração de três formas. Copiar só o topo daria um grupo vazio, que não é nada.
///
/// ⚠️ **A ordem dos filhos é preservada**, e isso é o significado numa subtração (`children[0]`
/// menos os seguintes). Uma cópia que baralhasse a ordem seria a mesma forma só às vezes.
///
/// Devolve `None` para um nó sem pai — a raiz **é** a peça, e uma segunda peça é um gesto da cena,
/// não uma edição desta.
pub fn duplicate(world: &mut World, entity: Entity, offset: [f32; 3]) -> Option<Entity> {
    let parent = world.get::<bevy_ecs::hierarchy::ChildOf>(entity)?.0;
    let copy = copy_subtree(world, entity, parent)?;
    translate_world(world, copy, offset);
    Some(copy)
}

/// A cópia recursiva, **sem recursão da linguagem**: uma pilha de `(origem, pai-do-destino)`.
fn copy_subtree(world: &mut World, from: Entity, into: Entity) -> Option<Entity> {
    let mut stack = vec![(from, into)];
    let mut first = None;
    while let Some((src, dst_parent)) = stack.pop() {
        let Some(node) = world.get::<FieldNode>(src).cloned() else {
            continue;
        };
        let pose = world.get::<FieldPose>(src).copied().unwrap_or_default();
        let name = unique_sibling_name(world, dst_parent, crate::shape_name(&node.shape));
        let copy = world.spawn((ph2d_ecs::Name::new(name), node, pose)).id();
        world.entity_mut(dst_parent).add_child(copy);
        first.get_or_insert(copy);
        // ⚠️ Em ordem INVERSA, porque a fila é uma pilha: assim os filhos nascem na ordem de
        // `Children`, que é a que a subtração lê.
        let kids: Vec<Entity> = world
            .get::<Children>(src)
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default();
        for k in kids.into_iter().rev() {
            stack.push((k, copy));
        }
    }
    first
}

/// ⭐ **Apaga um nó e o que está debaixo dele.**
///
/// ⚠️ **A raiz é recusada**: ela *é* a peça, e apagá-la deixaria o módulo sem nada para onde voltar
/// (a cena inicial só existe no primeiro quadro). Remover a peça é um gesto da **Hierarquia**, que é
/// onde os objetos do projeto se apagam — e de onde o desfazer a traz de volta.
///
/// Devolve `false` quando não apagou nada.
pub fn remove(world: &mut World, entity: Entity) -> bool {
    if world.get::<FieldNode>(entity).is_none()
        || world.get::<bevy_ecs::hierarchy::ChildOf>(entity).is_none()
    {
        return false;
    }
    world.entity_mut(entity).despawn();
    true
}
