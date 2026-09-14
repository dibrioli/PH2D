//! **A MÃO NO OSSO é um arrasto** — o irmão do [`super::tests`] pelo teto de 600 LOC, cortado por
//! RESPONSABILIDADE: ali mora o diff da pose, aqui quem o quadro considera estar a arrastar.
use super::*;

/// ⭐⭐⭐ **POSAR UM OSSO É UM ARRASTO** — sem isso cada quadro do gesto abre um passo de undo seu.
/// ⛔ Mede a COSTURA (`run`, não o `apply_samples`): é ela que lê os dois estados.
#[test]
fn posing_a_bone_opens_one_drag_bracket_like_the_gizmo() {
    use ph2d_app_skeleton::state::SkeletonState;
    let hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    let world = ph2d_ecs::SimWorld::new();
    let drive = ph2d_preview_drive::PreviewDrive::default();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause();
    let posando = SkeletonState {
        bone_pose: Some((7, ph2d_skeleton_render::BonePart::Body)),
        ..Default::default()
    };
    for (skeleton, esperado, porque) in [
        (&posando, true, "a mao no osso e' um arrasto"),
        (
            &SkeletonState::default(),
            false,
            "controlo: sem gesto nenhum nao ha' arrasto (senao o gate mede um `true` constante)",
        ),
    ] {
        let mut st = TimelineState::new();
        st.flags.auto_key = true;
        let mut ak = AutokeyState::default();
        super::run(
            &mut st,
            &ph,
            &mut ak,
            &mut ph2d_editor_core::ToastQueue::new(),
            &hero,
            &world,
            &drive,
            skeleton,
        );
        assert_eq!(ak.drag_active, esperado, "{porque}");
    }
}

/// ⭐⭐⭐ **O AUTOKEY GRAVA A POSE QUE A MÃO FEZ, e não só o osso debaixo do dedo.**
///
/// ⛔⛔ Puxar a PONTA de uma corrente é cinemática inversa: a corrente **inteira** dobra. Mas a
/// população que este passe amostrava era a **SELECÇÃO** (`gizmo.iter_selected()`), e agarrar um
/// osso **não** o selecciona (`despacho_clique_select`: o gesto escreve `bone_pose` e devolve) ⇒ a
/// pose que o artista fez ia para a animação **por um osso só**, e o resto dela perdia-se no
/// primeiro instante em que o apply voltasse a escrever pelas curvas.
///
/// ⭐ **A população certa já era CALCULADA** — a [`super::super::timeline_bridge::maos_do_quadro`],
/// construída na W8 para o apply *não* escrever por cima da mão. *Uma porta com um consumidor só
/// estava a metade do trabalho que sabia fazer.*
///
/// ⛔ O CONTROLO é o mesmo quadro **sem mão nenhuma**: ali a selecção continua a mandar sozinha, que
/// é o comportamento certo para uma edição que não é um arrasto de osso.
#[test]
fn autokey_records_every_bone_the_hand_moved_not_only_the_selected_one() {
    use ph2d_ecs::{Entity, SimWorld, Transform};
    use ph2d_timeline::PropKind;

    /// Dois ossos, AMBOS com `Rotation` keyada `0 → 0,9` em `0..2 s`; relógio parado a `1 s`, onde
    /// as duas curvas valem `0,45`.
    fn rig() -> (SimWorld, Entity, Entity, TimelineState, Playhead) {
        let mut sim = SimWorld::new();
        let raiz = ph2d_skeleton_live::bone::create(&mut sim, None, [0.0, 0.0], [1.0, 0.0])
            .map(Entity::from_bits)
            .expect("raiz");
        let ponta = ph2d_skeleton_live::bone::create(&mut sim, Some(raiz), [1.0, 0.0], [2.0, 0.0])
            .map(Entity::from_bits)
            .expect("ponta");
        let mut st = TimelineState::new();
        st.flags.auto_key = true;
        for e in [raiz, ponta] {
            for (t, v) in [(0.0, 0.0_f32), (2.0, 0.9)] {
                st.doc.insert_key(
                    e.to_bits(),
                    PropKind::Rotation,
                    ph2d_anim::RationalTime::from_seconds(t),
                    AnimValue::Float(v),
                    ph2d_anim::Interp::Linear,
                );
            }
        }
        let mut ph = Playhead::new(1.0 / 60.0);
        ph.pause();
        ph.seek(1.0);
        (sim, raiz, ponta, st, ph)
    }

    fn chaves(st: &TimelineState, e: Entity) -> usize {
        let alvo = st
            .doc
            .binding_for(e.to_bits(), PropKind::Rotation)
            .expect("o osso tem curva")
            .target;
        st.doc.active_clip().track(alvo).expect("a faixa").len()
    }

    for (segurando, esperado, porque) in [
        (
            true,
            (3, 3),
            "a IK dobrou os DOIS ossos: a pose que a mao fez tem de ir INTEIRA para a animacao",
        ),
        (
            false,
            (2, 3),
            "controlo: sem mao nenhuma manda a SELECCAO, e so' o osso escolhido e' cunhado",
        ),
    ] {
        let (mut sim, raiz, ponta, mut st, ph) = rig();
        // A IK dobrou a corrente inteira — as duas rotações saem da curva (`0,45`).
        for e in [raiz, ponta] {
            sim.world_mut().get_mut::<Transform>(e).unwrap().rotation = 0.77;
        }
        let mut hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
        {
            use ph2d_editor_core::panel::PanelHostInternal as _;
            hero.set_panel_visible("timeline", true);
        }
        hero.gizmo.selection = Some(ponta.to_bits());
        let skeleton = if segurando {
            ph2d_app_skeleton::state::SkeletonState {
                bone_pose: Some((ponta.to_bits(), ph2d_skeleton_render::BonePart::Body)),
                ..Default::default()
            }
        } else {
            ph2d_app_skeleton::state::SkeletonState::default()
        };
        super::run(
            &mut st,
            &ph,
            &mut AutokeyState::default(),
            &mut ph2d_editor_core::ToastQueue::new(),
            &hero,
            &sim,
            &ph2d_preview_drive::PreviewDrive::default(),
            &skeleton,
        );
        assert_eq!(
            (chaves(&st, raiz), chaves(&st, ponta)),
            esperado,
            "{porque}"
        );
    }
}

