//! **Behavioral SEAM sweep for §12 Physics Joint** (W3).
//!
//! A SWEEP, not a sample: every control the section paints is exercised here
//! and its exact action asserted. Choosing "the fullest state" and trusting it
//! to cover the rest is the premise that has rotted twice in this repo
//! ([[feedback_the_fullest_card_premise_rots]]) — and here it would rot
//! immediately, because no single kind paints all the rows.
//!
//! ⚠️ **The click sweep drives `click_at` — the real dispatcher — and not a
//! synthetic `WidgetEvent`.** A synthetic event skips the store's focusability
//! check, so a widget left out of `populate` stays painted, hit-registered,
//! arm-wired and **dead under the mouse**, with a green test beside it. W2c
//! shipped that exact hole for a day and the fix was this.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{InspectorJointInfo, InspectorNameInfo, JointFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_joint, set_current_inspector_name,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_0042;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// A joint of `kind_tag`, with both Pin switches ON so the section paints
/// every row it has. `bound`, because a connected joint is the state an artist
/// is in when they tune one.
fn joint(kind_tag: u8) -> InspectorJointInfo {
    InspectorJointInfo {
        entity_bits: ENTITY,
        kind_tag,
        body_a_name: "Hook".into(),
        body_b_name: "Plank".into(),
        bound: true,
        world_anchored: false,
        limits_enabled: true,
        limit_min_ui: -45.0,
        limit_max_ui: 45.0,
        motor_enabled: true,
        motor_mode_tag: 0,
        motor_speed_ui: 114.0,
        motor_target_ui: 0.0,
        motor_max_force: 10.0,
        rest_length: 1.0,
        stiffness: 30.0,
        damping: 0.5,
        max_length: 2.0,
        // No pick armed by default — the base fixture is a joint being tuned.
        pick_armed: 0,
        wheel_count: 2,
        // Breaking is ON in the base fixture so the thresholds are on screen for
        // the row sweeps; the switch's own OFF half is asserted by
        // `the_break_rows_follow_their_switch`.
        break_enabled: true,
        break_force: 100.0,
        break_torque: 50.0,
        // The engine's answer travels in the snapshot; the fixture states it the
        // way `build_joint_info` computes it, so a kind that gains the ability
        // does not silently lose the gate.
        breaks_on_torque: kind_tag == 0,
        // W-J8. The base fixture is a joint that is IN FORCE and whose bodies do
        // not collide — the two defaults, so a sweep sees the section an artist
        // actually opens.
        active: true,
        collide_connected: false,
    }
}

/// Paint the section, then click the middle of the widget with `id` — through
/// the real pointer dispatcher. Returns the actions the panel raised.
fn click_real(info: InspectorJointInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_joint(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("§12 never painted the widget {id:?}"));
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "clicking the middle of {id:?} produced {events:?} — the widget is \
         painted and hit-registered but the store does not consider it \
         focusable, so it is dead under the mouse"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_joint(None);
    out
}

fn commit(info: InspectorJointInfo, id: ph2d_a11y::NodeId, v: f64) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_joint(Some(info));
    host.set_number_value(id, v);
    let _ = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::ValueChanged(id));
    let out = host.drained_actions();
    set_current_inspector_joint(None);
    out
}

#[track_caller]
fn expect(actions: &[EditorAction], edit: JointFieldEdit, what: &str) {
    assert_eq!(
        actions,
        [EditorAction::InspectorJointEdit {
            entity_bits: ENTITY,
            edit,
        }],
        "{what} did not raise the edit it is supposed to"
    );
}

/// Tag da POLIA, e o ÚLTIMO da lista — é por ele que as varreduras por-tipo
/// deste arquivo sabem onde parar.
///
/// ⚠️ O doc daqui dizia *"o único tipo que NÃO pode partir"*, e isso deixou de
/// ser verdade no W-Pulley W2: o passe próprio dela publica a tensão, então ela
/// parte como todos. A constante sobreviveu ao fato por um commit, e quem a
/// pegou foi o aviso de código morto — o uso que a justificava tinha sumido
/// junto com a exceção.
const KIND_PULLEY: u8 = 7;

/// **Every kind chip is clickable and picks its own kind.**
#[test]
fn the_kind_chips_each_pick_their_own_kind() {
    for (i, &id) in ids::INSP_JOINT_KIND.iter().enumerate() {
        expect(
            &click_real(joint(0), id),
            JointFieldEdit::Kind(i as u8),
            &format!("kind chip {i}"),
        );
    }
}

/// **The two Pin switches are switches** — Off writes `false`, On writes
/// `true`, and each is reached by clicking it.
#[test]
fn the_pin_switches_write_the_side_they_are_on() {
    for (i, &id) in ids::INSP_JOINT_LIMITS.iter().enumerate() {
        expect(
            &click_real(joint(0), id),
            JointFieldEdit::LimitsEnabled(i == 1),
            &format!("limits switch {i}"),
        );
    }
    for (i, &id) in ids::INSP_JOINT_MOTOR.iter().enumerate() {
        expect(
            &click_real(joint(0), id),
            JointFieldEdit::MotorEnabled(i == 1),
            &format!("motor switch {i}"),
        );
    }
}

/// **Delete Joint is wired.**
#[test]
fn delete_joint_is_dispatched() {
    expect(
        &click_real(joint(0), ids::INSP_JOINT_REMOVE),
        JointFieldEdit::Remove,
        "Delete Joint",
    );
}

