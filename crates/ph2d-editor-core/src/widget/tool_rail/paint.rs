//! **Como um chip do rail se DESENHA** — a outra metade do [`super`], que diz o que um chip *é*.
//!
//! ⚠️ O corte é por RESPONSABILIDADE, não por tamanho: o pai carrega o MODELO (as entradas, o
//! preset de tamanho, a árvore de a11y) e este filho carrega a tinta. Foi o teto de 500 LOC dos
//! primitivos de widget que forçou a decisão, e a linha de corte já estava lá — a wave da UI viva
//! só a tornou visível.
//!
//! `paint_tool_rail`/`paint_tool_rail_t` continuam a ser re-exportados pelo pai, então **nenhum
//! caminho de chamada muda**.

use super::*;

/// **Quanto do hover este chip pode mostrar** — `1.0` (o neutro) sempre que o estado NÃO é uma
/// quantidade.
///
/// ⚠️ Um chip **activo** ou **premido** sai do eixo por decisão, não por omissão: *activo* responde
/// *"esta é a ferramenta na tua mão"*, e uma resposta que desvanece é uma resposta que se lê mal.
pub(super) fn rail_hover_t(state: ButtonState, is_active: bool, t: Option<f32>) -> Option<f32> {
    if is_active || !matches!(state, ButtonState::Normal | ButtonState::Hovered) {
        return None;
    }
    t
}

/// A mistura `repouso → hover` em espaço de TOKEN, ou a cor dura quando `t` já é o neutro.
///
/// ⚠️ Mistura-se o token e converte-se depois porque [`crate::motion::blend_token_color`] é o motor
/// ÚNICO deste eixo (o mesmo do `Button` e do `IconButton`) — uma segunda aritmética de cor aqui
/// divergiria da dele no dia em que um dos dois ganhasse gama.
pub(super) fn blend_or(
    t: Option<f32>,
    rest: ColorToken,
    hot: ColorToken,
    hard: ColorToken,
    theme: Theme,
) -> ph2d_vector::Color {
    if let Some(t) = t
        && let Some(c) =
            crate::motion::blend_token_color(Some(rest.resolve(theme)), Some(hot.resolve(theme)), t)
    {
        return crate::paint::token_to_vello(c);
    }
    resolve(hard, theme)
}

/// ⚠️ **Delega com o NEUTRO.** O eixo do hover vive em [`paint_tool_rail_t`]; esta assinatura é a
/// de sempre e pinta **exactamente** o que pintava antes da wave da UI viva — o molde do
/// `denoise_ml` / `denoise_ml_with_progress`.
pub fn paint_tool_rail(
    rail: &ToolRail,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
) {
    paint_tool_rail_t(
        rail,
        rect,
        scene,
        text_system,
        theme,
        store,
        &|_| None,
        false,
    );
}

