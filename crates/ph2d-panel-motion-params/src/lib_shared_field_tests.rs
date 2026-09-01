//! **O CAMPO DE TEXTO PARTILHADO do slot** — a metade de LEITURA da pergunta cuja metade de
//! ESCRITA o `on_text_commit` já respondia.
//!
//! Report do Enio (2026-08-30), sobre a cena `=109`: *"O painel não imprime o caminho do CSV
//! em lugar nenhum"*.
//!
//! ⚠️ **Quatro rows partilham UM campo por slot** (`param_text_id`) — `Text`, `Channels`
//! (*Custom…*), `Source` e `File` —, e o `snapshot_ids` declara essa partilha como desenho.
//! O `on_text_commit` decide de QUEM é o buffer quando ele volta; ninguém decidia **o que o
//! buffer mostra**, e o `seed_rows` semeava só a `Text`. As outras três abriam SEMPRE em
//! branco, com o placeholder no lugar do valor — e os doc-comments das três dizem, palavra por
//! palavra, o contrário (*"shown in the Custom field"*, *"fills the field"*, *"the current
//! path"*). *Três descrições e zero implementações.*
//!
//! ⚠️⚠️ **E a metade cara é a segunda**: `Blur` chama o `on_text_commit`. Um campo que abre
//! vazio e é tocado por engano **grava o vazio** — clicar no caminho e clicar noutro sítio
//! APAGAVA o ficheiro do nó, sem nada vermelho em lado nenhum.
//!
//! ⛔ As quatro condições de UI da casa (existe · pintado e registado · o clique chega ao
//! barramento · a sequência leva a algum lado) estavam **todas verdes** — elas perguntam pelo
//! WIDGET, e esta pergunta é pelo VALOR. É a quinta condição, e é ela que este ficheiro fixa.

use super::*;
use crate::shared_field::{PARAM_ROW_KINDS, param_row_kind, shared_text_param, shared_text_value};
use crate::snapshot::param_text_id;

fn viewport() -> ph2d_editor_core::zones::Rect {
    ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1920.0, 1080.0)
}

fn snapshot_of(row: ParamRow) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 11,
        title: "T".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: Default::default(),
        rows: vec![row],
    }
}

/// As quatro rows que partilham o campo, cada uma com um valor DIFERENTE — um valor comum
/// deixaria passar um seed que semeasse sempre o da primeira.
fn the_four_sharers() -> Vec<(&'static str, ParamRow, &'static str)> {
    vec![
        (
            "Text",
            ParamRow::Text(TextRow {
                name: "expr",
                label: "Expression".into(),
                value: "sin(t)".into(),
                problem: None,
                help: None,
            }),
            "sin(t)",
        ),
        (
            "Channels",
            ParamRow::Channels(ChannelsRow {
                label: "Column".into(),
                text_param: "column",
                mode_param: "mode",
                channels: vec![("Mass", "mass", 0)],
                selected: 1,
                custom: "inv_mass".into(),
                extra: Vec::new(),
            }),
            "inv_mass",
        ),
        (
            "Source",
            ParamRow::Source(SourceRow {
                label: "Shape".into(),
                param: "shape",
                options: vec!["star".into()],
                current: "a shape nobody drew".into(),
            }),
            "a shape nobody drew",
        ),
        (
            "File",
            ParamRow::File(FileRow {
                name: "file",
                label: "Table File".into(),
                value: "/tmp/ph2d_table_demo.csv".into(),
                missing: false,
            }),
            "/tmp/ph2d_table_demo.csv",
        ),
    ]
}

/// ⭐⭐⭐ **O campo MOSTRA o que o documento tem** — o report do Enio, nas quatro rows.
///
/// ⚠️ A afirmação é sobre o BUFFER, não sobre pixels: é ele que o `paint_text_input_with_buffer`
/// desenha e que o `on_text_commit` lê de volta. Medir o glifo pedia um oráculo de texto; medir
/// o buffer mata a mutação (`seed_rows` sem a arma ⇒ `""`).
#[test]
fn the_shared_text_field_shows_what_the_document_holds() {
    for (kind, row, expected) in the_four_sharers() {
        set_current_params(Some(snapshot_of(row)));
        let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
        let mut state = MotionParamsPanelState;
        let _ = host.paint::<MotionParamsPanel>(&mut state, viewport());
        assert_eq!(
            host.store().text(param_text_id(0)).unwrap_or_default(),
            expected,
            "a row {kind} abriu com o campo VAZIO — o artista nao ve o valor que o no' tem"
        );
        set_current_params(None);
    }
}

