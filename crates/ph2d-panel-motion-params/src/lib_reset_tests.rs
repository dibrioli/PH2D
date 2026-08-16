//! **A seta que devolve um param ao default** — as quatro condições de UI para ela.
//!
//! Irmão de `lib_tests`, cortado por assunto: aqui só a afordância de reverter. As quatro
//! perguntas que este arquivo responde, e que uma feature de UI deste repo só fecha com as
//! quatro: a seta EXISTE · ela é PINTADA e REGISTRADA · o clique CHEGA ao barramento · e a
//! sequência LEVA a algum lugar (a última mora na shell, no `motion_bridge_reset_tests`).

use super::*;
use crate::snapshot::param_reset_id;

fn viewport() -> ph2d_editor_core::zones::Rect {
    ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1920.0, 1080.0)
}

/// Duas rows escalares — só a primeira com override.
fn snapshot_with_one_modified() -> ParamsSnapshot {
    let row = |name: &'static str| {
        ParamRow::Scalar(ScalarRow {
            name,
            label: name.to_string(),
            value: 3.0,
            min: 0.0,
            max: 10.0,
            hard_min: 0.0,
            hard_max: 10.0,
            step: 0.1,
            integer: false,
            driven_by: None,
            display: RowDisplay::default(),
        })
    };
    ParamsSnapshot {
        node: 7,
        title: "Grid".into(),
        modified: ["rows".to_string()].into_iter().collect(),
        sections: Vec::new(),
        rows: vec![row("rows"), row("cols")],
    }
}

/// **A seta existe só onde há o que reverter — e as duas metades importam.**
///
/// Só a presença passaria com uma seta desenhada em toda row (o botão-morto: clicá-lo num
/// param intocado não faria nada, e a tela deixaria de dizer *o que eu mexi neste nó*). Só a
/// ausência passaria com nenhuma seta em lugar nenhum.
#[test]
fn the_revert_arrow_exists_only_where_there_is_something_to_revert() {
    set_current_params(Some(snapshot_with_one_modified()));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let painted = host.paint::<MotionParamsPanel>(&mut state, viewport());
    let has = |id| painted.iter().any(|(w, _)| *w == id);
    assert!(
        has(param_reset_id(0)),
        "a row com override tem de oferecer a seta"
    );
    assert!(
        !has(param_reset_id(1)),
        "a row intocada NÃO pode oferecer uma seta que não reverteria nada"
    );
    set_current_params(None);
}

/// **E um clique REAL nela chega ao barramento.**
///
/// `click_at` — não um `WidgetEvent` sintético — porque o sintético pula exatamente o que
/// pode estar quebrado: um botão pintado e hit-indexado que o `populate` esqueceu de
/// registrar fica MORTO sob o mouse, e um evento fabricado nunca descobre isso. É a falha
/// que este codebase já shipou mais de uma vez.
#[test]
fn a_real_click_on_the_revert_arrow_reaches_the_bus() {
    let _ = drain_param_intents();
    set_current_params(Some(snapshot_with_one_modified()));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let painted = host.paint::<MotionParamsPanel>(&mut state, viewport());
    let rect = painted
        .iter()
        .find(|(w, _)| *w == param_reset_id(0))
        .map(|(_, r)| *r)
        .expect("a seta foi pintada");
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        !events.is_empty(),
        "o ponteiro não alcançou a seta — ela está morta sob o mouse"
    );
    for ev in events {
        host.apply_panel_event::<MotionParamsPanel>(&mut state, ev);
    }
    assert_eq!(
        drain_param_intents(),
        vec![MotionParamIntent::ResetParam {
            node: 7,
            param: "rows".into(),
        }],
        "o clique tem de pedir a REMOÇÃO do override daquele param, e só dele"
    );
    set_current_params(None);
}

