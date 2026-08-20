//! ⭐ **A ponte com a CENA**: cada cilindro, cada caixa e cada operação é uma **entidade**.
//!
//! # A história curta deste arquivo
//!
//! - **W1** criou o componente e provou por gate que ele atravessa o snapshot — e **ninguém o
//!   produzia**. O gate media a metade errada: que o componente *sobrevive*, nunca que alguma coisa
//!   o *põe* no mundo. Enio, 2026-08-19: *"os objetos não aparecem na hierarchy"*.
//! - **W4** pôs a peça no mundo — como **um** objeto, com a árvore inteira escondida dentro dele.
//!   Enio, no mesmo dia: *"na hierarchy apenas um objeto e não 3 cilindro. Não há gizmo 3d para
//!   mover os objetos."* As duas frases são **um** defeito: um objeto que a cena não enumera não
//!   tem pose que um gizmo agarre.
//! - **W5** (aqui) faz a **hierarquia da cena ser a árvore de modelagem**, e o documento que o
//!   traçador avalia passa a ser **cozido** dela a cada quadro (`ph2d_field_ecs::cook`).
//!
//! ⚠️ **O MUNDO é a verdade.** O `Smoke::doc` é um cache do quadro para a thread do traçado, que
//! precisa de uma cópia própria de qualquer forma.

use ph2d_ecs::SimWorld;
use ph2d_field::{FieldDoc, NodeShape, Op, Primitive};
use ph2d_field_ecs::{FieldNode, FieldObject};

use crate::field3d_smoke::with_smoke;

/// O nome da peça na Hierarquia. É **conteúdo** (um `Name` que o artista renomeia), não chrome —
/// por isso não passa pelo i18n. Ver `ph2d_field_ecs::shape_name`.
const PART_NAME: &str = "Model";

/// Corre uma vez por quadro, antes do traçado. No-op silencioso quando o módulo não está armado.
/// Devolve **um pedido de seleção** quando a peça acabou de nascer — ver [`sync_scene`].
pub(crate) fn ecs_bridge(sim: &mut SimWorld, selected: Option<u64>) -> Option<u64> {
    let (initial, ms, pending) =
        with_smoke(|s| (s.doc.clone(), s.last_trace_ms, s.pending_move.take()))?;
    // ⭐ **O arrasto do gizmo entra AQUI**, antes do retrato e do cozimento, pela mesma razão que os
    // intents do painel: o mundo é a verdade e este é o único sítio que a escreve.
    if let Some((bits, motion)) = pending {
        let entity = bevy_ecs::entity::Entity::from_bits(bits);
        let world = sim.world_mut();
        match motion {
            crate::field3d_gizmo::Motion::Translate(d) => {
                ph2d_field_ecs::translate_world(world, entity, d);
            }
            crate::field3d_gizmo::Motion::Rotate { axis, angle } => {
                ph2d_field_ecs::rotate_world(world, entity, axis, angle);
            }
            crate::field3d_gizmo::Motion::Scale(f) => {
                ph2d_field_ecs::scale_by(world, entity, f);
            }
        }
    }
    let (cooked, born) = sync_scene_and_birth(sim, initial.as_ref(), ms);
    let anchor = anchor_for(sim, selected);
    with_smoke(|s| {
        s.gizmo = anchor;
        // ⚠️ Só se escreve quando MUDOU: atribuir todo quadro faria o documento parecer novo e
        // re-traçar para sempre, matando o "só se traça o que mudou".
        if s.doc != cooked {
            s.doc = cooked;
        }
    });
    born
}

/// O que a ponte **faz**, separado de **se** ela corre.
///
/// ⚠️ A separação existe para o gate: `ecs_bridge` pergunta pelo estado do smoke, e um teste não
/// consegue (nem deve) encená-lo. Aqui a peça inicial entra por parâmetro, e o resto é o caminho de
/// produção inteiro — mundo, entidades, intents, retrato, cozimento.
///
/// Devolve `None` quando não há geometria nenhuma: apagar o último filho de uma peça na Hierarquia
/// é um gesto normal, e o resultado normal dele é a tela ficar vazia.
#[cfg(test)]
pub(crate) fn sync_scene(
    sim: &mut SimWorld,
    initial: Option<&FieldDoc>,
    last_trace_ms: f32,
) -> Option<FieldDoc> {
    sync_scene_and_birth(sim, initial, last_trace_ms).0
}

