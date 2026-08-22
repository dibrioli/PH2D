//! ⭐ **A ESTRUTURA da peça** — nascer, agrupar, duplicar, apagar.
//!
//! ⚠️ **Só uma OPERAÇÃO pode ter filhos** (W31): no idioma do campo uma forma é uma **folha**, e o
//! cozimento nunca olha para os filhos dela — um nó largado ali fica no mundo, aparece na
//! Hierarquia e não entra em documento nenhum. A lei impõe-se na **derivação**
//! ([`promote_leaf_hosts`], chamada pelo cozimento do quadro), não em cada gesto.
//!
//! ⚠️ **Os dois predicados desta família são consumidos pelo PAINEL** ([`can_wrap`],
//! [`can_detach`], W34): quem pinta um botão tem de saber a resposta antes do clique e não pode
//! mutar o mundo para descobrir. Enquanto a regra viveu só dentro do gesto, o painel escreveu a
//! dele e as duas divergiram — o gesto de criar grupo ficou inalcançável durante uma wave inteira.

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;
use ph2d_field::{FieldError, NodeShape, Op, Primitive, set_shape_radius};

use crate::{FieldNode, FieldPose};

// ⚠️ Os irmãos entram pelo ROTEADOR (`super`), nunca pelo caminho da crate: é o que mantém
// `edit.rs` a ser a única lista de quem sai deste módulo.
use super::{translate_world, walk};

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

/// ⭐ **[`wrap_in_op`] aceitaria estes nós?** — a MESMA pergunta, sem mutar nada (W34).
///
/// # Por que ela existe separada
///
/// Quem **pinta** os botões de operação tem de saber a resposta antes de o artista clicar, e não
/// pode mutar o mundo para descobrir. Enquanto a regra viveu só dentro do `wrap_in_op`, o painel
/// escreveu a dele — e as duas divergiram: a W31 ensinou o gesto a aceitar **uma** forma sozinha e o
/// painel continuou a exigir **duas**, então o gesto de criar grupo ficou inalcançável e os gates
/// (que empurram a intenção) passaram verdes. ⚠️ *Uma affordance derivada de uma segunda cópia da
/// regra é uma affordance que envelhece sozinha.*
///
/// ⭐ **`wrap_in_op` consome esta função**, e é isso que impede a divergência de voltar: elas não são
/// duas leis parecidas, é uma lei e o seu único porteiro. O gate
/// `can_wrap_answers_exactly_what_wrap_in_op_does` mede a equivalência sobre uma tabela.
///
/// ⚠️ **UM basta** (W31): embrulhar uma forma sozinha é como se **cria um grupo** — Enio,
/// 2026-08-22, *"ainda não temos como criar novos grupos"*. O `>= 2` de origem vinha de o gesto ter
/// nascido como *«juntar os escolhidos»*; uma operação com um filho é o que ela sempre foi (um
/// `Union` de um é esse um), e passa a ter onde receber o segundo.
#[must_use]
pub fn can_wrap(world: &World, nodes: &[Entity]) -> bool {
    let Some(first) = nodes.first() else {
        return false;
    };
    // ⚠️ **Sem pai não há lugar onde pôr o grupo**: quem não tem `ChildOf` é a raiz da peça, e
    // embrulhá-la mudaria o que a Hierarquia mostra COMO peça.
    let Some(parent) = world
        .get::<bevy_ecs::hierarchy::ChildOf>(*first)
        .map(|c| c.0)
    else {
        return false;
    };
    nodes.iter().all(|n| {
        world.get::<FieldNode>(*n).is_some()
            && world.get::<bevy_ecs::hierarchy::ChildOf>(*n).map(|c| c.0) == Some(parent)
    })
}

/// ⭐ **Embrulha os nós dados numa operação nova**, que fica no lugar deles.
///
/// É a autoria da booleana: escolhem-se duas formas e diz-se *"tira esta daquela"*.
///
/// ⚠️ **A ORDEM é a que entra, e ela é o significado** na subtração: `children[0]` menos todos os
/// seguintes. Ordenar por qualquer outra coisa — pelos bits da entidade, pela ordem da consulta —
/// faria o gesto tirar a peça errada, de forma que parece aleatória entre sessões.
///
/// Devolve `None` exatamente quando [`can_wrap`] diz que não — lista vazia, um nó sem pai (a raiz),
/// algo que não é do campo, ou nós que não partilham pai. ⚠️ **Um nó basta** desde a W31, e esta
/// linha dizia «menos de dois» meses depois de a lei mudar: *um comentário que sobrevive à regra que
/// descreve é uma segunda fonte, e é sempre a errada.*
///
/// ⚠️ **Pai comum é EXIGIDO**, e não uma conveniência: mover um nó para debaixo de outra operação
/// muda o que ele é subtraído de — um segundo gesto, com o seu próprio desfazer. Um «embrulhar» que
/// o fizesse em silêncio seria dois gestos com um nome só.
pub fn wrap_in_op(world: &mut World, nodes: &[Entity], op: Op) -> Option<Entity> {
    if !can_wrap(world, nodes) {
        return None;
    }
    let parent = world.get::<bevy_ecs::hierarchy::ChildOf>(nodes[0])?.0;
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

/// ⭐ **Este nó pode ser DESTACADO da peça?** — duplicado ou apagado (W34).
///
/// Os dois gestos recusam a mesma coisa e pela mesma razão: um nó **sem pai** é a raiz, e a raiz *é*
/// a peça. Duplicá-la seria uma segunda peça (gesto da cena) e apagá-la deixaria o módulo sem nada
/// para onde voltar — as duas decisões estão escritas em [`duplicate`] e [`remove`].
///
/// ⚠️ Ela existe **separada** pela razão do [`can_wrap`]: quem pinta a fileira *Duplicar/Apagar* tem
/// de saber a resposta antes do clique e não pode mutar o mundo para descobrir. Enquanto a regra
/// viveu só dentro dos dois gestos, o painel publicava a fileira para **qualquer** seleção — e com a
/// raiz escolhida os dois botões apareciam e não faziam nada. *A recusa era uma decisão; a affordance
/// que a ignorava era um defeito.*
#[must_use]
pub fn can_detach(world: &World, entity: Entity) -> bool {
    world.get::<FieldNode>(entity).is_some()
        && world.get::<bevy_ecs::hierarchy::ChildOf>(entity).is_some()
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
    if !can_detach(world, entity) {
        return None;
    }
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
    if !can_detach(world, entity) {
        return false;
    }
    world.entity_mut(entity).despawn();
    true
}
