//! **O que se edita num nó da cena**, e as perguntas que o painel faz sobre ele.
//!
//! ⚠️ Nenhuma regra é inventada aqui: o raio muda por [`ph2d_field::set_shape_radius`] e o teto sai
//! de [`ph2d_field::round_limit`] / [`ph2d_field::characteristic_size`] — as **mesmas** funções que
//! a validação do documento cozido usa. Um painel que calculasse o próprio teto ofereceria valores
//! que a peça recusa, e o artista veria o controle parar sem explicação.

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;
use ph2d_field::xform::{
    quat_axis_angle, quat_conj, quat_mul, quat_normalize, quat_rotate, set_rotation_degree,
};
use ph2d_field::{
    Bound, Dim, FieldError, NodeShape, Op, Param, Primitive, Span, Unary, Xform,
    characteristic_size, round_limit, set_shape_radius,
};

use crate::{FieldMods, FieldNode, FieldPose};

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
        // Uma escultura não tem raio autorado: a aresta dela é a malha.
        NodeShape::Sampled { .. } => None,
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
            // ⚠️ A escala característica de uma escultura é a caixa dela, e a caixa vive no campo
            // amostrado, que o mundo não conhece. Não contribuir é a resposta certa quando há outra
            // peça a dar a escala — e o item aberto quando ela for a única.
            NodeShape::Sampled { .. } => {}
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
                    // ⚠️ **A mesma porta que recusa a escrita** decide se há faixa: um eixo que a
                    // trava de cardan tirou do mapa não é um slider curto, é um facto sem controle.
                    span: if ph2d_field::xform::rotation_axis_is_free(pose, k) {
                        Span::Turn(ROT_SPAN_DEG[k as usize])
                    } else {
                        Span::Locked
                    },
                },
            )
        }))
        .collect();
    match &node.shape {
        // Uma escultura tem pose e mais nada: o que ela é vive na malha.
        NodeShape::Sampled { .. } => {}
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
    // ⭐ **Os modificadores vêm por ÚLTIMO**, e é a ordem em que eles correm: primeiro o que a forma
    // é, depois o que se fez a ela. Uma linha de casca acima da largura da caixa leria como se a
    // parede fosse uma propriedade da caixa, e ela é uma operação sobre o resultado.
    // ⚠️ **Um modificador pode ter VÁRIOS números** (uma matriz tem contagem e espaçamento) — e
    // pode não ter nenhum (o espelho). O `flat_map` é o que exprime as duas pontas sem um caso
    // especial para cada.
    out.extend(
        mods_of(world, entity)
            .into_iter()
            .enumerate()
            .flat_map(|(slot, m)| {
                m.dims().into_iter().enumerate().map(move |(field, d)| {
                    (
                        Param::Mod {
                            slot: slot as u16,
                            field: field as u8,
                        },
                        d,
                    )
                })
            }),
    );
    out
}

/// ⭐ **A pilha de modificadores deste nó** — vazia quando ele não tem nenhum.
#[must_use]
pub fn mods_of(world: &World, entity: Entity) -> Vec<Unary> {
    world
        .get::<FieldMods>(entity)
        .map(|m| m.stack.clone())
        .unwrap_or_default()
}