/// **The two body eyedroppers are ALWAYS live and each arms its own slot.**
///
/// The redesign's point: with only the joint selected — no other object — both
/// pickers are painted, focusable, and dispatch `PickBodyA`/`PickBodyB` (the
/// shell then resolves the body from the next canvas click). A mismatch that
/// routed both to one slot would pass a one-button check; this exercises each.
#[test]
fn the_body_pickers_arm_their_own_slot() {
    expect(
        &click_real(joint(0), ids::INSP_JOINT_PICK_A),
        JointFieldEdit::PickBodyA,
        "pick Body A",
    );
    expect(
        &click_real(joint(0), ids::INSP_JOINT_PICK_B),
        JointFieldEdit::PickBodyB,
        "pick Body B",
    );
}

/// **O par Object|World ARMA o que ele nomeia** (W-JointWorld).
///
/// Sem este canal o pino de mundo é inautorável: o marcador é um componente, e
/// nada mais na tela o acrescenta.
#[test]
fn the_anchor_b_chips_ask_for_the_side_they_name() {
    expect(
        &click_real(joint(0), ids::INSP_JOINT_ANCHOR_B[1]),
        JointFieldEdit::AnchorToWorld(true),
        "anchor B to the world",
    );
    expect(
        &click_real(joint(0), ids::INSP_JOINT_ANCHOR_B[0]),
        JointFieldEdit::AnchorToWorld(false),
        "anchor B back to an object",
    );
}

