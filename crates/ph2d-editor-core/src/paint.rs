//! [`Paint`] trait + impls — widget tree → `vello::Scene` (M11+M12).
//!
//! This is the **lowering pass** that takes the data-only widget
//! representation (zones, panels, buttons, toasts) and emits Vello
//! draw commands. The shell calls each top-level widget's `paint`
//! once per frame, then hands the resulting `VectorScene` to
//! `ph2d_render::VelloPass` which composites onto the wgpu surface.
//!
//! ⚠️ The toast queue's `impl Paint` lives in `toast.rs`, next to the queue (auditoria A10,
//! 2026-09-12): painting it here made the foundation's painter depend on `toast` and `progress`.
//!
//! **Color resolution** goes through `ph2d_tokens::ColorToken::resolve(theme)`
//! so swapping Dark↔Light flips the entire UI.
//!
//! **Text rendering** uses [`paint_text`] / [`paint_text_centered`] —
//! parley layout → `vello::Scene::draw_glyphs` per parley `GlyphRun`.
//! Tabs / actions / button labels / toast messages all render visibly
//! with the system sans-serif fallback chain.

// `FloatingPanel` paint impl retired 2026-05-17 — struct itself
// remains in `crate::floating_panel` for tool event dispatch but no
// longer needs a paint helper import here.
// `paint_color_swatch`, `paint_radio_group`, `paint_slider`, and
// `paint_toggle` were used inside `impl Paint for FloatingPanel`
// (retired 2026-05-17). They still ship from `crate::widget` for
// direct consumers (Inspector, Grid Snap panel, etc.); paint.rs just
// no longer references them.
use crate::zones::{Layout, Rect, Zone};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{Color as TokenColor, ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Circle, Color, Fill, Rect as VelloRect, Stroke, VectorScene};

/// Per-frame paint context. Built by the shell, threaded through
/// every widget's [`Paint::paint`] call.
pub struct PaintCtx<'a> {
    pub theme: Theme,
    /// Window / surface rect in device pixels. Widgets that don't
    /// own their own anchoring (toasts, panels with `Free` anchor)
    /// position themselves relative to this.
    pub viewport: Rect,
    /// Owned long-lived parley state. v1 doesn't touch this; here
    /// to keep the trait signature stable when text lands.
    pub text: &'a mut TextSystem,
}

pub trait Paint {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx);
}

#[path = "paint_text.rs"]
mod text;

#[path = "paint_rounded.rs"]
mod rounded;
pub use rounded::{fill_rounded_rect, fill_rounded_rect_radii, stroke_rect, stroke_rounded_rect};

/// Apply the snap strategy to a glyph X. `None` returns `x` unchanged
/// (preserves subpixel kerning); `Half` snaps to 0.5 px (preserves ~50 %
/// of kerning); `Full` snaps to 1 px integer (full pixel alignment, no
/// subpixel kerning). Trade-off documented on [`ph2d_tokens::SnapX`].
fn snap_x_apply(x: f32, snap: ph2d_tokens::SnapX) -> f32 {
    match snap {
        ph2d_tokens::SnapX::None => x,
        ph2d_tokens::SnapX::Half => (x * 2.0).round() * 0.5,
        ph2d_tokens::SnapX::Full => x.round(),
    }
}

#[path = "paint_icons.rs"]
mod icons_paint;
pub use icons_paint::{paint_icon, paint_icon_path, paint_icon_rotated};
pub use text::{
    paint_text, paint_text_block, paint_text_elided, paint_text_rotated_ccw, paint_text_title,
    paint_text_title_elided,
};

/// Convert a `ph2d_tokens::Color` (sRGB 8-bit + alpha) to Vello's
/// `peniko::Color`. Vello stores its native palette as `[r,g,b,a]`
/// f32 in linear sRGB, but `from_rgba8` does the linearization for
/// us so we keep token semantics intact.
pub fn token_to_vello(tc: TokenColor) -> Color {
    Color::from_rgba8(tc.r, tc.g, tc.b, tc.a)
}

pub fn resolve(token: ColorToken, theme: Theme) -> Color {
    token_to_vello(token.resolve(theme))
}