/// ⭐ **Acrescenta um modificador ao nó**, no ponto neutro da natureza dele.
///
/// ⚠️ **O tamanho de nascimento vem da PEÇA**, não de uma constante: uma casca é uma fração da
/// menor peça sob o nó ([`subtree_scale`]), porque só quem vê a peça sabe o que é fino nela. Um
/// número absoluto seria invisível numa peça grande e engoliria uma pequena — nos dois casos o
/// artista conclui que o botão não fez nada.
///
/// Devolve `false` sem escrever nada quando a entidade não é um nó, ou é uma **escultura** (ver
/// abaixo) — e é o `false` que deixa quem chamou dizê-lo ao artista.
pub fn add_mod(world: &mut World, entity: Entity, kind: ph2d_field::UnaryKind) -> bool {
    let Some(node) = world.get::<FieldNode>(entity) else {
        return false;
    };
    // ⭐ **Uma escultura NÃO aceita modificadores, e a recusa é aqui** (W25).
    //
    // ⚠️ **A regra já existia — no documento** ([`ph2d_field::FieldError::ModsOnSampled`]) — e
    // ninguém a consultava antes de escrever. O componente entrava no mundo, o cozimento do quadro
    // seguinte recusava o documento inteiro, e a peça **inteira** desaparecia da tela com a
    // Hierarquia intacta. *Uma invariante que só o validador conhece é uma invariante que a UI
    // descobre partindo-se.*
    if matches!(node.shape, NodeShape::Sampled { .. }) {
        return false;
    }
    let born = Unary::born(kind, subtree_scale(world, entity));
    let mut e = world.entity_mut(entity);
    if let Some(mut m) = e.get_mut::<FieldMods>() {
        m.stack.push(born);
    } else {
        e.insert(FieldMods { stack: vec![born] });
    }
    true
}

/// ⭐ **Tira do nó o PRIMEIRO modificador daquela natureza**, e diz se tirou algum.
///
/// ⚠️ O primeiro, e não todos: a pilha é ordenada, e apagar em bloco tiraria um que o artista pôs
/// de propósito depois de outro. Devolve `false` quando não havia nenhum — é o que faz o botão ser
/// um interruptor honesto.
pub fn remove_mod(world: &mut World, entity: Entity, kind: ph2d_field::UnaryKind) -> bool {
    let Some(mut m) = world.get_mut::<FieldMods>(entity) else {
        return false;
    };
    let Some(i) = m.stack.iter().position(|u| u.kind() == kind) else {
        return false;
    };
    m.stack.remove(i);
    // ⚠️ **A pilha vazia sai do nó.** Um componente presente e vazio não muda a forma, mas muda os
    // BYTES — e o undo compara bytes: acrescentar e tirar um modificador deixaria a peça diferente
    // de si mesma, e o desfazer teria um passo a mais do que o artista fez.
    let empty = m.stack.is_empty();
    if empty {
        world.entity_mut(entity).remove::<FieldMods>();
    }
    true
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
        // ⚠️ A escrita passa pela porta do próprio modificador (`Unary::set_value`), que é a mesma
        // que a validação do documento usa — ver a nota lá sobre duas listas de regras.
        Param::Mod { slot, field } => {
            let id = entity.to_bits() as u32;
            let Some(mut m) = world.get_mut::<FieldMods>(entity) else {
                return Err(FieldError::BadRoot);
            };
            let Some(target) = m.stack.get_mut(slot as usize) else {
                return Err(FieldError::BadRoot);
            };
            let previous = *target;
            target.set_dim(id, field, value).inspect_err(|_| {
                // Uma recusa deixa o nó **como estava** — a invariante do módulo.
                *target = previous;
            })
        }
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
        NodeShape::Sampled { .. } => Err(FieldError::BadRoot),
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
    add_node(world, parent, NodeShape::Leaf(primitive), world_pos)
}

/// ⭐ **Acrescenta uma ESCULTURA** — o mesmo gesto, com a folha amostrada em vez de uma primitiva.
///
/// ⚠️ **O `key` é o CAMINHO do arquivo**, e isso não é conveniência: é o que torna a persistência
/// possível sem guardar a grade. Um projeto carregado sabe de onde regenerar cada escultura, e o
/// documento continua a pesar bytes em vez de megabytes.
///
/// # Errors
/// [`FieldError::BadRoot`] se `parent` não for um nó do campo.
pub fn add_sampled(
    world: &mut World,
    parent: Entity,
    key: &str,
    world_pos: [f32; 3],
) -> Result<Entity, FieldError> {
    add_node(
        world,
        parent,
        NodeShape::Sampled {
            key: key.to_string(),
        },
        world_pos,
    )
}