/// **Um pino de mundo NÃO oferece o conta-gotas do lado B — e OFERECE o do A.**
///
/// ⚠️ As duas metades no mesmo gate de propósito: *"o ícone sumiu"* sozinho
/// também é o que aconteceria se a seção inteira parasse de pintar, e aí o gate
/// estaria verde sobre a §12 quebrada. O lado A é o controle.
#[test]
fn a_world_pin_withholds_the_b_picker_but_keeps_the_a_one() {
    let mut info = joint(0);
    info.world_anchored = true;
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_joint(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_joint(None);
    assert!(
        rects.iter().any(|(n, _)| *n == ids::INSP_JOINT_PICK_A),
        "o conta-gotas do lado A tinha de continuar lá — ele é o CONTROLE"
    );
    assert!(
        !rects.iter().any(|(n, _)| *n == ids::INSP_JOINT_PICK_B),
        "o conta-gotas do lado B foi pintado num pino de MUNDO: não há corpo a \
         apontar, e ele armaria um pick que nenhum clique pode satisfazer"
    );
}

/// **The eyedroppers show for EVERY kind** — a body has two ends whatever the
/// constraint is, so the pickers are not gated on kind like the parameter rows.
#[test]
fn both_pickers_paint_for_every_kind() {
    for kind in 0u8..5 {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_joint(Some(joint(kind)));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_joint(None);
        for id in [ids::INSP_JOINT_PICK_A, ids::INSP_JOINT_PICK_B] {
            assert!(
                rects.iter().any(|(n, _)| *n == id),
                "kind {kind} did not paint the body picker {id:?}"
            );
        }
    }
}

/// **Every number box commits to its own field.**
///
/// The values are distinct on purpose: a handler that routed two boxes to the
/// same field would pass a sweep that committed the same number everywhere.
#[test]
fn each_number_box_commits_to_its_own_field() {
    let cases: [(u8, ph2d_a11y::NodeId, f64, JointFieldEdit); 8] = [
        (
            0,
            ids::INSP_JOINT_LIMIT_MIN,
            -30.0,
            JointFieldEdit::LimitMin(-30.0),
        ),
        (
            0,
            ids::INSP_JOINT_LIMIT_MAX,
            60.0,
            JointFieldEdit::LimitMax(60.0),
        ),
        (
            0,
            ids::INSP_JOINT_MOTOR_SPEED,
            90.0,
            JointFieldEdit::MotorSpeed(90.0),
        ),
        (
            0,
            ids::INSP_JOINT_MOTOR_FORCE,
            7.5,
            JointFieldEdit::MotorMaxForce(7.5),
        ),
        (
            1,
            ids::INSP_JOINT_REST_LENGTH,
            2.5,
            JointFieldEdit::RestLength(2.5),
        ),
        (
            1,
            ids::INSP_JOINT_STIFFNESS,
            42.0,
            JointFieldEdit::Stiffness(42.0),
        ),
        (
            1,
            ids::INSP_JOINT_DAMPING,
            3.25,
            JointFieldEdit::Damping(3.25),
        ),
        (
            2,
            ids::INSP_JOINT_MAX_LENGTH,
            4.75,
            JointFieldEdit::MaxLength(4.75),
        ),
    ];
    for (kind, id, v, edit) in cases {
        expect(&commit(joint(kind), id, v), edit, &format!("{id:?}"));
    }
}

/// **Every row the section paints for a kind belongs to that kind.**
///
/// The anti-dead-knob sweep, asked of EACH kind rather than of "the fullest
/// one": a rope must not paint a stiffness box, a spring must not paint a
/// motor. The rule comes from `JointKind::is_hinge`/`has_length`, which the
/// bridge also asks before handing parameters to the solver — so a knob
/// painted here that the solver ignores would be a knob that does nothing.
#[test]
fn each_kind_paints_only_the_rows_it_uses() {
    // ⚠️ **Duas famílias de rows pertencem a MAIS DE UM tipo, e por isso não
    // cabem na tabela "um dono" abaixo.** As de LIMITE deixaram de ser do Pin
    // quando o Slider chegou (W-J5, `has_limits`), e as de MOTOR quando o servo
    // chegou (W-J6, `has_motor`: um trilho e um guincho são dirigidos também).
    // Cada uma é afirmada com o próprio predicado — colapsá-las na do Pin é o que
    // deixaria um Slider sem curso ou uma Rope sem guincho, em silêncio.
    const LIMIT_ROWS: [ph2d_a11y::NodeId; 2] =
        [ids::INSP_JOINT_LIMIT_MIN, ids::INSP_JOINT_LIMIT_MAX];
    const MOTOR_ROWS: [ph2d_a11y::NodeId; 2] =
        [ids::INSP_JOINT_MOTOR_SPEED, ids::INSP_JOINT_MOTOR_FORCE];
    /// Só de uma Spring: uma roda tem mola mas não tem comprimento de repouso
    /// (a altura de marcha dela é ONDE o artista montou o carro).
    const REST_ONLY: [ph2d_a11y::NodeId; 1] = [ids::INSP_JOINT_REST_LENGTH];
    /// A MOLA — da Spring e da suspensão de um Wheel. Mesmos campos, mesmos
    /// ids, porque é a mesma coisa física.
    const SPRING_ROWS: [ph2d_a11y::NodeId; 2] =
        [ids::INSP_JOINT_STIFFNESS, ids::INSP_JOINT_DAMPING];
    /// O comprimento — da Rope (um teto) e do Rod (uma igualdade).
    const LENGTH_ROWS: [ph2d_a11y::NodeId; 1] = [ids::INSP_JOINT_MAX_LENGTH];

    // ⚠️ **A faixa cobre TODOS os chips que o painel pinta, e por duas vezes ela
    // não cobriu.** Era `0..4` quando o Slider chegou (a metade `kind == 4` da
    // asserção de limite nunca rodou), virou `0..5` e ficou assim quando o Rod
    // (5) e o Wheel (6) chegaram. Agora ela é o COMPRIMENTO do array de chips —
    // um tipo novo entra na varredura sem ninguém lembrar.
    for kind in 0u8..ids::INSP_JOINT_KIND.len() as u8 {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_joint(Some(joint(kind)));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_joint(None);
        let painted = |id: ph2d_a11y::NodeId| rects.iter().any(|(n, _)| *n == id);

        // As rows de limite pertencem a TRÊS tipos, então não cabem na tabela
        // "um dono" abaixo — são afirmadas à parte.
        for &id in &LIMIT_ROWS {
            let limited = kind == 0 || kind == 4 || kind == 6;
            assert_eq!(
                painted(id),
                limited,
                "kind {kind} {} {id:?}: um alcance existe no Pin, no Slider e no \
                 Wheel (onde ele é o CURSO da suspensão), e em mais nenhum",
                if limited {
                    "must paint"
                } else {
                    "must not paint"
                }
            );
        }
        // Um motor existe no Pin, no Slider, na Rope e no Wheel — e em mais
        // nenhum. Uma Spring é excluída por MECÂNICA e não por gosto: o rapier
        // modela mola COMO motor no mesmo eixo, então um segundo ali comeria a
        // rigidez que o artista autorou (`ph2d_physics::motor_axis` diz o mesmo
        // ao solver). ⚠️ Um Wheel carrega os DOIS e não colidem, porque a mola
        // dele mora no eixo LINEAR e o motor no ANGULAR.
        for &id in &MOTOR_ROWS {
            let driven = kind == 0 || kind == 2 || kind == 4 || kind == 6;
            assert_eq!(
                painted(id),
                driven,
                "kind {kind} {} {id:?}: um motor existe no Pin, no Slider, na Rope \
                 e no Wheel",
                if driven {
                    "must paint"
                } else {
                    "must not paint"
                }
            );
        }
        // A mola pertence à Spring E ao Wheel; o comprimento à Rope, ao Rod E à
        // Pulley (nela o número é a CORDA inteira — o mesmo campo, outro rótulo).
        for (owners, ids_) in [
            (&[1u8][..], &REST_ONLY[..]),
            (&[1, 6][..], &SPRING_ROWS[..]),
            (&[2, 5, 7][..], &LENGTH_ROWS[..]),
        ] {
            for &id in ids_ {
                let mine = owners.contains(&kind);
                assert_eq!(
                    painted(id),
                    mine,
                    "kind {kind} {} {id:?}, que pertence a {owners:?}",
                    if mine { "must paint" } else { "must not paint" }
                );
            }
        }
        // The kind chips and Delete are painted for every kind — the control
        // group that is always there. Without this the assertions above would
        // be satisfied by a section that painted nothing at all.
        for id in ids::INSP_JOINT_KIND.iter().chain(&[ids::INSP_JOINT_REMOVE]) {
            assert!(painted(*id), "kind {kind} did not paint {id:?}");
        }
    }
}

/// **The switches' rows appear only when the switch is ON.**
///
/// A limit box for a hinge with limits off is a control that changes a number
/// nothing reads — the §11 rule about a radius on a box, one level in.
#[test]
fn the_pin_rows_follow_their_switches() {
    for (limits, motor) in [(false, false), (true, false), (false, true)] {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_joint(Some(InspectorJointInfo {
            limits_enabled: limits,
            motor_enabled: motor,
            ..joint(0)
        }));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_joint(None);
        let painted = |id: ph2d_a11y::NodeId| rects.iter().any(|(n, _)| *n == id);
        assert_eq!(painted(ids::INSP_JOINT_LIMIT_MIN), limits);
        assert_eq!(painted(ids::INSP_JOINT_LIMIT_MAX), limits);
        assert_eq!(painted(ids::INSP_JOINT_MOTOR_SPEED), motor);
        assert_eq!(painted(ids::INSP_JOINT_MOTOR_FORCE), motor);
    }
}

/// **With no joint selected, none of these ids does anything.**
///
/// The ids live in the store for the whole session, so an event arm that did
/// not check the section is live would fire on whatever entity happened to be
/// selected — writing joint parameters onto a sprite.
#[test]
fn the_joint_arms_are_silent_when_nothing_is_a_joint() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_joint(None);
    for id in [
        ids::INSP_JOINT_KIND[1],
        ids::INSP_JOINT_REMOVE,
        ids::INSP_JOINT_LIMITS[1],
        ids::INSP_JOINT_MOTOR[0],
    ] {
        let outcome = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(id));
        assert!(
            !matches!(outcome, EventOutcome::Consumed),
            "{id:?} was consumed with no joint selected"
        );
    }
    assert!(
        host.drained_actions().is_empty(),
        "the §12 arms raised an action with no joint selected"
    );
}

