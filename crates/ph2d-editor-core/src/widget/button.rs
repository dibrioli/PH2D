//! [`Button`] — text/icon CTA, four kinds × six states.
//!
//! Same pattern as [`crate::widget::ColorSwatch`]: data + state enum +
//! token-resolved colors + AccessKit `Role::Button` node + colocated
//! [`paint_button`]. Accent ladder on a single hue: Normal→Accent,
//! Hover→AccentHover (brighter), Pressed→AccentPress (deeper). Hover
//! must NOT use AccentSoft — that dark, desaturated surface tone read
//! as "disabled" under the cursor. Danger follows the same ladder
//! rotated to the danger hue. Secondary (`Default`) buttons carry a
//! discrete `Border` outline (see [`Button::border_color`]).
//!
//! ⚠️ **A tabela de COR mudou-se para o irmão `button_surface/cor.rs`** (tecto de 500 LOC dos
//! primitivos, 2026-09-19) — é uma `impl` do mesmo tipo, logo nenhum chamador muda. O corte é por
//! responsabilidade: aqui mora *o que um botão É e como se pinta*; ali, *que cor ele tem*.

use crate::icons::IconId;
use crate::paint::{paint_icon, paint_text_centered, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ICON_BTN_SIZE_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ButtonState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
    /// Awaiting an async result. Body grays out, label is replaced
    /// with a spinner glyph (rendered by [`paint_button`]).
    Loading,
}

/// Visual variant. The geometry is identical across kinds — only the
/// token palette changes — except for [`ButtonKind::IconOnly`] which
/// renders a square chip with no label and an icon centered.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ButtonKind {
    /// Ghost / text-only on the panel surface. Default for secondary
    /// actions (Cancel, Reset).
    #[default]
    Default,
    /// Primary CTA. Filled `Accent` background.
    Accent,
    /// Destructive CTA. Filled `Danger` background.
    Danger,
    /// Square 36x36 chip with only an icon. Used in tool palettes
    /// and dense toolbars.
    IconOnly { icon: IconId },
}

#[derive(Clone, Debug)]
pub struct Button {
    pub id: NodeId,
    pub label: String,
    pub state: ButtonState,
    pub kind: ButtonKind,
    /// ⚠️ **Campo com NEUTRO** (`1.0`), o molde do `SkinParam` e dos canais do `KernelResolver`:
    /// quem não o define não sabe que ele existe, e pinta o que pintava antes.
    pub hover_t: f32,
    /// ⭐⭐⭐ **ONDE este botão está no grupo dele** (wave 20). Neutro = [`GroupPos::Only`], que
    /// arredonda os quatro cantos — quem não o define pinta o que pintava antes, ao bit.
    ///
    /// ⚠️ **A lei do grupo já vivia no chip segmentado desde a wave 10, e não no BOTÃO** — e é por
    /// isso que o editor de áudio junta `Apply | Cancel` e as quatro ferramentas de imagem
    /// desenham `Cancel | Apply` **separados**: o mesmo par, dois idiomas, porque um deles usa o
    /// widget que sabia a lei e o outro o que não sabia. *Uma lei que só metade dos widgets
    /// conhece produz dois dialectos no mesmo aplicativo.*
    pub cell: crate::widget::GroupCell,
}

