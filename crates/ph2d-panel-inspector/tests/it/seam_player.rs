//! **Varredura de SEAM da §14 Platform Player** (W5).
//!
//! Irmã de `seam_wheel.rs`, com a MESMA disciplina: uma varredura, não uma
//! amostra, e todo clique passa pelo `click_at` REAL. Um `WidgetEvent` sintético
//! pula a checagem de focabilidade do store, então um widget deixado de fora do
//! `populate` fica pintado, hit-registrado, com arm e **morto sob o mouse**, com
//! um teste verde ao lado (o buraco das 36 células do W2c).
//!
//! ⚠️ **A face VAZIA tem gate PRÓPRIO**, e é o mais importante do arquivo: até
//! esta seção existir, o `PlatformPlayer` era um componente que rodava em toda
//! cena de smoke (que constrói com código) e era **inalcançável no produto** —
//! o órfão que o `every_physics_component_is_authorable` reprovou por três waves.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{
    InspectorNameInfo, InspectorPlayerInfo, PlayerFieldEdit, PlayerLive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_name, set_current_inspector_player,
};
use ph2d_ui_testkit::MockPanelHost;

pub(crate) const ENTITY: u64 = 0x5EED_0081;
pub(crate) const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// Um corpo Dynamic que **já é** um player — o estado em que o artista está
/// quando afina um.
pub(crate) fn player() -> InspectorPlayerInfo {
    InspectorPlayerInfo {
        entity_bits: ENTITY,
        has_player: true,
        float_height: 0.9,
        // ⚠️ Premissa declarada: a altura ESTÁ acima do piso, então o botão de
        // ajuste é oferecido no rótulo curto. Uma fixture que chegasse abaixo do
        // piso deixaria o gate do aviso verde pelo motivo errado.
        min_float_height: 0.58,
        mode_tag: 0,
        // ⚠️ **Premissa declarada: o default é DESLIGADO e sem leitura viva** — é
        // o mundo com o toggle Physics desmarcado, que é o que o artista vê ao
        // abrir a cena. Uma fixture que já emitisse deixaria o gate do opt-in
        // verde pelo motivo errado.
        emits_signals: false,
        live: None,
        reaction_is_live: true,
        push_is_live: true,
        // ⚠️ Premissa declarada: este é o player DINÂMICO (`mode_tag: 0`), cuja
        // perna É uma mola — então as três rows dela e o botão de ajuste são
        // oferecidos. Uma fixture com a perna que POUSA deixaria verdes pelo
        // motivo errado todos os gates que contam rows do card LEG.
        spring_is_live: true,
        min_float_known: true,
        cling_distance: 0.25,
        spring_strength: 400.0,
        spring_damping: 0.5,
        speed: 6.0,
        acceleration: 60.0,
        air_acceleration: 20.0,
        // ⚠️ Premissa declarada: o NEUTRO, que é o mundo de antes da W-Brake.
        brake_scale: 1.0,
        max_slope_deg: 45.0,
        jump_height: 2.0,
        air_jumps: 2.0,
        air_jump_height: 1.5,
        takeoff_gravity: 1.0,
        takeoff_speed: 0.0,
        peak_gravity: 0.5,
        peak_speed: 1.5,
        fall_gravity: 2.0,
        cut_gravity: 4.0,
        coyote_time: 0.1,
        jump_buffer: 0.1,
        corner_reach: 0.12,
        corner_samples: 65.0,
        corner_lookahead: 2.0,
        // ⚠️ **AS PAREDES (W13) LIGADAS na fixture, e é declaração de premissa:**
        // a capacidade nasce DESLIGADA no produto, e uma fixture que herdasse o
        // zero varreria rows cujo valor nunca muda — verde sobre um controle que
        // ninguém provou vivo.
        wall_slide_speed: 3.0,
        wall_jump_height: 2.0,
        wall_jump_push: 6.0,
        wall_jump_lockout: 0.2,
        wall_reach: 0.08,
        foot_samples: 3.0,
        foot_spread: 1.0,
        wall_samples: 3.0,
        wall_spread: 1.0,
        wall_grab_stamina: 0.0,
        dash_speed: 0.0,
        dash_time: 0.15,
        dash_cooldown: 0.2,
        // ⚠️ **O AGACHAR (W15) LIGADO na fixture, pela razão exacta das paredes
        // acima:** ele nasce DESLIGADO no produto, e uma fixture que herdasse o
        // zero varreria rows cujo valor nunca muda.
        crouch_height: 0.55,
        crouch_speed: 2.0,
        // ⚠️ **O NADO (W-Swim) LIGADO na fixture, pela razão exacta dos três
        // acima:** ele nasce DESLIGADO no produto, e uma fixture que herdasse o
        // zero varreria rows cujo valor nunca muda.
        swim_speed: 4.0,
        swim_acceleration: 12.0,
        swim_enter: 1.0,
        // ⚠️ Valores DISTINTOS de todos os outros — o gate dos valores autorados
        // compara o que o store mostra com o que a fixture escreveu, e um número
        // repetido casaria por acidente.
        ledge_grab: 0.45,
        ledge_reach_y: 0.55,
        ledge_span: 0.12,
        ledge_offset_y: -0.08,
        ledge_speed: 3.5,
        glide_fall_speed: 2.5,
        // ⚠️ **O TETO DE QUEDA (W-Fall) LIGADO na fixture**, pela razão dos
        // quatro acima: ele nasce em zero no produto (a lei desligada), e uma
        // fixture que herdasse o zero varreria uma row cujo valor nunca muda.
        max_fall_speed: 30.0,
        // ⚠️ **`Full` é o default do produto**, e a fixture o declara em vez de o
        // herdar: um gate que chega ao estado por acidente inverte de sentido no
        // dia em que o default se move, e segue verde testando o oposto.
        platform_lift: 0,
        // ⚠️ A fixture DECLARA a premissa em vez de a herdar: ele pode andar
        // para fora de patamares (o mundo que já shipava), de pé e agachado, e
        // o agachar está ARMADO para a row condicional ser alcançável.
        walk_off_ledges: 0,
        crouch_walk_off_ledges: 0,
        crouch_armed: true,
        lift_momentum: 1.5,
        reaction_support: 1.0,
        reaction_movement: 0.0,
        // ⚠️ **O EMPURRÃO (W-KinPush) longe do default**, pela razão declarada
        // nas paredes e no agachar acima: uma fixture que herdasse o valor de
        // fábrica varreria uma row cujo número nunca muda.
        reaction_push: 0.75,
        // ⚠️ **Premissa declarada: NINGUÉM correu neste documento.** O botão de
        // descartar a corrida só existe quando existe corrida, então a fixture
        // base é a que NÃO o oferece — e é ela que dá sentido à metade de
        // ausência do gate `the_clear_run_button…`.
        recorded_run_seconds: 0.0,
        discarded_run_seconds: 0.0,
        // ⚠️ **Premissa declarada: TODAS as caixas marcadas**, pela razão das
        // capacidades LIGADAS acima: uma caixa desmarcada esconde rows (o Fall
        // Limit esconde o Max Fall), e a varredura não as alcançaria.
        abilities: [true; 7],
        show_jump_arc: false,
        feel: None,
    }
}

/// O host do produto com **todas as dobras do player ABERTAS** — elas nascem
/// fechadas (`populate`), e uma varredura sobre dobras fechadas não alcançaria
/// as rows de dentro.
pub(crate) fn host_aberto() -> MockPanelHost {
    let mut h = crate::host_em_metros();
    for id in ph2d_panel_inspector::ids::INSP_PLAYER_FOLDS {
        h.set_collapsed(id, false);
    }
    h
}

/// O MESMO player, com a saída de sinais LIGADA e uma leitura viva.
fn emitting() -> InspectorPlayerInfo {
    InspectorPlayerInfo {
        emits_signals: true,
        live: Some(PlayerLive {
            footing_tag: 2,
            facing: -1.0,
            velocity: [3.5, -0.25],
        }),
        ..player()
    }
}

/// O MESMO corpo, ainda sem o comportamento — a face vazia.
fn empty() -> InspectorPlayerInfo {
    InspectorPlayerInfo {
        has_player: false,
        ..player()
    }
}

pub(crate) fn painted(info: InspectorPlayerInfo) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    let mut host = host_aberto();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_player(None);
    rects
}

pub(crate) fn click_real(info: InspectorPlayerInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = host_aberto();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("a §14 nunca pintou o widget {id:?}"));
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "clicar no meio de {id:?} produziu {events:?} — o widget é pintado e \
         hit-registrado, mas o store não o considera focável: ele está morto \
         sob o mouse"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_player(None);
    out
}

fn commit(info: InspectorPlayerInfo, id: ph2d_a11y::NodeId, v: f64) -> Vec<EditorAction> {
    let mut host = host_aberto();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(info));
    host.set_number_value(id, v);
    let _ = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::ValueChanged(id));
    let out = host.drained_actions();
    set_current_inspector_player(None);
    out
}

#[track_caller]
pub(crate) fn expect(actions: &[EditorAction], edit: PlayerFieldEdit, what: &str) {
    assert_eq!(
        actions,
        [EditorAction::InspectorPlayerEdit {
            entity_bits: ENTITY,
            edit,
        }],
        "{what} não levantou a edição que devia"
    );
}

