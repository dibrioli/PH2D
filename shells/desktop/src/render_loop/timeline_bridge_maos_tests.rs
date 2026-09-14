//! ⭐⭐⭐ **A MÃO GANHA DA CURVA ENQUANTO ELA ESTÁ LÁ** — os gates da [`super::maos_do_quadro`].
//!
//! ⛔⛔ **O defeito** (report do dono, 2026-09-14: *«com a timeline aberta não é possível transformar
//! os ossos e criar key frames com AutoKey»*): com a lista vazia — o que o quadro entregava num
//! arrasto de osso — o apply reescrevia a rotação pela curva no quadro seguinte (`0,77 → 0,45`), e o
//! AutoKey, que corre DEPOIS dele, lia `mundo == curva`.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState};

/// Dois ossos cuja PONTA tem `Rotation` keyada `0 → 0,9` em `0..2 s` (a acção que a cena semeia), com
/// o relógio parado a meio, onde a curva vale `0,45`.
fn rig_keyado() -> (SimWorld, Entity, Entity, TimelineState, Playhead) {
    let mut sim = SimWorld::new();
    let raiz = ph2d_skeleton_live::bone::create(&mut sim, None, [0.0, 0.0], [1.0, 0.0])
        .map(Entity::from_bits)
        .expect("raiz");
    let ponta = ph2d_skeleton_live::bone::create(&mut sim, Some(raiz), [1.0, 0.0], [2.0, 0.0])
        .map(Entity::from_bits)
        .expect("ponta");
    let mut st = TimelineState::new();
    for (t, v) in [(0.0, 0.0_f32), (2.0, 0.9)] {
        st.doc.insert_key(
            ponta.to_bits(),
            PropKind::Rotation,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause();
    ph.seek(1.0);
    (sim, raiz, ponta, st, ph)
}

/// O estado do esqueleto com a ferramenta Bone a POSAR `e`.
fn posando(e: Entity) -> ph2d_app_skeleton::state::SkeletonState {
    ph2d_app_skeleton::state::SkeletonState {
        bone_pose: Some((e.to_bits(), ph2d_skeleton_render::BonePart::Body)),
        ..Default::default()
    }
}

fn rotation(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().rotation
}

/// Um quadro do dreno, com o que a mão segura.
fn quadro(sim: &mut SimWorld, st: &mut TimelineState, ph: &mut Playhead, maos: &[u64]) {
    super::run(
        sim.world_mut(),
        st,
        ph,
        &mut Vec::new(),
        maos,
        &mut crate::render_loop::autokey_pass::AutokeyState::default(),
        true,
        None,
        &mut super::SignalEmitter::default(),
        &mut ph2d_preview_drive::PreviewDrive::default(),
    );
}

/// ⭐⭐⭐ **A pose que a mão pôs SOBREVIVE ao apply** — e o controlo é a mesma cena com a mão vazia,
/// onde a curva ganha. *Sem o controlo, um apply que não escrevesse nada passaria neste gate.*
#[test]
fn a_bone_the_hand_is_posing_survives_the_apply() {
    let (mut sim, _raiz, ponta, mut st, mut ph) = rig_keyado();
    sim.world_mut()
        .get_mut::<Transform>(ponta)
        .unwrap()
        .rotation = 0.77;
    let maos = super::maos_do_quadro(None, &posando(ponta), &sim);
    quadro(&mut sim, &mut st, &mut ph, &maos);
    assert_eq!(
        rotation(&sim, ponta),
        0.77,
        "a curva escreveu por cima da mao: o osso volta debaixo do dedo e o AutoKey nao tem o que cunhar"
    );

    // ⛔ O CONTROLO: a MESMA cena sem mão nenhuma — a curva manda, que é o comportamento certo
    // quando ninguém está a posar.
    let (mut sim, _raiz, ponta, mut st, mut ph) = rig_keyado();
    sim.world_mut()
        .get_mut::<Transform>(ponta)
        .unwrap()
        .rotation = 0.77;
    quadro(&mut sim, &mut st, &mut ph, &[]);
    assert_eq!(
        rotation(&sim, ponta),
        0.45,
        "controlo: sem mao a curva TEM de escrever (senao este gate mede um apply inerte)"
    );
}

/// ⭐⭐ **De um osso vai o ESQUELETO INTEIRO** (a IK da ponta dobra a corrente toda), com o **guarda**
/// do sujeito que não é osso — ali o `skeleton_of` devolve a cena inteira, que é a leitura certa
/// dele para *«ninguém apontou»* e a errada aqui.
#[test]
fn the_whole_skeleton_is_held_and_a_stranger_holds_nothing() {
    let (mut sim, raiz, ponta, _st, _ph) = rig_keyado();
    let maos = super::maos_do_quadro(None, &posando(ponta), &sim);
    assert!(
        maos.contains(&raiz.to_bits()) && maos.contains(&ponta.to_bits()),
        "agarrar a ponta segura a corrente inteira: {maos:?}"
    );

    let estranho = sim.world_mut().spawn(Transform::default()).id();
    assert_eq!(
        super::maos_do_quadro(None, &posando(estranho), &sim),
        Vec::<u64>::new(),
        "um sujeito que nao e' osso nao pode segurar o esqueleto da cena inteira"
    );

    // ⛔ **E a mão ANTIGA não se perdeu ao ganhar a nova** — a forma de defeito que este repo já
    // pagou (*copiar um componente deixa para trás os irmãos*): sem esta metade, um arrasto de
    // gizmo passaria a brigar com a curva e nenhum gate o diria.
    let mut hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.drag = Some(ph2d_editor_core::gizmo::GizmoDragState {
        kind: ph2d_editor_core::gizmo::GizmoDragKind::Translate,
        entity_bits: estranho.to_bits(),
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: ph2d_editor_core::TransformSnapshot::IDENTITY,
        pivot_world: [0.0, 0.0],
        start_cursor_world: [0.0, 0.0],
        sprite_half_intrinsic: [0.5, 0.5],
        anchor_is_center: false,
        target: ph2d_editor_core::gizmo::GizmoTarget::PrimaryIndividual,
        parent_world: ph2d_editor_core::TransformSnapshot::IDENTITY,
        turns: 0,
    });
    assert_eq!(
        super::maos_do_quadro(Some(&hero), &Default::default(), &sim),
        vec![estranho.to_bits()],
        "o arrasto do gizmo de sprite e' a outra mao, e ela continua la'"
    );
}