/// **Os chips de modo do motor são clicáveis e pedem o modo que dizem** (W-J6).
///
/// A metade que um sweep de *pintura* não faz: pintado e hit-registrado não é
/// escolhível — só o clique REAL atravessa a checagem de focabilidade do store.
/// É o buraco que deixou os chips de kind do §11 nascerem mortos.
#[test]
fn the_motor_mode_chips_ask_for_the_mode_they_name() {
    for (i, &id) in ids::INSP_JOINT_MOTOR_MODE.iter().enumerate() {
        let acts = click_real(joint(0), id);
        expect(
            &acts,
            JointFieldEdit::MotorMode(i as u8),
            &format!("chip de modo {i}"),
        );
    }
}

/// **Cada modo pinta a SUA row, e só a sua.**
///
/// Velocity mostra Speed, Position mostra Target. Pintar as duas seria dois
/// números onde só um é lido — o knob-que-não-faz-nada que esta seção recusa em
/// toda parte —, e pintar nenhuma seria um modo sem instrução.
///
/// Mutação: o `if` do modo trocado por `true` (sempre Target) ⇒ a metade Velocity
/// fica vermelha; por `false` ⇒ a metade Position.
#[test]
fn each_motor_mode_paints_only_its_own_number() {
    for mode in 0u8..2 {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_joint(Some(InspectorJointInfo {
            motor_mode_tag: mode,
            ..joint(0)
        }));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_joint(None);
        let painted = |id: ph2d_a11y::NodeId| rects.iter().any(|(n, _)| *n == id);
        assert_eq!(
            painted(ids::INSP_JOINT_MOTOR_SPEED),
            mode == 0,
            "modo {mode}: Speed pertence ao Velocity"
        );
        assert_eq!(
            painted(ids::INSP_JOINT_MOTOR_TARGET),
            mode == 1,
            "modo {mode}: Target pertence ao Position"
        );
        // O controle: o card inteiro não sumiu.
        assert!(
            painted(ids::INSP_JOINT_MOTOR_FORCE),
            "modo {mode}: Max Force existe nos dois"
        );
    }
}

/// **Um motor desligado não pinta nem modo nem número** — e ligado pinta os dois.
///
/// Presença E ausência na mesma asserção, porque o card só é honesto se o
/// desligar realmente recolher a instrução em vez de deixá-la inerte na tela.
#[test]
fn switching_the_motor_off_takes_its_instruction_off_the_screen() {
    for on in [false, true] {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_joint(Some(InspectorJointInfo {
            motor_enabled: on,
            ..joint(0)
        }));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_joint(None);
        let painted = |id: ph2d_a11y::NodeId| rects.iter().any(|(n, _)| *n == id);
        for id in ids::INSP_JOINT_MOTOR_MODE
            .iter()
            .chain(&[ids::INSP_JOINT_MOTOR_SPEED, ids::INSP_JOINT_MOTOR_FORCE])
        {
            assert_eq!(painted(*id), on, "motor {on}: {id:?}");
        }
        // O interruptor em si está sempre lá — senão não haveria como religar.
        for id in &ids::INSP_JOINT_MOTOR {
            assert!(painted(*id), "o switch Motor existe nos dois estados");
        }
    }
}

