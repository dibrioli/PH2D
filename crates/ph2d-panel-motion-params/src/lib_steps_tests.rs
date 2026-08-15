//! Gates de COSTURA da row de PASSOS — as quatro condições que uma feature de painel tem
//! de satisfazer, e elas são independentes: o widget EXISTE · é pintado e REGISTRADO · o
//! clique CHEGA ao barramento · e a sequência LEVA a algum lugar.
//!
//! ⚠️ Aqui o painel é dirigido pelo `MockPanelHost` REAL — `paint` de verdade, retângulos
//! de hit de verdade, `click_at` de verdade. Um `WidgetEvent` sintético pula a checagem de
//! FOCABILIDADE do store, e é assim que uma faixa nasce *pintada, hit-registrada e morta
//! sob o mouse* (a cicatriz das 36 células da matriz de colisão da física).

use crate::snapshot::{
    param_checkbox_id, param_steps_add_id, param_steps_bar_id, param_steps_remove_id, param_text_id,
};
use crate::{
    MotionParamIntent, MotionParamsPanel, MotionParamsPanelState, ParamRow, ParamsSnapshot,
    StepsRow, drain_param_intents, set_current_params,
};
use ph2d_editor_core::interaction::InteractiveState;

fn steps_snapshot(value: &str) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Pattern".into(),
        modified: Default::default(),
        sections: Vec::new(),
        rows: vec![ParamRow::Steps(StepsRow {
            name: "table",
            label: "Table".into(),
            value: value.into(),
            min: 0.0,
            max: 1.0,
        })],
    }
}

fn viewport() -> ph2d_editor_core::zones::Rect {
    ph2d_editor_core::zones::Rect {
        x: 0.0,
        y: 0.0,
        w: 1200.0,
        h: 800.0,
    }
}

/// **A faixa é alcançável e viva:** cada barra é um `CurvePoint` REGISTRADO (a dispatch só
/// ativa um id focável — uma barra não registrada é pedra), e o `+` acrescenta um passo
/// através do clique de verdade.
#[test]
fn the_step_strip_is_reachable_and_wired() {
    let _ = drain_param_intents();
    set_current_params(Some(steps_snapshot("0.1 0.5 0.9")));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let rects = host.paint::<MotionParamsPanel>(&mut state, viewport());

    for i in 0..3 {
        let id = param_steps_bar_id("table", i);
        assert!(
            matches!(
                host.store().get(id),
                Some(InteractiveState::CurvePoint { .. })
            ),
            "a barra {i} tem de ser um CurvePoint vivo"
        );
        assert!(
            rects
                .iter()
                .any(|(rid, r)| *rid == id && r.w > 0.0 && r.h > 0.0),
            "a barra {i} tem de pintar um retângulo agarrável"
        );
    }

    let add = rects
        .iter()
        .find(|(id, r)| *id == param_steps_add_id(0) && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
        .expect("o botão + pinta um retângulo agarrável");
    for ev in host.click_at(add.x + add.w * 0.5, add.y + add.h * 0.5) {
        host.apply_panel_event::<MotionParamsPanel>(&mut state, ev);
    }
    let value = drain_param_intents()
        .into_iter()
        .find_map(|it| match it {
            MotionParamIntent::SetTextParam {
                param: "table",
                value,
                ..
            } => Some(value),
            _ => None,
        })
        .expect("clicar + emite um SetTextParam na tabela");
    assert_eq!(
        ph2d_steps::parse(&value),
        vec![0.1, 0.5, 0.9, 0.9],
        "o + repetiu o último passo (3 -> 4)"
    );
}

/// **`−` no último passo devolve o nó ao caminho LEGADO** — a string vazia é o sinal de
/// *nada autorado*, e é ela que faz os controles legados voltarem a ser pintados.
///
/// ⚠️ Sem esta metade o `−` deixaria a row numa lista de ZERO elementos: um estado que
/// desenha nada, não é o legado, e do qual só o campo de texto tiraria o artista.
#[test]
fn removing_the_last_step_returns_to_the_legacy_path() {
    let _ = drain_param_intents();
    set_current_params(Some(steps_snapshot("0.4")));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let rects = host.paint::<MotionParamsPanel>(&mut state, viewport());
    let rem = rects
        .iter()
        .find(|(id, r)| *id == param_steps_remove_id(0) && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
        .expect("o botão − pinta um retângulo agarrável");
    for ev in host.click_at(rem.x + rem.w * 0.5, rem.y + rem.h * 0.5) {
        host.apply_panel_event::<MotionParamsPanel>(&mut state, ev);
    }
    let value = drain_param_intents()
        .into_iter()
        .find_map(|it| match it {
            MotionParamIntent::SetTextParam {
                param: "table",
                value,
                ..
            } => Some(value),
            _ => None,
        })
        .expect("clicar − emite um SetTextParam");
    assert_eq!(
        value, "",
        "tirar o último passo escreve o sinal de nada autorado"
    );
}

/// **O `Type` troca a FACE, e o campo cru escreve no MESMO param.**
///
/// ⚠️ As duas metades importam: um checkbox que troca a face e um campo que não commita
/// seriam um editor que o artista abre e do qual não consegue sair com o valor.
#[test]
fn the_type_checkbox_swaps_the_face_and_the_raw_field_commits() {
    let _ = drain_param_intents();
    set_current_params(Some(steps_snapshot("0.1 0.5 0.9")));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let rects = host.paint::<MotionParamsPanel>(&mut state, viewport());

    // O checkbox é pintado e agarrável ao lado do `+`.
    let cb = rects
        .iter()
        .find(|(id, r)| *id == param_checkbox_id(0) && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
        .expect("o checkbox `Type` pinta um retângulo agarrável");
    for ev in host.click_at(cb.x + cb.w * 0.5, cb.y + cb.h * 0.5) {
        host.apply_panel_event::<MotionParamsPanel>(&mut state, ev);
    }
    // Nenhum `SetParam` sai daqui: numa row de passos este checkbox é a troca de FACE, não
    // um param booleano (um `SetParam` iria para um param que não existe).
    assert!(
        drain_param_intents().is_empty(),
        "trocar de face não é uma edição do documento"
    );

    // Repintado, a face crua está no ar e as barras saíram.
    let rects = host.paint::<MotionParamsPanel>(&mut state, viewport());
    assert!(
        rects
            .iter()
            .any(|(id, r)| *id == param_text_id(0) && r.w > 0.0 && r.h > 0.0),
        "com o `Type` marcado o campo de texto é oferecido"
    );
    assert!(
        !rects
            .iter()
            .any(|(id, _)| *id == param_steps_bar_id("table", 0)),
        "e as barras saíram — UMA face por vez"
    );

    // O campo cru commita no MESMO param da faixa.
    host.set_text(param_text_id(0), "0.2 0.8");
    host.apply_panel_event::<MotionParamsPanel>(
        &mut state,
        ph2d_editor_core::interaction::WidgetEvent::Submit(param_text_id(0)),
    );
    let value = drain_param_intents()
        .into_iter()
        .find_map(|it| match it {
            MotionParamIntent::SetTextParam {
                param: "table",
                value,
                ..
            } => Some(value),
            _ => None,
        })
        .expect("o campo cru commita na tabela");
    assert_eq!(ph2d_steps::parse(&value), vec![0.2, 0.8]);
}
