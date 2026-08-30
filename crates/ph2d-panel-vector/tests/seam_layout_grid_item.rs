//! **Um filho de moldura em GRADE não recebe Grow/Shrink.**
//!
//! ## O defeito, medido em 2026-08-30
//!
//! `grow`/`shrink` viajam para o `taffy` como `flex_grow`/`flex_shrink` — o trait de **flex**. Uma
//! moldura `LayoutDir::Grid` é resolvida por outro motor, e a medição é categórica:
//!
//! | | `flex_grow` |
//! |---|---:|
//! | `taffy-0.14.0/src/compute/grid/` | **0** ocorrências |
//! | `taffy-0.14.0/src/compute/flexbox.rs` | **13** |
//!
//! ⇒ sob uma grade os dois números **chegam ao motor e são descartados lá**. É a segunda espécie
//! de knob morto que a caça de 2026-08-30 nomeou — *o consumidor projecta o valor fora* —, e é a
//! que nenhuma sonda de «quem lê este campo?» apanha: ele **é** lido.
//!
//! ## A lei já estava escrita duas vezes
//!
//! O mesmo painel esconde as duas fileiras para um filho `absolute` (*«quem não está no fluxo não
//! reparte sobra nenhuma»*), e o `VecLayout::columns` do `ph2d-ecs` traz a regra em prosa: *«o
//! painel não o pinta onde ele não move um pixel»*. Esta é a terceira aplicação.
//!
//! ⚠️ **A pergunta é do PAI.** O filho não sabe como o pai dispõe; quem sabe é a shell, e por isso
//! o facto viaja no `LayoutItem::parent_is_grid` — do lado do `in_flow`, que já lê o mesmo
//! componente do mesmo pai.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::{LayoutFlow, LayoutItem, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

/// Publica uma moldura que flui e um filho dentro dela; `grid` diz como o PAI dispõe.
fn publish(grid: bool) {
    ph2d_panel_vector::state::set_layout_flow(Some(LayoutFlow {
        dir: if grid {
            ids::VECTOR_LAYOUT_DIR_GRID
        } else {
            ids::VECTOR_LAYOUT_DIR_ROW
        },
        gap: [0.0; 2],
        pad: [0.0; 4],
        align: ids::VECTOR_LAYOUT_ALIGN_START,
        justify: ids::VECTOR_LAYOUT_JUSTIFY_START,
        size: [
            ids::VECTOR_LAYOUT_SIZE_W_FIXED,
            ids::VECTOR_LAYOUT_SIZE_H_FIXED,
        ],
        min: [0.0; 2],
        max: [0.0; 2],
        columns: 2.0,
    }));
    ph2d_panel_vector::state::set_layout_item(Some(LayoutItem {
        grow: 0.0,
        shrink: 1.0,
        absolute: false,
        in_flow: true,
        parent_is_grid: grid,
    }));
}

const ROWS: [(&str, ph2d_a11y::NodeId); 2] = [
    ("Grow", ids::VECTOR_LAYOUT_ITEM_GROW),
    ("Shrink", ids::VECTOR_LAYOUT_ITEM_SHRINK),
];

/// **Controle POSITIVO: sob uma moldura que FLUI, as duas fileiras existem.**
///
/// Sem ele, um painel que deixasse de as pintar em qualquer caso passaria no teste de baixo — e a
/// feature estaria morta em vez de correctamente escondida.
#[test]
fn a_child_of_a_flex_frame_still_gets_grow_and_shrink() {
    publish(false);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for (name, id) in ROWS {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_some(),
            "a fileira {name} sumiu para um filho de moldura em Row — ali ela FUNCIONA, e \
             escondê-la seria tirar uma feature que existe"
        );
    }
}

/// **Sob uma GRADE, nenhuma das duas é oferecida.**
///
/// Mutação que tem de sangrar: tirar o `&& !it.parent_is_grid` de
/// `paint_layout::layout_item_rows`.
#[test]
fn a_child_of_a_grid_frame_is_not_offered_grow_or_shrink() {
    publish(true);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for (name, id) in ROWS {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_none(),
            "a fileira {name} foi pintada para um filho de moldura em GRADE. Os dois números são \
             `flex_grow`/`flex_shrink`, e o motor de grade do `taffy` não os lê (0 ocorrências em \
             `compute/grid/` contra 13 em `compute/flexbox.rs`): o artista arrasta e nada se move."
        );
    }
}