/// Convert our pixel `Rect` to a Vello `Rect` (which uses f64).
pub fn rect_to_vello(r: Rect) -> VelloRect {
    VelloRect::new(
        r.x as f64,
        r.y as f64,
        (r.x + r.w) as f64,
        (r.y + r.h) as f64,
    )
}

// ⭐ **O que a shell publica por quadro** (escala de raio · estilo das linhas · aparência · texto)
//    mudou-se para o irmão [`crate::published`] pelo tecto de 700 LOC; os caminhos `paint::…`
//    continuam a valer por este re-export.
pub use crate::published::{
    radius_scale, set_radius_scale, set_slider_style, set_text_rendering, set_ui_look,
    slider_style, text_rendering, ui_is_redesign, ui_look,
};

/// ⭐⭐ **A MOLDURA de um controlo, pela porta do TEMA** — o sítio único que substitui o
/// `stroke_rounded_rect` de repouso/estado nos pintores de widget.
///
/// Clássico: traça `classic_w` × `classic_colour`, byte-idêntico ao que o pintor sempre fez.
/// Moderno: traça o que a tabela de estados diz — nada em repouso (a pele plana), o anel de foco,
/// a moldura de erro. Ver [`ph2d_tokens::visuals::frame`].
///
/// ⚠️ A cor clássica chega já misturada no eixo do hover (é um `Color` do Vello): é por isso que a
/// porta recebe a cor pronta em vez de um token — o pintor não perde a animação que já tinha.
#[allow(clippy::too_many_arguments)]
pub fn stroke_frame(
    scene: &mut VectorScene,
    rect: Rect,
    radius: f32,
    theme: Theme,
    feel: ph2d_tokens::visuals::Feel,
    classic_w: f32,
    classic_colour: Color,
) {
    match ph2d_tokens::visuals::frame(theme, feel) {
        ph2d_tokens::visuals::Frame::Classic => {
            stroke_rounded_rect(scene, rect, radius, classic_w, classic_colour);
        }
        ph2d_tokens::visuals::Frame::Modern(s) if s.is_visible() => {
            stroke_rounded_rect(scene, rect, radius, s.width, token_to_vello(s.color));
        }
        ph2d_tokens::visuals::Frame::Modern(_) => {}
    }
}

/// ⭐⭐ **O ANEL POR PREENCHIMENTO, pela porta do TEMA** — a irmã do [`stroke_frame`] para a
/// moldura que **não é um traço**: um rectângulo maior na cor da borda, com o conteúdo pintado
/// por cima com um recuo. É a forma que a amostra de cor usa, e **o censo do traço não a vê**
/// (wave 5 do redesenho, 2026-09-05).
///
/// Devolve `Some((recuo, cor))` quando há anel, `None` quando o tema não quer nenhum. Clássico:
/// sempre o par que o pintor sempre usou. Moderno: o traço da tabela — logo **`None` em
/// repouso**, e o anel de foco (2 px no acento) quando o controlo está focado.
///
/// ⚠️ **O recuo é a LARGURA do anel**, e é por isso que ele volta daqui em vez de ser escolhido
/// no pintor: um anel de 2 px desenhado com recuo de 1 lê-se como meia moldura.
#[must_use]
pub fn fill_ring(
    theme: Theme,
    feel: ph2d_tokens::visuals::Feel,
    classic_w: f32,
    classic_colour: Color,
) -> Option<(f32, Color)> {
    match ph2d_tokens::visuals::frame(theme, feel) {
        ph2d_tokens::visuals::Frame::Classic => Some((classic_w, classic_colour)),
        ph2d_tokens::visuals::Frame::Modern(s) if s.is_visible() => {
            Some((s.width, token_to_vello(s.color)))
        }
        ph2d_tokens::visuals::Frame::Modern(_) => None,
    }
}

