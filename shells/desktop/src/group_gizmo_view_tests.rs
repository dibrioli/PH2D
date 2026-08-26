//! Os gates do gizmo de grupo/vazio (report do Enio, 2026-08-26).
//!
//! ⚠️ **O oráculo é a CAIXA, e nunca «a função devolveu `Some`»**: um gizmo publicado com
//! meia-extensão zero passa em qualquer teste de presença e é exatamente o defeito reportado.

use super::{EMPTY_HALF_PX, GroupBox, box_of};
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, Visibility};
use ph2d_flip::FlipDoc;
use ph2d_vec_scene::VecScene;

/// A régua do projeto nos testes — um metro tem 100 px de arte.
const PPM: f32 = 100.0;

fn sprite(size: [f32; 2]) -> ph2d_render::Sprite {
    ph2d_render::Sprite::atlas(0, size, [1.0; 4])
}

fn empty_root(sim: &mut SimWorld, name: &str) -> Entity {
    sim.world_mut()
        .spawn((Transform::IDENTITY, Name::new(name)))
        .id()
}

fn boxed(sim: &SimWorld, e: Entity) -> Option<GroupBox> {
    box_of(sim, &VecScene::new(), &FlipDoc::default(), e, PPM)
}

/// ⭐ **O report, na sua forma mais curta: um objeto vazio é AGARRÁVEL.**
///
/// ⚠️ A metade que interessa é a **extensão**: `Some(GroupBox)` com meia-extensão zero é um gizmo
/// que se publica e não se pega — as oito alças caem no mesmo pixel.
///
/// (Mutação: devolver `half: [0.0, 0.0]` no ramo `Empty` ⇒ RED.)
#[test]
fn an_empty_object_gets_a_box_wide_enough_to_grab() {
    let mut sim = SimWorld::new();
    let e = empty_root(&mut sim, "Object");
    let Some(GroupBox::Empty { half }) = boxed(&sim, e) else {
        panic!("um objeto vazio nao publicou o marcador do vazio");
    };
    assert!(
        half[0] > 0.0 && half[1] > 0.0,
        "marcador do vazio com meia-extensao {half:?} — o gizmo nasce colapsado"
    );
}

/// ⭐⭐ **A largura do marcador é DERIVADA da alça** — e o recurso é ela.
///
/// A caixa carrega oito alças de `HANDLE_SIZE_PX`; com meia-extensão de duas ela tem quatro de
/// largura, que é a menor em que a quina e o meio da aresta não se sobrepõem.
///
/// ⚠️ **Os dois lados:** grande de mais também é um defeito (um objeto vazio dominaria a cena), e
/// por isso há tecto. (Mutação: `EMPTY_HALF_PX = HANDLE_SIZE_PX * 0.4` ⇒ RED no piso.)
#[test]
fn the_empty_marker_is_wider_than_the_handles_it_carries() {
    let width_px = 2.0 * EMPTY_HALF_PX;
    let handle = ph2d_editor::HANDLE_SIZE_PX;
    assert!(
        width_px >= 4.0 * handle,
        "a caixa do vazio tem {width_px} px para alcas de {handle} px — a quina e o meio da \
         aresta sobrepoem-se"
    );
    assert!(
        width_px <= 8.0 * handle,
        "o marcador do vazio ({width_px} px) e' maior que oito alcas — um objeto que nao desenha \
         nada estaria a dominar a cena"
    );
}

/// ⭐⭐ **A caixa de um grupo é a UNIÃO dos filhos** — o pedido literal do report.
///
/// Dois quadrados de 1 m, um em `x = -2` e outro em `x = +2` ⇒ a caixa vai de `-2,5` a `+2,5`.
///
/// (Mutação: unir só o primeiro filho ⇒ RED com a meia-largura errada.)
#[test]
fn a_group_box_is_the_union_of_its_visible_children() {
    let mut sim = SimWorld::new();
    let root = empty_root(&mut sim, "Group");
    for x in [-2.0f32, 2.0] {
        sim.world_mut().spawn((
            Transform::from_translation(ph2d_core::Vec2::new(x, 0.0)),
            sprite([1.0, 1.0]),
            ChildOf(root),
        ));
    }
    let Some(GroupBox::Union { anchor, half }) = boxed(&sim, root) else {
        panic!("um grupo com dois filhos nao publicou uniao");
    };
    assert!(
        (anchor[0]).abs() < 1e-5,
        "a uniao nao esta' centrada: {anchor:?}"
    );
    assert!(
        (half[0] - 2.5).abs() < 1e-5 && (half[1] - 0.5).abs() < 1e-5,
        "a caixa do grupo mede {half:?} e devia medir [2.5, 0.5]"
    );
}

