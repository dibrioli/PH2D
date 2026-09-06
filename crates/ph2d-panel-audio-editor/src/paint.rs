//! Audio Editor panel paint — **right-docked in the shared Inspector slot**
//! (mirror of the Audio Mixer / Sprite Inspector dock pattern), NOT a floating
//! panel. Reads `ctx.layout.inspector` for its rect and registers the shared
//! `INSP_*` drag/resize handles so it moves/resizes with the dock slot.
//!
//! Compact controls only: a clip readout (name · position / duration), a
//! transport (Play/Pause · Stop · Loop) and Load / Export. The spacious waveform
//! + timeline are the separate floating overlay on the canvas.

use crate::state::AudioEditorState;
use crate::{AEDIT_CLOSE, AEDIT_FX_PARAMS, AEDIT_NAME, AEDIT_PANEL, AudioEditorPanel, snapshot};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::motion;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface_floating,
    paint_panel_title, panel_close_button_rect,
};
use ph2d_editor_core::widget::{
    AUDIO_EDITOR_SCROLLBAR_ID, ButtonState, SCROLLBAR_W, TextInputState, paint_scrollbar,
    scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color as VelloColor, VectorScene};

pub(crate) use crate::clipped_hits::ClippedHits;

/// A linha do transporte — a linha de painel de sempre, e por isso **o token**.
///
/// ⚠️ Era `28.0` escrito à mão: o valor que o `chrome.row-h` tinha. Ficou para trás no dia em que
/// o dono pediu linhas mais compactas (`28 → 24`, 2026-09-06) e o painel de áudio teria linhas
/// mais altas que todos os outros. *Uma cópia de um número do design system é uma divergência
/// com data marcada.*
pub(crate) const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

