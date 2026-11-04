//! ⭐ **C1 — a edição de um cartão PRESO, do dreno ao mundo e de volta** (`docs/CartoesUI/09`): prende-se
//! a A com B+C escolhidos; Z, modo, olho, opacidade e mistura caem em A, em B e C não; desfazer repõe A
//! (o passo é a fotografia do documento: a seleção não entra nela). O gesto real está em
//! `ph2d-panel-registry-init/tests/it/o_alfinete_dos_cartoes_c1.rs`.

use super::DrainOut;
use crate::undo::ProjectState;
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecPathRef, ZIndexOverride};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetStore, loose_rect_of};
use ph2d_editor_core::screens::hero::{BlendFieldEdit, InspectorVisibilityInfo, OrderingFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_render::register_render_components;
use ph2d_vec_edit::PenTool;
use ph2d_vec_scene::{BlendMode, Opacity, VecScene, rectangle};

fn captura(sim: &mut SimWorld, vec: &VecScene, reg: &ComponentRegistry) -> ProjectState {
    ProjectState::capture(
        &ph2d_preview_drive::PreviewDrive::default(),
        sim,
        vec,
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    )
}

/// O store com o solto `SEC` preso a `a` e a última pressão sobre ele.
fn com_gesto_no_preso(a: u64) -> WidgetStore {
    const SEC: ph2d_editor_core::NodeId = ph2d_editor_core::NodeId(0xC1);
    let (mut store, painel) = (
        WidgetStore::with_capacity(8),
        ph2d_editor_core::NodeId(0xC2),
    );
    store.detach_card(SEC, painel, (0.0, 0.0), 100.0);
    store.set_card_subject(painel, Some((a, "A".to_owned())));
    store.toggle_loose_pin(SEC);
    store.set_panel_rect(loose_rect_of(SEC), Rect::new(0.0, 0.0, 100.0, 100.0));
    store.note_card_press(10.0, 10.0);
    store
}

/// ⭐ C2: `Mix` no Material tira o componente, e sem ele o bloco deixa de se publicar: o solto preso diz
/// «Not available», como o da lista some.
#[test]
fn mix_on_a_pinned_material_takes_the_component_and_the_block_is_no_longer_published() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_render_components(&mut reg);
    let mut sim = SimWorld::new();
    let a = (Transform::default(), ph2d_ecs::BlendMode::Add);
    let a = sim.world_mut().spawn(a).id().to_bits();
    assert!(ph2d_inspector_ordering::build_blend_info(sim.world(), a, &[a], 1).is_some());
    let fila = EditorCommandQueue::default();
    ph2d_inspector_ordering::apply_blend_edit(&sim, a, BlendFieldEdit::Blend(0), &fila, &reg);
    apply_editor_commands(sim.world_mut(), &fila, &reg).expect("os comandos");
    assert!(ph2d_inspector_ordering::build_blend_info(sim.world(), a, &[a], 1).is_none());
}

/// ⭐ C2: Fill e Stroke pelas mesmas portas: as tintas de A mudam, as de B não; o passo de desfazer repõe A.
#[test]
fn a_pinned_fill_and_stroke_edit_lands_on_its_object_only_and_the_undo_step_restores_it() {
    use ph2d_app_vec::paint_stack::{self, StackVerb};
    use ph2d_app_vec::stroke_present::BaseVerb;
    use ph2d_vec_scene::{Paint, PaintEntry, Rgba8, StrokeSpec, VecPath};
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_render_components(&mut reg);
    let (mut sim, mut vec) = (SimWorld::new(), VecScene::new());
    let mut objecto = |nome: &str, sim: &mut SimWorld| {
        let mut p = rectangle([0.0, 0.0], [2.0, 2.0]);
        p.fill = Some(Paint::Solid(Rgba8::new(10, 20, 30, 255)));
        p.stroke = Some(StrokeSpec::new(Rgba8::new(40, 50, 60, 255), 1.0));
        p.paints.push(PaintEntry::fill(Paint::Solid(Rgba8::new(1, 1, 1, 255))));
        let id = vec.push_path(p);
        let e = (Transform::default(), Name::new(nome), VecPathRef(id));
        (sim.world_mut().spawn(e).id().to_bits(), id)
    };
    let ((a, pa), (_, pb)) = (objecto("A", &mut sim), objecto("B", &mut sim));
    let de_antes = (
        vec.path(pa).cloned().expect("A"),
        vec.path(pb).cloned().expect("B"),
    );
    let antes = captura(&mut sim, &vec, &reg);
    let store = com_gesto_no_preso(a);
    let mut pen = PenTool::default();
    pen.select_many(&[pb]);
    assert_eq!(paint_stack::alvo_do_gesto(&store, &sim), Some(pa));

    let alvos = paint_stack::alvos(&store, &pen, &sim);
    paint_stack::apply(&mut vec, &alvos, StackVerb::Eye(0));
    paint_stack::apply(&mut vec, &alvos, StackVerb::AddStroke);
    ph2d_app_vec_preso::apply_no_alvo(&mut vec, &pen, &store, &sim, BaseVerb::FillEye);
    let forma = vec.path_mut(pa).expect("A");
    assert!(ph2d_app_vec_preso::pick_base(
        forma,
        false,
        Rgba8::new(9, 9, 9, 255),
        true
    ));

    let (a2, b2): (&VecPath, &VecPath) = (vec.path(pa).expect("A"), vec.path(pb).expect("B"));
    assert!(!a2.paints[0].enabled && a2.paints.len() == 2 && a2.fill_is_hidden());
    assert_eq!(*b2, de_antes.1, "B não recebe");
    assert!(captura(&mut sim, &vec, &reg) != antes, "o documento mudou");
    let (v2, ..) = antes.restore(&mut sim, &reg);
    assert_eq!(*v2.path(pa).expect("A"), de_antes.0, "o desfazer não repôs A");
    assert_eq!(*v2.path(pb).expect("B"), de_antes.1);
}

