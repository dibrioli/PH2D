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

/// ⭐⭐ **Um filho ESCONDIDO não entra na caixa — mas os FILHOS dele continuam.**
///
/// ⚠️ **A segunda metade não é uma escolha, é o que o motor desenha:** `Visibility` é per-entidade
/// e *«does not propagate to descendants»* (o doc do `sim_extract::resolve_clip_grouping` diz-o
/// pelo nome e chama a propagação de *«a future wave»*). Saltar a sub-árvore daria uma caixa que
/// não envolve arte que está na tela. ⛔ **A 1.ª versão deste gate afirmava o contrário** e citava
/// a receita escondida como razão — a receita sai da tela por ser `MasterPiece`, não por
/// `Visibility`.
///
/// (Mutação: apagar a guarda de `Visibility` ⇒ RED; e voltar a saltar a sub-árvore ⇒ RED no neto.)
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
    // ⚠️ O NETO do escondido CONTA — ele desenha (o `hidden` do pai não desce até ele). O `x = 8`
    // do neto é RELATIVO ao pai escondido em `x = 2` ⇒ ele está em `x = 10`.
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(8.0, 0.0)),
        sprite([1.0, 1.0]),
        ChildOf(hidden),
    ));
    let Some(GroupBox::Union { anchor, half }) = boxed(&sim, root) else {
        panic!("nao publicou uniao");
    };
    // De `-2,5` (o filho visível) a `10,5` (o neto) ⇒ centro `4`, meia-largura `6,5`.
    assert!(
        (anchor[0] - 4.0).abs() < 1e-4 && (half[0] - 6.5).abs() < 1e-4,
        "anchor={anchor:?} half={half:?} — ou o filho escondido entrou, ou o neto dele ficou de \
         fora (e ele desenha)"
    );
}

/// ⭐ **E sem o neto, o filho escondido de facto não entra** — o controlo que separa as duas
/// metades do gate acima.
#[test]
fn a_hidden_leaf_child_is_not_in_the_box() {
    let mut sim = SimWorld::new();
    let root = empty_root(&mut sim, "Group");
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(-2.0, 0.0)),
        sprite([1.0, 1.0]),
        ChildOf(root),
    ));
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(2.0, 0.0)),
        sprite([1.0, 1.0]),
        Visibility::hidden(),
        ChildOf(root),
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
