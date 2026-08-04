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
use ph2d_editor_core::screens::hero::{InspectorNameInfo, InspectorPlayerInfo, PlayerFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_name, set_current_inspector_player,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_0081;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// Um corpo Dynamic que **já é** um player — o estado em que o artista está
/// quando afina um.
fn player() -> InspectorPlayerInfo {
    InspectorPlayerInfo {
        entity_bits: ENTITY,
        has_player: true,
        float_height: 0.9,
        // ⚠️ Premissa declarada: a altura ESTÁ acima do piso, então o botão de
        // ajuste é oferecido no rótulo curto. Uma fixture que chegasse abaixo do
        // piso deixaria o gate do aviso verde pelo motivo errado.
        min_float_height: 0.58,
        min_float_known: true,
        cling_distance: 0.25,
        spring_strength: 400.0,
        spring_damping: 0.5,
        speed: 6.0,
        acceleration: 60.0,
        air_acceleration: 20.0,
        max_slope_deg: 45.0,
        jump_height: 2.0,
        takeoff_gravity: 1.0,
        takeoff_speed: 0.0,
        peak_gravity: 0.5,
        peak_speed: 1.5,
        fall_gravity: 2.0,
        cut_gravity: 4.0,
    }
}

/// O MESMO corpo, ainda sem o comportamento — a face vazia.
fn empty() -> InspectorPlayerInfo {
    InspectorPlayerInfo {
        has_player: false,
        ..player()
    }
}

fn painted(info: InspectorPlayerInfo) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_player(None);
    rects
}

fn click_real(info: InspectorPlayerInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
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
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_player(Some(info));
    host.set_number_value(id, v);
    let _ = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::ValueChanged(id));
    let out = host.drained_actions();
    set_current_inspector_player(None);
    out
}

#[track_caller]
fn expect(actions: &[EditorAction], edit: PlayerFieldEdit, what: &str) {
    assert_eq!(
        actions,
        [EditorAction::InspectorPlayerEdit {
            entity_bits: ENTITY,
            edit,
        }],
        "{what} não levantou a edição que devia"
    );
}

/// **A FACE VAZIA é um botão, e ele CHEGA.**
///
/// O gate mais importante do arquivo: é este gesto que faz o comportamento
/// existir, e sem ele o componente é inalcançável no produto.
#[test]
fn the_empty_face_offers_the_one_gesture_that_creates_a_player() {
    expect(
        &click_real(empty(), ids::INSP_PLAYER_ADD),
        PlayerFieldEdit::Add,
        "Make Platform Player",
    );
}

/// ⚠️ **A face vazia NÃO oferece os knobs** — nem o Remove, nem o ajuste.
///
/// A metade de AUSÊNCIA: um controle que edita o que ainda não existe é o botão
/// morto que esta linha varre a cada wave.
#[test]
fn the_empty_face_offers_nothing_else() {
    let rects = painted(empty());
    for id in [
        ids::INSP_PLAYER_REMOVE,
        ids::INSP_PLAYER_FIT,
        ids::INSP_PLAYER_FLOAT,
        ids::INSP_PLAYER_SPEED,
        ids::INSP_PLAYER_MAX_SLOPE,
    ] {
        assert!(
            !rects.iter().any(|(n, _)| *n == id),
            "a face vazia pintou {id:?}, que edita um player que nao existe"
        );
    }
}

