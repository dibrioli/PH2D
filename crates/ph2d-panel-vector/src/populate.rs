//! Vector Style panel widget registration (called once at panel install).
//!
//! Registers the Width slider (seeded at the tool's default so the knob renders
//! correctly before the first drag — the slider is absolute-position, so the
//! initial store value drives both the render AND the drag baseline), its px
//! chip (linked via the shared affine mapping), the draw-mode buttons
//! (Pen / Rectangle / Ellipse / Polygon), the Polygon Sides slider + chip, the
//! three Boolean buttons, the Fill "None" button, and the Close (X) button. The
//! two colour swatches need NO store entry — their Down is handled by the
//! generic `is_picker_swatch` dispatch (pointer.rs), which short-circuits before
//! the normal widget-event path.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    ButtonState, DropdownState, SliderOrientation, SliderState, TextInputState,
};
use ph2d_tool_vector::params::{
    DASH_SLIDER_OFFSET, DASH_SLIDER_SCALE, DEFAULT_TEXT_LINE_HEIGHT, DEFAULT_TEXT_SIZE,
    DEFAULT_TEXT_TRACKING, DEFAULT_TEXT_WEIGHT, GAP_DEFAULT, GAP_SLIDER_OFFSET, GAP_SLIDER_SCALE,
    OPACITY_SLIDER_OFFSET, OPACITY_SLIDER_SCALE, TEXT_LINE_HEIGHT_SLIDER_OFFSET,
    TEXT_LINE_HEIGHT_SLIDER_SCALE, TEXT_SIZE_SLIDER_OFFSET, TEXT_SIZE_SLIDER_SCALE,
    TEXT_TRACKING_SLIDER_OFFSET, TEXT_TRACKING_SLIDER_SCALE, TEXT_WEIGHT_SLIDER_OFFSET,
    TEXT_WEIGHT_SLIDER_SCALE, WIDTH_SLIDER_OFFSET, WIDTH_SLIDER_SCALE, gap_to_slider,
    text_line_height_to_slider, text_size_to_slider, text_tracking_to_slider,
    text_weight_to_slider,
};
use ph2d_tool_vector::shapes;
use ph2d_tool_vector::{DEFAULT_STROKE_WIDTH_PX, params, px_to_slider};

/// Linear-gradient Angle slider mapping: track `0..1` → `0..360` degrees.
const GRAD_ANGLE_SLIDER_SCALE: f32 = 360.0; // LITERAL-PX-OK: degrees in a full turn (math constant)
/// Multi-point Influence slider mapping: track `0..1` → `0..4` (IDW strength).
const GRAD_INFLUENCE_SLIDER_SCALE: f32 = 4.0; // LITERAL-PX-OK: max IDW influence (domain constant)
/// Multi-point Jitter slider mapping: track `0..1` → `0..1` (per-texel grain).
const GRAD_JITTER_SLIDER_SCALE: f32 = 1.0; // LITERAL-PX-OK: jitter is already a 0..1 fraction

/// Register a plain action Button in the Normal state.
fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// Uma caixa numérica de faixa FIXA (as do conector + as das pontas do traço), semeada em
/// `value`. **`set_number_range` não é opcional**: sem ela o arrasto escala errado — o
/// gotcha conhecido da caixa limitada. (As caixas de parâmetro de FORMA não passam aqui: a
/// faixa delas é por-forma, e a shell a re-registra quando o foco muda.)
fn number_field(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    min: f64,
    max: f64,
    step: f64,
    value: f64,
) {
    store.register(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer: format!("{value}"),
            caret: 0,
            last_committed: value,
            selection_anchor: None,
        },
    );
    store.set_number_range(id, min, max, step);
}

/// Register a slider + its linked value chip, seeded at `track` / `display`.
fn slider_chip(
    store: &mut WidgetStore,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    track: f32,
    display: f64,
    scale: f32,
    offset: f32,
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
}

