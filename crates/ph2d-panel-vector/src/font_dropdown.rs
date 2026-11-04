//! The Font dropdown popover — a scrollable family list where **each row is drawn
//! in that family's own outline** (a real style preview, like Figma / Photoshop).
//!
//! Split from `paint_modes.rs` (panel LOC cap) and painted in a **deferred pass**
//! (after every body section) so the floating list lands on top. The chip's
//! open/close + light-dismiss + wheel-scroll come free from the generic `Dropdown`
//! dispatch (`InteractiveState::Dropdown`); the list, its scroll and its hits are the
//! foundation door's (`panel::paint_dropdown_list`) — this module only paints a ROW.
//!
//! The family list + the `VariableFont`s live on the shell; it pre-renders each
//! name into a `FontPreview.outline` (em-normalised, y-up) and publishes it. Here
//! we place that outline into the row rect with an `Affine` (scale + y-flip). When
//! there are no previews yet the popover asks the shell to build them
//! (`request_font_previews`) and shows a "Loading…" row for that one frame.

use crate::ids;
use crate::state;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, paint_text_centered, resolve};
use ph2d_editor_core::panel::{PaintCtx, paint_dropdown_list_measured};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, PopoverRow, resolve_opaque};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Color, Fill, VectorScene};

/// Glyph cap size as a fraction of the row height (leaves headroom for ascenders /
/// descenders so a preview never touches the neighbouring row).
const GLYPH_PX_RATIO: f64 = 0.62; // LITERAL-PX-OK: preview glyph size ÷ row height (list metric)
/// Baseline drop below the row's vertical centre, as a fraction of the glyph size
/// (optically centres the x-height, which sits above the baseline).
const GLYPH_BASELINE_RATIO: f64 = 0.34; // LITERAL-PX-OK: baseline offset ÷ glyph size (optical centring)

/// Paint the open font dropdown popover for `chip_rect` by the foundation door (only invoked when
/// the chip is open). With no previews yet it asks the shell for them and shows ONE «Loading…» row,
/// unselected.
pub(crate) fn paint(ctx: &mut PaintCtx, chip_rect: Rect, theme: Theme) {
    let n = state::with_font_previews(<[state::FontPreview]>::len);
    if n == 0 {
        // Lazy build: the shell scans + parses the system fonts only now, on the
        // first open — never on Text-mode entry.
        state::request_font_previews();
    }
    let options: Vec<DropdownOption<usize>> = (0..n.max(1))
        .map(|i| DropdownOption::new(ids::vector_text_font_option_id(i), i, String::new()))
        .collect();
    let mut dd = Dropdown::new(crate::ids::VECTOR_TEXT_FONT_DD, "", options).open(true);
    if n > 0 {
        dd.select(selected_index());
    }
    // A lista alta não sobe acima do topo do painel: as barras do topo pintam-se depois dela e tapavam-na.
    let mut region = ctx.layout.popover_region();
    if let Some(p) = ctx
        .host
        .store()
        .panel_rect(ph2d_editor_core::ids::VECTOR_PANEL)
    {
        let fundo = region.y + region.h;
        region.y = region.y.max(p.y);
        region.h = (fundo - region.y).max(0.0);
    }
    let mut linha = |scene: &mut VectorScene, text: &mut TextSystem, row: PopoverRow| {
        if n == 0 {
            paint_text_centered(
                text,
                scene,
                tr("panel.vector.text.loading_fonts"),
                row.rect,
                TypeToken::Base.px(),
                resolve(ColorToken::Text2, theme),
            );
        } else {
            paint_row(scene, text, row, theme);
        }
    };
    let largura = largura_pintada(ctx.text_system, chip_rect.h);
    paint_dropdown_list_measured(
        ctx,
        &dd,
        chip_rect,
        region,
        theme,
        Some(&mut linha),
        largura,
    );
}

/// A largura do nome mais largo COMO SE PINTA: no contorno dele, ou na fonte da interface se o contorno
/// sai vazio. O rótulo da lista não serve: cada linha desenha-se noutra fonte.
fn largura_pintada(text: &mut TextSystem, row_h: f32) -> f32 {
    let px = f64::from(row_h) * GLYPH_PX_RATIO;
    state::with_font_previews(|previews| {
        previews.iter().fold(0.0_f32, |w, pv| {
            let nome = if pv.outline.elements().is_empty() {
                ph2d_editor_core::paint::label_column_width(
                    text,
                    TypeToken::Base.px(),
                    [pv.display.as_str()],
                )
            } else {
                (pv.advance_em * px) as f32
            };
            w.max(nome)
        })
    })
}

