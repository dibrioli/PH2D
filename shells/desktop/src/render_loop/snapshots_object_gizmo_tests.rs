//! ⭐⭐⭐ **A CAIXA DE OBJECTO MORRE FORA DO SELECT** — o gate da porta única do
//! [`super::publish_gizmo`] (`build_view`), a lei do ADR-0112 alargada à família que ficava de fora
//! dela: a **SPRITE**.
//!
//! ⛔⛔ **O defeito que ele existe para não deixar voltar** (report do dono, 2026-09-13:
//! *«selecionar o osso não é mais possível»*): a imagem presa ao esqueleto passou a emitir
//! instância (plano `docs/Skeleton/03`, W2), logo a `gizmo::sprite_view` passou a devolver caixa.
//! Seleccionada, o `ids::GIZMO_BBOX_INTERIOR` que o gizmo regista cobre a arte inteira; o
//! `on_canvas` do despacho pede o `hit_index` VAZIO, fica falso em cima dela, o
//! `ramo_ferramenta_vetorial` nem corre — e **nenhum osso por cima da arte que ele deforma podia
//! ser apontado ou posado**.
//!
//! ⚠️ **O oráculo é a VIEW do passe, não a `sprite_view`**: a lei que se quer é *o passe não
//! publica*, e é ela que decide se alguma alça chega ao `hit_index`. Chamar a caixa da sprite
//! directamente mediria a função que a porta gateia, nunca a porta.
//!
//! ⚠️ **Com CONTROLO nos dois lados** — sem o braço `true` este gate ficaria verde sobre uma view
//! que nunca nasce (um `present` sem espelho devolve `None` sempre), que é a forma canónica de um
//! gate deste repo medir nada.

use super::*;
use ph2d_ecs::Entity;
use ph2d_editor_core::NodeId;

/// Uma sprite na cena **e** o espelho dela no presente — que é o que a caixa de sprite lê
/// (`GlobalTransform` do presente, `Sprite::size` da cena).
fn cena_com_uma_sprite() -> (SimWorld, PresentWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Painted arm"),
            ph2d_render::Sprite::atlas(0, [2.0, 1.0], [1.0; 4]),
        ))
        .id();
    let mut present = PresentWorld::new();
    present.world_mut().spawn((
        SimRef(e),
        ph2d_ecs::GlobalTransform::from_transform(Transform::IDENTITY),
    ));
    (sim, present, e)
}

/// Corre o passe do gizmo com a selecção pedida e devolve `(view do primário, nº de extras)`.
fn passe(
    sim: &SimWorld,
    present: &mut PresentWorld,
    primario: u64,
    extras: &[u64],
    object_gizmo_on: bool,
) -> (bool, usize) {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo.selection = Some(primario);
    hero.gizmo.extra_selection = extras.to_vec();
    super::publish_gizmo(
        &mut hero,
        sim,
        present,
        &Camera2d::default(),
        WindowSize {
            width: 800,
            height: 600,
        },
        (0.0, 0.0),
        &[],
        &ph2d_vec_scene::VecScene::default(),
        object_gizmo_on,
        &ph2d_vec_scene::VecViewState::default(),
        &FlipDoc::default(),
        Vec::new(),
        None,
        false,
    );
    (hero.gizmo.view.is_some(), hero.gizmo.extra_views.len())
}

/// ⭐⭐⭐ **Fora do Select da ferramenta vectorial, uma SPRITE seleccionada não publica caixa** — e
/// no Select publica.
///
/// ⚠️ O braço `true` é o CONTROLO: ele prova que a cena da fixtura produz uma caixa, senão o braço
/// `false` estaria a afirmar o vazio.
#[test]
fn a_selected_sprite_publishes_no_object_box_outside_select() {
    let (sim, mut present, e) = cena_com_uma_sprite();
    let bits = e.to_bits();
    assert!(
        passe(&sim, &mut present, bits, &[], true).0,
        "o CONTROLO caiu: com o gizmo de objecto ligado a sprite tem de publicar caixa — sem isto \
         o braco de baixo mede o vazio"
    );
    assert!(
        !passe(&sim, &mut present, bits, &[], false).0,
        "uma sprite seleccionada publicou caixa num modo de autoria: as alcas dela registam \
         hit-rects, o `on_canvas` do despacho fica falso em cima da arte e o ramo do modo Osso nem \
         corre -- nenhum osso por cima dela pode ser apontado ou posado (report do dono, 13/09)"
    );
}

/// ⭐⭐ **E a lei vale para as EXTRAS pela mesma porta** — uma multi-selecção esconderia o defeito
/// no segundo objecto se cada ramo respondesse por si.
#[test]
fn the_extras_of_a_multi_selection_obey_the_same_door() {
    let (mut sim, mut present, a) = cena_com_uma_sprite();
    let b = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Outra"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ))
        .id();
    present.world_mut().spawn((
        SimRef(b),
        ph2d_ecs::GlobalTransform::from_transform(Transform::IDENTITY),
    ));
    let (a, b) = (a.to_bits(), b.to_bits());
    assert_eq!(
        passe(&sim, &mut present, a, &[b], true).1,
        1,
        "o CONTROLO caiu: a extra tem de publicar caixa com o gizmo de objecto ligado"
    );
    assert_eq!(
        passe(&sim, &mut present, a, &[b], false).1,
        0,
        "a extra de uma multi-seleccao publicou caixa num modo de autoria -- o mesmo ladrao de \
         cliques, no segundo objecto"
    );
}
