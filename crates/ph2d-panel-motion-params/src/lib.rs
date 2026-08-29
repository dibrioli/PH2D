//! `ph2d-panel-motion-params` — the Motion Nodes node-params panel (M1.P1).
//!
//! Right-docked in the Inspector slot (takeover, mirror of `ph2d-panel-vector`),
//! visible only while the `motion` tool is active — the shell's `motion_bridge`
//! drives `panel_visible("motion_params")` and hides the real Inspector.
//!
//! Reads the selected node's [`ParamsSnapshot`] (published by the bridge) and
//! paints one canonical **label + slider + numeric chip** row per param, using a
//! fixed pool of positional slider/chip widgets (`param_slider_id(slot)`). A row
//! edit returns as a [`MotionParamIntent::SetParam`] the bridge applies to the
//! graph. The slider tracks the doc value every frame **except** while the user
//! is dragging the slider or typing in the chip (so undo / external edits
//! live-update the knob without fighting an in-progress interaction).

#![forbid(unsafe_code)]

mod curve_row;
mod events;
mod gradient_row;
mod number_rows;
mod palette_row;
mod rows_paint;
mod shaper_dispatch;
mod snapshot;
mod text_rows;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
// O sufixo `_tests.rs` NÃO é cosmético: os gates de HR-15 (`no_magic_numeric`,
// `no_literal_color`) isentam por ele os irmãos de teste extraídos. Um nome fora do
// padrão põe o arquivo de volta sob a regra de chrome, e as dimensões de um viewport
// de fixture viram "magic numbers" (pego na integração de 2026-07-30).
#[path = "lib_gradient_tests.rs"]
mod tests_gradient;

/// The **dual range** gates (soft slider vs hard box) — a sibling of `lib_tests`
/// split off at the panel LOC cap along the subject line, not by size.
#[cfg(test)]
#[path = "lib_range_tests.rs"]
mod tests_range;

/// A rolagem do corpo (doc 88 §B3) — irmão dos de faixa, cortado pelo mesmo assunto.
#[cfg(test)]
#[path = "lib_scroll_tests.rs"]
mod tests_scroll;

/// A afordância de **reverter ao default** — as quatro condições de UI dela.
#[cfg(test)]
#[path = "lib_reset_tests.rs"]
mod tests_reset;

/// The **display face** gates (doc 88) — the sibling of `tests_range` on the
/// other axis: that one pins how far the box reaches, this one pins what the
/// document hears when a number comes back from it.
#[cfg(test)]
#[path = "lib_unit_tests.rs"]
mod tests_unit;

/// **Quem é dono de que campo** quando o painel se re-semeia a cada quadro — o
/// valor é do seed, o estado é do dispatch.
#[cfg(test)]
#[path = "lib_seed_tests.rs"]
mod tests_seed;

/// A row de **FICHEIRO** — o botão que abre o diálogo, o campo que continua editável, e a
/// marca de *missing footage*.
#[cfg(test)]
#[path = "lib_file_tests.rs"]
mod tests_file;

use events::{on_click, on_text_commit, on_toggled, on_value_changed};
use number_rows::{
    ANGLE_DECIMALS, SEED_DECIMALS, mirror_chip, mirror_number, mirror_slider, next_seed,
    number_is_typing, number_value, paint_angle_row, paint_seed_row,
};
pub use snapshot::{
    AngleRow, ChannelsRow, ColorRow, CurveRow, EnumRow, FileRow, GradientRow, MAX_ENUM_OPTIONS,
    MAX_PARAM_ROWS, MotionParamIntent, PaletteRow, ParamRow, ParamsSnapshot, RowDisplay, ScalarRow,
    SeedRow, SourceRow, TextRow, ToggleRow, drain_param_intents, param_grad_swatch_id,
    param_pal_swatch_id, param_swatch_id, push_param_intent, scalar_text, set_current_params,
};
use snapshot::{
    CHANNELS_EXTRA_BASE, current_params, param_checkbox_id, param_chip_id, param_enum_id,
    param_file_browse_id, param_number_id, param_reroll_id, param_reset_id, param_slider_id,
    param_text_id,
};
use text_rows::{mirror_text, paint_text_row, text_is_typing, text_value};

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore, format_number};
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    ButtonState, CheckboxState, CheckboxValue, MOTION_PARAMS_SCROLLBAR_ID, NUMBER_INPUT_MIN_W_PX,
    SliderOrientation, SliderState, TextInputState, paint_scrollbar, scrollbar_is_needed,
    scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{Spacing, TypeToken};