/// **A joint can be torn apart whatever kind it is — but only a Pin can be torn
/// apart by TORQUE** (W-J7).
///
/// Two claims that pull in opposite directions, which is why they share a gate:
/// the switch and the force row are the section's only widgets offered to ALL
/// five kinds (every joint has a linear reaction, and it reads exactly), while
/// the torque row is the narrowest thing in the section. Splitting them would let
/// one drift into the other's rule.
///
/// ⚠️ The narrowing is a MEASUREMENT, not a preference: rapier publishes the
/// reaction of a limited or motorised angular axis and nothing for a locked one —
/// a Weld cantilever holds 4.905 N·m and reads `0.0000`.
///
/// Mutation: painting the torque row unconditionally — four kinds go red with a
/// threshold that can never be crossed.
#[test]
fn breaking_is_offered_to_every_kind_and_the_torque_row_follows_the_flag() {
    // ⚠️ **O painel HONRA a flag; ele não sabe quem a tem.** A versão anterior
    // deste gate afirmava `kind == 0` — uma segunda cópia da resposta do motor,
    // que envelheceu na hora em que o Wheel passou a reportar torque (medido:
    // 0,5125 N.m com tração). Quem tem a flag é gateado onde ela é COMPUTADA
    // (`ph2d_physics_ecs::JointKind::breaks_on_torque`, mais o gate de snapshot
    // na shell); aqui fica só a metade que é do painel.
    for kind in 0u8..ids::INSP_JOINT_KIND.len() as u8 {
        for torque in [false, true] {
            let mut host = MockPanelHost::with_panel::<InspectorPanel>();
            let mut state = InspectorState::default();
            set_current_inspector_joint(Some(InspectorJointInfo {
                breaks_on_torque: torque,
                ..joint(kind)
            }));
            let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
            set_current_inspector_joint(None);
            let painted = |id: ph2d_a11y::NodeId| rects.iter().any(|(n, _)| *n == id);
            // ⚠️ **TODOS, incluindo a POLIA.** Ela foi a exceção enquanto nada
            // publicava a reação dela — não vive no `ImpulseJointSet` —, e o
            // W-Pulley W2 fechou isso: o passe próprio dela publica a tensão
            // (`λ/dt`), medida contra `m·g` com razão 0,9999. A frase *"todo
            // joint pode ser arrancado"* voltou a valer para todos.
            let breakable = true;
            for &id in &ids::INSP_JOINT_BREAK {
                assert_eq!(
                    painted(id),
                    breakable,
                    "kind {kind}: o switch de ruptura é oferecido a todo tipo cuja \
                     reação o solver PUBLICA — e só a esses"
                );
            }
            assert_eq!(
                painted(ids::INSP_JOINT_BREAK_FORCE),
                breakable,
                "kind {kind}: e o limiar de força segue o switch"
            );
            assert_eq!(
                painted(ids::INSP_JOINT_BREAK_TORQUE),
                torque && breakable,
                "kind {kind}: a row de torque segue a flag que o motor mandou, e \
                 nada mais"
            );
        }
    }
}

/// **The thresholds are on screen only when breaking is on** — and the switch
/// itself is always there.
#[test]
fn the_break_rows_follow_their_switch() {
    for on in [false, true] {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_joint(Some(InspectorJointInfo {
            break_enabled: on,
            ..joint(0)
        }));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_joint(None);
        let painted = |id: ph2d_a11y::NodeId| rects.iter().any(|(n, _)| *n == id);
        assert!(
            painted(ids::INSP_JOINT_BREAK[0]),
            "the switch is always offered — it is how breaking is turned ON"
        );
        for id in [ids::INSP_JOINT_BREAK_FORCE, ids::INSP_JOINT_BREAK_TORQUE] {
            assert_eq!(
                painted(id),
                on,
                "breaking {}: {id:?}",
                if on { "on" } else { "off" }
            );
        }
    }
}

/// **The Breakable switch and its two numbers reach the bus**, through a real
/// pointer for the switch and a real commit for the numbers.
#[test]
fn the_break_controls_are_wired() {
    for (i, want) in [(0usize, false), (1, true)] {
        expect(
            &click_real(joint(0), ids::INSP_JOINT_BREAK[i]),
            JointFieldEdit::BreakEnabled(want),
            "the Breakable switch",
        );
    }
    expect(
        &commit(joint(0), ids::INSP_JOINT_BREAK_FORCE, 250.0),
        JointFieldEdit::BreakForce(250.0),
        "Break Force",
    );
    expect(
        &commit(joint(0), ids::INSP_JOINT_BREAK_TORQUE, 12.0),
        JointFieldEdit::BreakTorque(12.0),
        "Break Torque",
    );
}

/// **Selecionar uma joint MOSTRA os tetos que ela carrega** (W-J7).
///
/// A outra metade da costura: as rows escrevem (acima), mas seriam WRITE-ONLY se
/// o `sync_joint_fields` não as espelhasse — voltar à joint mostraria a semente
/// em vez do que o artista digitou. É exatamente a falha que a família das rows
/// de área shipou (W-AreaTorque) e que este gate existe para não repetir.
///
/// Mutação: tirar as duas linhas do `sync_joint_fields` — os dois assert leem a
/// semente (100 / 50) em vez de 250 / 12.
#[test]
fn selecting_a_joint_shows_the_thresholds_it_carries() {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::screens::hero::InspectorTransformInfo;
    use ph2d_panel_inspector::state::set_current_inspector_transform;

    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_joint(Some(InspectorJointInfo {
        break_force: 250.0,
        break_torque: 12.0,
        ..joint(0)
    }));
    // ⚠️ A borda de "mudou a seleção" que gateia o sync sai do snapshot de
    // TRANSFORM, não do da joint — toda entidade selecionada tem um.
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: ENTITY,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_joint(None);
    set_current_inspector_transform(None);

    let val = |id| match host.store().get(id) {
        Some(InteractiveState::NumberInput { value, .. }) => *value,
        other => panic!("{id:?} nao e um NumberInput registrado: {other:?}"),
    };
    assert!(
        (val(ids::INSP_JOINT_BREAK_FORCE) - 250.0).abs() < 1e-6,
        "Break Force mostra o autorado, mostra {}",
        val(ids::INSP_JOINT_BREAK_FORCE)
    );
    assert!(
        (val(ids::INSP_JOINT_BREAK_TORQUE) - 12.0).abs() < 1e-6,
        "Break Torque idem, mostra {}",
        val(ids::INSP_JOINT_BREAK_TORQUE)
    );
}