/// ⭐⭐ **A FACE VAZIA VOLTOU (2026-09-14, ordem do dono), e ela pinta UM botão e MAIS NADA.**
///
/// *«Um objeto de física (Physics Body) e todas as opções aparecem com ele (inclusive Collision
/// Shape e Platform Player).»* ⇒ a paleta do `+` passa a ter **uma** entrada de física, e a porta do
/// comportamento é outra vez o *Make Platform Player* desta secção — que é onde a pergunta vive, e
/// onde a física pode responder (num `Static` a mola do player é um impulso sobre massa infinita, e
/// o `+` genérico não sabe perguntar isso).
///
/// ⛔ **Ela morreu na F3** (ADR-0166) porque a §14 passou a pintar-se só COM o componente: a porta
/// ficava fechada sobre a própria chave. A reversão é do dono; o que **não** volta é a doença que a
/// F3 curou (um objecto pelado não mostra secção de física nenhuma).
///
/// ⚠️ **As DUAS metades importam, e a de ausência é a que apanha a regressão cara:** com
/// `has_player = false` o painel pinta **exactamente** o botão que cria o comportamento — e nenhum
/// dos knobs, que descreveriam um player que não existe. (Foi o que aconteceu quando a face saiu, e
/// estes dois gates foram quem o disse.)
#[test]
fn the_empty_face_paints_the_door_and_nothing_else() {
    let rects = painted(empty());
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_ADD),
        "a §14 nao pintou a porta: sem ela o comportamento e' inalcancavel, porque desde \
         2026-09-14 ele NAO esta' na paleta do `+`"
    );
    for id in [
        ph2d_panel_inspector::ids::INSP_PLAYER_REMOVE,
        ph2d_panel_inspector::ids::INSP_PLAYER_FIT,
        ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT,
        ph2d_panel_inspector::ids::INSP_PLAYER_SPEED,
        ph2d_panel_inspector::ids::INSP_PLAYER_MAX_SLOPE,
    ] {
        assert!(
            !rects.iter().any(|(n, _)| *n == id),
            "a §14 pintou {id:?} sobre um player que nao existe"
        );
    }
}

/// ⭐⭐ **E o botão da face está REGISTADO** — a metade que o gate acima não mede.
///
/// Aquele pergunta *«alguém o PINTA?»*; este pergunta *«alguém o REGISTA?»*, e são coisas
/// diferentes. As duas têm de valer ao mesmo tempo, e cada desemparelhamento é um defeito próprio:
/// **registado e não pintado** é o ÓRFÃO (invisível ao artista, e faz toda sonda de controlos
/// mortos apontar para um botão inexistente — foi assim que o
/// `ph2d-editor-core/tests/it/the_painted_control_reaches_a_consumer.rs` apanhou este mesmo id em
/// 2026-08-30, quando a F3 apagou a face e deixou o registo para trás); **pintado e não registado**
/// é um botão sem visual de estado, que não acende sob o dedo.
///
/// ⚠️ **A metade justa vem primeiro:** os irmãos do mesmo `register_button_ids` continuam
/// registados. Sem ela, apagar a chamada inteira passaria.
///
/// **Mutação:** tirar `ids::INSP_PLAYER_ADD` da lista do `populate_player.rs` ⇒ RED.
#[test]
fn the_door_of_the_empty_face_is_registered() {
    let host = MockPanelHost::with_panel_and_shared_chrome::<InspectorPanel>();
    for id in [
        ph2d_panel_inspector::ids::INSP_PLAYER_ADD,
        ph2d_panel_inspector::ids::INSP_PLAYER_REMOVE,
        ph2d_panel_inspector::ids::INSP_PLAYER_FIT,
        ph2d_panel_inspector::ids::INSP_PLAYER_CLEAR_RUN,
        ph2d_panel_inspector::ids::INSP_PLAYER_RESTORE_RUN,
        ph2d_panel_inspector::ids::INSP_PLAYER_FIT_CROUCH,
    ] {
        assert!(
            host.store().get(id).is_some(),
            "{id:?} nao esta' registado — um botao pintado sem registo nao acende sob o dedo, e \
             este gate mede a PRESENCA dos seis para que apagar a chamada inteira reprove"
        );
    }
}