/// Retained panel state — none needed: the selected node + its params live
/// shell-side (`MotionState`) and the pooled widgets re-seed from the published
/// snapshot each frame (interaction-gated). Unit struct so the typed registry can
/// default-construct it.
#[derive(Default)]
pub struct MotionParamsPanelState;

/// Zero-size marker implementing the typed node-params panel contract.
pub struct MotionParamsPanel;

impl Panel for MotionParamsPanel {
    type State = MotionParamsPanelState;

    const ID: &'static str = "motion_params";
    const NODE_ID: NodeId = ids::MOTION_PARAMS_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(_state: &mut MotionParamsPanelState, ctx: &mut PaintCtx) {
        if !ctx.host.panel_visible(MotionParamsPanel::ID) {
            ctx.host
                .store_mut()
                .clear_panel_rect(ids::MOTION_PARAMS_PANEL);
            return;
        }
        let rect = ctx.layout.inspector;
        let theme = ctx.host.theme();
        ctx.host
            .store_mut()
            .set_panel_rect(ids::MOTION_PARAMS_PANEL, rect);
        paint_panel_surface(rect, ctx.scene, theme);

        let snap = current_params();
        let title = snap.as_ref().map(|s| s.title.as_str()).unwrap_or("Motion");
        let title_size = paint_panel_title(rect, title, 0.0, ctx.scene, ctx.text_system, theme);

        let Some(snap) = snap else {
            return;
        };

        // Layout (mirror of the Vector panel body metrics).
        let inner_x = rect.x + PANEL_HEAD_PAD;
        let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
        let chip_w = NUMBER_INPUT_MIN_W_PX;
        let row_gap = Spacing::Sm.px();
        let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
        // ⚠️ **O corpo ROLA** (doc 88 §B3). Medido: uma linha escalar ocupa 34 px e o dock
        // comporta 24 — contra um teto de 16 e um pior nó de 15 params. A varredura PRO
        // consome essa folga, e o gate `a_full_panel_of_rows_fits_the_inspector` já dizia
        // em texto o que fazer no dia: *"o painel precisa ROLAR antes de o teto subir mais"*.
        //
        // ⚠️ O `push_clip` **não é enfeite**: sem ele as linhas roladas para cima desenham
        // por cima do título e para fora do painel — a rolagem ingênua que parece funcionar
        // no meio do percurso e quebra nas duas pontas.
        let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
        let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
        let scroll = ctx.host.store().panel_scroll(ids::MOTION_PARAMS_PANEL);
        ctx.scene.push_clip(&rect_to_vello(body_rect));
        // ⚠️ **UMA BANDA, DOIS CONSUMIDORES.** O `push_clip` da cena recorta o
        // DESENHO; sem o gémeo no `HitIndex`, uma linha rolada para cima continua
        // **registada** onde ninguém a vê — o hit-rect sobe para a faixa do
        // TÍTULO e o clique dele passa a valer ali.
        //
        // Enquanto o teto de linhas coube no dock isso era inofensivo por
        // ARITMÉTICA (o `max_scroll` era `0` e nada saía do corpo), e o gate
        // `the_scroll_is_inert_at_todays_row_cap_so_no_row_can_hide_under_the_title`
        // dizia, em texto, que *"o dia em que o teto subir é o dia em que a
        // blindagem passa a ser necessária"*. O teto subiu (20 → 24, a wave do
        // `motion.bezier_warp`), e este é o dia.
        //
        // ⚠️ **A ferramenta já existia e este painel não a chamava** — o
        // `HitIndex::push_clip` é a mesma pilha que o `section_header::body` usa
        // desde que nasceu. A cura não é código novo: é o segundo consumidor da
        // banda que já se calculava aqui.
        ctx.host.hit_index_mut().push_clip(body_rect);

        // Phase A — seed the pooled widgets from the doc values (skipping any row
        // being interacted with) + refresh each chip's range + slider↔chip link.
        seed_rows(ctx.host.store_mut(), &snap.rows);

        // Phase A — paint each row: a scalar slider-with-chip composite, or a
        // colour swatch (label + right-aligned swatch, mirror of the Vector
        // Stroke/Fill rows). Both use the SHARED source-of-truth painters.
        let label_font = TypeToken::Base.px();
        // `paint_rows` draws + registers hit rects (it holds `hit_index`); a Curve row's
        // `CurvePoint`/`Button` STORE states cannot be registered through the immutable
        // store here, so they ride back in `curve_widgets` for Phase C below.
        // ── A SEMENTE DAS SEÇÕES, **antes** do desenho que ela governa ───────────────
        //
        // O collapse genérico exige DOIS sítios: o hit-rect (no paint) e a MARCA aqui. Sem a
        // marca o cabeçalho pinta um chevron e não dobra — o título morto que o painel do
        // Vector já pagou. Marcado na fase mutável porque os títulos são do NÓ selecionado,
        // e o `populate` (estático) não os conhece.
        //
        // ⚠️ **Isto vivia na fase C, DEPOIS do `paint_rows`, e mudar de sítio foi uma
        // CORREÇÃO, não arrumação:** uma seção que nasce fechada era desenhada ABERTA no
        // primeiro quadro e só fechava no seguinte — um pisca visível, e o censo de altura
        // (que pinta exactamente uma vez) media o nó com a seção aberta e a acusar de
        // estourar o dock. *Uma semente que corre depois do desenho que ela governa mostra
        // um quadro do estado errado.*
        //
        // ⚠️ **`collapsed_choice` distingue «o artista não escolheu» de «ele escolheu
        // aberto»** — sem essa distinção este laço re-fecharia a gaveta a cada quadro, e o
        // clique de quem a abriu duraria um frame.
        {
            let store = ctx.host.store_mut();
            for (title, _) in &snap.sections {
                let id = rows_paint::sections::section_id(title);
                store.mark_collapsible_section(id);
                if snap.folded_by_default.contains(title) && store.collapsed_choice(id).is_none() {
                    store.set_collapsed(id, true);
                }
            }
        }
        let (curve_widgets, gradient_widgets, content_h) = {
            let scene = &mut *ctx.scene;
            let text_system = &mut *ctx.text_system;
            let (store, hit_index) = ctx.host.store_and_hit_index_mut();
            rows_paint::paint_rows(
                &snap.rows,
                inner_x,
                inner_w,
                chip_w,
                row_gap,
                body_top - scroll,
                label_font,
                store,
                hit_index,
                scene,
                text_system,
                theme,
                &snap.modified,
                &snap.sections,
            )
        };
        ctx.scene.pop_layer();
        // ⚠️ O `pop` vem ANTES da barra de rolagem, e de propósito: o thumb vive
        // no corpo mas não rola com ele — recortá-lo pela mesma banda seria
        // correcto hoje e uma armadilha no dia em que ele saísse um pixel.
        ctx.host.hit_index_mut().pop_clip();
        paint_scroll_chrome(
            ctx,
            body_rect,
            content_h + PANEL_HEAD_PAD,
            body_h,
            scroll,
            theme,
        );

        // Phase B (mutable store) — mark each colour swatch so a Down opens the
        // shared OKLCH picker (generic `is_picker_swatch` dispatch). Idempotent.
        // Phase C — register the Curve editor's per-frame `CurvePoint` handles (the
        // dispatch reads `canvas`/`index` off these to normalize a drag) + its buttons.
        {
            let store = ctx.host.store_mut();
            // O collapse genérico exige DOIS sítios: o hit-rect (no paint) e a MARCA aqui.
            // Sem a marca o cabeçalho pinta um chevron e não dobra — o título morto que o
            // painel do Vector já pagou. Marcado na fase mutável porque os títulos são do NÓ
            // selecionado, e o `populate` (estático) não os conhece.
            for row in &snap.rows {
                if let ParamRow::Color(c) = row {
                    store.register_picker_swatch(param_swatch_id(c.channels[0]));
                }
            }
            for &(id, parent, index, canvas) in &curve_widgets.points {
                store.register(
                    id,
                    InteractiveState::CurvePoint {
                        parent,
                        channel: 0,
                        index,
                        canvas,
                    },
                );
            }
            for &id in &curve_widgets.buttons {
                store.register(
                    id,
                    InteractiveState::Button {
                        state: ButtonState::Normal,
                    },
                );
            }
            // Gradient editor (doc 85): the position markers are `CurvePoint` handles (the
            // dispatch normalizes a drag off `canvas`), each stop swatch a picker swatch (a
            // Down opens the shared OKLCH picker; seeded here from the paint's srgb so it
            // opens on the stop's colour), and `+`/`−`/interp are buttons.
            for &(id, parent, index, canvas) in &gradient_widgets.markers {
                store.register(
                    id,
                    InteractiveState::CurvePoint {
                        parent,
                        channel: 0,
                        index,
                        canvas,
                    },
                );
            }
            for &(sid, srgb) in &gradient_widgets.swatches {
                store.register_picker_swatch(sid);
                store.set_widget_color(sid, srgb);
            }
            for &id in &gradient_widgets.buttons {
                store.register(
                    id,
                    InteractiveState::Button {
                        state: ButtonState::Normal,
                    },
                );
            }
        }
    }