/// Registra os widgets do painel Vector. Delegado a helpers por seção — o corpo
/// plano estourava o teto de 200 LOC por função dos painéis.
pub fn populate(store: &mut WidgetStore) {
    populate_sections(store);
    populate_shape(store);
    populate_ops(store);
    populate_style(store);
    populate_arrange(store);
    populate_connector(store);
}

/// Os três campos do **CONECTOR** (Route / Jetty / Spread).
///
/// Ficam AQUI, e não no módulo que os pinta, porque a gate `architecture_panel_wiring_parity`
/// exige a chamada de registro dentro do `populate.rs` — e ela tem razão: um widget pintado e
/// hit-testado mas NÃO registrado nunca vira focável, então Down/Up jamais disparam e o
/// controle nasce morto (a classe de bug dos pills do vetor). A gate pegou exatamente isto
/// quando o registro morava no módulo irmão.
fn populate_connector(store: &mut WidgetStore) {
    // Route: um BOTÃO que CICLA (a rota corrente vem do snapshot que a shell publica, não do
    // store — a verdade é do documento, e o painel é stateless).
    button(store, ids::VECTOR_CONNECTOR_ROUTE);
    // Jetty / Spread / Corner: caixas numéricas de faixa FIXA (ao contrário dos campos de
    // forma, que mudam com a forma em foco). O valor é re-semeado com o EFETIVO a cada frame
    // (Fase B do paint) — aqui só nasce o slot.
    // A tabela é ÚNICA (`paint_connector::NUMBER_FIELDS`): quem registra e quem desenha iteram a
    // MESMA lista. A alternativa já falhou — o campo Curve nasceu com id, desenho e evento, e
    // SEM a linha de registro aqui: ele pintava, aceitava o arrasto e não despachava nada. Um
    // controle morto, com a suíte inteira verde.
    for &(id, field) in crate::paint_connector::NUMBER_FIELDS {
        number_field(store, id, field.min, field.max, field.step, field.min);
    }
}

/// **Os cabeçalhos colapsáveis.** O collapse é dispatch GENÉRICO e exige DOIS sites: a
/// marca aqui (`mark_collapsible_section`) e o hit-rect do header no paint. Sem a marca,
/// `apply_click` nunca chama `toggle_collapsed` e o chevron vira um enfeite morto — a
/// seção pinta a promessa de dobrar e não dobra.
///
/// A lista é `ids::VECTOR_SECTIONS` (editor-core): uma seção nova entra lá e já nasce
/// colapsável, sem tocar aqui.
fn populate_sections(store: &mut WidgetStore) {
    for &id in ids::VECTOR_SECTIONS {
        store.mark_collapsible_section(id);
    }
}

