//! ⭐⭐⭐ **As NOTAS do Inspector — a pega, o menu do botão direito e o fantasma, pelo gesto REAL**
//! (ordem do dono, 2026-09-30: *«Coloque os 10 pontinhos de arrastar também nas notas. Botão
//! direito sobre as notas devem ter no menu a mudança de cor das notas, opções de apagar e
//! duplicar.»*).
//!
//! ⚠️ Os gestos passam pelo despachante com os rects que a pintura REGISTOU: é isso que prova que
//! a pega vence o título por baixo dela, que o botão direito sobre o TÍTULO abre o menu da nota
//! (até 2026-09-30 nunca abria — a pertença lia uma faixa de ids que os hashes nunca atingem) e
//! que a nota largada noutra secção se PINTA lá no quadro seguinte.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::screens::hero::{
    InspectorSpriteInfo, InspectorSpriteMixed, InspectorSpriteSource,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_sprite};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 600.0,
    h: 1600.0,
};
const SEC: u128 = 1_000_000_000;
const P: NodeId = core_ids::INSP_PANEL;

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

fn sprite() -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        emissive: 0.0,
        entity_bits: 0x5EC7_10A6,
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Atlas { key: 3 },
        source_precision: Some(ph2d_editor_core::Precision::Rgba8),
        sheet_label: None,
        source_pixels: Some((64, 64)),
        can_reimport: false,
        flip_x: false,
        flip_y: false,
        opacity: 1.0,
        tint_fill: false,
        hframes: 1,
        vframes: 1,
        frame: 0,
        tint: [1.0; 4],
        self_tint: [1.0; 4],
        per_corner_tint: [[1.0; 4]; 4],
        region_enabled: false,
        region_rect: [0.0, 0.0, 64.0, 64.0],
        region_filter_clip: true,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSpriteMixed::default(),
    }
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Rect {
    rects
        .iter()
        .rev()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} nao foi registado pela pintura"))
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐⭐ **A nota pinta-se no fim da SECÇÃO dela, tem a pega por cima do título, e o botão
/// direito sobre o título abre o menu DELA — com Duplicate e Delete a funcionar.** *Mutações: a
/// pertença pela faixa `800..=811` ⇒ o menu que abre é o de criar nota; a pega registada ANTES do
/// título ⇒ o Down cai na caixa de texto; `note_duplicate` a não inserir ⇒ fica uma nota.*
#[test]
fn a_nota_tem_pega_e_o_menu_dela_abre_no_titulo() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_sprite(Some(sprite()));
    assert_eq!(
        h.add_note(P, Some(core_ids::INSP_LIVE_COLOR_SECTION)),
        Some(0)
    );
    let r = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let color = rect_of(&r, core_ids::INSP_LIVE_COLOR_SECTION);
    let sheet = rect_of(&r, core_ids::INSP_LIVE_SHEET_SECTION);
    let nota = rect_of(&r, core_ids::NOTE_SLOT_IDS[0]);
    let titulo = rect_of(&r, core_ids::NOTE_TITLE_IDS[0]);
    let pega = rect_of(&r, core_ids::NOTE_GRIP_IDS[0]);
    assert!(
        color.y < nota.y && nota.y < sheet.y,
        "a nota nao se pintou no fim da Color & Tint: color {} · nota {} · sheet {}",
        color.y,
        nota.y,
        sheet.y
    );
    assert!(
        pega.x >= titulo.x + titulo.w - 0.5 && pega.x + pega.w <= nota.x + nota.w + 0.5,
        "a pega nao ficou a direita do titulo, dentro da nota: pega {pega:?} · titulo {titulo:?}"
    );
    let (px, py) = centro(pega);
    assert_eq!(
        h.hit_at(px, py),
        Some(core_ids::NOTE_GRIP_IDS[0]),
        "a pega nao vence o que esta por baixo dela"
    );

    // O botão direito sobre o TÍTULO abre o menu da nota.
    let (tx, ty) = centro(titulo);
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        tx,
        ty,
        SEC,
    ));
    let menu = h.store().context_menu().map(|m| m.kind);
    set_current_inspector_sprite(None);
    assert_eq!(
        menu,
        Some(ContextMenuKind::NoteBackground {
            panel: P,
            note_index: 0
        }),
        "o botao direito sobre o titulo da nota nao abriu o menu dela"
    );
    assert!(h.choose_context_menu_row(core_ids::CTX_MENU_NOTE_DUPLICATE));
    assert_eq!(
        h.store().notes_for_panel(P).len(),
        2,
        "Duplicate nao duplicou"
    );
    // e o Delete pelo mesmo caminho
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        tx,
        ty,
        2 * SEC,
    ));
    assert!(h.choose_context_menu_row(core_ids::CTX_MENU_NOTE_DELETE));
    assert_eq!(h.store().notes_for_panel(P).len(), 1, "Delete nao apagou");
}

/// ⭐⭐ **Arrastar a pega da nota para dentro da Sprite Sheet muda-a de secção — e enquanto a mão
/// anda, o FANTASMA pinta-se.** Um clique parado na pega não move nada. *Mutações: o `drop` a
/// não chamar `note_move` ⇒ a nota fica na Color & Tint; o fantasma não pintado ⇒ a geometria do
/// quadro a meio do arrasto é a mesma da de depois.*
#[test]
fn arrastar_a_pega_da_nota_muda_a_de_seccao_e_mostra_o_fantasma() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_sprite(Some(sprite()));
    h.add_note(P, Some(core_ids::INSP_LIVE_COLOR_SECTION));
    let r = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let pega = rect_of(&r, core_ids::NOTE_GRIP_IDS[0]);
    let sheet = rect_of(&r, core_ids::INSP_LIVE_SHEET_SECTION);
    let (px, py) = centro(pega);

    // clique parado: nada
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Primary,
        px,
        py,
        SEC,
    ));
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Up,
        PointerButton::Primary,
        px,
        py,
        SEC + 1,
    ));
    assert_eq!(
        h.store().notes_for_panel(P)[0].section,
        Some(core_ids::INSP_LIVE_COLOR_SECTION),
        "um clique parado na pega moveu a nota"
    );

    // o gesto: pega, arrasta para dentro da Sprite Sheet
    let alvo_y = sheet.y + sheet.h + 4.0;
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Primary,
        px,
        py,
        2 * SEC,
    ));
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Move,
        PointerButton::Primary,
        px,
        alvo_y,
        2 * SEC + 1,
    ));
    let (_, a_meio) = h.paint_and_count_geometry::<InspectorPanel>(&mut st, VIEWPORT);
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Up,
        PointerButton::Primary,
        px,
        alvo_y,
        2 * SEC + 2,
    ));
    let (_, depois) = h.paint_and_count_geometry::<InspectorPanel>(&mut st, VIEWPORT);
    assert_eq!(
        h.store().notes_for_panel(P)[0].section,
        Some(core_ids::INSP_LIVE_SHEET_SECTION),
        "a nota largada na Sprite Sheet nao mudou de seccao"
    );
    let r = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    set_current_inspector_sprite(None);
    let sheet = rect_of(&r, core_ids::INSP_LIVE_SHEET_SECTION);
    let nota = rect_of(&r, core_ids::NOTE_SLOT_IDS[0]);
    assert!(
        nota.y > sheet.y,
        "a nota nao se pinta na Sprite Sheet: nota {} · sheet {}",
        nota.y,
        sheet.y
    );
    assert!(
        a_meio > depois,
        "a meio do arrasto nada mais se pintou (sem fantasma): {a_meio} contra {depois}"
    );
}