/// **A higiene do par é oferecida em TODO tipo** (W-J8) — o Active, o Collide e
/// o Swap.
///
/// Varre os cinco tipos de propósito: nenhum deles é *mais* desmontável ou *mais*
/// re-etiquetável que outro, então um gating por tipo que aparecesse aqui seria
/// um controle desaparecendo de um lugar onde ele funciona. É a mesma varredura
/// que os chips de tipo e o Delete recebem, pelo mesmo motivo.
///
/// ⚠️ Dirige `click_at` — o despachante REAL. Um `WidgetEvent` sintético pula a
/// checagem de focabilidade do store, e foi assim que 36 células nasceram
/// pintadas, hit-registradas e **mortas sob o mouse** no W2c.
#[test]
fn the_pair_controls_are_offered_on_every_kind() {
    // ⚠️ **`INSP_JOINT_KIND.len()`, não `0..=4`.** A faixa parou em Slider e os
    // dois tipos seguintes (Rod, Wheel) nunca foram cobertos — o cluster do PAR
    // é pintado sem olhar o tipo, então ele funcionava, mas nada o dizia. É a
    // mesma rot das outras três listas escritas à mão desta seção.
    for kind_tag in 0u8..ids::INSP_JOINT_KIND.len() as u8 {
        for (i, &id) in ids::INSP_JOINT_ACTIVE.iter().enumerate() {
            expect(
                &click_real(joint(kind_tag), id),
                JointFieldEdit::Active(i == 1),
                &format!("kind {kind_tag}: active switch {i}"),
            );
        }
        for (i, &id) in ids::INSP_JOINT_COLLIDE.iter().enumerate() {
            expect(
                &click_real(joint(kind_tag), id),
                JointFieldEdit::CollideConnected(i == 1),
                &format!("kind {kind_tag}: collide switch {i}"),
            );
        }
        expect(
            &click_real(joint(kind_tag), ids::INSP_JOINT_SWAP),
            JointFieldEdit::Swap,
            &format!("kind {kind_tag}: Swap A/B"),
        );
    }
}

/// **O botão que acrescenta uma roldana existe SÓ na polia, e o clique chega.**
///
/// As duas metades num gate só, porque são a mesma decisão vista dos dois lados:
/// *"Add Wheel"* num Pin seria um controle que não pode fazer nada (um pino não
/// tem rota), e num Pulley que não despachasse seria a capacidade que o artista
/// pediu (o item 4) presa atrás de um botão morto.
#[test]
fn only_a_pulley_offers_the_add_wheel_button_and_the_click_lands() {
    for kind_tag in 0..=KIND_PULLEY {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_joint(Some(joint(kind_tag)));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_joint(None);
        let painted = rects.iter().any(|(n, _)| *n == ids::INSP_JOINT_ADD_WHEEL);
        assert_eq!(
            painted,
            kind_tag == KIND_PULLEY,
            "kind {kind_tag}: Add Wheel {} ser pintado",
            if kind_tag == KIND_PULLEY {
                "tinha de"
            } else {
                "não podia"
            }
        );
    }
    expect(
        &click_real(joint(KIND_PULLEY), ids::INSP_JOINT_ADD_WHEEL),
        JointFieldEdit::AddWheel,
        "Add Wheel",
    );
}

/// **E o Swap continua oferecido quando uma das pontas NÃO resolve.**
///
/// A metade que um gating em `bound` teria removido, e é o caso em que o botão é
/// mais útil: um joint cujo Body A foi apagado é exatamente quando o artista quer
/// que a ponta sobrevivente passe a ser A. Um controle que some justamente no
/// estado que ele conserta é pior que um que falta.
#[test]
fn the_swap_survives_a_body_that_no_longer_resolves() {
    let mut info = joint(0);
    info.body_a_name = String::new();
    info.bound = false;
    expect(
        &click_real(info, ids::INSP_JOINT_SWAP),
        JointFieldEdit::Swap,
        "Swap with a missing Body A",
    );
}