pub(crate) fn paint(_state: &mut AudioEditorState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(AudioEditorPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(AEDIT_PANEL);
        return;
    }
    // ⭐⭐ **ELE OCUPA A COLUNA, como todo painel docado** (`Slot::RightTop`).
    //
    // ⛔⛔ Até 2026-08-30 ele encaixava-se **a oeste** dela (`insp.x − 240 − gap`) para poder estar
    // aberto ao lado do Audio Mixer. Isso é uma **segunda coluna da direita**, e o modelo de áreas
    // recusa-a por aritmética: duas colunas por lado são `89,6 %` da largura do alvo de 1366.
    // Medido, ele publicava `168 480 px²` **por cima da área de desenho**.
    //
    // ⭐ O que ele queria — *MIX e WAVE abertos ao mesmo tempo* — é agora a regra 1 do modelo:
    // dois ocupantes do mesmo encaixe são **ABAS** (`screens::hero::slot_tabs`), e a faixa delas
    // já saiu deste rect antes de ele chegar aqui.
    let rect = ctx.slot;
    let theme = ctx.host.theme();
    ctx.host.store_mut().set_panel_rect(AEDIT_PANEL, rect);

    // Opaque backing (the glass surface would bleed the canvas/panel behind it).
    fill_rounded_rect(
        ctx.scene,
        rect,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    paint_panel_surface_floating(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    let title_size = paint_panel_title(
        rect,
        "Audio Editor",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        AEDIT_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    // The body (transport + edit ops + the whole effects rack) overflows the dock
    // height, so it is clipped + scrolled. Reserve the scrollbar rail unconditionally
    // — a layout that reflows the instant the content overflows is worse than one
    // that always leaves the rail empty.
    let pad = Spacing::Lg.px();
    let rail = SCROLLBAR_W + Spacing::Sm.px();
    let x = rect.x + pad;
    let w = (rect.w - pad * 2.0 - rail).max(1.0);
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let bottom_pad = Spacing::Lg.px();
    let body_h = (rect.y + rect.h - body_top - bottom_pad).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    // Read the scroll BEFORE the mutable store/hit borrows below.
    let scroll = ctx.host.store().panel_scroll(AEDIT_PANEL);
    let y = body_top - scroll;

    // Snapshot (shell → panel).
    let loaded = snapshot::loaded();
    let playing = snapshot::playing();
    let looping = snapshot::looping();
    let pos = snapshot::position_secs();
    let dur = snapshot::duration_secs();
    let undo_ok = snapshot::can_undo();
    let redo_ok = snapshot::can_redo();
    let has_sel = snapshot::has_selection();
    // Fold state, read BEFORE the mutable store/hit borrows below (same reason as the
    // scroll above): the paint pass cannot reach the store once `hit_index` holds it.
    let open = crate::paint_sections::SECTIONS.map(|id| {
        (
            !ctx.host.store().is_collapsed(id),
            ctx.host.store().section_open_live(id),
        )
    });

    sync_widget_buffers(ctx);

    // Read the name field's live buffer for painting (cloned so the scene borrow
    // below is free of the store).
    // ⚠️ Lido AQUI, junto do estado: mais abaixo o `hit_index` já pegou o `ctx.host`
    // emprestado como mutável.
    let name_hover_t = ctx.host.store().hover_live(AEDIT_NAME);
    let (name_state, name_text, name_caret, name_anchor) = match ctx.host.store().get(AEDIT_NAME) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };

    let (scene, text_system) = (&mut *ctx.scene, &mut *ctx.text_system);
    // Anything scrolled past the body's top/bottom is hidden — and, via
    // `ClippedHits`, unclickable.
    scene.push_clip(&rect_to_vello(body_rect));
    // ⚠️ **O empréstimo CONJUNTO existe para isto** (`PanelHostInternal::store_and_hit_index_mut`,
    // e o doc dele diz-o): sem ele o `hit_index_mut` tranca o host e o corpo fica sem forma de
    // perguntar como um widget se pinta — que foi exactamente por que este painel nasceu inerte.
    let (store, hits) = ctx.host.store_and_hit_index_mut();
    let hit_index = &mut ClippedHits::new(store, hits, body_rect);

    // The body is a stack of collapsible SECTIONS — see `paint_sections`.
    let body = crate::paint_sections::Body {
        open,
        loaded,
        undo_ok,
        redo_ok,
        has_sel,
        transport: crate::paint_sections::Transport {
            loaded,
            playing,
            looping,
            pos,
            dur,
        },
        name: crate::paint_sections::NameBox {
            state: name_state,
            hover_t: name_hover_t,
            text: name_text,
            caret: name_caret,
            anchor: name_anchor,
        },
    };
    let final_y =
        crate::paint_sections::paint_body(y, x, w, &body, scene, text_system, theme, hit_index);

    // Total scrollable height in body-local coords (undo the `- scroll` offset).
    let content_h = (final_y + scroll) - body_top + bottom_pad;
    scene.pop_layer();
    paint_scroll_chrome(ctx, body_rect, scroll, content_h, body_h, theme);

    // Re-register the close button last so body widgets can't shadow it.
    ctx.host
        .hit_index_mut()
        .register(AEDIT_CLOSE, panel_close_button_rect(rect));
}

/// Push shell-owned values into the widget buffers the shared dispatch also writes.
/// Both guards exist to avoid fighting the user: overwrite the name box only when a
/// NEW clip loads (and never while it holds focus), and the parameter sliders only
/// when the PANEL moved them programmatically (kind switch, reset, add, select) —
/// never on a user drag.
fn sync_widget_buffers(ctx: &mut PaintCtx) {
    if let Some(name) = snapshot::clip_name_needs_sync()
        && ctx.host.store().focus_id() != Some(AEDIT_NAME)
    {
        if let Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) = ctx.host.store_mut().get_mut(AEDIT_NAME)
        {
            *state = TextInputState::Normal;
            text.clear();
            text.push_str(&name);
            *caret = text.len();
            *selection_anchor = None;
        }
        snapshot::mark_name_synced();
    }
    if let Some(norms) = snapshot::fx_sliders_need_sync() {
        for (i, id) in AEDIT_FX_PARAMS.iter().enumerate() {
            if let Some(InteractiveState::Slider { value, .. }) = ctx.host.store_mut().get_mut(*id)
            {
                *value = norms[i];
            }
        }
    }
}

/// The scrollbar (only when the content overflows), plus the `content_h`/`visible_h`
/// the wheel dispatcher needs to clamp against. Then clamp any leftover over-scroll —
/// the content shrinks whenever a chain stage is removed, and a stale offset would
/// leave the body parked past its own end.
fn paint_scroll_chrome(
    ctx: &mut PaintCtx,
    body_rect: Rect,
    scroll: f32,
    content_h: f32,
    body_h: f32,
    theme: Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        paint_scrollbar(
            body_rect,
            scroll,
            content_h,
            body_h,
            ctx.host.store().scrollbar_visual(AUDIO_EDITOR_SCROLLBAR_ID),
            ctx.scene,
            theme,
        );
        // The thumb sits on the rail OUTSIDE the body's widgets, and its drag is routed
        // by `scrollbar_panel_for_id`, not by an `InteractiveState` — so it registers on
        // the raw hit index, unclipped.
        ctx.host
            .hit_index_mut()
            .register(AUDIO_EDITOR_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(AEDIT_PANEL, content_h);
    store.set_panel_visible_h(AEDIT_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(AEDIT_PANEL) > max_scroll {
        store.set_panel_scroll(AEDIT_PANEL, max_scroll);
    }
}

/// Seconds per minute — time-domain constant, not a UI metric.
const SECS_PER_MIN: f64 = 60.0; // LITERAL-PX-OK: seconds per minute (time math)

/// Format seconds as `m:ss.d` (one decimal), clamped at zero.
pub(crate) fn fmt_time(secs: f64) -> String {
    let s = secs.max(0.0);
    let m = (s / SECS_PER_MIN) as u64;
    let rem = s - (m as f64) * SECS_PER_MIN;
    format!("{m}:{rem:04.1}") // LITERAL-PX-OK: mm:ss.d time format spec, not a UI metric
}

/// **A cor de fundo de um botão deste painel, no eixo do hover.**
///
/// ⚠️ **A lei é a do CATÁLOGO; o que é do painel é só o tom de REPOUSO.** Os tokens quentes
/// (`BgElev` no hover, `AccentSoft` no press) e a *transição* saem exactamente de onde o
/// `widget::Button` os tira — [`motion::hover_axis`], a porta única —, então isto não é uma segunda
/// resposta a *«que cor tem um botão sob o rato?»*: é a mesma resposta com a superfície que este
/// painel já pinta.
///
/// ⚠️ **`enabled == false` é um estado DURO e sai antes do eixo.** Um botão desactivado não regista
/// hit, logo o ponteiro não o alcança — mas o `state` GUARDADO pode ter ficado `Hovered` do quadro
/// em que ele ainda estava vivo, e sem esta saída ele acenderia sozinho ao ser desactivado sob o
/// cursor.
///
/// ⚠️ **Em repouso o resultado é BYTE-IDÊNTICO ao que shipava:** `hover_axis` devolve `None` no
/// neutro ([`motion::SETTLED`]) e o chamador cai no token duro — que para `Normal` é o `Bg3` de
/// sempre.
fn action_bg(
    rest: ColorToken,
    hot: ColorToken,
    press: ColorToken,
    v: (ButtonState, f32),
    enabled: bool,
    theme: Theme,
) -> VelloColor {
    let token = if !enabled {
        rest.resolve(theme)
    } else {
        let (state, t) = v;
        if state == ButtonState::Pressed {
            press.resolve(theme)
        } else {
            let soft = matches!(state, ButtonState::Normal | ButtonState::Hovered);
            motion::hover_axis(soft, t, Some(rest.resolve(theme)), Some(hot.resolve(theme)))
                .unwrap_or_else(|| {
                    if state == ButtonState::Hovered {
                        hot.resolve(theme)
                    } else {
                        rest.resolve(theme)
                    }
                })
        }
    };
    VelloColor::from_rgba8(token.r, token.g, token.b, token.a) // LITERAL-COLOR-OK: token-bridge — `token` já é ColorToken-resolvido
}

/// A labeled action button: `Bg3` + `Text1` when enabled, dimmed to `Text2` when
/// not. Shared with the effects rack section (`paint_fx`).
///
/// A **disabled button does not register a hit rect**, so it cannot be clicked.
/// It used to register regardless ("disabled is a visual hint only"), which made
/// every dimmed control silently live: clicking the dimmed `Silence` with no
/// selection fell through to `target()` and zeroed the WHOLE clip (2026-07-09
/// audit). The panel dims and the seam refuses — two layers, since a dim alone is
/// cosmetic.
///
/// ⚠️ **Ele reage ao ponteiro desde 2026-08-15, e o que faltava não era o pintor:** os ids deste
/// painel sempre foram registados como `InteractiveState::Button` no `populate`, então o store
/// SABIA o hover — ninguém perguntava. Ver [`action_bg`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn button(
    rect: Rect,
    label: &str,
    enabled: bool,
    id: NodeId,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) {
    let fg = if enabled {
        ColorToken::Text1
    } else {
        ColorToken::Text2
    };
    let bg = action_bg(
        ColorToken::Bg3,
        ColorToken::BgElev,
        ColorToken::AccentSoft,
        hit_index.visual(id),
        enabled,
        theme,
    );
    fill_rounded_rect(
        scene,
        rect,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
        bg,
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
    if enabled {
        hit_index.register(id, rect);
    }
}

/// **A que FAMÍLIA do catálogo um toggle pertence** — repouso, quente, pressionado, texto.
///
/// ⚠️ **É uma `fn` e não um `match` inline por uma razão medida:** enquanto a escolha vivia dentro
/// do [`toggle`], a mutação que punha o toggle ACESO no eixo do solto **sobrevivia à suíte
/// inteira** — os gates da lei de cor chamavam [`action_bg`] com os tokens à mão e nunca viam qual
/// família o pintor escolhe. Separada, a escolha é ela própria afirmável.
///
/// ⚠️ **Desactivado colapsa os três tons no repouso**, e não é preguiça: um toggle inerte mantém a
/// superfície (um estado engatado-mas-inerte continua a LER como engatado) e perde só o contraste
/// do texto — a regra que já shipava, agora escrita onde se pode testar.
fn toggle_tokens(active: bool, enabled: bool) -> (ColorToken, ColorToken, ColorToken, ColorToken) {
    match (active, enabled) {
        (true, true) => (
            ColorToken::Accent,
            ColorToken::AccentHover,
            ColorToken::AccentPress,
            ColorToken::AccentFg,
        ),
        (false, true) => (
            ColorToken::Bg3,
            ColorToken::BgElev,
            ColorToken::AccentSoft,
            ColorToken::Text1,
        ),
        (_, false) => (
            ColorToken::Bg3,
            ColorToken::Bg3,
            ColorToken::Bg3,
            ColorToken::Text2,
        ),
    }
}

/// A labeled toggle button: `Accent` tint + `AccentFg` when engaged, else `Bg3`
/// + `Text1`.
///
/// Like [`button`], a **disabled toggle registers no hit rect** — it keeps its
/// surface (so an engaged-but-inert state still reads as engaged) but loses its
/// text contrast, and cannot be clicked.
///
/// ⚠️ **Engatado e solto sobem eixos DIFERENTES, e é o que o catálogo faz:** solto é a família
/// `Default` (`Bg3 → BgElev`), engatado é a família `Accent` (`Accent → AccentHover`, press
/// `AccentPress`). Usar o eixo do solto num toggle aceso deixaria o hover a ESCURECER a peça mais
/// clara da tela.
#[allow(clippy::too_many_arguments)]
pub(crate) fn toggle(
    rect: Rect,
    label: &str,
    active: bool,
    enabled: bool,
    id: NodeId,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) {
    let (rest, hot, press, fg) = toggle_tokens(active, enabled);
    let bg = action_bg(rest, hot, press, hit_index.visual(id), enabled, theme);
    fill_rounded_rect(
        scene,
        rect,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
        bg,
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
    if enabled {
        hit_index.register(id, rect);
    }
}

#[cfg(test)]
#[path = "paint_tests.rs"]
mod paint_tests;
