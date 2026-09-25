//! **A shell publica QUEM é cada objeto animado** (FASE C.3) — os gates do
//! `timeline_persist::publish_object_names`.
//!
//! ⚠️ Irmão de [`super::tests`] pelo teto de 600 LOC da shell, e o corte é por
//! ASSUNTO: lá fica o que ATRAVESSA o ficheiro do projeto; aqui, o que a shell publica ao painel
//! por quadro. ⛔ Não devolva um teste ao irmão — o teto volta a estourar no gate seguinte.

use super::publish_object_names;
use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_timeline::PropKind;

/// O mesmo mundo mínimo do irmão — duplicado de propósito: um helper partilhado entre dois
/// ficheiros de teste teria de viver num terceiro, e ele são **doze linhas**.
fn world_with(names: &[&str]) -> (SimWorld, Vec<u64>) {
    let mut sim = SimWorld::new();
    let bits = names
        .iter()
        .map(|n| {
            sim.world_mut()
                .spawn((ph2d_ecs::Transform::IDENTITY, Name::new(*n)))
                .id()
                .to_bits()
        })
        .collect();
    (sim, bits)
}

/// Um snapshot com uma track por entidade dada (só o que o `publish_object_names` lê).
fn view_of(entities: &[u64]) -> ph2d_timeline::TimelineViewSnapshot {
    let tracks = entities
        .iter()
        .map(|bits| ph2d_timeline::TrackView {
            target: ph2d_anim::AnimTarget::new(*bits),
            prop: PropKind::TranslationX,
            entity: *bits,
            missing: false,
            keys: Vec::new(),
            buffer_ghost: None,
            pre: ph2d_anim::Extrap::Hold,
            post: ph2d_anim::Extrap::Hold,
            expr: None,
        })
        .collect();
    ph2d_timeline::TimelineViewSnapshot {
        tracks,
        ..Default::default()
    }
}

/// **A shell publica o nome de quem tem track — e SÓ de quem tem.**
///
/// A metade do escopo é o ponto: a pergunta é sobre as rows que vão ser pintadas, e uma
/// cena de quinhentos objetos com três animados publica três nomes.
///
/// **Mutação que deve sangrar:** varrer o mundo em vez das tracks (o `Extra` entra no mapa).
#[test]
fn the_shell_publishes_a_name_for_every_animated_object_and_no_other() {
    let (sim, bits) = world_with(&["Ball", "Box", "Extra"]);
    let mut view = view_of(&bits[..2]);
    publish_object_names(&mut view, sim.world());

    assert_eq!(view.object_name(bits[0]), Some("Ball"));
    assert_eq!(view.object_name(bits[1]), Some("Box"));
    assert_eq!(
        view.object_names.len(),
        2,
        "o objeto sem track não é publicado: {:?}",
        view.object_names
    );
    assert_eq!(
        view.object_name(bits[2]),
        None,
        "e perguntar por ele devolve None, que é o que faz o rótulo cair no id curto"
    );
}

/// **Um objeto que sai das tracks sai do mapa.**
///
/// ⚠️ Não é higiene: os bits de entidade são RECICLADOS pelo bevy, então um nome que
/// sobrevive à track dele acabaria rotulando outro objeto — a mesma armadilha que faz o
/// load de projeto DESTACAR toda binding em vez de confiar nos bits salvos.
///
/// **Mutação que deve sangrar:** tirar o `retain` do `publish_object_names`.
#[test]
fn a_name_does_not_outlive_the_track_it_was_published_for() {
    let (sim, bits) = world_with(&["Ball", "Box"]);
    let mut view = view_of(&bits);
    publish_object_names(&mut view, sim.world());
    assert_eq!(view.object_names.len(), 2, "premissa: os dois publicados");

    // O Box perde a track (deletado, ou a track foi removida).
    view.tracks.retain(|t| t.entity == bits[0]);
    publish_object_names(&mut view, sim.world());
    assert_eq!(
        view.object_names.len(),
        1,
        "o nome do Box tem de sair com a track dele: {:?}",
        view.object_names
    );
    assert_eq!(view.object_name(bits[0]), Some("Ball"));
}

/// **Renomear o objeto muda o que o painel mostra, no mesmo frame.**
///
/// O rótulo é derivado a cada frame justamente por isto; o mapa tem de acompanhar, senão
/// a derivação lê um nome congelado e a cura vira um cache velho.
///
/// **Mutação que deve sangrar:** o `if slot != name.as_str()` virar `if slot.is_empty()`
/// (a reutilização da `String` passaria a ser um cache que nunca invalida).
#[test]
fn renaming_the_object_renames_its_rows() {
    let (mut sim, bits) = world_with(&["Ball"]);
    let mut view = view_of(&bits);
    publish_object_names(&mut view, sim.world());
    assert_eq!(view.object_name(bits[0]), Some("Ball"));

    let e = Entity::try_from_bits(bits[0]).expect("bits vivos");
    *sim.world_mut().get_mut::<Name>(e).expect("tem Name") = Name::new("Bola");
    publish_object_names(&mut view, sim.world());
    assert_eq!(
        view.object_name(bits[0]),
        Some("Bola"),
        "o nome publicado é o do mundo AGORA, não o do frame em que a track nasceu"
    );
}

/// **Um objeto SEM `Name` não publica nada — e um que PERDE o nome perde a entrada.**
///
/// A segunda metade é a que quase escapou: sem o `remove` no braço `else`, um objeto que
/// perde o `Name` continuaria rotulado com o nome que tinha.
#[test]
fn an_object_without_a_name_publishes_none() {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("Ghost")))
        .id()
        .to_bits();
    let mut view = view_of(&[bits]);
    publish_object_names(&mut view, sim.world());
    assert_eq!(view.object_name(bits), Some("Ghost"), "premissa");

    let e = Entity::try_from_bits(bits).expect("bits vivos");
    sim.world_mut().entity_mut(e).remove::<Name>();
    publish_object_names(&mut view, sim.world());
    assert_eq!(
        view.object_name(bits),
        None,
        "sem Name, o rótulo cai no id curto em vez de mostrar um nome que já não existe"
    );
}
