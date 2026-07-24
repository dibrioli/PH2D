//! Flip Style panel widget registration (called once at panel install).
//!
//! Registers the FIXED widgets: the three mode buttons (Select/Draw/Erase), the
//! four Brush sliders + chips (Size/Hardness/Opacity/Smoothing), the three Erase
//! sub-mode buttons, and the Layers toolbar (Add/Delete). Each brush slider is
//! seeded at the tool's default so its knob renders correctly before the first
//! drag (the slider is absolute-position — the store value drives the render AND
//! the drag baseline). The per-LAYER row widgets are dynamic (`register_if_absent`
//! at paint time in `paint_layers`); the Stroke swatch needs no store entry (its
//! Down is the generic `is_picker_swatch` dispatch).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_flip::{
    DEFAULT_HARDNESS, DEFAULT_OPACITY, DEFAULT_PRECISION, DEFAULT_SMOOTHING, DEFAULT_WIDTH_PX,
    GAP_MAX_PX, GROW_MAX, GROW_MIN, OPACITY_SLIDER_SCALE, PRECISION_MAX, PRECISION_MIN,
    TRAP_MAX_PX, WIDTH_SLIDER_OFFSET, WIDTH_SLIDER_SCALE, px_to_slider,
};

/// Register a plain action Button in the Normal state.
fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// Register a slider + its linked value chip, seeded at `track` / `display`.
///
/// **O range da caixa é registrado aqui, e isso não é opcional.** Sem
/// `set_number_range`, o drag-scrub deriva o passo do texto do buffer e anda a
/// ~50 unidades por PIXEL: um pixel de arrasto já bate no teto, e a caixa vira um
/// liga/desliga min↔max em vez de um controle. As setas do stepper também erram (a
/// Precision andaria de 1 em 1 numa faixa de 0,5 a 4). Nenhum teste pegava isso —
/// digitar o valor e dar Enter sempre funcionou; só o ARRASTO estava quebrado.
///
/// O domínio sai do próprio mapeamento: `display = track·scale + offset`, com
/// `track ∈ [0,1]` → `display ∈ [offset, offset + scale]`.
#[allow(clippy::too_many_arguments)] // slider + chip + mapeamento + range: um registro só
fn slider_chip(
    store: &mut WidgetStore,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    track: f32,
    display: f64,
    scale: f32,
    offset: f32,
    step: f64,
) {
    store.register(
        slider,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: track,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        chip,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: display,
            buffer: format!("{display}"),
            caret: 0,
            last_committed: display,
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(slider, chip, scale, offset);
    store.set_number_range(
        chip,
        f64::from(offset),
        f64::from(offset) + f64::from(scale),
        step,
    );
}

/// Registra os widgets fixos do painel Flip.
pub fn populate(store: &mut WidgetStore) {
    // Mode row (Select / Draw / Erase / Fill / Sculpt).
    button(store, ids::FLIP_MODE_SELECT);
    button(store, ids::FLIP_MODE_DRAW);
    button(store, ids::FLIP_MODE_ERASE);
    button(store, ids::FLIP_MODE_RESHAPE);
    button(store, ids::FLIP_MODE_EDIT);
    // Edit section (W6) — pintados só no modo Edit, registrados SEMPRE (mesma regra dos
    // botões de borracha logo abaixo: registrar é o que os torna focáveis, e o gate
    // `architecture_panel_wiring_parity` cobra a paridade paint↔register).
    button(store, ids::FLIP_EDIT_SELECT_ALL);
    button(store, ids::FLIP_EDIT_DESELECT);
    button(store, ids::FLIP_EDIT_DELETE);
    // O domínio da seleção (W8 + §4.B): traço, ponto ou pedaço-entre-cruzamentos.
    button(store, ids::FLIP_EDIT_DOM_STROKE);
    button(store, ids::FLIP_EDIT_DOM_POINT);
    button(store, ids::FLIP_EDIT_DOM_SEGMENT);
    // Shape (Draw): o traço carrega o próprio preenchimento?
    button(store, ids::FLIP_SHAPE_LINE);
    button(store, ids::FLIP_SHAPE_FILLED);

    // Brush sliders — seeded at the tool defaults.
    slider_chip(
        store,
        ids::FLIP_SIZE,
        ids::FLIP_SIZE_NUM,
        px_to_slider(DEFAULT_WIDTH_PX),
        DEFAULT_WIDTH_PX,
        WIDTH_SLIDER_SCALE,
        WIDTH_SLIDER_OFFSET,
        1.0, // step do dominio: unidades inteiras (px / %)
    );
    slider_chip(
        store,
        ids::FLIP_HARDNESS,
        ids::FLIP_HARDNESS_NUM,
        DEFAULT_HARDNESS,
        f64::from(DEFAULT_HARDNESS),
        1.0, // track 0..1 → 0..1 (identity)
        0.0,
        0.01, // LITERAL-PX-OK: passo do dominio (fracao 0..1), nao metrica de design
    );
    slider_chip(
        store,
        ids::FLIP_OPACITY,
        ids::FLIP_OPACITY_NUM,
        DEFAULT_OPACITY,
        f64::from(DEFAULT_OPACITY) * 100.0, // LITERAL-PX-OK: fraction→percent display
        OPACITY_SLIDER_SCALE,
        0.0,
        1.0, // step do dominio: unidades inteiras (px / %)
    );
    slider_chip(
        store,
        ids::FLIP_SMOOTHING,
        ids::FLIP_SMOOTHING_NUM,
        DEFAULT_SMOOTHING,
        f64::from(DEFAULT_SMOOTHING),
        1.0,
        0.0,
        0.01, // LITERAL-PX-OK: passo do dominio (fracao 0..1), nao metrica de design
    );

    // Fill (W4): modo, sub-modos do balde e os 3 sliders. Registrados sempre (só
    // PINTADOS no modo Fill) — um widget não-registrado não pode ser clicado.
    button(store, ids::FLIP_MODE_FILL);
    button(store, ids::FLIP_FILL_PAINT);
    button(store, ids::FLIP_FILL_BEHIND);
    button(store, ids::FLIP_FILL_UNPAINT);
    // Colorize (C2): modo + Apply/Clear. Registrados sempre, pintados só no modo
    // Colorize; a swatch usa o dispatch de picker (register_picker_swatch no paint).
    button(store, ids::FLIP_MODE_COLORIZE);
    button(store, ids::FLIP_COLORIZE_APPLY);
    button(store, ids::FLIP_COLORIZE_CLEAR);
    // Trace (Shift & Trace): modo + Reset. Registrados sempre, pintados só no modo Trace.
    button(store, ids::FLIP_MODE_TRACE);
    button(store, ids::FLIP_TRACE_RESET);
    // Bleed (6º smoke): quão fundo a cor entra pelo vão aberto (a lente). `0.5` = o pedágio
    // aprovado no 5º smoke. É o controle CONTÍNUO do vazamento; o Trap (reusado no Colorize)
    // é o selo BINÁRIO. Faixa 0..1 (fração), step contínuo.
    slider_chip(
        store,
        ids::FLIP_COLORIZE_BLEED,
        ids::FLIP_COLORIZE_BLEED_NUM,
        0.5,   // track (= a fração `colorize_bleed`); meio = DEFAULT_SQUEEZE
        50.0,  // LITERAL-PX-OK: display inicial em % do dominio, nao metrica de design
        100.0, // LITERAL-PX-OK: escala fracao 0..1 -> 0..100 %, nao metrica de design
        0.0,
        1.0, // step: % inteiro
    );
    slider_chip(
        store,
        ids::FLIP_GAP,
        ids::FLIP_GAP_NUM,
        0.0,
        0.0,
        GAP_MAX_PX as f32,
        0.0,
        1.0, // step do dominio: unidades inteiras (px / %)
    );
    slider_chip(
        store,
        ids::FLIP_GROW,
        ids::FLIP_GROW_NUM,
        // O default (0 px) na faixa [-8, +8]. Com a âncora no EIXO da linha (BUGS #14)
        // o zero já é exato em qualquer espessura e zoom — o Grow é só o ajuste
        // estilístico (off-register / vão deliberado).
        ((0.0 - GROW_MIN) / (GROW_MAX - GROW_MIN)) as f32,
        0.0,
        (GROW_MAX - GROW_MIN) as f32,
        GROW_MIN as f32,
        1.0, // step do dominio: unidades inteiras (px / %)
    );
    slider_chip(
        store,
        ids::FLIP_TRAP,
        ids::FLIP_TRAP_NUM,
        0.0, // default 0 = desligado (a bola e opt-in)
        0.0,
        TRAP_MAX_PX as f32,
        0.0,
        1.0, // step do dominio: unidades inteiras (px)
    );
    slider_chip(
        store,
        ids::FLIP_PRECISION,
        ids::FLIP_PRECISION_NUM,
        ((DEFAULT_PRECISION - PRECISION_MIN) / (PRECISION_MAX - PRECISION_MIN)) as f32,
        DEFAULT_PRECISION,
        (PRECISION_MAX - PRECISION_MIN) as f32,
        PRECISION_MIN as f32,
        0.1, // LITERAL-PX-OK: passo do dominio (Precision 0,5..4), nao metrica de design
    );

    // Erase sub-mode buttons (painted only in Erase mode, registered always).
    button(store, ids::FLIP_ERASE_SOFT);
    button(store, ids::FLIP_ERASE_HARD);
    button(store, ids::FLIP_ERASE_STROKE);

    // §4.C — os LINKS da borracha (Unified Paint Settings do Blender) + os sliders
    // PRÓPRIOS dela. Os toggles são pintados só no modo Erase; os sliders, só no modo
    // Erase E com o link desligado. Registrados SEMPRE (registrar é o que os torna
    // focáveis/clicáveis — a mesma regra dos botões de borracha acima).
    //
    // Os próprios nascem nos MESMOS defaults do pincel: deslinkar não pode fazer o
    // número saltar na cara do artista da primeira vez.
    button(store, ids::FLIP_LINK_SIZE);
    button(store, ids::FLIP_LINK_STRENGTH);
    slider_chip(
        store,
        ids::FLIP_ERASE_SIZE,
        ids::FLIP_ERASE_SIZE_NUM,
        px_to_slider(DEFAULT_WIDTH_PX),
        DEFAULT_WIDTH_PX,
        WIDTH_SLIDER_SCALE,
        WIDTH_SLIDER_OFFSET,
        1.0, // step do dominio: unidades inteiras (px / %)
    );
    slider_chip(
        store,
        ids::FLIP_ERASE_STRENGTH,
        ids::FLIP_ERASE_STRENGTH_NUM,
        DEFAULT_OPACITY,
        f64::from(DEFAULT_OPACITY) * 100.0, // LITERAL-PX-OK: fraction→percent display
        OPACITY_SLIDER_SCALE,
        0.0,
        1.0, // step do dominio: unidades inteiras (px / %)
    );

    // Os oito pincéis de escultura (W5; pintados só no modo Reshape, registrados
    // sempre — como os de cima). Um botão pintado e NÃO registrado aqui é focável
    // por ninguém e o clique dele é dropado em silêncio.
    for id in ids::FLIP_RESHAPE_KIND_IDS {
        button(store, id);
    }

    // Layers toolbar.
    button(store, ids::FLIP_LAYER_ADD);
    button(store, ids::FLIP_LAYER_DUPLICATE);
    button(store, ids::FLIP_LAYER_DELETE);

    // Close (X) chrome button.
    button(store, ids::FLIP_CLOSE);
}