/// [`paint_tool_rail`] **com o eixo do hover**: `hover_t(id)` é *quanto do hover está presente*
/// naquele chip, `0..1`, e o neutro é **`1.0`**.
///
/// ⚠️ **O que se anima no rail é a BORDA e o TINT, nunca o fundo** — medido: `Normal` e `Hovered`
/// pedem os dois `BgElev`, então um chip do rail nunca teve mudança de fundo a que agarrar. É por
/// isso que a diferença aqui é mais discreta que num botão de barra, e a nota existe para ninguém
/// procurar um bug onde há uma escolha de tema.
///
/// ⚠️ **Um chip ATIVO fica FORA do eixo.** *Activo* não é uma quantidade — é o estado que diz
/// *"esta é a ferramenta na tua mão"*, e desvanecê-lo entre dois valores faria o rail piscar a
/// resposta a uma pergunta que o artista precisa de ler de relance. Mesma lei do `Pressed` no
/// [`super::button::Button::bg_color`].
///
/// ⚠️ **`travels` é a permissão de MEXER, e é um `bool` de propósito.** O widget não conhece a
/// `UiMotion` (a closure existe exactamente para o desacoplar dela); o que ele precisa de saber é
/// o facto mínimo — *este chip pode crescer?* — e quem o responde é o chrome, que tem o relógio.
///
/// ⚠️ **A closure é `&dyn`, não genérica**, porque o chamador varre uma lista e o custo de uma
/// chamada indirecta por chip é ruído ao lado de desenhar o chip — e genérico aqui monomorfizaria
/// o corpo inteiro por cada sítio de pintura.
#[allow(clippy::too_many_arguments)] // o 8o e a permissao de mexer; ver o doc acima
pub fn paint_tool_rail_t(
    rail: &ToolRail,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    hover_t: &dyn Fn(NodeId) -> Option<f32>,
    travels: bool,
) {
    // Chip x is computed from the label column budget so the
    // label-to-chip gap is exactly `LABEL_TO_CHIP_GAP_PX`, regardless
    // of how wide the rail itself is set.
    let chip_x = rect.x + CHIP_X_OFFSET_PX;
    let sub_font = (TypeToken::Xs.px() - 2.0).max(Spacing::Md.px());
    let gap = Spacing::Xs.px();
    let chip_px = store.rail_button_size().chip_px();
    let mut y = rect.y;
    for (i, entry) in rail.entries.iter().enumerate() {
        if i > 0 {
            y += gap;
        }
        match entry {
            ToolRailEntry::Icon {
                id,
                icon,
                active,
                sub,
                ..
            } => {
                let rest_rect = Rect::new(chip_x, y, chip_px, chip_px);
                let chip_rect = rest_rect;
                // Halved 2026-05-24 (Lg → Sm, 12 → 6 px) per user
                // feedback that rail buttons looked too bubbly.
                let radius = Radius::Sm.px();
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let is_active = *active || state == ButtonState::Pressed;
                let t = rail_hover_t(state, is_active, hover_t(*id));
                // ⚠️ O chip CRESCE com o hover, e o hit fica no retangulo de repouso (quem o
                // regista e' o `left_rail`, com o `chip_rect` de antes deste `hover_lift`).
                let chip_rect = crate::motion::hover_lift(chip_rect, t.unwrap_or(0.0), travels);
                let bg = match state {
                    ButtonState::Hovered | ButtonState::Focused => ColorToken::BgElev,
                    ButtonState::Pressed => ColorToken::AccentSoft,
                    _ if is_active => ColorToken::AccentSoft,
                    _ => ColorToken::BgElev,
                };
                fill_rounded_rect(scene, chip_rect, radius, resolve(bg, theme));
                let (border, border_w) = match state {
                    ButtonState::Hovered | ButtonState::Focused => (ColorToken::BorderEmph, 1.0),
                    ButtonState::Pressed => (ColorToken::Accent, StrokeToken::Default.px()),
                    _ if is_active => (ColorToken::Accent, StrokeToken::Default.px()),
                    _ => (ColorToken::Border, 1.0),
                };
                let border_c =
                    blend_or(t, ColorToken::Border, ColorToken::BorderEmph, border, theme);
                stroke_rounded_rect(scene, chip_rect, radius, border_w, border_c);
                let fg = match state {
                    ButtonState::Hovered | ButtonState::Focused => ColorToken::Text1,
                    ButtonState::Pressed => ColorToken::Accent,
                    _ if is_active => ColorToken::Accent,
                    _ => ColorToken::Text2,
                };
                let fg = blend_or(t, ColorToken::Text2, ColorToken::Text1, fg, theme);
                paint_icon(scene, *icon, chip_rect, fg, StrokeToken::Default.px());
                // ⚠️ **O rótulo é medido contra o chip em REPOUSO, não contra o crescido.** O 6º
                // argumento é o `max_width` do LAYOUT (ver `paint_text::paint_text_rotated_ccw`),
                // então passar o rect do hover fazia a largura de QUEBRA do texto mudar quando o
                // rato pousava — um rótulo que roça o limite re-fluiria no hover, e no Expressivo
                // re-fluiria duas vezes (o percurso ultrapassa e volta). É a lei que o `hover_lift`
                // já carrega para o hit: *o desenho cresce, o que o vizinho MEDE não*.
                //
                // ⚠️ A POSIÇÃO não muda com isto — `chip_center_y` é invariante sob o `hover_lift`
                // (sobe `d` e cresce `2d`, logo o centro fica). O que muda é só a largura de layout.
                // **NÃO foi medido se algum rótulo de hoje chega a re-fluir** (são palavras curtas
                // contra 36-44 px): entra por ser a mesma lei, não por um sintoma reportado.
                paint_sub_label_vertical(
                    text_system,
                    scene,
                    sub,
                    sub_font,
                    rect.x,
                    rest_rect,
                    // M14.5 r4: Text2 keeps the label legible on the
                    // canvas-floating chrome without competing with the
                    // chip's primary content (Text3 was invisible on
                    // mid-luminance backdrops).
                    resolve(ColorToken::Text2, theme),
                );
                y += chip_px;
            }
            ToolRailEntry::Compound { id, face, sub, .. } => {
                let chip_rect = Rect::new(chip_x, y, chip_px, chip_px);
                // Halved 2026-05-24 (Lg → Sm, 12 → 6 px) per user
                // feedback that rail buttons looked too bubbly.
                let radius = Radius::Sm.px();
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let bg = match state {
                    ButtonState::Hovered | ButtonState::Focused => ColorToken::BgElev,
                    ButtonState::Pressed => ColorToken::AccentSoft,
                    _ => ColorToken::BgElev,
                };
                fill_rounded_rect(scene, chip_rect, radius, resolve(bg, theme));
                let (border, border_w) = match state {
                    ButtonState::Hovered | ButtonState::Focused => (ColorToken::BorderEmph, 1.0),
                    ButtonState::Pressed => (ColorToken::Accent, StrokeToken::Default.px()),
                    _ => (ColorToken::Border, 1.0),
                };
                stroke_rounded_rect(scene, chip_rect, radius, border_w, resolve(border, theme));
                let face_color = match state {
                    ButtonState::Pressed => ColorToken::Accent,
                    _ => ColorToken::Text1,
                };
                // Clip face text to the chip rect so longer labels
                // ("Selected", "Global") don't overflow the button
                // edges when the user picks Small rail size (chip <
                // text width). Font drops Xs → Xxs (11 → 10 px) to
                // give labels more room before the clip kicks in.
                let face_clip = ph2d_vector::Rect::new(
                    chip_rect.x as f64,
                    chip_rect.y as f64,
                    (chip_rect.x + chip_rect.w) as f64,
                    (chip_rect.y + chip_rect.h) as f64,
                );
                scene.push_clip(&face_clip);
                paint_text_centered(
                    text_system,
                    scene,
                    face,
                    chip_rect,
                    TypeToken::Xxs.px(),
                    resolve(face_color, theme),
                );
                scene.pop_layer();
                paint_sub_label_vertical(
                    text_system,
                    scene,
                    sub,
                    sub_font,
                    rect.x,
                    chip_rect,
                    // M14.5 r4: see Icon arm note.
                    resolve(ColorToken::Text2, theme),
                );
                y += chip_px;
            }
            ToolRailEntry::Swatch {
                id,
                color,
                active,
                sub,
                ..
            } => {
                let chip_rect = Rect::new(chip_x, y, chip_px, chip_px);
                let radius = Radius::Sm.px();
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let is_active = *active || state == ButtonState::Pressed;
                // The chip IS the colour box — fill it with the live paint colour (this button doubles as
                // the colour selector).
                let [cr, cg, cb, ca] = *color;
                let fill = ph2d_vector::Color::from_rgba8(cr, cg, cb, ca); // LITERAL-COLOR-OK: user paint colour
                fill_rounded_rect(scene, chip_rect, radius, fill);
                // State border — Accent when active/pressed/hovered, same as the icon chips.
                let (border, border_w) = match state {
                    ButtonState::Hovered | ButtonState::Focused => (ColorToken::BorderEmph, 1.0),
                    ButtonState::Pressed => (ColorToken::Accent, StrokeToken::Default.px()),
                    _ if is_active => (ColorToken::Accent, StrokeToken::Default.px()),
                    _ => (ColorToken::Border, 1.0),
                };
                stroke_rounded_rect(scene, chip_rect, radius, border_w, resolve(border, theme));
                paint_sub_label_vertical(
                    text_system,
                    scene,
                    sub,
                    sub_font,
                    rect.x,
                    chip_rect,
                    resolve(ColorToken::Text2, theme),
                );
                y += chip_px;
            }
            ToolRailEntry::Divider => {
                y += DIVIDER_GAP_PX;
                let line = Rect::new(
                    rect.x + (rect.w - Spacing::Xl2.px()) * 0.5,
                    y,
                    Spacing::Xl2.px(),
                    1.0,
                );
                scene.fill_rect(rect_to_vello(line), resolve(ColorToken::Border, theme));
                y += 1.0 + DIVIDER_GAP_PX;
            }
        }
    }
}