/// **Todos os números levantam a própria edição** — a varredura.
///
/// Uma tabela e não oito testes: uma row nova que esqueça o arm faz esta lista
/// falhar, e uma que esqueça a lista não existe (o pintor itera a mesma).
#[test]
fn every_number_raises_its_own_edit() {
    // ⚠️ A lista TEM de cobrir a tabela inteira, e é isso que a asserção no fim
    // afirma: um número novo que chegue ao painel e não a esta varredura seria
    // exatamente o arm esquecido que ela existe para pegar.
    //
    // ⚠️ **A contagem é DERIVADA da própria varredura**, e não um literal ao
    // lado dela: um número escrito à mão só sabe dizer *"a tabela mudou"* — e
    // quem o bumpa para ficar verde faz exactamente o que ele existia para
    // impedir. Comparada com o `len()` da lista, a asserção passa a afirmar a
    // propriedade que interessa: *toda row pintada é varrida aqui*.
    let sweep = [
        // W-Probes2 — a GEOMETRIA das amostras dos sensores.
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_FOOT_SAMPLES,
            5.0,
            PlayerFieldEdit::FootSamples(5.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_FOOT_SPREAD,
            0.4,
            PlayerFieldEdit::FootSpread(0.4),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_CORNER_SAMPLES,
            129.0,
            PlayerFieldEdit::CornerSamples(129.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_CORNER_AHEAD,
            3.5,
            PlayerFieldEdit::CornerLookahead(3.5),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_WALL_SAMPLES,
            9.0,
            PlayerFieldEdit::WallSamples(9.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_WALL_SPREAD,
            0.6,
            PlayerFieldEdit::WallSpread(0.6),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT,
            1.25,
            PlayerFieldEdit::FloatHeight(1.25),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_CLING,
            0.4,
            PlayerFieldEdit::ClingDistance(0.4),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_STIFFNESS,
            250.0,
            PlayerFieldEdit::SpringStrength(250.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_DAMPING,
            0.8,
            PlayerFieldEdit::SpringDamping(0.8),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_SPEED,
            9.0,
            PlayerFieldEdit::Speed(9.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_ACCEL,
            80.0,
            PlayerFieldEdit::Acceleration(80.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_AIR_ACCEL,
            15.0,
            PlayerFieldEdit::AirAcceleration(15.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_BRAKE,
            0.25,
            PlayerFieldEdit::BrakeScale(0.25),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_COYOTE,
            0.2,
            PlayerFieldEdit::CoyoteTime(0.2),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_BUFFER,
            0.15,
            PlayerFieldEdit::JumpBuffer(0.15),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_CORNER,
            0.18,
            PlayerFieldEdit::CornerReach(0.18),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_LIFT,
            0.9,
            PlayerFieldEdit::LiftMomentum(0.9),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_WALL_SLIDE,
            4.5,
            PlayerFieldEdit::WallSlideSpeed(4.5),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_WALL_JUMP,
            2.5,
            PlayerFieldEdit::WallJumpHeight(2.5),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_WALL_PUSH,
            7.5,
            PlayerFieldEdit::WallJumpPush(7.5),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_WALL_LOCK,
            0.3,
            PlayerFieldEdit::WallJumpLockout(0.3),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_WALL_REACH,
            0.15,
            PlayerFieldEdit::WallReach(0.15),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_WALL_GRAB,
            1.5,
            PlayerFieldEdit::WallGrabStamina(1.5),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_DASH_SPEED,
            18.0,
            PlayerFieldEdit::DashSpeed(18.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_DASH_TIME,
            0.18,
            PlayerFieldEdit::DashTime(0.18),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_DASH_COOL,
            0.25,
            PlayerFieldEdit::DashCooldown(0.25),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_CROUCH_HEIGHT,
            0.6,
            PlayerFieldEdit::CrouchHeight(0.6),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_CROUCH_SPEED,
            2.5,
            PlayerFieldEdit::CrouchSpeed(2.5),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_SWIM_SPEED,
            5.0,
            PlayerFieldEdit::SwimSpeed(5.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_SWIM_ACCEL,
            18.0,
            PlayerFieldEdit::SwimAcceleration(18.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_SWIM_ENTER,
            0.6,
            PlayerFieldEdit::SwimEnter(0.6),
        ),
        // W-Ledge — a BEIRADA.
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_LEDGE_GRAB,
            0.55,
            PlayerFieldEdit::LedgeGrab(0.55),
        ),
        // W-LedgeSensor — o SENSOR: posicao e extensao.
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_LEDGE_REACH_Y,
            0.7,
            PlayerFieldEdit::LedgeReachY(0.7),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_LEDGE_SPAN,
            0.2,
            PlayerFieldEdit::LedgeSpan(0.2),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_LEDGE_OFFSET_Y,
            -0.3,
            PlayerFieldEdit::LedgeOffsetY(-0.3),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_LEDGE_SPEED,
            4.5,
            PlayerFieldEdit::LedgeSpeed(4.5),
        ),
        // W-Glide — o PLANEIO.
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_GLIDE_FALL,
            2.5,
            PlayerFieldEdit::GlideFallSpeed(2.5),
        ),
        // W-Fall — o TETO DE QUEDA (a velocidade terminal).
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_MAX_FALL,
            45.0,
            PlayerFieldEdit::MaxFallSpeed(45.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_MAX_SLOPE,
            55.0,
            PlayerFieldEdit::MaxSlopeDeg(55.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_JUMP_HEIGHT,
            3.5,
            PlayerFieldEdit::JumpHeight(3.5),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_AIR_JUMPS,
            2.0,
            PlayerFieldEdit::AirJumps(2.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_AIR_JUMP_H,
            1.25,
            PlayerFieldEdit::AirJumpHeight(1.25),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_TAKEOFF_G,
            1.4,
            PlayerFieldEdit::TakeoffGravity(1.4),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_TAKEOFF_SPEED,
            2.5,
            PlayerFieldEdit::TakeoffSpeed(2.5),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_PEAK_G,
            0.3,
            PlayerFieldEdit::PeakGravity(0.3),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_PEAK_SPEED,
            2.0,
            PlayerFieldEdit::PeakSpeed(2.0),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_FALL_G,
            2.5,
            PlayerFieldEdit::FallGravity(2.5),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_REACT_SUPPORT,
            0.6,
            PlayerFieldEdit::ReactionSupport(0.6),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_REACT_MOVEMENT,
            0.35,
            PlayerFieldEdit::ReactionMovement(0.35),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_REACT_PUSH,
            0.45,
            PlayerFieldEdit::ReactionPush(0.45),
        ),
        (
            ph2d_panel_inspector::ids::INSP_PLAYER_CUT_G,
            5.0,
            PlayerFieldEdit::CutGravity(5.0),
        ),
    ];
    assert_eq!(
        ph2d_panel_inspector::PLAYER_ROW_COUNT,
        sweep.len(),
        "a tabela de rows cresceu; acrescente o numero novo a esta varredura"
    );
    for (id, v, edit) in sweep {
        expect(&commit(player(), id, v), edit, &format!("{id:?}"));
    }
}

/// Os dois botões do estado cheio chegam ao barramento pelo ponteiro real.
#[test]
fn the_fit_and_remove_buttons_reach_the_bus() {
    expect(
        &click_real(player(), ph2d_panel_inspector::ids::INSP_PLAYER_FIT),
        PlayerFieldEdit::FitFloatHeight,
        "Fit to Collider",
    );
    expect(
        &click_real(player(), ph2d_panel_inspector::ids::INSP_PLAYER_REMOVE),
        PlayerFieldEdit::Remove,
        "Remove Platform Player",
    );
}

/// ⚠️ **O piso geométrico aparece no rótulo do botão que o resolve.**
///
/// Um controle, uma mensagem. O gate não lê o texto (o painel não o publica),
/// mas afirma a metade que importa: com a forma sem mínimo computável — uma
/// CAIXA, cuja fórmula é outra — o botão **não é oferecido**, porque oferecer um
/// ajuste que não sabe para onde ajustar é pior que não o oferecer.
#[test]
fn a_shape_without_a_known_floor_gets_no_fit_button() {
    let boxed = InspectorPlayerInfo {
        mode_tag: 0,
        reaction_is_live: true,
        push_is_live: true,
        min_float_known: false,
        ..player()
    };
    let rects = painted(boxed);
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_FIT),
        "sem mínimo computável não há para onde ajustar"
    );
    // E o resto da seção continua vivo — a ausência é do botão, não da seção.
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT)
    );
}

/// ⚠️ **As rows NÃO são write-only** — o espelho existe.
///
/// É a falha que a família de zonas shipou uma vez (W-AreaTorque): digitar
/// funciona, e **re-selecionar mostra o seed** em vez do que foi autorado. Um
/// controle que esquece o próprio valor é pior que um controle ausente, porque
/// ele mente sobre o estado do documento.
///
/// O gate pinta com um info cujos oito números são todos DIFERENTES do seed do
/// `populate` — sem isso ele ficaria verde por coincidência.
#[test]
fn the_rows_show_what_was_authored_not_the_seed() {
    let info = InspectorPlayerInfo {
        float_height: 1.11,
        cling_distance: 0.33,
        spring_strength: 321.0,
        spring_damping: 0.77,
        speed: 7.5,
        acceleration: 44.0,
        air_acceleration: 12.0,
        brake_scale: 0.42,
        max_slope_deg: 61.0,
        jump_height: 3.75,
        air_jumps: 3.0,
        air_jump_height: 0.8,
        takeoff_gravity: 1.6,
        takeoff_speed: 2.25,
        peak_gravity: 0.35,
        peak_speed: 2.75,
        fall_gravity: 2.25,
        cut_gravity: 5.5,
        reaction_support: 0.4,
        reaction_movement: 0.65,
        reaction_push: 0.85,
        ..player()
    };
    let mut host = host_aberto();
    let mut state = InspectorState::default();
    // ⚠️ **O NOME faz parte da fixture, e a premissa é declarada:** o
    // `entity_changed` do sync — a porta que decide re-semear as caixas — só
    // dispara com uma entidade selecionada, e ele a lê do transform/nome/
    // visibilidade, nunca do info da §14. Sem esta linha o gate mede um sync
    // que nunca rodou e fica verde-sobre-nada.
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Hero".into(),
    }));
    set_current_inspector_player(Some(info));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let got: Vec<f64> = [
        ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT,
        ph2d_panel_inspector::ids::INSP_PLAYER_CLING,
        ph2d_panel_inspector::ids::INSP_PLAYER_STIFFNESS,
        ph2d_panel_inspector::ids::INSP_PLAYER_DAMPING,
        ph2d_panel_inspector::ids::INSP_PLAYER_SPEED,
        ph2d_panel_inspector::ids::INSP_PLAYER_ACCEL,
        ph2d_panel_inspector::ids::INSP_PLAYER_AIR_ACCEL,
        ph2d_panel_inspector::ids::INSP_PLAYER_BRAKE,
        ph2d_panel_inspector::ids::INSP_PLAYER_MAX_SLOPE,
        ph2d_panel_inspector::ids::INSP_PLAYER_JUMP_HEIGHT,
        ph2d_panel_inspector::ids::INSP_PLAYER_AIR_JUMPS,
        ph2d_panel_inspector::ids::INSP_PLAYER_AIR_JUMP_H,
        ph2d_panel_inspector::ids::INSP_PLAYER_TAKEOFF_G,
        ph2d_panel_inspector::ids::INSP_PLAYER_TAKEOFF_SPEED,
        ph2d_panel_inspector::ids::INSP_PLAYER_PEAK_G,
        ph2d_panel_inspector::ids::INSP_PLAYER_PEAK_SPEED,
        ph2d_panel_inspector::ids::INSP_PLAYER_FALL_G,
        ph2d_panel_inspector::ids::INSP_PLAYER_CUT_G,
    ]
    .iter()
    .map(|&id| host.store().number_value(id).unwrap_or(f64::NAN))
    .collect();
    set_current_inspector_player(None);
    set_current_inspector_name(None);
    let want = [
        1.11, 0.33, 321.0, 0.77, 7.5, 44.0, 12.0, 0.42, 61.0, 3.75, 3.0, 0.8, 1.6, 2.25, 0.35,
        2.75, 2.25, 5.5,
    ];
    for (g, w) in got.iter().zip(want) {
        assert!(
            (g - w).abs() < 1.0e-4,
            "a row mostra {g} onde o documento diz {w} — ela e' WRITE-ONLY: {got:?}"
        );
    }
}

// ── W9: OS CARDS E AS DICAS ──────────────────────────────────────────────────

/// **Todo controle da §14 tem uma DICA de hover** (W9).
///
/// Enio, 2026-08-04: *"não entendo vários parâmetros, então não sei julgar"* /
/// *"precisamos de dicas no mouse hover"*.
///
/// ⚠️ A infra já existia (`WidgetStore::set_tooltip` + o passe de hover do
/// `hero`), então o modo de falha não é *"não funciona"*: é **não registrar
/// nada** e ficar exatamente igual a antes, em silêncio.
///
/// # ⚠️ O ORÁCULO é o que a §14 PINTA, e a versão anterior não era
///
/// Ele varria `player_control_ids()` — a mesma tabela de onde tirava a
/// expectativa (`PLAYER_ROW_COUNT + 5`) — e por isso era **cego a tudo o que
/// não estivesse nela**. Medido: a §14 pinta **70** ids próprios e aquela
/// varredura cobria **57**; os **15** restantes são os controles segmentados
/// (`Body`, `Emit Signals`, `Platform Lift`, `Walk Off Ledges`, o irmão
/// agachado) e as opções deles. Doze deles TÊM dica — por sorte, não por gate:
/// apagá-las passava verde.
///
/// ⚠️ **O comentário antigo já nomeava o mecanismo para os BOTÕES** (*"derivar
/// este número da tabela de dicas o tornaria um oráculo auto-referente"*) e não
/// notou que o **conjunto varrido inteiro** tinha o mesmo defeito.
///
/// A varredura agora é a DIFERENÇA entre o painel com player e sem ele — o que
/// esta seção acrescenta à tela —, e não uma lista que alguém mantém.
///
/// **Mutação que deve sangrar:** apagar o laço de `set_tooltip` do
/// `populate_player`, **ou** a dica de um único controle segmentado.
#[test]
fn every_player_control_carries_a_hover_hint() {
    // ⚠️ **Chrome do PAINEL, não controles da §14** — e a isenção é nominal e
    // com recusa de entrada morta, nunca um `unwrap_or(true)`: uma isenção que
    // sobrevive ao sítio que ela isenta é onde o próximo controle sem dica se
    // esconde.
    //
    // ⚠️ Nenhuma OUTRA seção do Inspector dá dica a estes três, então exigi-la
    // aqui seria esta seção a legislar para o painel inteiro.
    // ⭐⭐ **O scrollbar SAIU desta lista em 2026-09-14, e a metade de obsolescência foi quem o
    // tirou.** A isenção dizia *«ele nasce porque a §14 estica o painel»* — e quando o rótulo das
    // rows passou a ficar à ESQUERDA (`widget::property_row_columns`), cada linha do Inspector
    // encolheu ~16 px, a §14 deixou de esticar o painel para além da altura dele, e o scrollbar
    // deixou de nascer. O gate reprovou a dizer *«isenção STALE»*, que é exactamente o trabalho
    // dela. ⇒ *a prova de que a conversão encurtou o painel não foi uma medição minha: foi uma
    // isenção a deixar de descrever alguma coisa.*
    // ⚠️ A pega TEM dica (o `pre_populate` dá-lha a todas); fica aqui como cromo porque é do
    //    cabeçalho e não da §14.
    // ⚠️ A BARRA DE ROLAGEM do painel esteve aqui de 2026-09-29 a 2026-09-30 e SAIU pela metade
    //    «stale» deste gate: o que fazia esta fixtura transbordar era o plano fechar um cartão
    //    antes de CADA secção da lista — no tema clássico, um separador inteiro por secção vazia.
    //    Com o `section_plan::Corredor` o corpo cabe, e a barra não aparece.
    // ⭐ **As DOBRAS saíram desta lista (GEF 05)**: cada uma diz em Plain English o que mora dentro,
    // como a dobra More do Top-Down.
    let chrome: Vec<(&str, ph2d_a11y::NodeId)> = vec![
        ("o cabecalho da secao", ids::INSP_LIVE_PLAYER_SECTION),
        ("a pega de arrasto do cabecalho", ids::INSP_LIVE_PLAYER_GRIP),
    ];

    let with_player = painted(player());
    let without: Vec<ph2d_a11y::NodeId> = {
        let mut host = host_aberto();
        let mut state = InspectorState::default();
        set_current_inspector_player(None);
        host.paint::<InspectorPanel>(&mut state, VIEWPORT)
            .into_iter()
            .map(|(id, _)| id)
            .collect()
    };
    let mut host = host_aberto();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(player()));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);

    let mine: std::collections::BTreeSet<_> = with_player
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| !without.contains(id))
        .collect();
    // ⚠️ Controle positivo: uma diferença vazia é um gate verde sobre qualquer
    // defeito, e ela acontece no dia em que o `painted` mudar de forma.
    assert!(
        mine.len() > 60,
        "a §14 tem de acrescentar dezenas de controles a' tela (viu {}) -- uma          varredura vazia passa VERDE sobre tudo",
        mine.len()
    );

    let mut used = vec![false; chrome.len()];
    let mut naked = Vec::new();
    for id in &mine {
        if let Some(i) = chrome.iter().position(|(_, c)| c == id) {
            used[i] = true;
            continue;
        }
        match host.store().tooltip_for(*id) {
            Some(tip) if tip.len() > 12 => {}
            Some(tip) => naked.push(format!("{id:?}: a dica e' curta demais: {tip:?}")),
            None => naked.push(format!("{id:?}: sem dica de hover")),
        }
    }
    set_current_inspector_player(None);

    assert!(
        naked.is_empty(),
        "controle da §14 que o artista nao consegue entender:\n  {}",
        naked.join("\n  ")
    );
    let stale: Vec<_> = chrome
        .iter()
        .zip(&used)
        .filter(|(_, u)| !**u)
        .map(|((why, _), _)| *why)
        .collect();
    assert!(
        stale.is_empty(),
        "isencao STALE -- o sitio isento ja' nao e' pintado, entao a linha sai:\n  {}",
        stale.join("\n  ")
    );
}

