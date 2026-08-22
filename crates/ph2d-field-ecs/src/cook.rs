//! **O cozimento**: a hierarquia da cena vira o documento que o traçador avalia.
//!
//! ⚠️ Uma direção só. Nada aqui escreve no mundo — quem autora é o gizmo, o painel e a Hierarquia.
//! *O passe publica o que as coisas são; ele não escreve o que elas são* (a lei do ADR-0153, no
//! outro sentido).

use std::collections::BTreeMap;

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;
use ph2d_field::{FieldDoc, FieldError, Node, NodeId, NodeKind, NodeShape, Xform};

use crate::{FieldMods, FieldNode, FieldPose};

/// **Coze a subárvore de `root` num [`FieldDoc`].**
///
/// A invariante da arena — *todo filho tem índice estritamente menor que o do pai* — sai de graça:
/// a travessia é **pós-ordem**, então um pai só é escrito depois de todos os filhos dele.
///
/// # ⭐ Um nó sem filhos não é um erro: ele não existe
///
/// Uma operação com zero filhos **não é emitida**, e o pai dela vê um filho a menos. Se isso
/// esvaziar o pai, ele também não é emitido; se esvaziar a raiz, a função devolve `None`.
///
/// ⚠️ **É de propósito, e substitui uma classe de erro por nada.** A alternativa seria devolver
/// `EmptyCombine` — e aí apagar o último cilindro de uma união deixaria a peça *inválida* em vez de
/// a deixar *vazia*, com uma mensagem de erro a explicar ao artista que o que ele acabou de fazer é
/// ilegal. Apagar o último filho de um grupo é um gesto normal; o resultado normal dele é não haver
/// mais nada ali.
///
/// Devolve `None` quando não há geometria nenhuma sob `root`.
///
/// # Errors
/// Propaga a validação de [`FieldDoc::new`] — um raio que não cabe, uma escala não-positiva. Não
/// pode devolver `BadRoot` nem `ForwardReference`: os dois são impossíveis por construção aqui.
#[must_use]
pub fn cook(world: &World, root: Entity) -> Option<Result<FieldDoc, FieldError>> {
    let mut nodes: Vec<Node> = Vec::new();
    let root_id = emit(world, root, &mut nodes)?;
    Some(FieldDoc::new(nodes, root_id))
}

/// Onde a travessia está: a descer (ainda vai empilhar filhos) ou a subir (já pode emitir).
enum Step {
    Down(Entity),
    Up(Entity),
}

/// Uma passagem pós-ordem, **sem recursão de pilha da linguagem**.
///
/// ⚠️ Iterativa de propósito: a profundidade da árvore é o que o artista agrupar, e uma recursão
/// aqui transformaria um agrupamento fundo num estouro de pilha — um crash cuja causa nada na tela
/// explica.
fn emit(world: &World, root: Entity, nodes: &mut Vec<Node>) -> Option<NodeId> {
    let mut stack = vec![Step::Down(root)];
    // O que cada entidade rendeu — `None` quando ela não rendeu nada (ver o doc de [`cook`]).
    // `BTreeMap`, nunca `HashMap`: a ordem de iteração de um mapa entra na ordem dos filhos, e a
    // ordem dos filhos é a espinha do determinismo (HR-5).
    let mut done: BTreeMap<Entity, Option<NodeId>> = BTreeMap::new();

    while let Some(step) = stack.pop() {
        match step {
            Step::Down(e) => {
                // Uma entidade sem `FieldNode` não é um nó do modelo — pode ser qualquer coisa que
                // alguém pendurou aqui. Ela não participa, e os filhos dela também não: um nó de
                // modelagem só é filho de outro nó de modelagem.
                if world.get::<FieldNode>(e).is_none() || is_hidden(world, e) {
                    continue;
                }
                stack.push(Step::Up(e));
                // Empilha em ordem inversa para o `pop` visitar na ordem de `Children` — a mesma
                // ordem que a Hierarquia mostra. Ela é load-bearing na SUBTRAÇÃO: `children[0]`
                // menos todos os seguintes.
                for c in kids(world, e).into_iter().rev() {
                    stack.push(Step::Down(c));
                }
            }
            Step::Up(e) => {
                let Some(node) = world.get::<FieldNode>(e) else {
                    continue;
                };
                let xform = world
                    .get::<FieldPose>(e)
                    .map_or(Xform::IDENTITY, |p| p.xform);
                // ⚠️ **A pilha é um componente OPCIONAL**, e a esmagadora maioria dos nós não a
                // tem: pô-la dentro do `FieldNode` custaria bytes em todo nó e mudaria a forma
                // posicional de um componente que já está gravado em projetos.
                let mods = world
                    .get::<FieldMods>(e)
                    .map(|m| m.stack.clone())
                    .unwrap_or_default();
                let id = match &node.shape {
                    NodeShape::Sampled { key } => {
                        nodes.push(Node {
                            xform,
                            kind: NodeKind::Sampled { key: key.clone() },
                            mods,
                        });
                        Some(NodeId(nodes.len() as u32 - 1))
                    }
                    NodeShape::Leaf(p) => {
                        nodes.push(Node {
                            xform,
                            kind: NodeKind::Leaf(p.clone()),
                            mods,
                        });
                        Some(NodeId(nodes.len() as u32 - 1))
                    }
                    NodeShape::Combine(op) => {
                        let children: Vec<NodeId> = kids(world, e)
                            .into_iter()
                            .filter_map(|k| done.get(&k).copied().flatten())
                            .collect();
                        if children.is_empty() {
                            None
                        } else {
                            nodes.push(Node {
                                xform,
                                kind: NodeKind::Combine { op: *op, children },
                                mods,
                            });
                            Some(NodeId(nodes.len() as u32 - 1))
                        }
                    }
                };
                done.insert(e, id);
            }
        }
    }
    done.get(&root).copied().flatten()
}

