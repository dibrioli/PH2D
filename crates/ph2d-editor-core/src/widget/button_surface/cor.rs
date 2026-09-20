//! **A COR de um [`Button`] em cada estado** — a tabela que o pintor lê.
//!
//! ⚠️ **Irmão do [`super::super::button`] pelo tecto de 500 LOC dos primitivos** (corte de
//! 2026-09-19), e o corte é por RESPONSABILIDADE: ali mora *o que um `Button` É* (os dados, os
//! construtores, o nó de AccessKit, o pintor); aqui, *que cor ele tem*. As duas crescem por
//! motivos diferentes — uma ganha um construtor, a outra ganha um estado na escada.
//!
//! ⚠️ **É uma `impl` do MESMO tipo, não uma porta nova:** nenhum chamador muda de caminho, e o
//! `bg_token` continua **privado ao módulo** — a privacidade dele é a lei que o irmão
//! [`super`] declara por escrito.

use crate::widget::{Button, ButtonKind, ButtonState};
use ph2d_tokens::{Color as TokenColor, ColorToken, Theme};

impl Button {
    /// Resolve the foreground (text + icon) token for the current
    /// state and kind.
    pub fn fg_color(&self, theme: Theme) -> TokenColor {
        if self.state == ButtonState::Disabled {
            return ColorToken::TextDisabled.resolve(theme);
        }
        match self.kind {
            ButtonKind::Default | ButtonKind::IconOnly { .. } => ColorToken::Text1.resolve(theme),
            ButtonKind::Accent | ButtonKind::Danger => ColorToken::AccentFg.resolve(theme),
        }
    }

    /// A cor de fundo, **misturada no eixo do hover**.
    ///
    /// ⚠️ **Quem escolhe a cor neste eixo é o ESCALAR, não o estado — e é isso que faz a saída
    /// funcionar.** Se o estado escolhesse, sair do hover seria instantâneo: no quadro em que o
    /// rato sai, `state` volta a `Normal` e `bg_color(Normal)` já é a cor de repouso, então não
    /// haveria nada entre onde a cor está e onde ela vai. Misturando *repouso → hover* por `t`, a
    /// entrada e a **saída** são a mesma expressão.
    ///
    /// ⚠️ `Pressed`, `Focused` e `Disabled` continuam a ser **estados duros**: eles não são uma
    /// *quantidade* de nada, e tratá-los como fracção faria um botão desactivado ter meia-desactivação.
    #[must_use]
    pub fn bg_color(&self, theme: Theme) -> Option<TokenColor> {
        if self.hover_t < 1.0 && matches!(self.state, ButtonState::Normal | ButtonState::Hovered) {
            let rest = self.bg_token(ButtonState::Normal).map(|t| t.resolve(theme));
            let hot = self
                .bg_token(ButtonState::Hovered)
                .map(|t| t.resolve(theme));
            return crate::motion::blend_token_color(rest, hot, self.hover_t);
        }
        self.bg_token(self.state).map(|t| t.resolve(theme))
    }