/// ⭐⭐ **Um filho ESCONDIDO não entra na caixa — e a sub-árvore dele também não.**
///
/// ⚠️ Isto não é asseio: a **receita** de um componente é escondida de propósito (F4.5), e uma
/// caixa que a envolvesse mediria um objeto que não está na tela.
///
/// (Mutação: apagar a guarda de `Visibility` ⇒ RED, a caixa volta a `2.5`.)
#[test]
fn a_hidden_child_is_not_in_the_box() {
    let mut sim = SimWorld::new();
    let root = empty_root(&mut sim, "Group");
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(-2.0, 0.0)),
        sprite([1.0, 1.0]),
        ChildOf(root),
    ));
    let hidden = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(2.0, 0.0)),
            sprite([1.0, 1.0]),
            Visibility::hidden(),
            ChildOf(root),
        ))
        .id();
    // o NETO do escondido também não conta
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(8.0, 0.0)),
        sprite([1.0, 1.0]),
        ChildOf(hidden),
    ));
    let Some(GroupBox::Union { anchor, half }) = boxed(&sim, root) else {
        panic!("nao publicou uniao");
    };
    assert!(
        (anchor[0] + 2.0).abs() < 1e-5 && (half[0] - 0.5).abs() < 1e-5,
        "o filho escondido entrou na caixa: anchor={anchor:?} half={half:?}"
    );
}

/// ⭐ **A caixa vive no espaço do PAI, não no mundo.**
///
/// ⚠️ [`crate::vec_gizmo_view::gizmo_view_from`] aplica a pose do pai à caixa que recebe; uma
/// caixa já em mundo seria transformada **duas vezes** e o gizmo derivaria do objeto.
///
/// (Mutação: semear a pilha com `xform_of_transform(world_transform(root))` ⇒ RED.)
#[test]
fn the_box_is_measured_in_the_parents_frame() {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(100.0, -50.0)),
            Name::new("Group"),
        ))
        .id();
    sim.world_mut()
        .spawn((Transform::IDENTITY, sprite([1.0, 1.0]), ChildOf(root)));
    let Some(GroupBox::Union { anchor, half }) = boxed(&sim, root) else {
        panic!("nao publicou uniao");
    };
    assert!(
        anchor[0].abs() < 1e-4 && anchor[1].abs() < 1e-4,
        "a caixa saiu em coordenadas de MUNDO: {anchor:?}"
    );
    assert!((half[0] - 0.5).abs() < 1e-5, "meia-extensao {half:?}");
}

/// ⭐ **Um filho GIRADO é envolvido pelos QUATRO cantos.**
///
/// Um quadrado de 1 m a 45° mede `√2` na diagonal ⇒ a caixa do pai tem meia-extensão `√2/2`.
/// Unir só `(mín, máx)` daria `0,5` — a caixa cortaria as quinas do filho.
///
/// (Mutação: unir só dois cantos ⇒ RED.)
#[test]
fn a_rotated_child_is_wrapped_by_its_four_corners() {
    let mut sim = SimWorld::new();
    let root = empty_root(&mut sim, "Group");
    sim.world_mut().spawn((
        Transform {
            rotation: std::f32::consts::FRAC_PI_4,
            ..Transform::IDENTITY
        },
        sprite([1.0, 1.0]),
        ChildOf(root),
    ));
    let Some(GroupBox::Union { half, .. }) = boxed(&sim, root) else {
        panic!("nao publicou uniao");
    };
    let want = std::f32::consts::SQRT_2 * 0.5;
    assert!(
        (half[0] - want).abs() < 1e-4 && (half[1] - want).abs() < 1e-4,
        "a caixa mede {half:?} e devia medir [{want}, {want}] — as quinas do filho girado ficaram \
         de fora"
    );
}

/// ⛔ **Uma JUNTA não ganha caixa** — ela já tem os dots, e a caixa engoliria o clique neles.
///
/// (Mutação: apagar a guarda `publishes_its_own_handles` ⇒ RED.)
#[test]
fn a_joint_publishes_no_box_so_its_dots_keep_the_click() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Pin"),
            ph2d_physics_ecs::PhysicsJoint::default(),
        ))
        .id();
    assert!(
        boxed(&sim, e).is_none(),
        "a junta publicou uma caixa — o interior dela regista Translate e rouba o clique nos dots"
    );
    // O controlo POSITIVO: sem o componente, a mesma entidade ganha o marcador.
    sim.world_mut()
        .entity_mut(e)
        .remove::<ph2d_physics_ecs::PhysicsJoint>();
    assert!(
        matches!(boxed(&sim, e), Some(GroupBox::Empty { .. })),
        "sem a junta a entidade continuou sem caixa — o gate estaria verde por outra razao"
    );
}

/// ⛔ **E uma peça de MODELAGEM 3D também não** — ela tem o gizmo do MODEL, e a pose dela nem é o
/// `Transform` da casa: a caixa sairia na origem do mundo.
///
/// (Mutação: tirar `FieldNode` da lista ⇒ RED.)
#[test]
fn a_modelling_part_publishes_no_box_either() {
    let mut sim = SimWorld::new();
    for marker in ["object", "node"] {
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new(marker)))
            .id();
        if marker == "object" {
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_field_ecs::FieldObject);
        } else {
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_field_ecs::FieldNode {
                    shape: ph2d_field::NodeShape::Leaf(ph2d_field::Primitive::Sphere {
                        radius: 1.0,
                    }),
                });
        }
        assert!(
            boxed(&sim, e).is_none(),
            "a peca de modelagem ({marker}) ganhou uma segunda caixa"
        );
    }
}