    fn apply_event(
        _state: &mut MotionParamsPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        let Some(snap) = current_params() else {
            return EventOutcome::Ignored;
        };
        match ev {
            // A Curve/Gradient handle drag arrives as ValueChanged(editor) — drain it FIRST
            // (the dispatch stashed the normalized point); else it is a scalar slider / chip.
            WidgetEvent::ValueChanged(id) => shaper_dispatch::on_gradient_drag(id, host, &snap)
                .or_else(|| shaper_dispatch::on_curve_drag(id, host, &snap))
                .unwrap_or_else(|| on_value_changed(id, host, &snap)),
            WidgetEvent::Toggled(id) => on_toggled(id, host, &snap),
            // A Curve/Gradient +/−/interp button, else a segmented option / seed re-roll.
            WidgetEvent::Click(id) => shaper_dispatch::on_gradient_click(id, &snap)
                .or_else(|| shaper_dispatch::on_curve_click(id, &snap))
                .unwrap_or_else(|| on_click(id, &snap)),
            // A formula field commits on Enter (Submit) or focus-loss (Blur).
            WidgetEvent::Submit(id) | WidgetEvent::Blur(id) => on_text_commit(id, host, &snap),
            _ => EventOutcome::Ignored,
        }
    }

    fn populate(store: &mut WidgetStore) {
        // Register the pooled widgets so the dispatch can route drags / clicks /
        // toggles even before the first paint seeds their values: a slider + chip,
        // a checkbox, and a row of segmented option buttons per slot.
        for slot in 0..MAX_PARAM_ROWS {
            store.register(
                param_slider_id(slot),
                InteractiveState::Slider {
                    state: SliderState::Normal,
                    value: 0.5,
                    orientation: SliderOrientation::Horizontal,
                },
            );
            store.register(param_chip_id(slot), number_input(0.0));
            // Standalone number box (Angle / Seed rows) + the Seed re-roll button.
            store.register(param_number_id(slot), number_input(0.0));
            store.register(
                param_reroll_id(slot),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
            // A seta de reverter ao default. Registrada aqui como qualquer botão pooled —
            // sem isto ela pinta, entra no hit index, e fica MORTA sob o mouse (o dispatch só
            // roteia clique para widget que o `populate` registrou).
            store.register(
                param_reset_id(slot),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
            store.register(
                param_text_id(slot),
                InteractiveState::TextInput {
                    state: TextInputState::Normal,
                    text: String::new(),
                    caret: 0,
                    selection_anchor: None,
                },
            );
            // O botão *Browse…* de uma File row. Sem isto ele pinta, entra no hit index, e
            // fica MORTO sob o dedo — o mesmo defeito que a seta de reverter teve, e que o
            // §5 do CLAUDE.md regista com o nome: *um controlo nunca pintado e um morto sob o
            // ponteiro dão o MESMO report*.
            store.register(
                param_file_browse_id(slot),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
            store.register(
                param_checkbox_id(slot),
                InteractiveState::Checkbox {
                    state: CheckboxState::Normal,
                    value: CheckboxValue::Unchecked,
                },
            );
            for opt in 0..MAX_ENUM_OPTIONS {
                store.register(
                    param_enum_id(slot, opt),
                    InteractiveState::Button {
                        state: ButtonState::Normal,
                    },
                );
            }
            // The live-column chips of a Channels row's Custom picker reuse the
            // enum-button pool from a base ABOVE the curated segments — register them
            // here too, or they paint + hit-register yet stay DEAD under the mouse
            // (the dispatch only routes a click to a widget `populate` registered).
            for j in 0..MAX_ENUM_OPTIONS {
                store.register(
                    param_enum_id(slot, CHANNELS_EXTRA_BASE + j),
                    InteractiveState::Button {
                        state: ButtonState::Normal,
                    },
                );
            }
        }
    }
}

/// True while any pooled param row is being interacted with — a slider dragged /
/// focused, a chip focused, or a standalone number box (Angle / Seed) focused.
/// The shell reads this as the undo-bracket edge (open on the false→true
/// transition, commit one step on true→false), so a whole slider drag — or a
/// whole type-into-the-angle-box — is a single undo step (M1.P1).
#[must_use]
pub fn any_param_editing(store: &WidgetStore) -> bool {
    (0..MAX_PARAM_ROWS).any(|slot| {
        matches!(
            store.slider(param_slider_id(slot)),
            Some((SliderState::Dragging | SliderState::Focused, _))
        ) || matches!(
            store.get(param_chip_id(slot)),
            Some(InteractiveState::NumberInput {
                state: TextInputState::Focused,
                ..
            })
        ) || number_is_typing(store, param_number_id(slot))
            || text_is_typing(store, param_text_id(slot))
    })
}

/// `[0,1]` slider track for a value in `[min, min+span]`.
pub(crate) fn normalized_track(value: f64, min: f64, span: f64) -> f32 {
    (((value - min) / span) as f32).clamp(0.0, 1.0)
}

/// The param value a slider `track` (`0..1`) maps to over `[min, max]`, rounded
/// to a whole number for integer params. Shared by `paint` (chip display) and
/// `apply_event` (the emitted `SetParam` value) so the knob and the doc agree.
pub(crate) fn row_value(track: f32, min: f64, max: f64, integer: bool) -> f64 {
    let span = (max - min).max(f64::EPSILON);
    let v = min + f64::from(track) * span;
    if integer { v.round() } else { v }
}

/// A `NumberInput` state seeded at `value` (buffer = its canonical formatting).
fn number_input(value: f64) -> InteractiveState {
    InteractiveState::NumberInput {
        state: TextInputState::Normal,
        value,
        buffer: format_number(value),
        caret: 0,
        last_committed: value,
        selection_anchor: None,
    }
}

/// Seed each pooled row from its doc value + refresh its range/link, EXCEPT for a
/// row currently being dragged (slider) or typed (chip) — so an in-progress
/// interaction owns the widget, but idle rows always mirror the doc (undo /
/// external edits live-update the knob).
fn seed_rows(store: &mut WidgetStore, rows: &[ParamRow]) {
    for (i, row) in rows.iter().enumerate().take(MAX_PARAM_ROWS) {
        // Toggle rows: sync the checkbox value to the doc while PRESERVING its
        // transient hover/press state (the dispatch owns that + flips the value
        // on click). Colour swatches seed from the bridge's `widget_color`; Enum
        // buttons are stateless (selection comes from the snapshot at paint).
        if let ParamRow::Toggle(t) = row {
            let cb_id = param_checkbox_id(i);
            let state = store
                .checkbox(cb_id)
                .map(|(s, _)| s)
                .unwrap_or(CheckboxState::Normal);
            store.register(
                cb_id,
                InteractiveState::Checkbox {
                    state,
                    value: if t.on {
                        CheckboxValue::Checked
                    } else {
                        CheckboxValue::Unchecked
                    },
                },
            );
            continue;
        }
        // Standalone number boxes (Angle in degrees, Seed as a whole number):
        // mirror the doc value when unfocused + register the range so the
        // drag-scrub is range-proportional and the stepper uses `step`.
        if let ParamRow::Angle(a) = row {
            let id = param_number_id(i);
            mirror_number(store, id, a.deg, ANGLE_DECIMALS);
            store.set_number_range(id, a.min_deg, a.max_deg, a.step_deg);
            continue;
        }
        if let ParamRow::Seed(s) = row {
            let id = param_number_id(i);
            mirror_number(store, id, s.value, SEED_DECIMALS);
            store.set_number_range(id, s.min, s.max, 1.0);
            continue;
        }
        // Text (formula) rows: mirror the doc formula into the field when unfocused.
        if let ParamRow::Text(t) = row {
            mirror_text(store, param_text_id(i), &t.value);
            continue;
        }
        let ParamRow::Scalar(row) = row else { continue };
        let slider_id = param_slider_id(i);
        let chip_id = param_chip_id(i);
        let span = (row.max - row.min).max(f64::EPSILON);

        let dragging = matches!(
            store.slider(slider_id),
            Some((SliderState::Dragging | SliderState::Focused, _))
        );
        let typing = matches!(
            store.get(chip_id),
            Some(InteractiveState::NumberInput {
                state: TextInputState::Focused,
                ..
            })
        );
        if !dragging && !typing {
            // Value from the doc, STATE from the dispatch — `number_rows` owns both
            // halves of that split now, for the reason written there.
            mirror_slider(store, slider_id, normalized_track(row.value, row.min, span));
            mirror_chip(store, chip_id, row.value);
        }
        // Range (chip typed-value clamp/step) + slider↔chip affine (track 0..1 →
        // value): display = track * span + min. Integer rows snap the chip.
        //
        // The CHIP gets the HARD range and the slider keeps the soft one: the
        // drag range and the legal range are different questions (Blender's soft
        // vs hard limits). Outside `[row.min, row.max]` the affine below saturates
        // the track at 0.0 / 1.0, so such a value cannot come back through the
        // slider — which is exactly why `on_value_changed` lets the chip speak for
        // itself out there. Both ENDS, since doc 88: the ceiling shipped alone and
        // the floor was pinned to the slider's `min`, so a param whose useful drag
        // starts at `0.01` could not be typed to `0.001`.
        store.set_number_range(chip_id, row.hard_min, row.hard_max, row.step);
        if row.integer {
            store.link_slider_number_mapped_integer(
                slider_id,
                chip_id,
                span as f32,
                row.min as f32,
            );
        } else {
            store.link_slider_number_mapped(slider_id, chip_id, span as f32, row.min as f32);
        }
    }
}

/// Desenha a barra de rolagem e **publica** o par `content_h`/`visible_h` que o dispatch da roda
/// consome.
///
/// ⚠️ **Publicar é a metade que não se vê e sem a qual a roda não faz nada:** o
/// `dispatch_wheel` deriva o `max_scroll` desses dois números, então um painel que recorta,
/// desloca e desenha o thumb — mas não publica — rola com o thumb e fica **inerte na roda**. É
/// o mesmo modo de falha silencioso que o arch-gate `scrollable_panels_intercept_the_wheel`
/// documenta para a quarta edição (o `cursor_over_hero_panel`), uma casa antes.
fn paint_scroll_chrome(
    ctx: &mut PaintCtx,
    body_rect: ph2d_editor_core::zones::Rect,
    content_h: f32,
    body_h: f32,
    scroll: f32,
    theme: ph2d_tokens::Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        paint_scrollbar(
            body_rect,
            scroll,
            content_h,
            body_h,
            ctx.host
                .store()
                .scrollbar_visual(MOTION_PARAMS_SCROLLBAR_ID),
            ctx.scene,
            theme,
        );
        ctx.host
            .hit_index_mut()
            .register(MOTION_PARAMS_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::MOTION_PARAMS_PANEL, content_h);
    store.set_panel_visible_h(ids::MOTION_PARAMS_PANEL, body_h);
    // ⚠️ O clamp existe porque o conteúdo ENCOLHE ao trocar de nó: um `field.remap` rolado até
    // o fim seguido de um `motion.grid` deixaria o rolamento além do fim de um corpo de três
    // linhas, e o painel abriria em branco.
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::MOTION_PARAMS_PANEL) > max_scroll {
        store.set_panel_scroll(ids::MOTION_PARAMS_PANEL, max_scroll);
    }
}