/// Width + os modos + o CATÁLOGO de formas (botões + campos genéricos).
fn populate_shape(store: &mut WidgetStore) {
    // Width slider — seeded at the tool's default (`px_to_slider(3px)`).
    store.register(
        ids::VECTOR_WIDTH,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: px_to_slider(DEFAULT_STROKE_WIDTH_PX),
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::VECTOR_WIDTH_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: DEFAULT_STROKE_WIDTH_PX,
            buffer: format!("{}", DEFAULT_STROKE_WIDTH_PX as i64),
            caret: 0,
            last_committed: DEFAULT_STROKE_WIDTH_PX,
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(
        ids::VECTOR_WIDTH,
        ids::VECTOR_WIDTH_NUM,
        WIDTH_SLIDER_SCALE,
        WIDTH_SLIDER_OFFSET,
    );

    // Modos (Select / Node / Pen / Shape / Text) + Convert. O pill **Shape** é o 5º: sem
    // ele, escolher uma forma punha a tool em `DrawMode::Shape` e a fileira de modos
    // ficava TODA apagada — o usuário não via em que modo estava.
    button(store, ids::VECTOR_CONVERT_TO_CURVES);
    button(store, ids::VECTOR_MODE_SELECT);
    button(store, ids::VECTOR_MODE_NODE);
    button(store, ids::VECTOR_MODE_PEN);
    button(store, ids::VECTOR_MODE_SHAPE);
    button(store, ids::VECTOR_MODE_TEXT);
    // O 6º: **Connect** (a linha que gruda em duas formas). Registrar aqui é o que o
    // torna clicável — pintar e dar hit-rect não basta.
    button(store, ids::VECTOR_MODE_CONNECT);

    // O 7º: **Build** (o Shape Builder). Mesma armadilha: registrar aqui é o que o torna
    // clicável — pintar e dar hit-rect não basta.
    button(store, ids::VECTOR_MODE_BUILD);

    // **Pick Shapes** (Blend). É um modo de tool, mas o botão dele mora na seção BLEND (não nesta
    // fileira) — registrar aqui é o que o torna clicável onde quer que seja pintado.
    button(store, ids::VECTOR_MODE_PICKBLEND);

    // O CATÁLOGO: um botão por forma + uma opção de dropdown por família. Registrados por
    // ÍNDICE — uma forma nova entra na tabela e já nasce clicável, sem tocar aqui.
    for i in 0..shapes::SHAPES.len() {
        button(store, ids::vector_shape_id(i));
    }
    for i in 0..shapes::ALL_GROUPS.len() {
        button(store, ids::vector_shape_group_id(i));
    }
    // O chip de CATEGORIA (um `Dropdown`): abrir / fechar / roda vêm de graça do dispatch
    // genérico. As OPÇÕES do popover são os botões de família acima.
    store.register_if_absent(
        ids::VECTOR_SHAPE_GROUP_DD,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );

    // Os campos de parâmetro (`MAX_SHAPE_FIELD_SLOTS`): caixas numéricas genéricas. A
    // FAIXA de cada uma depende da forma em foco, então a shell a re-registra quando o
    // foco muda (`set_number_range`) — aqui só existem os slots.
    for i in 0..ids::MAX_SHAPE_FIELD_SLOTS {
        store.register(
            ids::vector_shape_field_id(i),
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::from("0"),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        // O gêmeo BOTÃO do mesmo slot: quando o campo é uma ESCOLHA (`FieldUnit::Choice`),
        // é ele que fica clicável e cicla pelas opções — o slot numérico vira só o depósito
        // do valor. `populate` é estático (não sabe qual forma está em foco), então os dois
        // existem sempre e a PINTURA decide qual dos dois registra o hit.
        store.register(
            ids::vector_shape_choice_id(i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // Text "Size" slider (world units) — shown only in Text mode; seeded at the
    // default glyph size. The shell drain maps the track back to a size.
    slider_chip(
        store,
        ids::VECTOR_TEXT_SIZE,
        ids::VECTOR_TEXT_SIZE_NUM,
        text_size_to_slider(DEFAULT_TEXT_SIZE),
        DEFAULT_TEXT_SIZE,
        TEXT_SIZE_SLIDER_SCALE,
        TEXT_SIZE_SLIDER_OFFSET,
    );
    // Text "Weight" slider (`wght` 100..900) — shown only in Text mode; seeded at the
    // default weight (Regular 400).
    slider_chip(
        store,
        ids::VECTOR_TEXT_WEIGHT,
        ids::VECTOR_TEXT_WEIGHT_NUM,
        text_weight_to_slider(DEFAULT_TEXT_WEIGHT),
        DEFAULT_TEXT_WEIGHT,
        TEXT_WEIGHT_SLIDER_SCALE,
        TEXT_WEIGHT_SLIDER_OFFSET,
    );
    // Font-family picker prev / next (`<` / `>`) — cycle the chosen system font.
    button(store, ids::VECTOR_TEXT_FONT_PREV);
    button(store, ids::VECTOR_TEXT_FONT_NEXT);
    button(store, ids::VECTOR_TEXT_FONT_IMPORT);
    // Font dropdown chip (between the arrows): a `Dropdown` so the generic
    // open/close dispatch toggles its popover (the styled family list).
    store.register_if_absent(
        ids::VECTOR_TEXT_FONT_DD,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    // Paragraph: alignment L / C / R (buttons) + Line-height slider + Tracking slider.
    button(store, ids::VECTOR_TEXT_ALIGN_LEFT);
    button(store, ids::VECTOR_TEXT_ALIGN_CENTER);
    button(store, ids::VECTOR_TEXT_ALIGN_RIGHT);
    slider_chip(
        store,
        ids::VECTOR_TEXT_LINE_HEIGHT,
        ids::VECTOR_TEXT_LINE_HEIGHT_NUM,
        text_line_height_to_slider(DEFAULT_TEXT_LINE_HEIGHT),
        DEFAULT_TEXT_LINE_HEIGHT,
        TEXT_LINE_HEIGHT_SLIDER_SCALE,
        TEXT_LINE_HEIGHT_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_TEXT_TRACKING,
        ids::VECTOR_TEXT_TRACKING_NUM,
        text_tracking_to_slider(DEFAULT_TEXT_TRACKING),
        DEFAULT_TEXT_TRACKING,
        TEXT_TRACKING_SLIDER_SCALE,
        TEXT_TRACKING_SLIDER_OFFSET,
    );
    // Variation-axis number fields (one per non-wght axis the current font exposes).
    // Value + range are seeded per-frame from the published axes (paint Phase B), so
    // the fixed slots adapt to whatever axes the current font has.
    for i in 0..ids::MAX_TEXT_VARIATION_AXES {
        store.register(
            ids::vector_text_axis_id(i),
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::from("0"),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }

    populate_transform_fields(store);
}

/// Vértice, Boolean + Compound, Fill Rule, Snap, tipo de Fill, Align/Distribute.
/// O registro dos widgets do **Blend / Morph** — módulo irmão (teto de 600 LOC), par do
/// `paint_blend` que os pinta.
#[path = "populate_blend.rs"]
mod blend;

fn populate_ops(store: &mut WidgetStore) {
    // Vertex-type buttons (retype the selected vertex; shown only when a vertex
    // is selected, but registered unconditionally — the store is mode-agnostic)
    // + the Delete-node button.
    button(store, ids::VECTOR_VERT_CORNER);
    button(store, ids::VECTOR_VERT_SMOOTH);
    button(store, ids::VECTOR_VERT_SYMMETRIC);
    button(store, ids::VECTOR_VERT_DELETE);

    // Boolean op buttons (N-ary over the SELECTED closed regions) + compound row.
    blend::populate_blend(store);
    button(store, ids::VECTOR_BOOL_UNION);
    button(store, ids::VECTOR_BOOL_SUBTRACT);
    button(store, ids::VECTOR_BOOL_INTERSECT);
    button(store, ids::VECTOR_BOOL_EXCLUDE);
    button(store, ids::VECTOR_COMPOUND_MAKE);
    button(store, ids::VECTOR_COMPOUND_RELEASE);
    // Fill rule (compound paths only — the row hides for a single contour).
    button(store, ids::VECTOR_FILL_RULE_NONZERO);
    button(store, ids::VECTOR_FILL_RULE_EVENODD);
    // Shape snapping (tool setting, always visible). The grid toggle is in the
    // editor's universal Grid Snap panel.
    button(store, ids::VECTOR_SNAP_OFF);
    button(store, ids::VECTOR_SNAP_ON);

    // Fill-type selector (Solid / Linear / Radial) — act on the selected path.
    button(store, ids::VECTOR_FILL_KIND_SOLID);
    button(store, ids::VECTOR_FILL_KIND_LINEAR);
    button(store, ids::VECTOR_FILL_KIND_RADIAL);
    button(store, ids::VECTOR_FILL_KIND_MULTI);
    button(store, ids::VECTOR_GRAD_ADD_POINT);
    button(store, ids::VECTOR_GRAD_REMOVE_POINT);
    button(store, ids::VECTOR_GRAD_ADD_STOP);
    button(store, ids::VECTOR_GRAD_REMOVE_STOP);
    // Align + Distribute (multi-path object selection).
    button(store, ids::VECTOR_PIVOT_EDIT);
    button(store, ids::VECTOR_ALIGN_LEFT);
    button(store, ids::VECTOR_ALIGN_HCENTER);
    button(store, ids::VECTOR_ALIGN_RIGHT);
    button(store, ids::VECTOR_ALIGN_TOP);
    button(store, ids::VECTOR_ALIGN_VCENTER);
    button(store, ids::VECTOR_ALIGN_BOTTOM);
    button(store, ids::VECTOR_DISTRIBUTE_H);
    button(store, ids::VECTOR_DISTRIBUTE_V);
}

/// Gradiente (Angle / Influence / Jitter), opacidades e o traço (cap/join/dash).
fn populate_style(store: &mut WidgetStore) {
    // Linear-gradient Angle slider (track 0..1 → 0..360°) + its chip.
    slider_chip(
        store,
        ids::VECTOR_GRAD_ANGLE,
        ids::VECTOR_GRAD_ANGLE_NUM,
        0.0,
        0.0,
        GRAD_ANGLE_SLIDER_SCALE,
        0.0,
    );
    // Multi-point Influence (track 0..1 → 0..4); seeded at the default 1.0.
    slider_chip(
        store,
        ids::VECTOR_GRAD_INFLUENCE,
        ids::VECTOR_GRAD_INFLUENCE_NUM,
        1.0 / GRAD_INFLUENCE_SLIDER_SCALE,
        1.0,
        GRAD_INFLUENCE_SLIDER_SCALE,
        0.0,
    );
    // Multi-point Jitter (track 0..1 → 0..1); seeded at the default 0.0 (smooth).
    slider_chip(
        store,
        ids::VECTOR_GRAD_JITTER,
        ids::VECTOR_GRAD_JITTER_NUM,
        0.0,
        0.0,
        GRAD_JITTER_SLIDER_SCALE,
        0.0,
    );

    // Stroke / Fill Opacity sliders (0..100 %) — seeded at full opacity, matching
    // the tool's default opaque stroke/fill.
    slider_chip(
        store,
        ids::VECTOR_STROKE_OPACITY,
        ids::VECTOR_STROKE_OPACITY_NUM,
        1.0,
        100.0, // LITERAL-PX-OK: initial opacity display = 100 %
        OPACITY_SLIDER_SCALE,
        OPACITY_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_FILL_OPACITY,
        ids::VECTOR_FILL_OPACITY_NUM,
        1.0,
        100.0, // LITERAL-PX-OK: initial opacity display = 100 %
        OPACITY_SLIDER_SCALE,
        OPACITY_SLIDER_OFFSET,
    );

    // Stroke Cap / Join segmented buttons + Dash length slider (0 px = solid).
    button(store, ids::VECTOR_CAP_BUTT);
    button(store, ids::VECTOR_CAP_ROUND);
    button(store, ids::VECTOR_CAP_SQUARE);
    button(store, ids::VECTOR_JOIN_MITER);
    button(store, ids::VECTOR_JOIN_ROUND);
    button(store, ids::VECTOR_JOIN_BEVEL);
    slider_chip(
        store,
        ids::VECTOR_DASH,
        ids::VECTOR_DASH_NUM,
        0.0,
        0.0,
        DASH_SLIDER_SCALE,
        DASH_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_GAP,
        ids::VECTOR_GAP_NUM,
        gap_to_slider(GAP_DEFAULT),
        GAP_DEFAULT,
        GAP_SLIDER_SCALE,
        GAP_SLIDER_OFFSET,
    );
    populate_markers(store);
}

/// As **pontas do traço**: os dois chips (`Dropdown` — abrir/fechar/roda vêm de graça do
/// dispatch genérico) + as opções do popover, uma por ponta de `ALL_MARKERS` e por slot.
///
/// Registradas por ÍNDICE, como as formas do catálogo: uma ponta nova entra em
/// `ALL_MARKERS` e já nasce clicável, sem tocar aqui. Os slots existem sempre (o
/// `populate` é estático); o PAINT decide quais registram hit.
fn populate_markers(store: &mut WidgetStore) {
    for slot in 0..ids::MARKER_SLOTS {
        store.register_if_absent(
            crate::paint_markers::marker_dd_id(slot),
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
        for i in 0..ids::MAX_MARKER_OPTIONS {
            button(store, ids::vector_marker_option_id(slot, i));
        }
    }
    // **Tamanho / arredondamento** da ponta: caixas numéricas de faixa FIXA. O
    // `set_number_range` não é opcional — sem ele o arrasto escala errado (o gotcha da caixa
    // limitada). O valor é re-semeado com o efetivo da tool a cada frame (Fase B do paint).
    for (id, field) in [
        (ids::VECTOR_MARKER_SCALE, &params::MARKER_SCALE),
        (ids::VECTOR_MARKER_ROUND, &params::MARKER_ROUND),
    ] {
        number_field(store, id, field.min, field.max, field.step, field.min);
    }
    // **Both Ends** — um botão. Registrar aqui é o que o torna clicável: pintar e dar
    // hit-rect não basta (a gate `architecture_panel_wiring_parity` exige o registro DENTRO
    // do `populate.rs`, e ela tem razão — sem `InteractiveState` o widget nunca é focável e
    // Down/Up jamais disparam).
    button(store, ids::VECTOR_MARKER_BOTH);
}

/// Arrange (duplicate / z-order / flip / rotate), reshape de path, e o botão Close.
fn populate_arrange(store: &mut WidgetStore) {
    // Arrange: Duplicate + z-order restack + Flip buttons (act on the selected path).
    button(store, ids::VECTOR_ARRANGE_DUPLICATE);
    button(store, ids::VECTOR_ARRANGE_TO_BACK);
    button(store, ids::VECTOR_ARRANGE_BACKWARD);
    button(store, ids::VECTOR_ARRANGE_FORWARD);
    button(store, ids::VECTOR_ARRANGE_TO_FRONT);
    button(store, ids::VECTOR_ARRANGE_FLIP_H);
    button(store, ids::VECTOR_ARRANGE_FLIP_V);
    button(store, ids::VECTOR_ARRANGE_ROTATE_CW);
    button(store, ids::VECTOR_ARRANGE_ROTATE_CCW);

    // Path: Smooth / Sharpen / Simplify — reshape ALL vertices of the selected path.
    button(store, ids::VECTOR_PATH_SMOOTH);
    button(store, ids::VECTOR_PATH_SHARPEN);
    button(store, ids::VECTOR_PATH_SIMPLIFY);
    button(store, ids::VECTOR_PATH_SUBDIVIDE);
    button(store, ids::VECTOR_PATH_CLOSE);

    // Close (X) button.
    button(store, ids::VECTOR_CLOSE);
}

/// Register the four Transform number fields (X/Y/W/H) — standalone (NOT slider-
/// linked); seeded from the selected path's bbox each frame, edits route a
/// document command through the shell drain.
fn populate_transform_fields(store: &mut WidgetStore) {
    for id in [
        ids::VECTOR_TRANSFORM_X,
        ids::VECTOR_TRANSFORM_Y,
        ids::VECTOR_TRANSFORM_W,
        ids::VECTOR_TRANSFORM_H,
        ids::VECTOR_TRANSFORM_R,
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::from("0"),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        // NO `set_number_range` → the field is UNBOUNDED (no clamp, world coords
        // span any magnitude). The shell's `vector_bridge` calibrates the drag
        // scrub live via `set_number_drag_rate(px_to_world)` so a chip drag is 1:1
        // with the shape's on-screen movement at any zoom.
    }
}
