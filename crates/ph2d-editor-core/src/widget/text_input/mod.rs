//! [`TextInput`] — single-line text field.
//!
//! v1 paints layout-only: caret stays static at `caret_pos`, no IME
//! composing, no selection range. Real input handling lands when the
//! shell wires `winit::Event::KeyboardInput` into the editor (post
//! M13). The data shape here exists so widgets that compose
//! TextInput (NumberInput, Combobox) have a stable contract.

use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Width of the caret bar. The viewport reserves it on the right so a caret at
/// the very end of a scrolled line stays inside the clip instead of landing on
/// its boundary — where it would be trimmed away exactly while you are typing.
const CARET_W: f32 = 1.0; // LITERAL-PX-OK: a hairline caret

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TextInputState {
    #[default]
    Normal,
    Hovered,
    Focused,
    Disabled,
    /// Validation failed; border switches to `Danger`.
    Error,
}

#[derive(Clone, Debug)]
pub struct TextInput {
    pub id: NodeId,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub state: TextInputState,
    /// Byte offset of the caret within `value`. Out-of-range values
    /// are clamped at paint time. v1 draws the caret only when
    /// `state == Focused`.
    pub caret_byte: usize,
    /// Quanto do hover está presente. [`crate::motion::SETTLED`] = assente no estado que o campo
    /// diz ter, que é o mundo pré-UI-viva byte a byte.
    pub hover_t: f32,
}

impl TextInput {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            state: TextInputState::Normal,
            caret_byte: 0,
            hover_t: crate::motion::SETTLED,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.caret_byte = self.value.len();
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.state = state;
        self
    }

    /// **O par que o store publica** — `(estado, quanto do hover está presente)`, numa pergunta só.
    /// Irmão exacto do [`crate::widget::Button::visual`].
    #[must_use]
    pub fn visual(self, v: (TextInputState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::TextInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus)
            .build()
    }
}

/// Border tokens chosen by state. Promoted to a free function so
/// `text_area`/`number_input` can reuse the same palette.
pub(crate) fn border_token(state: TextInputState) -> ColorToken {
    match state {
        TextInputState::Disabled => ColorToken::Border,
        TextInputState::Hovered => ColorToken::BorderEmph,
        TextInputState::Focused => ColorToken::Accent,
        TextInputState::Error => ColorToken::Danger,
        TextInputState::Normal => ColorToken::Border,
    }
}

/// **Como um campo se sente** — o [`TextInputState`] reduzido ao vocabulário da porta da moldura.
/// Uma porta para os três pintores da família (`text_input` · `number_input` · `text_area`).
pub(crate) fn feel_of(state: TextInputState) -> ph2d_tokens::visuals::Feel {
    use ph2d_tokens::visuals::Feel;
    match state {
        TextInputState::Normal => Feel::Rest,
        TextInputState::Hovered => Feel::Hovered,
        TextInputState::Focused => Feel::Focused,
        TextInputState::Disabled => Feel::Disabled,
        TextInputState::Error => Feel::Error,
    }
}

pub(crate) fn fill_token(state: TextInputState) -> ColorToken {
    match state {
        TextInputState::Disabled => ColorToken::Bg2,
        _ => ColorToken::Bg1,
    }
}