/// ⛔⛔ **A MÃO DIZ *«OLHA TAMBÉM PARA ESTES»*, NUNCA *«CUNHA ESTES»*** — sem este gate a cura
/// acima seria indistinguível de espalhar uma chave por cada osso do esqueleto a cada quadro do
/// arrasto, que é o defeito OPOSTO e igualmente caro (uma corrente de vinte ossos, vinte faixas
/// novas, e a animação deixa de ser editável).
///
/// Quem filtra é o **DIFF** do [`super::apply_samples`]: um osso cuja pose é exactamente a da curva
/// não cunha nada. Aqui a corrente tem TRÊS ossos, a mão segura a ponta, e só DOIS se moveram.
#[test]
fn a_bone_the_hand_holds_but_did_not_move_keys_nothing() {
    use ph2d_ecs::{Entity, SimWorld, Transform};
    use ph2d_timeline::PropKind;

    let mut sim = SimWorld::new();
    let a = ph2d_skeleton_live::bone::create(&mut sim, None, [0.0, 0.0], [1.0, 0.0])
        .map(Entity::from_bits)
        .expect("a");
    let b = ph2d_skeleton_live::bone::create(&mut sim, Some(a), [1.0, 0.0], [2.0, 0.0])
        .map(Entity::from_bits)
        .expect("b");
    let c = ph2d_skeleton_live::bone::create(&mut sim, Some(b), [2.0, 0.0], [3.0, 0.0])
        .map(Entity::from_bits)
        .expect("c");
    let mut st = TimelineState::new();
    st.flags.auto_key = true;
    for e in [a, b, c] {
        for (t, v) in [(0.0, 0.0_f32), (2.0, 0.9)] {
            st.doc.insert_key(
                e.to_bits(),
                PropKind::Rotation,
                ph2d_anim::RationalTime::from_seconds(t),
                AnimValue::Float(v),
                ph2d_anim::Interp::Linear,
            );
        }
    }
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause();
    ph.seek(1.0);
    // `a` fica EXACTAMENTE onde a curva o pôs (`0,45`); `b` e `c` foram dobrados pela IK.
    sim.world_mut().get_mut::<Transform>(a).unwrap().rotation = 0.45;
    for e in [b, c] {
        sim.world_mut().get_mut::<Transform>(e).unwrap().rotation = 0.77;
    }
    let mut hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    {
        use ph2d_editor_core::panel::PanelHostInternal as _;
        hero.set_panel_visible("timeline", true);
    }
    hero.gizmo.selection = Some(c.to_bits());
    super::run(
        &mut st,
        &ph,
        &mut AutokeyState::default(),
        &mut ph2d_editor_core::ToastQueue::new(),
        &hero,
        &sim,
        &ph2d_preview_drive::PreviewDrive::default(),
        &ph2d_app_skeleton::state::SkeletonState {
            bone_pose: Some((c.to_bits(), ph2d_skeleton_render::BonePart::Body)),
            ..Default::default()
        },
    );
    let chaves = |e: Entity| {
        let alvo = st
            .doc
            .binding_for(e.to_bits(), PropKind::Rotation)
            .expect("o osso tem curva")
            .target;
        st.doc.active_clip().track(alvo).expect("a faixa").len()
    };
    assert_eq!(
        (chaves(a), chaves(b), chaves(c)),
        (2, 3, 3),
        "o osso que a mao segura e que NAO se moveu tem de ficar sem chave — quem filtra e' o diff"
    );
}