/// **Os cinco cards são PINTADOS, e as rows caem dentro deles** (W9).
///
/// ⚠️ O oráculo é a GEOMETRIA, não a existência de um id: um card é moldura, e
/// uma moldura que não contém o que emoldura é pior que nenhuma. O gate afirma
/// que cada row de um card está entre a primeira e a última row daquele card, e
/// que os cards não se cruzam — que é a forma de *"as caixas vazaram para fora
/// da caixa"* aparecer sem screenshot.
///
/// **Mutação que deve sangrar:** o pintor voltar a somar as rows em vez de usar
/// o `next_y` da moldura (as caixas passam a vazar do card).
#[test]
fn every_card_holds_its_own_rows_and_they_do_not_overlap() {
    let rects = painted(player());
    // ⚠️ **Controle positivo:** o laço abaixo percorre um `Vec` — uma lista vazia
    // não entra nele, e o gate passaria VERDE sem ter olhado para card nenhum.
    // (Os irmãos que iteram `INSP_PLAYER_MODE_IDS` não precisam disto: aquilo é
    // `[NodeId; 3]`, e encolher um array de tamanho fixo é erro de compilação —
    // ali o controle é o `rustc`.)
    let cards = ph2d_panel_inspector::player_card_spans();
    assert!(
        cards.len() >= 10,
        "a §14 tem uma dezena de cards (viu {}) -- uma varredura vazia e' verde \
         por vacuo",
        cards.len()
    );
    let mut spans: Vec<(f32, f32)> = Vec::new();
    for (title, rows) in cards {
        let ys: Vec<f32> = rows
            .iter()
            .map(|id| {
                rects
                    .iter()
                    .find(|(n, _)| n == id)
                    .map(|(_, r)| r.y)
                    .unwrap_or_else(|| panic!("o card {title} nao pintou {id:?}"))
            })
            .collect();
        let lo = ys.iter().cloned().fold(f32::INFINITY, f32::min);
        let hi = ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        // ⚠️ **A afirmação é DISTINÇÃO, e não `hi > lo`, e a troca é uma
        // correção do gate, não uma concessão a esta wave.** O `hi > lo` era um
        // proxy com dois defeitos: ele passa com três rows das quais DUAS
        // partilham o `y` (só apanha o colapso total), e é **impossível de
        // satisfazer por um card de UMA row** — onde `lo == hi` por aritmética,
        // sem que nada esteja errado. O card `GLIDE` é o primeiro de uma row
        // deste painel e foi ele que o expôs.
        for (a, ya) in ys.iter().enumerate() {
            for yb in ys.iter().skip(a + 1) {
                assert!(
                    (ya - yb).abs() > f32::EPSILON,
                    "o card {title} empilhou duas rows no mesmo y ({ya}) — a régua da \
                     moldura e a das rows discordam"
                );
            }
        }
        spans.push((lo, hi));
    }
    assert_eq!(spans.len(), cards_len(), "cada card da tabela foi medido");

    // ⚠️ **Desenho 03_character (10/11): os cards da TABELA já não são as caixas da
    // tela** — Walk e Jump partem-se em cartão + dobra «More», e o Body foi para
    // Advanced, no fim. A ordem da tabela deixou de ser a da tela, e o passo
    // «card a card» (`player_card_pitch`) deixou de ter par na tela. A prova de
    // «as caixas vazaram» passa a ser sobre o que a tela pinta: **nenhuma row do
    // player cruza outra**, em lado nenhum.
    let mut boxes: Vec<(ph2d_a11y::NodeId, Rect)> = ph2d_panel_inspector::player_card_spans()
        .into_iter()
        .flat_map(|(_, rows)| rows)
        .filter_map(|id| rects.iter().find(|(n, _)| *n == id).copied())
        .collect();
    boxes.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    for (i, (a, ra)) in boxes.iter().enumerate() {
        for (b, rb) in boxes.iter().skip(i + 1) {
            let cruza = ra.x < rb.x + rb.w
                && rb.x < ra.x + ra.w
                && ra.y < rb.y + rb.h
                && rb.y < ra.y + ra.h;
            assert!(
                !cruza,
                "duas rows do player se cruzam: {a:?} {ra:?} e {b:?} {rb:?}"
            );
        }
    }
}

fn cards_len() -> usize {
    ph2d_panel_inspector::player_card_spans().len()
}