/// ⭐ **A pose de mundo deste nó, se ele for de modelagem** — `None` para qualquer outra entidade.
///
/// ⚠️ Ela existe para o **re-parentar** (W30): quem arrasta uma linha da Hierarquia precisa de saber
/// onde o nó estava **antes** de o pai mudar, e a resposta só é sua quando o nó é deste módulo.
#[must_use]
pub fn field_world_xform(world: &World, entity: Entity) -> Option<Xform> {
    world.get::<FieldNode>(entity)?;
    Some(world_xform(world, entity))
}

/// ⭐ **Põe o nó NESTA pose de mundo**, resolvendo o local sob o pai que ele tem agora.
///
/// ⚠️ **É a metade que faz um re-parentar não teleportar a peça.** A pose que o documento guarda é
/// **local** (regra-mãe do módulo), então trocar de pai muda o que ela significa: sem esta função, o
/// nó salta para onde a mesma pose local cai debaixo do pai novo. A lei é a mesma que o Enio escreveu
/// para os sprites em 2026-06-08 — *"não pode saltar ao ganhar ou perder um pai"* —, e a álgebra é a
/// [`Xform::local_under`], que vive ao lado da `compose` que ela desfaz.
pub fn set_world_xform(world: &mut World, entity: Entity, target: Xform) {
    if world.get::<FieldNode>(entity).is_none() {
        return;
    }
    let parent = world
        .get::<ChildOf>(entity)
        .map(|c| c.0)
        .map_or(Xform::IDENTITY, |p| world_xform(world, p));
    let local = Xform::local_under(parent, target);
    if let Some(mut pose) = world.get_mut::<FieldPose>(entity) {
        pose.xform = local;
    }
}

/// ⭐ **O olho da Hierarquia apaga o nó da peça** (W28) — e com ele a subárvore inteira.
///
/// ⚠️ **Nada de componente novo:** o app já tem **uma** ideia de *escondido*
/// ([`ph2d_ecs::Visibility`], o ícone do olho), já a persiste, já a desfaz, e a Hierarquia já a
/// escreve em **qualquer** entidade — inclusive num nó de modelagem. O que faltava era o cozimento
/// perguntar. Até esta linha, clicar no olho de um cilindro escrevia o componente, acendia o ícone e
/// **não mudava a peça**: um controle pintado, despachado, e mudo.
///
/// ⚠️ **A ausência é visível**, e é a lei do componente (HR-5): um nó sem `Visibility` conta.
///
/// ⚠️ A recusa acontece na **DESCIDA**, e é isso que faz esconder um grupo esconder o que está
/// dentro dele: a travessia nunca chega aos filhos, e o pai vê um filho a menos — exactamente o
/// mesmo caminho de um nó apagado.
#[must_use]
pub fn is_hidden(world: &World, e: Entity) -> bool {
    world
        .get::<ph2d_ecs::Visibility>(e)
        .is_some_and(|v| v.hidden)
}

fn kids(world: &World, e: Entity) -> Vec<Entity> {
    world
        .get::<Children>(e)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default()
}

/// **A pose de MUNDO de um nó** — a cadeia de [`FieldPose`] composta da raiz para baixo.
///
/// ⚠️ É o que o gizmo desenha e o que ele agarra. A regra-mãe do vetorial vale igual aqui: *o que se
/// vê e se aponta é MUNDO; o que o documento guarda é LOCAL.*
///
/// Sobe por `ChildOf` até acabar — entidades sem [`FieldPose`] contribuem a identidade, que é o que
/// «este nível não move nada» significa.
#[must_use]
pub fn world_xform(world: &World, entity: Entity) -> Xform {
    let mut chain: Vec<Xform> = Vec::new();
    let mut cur = Some(entity);
    while let Some(e) = cur {
        chain.push(
            world
                .get::<FieldPose>(e)
                .map_or(Xform::IDENTITY, |p| p.xform),
        );
        cur = world.get::<ChildOf>(e).map(|c| c.0);
    }
    // Da raiz para a folha: `pai ∘ filho`.
    chain
        .into_iter()
        .rev()
        .fold(Xform::IDENTITY, Xform::compose)
}
