//! Os gates do gizmo de grupo/vazio (report do Enio, 2026-08-26).
//!
//! ⚠️ **O oráculo é a CAIXA, e nunca «a função devolveu `Some`»**: um gizmo publicado com
//! meia-extensão zero passa em qualquer teste de presença e é exatamente o defeito reportado.

use super::EMPTY_HALF_PX;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, Visibility};
use ph2d_editor::GizmoView;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

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

/// A `GizmoView` que o passe publicaria, com uma câmera de teste.
fn boxed(sim: &SimWorld, e: Entity) -> Option<GizmoView> {
    super::view(
        sim,
        e,
        &Camera2d::default(),
        WindowSize {
            width: 800,
            height: 600,
        },
        (0.0, 0.0),
        false,
        PPM,
    )
}

/// A meia-extensão de MUNDO da caixa publicada.
fn half_of(v: &GizmoView) -> [f32; 2] {
    [
        (v.bbox_max_world[0] - v.bbox_min_world[0]) * 0.5,
        (v.bbox_max_world[1] - v.bbox_min_world[1]) * 0.5,
    ]
}

/// ⭐ **O report, na sua forma mais curta: um objeto vazio é AGARRÁVEL.**
///
/// ⚠️ A metade que interessa é a **extensão**: uma `GizmoView` com meia-extensão zero passa em
/// qualquer teste de presença e não se pega — as oito alças caem no mesmo pixel.
///
/// (Mutação: publicar `half = [0.0, 0.0]` ⇒ RED.)
#[test]
fn an_empty_object_gets_a_box_wide_enough_to_grab() {
    let mut sim = SimWorld::new();
    let e = empty_root(&mut sim, "Object");
    let v = boxed(&sim, e).expect("um objeto vazio nao publicou gizmo");
    let half = half_of(&v);
    assert!(
        half[0] > 0.0 && half[1] > 0.0,
        "marcador do vazio com meia-extensao {half:?} — o gizmo nasce colapsado"
    );
}