/// **O botão de descartar a corrida existe SE, e SOMENTE SE, existe corrida**
/// (W17) — e clicá-lo chega ao barramento.
///
/// ⚠️ **As duas metades juntas, e a de AUSÊNCIA é a que carrega o desenho:** a
/// fita é a única coisa da §14 cujo readout é a própria existência do controle.
/// Sem corrida não há o que descartar, e um botão pintado sobre nada seria o
/// controle-morto que esta seção varre a cada wave.
///
/// ⚠️ E o número vai no RÓTULO, não num readout ao lado — o precedente do
/// `Fit to Collider (needs > 0.50 m)`, pintado dez linhas acima dele: *o aviso
/// mora no rótulo do próprio controle que o resolve*.
#[test]
fn the_clear_run_button_appears_only_when_a_run_exists() {
    let rects = painted(player());
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_CLEAR_RUN),
        "sem corrida gravada, o botao de descartar foi pintado sobre nada"
    );

    let with_run = InspectorPlayerInfo {
        recorded_run_seconds: 1.5,
        ..player()
    };
    expect(
        &click_real(with_run, ph2d_panel_inspector::ids::INSP_PLAYER_CLEAR_RUN),
        PlayerFieldEdit::ClearRun,
        "Clear Recorded Run",
    );
}

/// **O `Fit Crouch` existe SÓ com o agachar armado, e o piso vai no ROTULO**
/// (W18).
///
/// ⚠️ **Três estados, e o do meio é o que a wave entrega:** desligado (`0`) não há
/// botão — não há defeito a consertar e um botão que ligasse a capacidade pelas
/// costas conflataria dois gestos; ARMADO E ABAIXO do piso é onde o defeito vive
/// (o slider está morto, e nada na tela diz por quê); armado e acima, o botão
/// segue lá com o rótulo curto, como o `Fit to Collider` da perna de pé.
///
/// ⚠️ O gate **não lê o texto** (o painel não o publica) — ele afirma as
/// presenças, que é a metade que decide se o artista tem por onde descobrir o
/// número.
#[test]
fn the_crouch_fit_button_appears_only_when_the_crouch_is_armed() {
    // Desligado: nenhum botão.
    let off = InspectorPlayerInfo {
        crouch_height: 0.0,
        ..player()
    };
    assert!(
        !painted(off)
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_FIT_CROUCH),
        "o agachar esta' DESLIGADO e o botao de ajuste foi pintado sobre nada"
    );

    // Armado e abaixo do piso — o estado que a wave existe para nomear.
    let buried = InspectorPlayerInfo {
        crouch_height: 0.30,
        min_float_height: 0.58,
        mode_tag: 0,
        reaction_is_live: true,
        push_is_live: true,
        min_float_known: true,
        ..player()
    };
    expect(
        &click_real(buried, ph2d_panel_inspector::ids::INSP_PLAYER_FIT_CROUCH),
        PlayerFieldEdit::FitCrouchHeight,
        "Fit Crouch to Collider",
    );

    // Armado e acima: o botão fica, com o rótulo curto.
    let fine = InspectorPlayerInfo {
        crouch_height: 0.80,
        min_float_height: 0.58,
        mode_tag: 0,
        reaction_is_live: true,
        push_is_live: true,
        min_float_known: true,
        ..player()
    };
    assert!(
        painted(fine)
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_FIT_CROUCH),
        "com o agachar acima do piso o ajuste sumiu -- ele continua sendo o gesto \
         que responde *quao fundo este corpo consegue agachar*"
    );
}

/// **E uma forma sem piso computável não ganha o ajuste** — o irmão exato do
/// `a_shape_without_a_known_floor_gets_no_fit_button`.
///
/// Uma CAIXA tem outra fórmula (`half_y + half_x·tan θ`), e oferecer um ajuste
/// que não sabe para onde ajustar é pior que não o oferecer.
#[test]
fn a_shape_without_a_known_floor_gets_no_crouch_fit_either() {
    let boxed = InspectorPlayerInfo {
        crouch_height: 0.30,
        mode_tag: 0,
        reaction_is_live: true,
        push_is_live: true,
        min_float_known: false,
        ..player()
    };
    assert!(
        !painted(boxed)
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_FIT_CROUCH),
        "uma caixa recebeu um ajuste de agachar que nao sabe para onde ajustar"
    );
}

/// **DESCARTAR TEM VOLTA, e os dois botões nunca coexistem** (W24).
///
/// ⚠️ São TRÊS afirmações, e a segunda é a que importa: o botão de devolver é
/// **pintado, hittable e levanta o próprio verbo** pelo `click_at` REAL — sem
/// ela a corrida descartada ficaria guardada e inalcançável, que é o mesmo que
/// perdida.
///
/// A exclusividade é **por construção** (descartar esvazia a fita viva), e o
/// gate a afirma nos três estados: só corrida viva · só descartada · nenhuma.
#[test]
fn discarding_a_run_can_be_undone_and_the_two_buttons_never_coexist() {
    let runs = |live: f32, gone: f32| InspectorPlayerInfo {
        recorded_run_seconds: live,
        discarded_run_seconds: gone,
        ..player()
    };
    let has = |info: InspectorPlayerInfo, id: ph2d_a11y::NodeId| {
        painted(info).iter().any(|(n, _)| *n == id)
    };

    // 1. Corrida VIVA: o botao e' o de descartar.
    assert!(
        has(
            runs(4.0, 0.0),
            ph2d_panel_inspector::ids::INSP_PLAYER_CLEAR_RUN
        ),
        "com corrida viva o botao de descartar tem de estar na tela"
    );
    assert!(
        !has(
            runs(4.0, 0.0),
            ph2d_panel_inspector::ids::INSP_PLAYER_RESTORE_RUN
        ),
        "e o de devolver NAO"
    );

    // 2. Corrida DESCARTADA: o mesmo lugar oferece o caminho de volta, e o
    //    clique REAL levanta o verbo.
    assert!(
        has(
            runs(0.0, 4.0),
            ph2d_panel_inspector::ids::INSP_PLAYER_RESTORE_RUN
        ),
        "com corrida descartada o botao de devolver tem de estar na tela"
    );
    assert!(
        !has(
            runs(0.0, 4.0),
            ph2d_panel_inspector::ids::INSP_PLAYER_CLEAR_RUN
        ),
        "e o de descartar NAO -- nao ha' o que descartar"
    );
    expect(
        &click_real(
            runs(0.0, 4.0),
            ph2d_panel_inspector::ids::INSP_PLAYER_RESTORE_RUN,
        ),
        PlayerFieldEdit::RestoreRun,
        "devolver a corrida descartada",
    );

    // 3. Nenhuma das duas: nenhum botao. A ausencia e' o outro readout.
    assert!(!has(
        runs(0.0, 0.0),
        ph2d_panel_inspector::ids::INSP_PLAYER_CLEAR_RUN
    ));
    assert!(!has(
        runs(0.0, 0.0),
        ph2d_panel_inspector::ids::INSP_PLAYER_RESTORE_RUN
    ));

    // 4. ⚠️ **As DUAS cheias — o estado que a porta NAO produz** (W25).
    //
    //    A exclusividade tem duas camadas: a troca (`run_stash`) deixa sempre
    //    uma das fitas vazia, e o paint escolhe UMA mesmo que lhe entreguem as
    //    duas. Sem este caso a segunda camada nao tem gate — trocar o `else if`
    //    por dois `if` independentes **sobrevive** a todo estado alcancavel, o
    //    que foi medido no painel de MUNDO (a outra vista desta corrida) e vale
    //    aqui pela mesma razao.
    //
    //    Quem vence e' a corrida VIVA: devolver por cima dela apagaria o que o
    //    artista acabou de gravar.
    assert!(
        has(
            runs(4.0, 4.0),
            ph2d_panel_inspector::ids::INSP_PLAYER_CLEAR_RUN
        ),
        "com as duas cheias quem vence e' a corrida VIVA"
    );
    assert!(
        !has(
            runs(4.0, 4.0),
            ph2d_panel_inspector::ids::INSP_PLAYER_RESTORE_RUN
        ),
        "e devolver nao pode ser oferecido por cima de uma corrida viva"
    );
}

/// **O chip do MODO é pintado, vivo sob o mouse, e cada opção levanta o SEU
/// tag** (W-KinMove).
///
/// ⚠️ **`click_real`, e é a razão de este arquivo existir:** o `seg_row` registra
/// as opções num LAÇO, que é o ponto cego do `architecture_panel_wiring_parity`
/// (ele coleta `.register(ids::<LITERAL>`). Sem esta varredura, tirar o grupo do
/// `populate` deixa os dois chips pintados, hit-registrados e MORTOS, com a
/// suíte inteira verde — o buraco das 36 células do W2c.
#[test]
fn the_mode_chip_is_alive_and_each_option_raises_its_own_tag() {
    for (i, &id) in ph2d_panel_inspector::ids::INSP_PLAYER_MODE_IDS
        .iter()
        .enumerate()
    {
        let acts = click_real(player(), id);
        assert!(
            acts.iter().any(|a| matches!(
                a,
                EditorAction::InspectorPlayerEdit { edit: PlayerFieldEdit::Mode(t), .. }
                    if *t == i as u8
            )),
            "a opcao {i} do chip de modo tem de levantar Mode({i}): {acts:?}"
        );
    }
}