impl Button {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            state: ButtonState::Normal,
            kind: ButtonKind::Default,
            hover_t: 1.0,
            cell: crate::widget::GroupCell {
                col: crate::widget::GroupPos::Only,
                row: crate::widget::GroupPos::Only,
            },
        }
    }

    /// ⭐ **Este botão é uma PEÇA de um grupo** — só as bordas de fora do grupo arredondam.
    #[must_use]
    pub fn in_group(mut self, cell: crate::widget::GroupCell) -> Self {
        self.cell = cell;
        self
    }

    /// Convenience: filled accent CTA.
    pub fn accent(mut self) -> Self {
        self.kind = ButtonKind::Accent;
        self
    }

    /// Convenience: filled destructive CTA.
    pub fn danger(mut self) -> Self {
        self.kind = ButtonKind::Danger;
        self
    }

    /// Convenience: 36x36 icon-only chip. Label still required for
    /// AccessKit (screen readers narrate it).
    pub fn icon_only(mut self, icon: IconId) -> Self {
        self.kind = ButtonKind::IconOnly { icon };
        self
    }

    pub fn kind(mut self, kind: ButtonKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    /// **As DUAS metades numa chamada** — o par que a
    /// [`crate::panel::PanelHostInternal::button_visual`] devolve.
    ///
    /// ⚠️ **Existe para ser mais CURTO que a rota errada.** A alternativa é
    /// `.state(store.button_state(id).unwrap_or(Normal))` seguido de um `.hover_t(...)` que o sítio
    /// seguinte esquece — e um botão sem `t` cai no default `1.0`, que **salta**. Quando a rota
    /// certa é a mais curta de escrever, o sítio 103 nasce certo por preguiça e não por disciplina.
    #[must_use]
    pub fn visual(self, v: (ButtonState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    /// **Quanto do hover está presente**, `0..1`. Neutro = `1.0` ⇒ toda construção que não o
    /// define pinta **exactamente** o que pintava antes da wave da UI viva.
    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    pub fn font_size(&self) -> f32 {
        Self::label_font_px()
    }

    /// ⭐⭐⭐ **A fonte do rótulo, sem precisar de um botão** — quem pergunta *«este rótulo cabe?»*
    /// lê a resposta AQUI, e nunca de um token escolhido à parte. ⚠️ Uma régua que a adivinha mede
    /// outro programa: a medição que pagou esta porta está no
    /// `nenhum_chip_do_tween_sai_cortado.rs` do `ph2d-panel-inspector`.
    #[must_use]
    pub fn label_font_px() -> f32 {
        button_label_font()
    }

    pub fn padding(&self) -> f32 {
        Spacing::Lg.px()
    }

    /// ⭐⭐⭐ **A quina de um botão vem da PORTA DO TEMA** (wave 22), e a assinatura recebeu o tema
    /// por isso.
    ///
    /// ⛔⛔ **Ela devolvia `Radius::Md` (4) sem perguntar nada, e isso é um dialecto no widget
    /// mais usado do app:** um chip segmentado pintava `3` (o número do Godot Modern, wave 8) e um
    /// botão pintava `4`, lado a lado, na mesma fileira. A porta chegou aos PAINÉIS na wave 2/3 e
    /// não aos **primitivos** — e um primitivo que escolhe sozinho não é uma excepção: é a
    /// resposta que ~100 sítios herdam sem saber.
    pub fn radius(&self, theme: Theme) -> f32 {
        crate::paint::frame_radius(theme, Radius::Md.px())
    }

    /// Build the AccessKit node. Per ADR-0023 §10: every interactive
    /// widget exposes role + label + clickable action.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Button)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != ButtonState::Disabled)
            .action(Action::Click)
            .build()
    }
}

/// Suggested square edge for [`ButtonKind::IconOnly`].
pub const ICON_BUTTON_SIZE_PX: f32 = ICON_BTN_SIZE_PX;

/// Paint a button at the given rect. Honors [`ButtonKind`] for
/// background, focus ring, label/icon swap, and Loading→spinner glyph.
/// ⭐⭐ **O TAMANHO em que um botão escreve o rótulo dele — uma resposta, dois leitores.**
///
/// ⚠️ Ela existe porque quem **dispõe** uma fileira de botões precisa de medir as palavras na
/// fonte em que elas vão ser **pintadas** ([`crate::widget::segment_rects_for`]) — e escrever o
/// `TypeToken::Base` no painel seria a segunda cópia dela, que diverge no dia em que o token
/// mudar. *Medir num tamanho e pintar noutro corta exactamente na fronteira em que o corte
/// existe.*
#[must_use]
pub fn button_label_font() -> f32 {
    TypeToken::Base.px()
}

pub fn paint_button(
    button: &Button,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = button.radius(theme);
    // ⭐ As quatro quinas saem da POSIÇÃO no grupo (wave 20). Um botão sozinho devolve as quatro
    //    iguais ao `radius`, e o desenho é byte-idêntico ao de antes.
    let radii = button.cell.radii(radius);
    if let Some(bg) = button.bg_color(theme) {
        crate::paint::fill_rounded_rect_radii(
            scene,
            rect,
            radii,
            ph2d_vector::Color::from_rgba8(bg.r, bg.g, bg.b, bg.a), // LITERAL-COLOR-OK: token-bridge — `bg` is ColorToken-resolved
        );
    }
    // Discrete outline for secondary / ghost buttons (so a Normal-state
    // Cancel/Reset reads as a button, not bare text). Drawn before the
    // focus ring so the ring wins visually when focused.
    // ⭐ **Se o TEMA traça bordas** — a tabela de estados (`ph2d_tokens::visuals::Widgets`) diz:
    //    no clássico sim (o `Border` a 1 px de sempre); num tema moderno não (o Godot só as
    //    traça com *Draw Extra Borders*). Um botão secundário plano continua a ler-se como botão
    //    pelo fundo que ganha sob o rato, não por uma moldura permanente.
    let outlines = ph2d_tokens::visuals::Widgets::of(theme)
        .inactive
        .bg_stroke
        .is_visible();
    if let Some(b) = button.border_color(theme).filter(|_| outlines) {
        stroke_rounded_rect(
            scene,
            rect,
            radius,
            StrokeToken::Default.px(),
            ph2d_vector::Color::from_rgba8(b.r, b.g, b.b, b.a), // LITERAL-COLOR-OK: token-bridge — `b` is ColorToken::Border
        );
    }
    if button.focus_ring() {
        let ring = ColorToken::BorderEmph.resolve(theme);
        // FRAME-RAW-OK: o ANEL DE FOCO. ⚠️ O traco de repouso logo acima PERGUNTA ao tema (o
        // `Widgets::of(theme).inactive.bg_stroke.is_visible()` no `if`) e por isso nao leva
        // marcador — este nao pergunta de proposito: onde esta' o foco tem de ver-se em qualquer
        // pele, senao o teclado deixa de ter onde pousar.
        stroke_rounded_rect(
            scene,
            rect,
            radius,
            2.0,
            ph2d_vector::Color::from_rgba8(ring.r, ring.g, ring.b, ring.a), // LITERAL-COLOR-OK: token-bridge — `ring` is ColorToken::BorderEmph
        );
    }
    let fg_token = button.fg_color(theme);
    let fg = ph2d_vector::Color::from_rgba8(fg_token.r, fg_token.g, fg_token.b, fg_token.a); // LITERAL-COLOR-OK: token-bridge — `fg_token` is ColorToken-resolved
    match button.kind {
        ButtonKind::IconOnly { icon } => {
            paint_icon(scene, icon, rect, fg, StrokeToken::Default.px());
        }
        _ => {
            if button.state == ButtonState::Loading {
                paint_icon(scene, IconId::Spinner, rect, fg, StrokeToken::Default.px());
            } else {
                paint_text_centered(
                    text_system,
                    scene,
                    &button.label,
                    rect,
                    button.font_size(),
                    fg,
                );
            }
        }
    }
}

// Os gates deste primitivo vivem num irmão `#[path]` para continuarem módulo FILHO enquanto este
// ficheiro fica sob o tecto de LOC — o mesmo corte que o `command_palette` já pagou.
#[cfg(test)]
mod tests;
