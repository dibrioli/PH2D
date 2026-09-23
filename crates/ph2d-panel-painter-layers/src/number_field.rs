//! Drag-scrub numeric-field rows for the Grain / Shape param sections (Enio 2026-06-25) — the SAME
//! number box as the Inspector's Transform (the app standard): [`paint_number_input_with_buffer`] with
//! up/down steppers, click-drag scrub (vertical = precise, horizontal = fast, Shift = super-precise — the
//! foundational `number_input_drag` dispatch) and type-to-edit. X/Y pairs (Size, Offset) share one line
//! with red **X** / green **Y** axis tags like the reference; short-label per-pattern params pair
//! two-per-line, long labels go solo.
//!
//! Each box registers its `[min, max]` + `step` via `WidgetStore::set_number_range`, so the drag-scrub
//! is PROPORTIONAL to the range (a fixed drag spans `[min,max]`, no racing) + clamped, and the stepper
//! uses `step`. The live value is mirrored into the store each frame (when unfocused). The
//! committed/scrubbed REAL value forwards over `event::is_param_field`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{NumberInput, TextInputState, paint_number_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

/// ⭐⭐ **A linha que estas secções NÃO querem ver quebrar** — o par `X`/`Y` do *Size* / *Offset*.
///
/// ⚠️ Ver a spec §6-ter: é ela que diz à porta quanto a coluna do nome pode CEDER, e é da SECÇÃO e
/// não da linha — senão a coluna sai esfarrapada.
pub(crate) const SECTION_FIELDS: usize = 2;
/// Max label length (chars) for a per-pattern param to share its line with the next one.
const PAIR_MAX_LEN: usize = 7;
/// NumberInput steps registered via `set_number_range` — the stepper increment + the drag base.
/// `pub(crate)` so the Grain/Shape sections pass the right one per field.
pub(crate) const FINE_STEP: f64 = 0.01; // LITERAL-PX-OK: NumberInput step (0..1 / offset / depth / params)
pub(crate) const SIZE_STEP: f64 = 0.1; // LITERAL-PX-OK: NumberInput step (scale)
pub(crate) const ANGLE_STEP: f64 = 1.0; // LITERAL-PX-OK: NumberInput step (whole degrees)

/// Whether `id` is a Grain/Shape param NumberInput field (Angle / Offset / Size / Depth / per-pattern
/// param) — vs a main brush slider. Drives the panel's number-field `ValueChanged` route (`event.rs`),
/// which forwards the committed/scrubbed REAL value (the tool's real-value setters clamp it).
pub(crate) fn is_param_field(id: NodeId) -> bool {
    id == ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_ANGLE
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_OFFSET_X
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_SIZE_X
        || id == ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_SIZE_Y
        || ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_PARAMS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_BRUSH_STENCIL_FIELDS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_SHAPE_SLIDERS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_SHAPE_PARAMS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_SHAPE_DEPOSIT_FIELDS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_WATERCOLOR_FIELDS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_WETPAINT_FIELDS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_IMPASTO_FIELDS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_SUBSTRATE_FIELDS.contains(&id)
        || ph2d_tool_painter::ids::PAINTER_TAPER_FIELDS.contains(&id)
}

/// Format a param value: whole number when `decimals == 0` (Angle degrees), else fixed decimals (so the
/// scrub dispatch infers the fine `0.01` step from the `.` in the buffer).
fn fmt_val(v: f32, decimals: usize) -> String {
    if decimals == 0 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.decimals$}")
    }
}

/// Register (if absent) + mirror the live `value` into the store's `NumberInput` when unfocused, with a
/// `decimals`-formatted buffer (mirror of the Inspector's per-frame sync).
fn mirror_value(store: &mut WidgetStore, id: NodeId, value: f32, decimals: usize) {
    let text = fmt_val(value, decimals);
    let _ = store.register_if_absent(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: f64::from(value),
            buffer: text.clone(),
            caret: text.len(),
            last_committed: f64::from(value),
            selection_anchor: None,
        },
    );
    if store.focus_id() != Some(id)
        && let Some(InteractiveState::NumberInput {
            value: v,
            buffer,
            caret,
            last_committed,
            ..
        }) = store.get_mut(id)
    {
        *v = f64::from(value);
        buffer.clear();
        buffer.push_str(&text);
        *caret = buffer.len();
        *last_committed = f64::from(value);
    }
}

