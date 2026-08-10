//! **Sonda: a calha que cada rótulo de campo numérico recebe.**
//!
//! Existe porque os números do report de 2026-08-02 (*"label sobreposta"*) são citados nos
//! doc-comments do painel e na mensagem do commit — e um número citado que nenhuma sonda imprime
//! mais deixou de ser reproduzível.
//!
//! Rodar: `cargo test -p ph2d-panel-vector --test probe_gutters -- --ignored --nocapture`
//!
//! O que ela mediu no dia da correção (painel de 252 px de largura interna):
//!
//! | rótulo | calha ANTES | calha DEPOIS | quanto do rótulo ficava sob o campo |
//! |---|---|---|---|
//! | `All`  | 12,00 px | 18,55 px | 6,6 px |
//! | `Gap`  | 12,00 px | 27,20 px | 15,2 px |
//! | `Grow` | 12,00 px | 34,39 px | 22,4 px |
//!
//! A calha ANTES era a mesma para todos — `Spacing::Md` (8) + `Spacing::Xs` (4) —, dimensionada
//! quando todo rótulo desta função era `X`/`Y`/`W`/`H`.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::{LayoutFlow, LayoutItem, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn rect(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
}

#[test]
#[ignore = "sonda: imprime medidas, nao afirma nada"]
fn measure_the_label_gutters() {
    state::set_frame_clip(Some(true));
    state::set_layout_flow(Some(LayoutFlow {
        size: [
            ids::VECTOR_LAYOUT_SIZE_W_FIXED,
            ids::VECTOR_LAYOUT_SIZE_H_FIXED,
        ],
        min: [0.0; 2],
        max: [0.0; 2],
        dir: ids::VECTOR_LAYOUT_DIR_ROW,
        gap: [0.0, 0.0],
        pad: [0.0; 4],
        align: ids::VECTOR_LAYOUT_ALIGN_START,
        justify: ids::VECTOR_LAYOUT_JUSTIFY_START,
    }));
    state::set_layout_item(Some(LayoutItem {
        absolute: false,
        grow: 0.0,
        shrink: 0.0,
    }));
    let off = rect(ids::VECTOR_LAYOUT_DIR_OFF).expect("o chip Off");
    let wrap = rect(ids::VECTOR_LAYOUT_DIR_WRAP).expect("o chip Wrap");
    println!(
        "inner_x={:.2} inner_right={:.2} (largura interna {:.2})",
        off.x,
        wrap.x + wrap.w,
        wrap.x + wrap.w - off.x
    );
    for (id, name) in [
        (ids::VECTOR_LAYOUT_GAP_MAIN, "Gap"),
        (ids::VECTOR_LAYOUT_PAD_ALL, "All"),
        (ids::VECTOR_LAYOUT_ITEM_GROW, "Grow"),
    ] {
        let r = rect(id).expect("o campo");
        println!(
            "{name:>7}: calha={:.2}px campo=[{:.1}..{:.1}] w={:.1}",
            r.x - off.x,
            r.x,
            r.x + r.w,
            r.w
        );
    }
    state::set_frame_clip(None);
    state::set_layout_flow(None);
    state::set_layout_item(None);
}
