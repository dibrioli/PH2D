//! ⭐⭐⭐ **O CABEÇALHO DA ÁREA É A CASA DAS OPÇÕES DE EXIBIÇÃO DELA** — a decisão **D2**, medida
//! como GESTO.
//!
//! > *«Barra global para o que é do aplicativo inteiro; cabeçalho por área para o que é da
//! > ferramenta.»* — D2
//!
//! # ⚠️ Por que o gate carrega num PIXEL
//!
//! Um controlo pintado e registado pode estar **morto sob o dedo** — é a família que o
//! `CLAUDE.md` §5.0 nomeia, e que este repo já pagou nos quatro chips do vetor e no pill
//! `[SHEET]`. Um `apply_event(Click(id))` sintético passa com o controlo morto: ele salta
//! exactamente a metade que falha (o `HitIndex` resolver o ponto, e o `WidgetStore` ter estado
//! interactivo para armar o `active`).
//!
//! ⇒ aqui o gesto é `Down` no pixel + `Up`, pela mesma porta do produto.

use ph2d_editor_core::screens::hero::{HeroScreen, area_header};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 1024.0,
};

fn hero() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    // ⚠️ O cabeçalho é do chrome NOVO: no legado o trilho é vertical e a área não o tem.
    h.view.legacy_chrome = false;
    h
}

fn paint(h: &mut HeroScreen) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..2 {
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
    }
}

fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

/// Carrega no centro de `r`, pela porta do produto.
fn click_at(h: &mut HeroScreen, r: Rect) {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let arena = bumpalo::Bump::new();
    for ev in [
        pointer(PointerKind::Down, x, y),
        pointer(PointerKind::Up, x, y),
    ] {
        let events =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena);
        let evs: Vec<_> = events.to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

/// Onde cada opção está no ecrã, depois de um quadro.
fn option_rects(h: &mut HeroScreen) -> Vec<(ph2d_editor_core::NodeId, &'static str, Rect)> {
    paint(h);
    let mut text = TextSystem::without_system_fonts();
    let layout = ph2d_editor_core::screens::hero::frame_layout::frame_layout(h, VIEWPORT);
    area_header::option_rects(layout.area_header, &mut text)
}

/// ⭐⭐⭐ **Carregar em cada opção do cabeçalho vira o valor dela.**
#[test]
fn every_display_option_in_the_header_is_alive_under_the_finger() {
    let mut h = hero();
    let rects = option_rects(&mut h);
    assert_eq!(
        rects.len(),
        area_header::DISPLAY_OPTIONS.len(),
        "controlo: as opções não couberam no alvo de referência e o gate mediria o vazio"
    );
    for (id, label, r) in rects {
        let before = ph2d_editor_core::screens::hero::menu_bar::module_is_on(&h, id)
            .unwrap_or_else(|| panic!("{label} não está na tabela de verdade dos menus"));
        click_at(&mut h, r);
        let after = ph2d_editor_core::screens::hero::menu_bar::module_is_on(&h, id).unwrap();
        assert_ne!(
            before, after,
            "{label}: o clique no pixel do cabeçalho não virou o valor — o controlo está morto \
             sob o dedo"
        );
    }
}

/// ⛔⛔ **E elas SAÍRAM dos menus do app** — um sítio canónico por comando (D2).
///
/// ⚠️ Uma entrada repetida em dois menus é a tabela paralela com o sintoma pior: os dois estados a
/// discordar à vista.
#[test]
fn no_display_option_of_the_area_is_still_in_an_app_menu() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    use ph2d_editor_core::screens::hero::menu_rows::menu_rows;
    let kinds = [
        ContextMenuKind::MenuBarView,
        ContextMenuKind::MenuBarWindow,
        ContextMenuKind::MenuBarFile,
        ContextMenuKind::MenuBarEdit,
        ContextMenuKind::MenuBarRun,
        ContextMenuKind::ThemeSelector,
    ];
    let mut seen_rows = 0usize;
    let mut sins = Vec::new();
    for kind in kinds {
        for (id, label, _) in menu_rows(kind) {
            seen_rows += 1;
            if area_header::DISPLAY_OPTIONS.iter().any(|(o, _)| o == id) {
                sins.push(format!("{label} ({kind:?})"));
            }
        }
    }
    assert!(
        seen_rows >= 20,
        "controlo: só {seen_rows} linhas varridas — a varredura dos menus esvaziou-se"
    );
    assert!(
        sins.is_empty(),
        "opções de exibição da ÁREA ainda num menu do APP: {sins:?}"
    );
}

/// ⭐ **E o cabeçalho nunca partilha um pixel com a régua nem com a fila** — a lei do modelo.
#[test]
fn the_header_never_shares_a_pixel_with_the_regions_below_it() {
    let mut h = hero();
    paint(&mut h);
    let l = ph2d_editor_core::screens::hero::frame_layout::frame_layout(&h, VIEWPORT);
    assert!(
        l.area_header.h > 1.0,
        "controlo: o cabeçalho tem altura zero e o gate mediria o nada"
    );
    let head_bottom = l.area_header.y + l.area_header.h;
    assert!(
        l.tool_bar.y >= head_bottom - 0.01,
        "a fila começa dentro do cabeçalho ({} < {head_bottom})",
        l.tool_bar.y
    );
    assert!(
        l.draw_area.y >= head_bottom - 0.01,
        "a área de desenho começa dentro do cabeçalho"
    );
}