/// ⭐⭐⭐ **A caixa é a DELE, e ter filhos não a muda** (Enio, 2026-08-26, 3.ª volta):
///
/// > *«O objeto vazio deve permanecer com seu gizmo original e não se utilizar do gizmo dos
/// > filhos.»*
///
/// ⛔ A união dos filhos foi construída, medida e **rejeitada por veredito de produto** (a árvore
/// vive em `828bc88f4`). O gate mede o FIM: a caixa não muda quando nasce um filho longe.
///
/// (Mutação: unir a caixa dos filhos ⇒ RED.)
#[test]
fn a_group_keeps_its_own_box_and_does_not_borrow_the_childrens() {
    let mut sim = SimWorld::new();
    let root = empty_root(&mut sim, "Group");
    let alone = half_of(&boxed(&sim, root).expect("gizmo do vazio"));
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(50.0, 0.0)),
        sprite([9.0, 9.0]),
        ChildOf(root),
    ));
    let with_child = half_of(&boxed(&sim, root).expect("gizmo do grupo"));
    assert_eq!(
        alone, with_child,
        "a caixa do objeto vazio mudou por causa de um filho — ela passou a ser a dos filhos"
    );
    // ⚠️ E a caixa continua CENTRADA na origem dele, e não no centro do que ele contém — é esta a
    // metade que distingue as duas versões (a união deslocava a caixa para o centroide dos filhos,
    // deixando o pivô para trás).
    let v = boxed(&sim, root).expect("gizmo do grupo");
    let cx = (v.bbox_min_world[0] + v.bbox_max_world[0]) * 0.5;
    let cy = (v.bbox_min_world[1] + v.bbox_max_world[1]) * 0.5;
    assert!(
        (cx - v.pivot_world[0]).abs() < 1e-4 && (cy - v.pivot_world[1]).abs() < 1e-4,
        "a caixa ({cx}, {cy}) largou o pivo ({:?}) — ela foi para o centro dos filhos",
        v.pivot_world
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
        boxed(&sim, e).is_some(),
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

// ───────────── O anel: o report de 2026-08-26 (2.ª volta) ─────────────

/// ⭐⭐⭐ **O anel NÃO segue a seleção, e ter filhos não o apaga.**
///
/// > *«Se desseleciono o objeto vazio, o círculo some. O círculo só pode sumir no runtime.»*
/// > *«O gizmo do objeto vazio deve existir mesmo quando ele ganha filhos.»*
///
/// ⚠️ O censo **não recebe a seleção** — é essa ausência que é a cura, e por isso o gate mede a
/// LISTA e não um desenho.
///
/// (Mutação: `is_empty_object` devolver `false` quando há `Children` ⇒ RED no grupo.)
#[test]
fn every_empty_object_is_listed_children_or_not() {
    let mut sim = SimWorld::new();
    let lonely = empty_root(&mut sim, "Lonely");
    let group = empty_root(&mut sim, "Group");
    sim.world_mut()
        .spawn((Transform::IDENTITY, sprite([1.0, 1.0]), ChildOf(group)));
    let listed = super::empty_objects(&sim);
    assert!(
        listed.contains(&lonely) && listed.contains(&group),
        "o censo devolveu {listed:?} — faltou o vazio ou o grupo"
    );
}

/// ⛔ **Uma SPRITE não ganha anel** (Enio: *«se eu crio diretamente uma sprite não preciso do
/// círculo»*), e uma peça de RECEITA também não — ela não está na cena.
///
/// (Mutação: tirar o `MasterPiece` de `is_empty_object` ⇒ RED na receita.)
#[test]
fn what_draws_itself_and_what_is_not_on_the_canvas_get_no_ring() {
    let mut sim = SimWorld::new();
    let with_art = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Sprite"), sprite([1.0, 1.0])))
        .id();
    let recipe = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Recipe"),
            ph2d_ecs::MasterRoot,
        ))
        .id();
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let blind = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Hidden"),
            Visibility::hidden(),
        ))
        .id();
    let listed = super::empty_objects(&sim);
    for (what, e) in [
        ("uma sprite", with_art),
        ("a receita", recipe),
        ("um objeto com o olho fechado", blind),
    ] {
        assert!(
            !listed.contains(&e),
            "{what} ganhou anel — o censo devolveu {listed:?}"
        );
    }
    // Controlo POSITIVO: sem o `MasterRoot`, a mesma entidade entra.
    sim.world_mut()
        .entity_mut(recipe)
        .remove::<ph2d_ecs::MasterRoot>();
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    assert!(
        super::empty_objects(&sim).contains(&recipe),
        "sem a marca de receita a entidade continuou sem anel — o gate estaria verde por outra razao"
    );
}

