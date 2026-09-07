//! The effects rack section of the Audio Editor panel (W3 blocks 3a/3b).
//!
//! A **chain** of effects, rendered in order: `clip → stage₀ → … → stageₙ`. One
//! stage is selected; the selector (`◀ Name ⟲ ▶`) sets its kind and the sliders
//! tune it. Below, the chain list shows every stage with an eye toggle (per-stage
//! bypass), and the action row adds / removes / reorders the selected one.
//!
//! The rack **auditions live**: any edit marks it dirty and the shell renders the
//! whole chain into the sounding preview, so it is heard (and drawn) while you
//! tune. **Bypass** swaps the dry clip back in without losing the chain — the A/B.
//! **Apply** commits exactly that buffer as one undo step; **Cancel** throws it away.
//!
//! The panel is UI-only: the chain carries **normalized 0..1** slider values and a
//! kind index, and the shell publishes each slot's label + already-formatted value
//! (`audio/fx_params.rs`), so no DSP range or unit ever lands here.

use crate::paint::{
    ARROW_W, ClippedHits, ROW_H, action_bg, button, button_in_group, display_in_group,
    toggle_in_group,
};
use crate::{
    AEDIT_FX_ADD, AEDIT_FX_APPLY, AEDIT_FX_BYPASS, AEDIT_FX_CANCEL, AEDIT_FX_DOWN, AEDIT_FX_NEXT,
    AEDIT_FX_PARAMS, AEDIT_FX_PREV, AEDIT_FX_REMOVE, AEDIT_FX_RESET, AEDIT_FX_STAGE_ONS,
    AEDIT_FX_STAGES, AEDIT_FX_UP, AEDIT_PRESET_APPLY, AEDIT_PRESET_LOAD, AEDIT_PRESET_NEXT,
    AEDIT_PRESET_PREV, AEDIT_PRESET_SAVE, MAX_FX_STAGES, presets, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{paint_text, paint_text_centered, resolve};
use ph2d_editor_core::widget::{
    ButtonState, IconButtonStyle, IconGlyph, SEGMENT_HAIRLINE, Slider, SliderOrientation,
    paint_icon_button, paint_slider,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A altura de uma linha da CADEIA — **a linha do app** (wave 21). Ela era
/// `TypeToken::Sm + Spacing::Sm` (≈ 18 px), uma terceira resposta a *«que altura tem uma
/// linha?»* que nenhum teste podia ver, porque estava certa sozinha.
const CHAIN_ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// Buttons in the chain action row (Add · Remove · Up · Down).
const ACTION_BUTTONS: f32 = 4.0; // LITERAL-PX-OK: fixed count, divides the row width
/// The painter's shared borrows, bundled so each section fits one argument list.
struct Ctx<'a, 'h> {
    scene: &'a mut VectorScene,
    text_system: &'a mut TextSystem,
    theme: Theme,
    hit_index: &'a mut ClippedHits<'h>,
}

/// Paint the rack starting at `y`; returns the `y` below it. `loaded` dims the
/// controls when there is no clip to act on.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_fx_section(
    y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let ctx = &mut Ctx {
        scene,
        text_system,
        theme,
        hit_index,
    };
    let y = paint_head(y, x, w, loaded, row_h, ctx);
    let y = paint_params(y, x, w, loaded, ctx);
    let y = paint_chain(y, x, w, loaded, row_h, ctx);
    paint_commit_row(y, x, w, loaded, row_h, ctx)
}

/// **Um ÍCONE que é peça de um grupo** — o `↺` do selector de efeito.
///
/// ⚠️ **O irmão solto (`icon_button`) não pinta superfície nenhuma**, e dentro de um corpo isso
/// abre um buraco: as vizinhas têm fundo e ele não. Aqui ele ganha a superfície e as quinas da
/// célula, como qualquer outra peça.
fn icon_in_group(
    rect: Rect,
    icon: IconId,
    enabled: bool,
    id: NodeId,
    pos: ph2d_editor_core::widget::GroupCell,
    ctx: &mut Ctx,
) {
    let bg = action_bg(
        ColorToken::Bg3,
        ColorToken::BgElev,
        ColorToken::AccentSoft,
        ctx.hit_index.visual(id),
        enabled,
        ctx.theme,
    );
    ph2d_editor_core::paint::fill_rounded_rect_radii(
        ctx.scene,
        rect,
        pos.radii(ph2d_editor_core::paint::frame_radius(
            ctx.theme,
            Radius::Sm.px(),
        )),
        bg,
    );
    ph2d_editor_core::paint::paint_icon(
        ctx.scene,
        icon,
        rect,
        resolve(
            if enabled {
                ColorToken::Text1
            } else {
                ColorToken::TextDisabled
            },
            ctx.theme,
        ),
        ph2d_tokens::StrokeToken::Default.px(),
    );
    if enabled {
        ctx.hit_index.register(id, rect);
    }
}

/// ⭐⭐⭐ **O cabeçalho da secção EFFECTS — TRÊS fileiras, UM corpo.**
///
/// `◀ preset ▶` · `Apply | Save | Load` · `◀ efeito ↺ ▶`. Report do dono, 2026-09-07 (*«pode
/// juntar»*, sobre o vão que sobrava entre as duas primeiras e a terceira).
///
/// ⚠️ **As três respondem à mesma pergunta — *o que estou a editar?*** Um preset é um ponto de
/// partida para a cadeia; as três ordens agem sobre ele; o selector diz que efeito da cadeia está
/// aberto nos parâmetros abaixo. Separá-las dizia ao olho que são três controlos.
///
/// ⚠️ **As fileiras têm larguras DESIGUAIS entre si** (`3` peças · `3` iguais · `4` peças), e é o
/// [`ph2d_editor_core::widget::block_cells_of`] que as exprime — o `block_cells` reparte cada
/// fileira em partes iguais e não chega aqui.
fn paint_head(y: f32, x: f32, w: f32, loaded: bool, row_h: f32, ctx: &mut Ctx) -> f32 {
    let has_presets = presets::preset_count() > 0;
    let (kind, _) = snapshot::fx_sel_stage();
    let icon_w = row_h;

    let preset_mid = ph2d_editor_core::widget::stepper_middle_w(w, ARROW_W);
    // ⚠️ Os fios são `peças − 1`, DERIVADO: escrever o número à mão aqui é a mesma espécie de
    //    literal que o `stepper_middle_w` existe para evitar.
    const FX_PIECES: f32 = 4.0; // LITERAL-PX-OK: contagem de pecas do selector, nao um px
    let fx_mid = (w - ARROW_W * 2.0 - icon_w - SEGMENT_HAIRLINE * (FX_PIECES - 1.0)).max(1.0);
    const ACTIONS: f32 = 3.0; // LITERAL-PX-OK: contagem de ordens de preset, nao um px
    let third = ((w - SEGMENT_HAIRLINE * (ACTIONS - 1.0)) / ACTIONS).max(1.0);

    let block = ph2d_editor_core::widget::block_cells_of(
        Rect::new(x, y, w, 0.0),
        &[
            &[ARROW_W, preset_mid, ARROW_W],
            &[third, third, third],
            &[ARROW_W, fx_mid, icon_w, ARROW_W],
        ],
        row_h,
    );

    // Fileira 0 — o preset escolhido.
    button_in_group(
        block[0][0].0,
        "\u{25c0}",
        has_presets,
        AEDIT_PRESET_PREV,
        block[0][0].1,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    display_in_group(
        block[0][1].0,
        &presets::preset_name(),
        has_presets,
        block[0][1].1,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
    );
    button_in_group(
        block[0][2].0,
        "\u{25b6}",
        has_presets,
        AEDIT_PRESET_NEXT,
        block[0][2].1,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );

    // Fileira 1 — o que se faz com ele. Apply carrega o preset de fábrica escolhido (audiciona já);
    // Save / Load são FICHEIROS de preset por diálogo nativo — o browser do SO é o picker deles.
    for (i, (label, on, id)) in [
        ("Apply", has_presets, AEDIT_PRESET_APPLY),
        ("Save", loaded, AEDIT_PRESET_SAVE),
        ("Load", loaded, AEDIT_PRESET_LOAD),
    ]
    .into_iter()
    .enumerate()
    {
        button_in_group(
            block[1][i].0,
            label,
            on,
            id,
            block[1][i].1,
            ctx.scene,
            ctx.text_system,
            ctx.theme,
            ctx.hit_index,
        );
    }

    // Fileira 2 — que efeito da cadeia está aberto, e o `↺` que o repõe.
    button_in_group(
        block[2][0].0,
        "\u{25c0}",
        loaded,
        AEDIT_FX_PREV,
        block[2][0].1,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    display_in_group(
        block[2][1].0,
        &snapshot::fx_kind_name(kind),
        loaded,
        block[2][1].1,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
    );
    icon_in_group(
        block[2][2].0,
        IconId::Reset,
        loaded && !snapshot::fx_at_defaults(),
        AEDIT_FX_RESET,
        block[2][2].1,
        ctx,
    );
    button_in_group(
        block[2][3].0,
        "\u{25b6}",
        loaded,
        AEDIT_FX_NEXT,
        block[2][3].1,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );

    y + ph2d_editor_core::widget::grid_height(3, row_h) + ph2d_tokens::control_gap_px()
}

/// One row per parameter of the selected stage: `label ......... value`, with the
/// slider under it. Slots the effect doesn't use are simply not painted — and not
/// hit-registered, so a stale slider can't be grabbed.
fn paint_params(mut y: f32, x: f32, w: f32, loaded: bool, ctx: &mut Ctx) -> f32 {
    let gap = Spacing::Xs.px();
    let views = snapshot::fx_param_views();
    let norms = snapshot::fx_norms();
    let label_h = TypeToken::Xs.px();
    let half = (w * 0.5).max(1.0);
    for (i, (label, value)) in views.iter().enumerate().take(AEDIT_FX_PARAMS.len()) {
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            label,
            Rect::new(x, y, half, label_h),
            TypeToken::Xs.px(),
            resolve(ColorToken::Text2, ctx.theme),
        );
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            value,
            Rect::new(x + half, y, half, label_h),
            TypeToken::Xs.px(),
            resolve(text_tone(loaded), ctx.theme),
        );
        y += label_h + Spacing::Xs.px();

        let id = AEDIT_FX_PARAMS[i];
        let track = Rect::new(x, y, w, Spacing::Md.px());
        let mut slider = Slider::new(id, label.as_str()).orientation(SliderOrientation::Horizontal);
        slider.set_value(norms[i]);
        paint_slider(&slider, track, ctx.scene, ctx.theme);
        if loaded {
            ctx.hit_index.register(id, track);
        }
        y += Spacing::Md.px() + gap;
    }

    // **The room.** Only the Convolution Reverb has one, and it is the one thing in the rack
    // that is not a number — so it gets a control the parameter table cannot express. The row
    // simply is not there for the other 38 effects: a permanently-dimmed "Load IR" button
    // would be 38 kinds of noise to spare one kind of special case.
    if snapshot::fx_needs_ir() {
        let read = snapshot::fx_ir_readout();
        button(
            Rect::new(x, y, w, ROW_H),
            if read.is_empty() {
                "Load IR\u{2026}"
            } else {
                "Change IR\u{2026}"
            },
            loaded,
            crate::AEDIT_FX_LOAD_IR,
            ctx.scene,
            ctx.text_system,
            ctx.theme,
            ctx.hit_index,
        );
        y += ph2d_tokens::row_pitch_px();
        // Which room is loaded — an empty slot has to look empty, or the user is left
        // wondering why a fully-wet reverb does nothing (it is bypassed: no room, no reverb).
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            if read.is_empty() {
                "no room loaded"
            } else {
                &read
            },
            Rect::new(x, y, w, label_h),
            TypeToken::Xs.px(),
            resolve(
                if read.is_empty() {
                    ColorToken::Warn
                } else {
                    ColorToken::Text3
                },
                ctx.theme,
            ),
        );
        y += label_h + gap;
    }
    y + ph2d_tokens::control_gap_px()
}