/// **Toda row NUMÉRICA que a §12 pinta é SEMEADA, ESPELHADA e ROTEADA** — as
/// três metades que fazem uma caixa de número ser um controle e não um enfeite.
///
/// ## Por que este gate existe
///
/// A row **Ratio** da polia (W-Pulley) shipou com as três faltando de uma vez:
/// nunca foi registrada no `populate_physics` (logo, morta sob o mouse), nunca
/// entrou no `sync_joint_fields` (logo, a caixa mostrava a semente em vez do
/// valor autorado — foi o que o artista reportou), e não tinha braço no
/// `event_joint` nem variante em `JointFieldEdit` (logo, digitar nela não fazia
/// **nada**). O doc-comment do `sync_joint_fields` **já avisava** contra
/// exatamente isto, nomeando a wave anterior que o cometeu (W-AreaTorque) — e a
/// wave seguinte o cometeu de novo, porque um aviso em prosa não falha.
///
/// O seam de PRESENÇA ao lado ficou verde o tempo todo: ele pergunta *a row está
/// na tela?*, e a row estava.
///
/// ## Como ele acha as rows sem uma lista à mão
///
/// A lista de rows numéricas **não é escrita aqui** — ela é a DIFERENÇA entre o
/// que o Inspector pinta com uma joint selecionada e o que ele pinta sem
/// nenhuma, menos os chips e botões enumerados abaixo. Uma row nova entra na
/// varredura sem ninguém lembrar dela, que é a propriedade que a lista à mão não
/// tem (a faixa `0..4` deste mesmo arquivo apodreceu duas vezes).
///
/// ⚠️ **Um chip novo esquecido em `NOT_A_NUMBER` faz o gate exigir um número
/// dele e FALHAR** — a direção segura. É a lista de números que não pode ficar
/// incompleta, e ela é derivada.
///
/// ⚠️ **O que ele NÃO prova:** que cada id espelha o campo CERTO do snapshot.
/// Ele prova que o valor veio do snapshot (está na faixa das sentinelas) e que
/// duas rows não mostram o mesmo campo; o pareamento id↔campo é afirmado pelos
/// gates de escrita (`commit`/`expect`) row a row.
#[test]
fn every_number_row_the_section_paints_is_seeded_synced_and_routed() {
    // Os ids da §12 que NÃO são caixas de número: os chips segmentados, os
    // botões e os ids de GRUPO que eles carregam.
    let mut not_a_number: Vec<ph2d_a11y::NodeId> = Vec::new();
    for group in [
        &ids::INSP_JOINT_KIND[..],
        &ids::INSP_JOINT_LIMITS[..],
        &ids::INSP_JOINT_MOTOR[..],
        &ids::INSP_JOINT_MOTOR_MODE[..],
        &ids::INSP_JOINT_BREAK[..],
        &ids::INSP_JOINT_ACTIVE[..],
        &ids::INSP_JOINT_COLLIDE[..],
        // W-JointWorld: o par Object|World do lado B.
        &ids::INSP_JOINT_ANCHOR_B[..],
    ] {
        not_a_number.extend_from_slice(group);
    }
    not_a_number.extend_from_slice(&[
        ids::INSP_JOINT_KIND_GROUP,
        ids::INSP_JOINT_LIMITS_GROUP,
        ids::INSP_JOINT_MOTOR_GROUP,
        ids::INSP_JOINT_MOTOR_MODE_GROUP,
        ids::INSP_JOINT_BREAK_GROUP,
        ids::INSP_JOINT_ACTIVE_GROUP,
        ids::INSP_JOINT_COLLIDE_GROUP,
        ids::INSP_JOINT_ANCHOR_B_GROUP,
        ids::INSP_JOINT_SWAP,
        // O botão que acrescenta uma roldana (W-Pulley W1) — botão, não número.
        ids::INSP_JOINT_ADD_WHEEL,
        ids::INSP_JOINT_REMOVE,
        ids::INSP_JOINT_PICK_A,
        ids::INSP_JOINT_PICK_B,
        ids::INSP_LIVE_JOINT_SECTION,
        ids::INSP_LIVE_JOINT_COLOR,
        // ⚠️ A BARRA DE ROLAGEM do Inspector, e ela não é da §12: a seção só a
        // faz aparecer porque o conteúdo passa a não caber. O gate a acusou
        // sozinho, que é a direção segura de falha desta lista.
        ph2d_editor_core::widget::INSPECTOR_SCROLLBAR_ID,
    ]);

    // As sentinelas: distintas entre si, nenhuma igual a uma semente do
    // `populate_physics` (-45 · 45 · 114 · 0 · 10 · 1 · 30 · 0,5 · 100 · 50),
    // e todas dentro da faixa MAIS ESTREITA de todas as rows (a razão, 0,01..100)
    // — uma sentinela fora da faixa seria clampada e o gate mediria o clamp.
    const SENTINELS: [f32; 12] = [
        61.0, 62.0, 63.0, 64.0, 65.0, 66.0, 67.0, 68.0, 69.0, 70.0, 71.0, 72.0,
    ];
    let numbered = |kind: u8| InspectorJointInfo {
        limit_min_ui: SENTINELS[0],
        limit_max_ui: SENTINELS[1],
        motor_speed_ui: SENTINELS[2],
        motor_target_ui: SENTINELS[3],
        motor_max_force: SENTINELS[4],
        rest_length: SENTINELS[5],
        stiffness: SENTINELS[6],
        damping: SENTINELS[7],
        max_length: SENTINELS[8],
        break_force: SENTINELS[10],
        break_torque: SENTINELS[11],
        ..joint(kind)
    };

    // ⚠️ O `Name` entra nos DOIS passes, e é o que torna o controle um
    // controle: ele existe porque o `entity_changed` do sync — a porta que
    // decide re-semear as caixas — só dispara com uma entidade selecionada, e
    // pô-lo só no experimento faria a seção NOME inteira aparecer na diferença.
    // (Foi o que aconteceu: o gate nasceu acusando um id do nome.)
    let name = || {
        set_current_inspector_name(Some(InspectorNameInfo {
            entity_bits: ENTITY,
            name: "Hook : Plank".into(),
        }));
    };

    for kind in 0u8..ids::INSP_JOINT_KIND.len() as u8 {
        // O que o Inspector pinta SEM joint nenhuma — o controle que isola os
        // ids da §12 dos das outras seções.
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        name();
        let base: Vec<_> = host
            .paint::<InspectorPanel>(&mut state, VIEWPORT)
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        set_current_inspector_name(None);

        // E com ela.
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        name();
        set_current_inspector_joint(Some(numbered(kind)));
        let painted: Vec<_> = host
            .paint::<InspectorPanel>(&mut state, VIEWPORT)
            .into_iter()
            .map(|(n, _)| n)
            .collect();

        let rows: Vec<_> = painted
            .iter()
            .copied()
            .filter(|id| !base.contains(id) && !not_a_number.contains(id))
            .collect();
        assert!(
            !rows.is_empty(),
            "kind {kind}: a §12 não pintou caixa de número nenhuma — ou o \
             controle sem joint passou a pintar a seção, e aí este gate está cego"
        );

        let mut seen: Vec<f64> = Vec::new();
        for &id in &rows {
            let v = host.store().number_value(id).unwrap_or_else(|| {
                panic!(
                    "kind {kind}: {id:?} é pintada e registrada no hit index mas o \
                     store não a conhece como número — ela não está no \
                     `populate_physics`, logo está morta sob o mouse"
                )
            });
            assert!(
                SENTINELS.iter().any(|s| (v - f64::from(*s)).abs() < 1e-6),
                "kind {kind}: {id:?} mostra {v}, que não é nenhum valor do \
                 snapshot — ela não está em `sync_joint_fields`, então a caixa é \
                 WRITE-ONLY: digitar funciona e re-selecionar mostra a semente"
            );
            assert!(
                !seen.iter().any(|s| (s - v).abs() < 1e-6),
                "kind {kind}: {id:?} mostra {v}, o MESMO campo que outra row — \
                 duas caixas espelhando um valor só"
            );
            seen.push(v);
        }
        set_current_inspector_joint(None);
        set_current_inspector_name(None);

        // E a terceira metade: digitar nela chega ao barramento.
        for &id in &rows {
            let actions = commit(numbered(kind), id, 7.0);
            assert!(
                matches!(
                    actions.as_slice(),
                    [EditorAction::InspectorJointEdit { .. }]
                ),
                "kind {kind}: mudar {id:?} produziu {actions:?} — a row não tem \
                 braço no `event_joint`, então digitar nela não faz nada"
            );
        }
    }
}

