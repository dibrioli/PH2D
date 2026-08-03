//! Os gates do default derivado de **Resize Box**.
//!
//! O que só se pode afirmar aqui: que a regra é da HIERARQUIA (moldura, ou filho de uma) e não de
//! uma lista guardada, que a filiação conta **um** nível, e que o override vence o default nas
//! duas direcções — incluindo a que importa, *desmarcar uma moldura*.

use super::*;
use bevy_ecs::world::World;

/// Uma entidade sem nada. Devolve `(world, entidade)`.
fn lone() -> (World, Entity) {
    let mut w = World::new();
    let e = w.spawn_empty().id();
    (w, e)
}

/// **Uma moldura nasce marcada.**
#[test]
fn a_frame_resizes_its_box_by_default() {
    let (mut w, e) = lone();
    w.entity_mut(e).insert(VecFrame { clip: false });
    assert!(resizes_box(&w, e));
}

/// **Um filho de moldura nasce marcado — e um objeto solto NÃO.**
///
/// ⚠️ O segundo é o controlo, e é a metade que o Enio nomeou: *o comportamento anterior era
/// correto para objetos de game*. Sem ele o gate não distingue *"o default é a hierarquia"* de
/// *"o default é sempre verdadeiro"*.
#[test]
fn a_frames_child_resizes_its_box_and_a_loose_object_does_not() {
    let mut w = World::new();
    let frame = w.spawn(VecFrame { clip: false }).id();
    let kid = w.spawn(ChildOf(frame)).id();
    let loose = w.spawn_empty().id();
    assert!(resizes_box(&w, kid), "o filho de moldura");
    assert!(
        !resizes_box(&w, loose),
        "um objeto de game escala, como sempre"
    );
}

/// **A filiação conta UM nível.** Um neto — filho de um grupo comum que por acaso está numa
/// moldura — é geometria dentro de um grupo, e um grupo escala.
#[test]
fn the_rule_reaches_one_level_of_parenthood() {
    let mut w = World::new();
    let frame = w.spawn(VecFrame { clip: false }).id();
    let group = w.spawn(ChildOf(frame)).id();
    let grandkid = w.spawn(ChildOf(group)).id();
    assert!(resizes_box(&w, group), "o filho direto");
    assert!(!resizes_box(&w, grandkid), "o neto escala com o grupo");
}

/// **Um filho de NÃO-moldura não herda a regra** — a pergunta é sobre o pai ser moldura, não
/// sobre haver um pai.
#[test]
fn a_child_of_a_plain_group_is_not_covered() {
    let mut w = World::new();
    let group = w.spawn_empty().id();
    let kid = w.spawn(ChildOf(group)).id();
    assert!(!resizes_box(&w, kid));
}

/// **O override vence nas DUAS direcções.**
///
/// ⚠️ A direcção que importa é desmarcar uma MOLDURA: é ela que devolve ao artista o comportamento
/// de objeto de game num contentor, que foi o pedido. A outra (marcar uma forma solta) é o que
/// deixa uma forma dentro de um fluxo ter a caixa como entrada de disposição.
#[test]
fn the_override_wins_over_the_derived_default_both_ways() {
    let mut w = World::new();
    let frame = w
        .spawn((VecFrame { clip: false }, VecResizeBox(false)))
        .id();
    let loose = w.spawn(VecResizeBox(true)).id();
    assert!(!resizes_box(&w, frame), "a moldura desmarcada escala tudo");
    assert!(resizes_box(&w, loose), "a forma marcada reescreve a caixa");
}

/// **O default é o que a ausência produz** — e é isso que torna o destacamento honesto: um
/// componente removido no valor de fábrica deixa a resposta exatamente onde estava.
#[test]
fn removing_the_override_returns_to_the_derived_answer() {
    let mut w = World::new();
    let frame = w
        .spawn((VecFrame { clip: false }, VecResizeBox(false)))
        .id();
    assert!(!resizes_box(&w, frame));
    w.entity_mut(frame).remove::<VecResizeBox>();
    assert!(resizes_box(&w, frame), "voltou ao default derivado");
}

/// **Uma INSTÂNCIA escala a pose — mesmo dentro de uma moldura** (plano UI/UX W5).
///
/// ⚠️ Nasceu porque a isenção shipou **sem gate**: a mutação que a remove passava em toda a suíte,
/// e o preço é do artista — a alça reescreveria o retângulo-SUPORTE da cópia (o número que ninguém
/// olha) e o desenho, que é derivado, ficaria exatamente onde estava. Um arrasto que não faz nada.
#[test]
fn an_instance_scales_its_pose_even_inside_a_frame() {
    let mut w = World::new();
    let frame = w.spawn(VecFrame { clip: false }).id();
    let plain_kid = w.spawn(crate::ChildOf(frame)).id();
    let instance_kid = w
        .spawn((crate::ChildOf(frame), crate::VecInstance::new(1)))
        .id();
    assert!(
        resizes_box(&w, plain_kid),
        "um filho comum de moldura reescreve a caixa (o controle)"
    );
    assert!(
        !resizes_box(&w, instance_kid),
        "a instância reescreveria o suporte e o desenho não se mexeria"
    );
    // ⚠️ E o override do artista continua a vencer: a isenção é um DEFAULT, não uma proibição.
    w.entity_mut(instance_kid).insert(VecResizeBox(true));
    assert!(resizes_box(&w, instance_kid));
}