/// Helper — paint a short uppercase tag vertically (CCW-rotated)
/// in the column to the LEFT of `chip_rect`.
///
/// Layout decisions (mirror user feedback):
///   - Label hugs the LEFT edge of the rail (`LABEL_LEFT_PAD`).
///   - Label is vertically centered with the chip — we measure the
///     real text width via `prefix_width` and offset the bottom
///     anchor by half that width so the rotated text's midpoint
///     lands on `chip.center_y`.
///   - Gap to the chip is fixed by `LABEL_TO_CHIP_GAP_PX`; the
///     chip's x already accounts for it via `CHIP_X_OFFSET_PX`.
fn paint_sub_label_vertical(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    font_size: f32,
    rail_left_x: f32,
    chip_rect: Rect,
    color: ph2d_vector::Color,
) {
    if text.is_empty() {
        return;
    }
    // Measure the unrotated text width — after 90° CCW this is the
    // text's VERTICAL extent on screen.
    let text_w = text_system.prefix_width(text, font_size);
    let chip_center_y = chip_rect.y + chip_rect.h * 0.5;
    // After rotation, the parley layout's (0, 0) — i.e. our anchor —
    // becomes the BOTTOM-LEFT of the rotated bbox. Center the text
    // vertically on the chip by offsetting from `chip_center_y` by
    // half the rotated extent (= text_w).
    let anchor_x = rail_left_x + LABEL_LEFT_PAD;
    let anchor_y = chip_center_y + text_w * 0.5;
    paint_text_rotated_ccw(
        text_system,
        scene,
        text,
        anchor_x,
        anchor_y,
        font_size,
        chip_rect.h,
        color,
    );
}