/// **Todos os oito números levantam a própria edição** — a varredura.
///
/// Uma tabela e não oito testes: uma row nova que esqueça o arm faz esta lista
/// falhar, e uma que esqueça a lista não existe (o pintor itera a mesma).
#[test]
fn every_number_raises_its_own_edit() {
    // ⚠️ A lista TEM de cobrir a tabela inteira, e é isso que a asserção abaixo
    // afirma: um número novo que chegue ao painel e não a esta varredura seria
    // exatamente o arm esquecido que ela existe para pegar.
    assert_eq!(
        ph2d_panel_inspector::PLAYER_ROW_COUNT,
        14,
        "a tabela de rows cresceu; acrescente o numero novo a esta varredura"
    );
    for (id, v, edit) in [
        (
            ids::INSP_PLAYER_FLOAT,
            1.25,
            PlayerFieldEdit::FloatHeight(1.25),
        ),
        (
            ids::INSP_PLAYER_CLING,
            0.4,
            PlayerFieldEdit::ClingDistance(0.4),
        ),
        (
            ids::INSP_PLAYER_STIFFNESS,
            250.0,
            PlayerFieldEdit::SpringStrength(250.0),
        ),
        (
            ids::INSP_PLAYER_DAMPING,
            0.8,
            PlayerFieldEdit::SpringDamping(0.8),
        ),
        (ids::INSP_PLAYER_SPEED, 9.0, PlayerFieldEdit::Speed(9.0)),
        (
            ids::INSP_PLAYER_ACCEL,
            80.0,
            PlayerFieldEdit::Acceleration(80.0),
        ),
        (
            ids::INSP_PLAYER_AIR_ACCEL,
            15.0,
            PlayerFieldEdit::AirAcceleration(15.0),
        ),
        (
            ids::INSP_PLAYER_MAX_SLOPE,
            55.0,
            PlayerFieldEdit::MaxSlopeDeg(55.0),
        ),
        (
            ids::INSP_PLAYER_JUMP_HEIGHT,
            3.5,
            PlayerFieldEdit::JumpHeight(3.5),
        ),
        (
            ids::INSP_PLAYER_TAKEOFF_G,
            1.4,
            PlayerFieldEdit::TakeoffGravity(1.4),
        ),
        (
            ids::INSP_PLAYER_TAKEOFF_SPEED,
            2.5,
            PlayerFieldEdit::TakeoffSpeed(2.5),
        ),
        (
            ids::INSP_PLAYER_PEAK_G,
            0.3,
            PlayerFieldEdit::PeakGravity(0.3),
        ),
        (
            ids::INSP_PLAYER_PEAK_SPEED,
            2.0,
            PlayerFieldEdit::PeakSpeed(2.0),
        ),
        (
            ids::INSP_PLAYER_FALL_G,
            2.5,
            PlayerFieldEdit::FallGravity(2.5),
        ),
        (
            ids::INSP_PLAYER_CUT_G,
            5.0,
            PlayerFieldEdit::CutGravity(5.0),
        ),
    ] {
        expect(&commit(player(), id, v), edit, &format!("{id:?}"));
    }
}

/// Os dois botões do estado cheio chegam ao barramento pelo ponteiro real.
#[test]
fn the_fit_and_remove_buttons_reach_the_bus() {
    expect(
        &click_real(player(), ids::INSP_PLAYER_FIT),
        PlayerFieldEdit::FitFloatHeight,
        "Fit to Collider",
    );
    expect(
        &click_real(player(), ids::INSP_PLAYER_REMOVE),
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
        min_float_known: false,
        ..player()
    };
    let rects = painted(boxed);
    assert!(
        !rects.iter().any(|(n, _)| *n == ids::INSP_PLAYER_FIT),
        "sem mínimo computável não há para onde ajustar"
    );
    // E o resto da seção continua vivo — a ausência é do botão, não da seção.
    assert!(rects.iter().any(|(n, _)| *n == ids::INSP_PLAYER_FLOAT));
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
        max_slope_deg: 61.0,
        jump_height: 3.75,
        takeoff_gravity: 1.6,
        takeoff_speed: 2.25,
        peak_gravity: 0.35,
        peak_speed: 2.75,
        fall_gravity: 2.25,
        cut_gravity: 5.5,
        ..player()
    };
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
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
        ids::INSP_PLAYER_FLOAT,
        ids::INSP_PLAYER_CLING,
        ids::INSP_PLAYER_STIFFNESS,
        ids::INSP_PLAYER_DAMPING,
        ids::INSP_PLAYER_SPEED,
        ids::INSP_PLAYER_ACCEL,
        ids::INSP_PLAYER_AIR_ACCEL,
        ids::INSP_PLAYER_MAX_SLOPE,
        ids::INSP_PLAYER_JUMP_HEIGHT,
        ids::INSP_PLAYER_TAKEOFF_G,
        ids::INSP_PLAYER_TAKEOFF_SPEED,
        ids::INSP_PLAYER_PEAK_G,
        ids::INSP_PLAYER_PEAK_SPEED,
        ids::INSP_PLAYER_FALL_G,
        ids::INSP_PLAYER_CUT_G,
    ]
    .iter()
    .map(|&id| host.store().number_value(id).unwrap_or(f64::NAN))
    .collect();
    set_current_inspector_player(None);
    set_current_inspector_name(None);
    let want = [
        1.11, 0.33, 321.0, 0.77, 7.5, 44.0, 12.0, 61.0, 3.75, 1.6, 2.25, 0.35, 2.75, 2.25, 5.5,
    ];
    for (g, w) in got.iter().zip(want) {
        assert!(
            (g - w).abs() < 1.0e-4,
            "a row mostra {g} onde o documento diz {w} — ela e' WRITE-ONLY: {got:?}"
        );
    }
}