/// ⭐⭐ **Os gates da [`super::populacao`]** — a porta que decide de quem este passe olha a pose.
/// ⛔ Sem eles, duas mutações sobreviviam: apagar o filtro de repetidos e perder a mão do GIZMO
/// (que a selecção cobre **por acaso**, e por acaso não é uma lei).
#[test]
fn the_population_is_the_selection_plus_the_hand_without_repeats() {
    use ph2d_ecs::{Entity, SimWorld};
    let mut sim = SimWorld::new();
    let raiz = ph2d_skeleton_live::bone::create(&mut sim, None, [0.0, 0.0], [1.0, 0.0])
        .map(Entity::from_bits)
        .expect("raiz");
    let ponta = ph2d_skeleton_live::bone::create(&mut sim, Some(raiz), [1.0, 0.0], [2.0, 0.0])
        .map(Entity::from_bits)
        .expect("ponta");
    let outro = sim.world_mut().spawn(ph2d_ecs::Transform::default()).id();
    let drive = ph2d_preview_drive::PreviewDrive::default();
    let posando = ph2d_app_skeleton::state::SkeletonState {
        bone_pose: Some((ponta.to_bits(), ph2d_skeleton_render::BonePart::Body)),
        ..Default::default()
    };

    // A selecção é a PONTA, que a mão também segura ⇒ ela entra UMA vez, à frente, e a raiz vem
    // atrás porque a IK a dobra.
    let mut hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.selection = Some(ponta.to_bits());
    assert_eq!(
        super::populacao(&hero, &posando, &sim, &drive),
        vec![ponta.to_bits(), raiz.to_bits()],
        "a seleccao vem a` frente, a mao a seguir, e o osso agarrado NAO se repete"
    );

    // ⛔ **A mão do GIZMO é a outra mão, e ela não pode viver de a selecção a cobrir por acaso.**
    let mut hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.drag = Some(ph2d_editor_core::gizmo::GizmoDragState {
        kind: ph2d_editor_core::gizmo::GizmoDragKind::Translate,
        entity_bits: outro.to_bits(),
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
        super::populacao(&hero, &Default::default(), &sim, &drive),
        vec![outro.to_bits()],
        "o que o gizmo arrasta tem de ser olhado mesmo sem estar na seleccao"
    );

    // ⛔ E o CONTROLO de que a porta não devolve tudo: sem mão nem selecção, ninguém.
    let hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    assert!(
        super::populacao(&hero, &Default::default(), &sim, &drive).is_empty(),
        "sem mao nem seleccao a porta nao olha para ninguem"
    );

    // ⛔⛔ **PRÉ-VISUALIZAÇÃO NÃO É AUTORIA** (auditoria de 2026-09-08, e até 2026-09-14 esta lei
    // vivia SEM gate): um osso que um MOTOR conduz — o osso inteligente, a âncora de IK — escreve
    // pose **entre** o apply e este passe, e cunhar a partir dela faria uma chave da saída do
    // motor. ⚠️ Ele é saltado **mesmo estando na mão**, que é a metade que a mutação apanhou.
    let mut conduzido = ph2d_preview_drive::PreviewDrive::default();
    conduzido.driven(
        raiz,
        ph2d_preview_drive::Driven::SolverPose(ph2d_ecs::Transform::default()),
        ph2d_preview_drive::Driven::SolverPose(ph2d_ecs::Transform {
            rotation: 0.3,
            ..Default::default()
        }),
    );
    let mut hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.selection = Some(ponta.to_bits());
    assert_eq!(
        super::populacao(&hero, &posando, &sim, &conduzido),
        vec![ponta.to_bits()],
        "um osso sob conducao de um MOTOR nao pode entrar na populacao, nem pela mao"
    );
}

/// ⚠️ **O caso REAL do artista: um rig NOVO, sem curva nenhuma.** Os gates acima partem de ossos já
/// keyados; aqui mede-se o primeiro gesto de uma animação — dobrar a corrente e ver se ela fica
/// gravada. ⛔ Se o passe só soubesse cunhar por cima de uma curva existente, o primeiro arrasto de
/// toda animação deste app seria mudo.
#[test]
fn the_first_pose_of_a_fresh_rig_is_recorded_for_every_bone_it_bent() {
    use ph2d_ecs::{Entity, SimWorld, Transform};
    use ph2d_timeline::PropKind;
    let mut sim = SimWorld::new();
    let raiz = ph2d_skeleton_live::bone::create(&mut sim, None, [0.0, 0.0], [1.0, 0.0])
        .map(Entity::from_bits)
        .expect("raiz");
    let ponta = ph2d_skeleton_live::bone::create(&mut sim, Some(raiz), [1.0, 0.0], [2.0, 0.0])
        .map(Entity::from_bits)
        .expect("ponta");
    let mut st = TimelineState::new();
    st.flags.auto_key = true;
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause();
    ph.seek(1.0);
    let mut hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    {
        use ph2d_editor_core::panel::PanelHostInternal as _;
        hero.set_panel_visible("timeline", true);
    }
    hero.gizmo.selection = Some(ponta.to_bits());
    let posando = ph2d_app_skeleton::state::SkeletonState {
        bone_pose: Some((ponta.to_bits(), ph2d_skeleton_render::BonePart::Body)),
        ..Default::default()
    };
    // ⚠️ **Um arrasto são DOIS quadros, e é preciso dizê-lo:** sem curva não há de que a pose esteja
    // «fora», então quem mede a mudança é a BASELINE — o 1.º quadro do gesto estabelece-a e os
    // seguintes cunham a diferença. ⛔ Um teste de UM quadro leria «não grava nada» sobre um passe
    // são, que é o erro que esta fixtura quase fabricou.
    let mut ak = AutokeyState::default();
    fn quadro(
        st: &mut TimelineState,
        ak: &mut AutokeyState,
        ph: &Playhead,
        hero: &ph2d_editor_core::HeroScreen,
        sim: &ph2d_ecs::SimWorld,
        posando: &ph2d_app_skeleton::state::SkeletonState,
    ) {
        super::run(
            st,
            ph,
            ak,
            &mut ph2d_editor_core::ToastQueue::new(),
            hero,
            sim,
            &ph2d_preview_drive::PreviewDrive::default(),
            posando,
        );
    }
    quadro(&mut st, &mut ak, &ph, &hero, &sim, &posando); // o gesto abre com a pose de repouso
    for e in [raiz, ponta] {
        sim.world_mut().get_mut::<Transform>(e).unwrap().rotation = 0.6;
    }
    quadro(&mut st, &mut ak, &ph, &hero, &sim, &posando); // e a IK dobrou a corrente
    for e in [raiz, ponta] {
        assert!(
            st.doc
                .binding_for(e.to_bits(), PropKind::Rotation)
                .is_some(),
            "o primeiro gesto de uma animacao tem de gravar TODO osso que ele dobrou"
        );
    }
}
