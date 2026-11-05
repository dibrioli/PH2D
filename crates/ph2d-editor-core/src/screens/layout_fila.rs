//! A fila que cobre o topo do `HeroLayout` e a região dos popovers. Irmão de `layout.rs` pelo tecto de LOC.

use super::*;

impl HeroLayout {
    /// A fila ② quando ela atravessa a janela POR CIMA dos painéis; em baixo, altura zero: quem começa
    /// «abaixo da barra» não tem nada de que se afastar (as listas saíam da janela).
    #[must_use]
    pub fn fila_que_cobre_o_topo(&self) -> Rect {
        if self.tool_bar.y <= self.draw_area.y {
            self.tool_bar
        } else {
            Rect::new(self.tool_bar.x, self.tool_bar.y, self.tool_bar.w, 0.0)
        }
    }

    /// **A região em que um popover flutuante pode nascer** — a BANDA DE CHROME, e nunca a janela
    /// inteira.
    ///
    /// Um popover longo não cabe em lado nenhum, e aí
    /// [`crate::widget::Dropdown::popover_rect_clamped`] toma *o lado com mais espaço, clampado à
    /// região que lhe deram*. Dada a JANELA, "mais espaço" é quase sempre para CIMA, e o clamp
    /// encosta a lista na borda de cima da janela — medido no frame real com o picker de token
    /// (81 linhas, `PH2D_BUILD_SMOKE=50`): a primeira linha nascia em **`y ∈ [2, 30]`**, ou seja
    /// **2 px da borda da janela**, 64 px ACIMA do painel a que pertence, por cima da barra de topo
    /// (`(14, 14, 1338, 64)`) — e `WidgetStore::panel_at` respondia `None` ali, porque ali não há
    /// painel nenhum. O `HitIndex` resolvia o clique (o popover regista por último), mas quem
    /// pergunta *"isto está sobre um painel?"* — e o gestor de janelas, que é dono da faixa de
    /// borda — não tem por que concordar. A cura não é disputar aquela faixa: é não nascer lá.
    ///
    /// A banda devolvida é **exatamente** a que o rail e os painéis docados ocupam (o `left_rail`
    /// abrange-a por construção), então um popover aterra onde o painel dele vive. Nenhuma
    /// constante nova: mover a banda de chrome move o popover junto.
    ///
    /// Largura = a da viewport, de propósito — um popover pode transbordar para o canvas na
    /// horizontal (é o que a lista de 182 px faz por cima da cena), e é só na VERTICAL que ele
    /// saía do sítio.
    ///
    /// ⛔⛔ **A âncora MUDOU em 2026-08-30, e a antiga era uma armadilha nomeada.** Ela era o
    /// `left_rail` — *«ele abrange a banda por construção»* —, e isso deixou de ser verdade no
    /// instante em que o trilho passou a poder estar **fora**: com `rail_w = 0` o rect dele é
    /// `(0, 0, 0, h)` e a região virava **a janela inteira**, que é exactamente o defeito que ela
    /// existe para curar (a 1.ª linha do picker a nascer a 2 px da borda).
    ///
    /// ⭐ Hoje ela sai da **coluna do dock**, que é a banda por construção e não por acaso: o
    /// popover nasce onde o painel dele vive, e o painel vive ali. *Uma região derivada de um
    /// chrome que pode desaparecer não é uma região — é uma coincidência.*
    #[must_use]
    pub fn popover_region(&self) -> Rect {
        Rect::new(
            self.viewport.x,
            self.inspector.y,
            self.viewport.w,
            self.inspector.h,
        )
    }
}
