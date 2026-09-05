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
use crate::widget::{chip_axis_color, chip_axis_t};
/// ⚠️ **Delega com o NEUTRO.** O eixo do hover vive em [`paint_tool_rail_t`]; esta assinatura é a
/// de sempre e pinta **exactamente** o que pintava antes da wave da UI viva — o molde do
/// `denoise_ml` / `denoise_ml_with_progress`.
/// **Como um chip do trilho se sente** — o estado do botão e o «está em mãos» reduzidos ao
/// vocabulário da porta da moldura ([`ph2d_tokens::visuals::Feel`]).
fn chip_feel(state: ButtonState, is_active: bool) -> ph2d_tokens::visuals::Feel {
    use ph2d_tokens::visuals::Feel;
    match state {
        ButtonState::Pressed => Feel::Active,
        _ if is_active => Feel::Active,
        ButtonState::Hovered => Feel::Hovered,
        ButtonState::Focused => Feel::Focused,
        ButtonState::Disabled => Feel::Disabled,
        _ => Feel::Rest,
    }
}

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
    paint_tool_rail_axis(
        rail,
        rect,
        scene,
        text_system,
        theme,
        store,
        hover_t,
        travels,
        RailAxis::Vertical,
    );
}

/// [`paint_tool_rail_t`] **com o eixo**: a coluna de sempre, ou a fila horizontal por cima da área
/// de desenho.
///
/// ⭐⭐ **A geometria não está aqui** — ela vem de [`super::entry_rects`], a mesma porta que o
/// registo de hit pergunta. Enquanto ela não existia, este laço e o do `hero::left_rail` eram duas
/// aritméticas gémeas, e *«o hit tem de espelhar exactamente o que o pintor pinta»* era um
/// comentário, não uma lei.
///
/// ⚠️ **O que muda com o eixo é o RÓTULO, e só ele:** rodado à esquerda do chip na coluna, direito
/// por cima dele na fila. Um rótulo rodado numa fila horizontal comeria a altura da banda.
#[allow(clippy::too_many_arguments)]
pub fn paint_tool_rail_axis(
    rail: &ToolRail,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    hover_t: &dyn Fn(NodeId) -> Option<f32>,
    travels: bool,
    axis: RailAxis,
) {
    let sub_font = (TypeToken::Xs.px() - 2.0).max(Spacing::Md.px());
    let slots = super::entry_rects(rail, rect, store.rail_button_size(), axis);
    for (slot, entry) in slots.iter().zip(rail.entries.iter()) {
        let slot_rect = slot.rect;
        match entry {
            ToolRailEntry::Icon {
                id,
                icon,
                active,
                sub,
                ..
            } => {
                let rest_rect = slot_rect;
                let chip_rect = rest_rect;
                // Halved 2026-05-24 (Lg → Sm, 12 → 6 px) per user
                // feedback that rail buttons looked too bubbly.
                let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let is_active = *active || state == ButtonState::Pressed;
                let t = chip_axis_t(state, is_active, hover_t(*id));
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
                    chip_axis_color(t, ColorToken::Border, ColorToken::BorderEmph, border, theme);
                crate::paint::stroke_frame(
                    scene,
                    chip_rect,
                    radius,
                    theme,
                    chip_feel(state, is_active),
                    border_w,
                    border_c,
                );
                let fg = match state {
                    ButtonState::Hovered | ButtonState::Focused => ColorToken::Text1,
                    ButtonState::Pressed => ColorToken::Accent,
                    _ if is_active => ColorToken::Accent,
                    _ => ColorToken::Text2,
                };
                let fg = chip_axis_color(t, ColorToken::Text2, ColorToken::Text1, fg, theme);
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
                paint_sub_label(
                    text_system,
                    scene,
                    sub,
                    sub_font,
                    rect.x,
                    rest_rect,
                    axis,
                    // M14.5 r4: Text2 keeps the label legible on the
                    // canvas-floating chrome without competing with the
                    // chip's primary content (Text3 was invisible on
                    // mid-luminance backdrops).
                    resolve(ColorToken::Text2, theme),
                );
            }
            ToolRailEntry::Compound { id, face, sub, .. } => {
                let chip_rect = slot_rect;
                // Halved 2026-05-24 (Lg → Sm, 12 → 6 px) per user
                // feedback that rail buttons looked too bubbly.
                let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                // ⚠️ **Esta variante estava FORA do eixo** (auditoria de 2026-08-23): das três
                // do rail, só a `Tool` misturava — as outras duas resolviam a borda pelo estado
                // DURO e saltavam ao lado da irmã, na mesma coluna.
                let t = chip_axis_t(state, false, hover_t(*id));
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
                let border_c =
                    chip_axis_color(t, ColorToken::Border, ColorToken::BorderEmph, border, theme);
                crate::paint::stroke_frame(
                    scene,
                    chip_rect,
                    radius,
                    theme,
                    chip_feel(state, false),
                    border_w,
                    border_c,
                );
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
                paint_sub_label(
                    text_system,
                    scene,
                    sub,
                    sub_font,
                    rect.x,
                    chip_rect,
                    axis,
                    // M14.5 r4: see Icon arm note.
                    resolve(ColorToken::Text2, theme),
                );
            }
            ToolRailEntry::Swatch {
                id,
                color,
                active,
                sub,
                ..
            } => {
                let chip_rect = slot_rect;
                let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
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
                // ⚠️ **A amostra é a 3ª variante, e estava fora do eixo pela mesma razão.** O
                // FUNDO dela é a tinta do artista (não um token), então o eixo só tem a moldura a
                // que agarrar — e é justamente ela que diz *"o dedo está aqui"*.
                let border_c = chip_axis_color(
                    chip_axis_t(state, is_active, hover_t(*id)),
                    ColorToken::Border,
                    ColorToken::BorderEmph,
                    border,
                    theme,
                );
                crate::paint::stroke_frame(
                    scene,
                    chip_rect,
                    radius,
                    theme,
                    chip_feel(state, is_active),
                    border_w,
                    border_c,
                );
                paint_sub_label(
                    text_system,
                    scene,
                    sub,
                    sub_font,
                    rect.x,
                    chip_rect,
                    axis,
                    resolve(ColorToken::Text2, theme),
                );
            }
            // ⭐ **O chip de CAMINHO** — as ferramentas de imagem, cujo ícone vem do manifesto.
            // ⚠️ A matriz de tinta é a MESMA do braço `Icon`, e a única diferença é a rota do
            // glifo (`paint_icon_path` em vez de `paint_icon`): dois chips do mesmo trilho não
            // podem prometer o mesmo gesto com desenhos diferentes.
            ToolRailEntry::Glyph {
                id,
                path,
                active,
                sub,
                ..
            } => {
                let rest_rect = slot_rect;
                let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let is_active = *active || state == ButtonState::Pressed;
                let t = chip_axis_t(state, is_active, hover_t(*id));
                let chip_rect = crate::motion::hover_lift(rest_rect, t.unwrap_or(0.0), travels);
                let bg = match state {
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
                    chip_axis_color(t, ColorToken::Border, ColorToken::BorderEmph, border, theme);
                crate::paint::stroke_frame(
                    scene,
                    chip_rect,
                    radius,
                    theme,
                    chip_feel(state, is_active),
                    border_w,
                    border_c,
                );
                // ⚠️ **O glifo vai pelo pintor CANÓNICO** (`IconButtonStyle::Plain`: sem fundo
                // nem moldura, só o desenho), e não por um `paint_icon_path` à mão. É a mesma
                // rota que o chip da barra de topo usa, e existe uma cerca a exigi-la
                // (`canonical_icon_button`): um caminho de manifesto pintado à mão é como as
                // pills de Image Tools hand-rolled chrome antes de haver um pintor só.
                // ⚠️ O `icon_tint` interno reproduz o mesmo mapa Text2/Text1/Accent que o braço
                // `Icon` resolve à mão.
                crate::widget::paint_icon_button(
                    chip_rect,
                    crate::widget::IconGlyph::Path(path),
                    crate::widget::IconButtonStyle::Plain,
                    (state, t.unwrap_or(crate::motion::SETTLED)),
                    scene,
                    theme,
                );
                paint_sub_label(
                    text_system,
                    scene,
                    sub,
                    sub_font,
                    rect.x,
                    rest_rect,
                    axis,
                    resolve(ColorToken::Text2, theme),
                );
            }
            ToolRailEntry::Divider => {
                scene.fill_rect(rect_to_vello(slot_rect), resolve(ColorToken::Border, theme));
            }
        }
    }
}

/// **O rótulo, no eixo em que este rail corre** — rodado à esquerda do chip numa coluna, direito
/// por cima dele numa fila.
///
/// ⚠️ Uma função só, e não duas chamadas nos braços: os três braços do pintor pedem o rótulo, e a
/// escolha do eixo repetida três vezes seria três sítios onde um deles fica para trás.
#[allow(clippy::too_many_arguments)]
fn paint_sub_label(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    font_size: f32,
    rail_left_x: f32,
    chip_rect: Rect,
    axis: RailAxis,
    color: ph2d_vector::Color,
) {
    match axis {
        RailAxis::Vertical => paint_sub_label_vertical(
            text_system,
            scene,
            text,
            font_size,
            rail_left_x,
            chip_rect,
            color,
        ),
        RailAxis::Horizontal => {
            paint_sub_label_above(text_system, scene, text, font_size, chip_rect, color)
        }
    }
}

/// Helper — o rótulo direito, centrado POR CIMA do chip (a fila horizontal).
///
/// ⚠️ A banda que ele ocupa é a mesma [`LABEL_VISUAL_EXTENT_PX`] que a coluna reserva para o
/// rodado — o mesmo número, o mesmo tipo de letra, medido do mesmo sítio. É o que faz a fila e a
/// coluna terem chips do mesmo tamanho.
fn paint_sub_label_above(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    font_size: f32,
    chip_rect: Rect,
    color: ph2d_vector::Color,
) {
    if text.is_empty() {
        return;
    }
    let band = Rect::new(
        chip_rect.x,
        chip_rect.y - LABEL_TO_CHIP_GAP_PX - LABEL_VISUAL_EXTENT_PX,
        chip_rect.w,
        LABEL_VISUAL_EXTENT_PX,
    );
    crate::paint::paint_text_centered(text_system, scene, text, band, font_size, color);
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
