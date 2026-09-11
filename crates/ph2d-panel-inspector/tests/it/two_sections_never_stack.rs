//! ⭐⭐⭐ **DUAS SECÇÕES NUNCA SE PINTAM UMA POR CIMA DA OUTRA.**
//!
//! # O defeito que este gate existe para apagar, e porque nenhum outro o via
//!
//! Report do dono, 2026-09-09, com foto: *«secção Signal toda embolada e mal formatada»*. As
//! secções **TIMERS** e **SIGNAL ACTIONS** desenhavam-se exactamente no mesmo `y` — cabeçalho sobre
//! cabeçalho, o campo de um por cima do botão do outro.
//!
//! A causa: no orquestrador das secções com estado de painel, o `y` devolvido por
//! `paint_anchor_section` e por `paint_timer_section` era **descartado** (`f(...)` em vez de
//! `y = f(...)`), e as seguintes recebiam o `y` de antes delas. ⚠️ **Foi introduzido na wave do
//! `Timer` e repetido na do `SignalActions`** — e ficou invisível porque só se vê quando **duas**
//! daquelas secções estão presentes no MESMO objecto, o que exige um objecto com dois desses
//! componentes.
//!
//! ⛔⛔ **E nenhum portão desta casa o via, por construção:**
//!
//! | portão | o que ele pergunta | porque é cego a isto |
//! |---|---|---|
//! | `architecture_panel_wiring_parity` | *este id chega ao `hit_index`?* | chega — **os dois** chegam, sobrepostos |
//! | `hr12_widgets_a11y` | *o ficheiro delega a11y?* | delega |
//! | `architecture_panel_loc_cap` | *quantas linhas tem?* | outra grandeza |
//! | `seam_*` | *o clique chega à ferramenta?* | chega — ao que ficar por cima |
//!
//! *Nenhum instrumento do repo perguntava ONDE uma coisa é desenhada.* O `player_card_spans` é o
//! único vizinho, e mede a §14 sozinha.
//!
//! ⇒ a régua é a **GEOMETRIA das bandas de cabeçalho**, lida do índice de acerto que o painel de
//! facto publica — nunca de uma soma de alturas escrita ao lado do pintor, que seria a segunda
//! resposta à mesma pergunta.

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{
    InspectorActionInfo, InspectorActionRow, InspectorTimerInfo, InspectorTimerRow,
    InspectorTransformInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_action, set_current_inspector_timer,
    set_current_inspector_transform,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn transform() -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: 0xABCD_1234,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }
}

fn timers() -> InspectorTimerInfo {
    InspectorTimerInfo {
        entity_bits: 0xABCD_1234,
        rows: vec![InspectorTimerRow {
            name: "gatilho".into(),
            duration_s: 3.0,
            repeat: false,
            autostart: true,
            signal: "abre".into(),
        }],
        selected_count: 1,
    }
}

fn actions() -> InspectorActionInfo {
    InspectorActionInfo {
        entity_bits: 0xABCD_1234,
        rows: vec![
            InspectorActionRow {
                on: "abre".into(),
                target: "Porta".into(),
                verb_tag: 3,
                arg: String::new(),
                uses_arg: false,
            },
            InspectorActionRow {
                on: "abre".into(),
                target: "Motor".into(),
                verb_tag: 0,
                arg: String::new(),
                uses_arg: true,
            },
        ],
        verb_labels: vec![
            "Start Timer".into(),
            "Stop Timer".into(),
            "Show".into(),
            "Hide".into(),
            "Toggle Visibility".into(),
        ],
        selected_count: 1,
    }
}

/// Pinta o painel **pela porta do produto** e devolve as bandas de cabeçalho das secções vivas,
/// na ordem em que foram registadas.
///
/// ⚠️ **O oráculo é o índice de acerto**, e não uma conta paralela: é ele que o despacho consulta,
/// e é ali que uma sobreposição de facto acontece.
fn section_bands() -> Vec<(&'static str, Rect)> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_transform(Some(transform()));
    set_current_inspector_timer(Some(timers()));
    set_current_inspector_action(Some(actions()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_transform(None);
    set_current_inspector_timer(None);
    set_current_inspector_action(None);

    let nomes: [(&str, ph2d_a11y::NodeId); 3] = [
        ("Transform", ids::INSP_LIVE_TRANSFORM_SECTION),
        ("Timers", ids::INSP_LIVE_TIMER_SECTION),
        ("Signal Actions", ids::INSP_LIVE_ACTION_SECTION),
    ];
    let mut out = Vec::new();
    for (nome, id) in nomes {
        if let Some((_, r)) = rects.iter().find(|(n, _)| *n == id).copied() {
            out.push((nome, r));
        }
    }
    out
}

/// ⭐⭐⭐ **As bandas de cabeçalho de duas secções vivas NÃO se cruzam.**
///
/// **Mutação que deve sangrar:** trocar um `y = paint_<x>_section(…)` por `paint_<x>_section(…)`
/// no `paint_stateful`.
#[test]
fn two_live_sections_never_share_a_band() {
    let bandas = section_bands();
    assert!(
        bandas.len() >= 3,
        "a fixtura nao produziu as tres seccoes: {:?}",
        bandas.iter().map(|(n, _)| *n).collect::<Vec<_>>()
    );
    for i in 0..bandas.len() {
        for j in (i + 1)..bandas.len() {
            let (na, a) = bandas[i];
            let (nb, b) = bandas[j];
            let cruza = a.y < b.y + b.h && b.y < a.y + a.h;
            assert!(
                !cruza,
                "os cabecalhos de '{na}' (y {:.1}..{:.1}) e '{nb}' (y {:.1}..{:.1}) ocupam a MESMA \
                 banda — as duas seccoes desenham-se uma por cima da outra",
                a.y,
                a.y + a.h,
                b.y,
                b.y + b.h,
            );
        }
    }
}

/// **E elas descem na ORDEM em que se pintam** — a de baixo começa depois de a de cima acabar.
///
/// ⚠️ A metade que a anterior não cobre: duas bandas podem não se cruzar e ainda assim estar
/// trocadas, o que faria a lista de uma secção aparecer debaixo do cabeçalho da outra.
#[test]
fn each_section_starts_below_the_one_before_it() {
    let bandas = section_bands();
    for par in bandas.windows(2) {
        let (na, a) = par[0];
        let (nb, b) = par[1];
        assert!(
            b.y >= a.y + a.h,
            "'{nb}' comeca em {:.1} e '{na}' so' acaba em {:.1} — a ordem de pintura e a ordem no \
             ecra discordam",
            b.y,
            a.y + a.h,
        );
    }
}