/// ⭐⭐⭐ **A COR DE FUNDO de um campo — a porta ÚNICA dos três pintores da família**
/// (`text_input` · `number_input` · `text_area`).
///
/// ⛔⛔ **Report do dono, 2026-09-14: *«caixas de input numérico sem cor de fundo»*.** Medido: o
/// `number_input` e a `text_area` pintavam `resolve(fill_token(state))` = **`Bg1`**, que é
/// **exactamente** o token de um cartão de secção ([`crate::widget::section_cards::CardDepth`]) —
/// e é sobre cartões que o Inspector põe as linhas dele. Distância medida: **`0/255` nos oito
/// temas**. Num tema moderno a moldura de repouso é ZERO (é a lei do `LineEdit` do Godot que esta
/// casa adoptou) ⇒ *não havia caixa nenhuma*, só o número solto sobre o cartão.
///
/// ⚠️⚠️ **E o `text_input` já perguntava ao tema — os outros dois é que não.** A resposta certa
/// existia, tinha UM chamador, e dois pintores irmãos usavam uma terceira. *Uma lei com uma porta
/// e dois consumidores fora dela não é uma lei; é uma coincidência que ainda não divergiu.*
///
/// # A escada, por família
///
/// - **Clássica** (a moldura de repouso existe): devolve o que os pintores sempre devolveram —
///   `Bg1`, ou `Bg2` desactivado. **Byte-idêntico**, e ali a caixa lê-se pela borda.
/// - **Moderna** (sem moldura em repouso): devolve o [`ph2d_tokens::visuals::Chrome::field_fill`],
///   que desde 2026-09-14 é *um degrau abaixo da superfície mais funda em que um campo pode
///   assentar* — `10/255` do painel no `Dark` e no `Gray`, `11` da subsecção no `Light`.
///
/// ⏳ **O estado DESACTIVADO não tem tinta própria num tema moderno** — a distinção viaja no
/// TEXTO (`ColorToken::TextDisabled`, que os três pintores já aplicam). É o comportamento que o
/// `text_input` já shipava; fica **nomeado** aqui em vez de inventar um segundo tom.
pub(crate) fn field_fill(state: TextInputState, theme: Theme) -> ph2d_vector::Color {
    let chrome = ph2d_tokens::visuals::Chrome::of(theme);
    if chrome.field_border.is_visible() {
        resolve(fill_token(state), theme)
    } else {
        crate::paint::token_to_vello(chrome.field_fill)
    }
}

/// **A cor da borda de um campo, já com o eixo do hover** — a porta ÚNICA dos três pintores da
/// família (`text_input` · `number_input` · `text_area`), pelo mesmo motivo que o
/// [`border_token`] é livre: eles partilham a paleta, e uma segunda cópia da mistura divergiria
/// no primeiro caso especial.
///
/// ⚠️ **Só o par `Normal ⇄ Hovered` interpola.** `Focused`, `Error` e `Disabled` nomeiam um
/// ESTADO, não uma quantidade — um `Accent` a meio caminho leria como *meio-focado*, e meia
/// desactivação não quer dizer nada. É a mesma cerca que o `Button::bg_color` planta.
///
/// ⚠️ **Quem escolhe a cor no eixo é o ESCALAR, não o estado**, e é isso que faz a SAÍDA
/// funcionar: no quadro em que o rato sai, o estado já voltou a `Normal`, então se ele decidisse
/// não haveria nada entre a cor de agora e a de repouso.
pub(crate) fn border_color(
    state: TextInputState,
    hover_t: f32,
    theme: Theme,
) -> ph2d_vector::Color {
    let soft = matches!(state, TextInputState::Normal | TextInputState::Hovered);
    crate::motion::hover_axis(
        soft,
        hover_t,
        Some(ColorToken::Border.resolve(theme)),
        Some(ColorToken::BorderEmph.resolve(theme)),
    )
    .map_or_else(
        || resolve(border_token(state), theme),
        crate::paint::token_to_vello,
    )
}

pub fn paint_text_input(
    input: &TextInput,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_text_input_with_buffer(input, None, None, None, rect, scene, text_system, theme)
}