/// ⭐⭐⭐ **A TINTA DE REPOUSO de um controlo com corpo, pela porta do TEMA** — a irmã do
/// [`stroke_frame`] para o PREENCHIMENTO.
///
/// ⛔⛔ **Report do dono, 2026-09-14, com foto da secção *Sprite Sheet*: *«Checkbox invisível»*.**
/// Na foto a caixa *Centered*, MARCADA, lê-se (é `Accent`); as de *Flip H* e *Flip V*,
/// desmarcadas, não existem. Medido: uma caixa desmarcada enche com `ColorToken::Bg1`, que é
/// **exactamente** o token de um cartão de secção, e num tema moderno a moldura de repouso é
/// ZERO ⇒ `0/255` e sem contorno.
///
/// ⚠️⚠️ **E o comentário ao lado do código já DESCREVIA a lei certa** — *«num tema moderno a caixa
/// é plana (marcada = acento cheio, desmarcada = **um degrau abaixo do painel**)»* — sobre um
/// código que pintava o cartão. *Um doc que descreve a lei que o código não implementa faz o
/// defeito parecer auditado.*
///
/// ⛔ **Não era um defeito da checkbox: eram CINCO pintores.** O censo do mesmo dia achou
/// `number_input` · `text_area` · `checkbox` · `combobox` · `dropdown` · `radio_group`, todos a
/// encher `Bg1` em repouso e todos a pedir a moldura ao tema — que num tema moderno não traça
/// nada. *Seis respostas à mesma pergunta divergem quando uma superfície se mexe, e uma
/// superfície mexeu-se em 2026-09-05, quando o painel desceu para o cartão se ler.*
///
/// # A escada
///
/// - **Clássica** (há moldura de repouso): devolve o token que o pintor sempre usou —
///   **byte-idêntico**, e ali o corpo lê-se pela borda.
/// - **Moderna** (sem moldura): `Feel::Rest`/`Disabled` devolvem o
///   [`ph2d_tokens::visuals::Chrome::field_fill`] (um degrau abaixo da superfície mais funda em
///   que um controlo assenta); `Hovered`/`Active` devolvem o corpo quente da tabela de estados,
///   senão um controlo cujo repouso afundou perderia o eixo do hover que tinha.
///
/// ⚠️ **Devolve um [`ph2d_tokens::Color`] e não um `Color` do Vello**, de propósito: é o que o
/// [`crate::motion::hover_axis`] consome, e os pintores desta família interpolam o repouso.
#[must_use]
pub fn body_fill(
    theme: Theme,
    feel: ph2d_tokens::visuals::Feel,
    classic: ph2d_tokens::ColorToken,
) -> ph2d_tokens::Color {
    use ph2d_tokens::visuals::Feel;
    let chrome = ph2d_tokens::visuals::Chrome::of(theme);
    if chrome.field_border.is_visible() {
        return classic.resolve(theme);
    }
    match feel {
        Feel::Hovered | Feel::Active => ph2d_tokens::visuals::Widgets::of(theme).hovered.bg_fill,
        _ => chrome.field_fill,
    }
}