/// The chain: a header carrying the `+ | trash | ▲ | ▼` actions, then one row per
/// stage in render order. Clicking a row selects it (the selector + sliders follow);
/// the eye toggles it in and out of the render without dropping it — the per-stage
/// half of the A/B. The actions all act on the SELECTED stage. Order matters: a
/// filter before a reverb is not the same as after.
///
/// The panel does not scroll, so the actions ride the header rather than claiming a
/// row of their own.
fn paint_chain(mut y: f32, x: f32, w: f32, loaded: bool, row_h: f32, ctx: &mut Ctx) -> f32 {
    let count = snapshot::fx_stage_count();
    let sel = snapshot::fx_sel();

    let label_h = TypeToken::Xs.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Chain",
        x,
        y + (row_h - label_h) * 0.5,
        label_h,
        w,
        resolve(ColorToken::Text2, ctx.theme),
    );
    let actions = [
        (IconId::Add, loaded && count < MAX_FX_STAGES, AEDIT_FX_ADD),
        (IconId::Trash, loaded && count > 1, AEDIT_FX_REMOVE),
        (IconId::ChevronUp, loaded && sel > 0, AEDIT_FX_UP),
        (
            IconId::ChevronDown,
            loaded && sel + 1 < count,
            AEDIT_FX_DOWN,
        ),
    ];
    // Right-aligned, so the header reads `Chain ........ + 🗑 ▲ ▼`.
    let actions_x = x + w - row_h * ACTION_BUTTONS;
    for (i, (glyph, enabled, id)) in actions.into_iter().enumerate() {
        let rect = Rect::new(actions_x + row_h * i as f32, y, row_h, row_h);
        icon_button(rect, glyph, enabled, id, ctx);
    }
    y += row_h + ph2d_tokens::control_gap_px();

    // ⭐⭐⭐ **A cadeia é uma LISTA, e passa a obedecer às leis de uma** (wave 21, report do dono:
    //    *«sem layout adequado… estude o layout e corrija»*). Ela tinha **três** respostas só
    //    dela: a altura (`TypeToken::Sm + Spacing::Sm` ≈ 18 px, contra as 22 de toda linha do
    //    app), o vão (zero, escrito como a ausência de um termo) e o realce (`Bg3`, o tom de um
    //    botão parado). ⚠️ **E ela não estava no censo de listas** — que é uma lista DECLARADA à
    //    mão: *uma superfície que ninguém declarou escapa a um censo por declaração.*
    let stage_h = CHAIN_ROW_H;
    // A folga do cartão que envolve a secção: é até aí que a faixa de uma linha sangra.
    let bleed = Spacing::Xs.px();
    for i in 0..count.min(MAX_FX_STAGES) {
        let Some((name, enabled)) = snapshot::fx_stage_view(i) else {
            continue;
        };
        let row = Rect::new(x, y, w, stage_h);
        // ⭐ A listra da paridade (wave 18) vem primeiro, e o realce por cima dela.
        ph2d_editor_core::widget::paint_row_stripe(
            ctx.scene,
            Rect::new(x - bleed, y, w + bleed * 2.0, stage_h),
            ctx.theme,
            ph2d_editor_core::widget::section_cards::CardDepth::Section.token(),
            i,
        );
        ph2d_editor_core::widget::paint_row_highlight(
            ctx.scene,
            row,
            ctx.theme,
            if i == sel {
                ph2d_editor_core::widget::RowHighlight::Selected
            } else {
                ph2d_editor_core::widget::RowHighlight::None
            },
            bleed,
        );
        // The eye sits at the row's right edge and swallows its own clicks; the
        // rest of the row selects. Register the row FIRST so the eye's rect, which
        // is registered after, wins the overlap.
        if loaded {
            ctx.hit_index.register(AEDIT_FX_STAGES[i], row);
        }
        let tone = if !enabled {
            ColorToken::Text2
        } else if i == sel {
            ColorToken::Accent
        } else {
            text_tone(loaded)
        };
        let fs = TypeToken::Xs.px();
        paint_text(
            ctx.text_system,
            ctx.scene,
            &format!("{}. {name}", i + 1),
            x + Spacing::Sm.px(),
            y + (stage_h - fs) * 0.5,
            fs,
            w,
            resolve(tone, ctx.theme),
        );
        let eye = Rect::new(x + w - stage_h, y, stage_h, stage_h);
        let glyph = if enabled {
            IconId::Eye
        } else {
            IconId::EyeClosed
        };
        icon_button(eye, glyph, loaded, AEDIT_FX_STAGE_ONS[i], ctx);
        y += stage_h + ph2d_tokens::list_row_gap_px();
    }
    // ⛔ **A lista acaba com o vão de um controlo, e o `Bypass` NÃO lhe encosta** (report do dono,
    //    2026-09-07): colada a um botão, a última linha da lista volta a ler-se como um botão —
    //    que é exactamente o defeito que esta wave veio curar.
    y + ph2d_tokens::control_gap_px()
}