/// ⭐⭐⭐ **O ESTADO de um campo, sem pintar nada** — espelhar o valor vivo e registar a faixa.
///
/// ⛔⛔ **Ela existe porque este painel passou a usar a PORTA do app** (2026-09-15,
/// [`ph2d_editor_core::widget::paint_property_fields_row`]), e a porta pinta **a partir do store**:
/// ela recebe `&WidgetStore`, não `&mut`. ⇒ o que era um `chip` que fazia três coisas parte-se em
/// duas — *o estado de um campo* (aqui) e *o desenho de uma linha* (a porta).
///
/// ⚠️ **O corte é honesto e não foi inventado para a conversão:** a faixa
/// (`WidgetStore::set_number_range`) é o que torna o arrasto PROPORCIONAL ao intervalo, e isso é um
/// facto do campo que vale mesmo quando ninguém o está a desenhar.
pub(crate) fn arm_field(
    store: &mut WidgetStore,
    id: NodeId,
    value: f32,
    min: f32,
    max: f32,
    step: f64,
    decimals: usize,
) {
    mirror_value(store, id, value, decimals);
    store.set_number_range(id, f64::from(min), f64::from(max), step);
}

/// One number box filling `rect` (the Inspector widget): mirror the live value, register its
/// `[min, max]` range + `step` (so the drag-scrub is range-proportional + clamped and the stepper uses
/// `step` — see `WidgetStore::set_number_range`), then paint with steppers + the edit buffer + caret.
/// `pub(crate)` so the per-layer-colour opacity box (factory-id) reuses the exact app-standard widget.
#[allow(clippy::too_many_arguments)]
pub(crate) fn chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    id: NodeId,
    value: f32,
    min: f32,
    max: f32,
    step: f64,
    decimals: usize,
) {
    {
        let store = ctx.host.store_mut();
        mirror_value(store, id, value, decimals);
        store.set_number_range(id, f64::from(min), f64::from(max), step);
    }
    let (state, _v, buf, caret, anchor) = read_number_input(ctx.host.store(), id);
    let buf = buf.to_string();
    let input = NumberInput::new(id, "", f64::from(value))
        .step(step)
        .visual((state, ctx.host.store().hover_live(id)));
    paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
}

/// Label + ONE number box (Angle / Depth / a solo param). Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_num_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label_txt: &str,
    id: NodeId,
    value: f32,
    min: f32,
    max: f32,
    step: f64,
    decimals: usize,
    // ⭐⭐ **A coluna é da SECÇÃO** — ver [`ph2d_editor_core::property_row::Seccao`] e o report do
    //    dono de 2026-09-15 (*«a caixa recua quando na verdade o nome deveria criar as colunas»*).
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    arm_field(ctx.host.store_mut(), id, value, min, max, step, decimals);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    ph2d_editor_core::property_row::paint_fields_row(
        ctx.scene,
        ctx.text_system,
        theme,
        hit_index,
        store,
        x,
        content_w,
        y,
        label_txt,
        &[id],
        step,
        None,
        sec,
    )
}

/// Label + TWO number boxes on one line, with red **X** / green **Y** axis tags (Size / Offset), like the
/// Inspector Transform reference. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_num_xy(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label_txt: &str,
    id_x: NodeId,
    vx: f32,
    id_y: NodeId,
    vy: f32,
    min: f32,
    max: f32,
    step: f64,
    decimals: usize,
    // ⭐⭐ **A coluna é da SECÇÃO** — ver [`paint_num_row`].
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    // ⭐⭐⭐ **O `X`/`Y` viaja no NOME, e as letras coloridas saíram** — a mesma decisão que o
    // Inspector tomou em 2026-09-15, por ordem do dono (*«Position X/Y Caixa Caixa»*). Medido lá: a
    // coluna própria da letra custa `~52 px` de largura de painel antes de as duas caixas ficarem
    // lado a lado, e era isso que partia a disposição.
    {
        let store = ctx.host.store_mut();
        arm_field(store, id_x, vx, min, max, step, decimals);
        arm_field(store, id_y, vy, min, max, step, decimals);
    }
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    ph2d_editor_core::property_row::paint_fields_row(
        ctx.scene,
        ctx.text_system,
        theme,
        hit_index,
        store,
        x,
        content_w,
        y,
        label_txt,
        &[id_x, id_y],
        step,
        None,
        sec,
    )
}

