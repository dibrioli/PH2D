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
    DEFAULT_TEXT_LINE_HEIGHT, DEFAULT_TEXT_SIZE, DEFAULT_TEXT_TRACKING, DEFAULT_TEXT_WEIGHT,
    DEFAULT_TEXT_WRAP, TEXT_LINE_HEIGHT_SLIDER_OFFSET, TEXT_LINE_HEIGHT_SLIDER_SCALE,
    TEXT_SIZE_SLIDER_OFFSET, TEXT_SIZE_SLIDER_SCALE, TEXT_TRACKING_SLIDER_OFFSET,
    TEXT_TRACKING_SLIDER_SCALE, TEXT_WEIGHT_SLIDER_OFFSET, TEXT_WEIGHT_SLIDER_SCALE,
    TEXT_WRAP_SLIDER_OFFSET, TEXT_WRAP_SLIDER_SCALE, WIDTH_SLIDER_OFFSET, WIDTH_SLIDER_SCALE,
    text_line_height_to_slider, text_size_to_slider, text_tracking_to_slider,
    text_weight_to_slider, text_wrap_to_slider,
};
use ph2d_tool_vector::shapes;
use ph2d_tool_vector::{DEFAULT_STROKE_WIDTH_PX, px_to_slider};

/// Linear-gradient Angle slider mapping: track `0..1` → `0..360` degrees.
const GRAD_ANGLE_SLIDER_SCALE: f32 = 360.0; // LITERAL-PX-OK: degrees in a full turn (math constant)
/// Multi-point Influence slider mapping: track `0..1` → `0..4` (IDW strength).
const GRAD_INFLUENCE_SLIDER_SCALE: f32 = 4.0; // LITERAL-PX-OK: max IDW influence (domain constant)
/// Multi-point Jitter slider mapping: track `0..1` → `0..1` (per-texel grain).
const GRAD_JITTER_SLIDER_SCALE: f32 = 1.0; // LITERAL-PX-OK: jitter is already a 0..1 fraction

/// Os pills de modo (Select … Fillet / Chamfer) + Convert — módulo irmão pelo teto de LOC.
#[path = "populate_modes.rs"]
mod modes;

/// Os dois knobs do LÁPIS — irmão pelo teto de LOC (600) do painel.
#[path = "populate_pencil.rs"]
mod pencil;

/// Os controles da seção CONSTRAINTS (plano UI/UX W3) — irmão pelo mesmo teto.
#[path = "populate_anchors.rs"]
pub(crate) mod anchors;
/// Os quatro verbos da seção COMPONENT (plano UI/UX W5) — irmão pelo mesmo teto.
#[path = "populate_components.rs"]
mod components;
/// Os controles da seção FRAME (plano UI/UX W0) — irmão pelo teto de LOC.
#[path = "populate_frame.rs"]
mod frame;
/// Os controles da seção LAYOUT (plano UI/UX W2, ADR-0153) — irmão pelo mesmo teto.
#[path = "populate_layout.rs"]
pub(crate) mod layout;
/// Os controles da SIMETRIA de desenho — irmão pelo teto de LOC (600) do painel.
#[path = "populate_symmetry.rs"]
mod symmetry;
/// Os dois chips de TOKEN (plano UI/UX W4) — irmão pelo teto de LOC.
#[path = "populate_tokens.rs"]
mod tokens;

/// **As OPERAÇÕES** — vértice, topologia (as três da W4 + o corte), booleana, regra de
/// preenchimento, o ímã, o tipo de fill e o alinhamento. Irmão pelo teto de 600 LOC, e o corte é
/// por RESPONSABILIDADE: lá ficam os comandos que AGEM sobre a seleção, aqui a forma e a moldura
/// dela (catálogo, arrange, transform, seções).
/// O registro dos widgets do **Blend / Morph** — módulo irmão (teto de 600 LOC), par do
/// `paint_blend` que os pinta.
#[path = "populate_blend.rs"]
mod blend;

/// O registro dos widgets do **Envelope** — módulo irmão (teto de 600 LOC), par do
/// `paint_envelope` que os pinta.
#[path = "populate_envelope.rs"]
mod envelope;

/// ⭐ O registro dos widgets do **ESQUELETO** — módulo irmão, par do `paint_bone`.
#[path = "populate_bone.rs"]
mod bone;

/// O registro dos widgets dos **Effects** (ADR-0132) — módulo irmão, par do `paint_effects`.
#[path = "populate_effects.rs"]
mod effects;