/// ⭐⭐⭐ **A receita que é um GRUPO recupera anel, dedo e caixa enquanto está a ser editada.**
///
/// ⛔ Era este o defeito §1.5 da auditoria de 2026-08-27, e é a **outra metade** da lei que o
/// `off_canvas` recebeu na F4.6: `is_empty_object` conhecia o `MasterPiece` e não o
/// `MasterEditing`. A raiz de **toda** receita nascida de *Make Component* sobre um grupo ou um rig
/// é `Transform` + `Name` + `MasterRoot`, sem `Sprite` ⇒ ela caía neste ficheiro, e ficava sem
/// anel, sem caixa e impegável **no único estado em que está na tela**. Mover a receita inteira era
/// inalcançável por gesto de canvas.
///
/// ⚠️ **Os TRÊS consumidores no mesmo gate, de propósito** — a tinta ([`super::empty_objects`]), o
/// dedo ([`super::pick_empty_at_world`]) e a caixa ([`super::view`]). Eles caem juntos porque a
/// pergunta é uma só, e um gate sobre um deles deixaria os outros dois a apodrecer.
///
/// ⚠️ E o gesto é o de verdade: quem acende é `master_editing::mark`, a mesma função que o quadro
/// chama — não um `insert(MasterEditing)` à mão, que mede a marca em vez do fim.
///
/// (Mutação: pôr `is_empty_object` a testar `MasterPiece.is_none()` outra vez ⇒ RED nas três.)
#[test]
fn the_recipe_being_edited_gets_its_ring_its_finger_and_its_box_back() {
    let mut sim = SimWorld::new();
    let recipe = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Rig"), ph2d_ecs::MasterRoot))
        .id();
    // A forma real: a raiz não desenha nada, os filhos é que têm arte.
    sim.world_mut()
        .spawn((Transform::IDENTITY, sprite([1.0, 1.0]), ChildOf(recipe)));
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    // Controlo NEGATIVO: sem ninguém a editar, a receita não está na cena e não tem anel.
    crate::render_loop::master_editing::mark(&mut sim, None::<u64>);
    assert!(
        !super::empty_objects(&sim).contains(&recipe),
        "a receita ganhou anel sem ninguem a editar — o gate mediria o estado errado"
    );

    // O gesto: escolher a linha dela na Hierarquia.
    crate::render_loop::master_editing::mark(&mut sim, Some(recipe.to_bits()));
    assert!(
        super::empty_objects(&sim).contains(&recipe),
        "a receita editada continua sem anel — ela nao tem UM pixel no canvas"
    );
    assert!(
        super::pick_empty_at_world(&sim, [0.0, 0.0], PPM).contains(&recipe.to_bits()),
        "o centro da receita editada nao pega — mover a receita inteira e' inalcancavel por gesto"
    );
    let half = half_of(&boxed(&sim, recipe).expect("a receita editada nao publica GizmoView"));
    assert!(
        half[0] > 0.0 && half[1] > 0.0,
        "a caixa da receita editada saiu com meia-extensao {half:?} — um gizmo colapsado"
    );
}

/// ⭐⭐ **O anel PEGA — é isso que o torna um gizmo e não um desenho.**
///
/// > *«Não consigo transformar o objeto total a partir do centro do objeto vazio.»*
///
/// ⚠️ Os dois lados: o centro pega, e um ponto **fora** do disco não — senão o objeto vazio
/// roubaria o clique de tudo o que estivesse por perto.
///
/// (Mutação: `pick_empty_at_world` devolver sempre a lista inteira ⇒ RED no ponto de fora.)
#[test]
fn the_ring_takes_the_click_at_the_centre_and_not_beyond_it() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(3.0, -1.0)),
            Name::new("Group"),
        ))
        .id();
    let r = super::marker_world_radius(&sim, e, PPM);
    assert!(r > 0.0, "raio do marcador nao positivo: {r}");
    assert_eq!(
        super::pick_empty_at_world(&sim, [3.0, -1.0], PPM),
        vec![e.to_bits()],
        "o centro do anel nao pegou"
    );
    assert!(
        super::pick_empty_at_world(&sim, [3.0 + r * 1.5, -1.0], PPM).is_empty(),
        "o anel pegou um ponto a 1,5 raio — ele rouba o clique dos vizinhos"
    );
}

/// ⚠️ **A escala entra pela MÉDIA GEOMÉTRICA** (`√|sx·sy|`), a mesma lei do traço vetorial sob
/// escala não-uniforme — para escala uniforme é a própria escala, e é invariante à rotação.
///
/// Um anel que fosse elipse precisaria de um teste de elipse no dedo, e a tinta e o dedo
/// divergiriam no dia em que um dos dois esquecesse.
///
/// (Mutação: usar `scale.x` sozinho ⇒ RED.)
#[test]
fn the_ring_scales_by_the_geometric_mean() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                scale: ph2d_core::Vec2::new(4.0, 1.0),
                ..Transform::IDENTITY
            },
            Name::new("Squashed"),
        ))
        .id();
    let base = EMPTY_HALF_PX / PPM;
    let got = super::marker_world_radius(&sim, e, PPM);
    assert!(
        (got - base * 2.0).abs() < 1e-5,
        "raio {got} — a media geometrica de (4, 1) e' 2, entao esperava-se {}",
        base * 2.0
    );
}