/// ⭐⭐⭐ **Tocar no campo e sair NÃO apaga o valor.**
///
/// O `Blur` comita. Com o campo semeado, o que volta é o MESMO valor (uma edição nula); sem o
/// seed voltava `""`, e a única forma de o artista descobrir era o nó ficar mudo.
#[test]
fn blurring_the_field_without_typing_never_erases_the_value() {
    for (kind, row, expected) in the_four_sharers() {
        let _ = drain_param_intents();
        set_current_params(Some(snapshot_of(row)));
        let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
        let mut state = MotionParamsPanelState;
        let _ = host.paint::<MotionParamsPanel>(&mut state, viewport());
        host.apply_panel_event::<MotionParamsPanel>(
            &mut state,
            WidgetEvent::Blur(param_text_id(0)),
        );
        let got = drain_param_intents();
        for intent in &got {
            if let MotionParamIntent::SetTextParam { value, .. } = intent {
                assert_eq!(
                    value, expected,
                    "a row {kind} gravou «{value}» ao perder o foco — tocar no campo APAGOU o valor"
                );
            }
        }
        set_current_params(None);
    }
}

/// ⭐⭐ **As duas metades da mesma pergunta concordam, variante a variante.**
///
/// Quem consegue COMITAR o campo partilhado tem de conseguir PREENCHÊ-LO, e ao contrário. Foi
/// a divergência entre as duas listas que produziu o defeito: o `on_text_commit` tinha quatro
/// armas e o `seed_rows` tinha uma.
///
/// ⚠️ **O censo é de DUAS metades**, porque um `match` exaustivo não guarda a lista que um laço
/// percorre: as funções são exaustivas **sem braço curinga** (uma variante nova é erro de
/// compilação nas duas), e o [`PARAM_ROW_KINDS`] obriga a amostra a cobri-las todas.
#[test]
fn every_row_that_can_commit_the_shared_field_also_fills_it() {
    let all: Vec<ParamRow> = vec![
        ParamRow::Scalar(ScalarRow {
            name: "s",
            label: "S".into(),
            value: 0.0,
            min: 0.0,
            hard_min: 0.0,
            max: 1.0,
            hard_max: 1.0,
            step: 0.1,
            integer: false,
            driven_by: None,
            display: RowDisplay::new(1.0, ""),
        }),
        ParamRow::Color(ColorRow {
            label: "C".into(),
            channels: ["r", "g", "b", "a"],
            srgb: [0, 0, 0, 255],
        }),
        ParamRow::Toggle(ToggleRow {
            name: "t",
            label: "T".into(),
            on: false,
        }),
        ParamRow::Enum(EnumRow {
            name: "e",
            label: "E".into(),
            selected: 0,
            labels: &["A", "B"],
        }),
        ParamRow::Angle(AngleRow {
            name: "a",
            label: "A".into(),
            deg: 0.0,
            min_deg: 0.0,
            max_deg: 360.0,
            step_deg: 1.0,
        }),
        ParamRow::Seed(SeedRow {
            name: "sd",
            label: "Sd".into(),
            value: 0.0,
            min: 0.0,
            max: 99.0,
        }),
        ParamRow::Curve(CurveRow {
            name: "c",
            label: "C".into(),
            value: String::new(),
        }),
        ParamRow::Gradient(GradientRow {
            name: "g",
            label: "G".into(),
            value: String::new(),
        }),
        ParamRow::Palette(PaletteRow {
            name: "p",
            label: "P".into(),
            value: String::new(),
        }),
    ]
    .into_iter()
    .chain(the_four_sharers().into_iter().map(|(_, row, _)| row))
    .collect();

    let mut seen = std::collections::BTreeSet::new();
    for row in &all {
        seen.insert(param_row_kind(row));
        assert_eq!(
            shared_text_param(row).is_some(),
            shared_text_value(row).is_some(),
            "a variante {row:?} responde a uma metade e nao a' outra — \
             ou ela comita sem mostrar, ou mostra sem comitar"
        );
    }
    assert_eq!(
        seen.len(),
        PARAM_ROW_KINDS,
        "a amostra cobre {} das {PARAM_ROW_KINDS} variantes — as que faltam nao foram medidas",
        seen.len()
    );
}