/// ⭐⭐⭐ **A tinta de um controlo que assenta num CAMPO — e não num cartão.**
///
/// ⛔⛔⛔ **Report do dono, 2026-09-15, com foto:** *«O Checkbox desmarcado é invisível»*. É a
/// MESMA queixa de 2026-09-14 (*«Checkbox invisível»*, que fez nascer a [`body_fill`]) **um degrau
/// mais fundo — e quem moveu o degrau foi a wave da véspera.**
///
/// A [`body_fill`] responde *«um corpo que assenta num CARTÃO»* e devolve
/// [`ph2d_tokens::visuals::Chrome::field_fill`] em repouso: um degrau abaixo da superfície mais
/// funda. Isso resolvia a queixa **enquanto a marca assentava no cartão**. Em 2026-09-15 ela
/// mudou-se para dentro de uma CAIXA DE CAMPO — cujo fundo é exactamente `field_fill`. Medido:
///
/// | tema | a caixa | a marca desmarcada | distância |
/// |---|---|---|---|
/// | Dark | `#090909` | `#090909` | **`0`/255** ⛔ |
/// | Gray | `#121212` | `#121212` | **`0`/255** ⛔ |
/// | Light | `#DBDBDB` | `#DBDBDB` | **`0`/255** ⛔ |
/// | Oled | `#000000` | `#000000` | `0`, e ali **a moldura lê-a** (*Draw Extra Borders*) |
///
/// ⇒ *o mesmo pixel duas vezes*, e num tema moderno a moldura de repouso de uma marca mede `0`.
/// **A marcada continuava a ler-se porque é `Accent`** — foi por isso que o report é sobre metade
/// das caixas, exactamente como no dia anterior.
///
/// ⚠️⚠️ **A lição, e ela é do `CLAUDE.md` §0.0:** *quem move a superfície em que uma nota assenta
/// tem de reconferir a nota.* A cura de 14/09 estava calibrada contra o CARTÃO e ficou escrita como
/// se fosse absoluta.
///
/// # A escada
///
/// - **Clássica** (há moldura de repouso): o token que o pintor sempre usou — **byte-idêntico**.
/// - **Moderna**: a família de estados de um WIDGET
///   ([`ph2d_tokens::visuals::Widgets`]), que é o vocabulário certo para uma peça interactiva
///   pousada noutra superfície — e cujo `hovered` a [`body_fill`] já usava na ponta quente. *As
///   duas pontas do eixo passam a ser a mesma família.*
///
/// Medido contra a caixa (barra [`ph2d_tokens::derive::SURFACE_STEP`] = `10`/255):
/// `inactive` dá **`32`** (Dark) · **`43`** (Gray) · **`11`** (Light); `noninteractive` dá `18` ·
/// `22` · `26`. ⛔ No OLED tudo satura a zero e quem separa é a moldura, que é a cláusula que o
/// `a_field_is_never_the_colour_of_what_it_sits_on` já declara por escrito.
#[must_use]
pub fn on_field_fill(
    theme: Theme,
    feel: ph2d_tokens::visuals::Feel,
    classic: ph2d_tokens::ColorToken,
) -> ph2d_tokens::Color {
    use ph2d_tokens::visuals::Feel;
    let chrome = ph2d_tokens::visuals::Chrome::of(theme);
    if chrome.field_border.is_visible() {
        return classic.resolve(theme);
    }
    let w = ph2d_tokens::visuals::Widgets::of(theme);
    match feel {
        Feel::Hovered | Feel::Active => w.hovered.bg_fill,
        Feel::Disabled => w.noninteractive.bg_fill,
        _ => w.inactive.bg_fill,
    }
}

/// ⭐ **O RAIO de um controlo, pela porta do TEMA** — ver [`ph2d_tokens::visuals::radius`].
#[must_use]
pub fn frame_radius(theme: Theme, classic: f32) -> f32 {
    ph2d_tokens::visuals::radius(theme, classic)
}

/// Fill a circle centered at `(cx, cy)` with radius `r` — curve-editor control-
/// point handles (W4 §3) and any round dot. `r <= 0` is a no-op.
pub fn fill_circle(scene: &mut VectorScene, cx: f32, cy: f32, r: f32, color: Color) {
    if r <= 0.0 {
        return;
    }
    let circle = Circle::new((cx as f64, cy as f64), r as f64);
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &circle);
}

/// Stroke a polyline through `points` (round joins/caps) — a smooth curve drawn
/// through its sampled output (W4 §3), replacing the dense-dot fallback. Fewer
/// than two points is a no-op.
pub fn stroke_polyline(scene: &mut VectorScene, points: &[(f32, f32)], width: f32, color: Color) {
    if points.len() < 2 {
        return;
    }
    let mut path = BezPath::new();
    path.move_to((points[0].0 as f64, points[0].1 as f64));
    for &(x, y) in &points[1..] {
        path.line_to((x as f64, y as f64));
    }
    let stroke = Stroke::new(width as f64);
    scene
        .inner_mut()
        .stroke(&stroke, Affine::IDENTITY, color, None, &path);
}

