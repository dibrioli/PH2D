//! ⭐⭐⭐ **A PEGA reordena as secções — o gesto REAL, do Down ao quadro seguinte** (ordem do dono,
//! 2026-09-29: *«um ícone de 10 pontos que serve para arrastar e reorganizar as seções»*).
//!
//! ⚠️ O gesto passa pelo despachante com os rects que a pintura REGISTOU — é isso que prova que a
//! pega está viva sob o rato (e não só desenhada), que ela vence o cabeçalho que dobra a secção, e
//! que a ordem gravada chega à PINTURA seguinte. Uma lei de ordem verde numa função pura não diz
//! nada sobre nenhuma destas três costuras.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
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

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn sprite() -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        emissive: 0.0,
        entity_bits: 0x5EC7_10A5,
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
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} nao foi registado pela pintura"))
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐⭐ **Arrastar a pega da Sprite Sheet para cima do Render Source põe-na no topo das três**, e
/// um clique PARADO na pega não reordena nem dobra nada. *Mutações: tirar o registo da pega no
/// `begin_section` ⇒ o Down cai no cabeçalho e dobra a secção; o plano a ignorar a ordem ⇒ a
/// Sprite Sheet continua em baixo; o plano a não devolver o fantasma ⇒ o quadro a meio do arrasto
/// tem a mesma geometria do de depois.*
#[test]
fn arrastar_a_pega_poe_a_seccao_onde_se_larga() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_sprite(Some(sprite()));
    let r = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let render = rect_of(&r, core_ids::INSP_LIVE_RENDER_SECTION);
    let sheet = rect_of(&r, core_ids::INSP_LIVE_SHEET_SECTION);
    let pega = rect_of(&r, core_ids::INSP_LIVE_SHEET_GRIP);
    assert!(
        sheet.y > render.y,
        "fixtura: a Sprite Sheet comeca por baixo"
    );
    let dobrada = h.store().is_collapsed(core_ids::INSP_LIVE_SHEET_SECTION);

    // Um clique PARADO na pega: nada muda.
    let (px, py) = centro(pega);
    let _ = h.dispatch_pointer_event(pointer(PointerKind::Down, px, py, SEC));
    let _ = h.dispatch_pointer_event(pointer(PointerKind::Up, px, py, SEC + 1));
    assert!(
        h.store().section_order().is_empty(),
        "um clique parado reordenou"
    );
    assert_eq!(
        h.store().is_collapsed(core_ids::INSP_LIVE_SHEET_SECTION),
        dobrada,
        "o Down na pega caiu no cabecalho e dobrou a seccao"
    );

    // O gesto: pega, arrasta até acima do meio do cabeçalho do Render Source, larga.
    let alvo_y = render.y + 1.0;
    let _ = h.dispatch_pointer_event(pointer(PointerKind::Down, px, py, 2 * SEC));
    let _ = h.dispatch_pointer_event(pointer(PointerKind::Move, px, alvo_y, 2 * SEC + 1));
    // ⭐ A meio do arrasto o FANTASMA pinta-se (2026-09-30: *«permita ver o card sendo arrastado,
    //    menor e meio transparente»*) — a secção inteira uma segunda vez, logo muito mais geometria.
    let (_, a_meio) = h.paint_and_count_geometry::<InspectorPanel>(&mut st, VIEWPORT);
    let _ = h.dispatch_pointer_event(pointer(PointerKind::Up, px, alvo_y, 2 * SEC + 2));
    let (_, depois) = h.paint_and_count_geometry::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        a_meio > depois,
        "a meio do arrasto nada mais se pintou (sem fantasma): {a_meio} contra {depois}"
    );
    assert!(
        !h.store().section_order().is_empty(),
        "a queda nao gravou ordem nenhuma"
    );

    let r = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let render = rect_of(&r, core_ids::INSP_LIVE_RENDER_SECTION);
    let sheet = rect_of(&r, core_ids::INSP_LIVE_SHEET_SECTION);
    let color = rect_of(&r, core_ids::INSP_LIVE_COLOR_SECTION);
    set_current_inspector_sprite(None);
    assert!(
        sheet.y < render.y && render.y < color.y,
        "a Sprite Sheet nao subiu para o topo das tres: sheet {} · render {} · color {}",
        sheet.y,
        render.y,
        color.y
    );
}
