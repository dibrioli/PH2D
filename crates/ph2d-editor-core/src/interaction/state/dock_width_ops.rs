//! ⭐ **A LARGURA DAS DUAS COLUNAS** — a divisória que o artista arrasta (decisão **D4**).
//!
//! ⚠️ **Cortado do `chrome_ops.rs` em 2026-08-30 pelo tecto de LOC (706/700), e o corte é por
//! RESPONSABILIDADE:** aquele ficheiro é o saco do chrome (cor de widget, raio, vsync, tamanho de
//! botão) e isto é **uma** pergunta com dois lados — *quanto ela mede* e *o artista escolheu?*.
//!
//! ⛔ **As duas leituras não são a mesma, e a diferença decide o que se GRAVA:** a
//! [`WidgetStore::dock_width`] devolve sempre um número (o default quando ninguém arrastou) e a
//! [`WidgetStore::dock_width_choice`] devolve **a escolha**. Persistir a primeira escreveria o
//! default como se fosse uma decisão do artista.

use super::WidgetStore;

impl WidgetStore {
    /// **A largura AUTORADA de uma coluna docada** — a de fábrica até alguém arrastar a borda.
    ///
    /// ⚠️ Clampada na PORTA e não em cada leitor: uma coluna que possa encolher a zero ou comer a
    /// janela é estado inalcançável de volta (não sobra borda para agarrar).
    pub fn dock_width(&self, side: crate::screens::layout::DockSide) -> f32 {
        use crate::screens::layout::{ChromeBands, DockSide};
        let stored = match side {
            DockSide::Left => self.dock_w_left,
            DockSide::Right => self.dock_w_right,
        };
        let base = match side {
            DockSide::Left => ChromeBands::DEFAULT.left_dock_w,
            DockSide::Right => ChromeBands::DEFAULT.right_dock_w,
        };
        crate::math::safe_clamp(stored.unwrap_or(base), Self::DOCK_W_MIN, Self::DOCK_W_MAX)
    }

    /// ⭐ **A ESCOLHA do artista, ou `None`** — o irmão de [`Self::dock_width`], que devolve
    /// sempre um número (o default quando ninguém arrastou).
    ///
    /// ⚠️ **A distinção decide o que se GRAVA.** Persistir o valor de `dock_width` escreveria o
    /// default como se fosse uma escolha — e no dia em que o default mudasse, toda arrumação
    /// gravada continuaria a prender a coluna no número velho, sem ninguém ter pedido nada.
    #[must_use]
    pub fn dock_width_choice(&self, side: crate::screens::layout::DockSide) -> Option<f32> {
        match side {
            crate::screens::layout::DockSide::Left => self.dock_w_left,
            crate::screens::layout::DockSide::Right => self.dock_w_right,
        }
    }

    /// Escreve a largura de uma coluna, já clampada.
    pub fn set_dock_width(&mut self, side: crate::screens::layout::DockSide, w: f32) {
        let w = crate::math::safe_clamp(w, Self::DOCK_W_MIN, Self::DOCK_W_MAX);
        match side {
            crate::screens::layout::DockSide::Left => self.dock_w_left = Some(w),
            crate::screens::layout::DockSide::Right => self.dock_w_right = Some(w),
        }
    }

    /// ⚠️ **O mínimo é o do painel** (`PANEL_MIN_W_PX`, 220) — abaixo dele o cabeçalho e uma linha
    /// deixam de caber juntos. O máximo é medido pelo mesmo critério do `clamp_panel_rect`: 70 % de
    /// uma janela de referência, para uma coluna nunca comer a área de desenho inteira.
    const DOCK_W_MIN: f32 = ph2d_tokens::PANEL_MIN_W_PX;
    const DOCK_W_MAX: f32 = 720.0; // LITERAL-PX-OK: teto de largura de coluna docada
}