/// **E ele é oferecido em TODO modo** — inclusive já cinemático, que é o caminho
/// de volta.
///
/// ⚠️ Sem esta metade, um artista que escolhe `Kinematic` fica sem o controle que
/// o traria de volta: a quarta condição de UI (*a sequência leva a algum lugar*)
/// falha de um lado só, e um gate que só testa a ida não a vê.
#[test]
fn the_mode_chip_is_painted_in_both_modes() {
    for tag in [0u8, 1, 2] {
        let mut info = player();
        info.mode_tag = tag;
        // O *puro sangue* é o único cujo mundo não o ouve — e o chip que o
        // desfaz tem de continuar na tela, senão ele é um beco sem saída.
        info.reaction_is_live = tag != 2;
        info.push_is_live = tag == 1;
        let rects = painted(info);
        for &id in &ph2d_panel_inspector::ids::INSP_PLAYER_MODE_IDS {
            assert!(
                rects.iter().any(|(n, _)| *n == id),
                "com mode_tag {tag} o chip {id:?} tem de estar na tela"
            );
        }
    }
}

/// **O card da 3ª LEI é oferecido a quem a tem, e SÓ a quem a tem** (W-KinPure).
///
/// ⚠️ A metade da AUSÊNCIA é a que carrega a wave: sob o *puro sangue* os três
/// escalares não são lidos por ninguém, e a lei deste módulo é que um controle
/// que o solver ignora **não existe**. A metade da PRESENÇA é o que impede a
/// cura de virar *"o card sumiu"*.
#[test]
fn the_reaction_card_follows_who_is_heard_by_the_world() {
    const REACT: [ph2d_a11y::NodeId; 3] = [
        ph2d_panel_inspector::ids::INSP_PLAYER_REACT_SUPPORT,
        ph2d_panel_inspector::ids::INSP_PLAYER_REACT_MOVEMENT,
        ph2d_panel_inspector::ids::INSP_PLAYER_REACT_PUSH,
    ];

    let mut heard = player();
    heard.reaction_is_live = true;
    let rects = painted(heard);
    for id in REACT {
        assert!(
            rects.iter().any(|(n, _)| *n == id),
            "quem o mundo OUVE tem o card: {id:?} faltou"
        );
    }

    let mut scenery = player();
    scenery.mode_tag = 2;
    scenery.reaction_is_live = false;
    scenery.push_is_live = false;
    let rects = painted(scenery);
    for id in REACT {
        assert!(
            !rects.iter().any(|(n, _)| *n == id),
            "sob o puro sangue o card nao existe: {id:?} foi pintado e esta' MORTO"
        );
    }
    // ⚠️ E o resto da seção continua lá — esconder UM card não é esconder a
    // ferramenta (sem esta linha, um `return` no topo do laço passaria).
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_SPEED),
        "os outros cards seguem pintados"
    );
}

/// **As rows da MOLA seguem a perna que é uma mola** (auditoria de 2026-08-15).
///
/// ⚠️ **A metade da AUSÊNCIA é a wave:** sob a perna que POUSA (`Support::Snap`,
/// o modo cinemático) a lei sobrescreve o `float_height` pela geometria do corpo
/// **antes de o motor o ver**, e a rigidez e o amortecimento não têm mola que os
/// consuma — três números que o artista afina e que ninguém lê, mais um botão
/// cujo efeito a lei apaga no mesmo tique.
///
/// ⚠️ **E a metade da PRESENÇA é sobre o card, não sobre as rows:** a `Cling
/// Distance` fica, porque sob Snap ela é o `snap_distance` E o `step_height` do
/// controlador — o número mais vivo da seção. Sem esta asserção a cura
/// preguiçosa (esconder o card LEG inteiro, o precedente do `Pure`) passaria, e
/// levaria embora o único controle que ali funciona.
#[test]
fn the_spring_rows_follow_the_leg_that_is_a_spring() {
    const SPRING_ONLY: [ph2d_a11y::NodeId; 3] = [
        ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT,
        ph2d_panel_inspector::ids::INSP_PLAYER_STIFFNESS,
        ph2d_panel_inspector::ids::INSP_PLAYER_DAMPING,
    ];

    // A perna elástica: as três rows E o botão de ajuste na tela.
    let mut sprung = player();
    sprung.spring_is_live = true;
    let rects = painted(sprung);
    for id in SPRING_ONLY {
        assert!(
            rects.iter().any(|(n, _)| *n == id),
            "com perna de MOLA a row {id:?} tem de estar na tela"
        );
    }
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_FIT),
        "e o botao que conserta a altura tambem"
    );

    // A perna que pousa: as três somem, e o card sobrevive pela `Cling`.
    let mut snapped = player();
    snapped.mode_tag = 1;
    snapped.spring_is_live = false;
    let rects = painted(snapped);
    for id in SPRING_ONLY {
        assert!(
            !rects.iter().any(|(n, _)| *n == id),
            "sob a perna que POUSA a row {id:?} foi pintada e esta' MORTA"
        );
    }
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_FIT),
        "e o botao escreveria um numero que a lei sobrescreve no mesmo tique"
    );
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_CLING),
        "⚠️ a Cling Distance FICA -- sob Snap ela e' o snap_distance E o \
         step_height do controlador, e esconder o card levaria o unico \
         controle que ali funciona"
    );
    // E o resto da seção continua lá: esconder rows não é esconder a ferramenta.
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_SPEED),
        "os outros cards seguem pintados"
    );
}

/// **O `Push on Bodies` só é OFERECIDO a quem o lê** (report do Enio,
/// 2026-08-09: *"para Dynamic parece não funcionar; se não se aplica deve ser
/// retirado do painel, assim como qualquer input morto"*).
///
/// ⚠️ **E os DOIS VIZINHOS são o controle, não decoração:** *Weight on Ground* e
/// *Push on Ground* são aplicados pela MOLA e portanto vivos em Dynamic. Sem
/// eles no gate, a cura preguiçosa — esconder o card inteiro, que é o que o
/// precedente do `Pure` faz — passaria, e tiraria do artista dois controles que
/// funcionam.
///
/// A terceira asserção é a que impede o oposto: escondê-lo TAMBÉM no cinemático
/// seria trocar um knob morto por um knob que falta.
#[test]
fn the_shove_is_offered_only_to_the_mode_that_reads_it() {
    let mut dynamic = player();
    dynamic.mode_tag = 0;
    dynamic.reaction_is_live = true;
    dynamic.push_is_live = false;
    let rects = painted(dynamic);
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_REACT_PUSH),
        "em Dynamic o empurrao lateral nao e' lido por ninguem, e um slider \
         inerte ensina a desconfiar dos outros"
    );
    for live in [
        ph2d_panel_inspector::ids::INSP_PLAYER_REACT_SUPPORT,
        ph2d_panel_inspector::ids::INSP_PLAYER_REACT_MOVEMENT,
    ] {
        assert!(
            rects.iter().any(|(n, _)| *n == live),
            "os vizinhos sao aplicados pela MOLA e continuam vivos em Dynamic: \
             {live:?} sumiu -- esconder o card inteiro nao e' a cura"
        );
    }

    let mut kinematic = player();
    kinematic.mode_tag = 1;
    let rects = painted(kinematic);
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_REACT_PUSH),
        "e no modo que o LE' ele tem de estar la'"
    );
}

// ── A SAÍDA (`W-PlayerOut`, A3) ──────────────────────────────────────────────

/// **O chip `Emit Signals` pinta, regista, e o clique CHEGA ao barramento.**
///
/// ⚠️ **As duas direções**, e não só a de ligar: um chip cujo `Off` não despacha
/// é um opt-in que não se desfaz, e o artista fica com toasts para sempre.
///
/// ⚠️ Desenho 03_character (10/11): o seletor `Off|On` virou a CAIXA
/// `INSP_PLAYER_EMIT_BOX` (evento `Toggled`); a prova das duas direções é a mesma.
#[test]
fn the_emit_signals_chip_reaches_the_bus_in_both_directions() {
    expect(
        &toggle_real(player(), ph2d_panel_inspector::ids::INSP_PLAYER_EMIT_BOX),
        PlayerFieldEdit::EmitSignals(true),
        "ligar a saída de sinais",
    );
    expect(
        &toggle_real(emitting(), ph2d_panel_inspector::ids::INSP_PLAYER_EMIT_BOX),
        PlayerFieldEdit::EmitSignals(false),
        "desligar a saída de sinais",
    );
}

/// **Cada caixa de habilidade chega ao barramento, nas duas direções e com a
/// SUA habilidade** — um índice trocado entre `INSP_PLAYER_ABILITY` e
/// `PlayerAbility::ALL` ligaria outra coisa com tudo pintado e vivo.
#[test]
fn every_ability_box_reaches_the_bus_in_both_directions() {
    use ph2d_editor_core::screens::hero::PlayerAbility;
    for (i, &id) in ph2d_panel_inspector::ids::INSP_PLAYER_ABILITY
        .iter()
        .enumerate()
    {
        let a = PlayerAbility::ALL[i];
        let off = InspectorPlayerInfo {
            abilities: [false; 7],
            ..player()
        };
        expect(
            &toggle_real(off, id),
            PlayerFieldEdit::Ability(a, true),
            "marcar",
        );
        expect(
            &toggle_real(player(), id),
            PlayerFieldEdit::Ability(a, false),
            "desmarcar",
        );
    }
}