/// Tool palette paint helper — for each `(rect, label, is_active)`
/// triple, draws a 36×36 icon chip in the TopRight zone:
///   - Active: AccentPrimary background + inverted text
///   - Hover (when state != Normal): SurfaceElevated
///   - Idle: Surface
///
/// Until a real icon set ships, the icon is just the first character
/// of the tool's label centered in the chip.
pub fn paint_tool_palette_icons(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    icons: &[(Rect, &str, bool)],
    theme: Theme,
) {
    for &(rect, label, is_active) in icons {
        let (bg, fg) = if is_active {
            (
                resolve(ColorToken::Accent, theme),
                resolve(ColorToken::AccentFg, theme),
            )
        } else {
            (
                resolve(ColorToken::Bg1, theme),
                resolve(ColorToken::Text1, theme),
            )
        };
        scene.fill_rect(rect_to_vello(rect), bg);
        // 1-px Border ring on inactive icons so they don't
        // disappear into the surrounding zone fill.
        if !is_active {
            let border_color = resolve(ColorToken::Border, theme);
            stroke_rect(scene, rect, 1.0, border_color);
        }
        let glyph = label.chars().next().unwrap_or('?').to_string();
        paint_text_centered(text_system, scene, &glyph, rect, 18.0, fg);
    }
}

/// Center `text` inside `rect` (horizontally + vertically).
/// ⭐⭐ **O que sobra da largura de uma caixa depois do RESPIRO das duas bordas.**
///
/// Enio, 2026-09-06, com duas fotos: *«algumas palavras ou mesmo os 3 pontos ficam muito próximos
/// da borda do botão. Deveria ter algum espaço.»* — `Surface Smo…` acabava colado à moldura.
///
/// ⚠️ **Ele só MORDE quando o texto não caberia à mesma**: a centragem é simétrica, logo deflacionar
/// os dois lados não move um rótulo que cabe — muda apenas o orçamento com que ele é elidido. *Um
/// recuo que só se paga no caso apertado é o único que não custa nada no caso comum.*
///
/// **O número é do modelo:** `base_margin · 2` = 8 px, a margem de conteúdo horizontal de um botão
/// no Godot 4.6 «Modern» (`theme_modern.cpp:289`) — e é o `Spacing::Md`, que a escada desta casa
/// já nomeia *«default inline padding»*. ⛔ Não é o `Button::padding()` (12 px): aquele é o recuo
/// de um botão que dimensiona a si próprio, e aqui a largura vem de fora.
fn label_budget(w: f32) -> f32 {
    (w - ph2d_tokens::Spacing::Md.px() * 2.0).max(1.0)
}

pub fn paint_text_centered(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    rect: Rect,
    font_size: f32,
    color: Color,
) {
    // ⚠️ **Centra o que vai ser PINTADO, não o que foi pedido.** O `paint_text` elide desde
    // 2026-09-06 (report do dono: a palavra que não cabe *«passa para baixo e some»*), e medir
    // aqui o texto INTEIRO punha um rótulo elidido fora do centro — pior, com `rect.w` como
    // orçamento de quebra o layout devolvia DUAS linhas e o `y` centrava-as, deixando a primeira
    // acima do topo da caixa. *Uma centragem que mede outra coisa do que se pinta é um deslocamento
    // com cara de arredondamento.*
    let shown = crate::text_elide::fit(text_system, text, font_size, label_budget(rect.w));
    let layout = text_system.layout(&shown, font_size, f32::INFINITY);
    let text_w = layout.width();
    let text_h = layout.height();
    let x = rect.x + (rect.w - text_w) / 2.0;
    let y = rect.y + (rect.h - text_h) / 2.0;
    paint_text(text_system, scene, &shown, x, y, font_size, rect.w, color);
}

// -----------------------------------------------------------------------
// Layout — 4 zones backdrop
// -----------------------------------------------------------------------

impl Paint for Layout {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        // Center is canvas — the sprite layer underneath shows through;
        // we deliberately do NOT paint Center.
        let surface = resolve(ColorToken::Bg1, ctx.theme);
        let border = resolve(ColorToken::Border, ctx.theme);
        let border_emphasis = resolve(ColorToken::BorderEmph, ctx.theme);
        let label_color = resolve(ColorToken::Text2, ctx.theme);

