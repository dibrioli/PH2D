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
        p.paints
            .push(PaintEntry::fill(Paint::Solid(Rgba8::new(1, 1, 1, 255))));
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
    ph2d_host_presos::apply_no_alvo(&mut vec, &pen, &store, &sim, BaseVerb::FillEye);
    let forma = vec.path_mut(pa).expect("A");
    assert!(ph2d_host_presos::pick_base(
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
    assert_eq!(
        *v2.path(pa).expect("A"),
        de_antes.0,
        "o desfazer não repôs A"
    );
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

/// ⭐ O quadro de «Collides with»: a ação pelo braço real do dreno, um passo de desfazer, Ctrl+Z devolve.
#[test]
fn o_quadro_da_fila_e_um_passo_de_desfazer() {
    use crate::undo::{ProjectUndo, SelectionMark};
    let (mut sim, vec) = (SimWorld::new(), VecScene::new());
    let (mut reg, mut fila) = (ComponentRegistry::new(), ProjectUndo::default());
    register_ecs_components(&mut reg);
    let mut ponte = ph2d_physics_ecs::PhysicsBridge::new();
    let mut mundo = Default::default();
    ph2d_panel_physics::set_current_physics(Some(Default::default()));
    let mut antes = captura(&mut sim, &vec, &reg);
    antes.regras.physics = ponte.settings();
    let acao = EditorAction::LayerPairToggle { a: 3, b: 1 };
    let mut app = crate::App::new();
    assert!(
        app.fase_bus_inspector_identity(acao, &mut DrainOut::default())
            .is_none()
    );
    for pedido in ph2d_panel_physics::drain_intents() {
        let ph2d_panel_physics::PhysicsIntent::SetSettings(s, t) = pedido else {
            panic!("o quadro só pede settings");
        };
        ph2d_app_physics::panel_bridge::settings_do_painel(&mut ponte, &mut mundo, *s, *t);
    }
    let mut depois = antes.clone();
    depois.regras.physics = ponte.settings();
    assert_ne!(depois, antes, "a matriz mudou");
    fila.push_undo(antes.clone(), SelectionMark::default());
    let (volta, _) = fila
        .undo(depois, SelectionMark::default())
        .expect("o passo");
    assert_eq!(
        (volta, fila.depth()),
        (antes, 0),
        "um passo, e Ctrl+Z devolve"
    );
}

/// ⭐ «Move and remove» (onda 9b) é UM passo e Ctrl+Z devolve camada, nome, regra e objetos (onda 9d): o
/// braço real da shell (`acao_das_camadas`) tira a camada; a linha da matriz volta ao padrão como o painel a pede.
/// Mutação: tirar o `camadas.install` do `Regras::repor` deixa o nome perdido e reprova aqui.
#[test]
fn mover_e_remover_e_um_passo_e_ctrl_z_devolve_camada_nome_regra_e_objetos() {
    use crate::undo::{ProjectUndo, Regras, SelectionMark};
    use ph2d_physics_ecs::{Collider, LayerMatrix, LayerNames, PhysicsBridge, RigidBody};
    let (mut sim, vec) = (SimWorld::new(), VecScene::new());
    let (mut reg, mut fila) = (ComponentRegistry::new(), ProjectUndo::default());
    register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let mut ponte = PhysicsBridge::new();
    let mut s = ponte.settings();
    let mut m = LayerMatrix::from_rows(s.layer_matrix);
    m.set(2, 1, false);
    s.layer_matrix = m.rows();
    ponte.set_settings(s);
    LayerNames::show(sim.world_mut(), 2);
    LayerNames::rename(sim.world_mut(), 2, "Enemy Bats");
    let col = Collider {
        layer: 2,
        ..Collider::default()
    };
    sim.world_mut()
        .spawn((Transform::default(), RigidBody::default(), col));
    let foto = |sim: &mut SimWorld, ponte: &PhysicsBridge| {
        let mut e = captura(sim, &vec, &reg);
        e.regras = Regras::de(ponte, sim);
        e
    };
    let antes = foto(&mut sim, &ponte);
    let acao = EditorAction::LayerRemove { layer: 2, to: 0 };
    ph2d_app_physics::panel_bridge::acao_das_camadas(&mut sim, &mut None, acao);
    let mut m = LayerMatrix::from_rows(ponte.settings().layer_matrix);
    (0..ph2d_physics_ecs::MAX_LAYERS).for_each(|b| m.set(2, b, true));
    ponte.set_settings(ph2d_physics_ecs::PhysicsSettings {
        layer_matrix: m.rows(),
        ..ponte.settings()
    });
    let depois = foto(&mut sim, &ponte);
    assert_eq!(depois.regras.camadas.0[2], "", "o nome saiu");
    fila.push_undo(antes.clone(), SelectionMark::default());
    let (volta, _) = fila
        .undo(depois, SelectionMark::default())
        .expect("o passo");
    assert_eq!((&volta, fila.depth()), (&antes, 0), "um passo devolve tudo");
    let _ = volta.restore(&mut sim, &reg);
    volta.regras.repor(&mut ponte, &mut sim);
    let nomes = LayerNames::of(sim.world());
    assert_eq!(nomes.0[2], "Enemy Bats", "o nome voltou");
    assert!(nomes.1 & (1 << 2) != 0, "a camada voltou à lista");
    assert!(
        !LayerMatrix::from_rows(ponte.settings().layer_matrix).collides(2, 1),
        "a regra voltou"
    );
    let mut q = sim.world_mut().query::<&Collider>();
    let camadas: Vec<u8> = q.iter(sim.world()).map(|c| c.layer).collect();
    assert_eq!(camadas, [2], "o objeto voltou à camada dele");
}

/// ⭐ GEF02, dono 09/11: «Collides with» com DUAS formas selecionadas, em QUADROS do caminho real: o dreno
/// `fase_bus_physics_sections` e a aplicação `inspector_commits::drain_section_edits` correm em cada quadro;
/// quadro 1 = o clique, quadro 2 = vazio. O passo de desfazer é a lei de `post_frame_undo_steps` (o quadro cuja
/// captura difere do baseline empurra o baseline e torna-se o baseline), modelada aqui porque o `App` inteiro
/// não corre sem janela (`capture_project` pede o `gfx`); o quadro 2 conta como quadro COM entrada (o soltar do
/// botão), o caso em que um segundo passo NASCE em vez de se fundir. Ctrl+Z depois do quadro 2 devolve AS DUAS
/// formas e a fila fica a zero. Mutações: só a primária · `take(1)` · a 2.ª forma aplicada um quadro depois.
#[test]
fn colide_com_duas_formas_e_um_passo_e_ctrl_z_devolve_as_duas() {
    use crate::undo::{ProjectUndo, SelectionMark};
    use ph2d_editor_core::PhysicsFieldEdit;
    use ph2d_physics_ecs::{BodyKind, ColliderShape, CollisionOverride, RigidBody};
    let (mut sim, vec) = (SimWorld::new(), VecScene::new());
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let corpo = sim
        .world_mut()
        .spawn((
            Name::new("Crate"),
            Transform::default(),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
        ))
        .id();
    let bola = ColliderShape::Ball { radius: 0.5 };
    let a = ph2d_physics_ecs::nasce_com_forma(sim.world_mut(), corpo, bola);
    let b = ph2d_physics_ecs::nasce_com_forma(sim.world_mut(), corpo, bola);
    let tem = |sim: &SimWorld, e| sim.world().get::<CollisionOverride>(e).copied();
    assert_eq!((tem(&sim, a), tem(&sim, b)), (None, None));
    let mut app = crate::App::new();
    let hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    let mut fila_de_comandos = EditorCommandQueue::default();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let mut baseline = captura(&mut sim, &vec, &reg);
    let antes = baseline.clone();
    let mut fila = ProjectUndo::default();
    let mut quadro = |sim: &mut SimWorld, acao: Option<EditorAction>| {
        let mut pd = DrainOut {
            inspector_selection: vec![a.to_bits(), b.to_bits()],
            ..DrainOut::default()
        };
        if let Some(acao) = acao {
            assert!(app.fase_bus_physics_sections(acao, &mut pd).is_none());
        }
        let _ = super::super::inspector_commits::drain_section_edits(
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &pd.physics_edits,
            &[],
            &hero,
            sim,
            &mut fila_de_comandos,
            &reg,
            &mut toasts,
        );
        let atual = captura(sim, &vec, &reg);
        if atual != baseline {
            fila.push_undo(
                std::mem::replace(&mut baseline, atual),
                SelectionMark::default(),
            );
        }
    };
    let clique = EditorAction::InspectorPhysicsEdit {
        entity_bits: a.to_bits(),
        edit: PhysicsFieldEdit::CollidesWith {
            layer: 0,
            on: false,
        },
    };
    quadro(&mut sim, Some(clique));
    let quer = Some(CollisionOverride {
        include: 0,
        exclude: 1,
    });
    eprintln!("quadro 1: a {:?} · b {:?}", tem(&sim, a), tem(&sim, b));
    assert_eq!(
        (tem(&sim, a), tem(&sim, b)),
        (quer, quer),
        "o MESMO quadro muda as duas formas"
    );
    quadro(&mut sim, None);
    let depois = baseline.clone();
    assert_ne!(depois, antes, "o documento mudou");
    eprintln!("passos na fila depois do quadro 2: {}", fila.depth());
    let (volta, _) = fila
        .undo(depois, SelectionMark::default())
        .expect("o passo");
    let _ = volta.restore(&mut sim, &reg);
    // ⚠️ Pelo MUNDO, não pelos `Entity` de antes: o restauro pode recriar as entidades, e um id
    // velho leria `None` mesmo com a exceção ainda lá.
    let excecoes = {
        let mut q = sim
            .world_mut()
            .query::<(&ph2d_physics_ecs::Collider, Option<&CollisionOverride>)>();
        q.iter(sim.world())
            .map(|(_, o)| o.copied())
            .collect::<Vec<_>>()
    };
    eprintln!(
        "Ctrl+Z: formas {excecoes:?} · fila {} · documento = o de antes {}",
        fila.depth(),
        volta == antes
    );
    assert_eq!(
        (excecoes, fila.depth()),
        (vec![None, None], 0),
        "um passo, e Ctrl+Z devolve as duas"
    );
    assert_eq!(volta, antes);
}

/// ⭐ GEF02, dono 09/11: duas formas de linhas DIVERGENTES selecionadas (a primária sem exceção, a outra sem a
/// camada 0). A §11 que a shell publica (`build_physics_info` + `com_a_selecao`, o par de
/// `snapshots_inspector_physics`) acende o quadro MISTO da camada 0 — o clique nele pede `on: true` (aceso para
/// todas), onde um quadro aceso pediria `off` — e «Use layer default» aparece; clicado no Inspector pintado, o dreno
/// real espalha-o e a aplicação real tira a exceção das DUAS. Mutação: o misto lido só da primária.
#[cfg(feature = "panel-inspector")]
#[test]
fn duas_formas_divergentes_mostram_o_misto_e_o_default_tira_a_excecao_das_duas() {
    use super::super::bus_hierarchy::tests::{
        clica_no_inspector, inspector_da_fisica, sai_do_inspector,
    };
    use ph2d_editor_core::PhysicsFieldEdit;
    use ph2d_panel_inspector::ids::{INSP_PHYS_COLLIDES, INSP_PHYS_LAYER_DEFAULT};
    use ph2d_physics_ecs::{BodyKind, ColliderShape, CollisionOverride, RigidBody};
    let mut sim = SimWorld::new();
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let corpo = sim
        .world_mut()
        .spawn((
            Name::new("Crate"),
            Transform::default(),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
        ))
        .id();
    let bola = ColliderShape::Ball { radius: 0.5 };
    let a = ph2d_physics_ecs::nasce_com_forma(sim.world_mut(), corpo, bola);
    let b = ph2d_physics_ecs::nasce_com_forma(sim.world_mut(), corpo, bola);
    let hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    let mut fila_de_comandos = EditorCommandQueue::default();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let mut aplica = |sim: &mut SimWorld, edits: &[(u64, PhysicsFieldEdit)]| {
        let _ = super::super::inspector_commits::drain_section_edits(
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            edits,
            &[],
            &hero,
            sim,
            &mut fila_de_comandos,
            &reg,
            &mut toasts,
        );
    };
    let sem_0 = PhysicsFieldEdit::CollidesWith {
        layer: 0,
        on: false,
    };
    aplica(&mut sim, &[(b.to_bits(), sem_0)]);
    let tem = |sim: &SimWorld, e| sim.world().get::<CollisionOverride>(e).copied();
    assert_eq!(tem(&sim, a), None);
    assert!(tem(&sim, b).is_some(), "a exceção na 2.ª forma");
    let sel = [a.to_bits(), b.to_bits()];
    let info = ph2d_app_physics::inspector::body::build_physics_info(
        sim.world(),
        a.to_bits(),
        0,
        0,
        0,
        false,
        0,
        (0.0, 5.0),
        0,
        &ph2d_tags::TagTree::new(),
    )
    .map(|i| ph2d_app_physics::inspector::body::com_a_selecao(sim.world(), i, &sel))
    .expect("a §11 da primária");
    eprintln!(
        "§11 da primária: misto {:#b} · difere {} · fixos {:#b}",
        info.collides_mixed, info.collides_differs, info.collides_pinned
    );
    let (mut host, mut st, r) = inspector_da_fisica(info, a.to_bits());
    let achado = |id| {
        r.iter()
            .rev()
            .find(|(i, q)| *i == id && q.w > 0.0)
            .map(|(_, q)| *q)
    };
    let quadro_0 = achado(INSP_PHYS_COLLIDES[0]);
    let default = achado(INSP_PHYS_LAYER_DEFAULT);
    let pede_0 = quadro_0.map(|q| clica_no_inspector(&mut host, &mut st, q));
    let pede_default = default.map(|q| clica_no_inspector(&mut host, &mut st, q));
    sai_do_inspector();
    eprintln!("quadro da camada 0 pede {pede_0:?}; «Use layer default» pede {pede_default:?}");
    let edicao = |edit| {
        vec![EditorAction::InspectorPhysicsEdit {
            entity_bits: a.to_bits(),
            edit,
        }]
    };
    assert_eq!(
        pede_0,
        Some(edicao(PhysicsFieldEdit::CollidesWith {
            layer: 0,
            on: true
        })),
        "o quadro misto acende para todas"
    );
    let reset = pede_default.expect("«Use layer default» com uma exceção na seleção");
    assert_eq!(reset, edicao(PhysicsFieldEdit::UseLayerDefault));
    let mut pd = DrainOut {
        inspector_selection: sel.to_vec(),
        ..DrainOut::default()
    };
    let mut app = crate::App::new();
    for acao in reset {
        assert!(app.fase_bus_physics_sections(acao, &mut pd).is_none());
    }
    aplica(&mut sim, &pd.physics_edits);
    eprintln!(
        "depois do reset: a {:?} · b {:?}",
        tem(&sim, a),
        tem(&sim, b)
    );
    assert_eq!(
        (tem(&sim, a), tem(&sim, b)),
        (None, None),
        "o reset tira a exceção das duas"
    );
}