/// O clique REAL numa caixa: tem de produzir o `Toggled` dela (focável e viva).
pub(crate) fn toggle_real(info: InspectorPlayerInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = host_aberto();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("a §14 nunca pintou a caixa {id:?}"));
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Toggled(c) if *c == id)),
        "clicar no meio de {id:?} produziu {events:?} — a caixa está morta sob o mouse"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_player(None);
    out
}

/// **O CHIP DA POLÍTICA DE PLATAFORMA chega ao barramento, nas TRÊS opções**
/// (`W-Leave`).
///
/// ⚠️ **As três, e não só a que muda o comportamento:** um chip cujo `Full` não
/// despacha é uma escolha que não se desfaz, e o artista fica preso na política
/// que experimentou. É a mesma metade que o irmão do `Emit Signals` afirma.
///
/// ⚠️ **O índice É o `PlatformLift::tag`**, e o gate o afirma: uma tabela de
/// rótulos e uma lista de ids que discordassem sobre a ordem dariam um chip que
/// seleciona outra coisa, com tudo pintado e vivo.
#[test]
fn the_platform_lift_chip_reaches_the_bus_in_every_option() {
    for (i, tag) in [(0usize, "Full"), (1, "Up Only"), (2, "None")] {
        expect(
            &click_real(
                player(),
                ph2d_panel_inspector::ids::INSP_PLAYER_LIFT_POLICY_IDS[i],
            ),
            PlayerFieldEdit::PlatformLift(i as u8),
            tag,
        );
    }
}

/// **OS DOIS CHIPS DA TRAVA DE BEIRADA chegam ao barramento** (`W-Brink`).
///
/// ⚠️ **`0` é *pode andar para fora*, e o gate o afirma:** o campo guarda a
/// CAPACIDADE, então o índice `0` do chip tem de mandar `true`. Uma tabela de
/// rótulos que dissesse o contrário daria um chip que arma a trava quando o
/// artista a desarma — pintado, vivo, e a fazer o oposto.
#[test]
fn the_walk_off_ledges_chips_reach_the_bus_in_every_option() {
    for (i, allowed, tag) in [(0usize, true, "Yes"), (1, false, "Stop At Edge")] {
        expect(
            &click_real(
                player(),
                ph2d_panel_inspector::ids::INSP_PLAYER_WALK_OFF_IDS[i],
            ),
            PlayerFieldEdit::WalkOffLedges(allowed),
            tag,
        );
        expect(
            &click_real(
                player(),
                ph2d_panel_inspector::ids::INSP_PLAYER_CROUCH_WALK_OFF_IDS[i],
            ),
            PlayerFieldEdit::CrouchWalkOffLedges(allowed),
            tag,
        );
    }
}

/// **A row do AGACHAR só é oferecida com o agachar autorado** — sem ele a
/// [`ph2d_platformer::walk_for`] devolve a config de pé e o número nunca é lido:
/// a row seria um controle que não pode agir.
///
/// ⚠️ As duas metades no mesmo teste: a ausência sozinha passaria com uma row
/// que nunca é pintada, e a presença sozinha com uma que é pintada sempre.
#[test]
fn the_crouching_row_is_offered_only_where_crouching_is_authored() {
    let armed = painted(player());
    let no_crouch = painted(InspectorPlayerInfo {
        crouch_armed: false,
        ..player()
    });
    let has = |rects: &[(ph2d_a11y::NodeId, Rect)]| {
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_CROUCH_WALK_OFF_IDS[0])
    };
    assert!(has(&armed), "com o agachar autorado a row e' pintada");
    assert!(
        !has(&no_crouch),
        "sem ele a row seria um controle que a lei nunca le'"
    );
    // E a row de PE' fica nas duas — ela nao depende do agachar.
    let standing = |rects: &[(ph2d_a11y::NodeId, Rect)]| {
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_WALK_OFF_IDS[0])
    };
    assert!(standing(&armed) && standing(&no_crouch));
}

/// **O readout vive na face COM o comportamento, e some da face vazia.**
///
/// ⚠️ Ele não tem id — *um readout que despacha mente* —, então o oráculo não
/// pode ser um retângulo de hit. O que se afirma é o que ele DESLOCA: um bloco
/// de três linhas empurra tudo o que vem depois, e a face vazia não o tem.
#[test]
fn the_live_readout_takes_room_only_where_there_is_a_player() {
    let with = painted(emitting());
    let empty_face = painted(empty());
    let mode_y = |rects: &[(ph2d_a11y::NodeId, Rect)]| {
        rects
            .iter()
            .find(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_MODE_IDS[0])
            .map(|(_, r)| r.y)
    };
    assert!(
        mode_y(&with).is_some(),
        "a face COM player pinta o chip de modo"
    );
    assert!(
        mode_y(&empty_face).is_none(),
        "sem player o painel nao pinta nada — a guarda da F3 (a face vazia morreu)"
    );

    // ⚠️ Desenho 03_character (10/11): a caixa da saída mora em Advanced, no FIM
    // (antes, um chip acima dos cards). O que fica é: com player ela é pintada,
    // na face vazia não.
    let emit = |rects: &[(ph2d_a11y::NodeId, Rect)]| {
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_EMIT_BOX)
    };
    assert!(emit(&with), "a caixa da saída é pintada com o player vivo");
    assert!(!emit(&empty_face), "e não na face vazia");
}

/// ⭐ **Cada dobra do Platform Player abre pelo CLIQUE no título** — o smoke do dono (10/11) achou-as
/// todas mortas: pintadas e registadas, mas o store não as marcava como secção dobrável.
#[test]
fn cada_dobra_do_player_abre_pelo_clique_no_titulo() {
    for id in ph2d_panel_inspector::ids::INSP_PLAYER_FOLDS {
        let mut host = crate::host_em_metros();
        for outra in ph2d_panel_inspector::ids::INSP_PLAYER_FOLDS {
            host.set_collapsed(outra, outra != id);
        }
        // A Wall Sensor mora dentro de Walls: Walls aberta para ela aparecer.
        if id == ph2d_panel_inspector::ids::INSP_PLAYER_FOLD_WALL_SENSOR {
            host.set_collapsed(ph2d_panel_inspector::ids::INSP_PLAYER_CARD_WALL, false);
        }
        host.set_collapsed(id, true);
        let mut state = InspectorState::default();
        set_current_inspector_player(Some(player()));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        let rect = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("a dobra {id:?} não foi pintada"));
        for ev in host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5) {
            let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
        }
        let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_player(None);
        assert!(
            !host.store().is_collapsed(id),
            "clicar no título de {id:?} não a abriu"
        );
    }
}