/// O corpo partilhado: nasce o nó, entra na hierarquia, e a pose vai para o MUNDO pedido.
fn add_node(
    world: &mut World,
    parent: Entity,
    shape: NodeShape,
    world_pos: [f32; 3],
) -> Result<Entity, FieldError> {
    if world.get::<FieldNode>(parent).is_none() {
        return Err(FieldError::BadRoot);
    }
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
    // ⭐ **UM basta** (W31): embrulhar uma forma sozinha é como se **cria um grupo** — e era o que
    // faltava. Enio, 2026-08-22: *"ainda não temos como criar novos grupos"*. O `>= 2` de origem
    // vinha de o gesto ter nascido como *«juntar os escolhidos»*; a operação com um filho é a
    // mesma coisa que ela sempre foi (um `Union` de um é esse um), e passa a ter onde receber o
    // segundo.
    if nodes.is_empty() {
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

/// ⭐ **SÓ UMA OPERAÇÃO PODE TER FILHOS** — e esta função repara quem quebrou a lei (W31).
///
/// # O defeito, com as palavras do Enio
///
/// *"Se coloco um objeto como filho do outro ele some."* (2026-08-22). E some mesmo: no idioma do
/// campo, uma **forma** é uma folha — o cozimento emite-a e **nunca olha para os filhos dela**. Um
/// nó largado ali fica no mundo, aparece na Hierarquia, e não é referenciado por documento nenhum.
/// *Uma árvore que a UI aceita e a linguagem não exprime é um objeto que desaparece em silêncio.*
///
/// # A cura: PROMOVER o anfitrião, não recusar o gesto
///
/// A forma que recebeu o filho passa a viver dentro de uma **união** nova, no lugar dela — e o filho
/// entra ao lado. ⭐ **A peça na tela não muda com isto**: os dois já lá estavam, e a união deles é
/// exactamente o que se via. O artista ganha o aninhamento que pediu, e não perde nada.
///
/// ⚠️ **A ordem dos irmãos é preservada**, e não é cerimónia: em `children[0] menos os seguintes`, a
/// primeira posição é a **base** da subtração. Um grupo acrescentado no fim mudaria quem corta quem.
///
/// Devolve quantos anfitriões foram promovidos.
pub fn promote_leaf_hosts(world: &mut World, root: Entity) -> usize {
    let hosts: Vec<Entity> = walk(world, root)
        .into_iter()
        .map(|(e, _)| e)
        .filter(|e| {
            matches!(
                world.get::<FieldNode>(*e).map(|n| &n.shape),
                Some(NodeShape::Leaf(_) | NodeShape::Sampled { .. })
            ) && world
                .get::<Children>(*e)
                .is_some_and(|c| c.iter().any(|k| world.get::<FieldNode>(*k).is_some()))
        })
        .collect();
    let mut done = 0;
    for host in hosts {
        let Some(parent) = world.get::<ChildOf>(host).map(|c| c.0) else {
            // Uma folha SEM pai é a raiz da peça, e a raiz é dona do objeto: promovê-la mudaria o
            // que a Hierarquia mostra como peça. Quem chega aqui é um caso que não existe hoje.
            continue;
        };
        let siblings: Vec<Entity> = world
            .get::<Children>(parent)
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default();
        let kids: Vec<Entity> = world
            .get::<Children>(host)
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default();

        let shape = NodeShape::Combine(Op::Union(ph2d_field::Blend::Sharp));
        let name = unique_sibling_name(world, parent, crate::shape_name(&shape));
        let group = world
            .spawn((
                ph2d_ecs::Name::new(name),
                FieldNode { shape },
                FieldPose::default(),
            ))
            .id();
        // O grupo toma o LUGAR do anfitrião entre os irmãos.
        world.entity_mut(group).insert(ChildOf(parent));
        world.entity_mut(group).add_child(host);
        for k in kids {
            world.entity_mut(group).add_child(k);
        }
        // …e a ordem dos irmãos é reposta com o grupo onde o anfitrião estava.
        let order: Vec<Entity> = siblings
            .into_iter()
            .map(|s| if s == host { group } else { s })
            .collect();
        for s in order {
            world.entity_mut(s).remove::<ChildOf>();
            world.entity_mut(s).insert(ChildOf(parent));
        }
        done += 1;
    }
    done
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