/// `Bypass` (global A/B) over `Apply | Cancel`. Bypass mutes the whole chain so the
/// dry clip sounds and shows, without losing it — the fastest before/after there is.
/// Apply turns exactly the buffer you heard into one undo step; Cancel drops it.
/// Both are only meaningful while something is auditioning.
fn paint_commit_row(y: f32, x: f32, w: f32, loaded: bool, row_h: f32, ctx: &mut Ctx) -> f32 {
    let auditioning = snapshot::fx_auditioning();
    let bypassed = snapshot::fx_bypass();
    // ⭐⭐⭐ **Os três são UM corpo** (report do dono, 2026-09-07: *«os botões abaixo deveriam ser
    //    do mesmo grupo juntos»*). ⚠️ **A wave 20b separou-os e a 20c pôs o `Bypass` na lista — as
    //    duas leituras estavam erradas, e o que as gerou foi eu procurar o sítio dele pela FUNÇÃO
    //    («é um estado, não uma ordem») em vez de pela SUPERFÍCIE.** Estes três partilham a mesma
    //    faixa de acção no fim da secção, e é isso que o olho lê; a lista acima é outra superfície,
    //    e o que os separa é o vão que ela agora deixa.
    //
    // Bypass mutes the whole chain so the dry clip sounds and shows, without losing it — the
    // fastest before/after there is. Apply turns exactly the buffer you heard into one undo step;
    // Cancel drops it. Apply is dimmed while bypassed: what sounds is the dry clip, so committing
    // would land nothing.
    let block = ph2d_editor_core::widget::block_cells(Rect::new(x, y, w, 0.0), &[1, 2], row_h);
    toggle_in_group(
        block[0][0].0,
        block[0][0].1,
        "Bypass",
        bypassed,
        auditioning,
        AEDIT_FX_BYPASS,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    for (i, (label, on, id)) in [
        ("Apply", loaded && !bypassed, AEDIT_FX_APPLY),
        ("Cancel", auditioning, AEDIT_FX_CANCEL),
    ]
    .into_iter()
    .enumerate()
    {
        button_in_group(
            block[1][i].0,
            label,
            on,
            id,
            block[1][i].1,
            ctx.scene,
            ctx.text_system,
            ctx.theme,
            ctx.hit_index,
        );
    }
    y + ph2d_editor_core::widget::grid_height(2, row_h) + ph2d_tokens::control_gap_px()
}

/// A frameless icon button. Disabled ones are dimmed and — crucially — do **not**
/// register a hit rect, so they cannot be clicked (2026-07-09 audit).
fn icon_button(rect: Rect, glyph: IconId, enabled: bool, id: NodeId, ctx: &mut Ctx) {
    // ⚠️ **A wave própria que o comentário anterior nomeava ACONTECEU (2026-08-15).** Ele dizia que
    //    a rack «pinta sem store» e que estes botões estavam «registados, clicáveis e inertes sob o
    //    rato desde que nasceram» — verdade, e a causa não era o pintor: os ids sempre estiveram
    //    registados como `InteractiveState::Button` no `populate`, então o store SABIA o hover e
    //    ninguém perguntava. O `ClippedHits` que este painter já recebia passou a carregar o store.
    //
    // ⚠️ **`Disabled` continua a ser DURO e vem primeiro:** um botão desactivado não regista hit,
    //    mas o `state` guardado pode ter ficado `Hovered` do quadro em que ele ainda estava vivo.
    let state = if enabled {
        ctx.hit_index.visual(id)
    } else {
        (ButtonState::Disabled, ph2d_editor_core::motion::SETTLED)
    };
    paint_icon_button(
        rect,
        IconGlyph::Builtin(glyph),
        IconButtonStyle::Plain,
        state,
        ctx.scene,
        ctx.theme,
    );
    if enabled {
        ctx.hit_index.register(id, rect);
    }
}

/// Foreground tone for text that dims with the clip's presence.
fn text_tone(loaded: bool) -> ColorToken {
    if loaded {
        ColorToken::Text1
    } else {
        ColorToken::Text2
    }
}