        for zone in [Zone::TopLeft, Zone::TopRight, Zone::Sidebar] {
            let r = self.rect(zone);
            if r.area() <= 0.0 {
                continue; // Zen mode — zone collapsed.
            }
            // Surface fill.
            scene.fill_rect(rect_to_vello(r), surface);

            // Zone label — top-left/right have CREATE/EDIT per ADR-0023
            // §3, sidebar gets MOD label oriented vertically (we just
            // render text horizontally for now — vertical text needs
            // vello transform handling that's a follow-up).
            let label = match zone {
                Zone::TopLeft => Some(tr("chrome.zone.edit")),
                Zone::TopRight => Some(tr("chrome.zone.create")),
                Zone::Sidebar => Some(tr("chrome.zone.mod")),
                Zone::Center => None,
            };
            if let Some(label) = label {
                // Inset 12 px from the seam-facing edge so the label
                // doesn't kiss the divider line.
                let label_rect = match zone {
                    Zone::Sidebar => Rect {
                        x: r.x,
                        y: r.y + 12.0,
                        w: r.w,
                        h: 14.0,
                    },
                    _ => Rect {
                        x: r.x + 16.0,
                        y: r.y,
                        w: r.w - 32.0,
                        h: r.h,
                    },
                };
                paint_text_centered(ctx.text, scene, label, label_rect, 11.0, label_color);
            }

            // 1-px divider along the seam toward Center (BorderEmphasis
            // when sidebar; Border for top zones — quieter line above
            // the canvas).
            let divider = match zone {
                Zone::TopLeft | Zone::TopRight => Rect {
                    x: r.x,
                    y: r.y + r.h - 1.0,
                    w: r.w,
                    h: 1.0,
                },
                Zone::Sidebar => match self.sidebar_side {
                    crate::zones::SidebarSide::Right => Rect {
                        x: r.x,
                        y: r.y,
                        w: 1.0,
                        h: r.h,
                    },
                    crate::zones::SidebarSide::Left => Rect {
                        x: r.x + r.w - 1.0,
                        y: r.y,
                        w: 1.0,
                        h: r.h,
                    },
                },
                Zone::Center => continue,
            };
            let divider_color = if matches!(zone, Zone::Sidebar) {
                border_emphasis
            } else {
                border
            };
            scene.fill_rect(rect_to_vello(divider), divider_color);
        }

        // Mirror-side toggle button on the sidebar — small chip at top
        // edge. Click target exposed via [`Layout::mirror_button_rect`]
        // so the shell can hit-test it.
        if let Some(btn) = self.mirror_button_rect() {
            scene.fill_rect(rect_to_vello(btn), resolve(ColorToken::BgElev, ctx.theme));
            let glyph = match self.sidebar_side {
                crate::zones::SidebarSide::Right => "<",
                crate::zones::SidebarSide::Left => ">",
            };
            paint_text_centered(ctx.text, scene, glyph, btn, 14.0, label_color);
        }
    }
}

// -----------------------------------------------------------------------
// FloatingPanel paint retired 2026-05-17.
//
// The legacy `impl Paint for FloatingPanel` rendered a Procreate-style
// drawer with an Accent-colored tab strip + Accent-tinted control row.
// That decoration predated the dark-glass canonical surface used by
// Inspector / Hierarchy / Widget Gallery and stuck out visually (pink
// Transform panel hovering over a dark editor). It was the last
// residual old-decoration site flagged in the 2026-05-17 UI audit.
//
// `Tool::build_panel()` and the `FloatingPanel` struct itself are
// kept — they still encode the tool's intended controls + tab model,
// useful as input to a future panel re-paint that uses
// `paint_panel_surface` (the canonical dark-glass painter). Removing
// only the `Paint` trait impl (and its `tab_color` / `action_label`
// helpers) cuts the visual without touching the data model.
// -----------------------------------------------------------------------

// Button paint helper now lives at crate::widget::paint_button — see
// `widget/button.rs` for the rounded-rect, kind-aware implementation.

