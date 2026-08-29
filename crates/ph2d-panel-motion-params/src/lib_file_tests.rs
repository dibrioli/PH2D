//! **A row de FICHEIRO** — as quatro condições de UI da casa para ela.
//!
//! O pedido do Enio (2026-08-28): *"sim. vamos criar os botões"* — sobre o `ParamWidget::File`,
//! que o plano [93](../../../docs/Motion%20Nodes/93_plano_lsystem_datasource_celanim.md) §4
//! nomeou como a peça que **cura dois nós de uma vez**: o `audio.bands`, que até aqui pedia o
//! caminho DIGITADO à mão, e o `data.table` que ainda vem.
//!
//! As quatro perguntas: o botão EXISTE · é PINTADO e REGISTADO · o clique CHEGA ao barramento
//! (com `click_at`, nunca um `WidgetEvent` fabricado — o sintético salta exactamente o que pode
//! estar partido) · e a SEQUÊNCIA leva a algum lado (essa metade mora na shell,
//! `motion_bridge_params_file_tests`).

use super::*;
use crate::snapshot::param_file_browse_id;

fn viewport() -> ph2d_editor_core::zones::Rect {
    ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1920.0, 1080.0)
}

fn snapshot_with_file(value: &str, missing: bool) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 11,
        title: "Audio Bands".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: Default::default(),
        rows: vec![ParamRow::File(FileRow {
            name: "file",
            label: "Audio File".into(),
            value: value.to_string(),
            missing,
        })],
    }
}

/// **O botão é pintado E o campo continua lá** — as duas metades, porque cada uma sozinha
/// descreve um painel partido.
///
/// Só o botão: o artista não consegue colar um caminho nem corrigir um que mudou de sítio.
/// Só o campo: é o painel de HOJE, que é o defeito que esta wave veio curar.
#[test]
fn a_file_row_paints_both_the_browse_button_and_the_editable_path() {
    set_current_params(Some(snapshot_with_file("/tmp/song.wav", false)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let painted = host.paint::<MotionParamsPanel>(&mut state, viewport());
    let has = |id| painted.iter().any(|(w, _)| *w == id);
    assert!(
        has(param_file_browse_id(0)),
        "o botao Browse tem de existir"
    );
    assert!(
        has(crate::snapshot::param_text_id(0)),
        "o campo de caminho tem de continuar editavel"
    );
    set_current_params(None);
}

/// **E o botão não se sobrepõe ao campo.**
///
/// Duas superfícies interactivas no mesmo rectângulo é o defeito que só um smoke apanha: o
/// dedo chega a uma delas e a outra fica morta, sem nada vermelho em lado nenhum. A
/// disposição é `[rótulo … Browse]` numa linha e o campo na de baixo, então os dois rects não
/// se tocam **em nenhum dos dois eixos ao mesmo tempo**.
#[test]
fn the_browse_button_and_the_path_field_do_not_overlap() {
    set_current_params(Some(snapshot_with_file("/tmp/song.wav", false)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let painted = host.paint::<MotionParamsPanel>(&mut state, viewport());
    let get = |id| {
        painted
            .iter()
            .find(|(w, _)| *w == id)
            .map(|(_, r)| *r)
            .expect("pintado")
    };
    let b = get(param_file_browse_id(0));
    let f = get(crate::snapshot::param_text_id(0));
    let overlaps_x = b.x < f.x + f.w && f.x < b.x + b.w;
    let overlaps_y = b.y < f.y + f.h && f.y < b.y + b.h;
    assert!(
        !(overlaps_x && overlaps_y),
        "o botao {b:?} e o campo {f:?} partilham pixels — um deles fica morto sob o dedo"
    );
    set_current_params(None);
}

/// **Um clique REAL no botão pede o diálogo à shell.**
///
/// ⚠️ O painel **não abre** o diálogo, e o intent é a prova: ele leva o par `(nó, param)` e
/// nada mais. Um `rfd::FileDialog` congela o loop, e só a shell tem a porta que declara o
/// congelamento — abrir um daqui é o defeito que a `modal.rs` existe para não ter.
#[test]
fn a_real_click_on_browse_asks_the_shell_for_a_dialog() {
    let _ = drain_param_intents();
    set_current_params(Some(snapshot_with_file("", false)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let painted = host.paint::<MotionParamsPanel>(&mut state, viewport());
    let rect = painted
        .iter()
        .find(|(w, _)| *w == param_file_browse_id(0))
        .map(|(_, r)| *r)
        .expect("o botao foi pintado");
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        !events.is_empty(),
        "o ponteiro nao alcancou o botao — ele esta morto sob o mouse"
    );
    for ev in events {
        host.apply_panel_event::<MotionParamsPanel>(&mut state, ev);
    }
    assert_eq!(
        drain_param_intents(),
        vec![MotionParamIntent::PickFile {
            node: 11,
            param: "file",
        }],
        "o clique tem de PEDIR o dialogo, com o par (no, param) e mais nada"
    );
    set_current_params(None);
}

/// **O caminho escrito à mão continua a chegar ao documento.**
///
/// O campo reutiliza o `param_text_id` do slot, partilhado com a `Text` row, o *Custom…* da
/// `Channels` e o campo da `Source`. O `on_text_commit` é o único sítio que decide de quem é o
/// buffer — sem a arma da `File` lá, o caminho digitado seria engolido em silêncio.
#[test]
fn a_typed_path_still_reaches_the_document() {
    let _ = drain_param_intents();
    set_current_params(Some(snapshot_with_file("", false)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let _ = host.paint::<MotionParamsPanel>(&mut state, viewport());
    let id = crate::snapshot::param_text_id(0);
    host.set_text(id, "/home/you/other.flac");
    host.apply_panel_event::<MotionParamsPanel>(&mut state, WidgetEvent::Submit(id));
    assert_eq!(
        drain_param_intents(),
        vec![MotionParamIntent::SetTextParam {
            node: 11,
            param: "file",
            value: "/home/you/other.flac".into(),
        }],
    );
    set_current_params(None);
}

/// **A marca de ausência custa uma LINHA, e só quando o ficheiro falta.**
///
/// ⚠️ Ela é texto pintado, não um widget — não se clica nela —, então o que se mede é o que
/// ela EMPURRA: a row seguinte. Sem o par (ausente ⇒ empurra · presente ⇒ não empurra) uma
/// marca desenhada sempre, ou nunca, passaria os dois — é a forma de afirmação que mutação
/// nenhuma mata.
#[test]
fn the_missing_mark_costs_a_line_and_only_when_it_is_missing() {
    let mut tops = Vec::new();
    for missing in [false, true] {
        let mut snap = snapshot_with_file("/tmp/gone.wav", missing);
        snap.rows.push(ParamRow::Toggle(ToggleRow {
            name: "after",
            label: "After".into(),
            on: false,
        }));
        set_current_params(Some(snap));
        let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
        let mut state = MotionParamsPanelState;
        let painted = host.paint::<MotionParamsPanel>(&mut state, viewport());
        tops.push(
            painted
                .iter()
                .find(|(w, _)| *w == crate::snapshot::param_checkbox_id(1))
                .map(|(_, r)| r.y)
                .expect("a row seguinte foi pintada"),
        );
    }
    assert!(
        tops[1] > tops[0],
        "com o ficheiro em falta a row seguinte tem de descer (a marca ocupa uma linha): \
         {:?} contra {:?}",
        tops[1],
        tops[0]
    );
    set_current_params(None);
}