/// Per-pattern params (all `0..1`, step `0.01`): pair two consecutive SHORT-label params on one line
/// (each ≤ [`PAIR_MAX_LEN`] chars, e.g. Voronoi's Metric / Edges), else one per line. Returns the next `y`.
/// ⭐⭐⭐ **A PORTA das fileiras de um PADRÃO — a lei de que existiam três cópias.**
///
/// ⛔⛔⛔ **Report do dono, 2026-09-21 (com foto): a secção `SHAPE ▸ Texture` mostrava
/// `paint_brush.pattern_param.contrast` em vez de `Contrast`.** O laço que monta estas fileiras
/// estava escrito **três vezes** — [`super::paint_texture`], [`super::paint_watercolor_paper`] e
/// [`super::paint_shape`] — e duas delas traduziam o rótulo. A terceira passava a **chave crua**.
///
/// ⚠️ *Uma lei escrita em três sítios viaja para os dois de que alguém se lembrou* — e o
/// `s.label` é uma CHAVE, logo esquecer a tradução não dá erro de compilação nem de tipo: dá um
/// identificador pintado no ecrã do artista.
///
/// ⛔ E nenhum dos 30 censos de texto a via: eles procuram **literais** no fonte, e aqui não há
/// literal nenhum — há um campo. Quem a apanha é a régua que lê o ECRÃ
/// (`nenhum_rotulo_do_app_pinta_uma_chave`).
pub(crate) fn params_de_padrao(
    kind: ph2d_tool_painter::TextureKind,
    ids: &[NodeId],
    valores: &[f32],
) -> Vec<(&'static str, NodeId, f32)> {
    ph2d_tool_painter::param_specs(kind)
        .iter()
        .enumerate()
        .map(|(i, s)| (ph2d_i18n::tr(s.label), ids[i], valores[i]))
        .collect()
}

pub(crate) fn paint_num_params(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    params: &[(&str, NodeId, f32)],
    // ⛔⛔ **A coluna é a da SECÇÃO que chama** (2026-09-23). Ela saía desta tabela, e as fileiras
    //    do padrão arrancavam `25 px` à direita das linhas do mesmo cartão (varredura
    //    `onde_comeca_o_valor`); a secção passou a medir estes nomes — ver `seccoes::COM_PADROES`.
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let font = ph2d_tokens::TypeToken::Sm.px();
    let gap = Spacing::Xs.px();
    let half = ((content_w - gap) * 0.5).max(0.0);
    let mut i = 0;
    while i < params.len() {
        let (l0, id0, v0) = params[i];
        // ⭐⭐⭐ **Emparelhar é uma escolha de PRODUTO; cabe é uma MEDIÇÃO** — report do dono,
        // 2026-09-15, com duas fotos: *«Em Grain: Voronoi : Metric e Edges os nomes somem ao
        // estreitar o painel. Melhor seria quebrar a linha»*.
        //
        // ⚠️⚠️ **O `PAIR_MAX_LEN` diz QUAIS params podem partilhar uma fileira** (uma decisão de
        // desenho sobre nomes curtos) e **nunca soube se eles CABEM ali**: numa metade estreita a
        // coluna do nome fica menor que a reticência e o pintor devolve string vazia — a resposta
        // certa dele para uma coluna degenerada, e a errada para quem escolheu emparelhar.
        // *O degrau a seguir a «não cabe o nome» não é apagar o nome — é deixar de emparelhar.*
        //
        // ⛔ A pergunta vai à PORTA (`property_row_fits`), não a uma segunda aritmética daqui.
        let elegivel = i + 1 < params.len()
            && l0.len() <= PAIR_MAX_LEN
            && params[i + 1].0.len() <= PAIR_MAX_LEN;
        let cabe = elegivel && {
            let w0 = ctx.text_system.prefix_width(l0, font);
            let w1 = ctx.text_system.prefix_width(params[i + 1].0, font);
            ph2d_editor_core::property_row::property_row_fits(half, w0)
                && ph2d_editor_core::property_row::property_row_fits(half, w1)
        };
        if cabe {
            let (l1, id1, v1) = params[i + 1];
            half_param(ctx, theme, x, half, y, l0, id0, v0, sec);
            half_param(ctx, theme, x + half + gap, half, y, l1, id1, v1, sec);
            y += ph2d_tokens::row_pitch_px();
            i += 2;
        } else {
            y = paint_num_row(
                ctx, theme, x, content_w, y, l0, id0, v0, 0.0, 1.0, FINE_STEP, 2, sec,
            );
            i += 1;
        }
    }
    y
}

/// One `[label | box]` half of a paired-param line, within `[x, x + w]`.
#[allow(clippy::too_many_arguments)]
fn half_param(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
    label_txt: &str,
    id: NodeId,
    v: f32,
    sec: ph2d_editor_core::property_row::Seccao,
) {
    // ⭐⭐ **Uma METADE é uma linha de propriedade dentro da largura dela** — a porta trabalha sobre
    // qualquer `[x, w]`, e por isso a coluna do nome aqui é a metade da METADE, não uma constante.
    //
    // ⚠️ **DOIS pontos de animação nesta fileira, e está certo:** *«um ponto por LINHA, nunca por
    // campo»* fala de uma propriedade com várias componentes; aqui são **duas propriedades
    // diferentes** lado a lado, e um ponto só diria que são uma.
    arm_field(ctx.host.store_mut(), id, v, 0.0, 1.0, FINE_STEP, 2);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    ph2d_editor_core::property_row::paint_fields_row(
        ctx.scene,
        ctx.text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label_txt,
        &[id],
        FINE_STEP,
        None,
        sec,
    );
}