/// Like [`paint_text_input`] but draws an override `buffer` and
/// caret offset when the caller has a live
/// [`crate::interaction::WidgetStore`] entry for the input. Reading
/// from the store avoids per-frame allocations that would happen if
/// the caller copied `store.text(id)` into `TextInput.value`.
/// `selection_anchor` is the other end of an active selection (for
/// double-click "select all" + Shift+Arrow); when None, no selection
/// is drawn.
/// ⭐⭐⭐ **O RECUO HORIZONTAL de um campo de texto** — e ele tem um SEGUNDO leitor que não pinta.
///
/// ⛔⛔ **O caret copiava este número**, e a cópia era o valor de **FÁBRICA**: `rect.x + 12.0` no
/// mapeador de clique contra `Spacing::Lg.px()` aqui. Desde que a escala numérica virou
/// **autorável**, o pintor lê o valor VIVO e a cópia não — logo bastava o artista mexer no
/// `spacing.lg` para o utilizador clicar numa letra e o cursor cair noutra.
///
/// ⚠️ **A família já tinha sido diagnosticada e curada pela METADE:** o `TextArea` ganhou a porta
/// dele (`text_area_metrics`) com este mecanismo escrito ao lado, e os outros **três** braços do
/// mesmo `match` ficaram a copiar. *Curar um braço de uma família deixa os outros com o defeito e
/// com a aparência de resolvido.*
#[must_use]
pub fn field_pad_x() -> f32 {
    Spacing::Lg.px()
}

/// **Onde o texto de um campo começa** — a porta que o pintor usa e que o caret PERGUNTA.
#[must_use]
pub fn text_origin_x(rect: Rect) -> f32 {
    rect.x + field_pad_x()
}