/// One visible family row: the name in its own outline (falling back to the UI font when the
/// outline is empty). The selected highlight is the list's.
fn paint_row(scene: &mut VectorScene, text_system: &mut TextSystem, row: PopoverRow, theme: Theme) {
    let r = row.rect;
    // `resolve_opaque` conhece a tinta do cartão: sobre o realce dele, o `AccentFg` a secas não se lia.
    let color = resolve_opaque(row.ink(), theme);
    let pad = Spacing::Md.px();
    state::with_font_previews(|previews| {
        let Some(pv) = previews.get(row.index) else {
            return;
        };
        if pv.outline.elements().is_empty() {
            // No self-name glyphs (rare) — show the display name in the UI font.
            paint_text(
                text_system,
                scene,
                &pv.display,
                r.x + pad,
                r.y + (r.h - TypeToken::Base.px()) * 0.5,
                TypeToken::Base.px(),
                (r.w - pad * 2.0).max(1.0),
                color,
            );
        } else {
            // A lista não passa da janela: um nome mais largo do que ela corta-se na linha.
            let cabe = (pv.advance_em * f64::from(r.h) * GLYPH_PX_RATIO) as f32 + pad <= r.w;
            if !cabe {
                scene.push_clip(&ph2d_editor_core::paint::rect_to_vello(r));
            }
            fill_name_outline(scene, pv, r, pad, color);
            if !cabe {
                scene.pop_layer();
            }
        }
    });
}

/// Place the em-normalised (y-up) name outline into the row: scale to a pixel cap
/// size, flip Y, and translate to the baseline. Filled `NonZero` (glyph winding).
fn fill_name_outline(
    scene: &mut VectorScene,
    pv: &state::FontPreview,
    r: Rect,
    pad: f32,
    color: Color,
) {
    let px = r.h as f64 * GLYPH_PX_RATIO;
    let baseline = r.y as f64 + r.h as f64 * 0.5 + px * GLYPH_BASELINE_RATIO;
    let x0 = r.x as f64 + pad as f64;
    let affine = Affine::translate((x0, baseline)) * Affine::scale_non_uniform(px, -px);
    scene.inner_mut().fill(
        Fill::NonZero,
        affine,
        &Brush::Solid(color),
        None,
        &pv.outline,
    );
}

/// Index of the currently-chosen family in the published preview list (0 = the
/// bundled default when nothing matches).
fn selected_index() -> usize {
    let cur = state::current_text_font();
    state::with_font_previews(|previews| {
        previews
            .iter()
            .position(|pv| Some(pv.display.as_str()) == cur.as_deref())
            .unwrap_or(0)
    })
}

/// **Uma linha de família escolhida no popover aberto.**
///
/// ⚠️ O light-dismiss genérico **não dispara aqui** — o clique é DENTRO do popover —, então sem
/// este fecho manual a lista fica pendurada sobre a seção depois de escolher, e o artista tem de
/// a dispensar à mão. O índice segue por `SelectOption`; a shell o resolve para uma família e
/// re-gera o texto.
///
/// Mora ao lado do paint do popover (e não no `apply_event`) porque é o MESMO assunto e pelo teto
/// de 200 LOC por função dos painéis — o padrão de delegação das pontas e do conector.
pub(crate) fn apply_event(
    host: &mut dyn ph2d_editor_core::panel::PanelHostInternal,
    ev: ph2d_editor_core::interaction::WidgetEvent,
) -> bool {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::tool::PanelEvent;
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(i) = state::font_option_index(id) else {
        return false;
    };
    if let Some(InteractiveState::Dropdown {
        open,
        selected_index,
        ..
    }) = host.store_mut().get_mut(crate::ids::VECTOR_TEXT_FONT_DD)
    {
        *open = false;
        *selected_index = Some(i);
    }
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            crate::ids::VECTOR_TEXT_FONT_DD,
            i.to_string(),
        )));
    true
}