    /// O token de fundo de UM estado. `None` para o *ghost* (Default + IconOnly em Normal): o
    /// rectângulo fica transparente e o rótulo/ícone pintam sobre a superfície do painel.
    ///
    /// ⚠️ Este doc estava **ÓRFÃO** no topo do `hover_t` desde a wave do F0 — a inserção de um
    /// membro entre um doc e a função dele é o mesmo deslize que o `tick_motion` sofreu no corte do
    /// `live.rs`. Reposto no membro que de facto codifica a regra.
    fn bg_token(&self, state: ButtonState) -> Option<ColorToken> {
        let token = match (self.kind, state) {
            (_, ButtonState::Disabled) => match self.kind {
                ButtonKind::Default | ButtonKind::IconOnly { .. } => return None,
                _ => ColorToken::Border,
            },
            (ButtonKind::Default, ButtonState::Hovered | ButtonState::Focused) => {
                ColorToken::BgElev
            }
            (ButtonKind::Default, ButtonState::Pressed) => ColorToken::AccentSoft,
            // ⭐⭐⭐ **UM BOTÃO PARADO PINTA `Bg2`** (report do dono, 2026-09-20: *«não se pode
            //    saber que é um botão pois só aparece o nome … todo o app tem essa aparência»*).
            //
            // ⛔⛔ **Havia DUAS leis para «que cor tem um botão em repouso», e o doc de uma
            //    afirmava ser a outra.** O
            //    [`crate::widget::button_surface::flat_button_surface`] — que cinco sítios de
            //    pintura usam — diz `repouso Bg2 … as mesmas superfícies que o `Button` canónico
            //    usa*, e o `Button` canónico devolvia **`None`**. É por isso que na foto do dono os
            //    chips segmentados (`S | B`, `Basic | Pro`, `Move | Rotate | Scale`) se leem como
            //    botões e o `Draw`, o `Filter Collisions` e o `Apply to all tools` se leem como
            //    legendas: *o chip conhecia a lei e o Button não* — a MESMA assimetria entre estes
            //    dois widgets que a wave 20 já registou para a lei do GRUPO.
            //
            // ⭐ **E a tabela de design desta linha já o declarava**
            //    (`pesquisa/08 §7.16`): *«um botão em repouso PINTA `Bg2`»*, com a coluna
            //    `botão (Bg2) = #292929` no Dark, contra o painel a `#131313` e o cartão a
            //    `#1f1f1f`. ⇒ isto não escolhe cor nenhuma: põe o widget a obedecer à escada que
            //    já foi medida e aprovada.
            //
            // ⛔⛔ **A cura NÃO é devolver a moldura**, e a razão está no `paint_button`: num tema
            //    moderno o traço de repouso é filtrado por `Widgets::of(theme).inactive.bg_stroke`
            //    (o Godot só traça com *Draw Extra Borders*), e o redesenho tirou-a de propósito.
            //    *A afordância de um botão plano é a SUPERFÍCIE dele, não um contorno.*
            //
            // ⚠️⚠️ **E o `border_color` logo abaixo já escrevia o report inteiro** — *«without it
            //    a Normal-state Cancel / Reset is bare text indistinguishable from a label»* —, só
            //    que o tema moderno desligava a coisa que a nota prometia. *Uma promessa escrita
            //    num sítio e desligada noutro.*
            (ButtonKind::Default, _) => ColorToken::Bg2,
            (ButtonKind::IconOnly { .. }, ButtonState::Hovered | ButtonState::Focused) => {
                ColorToken::BgElev
            }
            (ButtonKind::IconOnly { .. }, ButtonState::Pressed) => ColorToken::AccentSoft,
            (ButtonKind::IconOnly { .. }, _) => return None,
            (ButtonKind::Accent, ButtonState::Pressed) => ColorToken::AccentPress,
            // Hover must use the dedicated (brighter) `AccentHover`, NOT
            // `AccentSoft` — AccentSoft is a dark, desaturated surface
            // tone (L≈0.28) that read as "disabled" under the cursor.
            (ButtonKind::Accent, ButtonState::Hovered) => ColorToken::AccentHover,
            (ButtonKind::Accent, _) => ColorToken::Accent,
            (ButtonKind::Danger, ButtonState::Pressed | ButtonState::Hovered) => {
                ColorToken::DangerSoft
            }
            (ButtonKind::Danger, _) => ColorToken::Danger,
        };
        Some(token)
    }

    /// Resolve the discrete outline token. Secondary / ghost
    /// (`Default`) buttons always carry a `Border` outline so they
    /// read as buttons even with no fill — without it a Normal-state
    /// Cancel / Reset is bare text indistinguishable from a label.
    /// Filled CTAs (`Accent` / `Danger`) need no outline (the fill is
    /// the affordance); `IconOnly` toolbar chips stay frameless.
    /// `None` ⇒ no outline.
    pub fn border_color(&self, theme: Theme) -> Option<TokenColor> {
        match self.kind {
            ButtonKind::Default if self.state != ButtonState::Disabled => {
                Some(ColorToken::Border.resolve(theme))
            }
            _ => None,
        }
    }

    /// Show a focus ring? True only when focused (per WCAG 2.4.7).
    pub fn focus_ring(&self) -> bool {
        self.state == ButtonState::Focused
    }
}