// -----------------------------------------------------------------------
// ToastQueue — stacked at top-center
// -----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::IconId;

    /// `text_rendering` thread-local round-trip + default.
    #[test]
    fn text_rendering_thread_local_roundtrip() {
        // Start from a known state; not testing default-on-fresh-thread
        // (other tests in this binary may have set the cell).
        set_text_rendering(ph2d_tokens::TextRendering::Default);
        assert_eq!(text_rendering(), ph2d_tokens::TextRendering::Default);
        set_text_rendering(ph2d_tokens::TextRendering::CrispHeavy);
        assert_eq!(text_rendering(), ph2d_tokens::TextRendering::CrispHeavy);
        // Reset so we don't leak state into sibling tests.
        set_text_rendering(ph2d_tokens::TextRendering::Default);
    }

    /// Token → Vello color round-trips bytes accurately enough for
    /// our 8-bit palette (no banding from the sRGB linearization).
    #[test]
    fn token_to_vello_preserves_visible_difference() {
        let bg0 = ColorToken::Bg0.resolve(Theme::Forge);
        let bg1 = ColorToken::Bg1.resolve(Theme::Forge);
        // Tokens are different ⇒ vello colors must differ.
        assert_ne!(token_to_vello(bg0), token_to_vello(bg1));
    }

    /// Painting a 4-zone Layout doesn't panic and emits at least the
    /// 3 non-Center zones' rects (Center stays transparent).
    #[test]
    fn layout_paint_smoke() {
        let layout = Layout::new(1024.0, 768.0);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut ctx = PaintCtx {
            theme: Theme::Forge,
            viewport: Rect::new(0.0, 0.0, 1024.0, 768.0),
            text: &mut text,
        };
        layout.paint(&mut scene, &mut ctx);
        // Vello doesn't expose a public command count, so we settle
        // for "encoder didn't panic + reset is idempotent".
        scene.reset();
    }

    // `floating_panel_paint_collapsed_emits_only_chip` removed
    // alongside the `impl Paint for FloatingPanel` retirement
    // (2026-05-17). The struct itself stays for `Tool::build_panel`
    // event dispatch; only the legacy decoration was dropped.

    #[test]
    fn fill_rounded_rect_smoke() {
        let mut scene = VectorScene::new();
        let color = Color::WHITE;
        // sharp fall-through
        fill_rounded_rect(&mut scene, Rect::new(0.0, 0.0, 10.0, 10.0), 0.0, color);
        // rounded
        fill_rounded_rect(&mut scene, Rect::new(0.0, 0.0, 24.0, 24.0), 6.0, color);
    }

    #[test]
    fn stroke_rect_smoke() {
        let mut scene = VectorScene::new();
        stroke_rect(
            &mut scene,
            Rect::new(0.0, 0.0, 100.0, 50.0),
            1.0,
            Color::WHITE,
        );
        stroke_rect(
            &mut scene,
            Rect::new(0.0, 0.0, 100.0, 50.0),
            2.0,
            Color::BLACK,
        );
    }

    #[test]
    fn stroke_rounded_rect_smoke() {
        let mut scene = VectorScene::new();
        stroke_rounded_rect(
            &mut scene,
            Rect::new(0.0, 0.0, 80.0, 40.0),
            8.0,
            1.5,
            Color::WHITE,
        );
        // sharp fall-through
        stroke_rounded_rect(
            &mut scene,
            Rect::new(0.0, 0.0, 80.0, 40.0),
            0.0,
            1.0,
            Color::WHITE,
        );
    }

    #[test]
    fn paint_icon_smoke() {
        let mut scene = VectorScene::new();
        for icon in [
            IconId::Add,
            IconId::Check,
            IconId::Save,
            IconId::Settings,
            IconId::Sprite,
        ] {
            paint_icon(
                &mut scene,
                icon,
                Rect::new(10.0, 10.0, 36.0, 36.0),
                Color::WHITE,
                1.5,
            );
        }
    }

    #[test]
    fn paint_icon_zero_size_does_nothing() {
        let mut scene = VectorScene::new();
        // Zero/negative size must not panic and must early-return.
        paint_icon(
            &mut scene,
            IconId::Add,
            Rect::new(0.0, 0.0, 0.0, 0.0),
            Color::WHITE,
            1.5,
        );
        paint_icon(
            &mut scene,
            IconId::Add,
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Color::WHITE,
            1.5,
        );
    }
}