#[test]
fn a_pinned_card_edit_lands_on_its_object_only_and_the_undo_step_restores_it() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_render_components(&mut reg);
    let (mut sim, mut vec) = (SimWorld::new(), VecScene::new());
    let mut objecto = |nome: &str, sim: &mut SimWorld| {
        let id = vec.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
        let e = (Transform::default(), Name::new(nome), VecPathRef(id));
        (sim.world_mut().spawn(e).id().to_bits(), id)
    };
    let ((a, pa), (b, pb), (c, _)) = (
        objecto("A", &mut sim),
        objecto("B", &mut sim),
        objecto("C", &mut sim),
    );
    let antes = captura(&mut sim, &vec, &reg);
    let store = com_gesto_no_preso(a);

    // O dreno: com a seleção em B+C e o gesto no preso, nada se espalha.
    let mut pd = DrainOut {
        inspector_selection: store.card_fan_out(2, [b, c].into_iter()),
        ..DrainOut::default()
    };
    let mut app = crate::App::new();
    for acao in [
        EditorAction::InspectorOrderingEdit {
            entity_bits: a,
            edit: OrderingFieldEdit::ZIndex(Some(7)),
        },
        EditorAction::InspectorBlendEdit {
            entity_bits: a,
            edit: BlendFieldEdit::Blend(3),
        },
        EditorAction::InspectorVisibilityEdit(InspectorVisibilityInfo {
            entity_bits: a,
            visible: false,
            mixed: false,
        }),
    ] {
        assert!(app.fase_bus_inspector_sections(acao, &mut pd).is_none());
    }
    let fila = EditorCommandQueue::default();
    for (e, ed) in std::mem::take(&mut pd.ordering_edits) {
        ph2d_inspector_ordering::apply_ordering_edit(&sim, e, ed, &fila, &reg);
    }
    for (e, ed) in std::mem::take(&mut pd.blend_edits) {
        ph2d_inspector_ordering::apply_blend_edit(&sim, e, ed, &fila, &reg);
    }
    apply_editor_commands(sim.world_mut(), &fila, &reg).expect("os comandos");
    assert_eq!(pd.visibility_edits, [(a, false)]);

    // A aparência: as formas do pedido são a do preso, nunca a seleção da caneta.
    let alvos = ph2d_app_vec::paint_stack::alvos(&store, &PenTool::default(), &sim);
    assert_eq!(alvos, [pa]);
    ph2d_app_vec::appearance::set_opacity(&mut vec, &alvos, 0.25);
    ph2d_app_vec::appearance::set_blend(&mut vec, &alvos, BlendMode::Multiply.to_u8());

    let z = |sim: &SimWorld, bits: u64| {
        sim.world()
            .get::<ZIndexOverride>(Entity::from_bits(bits))
            .map(|z| z.0)
    };
    assert_eq!((z(&sim, a), z(&sim, b), z(&sim, c)), (Some(7), None, None));
    let modo = |sim: &SimWorld, bits: u64| {
        sim.world()
            .get::<ph2d_ecs::BlendMode>(Entity::from_bits(bits))
            .map(|m| m.tag())
    };
    assert_eq!((modo(&sim, a), modo(&sim, b)), (Some(3), None));
    let op = |v: &VecScene, id| v.path(id).expect("a forma").opacity;
    assert_eq!(op(&vec, pa), Opacity::new(0.25));
    assert_eq!(op(&vec, pb), Opacity::new(1.0), "B não recebe");

    // O passo do desfazer: a fotografia de antes repõe A; a seleção não entra nela.
    let depois = captura(&mut sim, &vec, &reg);
    assert!(depois != antes, "o documento mudou");
    let (v2, ..) = antes.restore(&mut sim, &reg);
    let bits_de = |sim: &mut SimWorld, nome: &str| {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == nome)
            .map(|(e, _)| e.to_bits())
            .expect("o objecto")
    };
    let a2 = bits_de(&mut sim, "A");
    assert_eq!(z(&sim, a2), None, "o desfazer não repôs o Z de A");
    assert_eq!(modo(&sim, a2), None, "o desfazer não repôs o modo de A");
    assert_eq!(
        op(&v2, pa),
        Opacity::new(1.0),
        "o desfazer não repôs a opacidade de A"
    );
    assert_eq!(v2.path(pa).expect("a forma").blend, BlendMode::Normal);
}
