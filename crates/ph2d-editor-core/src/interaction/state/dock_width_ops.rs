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

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐ **A FAIXA DO FUNDO** — o irmão VERTICAL das duas colunas (Enio, 2026-08-31: *«em nodes,
// arrastar a timeline na vertical deve ajustar o canvas dos nós e não deixar espaços vazios nem
// sobrepor os nodes»*).
//
// ⛔⛔ **O que ele arrastava não era uma banda: era o painel a SOLTAR-SE dela.** A costura do
// timeline escrevia um rect livre (`TimelinePanelState::rect`), e a partir daí o painel ignorava a
// faixa que o layout lhe dava — daí o espaço vazio por cima dele na foto, e a sobreposição no
// outro sentido. *Uma borda de painel docado que devolve um rect livre é um painel que deixa de
// estar docado quando se lhe toca.*
//
// ⇒ o topo da faixa passa a ser uma **costura**, exactamente como a borda interior de uma coluna:
// ela escreve uma MEDIDA, e quem partilha a banda (o grafo de nós, por `dock_timeline_into_motion`)
// segue por construção.
// ─────────────────────────────────────────────────────────────────────────────

impl WidgetStore {
    /// **A altura AUTORADA da faixa do fundo** — a de fábrica até alguém arrastar o topo dela.
    ///
    /// ⚠️ Clampada na PORTA, pela mesma razão da [`Self::dock_width`]: uma faixa que possa encolher
    /// a zero não deixa borda para agarrar de volta.
    #[must_use]
    pub fn dock_bottom_h(&self) -> f32 {
        crate::math::safe_clamp(
            self.dock_h_bottom
                .unwrap_or(crate::screens::layout::TIMELINE_DOCK_H),
            Self::DOCK_H_MIN,
            Self::DOCK_H_MAX,
        )
    }

    /// ⭐ **A ESCOLHA do artista, ou `None`** — o irmão de [`Self::dock_bottom_h`]. A distinção
    /// decide o que se GRAVA; ver [`Self::dock_width_choice`].
    #[must_use]
    pub fn dock_bottom_h_choice(&self) -> Option<f32> {
        self.dock_h_bottom
    }

    /// Escreve a altura da faixa do fundo, já clampada.
    pub fn set_dock_bottom_h(&mut self, h: f32) {
        self.dock_h_bottom = Some(crate::math::safe_clamp(
            h,
            Self::DOCK_H_MIN,
            Self::DOCK_H_MAX,
        ));
    }

    /// ⚠️ **O mínimo é o do painel do timeline** (`geom::MIN_H`, privado dele): abaixo de 120 px o
    /// cabeçalho, a fila de transporte e a régua deixam de caber juntos. ⛔ O número está aqui e
    /// não lá porque quem clampa é a PORTA da medida, e a faixa pode um dia ter outro inquilino —
    /// mas os dois têm de concordar, e há gate a exigi-lo.
    const DOCK_H_MIN: f32 = 120.0; // LITERAL-PX-OK: piso da faixa do fundo (= `timeline::geom::MIN_H`)
    /// ⚠️ O tecto é o mesmo critério da largura de coluna: uma faixa nunca come a área de desenho
    /// inteira. Numa janela baixa o layout aperta-o ainda mais contra a banda de chrome.
    const DOCK_H_MAX: f32 = 720.0; // LITERAL-PX-OK: tecto da faixa do fundo
}

impl WidgetStore {
    /// ⭐ **Publica o que a fila de ferramentas não coube** — ver o campo.
    pub fn set_tool_overflow(&mut self, entries: Vec<crate::widget::ToolRailEntry>) {
        self.tool_overflow = entries;
    }

    /// O que ficou atrás do `⋯` neste quadro.
    #[must_use]
    pub fn tool_overflow(&self) -> &[crate::widget::ToolRailEntry] {
        &self.tool_overflow
    }

    /// ⭐⭐⭐ **Publica o que o módulo com o canvas contribui neste quadro** — ver os dois campos.
    ///
    /// `menus` são os **pulldowns da fila** (a metade 2 da D2); `contrib` é o que ele acrescenta a
    /// menus que **já existem** (a metade 1 — hoje o *File*).
    ///
    /// ⚠️ **UMA porta para as duas metades, de propósito.** Elas são a mesma pergunta da D2 lida
    /// duas vezes (*este comando é do app ou do editor?*), e duas portas dariam duas leis do
    /// «escrito em todo quadro» a envelhecer em separado.
    ///
    /// ⚠️ **Chamado em todo quadro, vazio incluído.** Escrever só quando há módulo armado deixaria
    /// o chip do 3D na fila depois de o módulo fechar — pintado, e a despachar para um painel que
    /// já não existe — e uma linha *Export Draft* no menu *File* que não exporta nada.
    pub fn set_area_commands(
        &mut self,
        menus: Vec<crate::interaction::AreaMenu>,
        contrib: Vec<(
            crate::interaction::ContextMenuKind,
            Vec<crate::widget::ToolRailEntry>,
        )>,
    ) {
        self.area_menus = menus;
        self.menu_contrib = contrib;
    }

    /// Os pulldowns que a área contribui neste quadro — um chip por cada, na ordem.
    #[must_use]
    pub fn area_menus(&self) -> &[crate::interaction::AreaMenu] {
        &self.area_menus
    }

    /// O corpo do pulldown `slot`, ou vazio se não há tal pulldown neste quadro.
    ///
    /// ⚠️ Vazio e não `panic`: o `slot` vem de um `ContextMenuRequest` que sobrevive ao quadro em
    /// que foi aberto, e fechar o módulo com o menu aberto é um gesto legítimo.
    #[must_use]
    pub fn area_menu_rows(&self, slot: u8) -> &[crate::widget::ToolRailEntry] {
        self.area_menus
            .get(usize::from(slot))
            .map_or(&[], |m| &m.rows)
    }

    /// O que um módulo acrescenta a `kind` neste quadro — vazio quando ninguém contribui.
    #[must_use]
    pub fn menu_contrib(
        &self,
        kind: crate::interaction::ContextMenuKind,
    ) -> &[crate::widget::ToolRailEntry] {
        self.menu_contrib
            .iter()
            .find(|(k, _)| *k == kind)
            .map_or(&[], |(_, rows)| rows)
    }
}