/// ⭐ **A fechar, nenhum campo de uma dobra fica por cima do que sobe** (smoke do dono 10/11): o campo curto
/// que o fluxo dos cartões guarda «à espera» do vizinho era pintado depois do `finish` da dobra, fora do
/// recorte, e ficava parado sobre o título seguinte durante a animação.
#[test]
fn a_fechar_nenhum_campo_da_dobra_passa_por_cima_do_titulo_seguinte() {
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    use ph2d_panel_inspector::ids as i;
    let dobras = dobras_do_player();
    let janela = Rect::new(0.0, 0.0, 1600.0, 3200.0);
    let layout = || {
        HeroLayout::for_viewport_bands(
            janela,
            false,
            ChromeBands::DEFAULT,
            CenterSplit::None,
            DockSides::BOTH,
        )
    };
    ph2d_panel_inspector::set_cards_on(true);
    let mut vistos = 0;
    let mut falhas = Vec::new();
    for (dobra, dentro, seguinte) in dobras {
        let mut host = crate::host_em_metros();
        host.store_mut()
            .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION, false);
        for id in i::INSP_PLAYER_FOLDS {
            host.set_collapsed(id, id != dobra);
        }
        let mut state = InspectorState::default();
        set_current_inspector_player(Some(player()));
        // Aberta e assentada: a dobra lembra a altura do corpo, que é a régua do recorte.
        let _ = crate::o_cartao_da_fisica::pinta(&mut host, &mut state);
        for t in [0.75, 0.5, 0.25] {
            host.set_collapsed(dobra, true);
            host.store_mut().set_section_open_live(dobra, t);
            let rects = host.paint_with_layout::<InspectorPanel>(&mut state, layout(), janela);
            let Some(titulo) = crate::o_cartao_da_fisica::achado(&rects, seguinte) else {
                falhas.push(format!(
                    "{dobra:?}: o título seguinte {seguinte:?} não foi pintado"
                ));
                continue;
            };
            for &id in dentro {
                let Some(r) = crate::o_cartao_da_fisica::achado(&rects, id) else {
                    continue;
                };
                vistos += 1;
                if r.y + r.h > titulo.y + 0.5 {
                    falhas.push(format!(
                        "{dobra:?} a {t:.2}: o campo {id:?} (fim {:.1}) passa o título seguinte ({:.1})",
                        r.y + r.h,
                        titulo.y
                    ));
                }
            }
        }
        set_current_inspector_player(None);
    }
    ph2d_panel_inspector::set_cards_on(false);
    assert!(
        vistos > 0,
        "a régua não viu campo nenhum: está a ler o sítio errado"
    );
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Cada dobra do player, as rows de dentro e o título que vem logo abaixo dela.
fn dobras_do_player() -> [(
    ph2d_a11y::NodeId,
    &'static [ph2d_a11y::NodeId],
    ph2d_a11y::NodeId,
); 11] {
    use ph2d_a11y::NodeId;
    use ph2d_panel_inspector::ids as i;
    let d: [(NodeId, &'static [NodeId], NodeId); 11] = [
        (
            i::INSP_PLAYER_FOLD_WALK_MORE,
            &[
                i::INSP_PLAYER_AIR_ACCEL,
                i::INSP_PLAYER_BRAKE,
                i::INSP_PLAYER_MAX_SLOPE,
            ],
            i::INSP_PLAYER_FOLD_JUMP_MORE,
        ),
        (
            i::INSP_PLAYER_FOLD_JUMP_MORE,
            &[
                i::INSP_PLAYER_AIR_JUMP_H,
                i::INSP_PLAYER_TAKEOFF_G,
                i::INSP_PLAYER_TAKEOFF_SPEED,
                i::INSP_PLAYER_PEAK_G,
                i::INSP_PLAYER_PEAK_SPEED,
                i::INSP_PLAYER_FALL_G,
                i::INSP_PLAYER_CUT_G,
            ],
            i::INSP_PLAYER_CARD_WALL,
        ),
        (
            i::INSP_PLAYER_CARD_WALL,
            &[
                i::INSP_PLAYER_WALL_SLIDE,
                i::INSP_PLAYER_WALL_JUMP,
                i::INSP_PLAYER_WALL_PUSH,
                i::INSP_PLAYER_WALL_LOCK,
                i::INSP_PLAYER_WALL_GRAB,
            ],
            i::INSP_PLAYER_CARD_DASH,
        ),
        (
            i::INSP_PLAYER_CARD_DASH,
            &[
                i::INSP_PLAYER_DASH_SPEED,
                i::INSP_PLAYER_DASH_TIME,
                i::INSP_PLAYER_DASH_COOL,
            ],
            i::INSP_PLAYER_CARD_CROUCH,
        ),
        (
            i::INSP_PLAYER_CARD_CROUCH,
            &[i::INSP_PLAYER_CROUCH_HEIGHT, i::INSP_PLAYER_CROUCH_SPEED],
            i::INSP_PLAYER_CARD_SWIM,
        ),
        (
            i::INSP_PLAYER_CARD_SWIM,
            &[
                i::INSP_PLAYER_SWIM_SPEED,
                i::INSP_PLAYER_SWIM_ACCEL,
                i::INSP_PLAYER_SWIM_ENTER,
            ],
            i::INSP_PLAYER_CARD_LEDGE,
        ),
        (
            i::INSP_PLAYER_CARD_LEDGE,
            &[
                i::INSP_PLAYER_LEDGE_GRAB,
                i::INSP_PLAYER_LEDGE_REACH_Y,
                i::INSP_PLAYER_LEDGE_SPAN,
                i::INSP_PLAYER_LEDGE_OFFSET_Y,
                i::INSP_PLAYER_LEDGE_SPEED,
            ],
            i::INSP_PLAYER_CARD_GLIDE,
        ),
        (
            i::INSP_PLAYER_CARD_GLIDE,
            &[i::INSP_PLAYER_GLIDE_FALL],
            i::INSP_PLAYER_CARD_FORGIVE,
        ),
        (
            i::INSP_PLAYER_CARD_FORGIVE,
            &[
                i::INSP_PLAYER_COYOTE,
                i::INSP_PLAYER_BUFFER,
                i::INSP_PLAYER_CORNER,
                i::INSP_PLAYER_CORNER_SAMPLES,
                i::INSP_PLAYER_CORNER_AHEAD,
                i::INSP_PLAYER_LIFT,
            ],
            i::INSP_PLAYER_CARD_LEG,
        ),
        (
            i::INSP_PLAYER_CARD_LEG,
            &[
                i::INSP_PLAYER_FLOAT,
                i::INSP_PLAYER_CLING,
                i::INSP_PLAYER_STIFFNESS,
                i::INSP_PLAYER_DAMPING,
                i::INSP_PLAYER_FOOT_SAMPLES,
                i::INSP_PLAYER_FOOT_SPREAD,
            ],
            i::INSP_PLAYER_FOLD_ADVANCED,
        ),
        (
            i::INSP_PLAYER_FOLD_ADVANCED,
            &[i::INSP_PLAYER_MAX_FALL, i::INSP_PLAYER_REACT_SUPPORT],
            i::INSP_PLAYER_REMOVE,
        ),
    ];
    d
}

/// ⭐ **A abrir, todo campo que já cabe na banda aparece inteiro** (smoke do dono 10/11: «os últimos campos têm
/// um pequeno delay para aparecer»).
#[test]
fn a_abrir_todo_campo_dentro_da_banda_aparece_inteiro() {
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    let janela = Rect::new(0.0, 0.0, 1600.0, 3200.0);
    let layout = || {
        HeroLayout::for_viewport_bands(
            janela,
            false,
            ChromeBands::DEFAULT,
            CenterSplit::None,
            DockSides::BOTH,
        )
    };
    ph2d_panel_inspector::set_cards_on(true);
    let mut vistos = 0;
    let mut falhas = Vec::new();
    // O More do Walk tem o cartão Jump entre ele e o título seguinte: a régua do título não o mede.
    for (dobra, dentro, seguinte) in dobras_do_player()
        .into_iter()
        .filter(|d| d.0 != ph2d_panel_inspector::ids::INSP_PLAYER_FOLD_WALK_MORE)
    {
        let mut host = crate::host_em_metros();
        host.store_mut()
            .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION, false);
        for id in ph2d_panel_inspector::ids::INSP_PLAYER_FOLDS {
            host.set_collapsed(id, id != dobra);
        }
        let mut state = InspectorState::default();
        set_current_inspector_player(Some(player()));
        let aberto = crate::o_cartao_da_fisica::pinta(&mut host, &mut state);
        let cheio: Vec<(ph2d_a11y::NodeId, Rect)> = dentro
            .iter()
            .filter_map(|&id| crate::o_cartao_da_fisica::achado(&aberto, id).map(|r| (id, r)))
            .collect();
        // Passos de quadro (2 %), como a animação real: o cartão mede-se no quadro anterior.
        for n in 1..=48 {
            let t = n as f32 * 0.02;
            host.set_collapsed(dobra, false);
            host.store_mut().set_section_open_live(dobra, t);
            let rects = host.paint_with_layout::<InspectorPanel>(&mut state, layout(), janela);
            let Some(titulo) = crate::o_cartao_da_fisica::achado(&rects, seguinte) else {
                continue;
            };
            if n % 12 != 0 {
                continue;
            }
            for &(id, r) in &cheio {
                // O vão até ao título seguinte e um passo de quadro da revelação.
                if r.y + r.h > titulo.y - 20.0 {
                    continue;
                }
                vistos += 1;
                let agora = crate::o_cartao_da_fisica::achado(&rects, id);
                if agora.is_none_or(|q| (q.h - r.h).abs() > 0.5) {
                    falhas.push(format!(
                        "{dobra:?} a {t:.2}: o campo {id:?} (fim {:.1}) cabe na banda (título seguinte em {:.1}) e está {agora:?}",
                        r.y + r.h,
                        titulo.y
                    ));
                }
            }
        }
        set_current_inspector_player(None);
    }
    ph2d_panel_inspector::set_cards_on(false);
    assert!(vistos > 0, "a régua não viu campo nenhum");
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// ⭐ **Toda dobra do Platform Player tem dica de hover** — o que mora dentro, em Plain English, e
/// nenhuma repete a de outra. Mutação que deve sangrar: tirar o `set_tooltip` do laço das dobras.
#[test]
fn toda_dobra_do_player_tem_dica() {
    let mut host = host_aberto();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(player()));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_player(None);
    let mut vistas = std::collections::BTreeSet::new();
    for id in ph2d_panel_inspector::ids::INSP_PLAYER_FOLDS {
        let dica = host.store().tooltip_for(id).unwrap_or_default();
        assert!(dica.len() > 12, "a dobra {id:?} sem dica: {dica:?}");
        assert!(vistas.insert(dica.to_owned()), "dica repetida: {dica}");
    }
}

/// ⭐ **`open_all_sections` abre as dobras do Platform Player** — o aberto herdado (ondas 2–3) dizia
/// que não; esta é a prova pela porta do produto: cada row de dentro de cada dobra pinta-se.
#[test]
fn abrir_todas_as_seccoes_abre_as_dobras_do_player() {
    let mut host = crate::host_em_metros();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(player()));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    host.open_all_sections();
    let pintados: Vec<_> = host
        .paint::<InspectorPanel>(&mut state, VIEWPORT)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    set_current_inspector_player(None);
    for (dobra, dentro, _) in dobras_do_player() {
        assert!(!host.store().is_collapsed(dobra), "{dobra:?} ficou fechada");
        for id in dentro {
            assert!(
                pintados.contains(id),
                "{id:?} (dentro de {dobra:?}) não pintou"
            );
        }
    }
}
