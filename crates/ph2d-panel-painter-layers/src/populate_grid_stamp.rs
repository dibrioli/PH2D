//! Os widgets do card **Grid Stamp** — os quatro sliders da grade (tamanho da célula e deslocamento,
//! um par por eixo), os quatro chips numéricos ligados a eles, e o checkbox `Show Grid`.
//!
//! Eles moram juntos porque são UM card: pintam juntos, aparecem e somem juntos com o método, e
//! espalhá-los pelas listas gerais (sliders numa, toggles noutra, chips numa terceira) é como o
//! quinto nasceria sem par — o modo de falha que este painel já pagou é justamente o widget que
//! pinta, registra hit rect e fica **morto sob o mouse** por faltar num `populate` distante.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

/// O incremento do stepper/arrasto do chip no trilho `0..1` (valor de COMPORTAMENTO, não token).
const STEP: f64 = 0.01; // LITERAL-PX-OK: chip 0..1 track step (non-design behaviour value)

pub(crate) fn register_grid_stamp(store: &mut WidgetStore) {
    use ph2d_editor_core::ids as core_ids;
    let pairs = [
        (
            core_ids::PAINTER_BRUSH_GRID_CELL[0],
            core_ids::PAINTER_BRUSH_GRID_CELL_CHIPS[0],
        ),
        (
            core_ids::PAINTER_BRUSH_GRID_CELL[1],
            core_ids::PAINTER_BRUSH_GRID_CELL_CHIPS[1],
        ),
        (
            core_ids::PAINTER_BRUSH_GRID_OFFSET[0],
            core_ids::PAINTER_BRUSH_GRID_OFFSET_CHIPS[0],
        ),
        (
            core_ids::PAINTER_BRUSH_GRID_OFFSET[1],
            core_ids::PAINTER_BRUSH_GRID_OFFSET_CHIPS[1],
        ),
    ];
    for (slider, chip) in pairs {
        store.register(
            slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number(slider, chip);
        store.set_number_range(chip, 0.0, 1.0, STEP);
    }
    // `Show Grid` — desenho, nunca um parametro do carimbo (ver `toggle_grid_show`).
    store.register(
        core_ids::PAINTER_BRUSH_GRID_SHOW,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}