/// A mesma coisa, mais **quem selecionar quando a peça acaba de nascer**.
///
/// ⭐ *Feature nova = auto-play*: um gizmo que só aparece depois de o artista adivinhar que tem de
/// clicar numa linha da Hierarquia é um gizmo que a maioria nunca vê. Ao nascer, a peça seleciona o
/// **primeiro filho** — um objeto de verdade, com setas em cima dele —, e não a raiz, que é o grupo
/// inteiro. Uma vez, e só nessa: re-selecionar todo quadro tiraria da mão do artista o direito de
/// escolher outro.
pub(crate) fn sync_scene_and_birth(
    sim: &mut SimWorld,
    initial: Option<&FieldDoc>,
    last_trace_ms: f32,
) -> (Option<FieldDoc>, Option<u64>) {
    let mut born = None;
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldObject)>();
    let root = match q.iter(world).next().map(|(e, _)| e) {
        Some(e) => e,
        // A primeira vez: a peça inicial explode em objetos. Depois disto a **cena** é a fonte e
        // ninguém volta a chamar isto — inclusive porque `initial` deixa de existir.
        None => {
            let Some(doc) = initial else {
                return (None, None);
            };
            let root = ph2d_field_ecs::spawn_doc(world, doc, PART_NAME);
            born = Some(
                world
                    .get::<bevy_ecs::hierarchy::Children>(root)
                    .and_then(|c| c.iter().copied().next())
                    .unwrap_or(root)
                    .to_bits(),
            );
            root
        }
    };

    // As edições do painel escrevem no COMPONENTE do nó, que é a peça de verdade.
    for intent in ph2d_panel_model3d::drain_intents() {
        match intent {
            // ⭐ O verbo do gizmo é estado de VISTA: ele não entra no mundo, entra no smoke.
            ph2d_panel_model3d::ModelIntent::SetGizmoMode { slot } => {
                if let Some(mode) = crate::field3d_gizmo::Mode::ALL.get(slot).copied() {
                    with_smoke(|s| {
                        s.gizmo_mode = mode;
                        // Trocar de verbo com uma alça agarrada deixaria um arrasto órfão.
                        s.drag = None;
                        s.gizmo_hot = None;
                    });
                }
            }
            ph2d_panel_model3d::ModelIntent::SetRadius { entity, radius } => {
                // Uma recusa é informação, não erro: o nó diz que aquele raio não cabe, e o retrato
                // publicado logo abaixo devolve o controle ao valor que ficou.
                let _ = ph2d_field_ecs::set_radius(
                    world,
                    bevy_ecs::entity::Entity::from_bits(entity),
                    radius,
                );
            }
        }
    }

    publish_snapshot(world, root, last_trace_ms);
    // ⚠️ Uma peça inválida (um raio que deixou de caber porque a escala do pai mudou) devolve
    // `None` aqui, e a tela mostra o que o cozimento **de facto** produziu. Guardar o último
    // documento válido faria a tela mentir sobre a cena — que é exatamente o defeito que este
    // módulo acabou de pagar no cache do traçado.
    (ph2d_field_ecs::cook(world, root).and_then(Result::ok), born)
}

/// **A ponte com o painel**: publica o retrato da peça.
///
/// ⭐ **A ordem é load-bearing.** Drenar ANTES de publicar é o que faz a edição aparecer no mesmo
/// quadro: se o retrato saísse primeiro, o painel pintaria o valor antigo por um quadro e o
/// controle daria um salto para trás debaixo do dedo — o sintoma clássico de um espelho publicado
/// cedo demais.
fn publish_snapshot(world: &bevy_ecs::world::World, root: bevy_ecs::entity::Entity, ms: f32) {
    let all = ph2d_field_ecs::walk(world, root);
    let rows: Vec<ph2d_panel_model3d::RadiusRow> = all
        .iter()
        .filter_map(|&(e, depth)| {
            Some(ph2d_panel_model3d::RadiusRow {
                entity: e.to_bits(),
                depth,
                kind_key: kind_key(&world.get::<FieldNode>(e)?.shape),
                // ⚠️ O raio E o teto vêm os DOIS do nó. Um painel que guardasse o seu próprio valor
                // teria duas verdades sobre o mesmo número, e a que aparece na tela seria a errada
                // sempre que algo o mudasse de outro lado — um desfazer, um arquivo aberto.
                radius: ph2d_field_ecs::radius_of(world, e)?,
                bound: ph2d_field_ecs::radius_bound(world, e)?,
            })
        })
        .collect();
    // ⚠️ A lista de verbos é **derivada de `Mode::ALL`**, que é a fonte da contagem. O painel não
    // conhece o enum — acrescentar um verbo lá faz o seletor seguir sem uma linha de mudança.
    let active = with_smoke(|s| s.gizmo_mode).unwrap_or_default();
    let modes = crate::field3d_gizmo::Mode::ALL
        .iter()
        .map(|m| ph2d_panel_model3d::ModeChip {
            key: m.key(),
            active: *m == active,
        })
        .collect();
    ph2d_panel_model3d::publish(ph2d_panel_model3d::ModelSnapshot {
        modes,
        rows,
        node_count: all.len(),
        last_trace_ms: ms,
    });
}

/// ⭐ **Onde o gizmo tem de aparecer** — a pose de MUNDO do nó selecionado.
///
/// ⚠️ A seleção é a do **app** (`hero.gizmo.selection`), e não uma deste módulo: clicar numa linha
/// da Hierarquia é o gesto que faz as setas aparecerem. Uma seleção própria seria uma segunda ideia
/// de *"o que está selecionado"* dentro do mesmo aplicativo, e as duas divergiriam no primeiro
/// clique.
///
/// Devolve `None` quando o selecionado não é um nó de modelagem — um sprite selecionado não pode
/// fazer aparecer um gizmo 3D em cima dele.
fn anchor_for(sim: &mut SimWorld, selected: Option<u64>) -> Option<crate::field3d_gizmo::Anchor> {
    let bits = selected?;
    let entity = bevy_ecs::entity::Entity::from_bits(bits);
    let world = sim.world_mut();
    world.get::<FieldNode>(entity)?;
    Some(crate::field3d_gizmo::Anchor {
        entity: bits,
        origin: ph2d_field_ecs::world_xform(world, entity).translation,
    })
}

/// A chave i18n do que um nó é. ⚠️ Uma **chave**, nunca um rótulo pronto (HR-15).
pub(crate) fn kind_key(shape: &NodeShape) -> &'static str {
    match shape {
        NodeShape::Combine(op) => match op {
            Op::Union(_) => "panel.model3d.kind.union",
            Op::Intersection(_) => "panel.model3d.kind.intersection",
            Op::Difference(_) => "panel.model3d.kind.difference",
        },
        NodeShape::Leaf(p) => match p {
            Primitive::Cylinder { .. } => "panel.model3d.kind.cylinder",
            Primitive::Extrude { .. } => "panel.model3d.kind.extrude",
            _ => "panel.model3d.kind.box",
        },
    }
}

#[cfg(test)]
#[path = "field3d_scene_tests.rs"]
mod tests;
