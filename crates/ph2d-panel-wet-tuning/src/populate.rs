//! Widget registration — walked from [`crate::rows`], so a painted row cannot
//! be an unregistered one (an id the store never heard of is not focusable
//! and its click is dropped in silence — the wiring-parity class of bug).

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

use crate::rows;

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub fn populate(store: &mut WidgetStore) {
    for row in rows::rows() {
        if let Some(opts) = &row.options {
            // A CHOICE row is buttons (one per option) + the reset — no slider, no chip.
            for (_, id) in opts {
                button(store, *id);
            }
            button(store, row.reset);
            continue;
        }
        store.register(
            row.slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            row.chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        // Track (0..1) ↔ chip (real value): scale/offset from the def range.
        store.link_slider_number_mapped(
            row.slider,
            row.chip,
            (row.max - row.min) as f32,
            row.min as f32,
        );
        // ⚠️ Not optional: without a registered range+step the chip derives
        // its drag step from the buffer text and scrubs ~50 units per pixel
        // (min↔max switch; typing still works, which hides it).
        store.set_number_range(row.chip, row.min, row.max, row.step);
        button(store, row.reset);
    }
    // ⭐⭐ **Um cabeçalho é uma secção DOBRÁVEL da casa** (`mark_collapsible_section`), sem estado de
    //    botão — o cânone da Física (24/09): o `apply_click` do despacho dobra-o e ainda emite o `Click`.
    //    Registados como `Button`, os onze cabeçalhos liam-se como onze COMANDOS na catraca
    //    `a_carga_de_comandos_de_um_painel_so_encolhe` (27/10: `26` contra `16`).
    for section in rows::SECTIONS {
        store.mark_collapsible_section(section.header);
        button(store, section.reset);
        // Os avançados (25/10) nascem dobrados — pela porta que não pisa a escolha do artista.
        if ph2d_tool_painter::ids::WET_TUNING_GRUPOS_AVANCADOS.contains(&section.group) {
            store.set_collapsed_if_unchosen(section.header, true);
        }
    }
    store.mark_collapsible_section(crate::plano::EXPERIMENTAL);
    button(store, ph2d_tool_painter::ids::WET_TUNING_KM_MIXING);
    button(store, ph2d_tool_painter::ids::WET_TUNING_KM_GLAZE);
    button(store, ph2d_tool_painter::ids::WET_TUNING_CLOSE);
    // ⭐⭐ **A PEGA de dez pontos das secções que se ARRASTAM** (2026-09-30, *«siga com os outros
    //    painéis»*): `Plain` para o `is_focusable` responder e o rato a acender, com a mesma dica
    //    do Inspector. As seis se arrastam (`crate::plano`).
    for id in crate::plano::seccoes_moveis() {
        let grip = ids::grip_de(id);
        store.register(grip, InteractiveState::Plain);
        store.set_tooltip(grip, ph2d_i18n::tr("chrome.section.grip_hint"));
    }
    // Drag + resize handles, parented to the panel's own NodeId — the same
    // panel-agnostic machinery the Inspector/Hierarchy chrome uses: Down on
    // one of these anchors the store's drag/resize state, Moves accumulate
    // into `blender_picker_offset` / `panel_resize_delta` under the PARENT
    // id, and `paint` reads those back to place the rect.
    store.register(
        ph2d_tool_painter::ids::WET_TUNING_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::WET_TUNING_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ph2d_tool_painter::ids::WET_TUNING_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::WET_TUNING_PANEL,
            kind: BlenderHitKind::ResizeHandle,
        },
    );
    store.register(
        ph2d_tool_painter::ids::WET_TUNING_RESIZE_HANDLE_BL,
        InteractiveState::BlenderHit {
            parent: ids::WET_TUNING_PANEL,
            kind: BlenderHitKind::ResizeHandleBl,
        },
    );
}
