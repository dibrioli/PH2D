//! ⭐⭐⭐ **As secções do Audio Editor abrem o menu de TEMA e ARRASTAM pela pega** — o gesto REAL,
//! pelo despachante, com os rects que a pintura registou (ordem do dono, 2026-09-30: *«siga com os
//! outros painéis»*, depois de as duas coisas nascerem no Inspector e chegarem ao Vector e ao
//! Painter). Os irmãos deste gate vivem nesses painéis.
//!
//! ⚠️ Uma lei de ordem verde numa função pura não diz nada sobre as três costuras que este gate
//! percorre: o título ser um alvo do menu de SECÇÃO (e não o de notas), a pega estar viva sob o
//! rato e vencer o cabeçalho que dobra, e a ordem gravada chegar à PINTURA seguinte.
//!
//! ⚠️ As oito secções deste painel ARRASTAM-SE todas — não há fixa nem aninhada (o doc do
//! `paint_sections::paint_body` diz porquê), logo o gate mede as duas costuras da móvel.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_audio_editor::state::AudioEditorState;
use ph2d_panel_audio_editor::{
    AEDIT_SEC_DELIVERY, AEDIT_SEC_EDIT, AEDIT_SEC_FX, AEDIT_SEC_LOOP, AEDIT_SEC_MARKERS,
    AEDIT_SEC_SPECTRAL, AEDIT_SEC_TRANSPORT, AEDIT_SEC_VARIATIONS, AudioEditorPanel,
};
use ph2d_ui_testkit::MockPanelHost;

const SEC: u128 = 1_000_000_000;

/// Alto de propósito: o corpo transborda o encaixe, e o gate quer as secções à vista.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 600.0,
    h: 4000.0,
};

/// As secções do painel, pela ordem natural.
const SECCOES: [NodeId; 8] = [
    AEDIT_SEC_TRANSPORT,
    AEDIT_SEC_LOOP,
    AEDIT_SEC_EDIT,
    AEDIT_SEC_SPECTRAL,
    AEDIT_SEC_FX,
    AEDIT_SEC_MARKERS,
    AEDIT_SEC_VARIATIONS,
    AEDIT_SEC_DELIVERY,
];

fn pointer(kind: PointerKind, button: PointerButton, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .rev()
        .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// As secções à vista, pela ordem pintada — `(id, rect do cabeçalho)`.
fn a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = SECCOES
        .iter()
        .filter_map(|&id| rect_of(rects, id).map(|r| (id, r)))
        .collect();
    v.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    v
}

fn pintado() -> (MockPanelHost, AudioEditorState, Vec<(NodeId, Rect)>) {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut st = AudioEditorState;
    let rects = host.paint::<AudioEditorPanel>(&mut st, VIEWPORT);
    (host, st, rects)
}

/// ⭐⭐ **O botão direito no título de uma secção abre o menu de tema DELA, e ela tem pega.**
/// *Mutação: o cabeçalho registado pelo `hit_index.register` de antes ⇒ o menu aberto é o de
/// notas e a pega não existe no índice.*
#[test]
fn uma_seccao_tem_menu_de_tema_e_pega() {
    let (mut host, _, rects) = pintado();
    let vista = a_vista(&rects);
    assert_eq!(
        vista.len(),
        SECCOES.len(),
        "fixtura: com o viewport alto, os oito cabeçalhos estão à vista ({vista:?})"
    );
    for &(alvo, _) in &vista {
        assert!(
            rect_of(&rects, ph2d_editor_core::ids::grip_de(alvo)).is_some(),
            "a secção {alvo:?} não registou a pega"
        );
    }
    let (alvo, cab) = vista[1];
    // ⚠️ Num ponto do título à ESQUERDA da pega e do resumo — a pega é outro alvo.
    let (x, y) = (cab.x + cab.w * 0.25, cab.y + cab.h * 0.5);
    let _ = host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        x,
        y,
        SEC,
    ));
    let menu = host.store().context_menu().map(|m| m.kind);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == alvo),
        "o botão direito no título abriu {menu:?} em vez do menu de tema da secção"
    );
}

/// ⭐⭐ **Arrastar a pega da segunda secção para cima da primeira põe-na à frente** — e a pintura
/// seguinte segue a ordem gravada. *Mutação: o plano pela ordem natural ⇒ a segunda continua
/// atrás.*
#[test]
fn arrastar_a_pega_poe_a_seccao_onde_se_larga() {
    let (mut host, mut st, rects) = pintado();
    let vista = a_vista(&rects);
    assert!(vista.len() >= 2, "fixtura: duas secções à vista");
    let (primeira, cab1) = vista[0];
    let (segunda, _) = vista[1];
    let pega =
        rect_of(&rects, ph2d_editor_core::ids::grip_de(segunda)).expect("a segunda tem pega");
    let (px, py) = (pega.x + pega.w * 0.5, pega.y + pega.h * 0.5);
    let alvo_y = cab1.y + 1.0;
    for (kind, y, t) in [
        (PointerKind::Down, py, SEC),
        (PointerKind::Move, alvo_y, SEC + 1),
        (PointerKind::Up, alvo_y, SEC + 2),
    ] {
        let _ = host.dispatch_pointer_event(pointer(kind, PointerButton::Primary, px, y, t));
    }
    assert_eq!(
        host.store().section_order().first(),
        Some(&segunda),
        "a ordem gravada não pôs a secção arrastada à frente"
    );
    let depois = a_vista(&host.paint::<AudioEditorPanel>(&mut st, VIEWPORT));
    let pos = |id: NodeId| depois.iter().position(|(n, _)| *n == id);
    assert!(
        pos(segunda) < pos(primeira),
        "a pintura seguinte não seguiu a ordem gravada ({depois:?})"
    );
}

/// ⭐ **A pega ACENDE sob o rato e traz a dica** — duas metades, cada uma de um dono.
///
/// - **O rato acende a PEGA e não o cabeçalho que dobra:** ela é registada DEPOIS do cabeçalho
///   (`section_plan::regista_cabecalho`), e o índice resolve o último primeiro. É isso que acende
///   os dez pontos no `section_plan::cabecalho`.
/// - **A dica é a do `populate`** (`chrome.section.grip_hint`, a mesma do Inspector).
///
/// ⚠️ MEDIDO (prova de mutação de 2026-09-30): o registo `Plain` da pega no `populate` **não é
/// observável por gesto nenhum** — o arrasto resolve a pega pelo LIVRO do quadro e o hover pelo
/// índice de acerto, e a mutação que tira só o `register` SOBREVIVE a este ficheiro inteiro. O que
/// ele compra é a paridade com o Inspector/Vector/Painter (`is_focusable`). *Mutação que sangra: o
/// laço da pega fora do `populate` ⇒ não há dica.*
#[test]
fn a_pega_acende_sob_o_rato_e_tem_dica() {
    let (mut host, _, rects) = pintado();
    let (alvo, _) = a_vista(&rects)[0];
    let pega_id = ph2d_editor_core::ids::grip_de(alvo);
    let pega = rect_of(&rects, pega_id).expect("a secção tem pega");
    let _ = host.dispatch_pointer_event(pointer(
        PointerKind::Move,
        PointerButton::Primary,
        pega.x + pega.w * 0.5,
        pega.y + pega.h * 0.5,
        SEC,
    ));
    assert_eq!(
        host.store().hot_id(),
        Some(pega_id),
        "o rato sobre a pega não a acendeu"
    );
    assert!(
        host.store()
            .tooltip_for(pega_id)
            .is_some_and(|t| !t.is_empty()),
        "a pega não tem dica"
    );
}