/// **Reverter uma COR desfaz os quatro canais.**
///
/// Um swatch dobra RGBA em quatro params, então uma seta que emitisse um só deixaria a cor
/// meio-revertida — um verde que vira um verde diferente, que é pior que não reverter. Quem
/// sabe quantos params uma row edita é `ParamRow::params`, uma vez.
#[test]
fn reverting_a_colour_undoes_all_four_channels() {
    let _ = drain_param_intents();
    set_current_params(Some(ParamsSnapshot {
        node: 3,
        title: "Tint".into(),
        modified: ["c_g".to_string()].into_iter().collect(),
        sections: Vec::new(),
        rows: vec![ParamRow::Color(ColorRow {
            label: "Colour".into(),
            channels: ["c_r", "c_g", "c_b", "c_a"],
            srgb: [255, 0, 0, 255],
        })],
    }));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let painted = host.paint::<MotionParamsPanel>(&mut state, viewport());
    assert!(
        painted.iter().any(|(w, _)| *w == param_reset_id(0)),
        "UM canal com override já torna a cor modificada"
    );
    host.apply_panel_event::<MotionParamsPanel>(
        &mut state,
        ph2d_editor_core::interaction::WidgetEvent::Click(param_reset_id(0)),
    );
    let got: Vec<String> = drain_param_intents()
        .into_iter()
        .filter_map(|i| match i {
            MotionParamIntent::ResetParam { param, .. } => Some(param),
            _ => None,
        })
        .collect();
    assert_eq!(got, vec!["c_r", "c_g", "c_b", "c_a"], "os QUATRO canais");
    set_current_params(None);
}

/// Três rows, as duas últimas numa seção "Range".
fn snapshot_with_a_section() -> ParamsSnapshot {
    let row = |name: &'static str| {
        ParamRow::Scalar(ScalarRow {
            name,
            label: name.to_string(),
            value: 3.0,
            min: 0.0,
            max: 10.0,
            hard_min: 0.0,
            hard_max: 10.0,
            step: 0.1,
            integer: false,
            driven_by: None,
            display: RowDisplay::default(),
        })
    };
    ParamsSnapshot {
        node: 7,
        title: "Remap".into(),
        modified: Default::default(),
        sections: vec![("Range".to_string(), 1)],
        rows: vec![row("contour"), row("min"), row("max")],
    }
}

/// **A seção pinta um cabeçalho, e ele DOBRA.**
///
/// O collapse genérico exige DOIS sítios (a marca no store e o hit-rect no paint), e a
/// falha de esquecer um é um cabeçalho que desenha um chevron e não responde — o título
/// morto que o painel do Vector já pagou. As duas metades num gate só: a de cima prova que
/// o cabeçalho é alcançável, a de baixo que dobrar ESCONDE as rows dele.
#[test]
fn a_section_header_is_reachable_and_folding_it_hides_its_rows() {
    use crate::rows_paint::sections::section_id;
    set_current_params(Some(snapshot_with_a_section()));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;

    let painted = host.paint::<MotionParamsPanel>(&mut state, viewport());
    let id = section_id("Range");
    assert!(
        painted.iter().any(|(w, _)| *w == id),
        "o cabeçalho da seção tem de entrar no hit index"
    );
    // Aberta: as rows da seção têm widgets.
    assert!(
        painted.iter().any(|(w, _)| *w == param_slider_id(1)),
        "com a seção aberta, a row dentro dela é desenhada"
    );

    // O clique no cabeçalho dobra (dispatch GENÉRICO — se a marca faltar, nada acontece).
    let rect = painted
        .iter()
        .find(|(w, _)| *w == id)
        .map(|(_, r)| *r)
        .expect("pintado");
    let _ = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    // ⚠️ **A dobra é ANIMADA (F4b)**: o flag semântico vira no quadro do clique, mas o `t` ainda
    // desce, e um harness de painel não tem o tique do `HeroScreen`. Sem isto o gate afirmaria
    // *"a row sumiu"* sobre um produto que a está a esconder **gradualmente** — reprovaria a
    // animação em vez de a medir.
    host.settle_section_folds();
    let painted = host.paint::<MotionParamsPanel>(&mut state, viewport());
    assert!(
        !painted.iter().any(|(w, _)| *w == param_slider_id(1)),
        "dobrada, a row de dentro não pode continuar desenhada"
    );
    assert!(
        painted.iter().any(|(w, _)| *w == param_slider_id(0)),
        "e a row SOLTA, fora da seção, continua lá"
    );
    set_current_params(None);
}