/// O registro dos widgets do **Pattern on Path** (plano 23) — irmão pelo teto de 600 LOC.
#[path = "populate_patternpath.rs"]
mod patternpath;
#[path = "populate_texture_pattern.rs"]
mod texture_pattern;

/// O registro dos widgets do **Contour** (pesquisa `20_*` #9) — irmão pelo teto de 600 LOC.
#[path = "populate_contour.rs"]
mod contour;

/// O registro dos widgets do **Filters** (FX raster, plano 24) — irmão pelo teto de 600 LOC.
#[path = "populate_filters.rs"]
mod filters;

/// O registro dos widgets de **ESTILO** (traço, tracejado, pontas, preenchimento) — módulo
/// irmão pelo teto de 600 LOC deste arquivo.
#[path = "populate_style.rs"]
mod style;

#[path = "populate_ops.rs"]
mod ops;

/// O re-registro POR FRAME da faixa de cada parâmetro de efeito — só no frame se sabe que
/// efeito ocupa cada linha. Re-exportado daqui porque o `paint` o chama pelo caminho do módulo.
pub(crate) use effects::seed_effect_ranges;

/// Register a plain action Button in the Normal state.
pub(crate) fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
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
pub(crate) fn number_field(
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

/// ⭐⭐⭐ **Um campo numérico de COORDENADA DE MUNDO — sem faixa, de propósito.**
///
/// ⚠️ **É uma grandeza diferente da de um `number_field` com faixa**, e a diferença é a lei desta
/// casa: uma largura de traço tem um tecto (o recurso é a caneta), e uma **coordenada de mundo não
/// tem nenhum** — ela abrange qualquer magnitude, e um clamp ali torna posições legítimas
/// inalcançáveis por digitação. O arrasto é calibrado ao vivo pelo `vector_bridge`
/// (`set_number_drag_rate`), então o campo é 1:1 com o ecrã em qualquer zoom.
///
/// ⛔ Ela existe porque a lei estava escrita num COMENTÁRIO dentro de um laço, e a wave seguinte
/// (o deslocamento de uma camada, v21) registou o campo dela com a faixa da **largura do traço** —
/// um tecto emprestado de outro recurso, que é o defeito que o `CLAUDE.md` §0.0 nomeia. *Uma lei em
/// comentário não é uma lei; só uma PORTA é.*
pub(crate) fn world_number_field(store: &mut WidgetStore, id: ph2d_a11y::NodeId, value: f64) {
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
    // ⛔ SEM `set_number_range` — ver o doc acima.
}

/// Register a slider + its linked value chip, seeded at `track` / `display`.
pub(crate) fn slider_chip(
    store: &mut WidgetStore,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    track: f32,
    display: f64,
    scale: f32,
    offset: f32,
) {
    slider_chip_inner(store, slider, chip, track, display, scale, offset, false);
}

/// Como [`slider_chip`], mas o valor digitado é **arredondado ao inteiro** antes de voltar ao
/// slider — para o chip cuja unidade pintada é uma CONTAGEM (o Steps do blend). Sem isto, digitar
/// "3.5" deixa o chip em 3,5 enquanto o painel mostra "4", e o Tab revela a inconsistência.
///
/// Os dois delegam ao mesmo `_inner`, que é a forma que o próprio
/// `WidgetStore::link_slider_number_mapped{,_integer}` usa — a decisão "inteiro?" mora num
/// parâmetro, não num 2º construtor que pode divergir do 1º.
pub(super) fn slider_chip_int(
    store: &mut WidgetStore,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    track: f32,
    display: f64,
    scale: f32,
    offset: f32,
) {
    slider_chip_inner(store, slider, chip, track, display, scale, offset, true);
}

#[allow(clippy::too_many_arguments)]
fn slider_chip_inner(
    store: &mut WidgetStore,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    track: f32,
    display: f64,
    scale: f32,
    offset: f32,
    integer: bool,
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
    if integer {
        store.link_slider_number_mapped_integer(slider, chip, scale, offset);
    } else {
        store.link_slider_number_mapped(slider, chip, scale, offset);
    }
}

/// Registra os widgets do painel Vector. Delegado a helpers por seção — o corpo
/// plano estourava o teto de 200 LOC por função dos painéis.
pub fn populate(store: &mut WidgetStore) {
    populate_sections(store);
    populate_shape(store);
    ops::populate_ops(store);
    style::populate_style(store);
    populate_arrange(store);
    populate_connector(store);
    // ⭐ AS SETAS do Morph (plano 32 W4) — o pool inteiro. ⛔ Aqui e não no módulo que as pinta:
    // a gate `architecture_panel_wiring_parity` exige a chamada de registro dentro do
    // `populate.rs`, e ela tem razão (um widget pintado e não registrado nasce morto).
    modes::populate_morph_arrows(store);
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

    // **Os dois knobs do LÁPIS.** Sem o `register` o slider fica pintado e MORTO sob o ponteiro
    // (a checagem de focabilidade mora no store) — a falha que as 36 células da matriz de física
    // e os dez chips do Painter já pagaram.
    pencil::pencil_knobs(store);
    symmetry::symmetry_controls(store);
    frame::frame_controls(store);
    layout::layout_controls(store);
    anchors::anchor_controls(store);
    components::component_controls(store);
    tokens::token_controls(store);

    // Os pills de MODO (Select … Fillet / Chamfer) + Convert — módulo irmão pelo teto de LOC.
    modes::mode_buttons(store);

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
    // **Width: Auto | Fixed** + a largura. Registados SEMPRE, inclusive o slider que só é
    // pintado no modo Fixed: o `populate` corre uma vez na instalação do painel, e registar "o
    // que está visível agora" deixaria o slider morto sob o rato no primeiro clique em Fixed.
    button(store, ids::VECTOR_TEXT_WRAP_AUTO);
    button(store, ids::VECTOR_TEXT_WRAP_FIXED);
    slider_chip(
        store,
        ids::VECTOR_TEXT_WRAP_W,
        ids::VECTOR_TEXT_WRAP_W_NUM,
        text_wrap_to_slider(DEFAULT_TEXT_WRAP),
        DEFAULT_TEXT_WRAP,
        TEXT_WRAP_SLIDER_SCALE,
        TEXT_WRAP_SLIDER_OFFSET,
    );
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

/// Arrange (duplicate / z-order / flip / rotate), reshape de path, e o botão Close.
fn populate_arrange(store: &mut WidgetStore) {
    // Arrange: Duplicate + z-order restack + Flip buttons (act on the selected path).
    button(store, ids::VECTOR_ARRANGE_DUPLICATE);
    // **O Z-INDEX global.** ⚠️ **Sem `set_number_range`**, e é a decisão dos campos do Transform
    // pelo mesmo motivo: o teto real é o do `ZIndexOverride::Z_MAX`, aplicado na PORTA de escrita
    // da shell (onde a regra é do documento), e um clamp no widget seria a segunda resposta —
    // divergindo no dia em que o componente mudasse de faixa.
    store.register(
        ids::VECTOR_ARRANGE_Z,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.0,
            buffer: String::from("0"),
            caret: 0,
            last_committed: 0.0,
            selection_anchor: None,
        },
    );
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
        // As duas do NÓ: mesmo widget, mesma rota, mesma vida sob o mouse. Elas entram nesta
        // lista e não numa nova porque a pergunta é a mesma — *este id é um campo numérico que a
        // shell possui?* —, e uma segunda lista é como a terceira nasce sem o `register`.
        ids::VECTOR_VERT_X,
        ids::VECTOR_VERT_Y,
    ] {
        // A lei (campo SEM faixa) vive na porta, e não neste laço — ver [`world_number_field`].
        world_number_field(store, id, 0.0);
    }
    // **Resize Box** (plano UI/UX W3b) — sem este registo o checkbox ficaria pintado, com
    // hit-rect, e MORTO sob o rato: a checagem de focabilidade mora no store. É o defeito que
    // este painel já pagou cinco vezes (os pills de modo, o Cut, a simetria, o layout, as
    // âncoras), e o seam é o que o prova.
    store.register(
        ids::VECTOR_TRANSFORM_RESIZE_BOX,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // ⭐ **Stroke** (plano 34) — a caixa que diz se ESTA forma tem traço. Mesmo registo, e pela
    // mesma razão que o irmão acima: sem ele a caixa fica pintada, com hit-rect, e MORTA sob o
    // rato — a checagem de focabilidade mora no store.
    store.register(
        ids::VECTOR_STROKE_PRESENT,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    populate_appearance(store);
}

/// ⭐ **O registo da secção APPEARANCE** — módulo irmão pelo tecto de LOC do painel.
#[path = "populate_appearance.rs"]
mod appearance;
pub(crate) use appearance::populate_appearance;