/// **O `Rope Length` de uma POLIA acompanha o número que a ponte DERIVOU**
/// (2026-07-29).
///
/// Enio: *"diferente das outras joints que mostram o tamanho real da corda, essa
/// junta não mostra"*. O `L0` de uma polia não é digitado — a ponte o semeia da
/// rota, e re-deriva a cada vez que o artista dimensiona uma roldana. Os irmãos
/// desta família sincam sob `entity_changed`, o contrato certo para um número que
/// só o artista muda, e é esse contrato que um número **derivado pelo produto**
/// quebra: a seleção não mudou, então a caixa mostrava o valor de antes.
///
/// As duas metades num gate só, porque são uma decisão: **acompanha** quando o
/// produto muda, e **não atropela** quando o artista está digitando.
///
/// Mutação: pôr a chamada de volta dentro do `if entity_changed` ⇒ a 1ª metade
/// cai (a caixa fica em 2,0); tirar o guard de foco ⇒ a 2ª cai.
#[test]
fn a_pulleys_rope_length_follows_the_derived_number_unless_you_are_typing() {
    const PULLEY: u8 = 7;
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();

    // A ponte semeou 11,965 na corda; a caixa ainda não sabe.
    let mut info = joint(PULLEY);
    info.max_length = 11.965;
    set_current_inspector_joint(Some(info));
    // Dois paints: no segundo a SELEÇÃO não mudou, que é exactamente o frame em
    // que o valor derivado tinha de chegar e não chegava.
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let shown = host
        .store()
        .number_value(ph2d_editor_core::ids::INSP_JOINT_MAX_LENGTH);
    assert!(
        matches!(shown, Some(v) if (v - 11.965).abs() < 1.0e-3),
        "a row mostrou {shown:?} sobre uma corda de 11,965 m — o número derivado \
         não chegou à caixa, e é isso que *não mostra o tamanho real da corda* é"
    );

    // E com a caixa FOCADA o sync não pode atropelar a edição em curso.
    host.store_mut()
        .set_focus(Some(ph2d_editor_core::ids::INSP_JOINT_MAX_LENGTH));
    host.set_number_value(ph2d_editor_core::ids::INSP_JOINT_MAX_LENGTH, 42.0);
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let typed = host
        .store()
        .number_value(ph2d_editor_core::ids::INSP_JOINT_MAX_LENGTH);
    assert!(
        matches!(typed, Some(v) if (v - 42.0).abs() < 1.0e-3),
        "a caixa focada foi sobrescrita ({typed:?} em vez de 42) — o sync apagou o \
         que o artista estava digitando"
    );
    set_current_inspector_joint(None);
}