#[allow(clippy::too_many_arguments)]
pub fn paint_text_input_with_buffer(
    input: &TextInput,
    buffer: Option<&str>,
    caret: Option<usize>,
    selection_anchor: Option<usize>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐ O cromo do campo é do TEMA (`ph2d_tokens::visuals::Chrome`): no clássico o `Radius::Sm`,
    //    o `Bg1` e a moldura permanente de sempre; num tema moderno o raio `4`, um fundo um degrau
    //    abaixo do painel, e **moldura só no foco** (o `LineEdit` do Godot) — ou no erro.
    let chrome = ph2d_tokens::visuals::Chrome::of(theme);
    let radius = chrome.field_radius;
    // ⭐ Pela porta — ver [`field_fill`]. Ela nasceu deste `if`, quando o report do dono mostrou
    //   que os dois pintores irmãos não o tinham.
    fill_rounded_rect(scene, rect, radius, field_fill(input.state, theme));
    if chrome.field_border.is_visible() {
        let stroke_w = if input.state == TextInputState::Focused {
            chrome.field_focus.width
        } else {
            chrome.field_border.width
        };
        stroke_rounded_rect(
            scene,
            rect,
            radius,
            stroke_w,
            border_color(input.state, input.hover_t, theme),
        );
    } else {
        match input.state {
            TextInputState::Focused => stroke_rounded_rect(
                scene,
                rect,
                radius,
                chrome.field_focus.width,
                crate::paint::token_to_vello(chrome.field_focus.color),
            ),
            TextInputState::Error => {
                stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Danger, theme))
            }
            _ => {}
        }
    }

    let pad_x = field_pad_x();
    let pad_y = Spacing::Md.px();
    let font_size = TypeToken::Base.px();
    let inner_x = rect.x + pad_x;
    let inner_y = rect.y + (rect.h - font_size) * 0.5 - pad_y * 0.0;
    let inner_w = (rect.w - pad_x * 2.0).max(0.0);

    let displayed: &str = buffer.unwrap_or(input.value.as_str());
    let displayed_caret = caret.unwrap_or(input.caret_byte);

    // **A text field is one LINE, so it SCROLLS — it does not wrap.** `paint_text`
    // breaks the run at `max_width`, so a name longer than the box grew a second
    // line and spilled out of the field, over whatever sat below it (Enio,
    // 2026-07-16). The line is laid out unbounded (`f32::INFINITY` — the same "do
    // not wrap" value `prefix_width` itself passes), clipped to the inner box, and
    // slid left just far enough to keep the caret inside: the viewport every text
    // field in every toolkit has.
    //
    // Only a FOCUSED field scrolls. With no caret to follow there is nothing to
    // chase, and a reader looking at an unfocused field wants the BEGINNING of the
    // name — scrolling it to the end would hide the part that identifies it.
    let focused = input.state == TextInputState::Focused;
    let caret_w = focused.then(|| {
        text_system.prefix_width(
            &displayed[..displayed_caret.min(displayed.len())],
            font_size,
        )
    });
    let text_x = inner_x - caret_scroll(focused, inner_w, caret_w.unwrap_or(0.0));
    scene.push_clip(&crate::paint::rect_to_vello(Rect::new(
        inner_x, rect.y, inner_w, rect.h,
    )));

    if input.state == TextInputState::Focused
        && let Some(anchor) = selection_anchor
        && anchor != displayed_caret
    {
        let (sel_start, sel_end) = if anchor < displayed_caret {
            (anchor, displayed_caret)
        } else {
            (displayed_caret, anchor)
        };
        let sel_start = sel_start.min(displayed.len());
        let sel_end = sel_end.min(displayed.len());
        let prefix_w = text_system.prefix_width(&displayed[..sel_start], font_size);
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system.prefix_width(&displayed[sel_start..sel_end], font_size)
        };
        // The highlight rides the same viewport as the glyphs it covers; the clip
        // trims whatever runs past the box, so it needs no clamp of its own.
        let sel_x = text_x + prefix_w;
        let sel_w = mid_w;
        if sel_w > 0.0 {
            // ⚠️ O `pad_y` é fixo e a altura do host é variável: num campo mais baixo que dois
            // paddings a subtração fica NEGATIVA — um retângulo que se estende para CIMA do
            // próprio topo.
            //
            // ⚠️ **MEDIDO, e o piso é HIGIENE, não a cura de um defeito visível:** o `push_clip`
            // acima já apara a seleção à caixa interna, então um retângulo invertido é recortado
            // antes de chegar à tela — um gate escrito contra a CENA não consegue distinguir as
            // duas versões, e o que eu tinha escrito passava com o defeito reinstalado. Ele fica
            // porque geometria malformada é lida errada pelo próximo consumidor do retângulo (um
            // hit-test, um recorte diferente), e custa uma chamada.
            let sel_rect = Rect::new(
                sel_x,
                rect.y + pad_y,
                sel_w,
                (rect.h - pad_y * 2.0).max(0.0),
            );
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }

    if displayed.is_empty() && !input.placeholder.is_empty() {
        paint_text(
            text_system,
            scene,
            &input.placeholder,
            inner_x,
            inner_y,
            font_size,
            f32::INFINITY,
            resolve(ColorToken::Text3, theme),
        );
    } else if !displayed.is_empty() {
        let color = if input.state == TextInputState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        paint_text(
            text_system,
            scene,
            displayed,
            text_x,
            inner_y,
            font_size,
            f32::INFINITY,
            resolve(color, theme),
        );
    }

    if let Some(caret_w) = caret_w {
        // ⚠️ Mesma aritmética da seleção acima, o mesmo piso e a MESMA medição: o recorte apara,
        // então isto é higiene de geometria, não a cura de algo que se vê.
        let caret_rect = Rect::new(
            text_x + caret_w,
            rect.y + pad_y,
            CARET_W,
            (rect.h - pad_y * 2.0).max(0.0),
        );
        scene.fill_rect(
            crate::paint::rect_to_vello(caret_rect),
            resolve(ColorToken::Accent, theme),
        );
    }
    scene.pop_layer();
}

/// How far the single line is slid LEFT so the caret stays inside the box.
///
/// Pure, so the rule can be stated and tested without a scene. Exactly as far as
/// the caret overhangs, plus the caret's own width — a caret parked ON the right
/// boundary is trimmed by the clip precisely while you are typing at the end of
/// the name, which is the one moment you need to see it. An unfocused field never
/// scrolls: there is no caret to follow, and the reader wants the name's start.
fn caret_scroll(focused: bool, inner_w: f32, caret_w: f32) -> f32 {
    if focused {
        (caret_w - (inner_w - CARET_W)).max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests;
