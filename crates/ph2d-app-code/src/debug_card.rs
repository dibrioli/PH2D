//! A cola do quadro do depurador ([`crate::debug`]): a altura, a pintura e o clique moram no
//! `ph2d_panel_code::debug_card` (plano 14, as três aparências); aqui o quadro liga-se à aba.

use crate::commands::{CMD_CONTINUE, CMD_NEXT_LINE};
use crate::doc::CodeDoc;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::debug_card::CardClick;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

impl CodeDoc {
    /// Os cartões da aba: o código em cima, o depurador, a Bancada e a faixa, encolhidos pela ordem
    /// até o código guardar as suas linhas ([`ph2d_panel_code::stack`]).
    pub(crate) fn stack(&self, area: Rect, open: bool) -> ph2d_panel_code::stack::Placed {
        use ph2d_panel_code::stack::{code_least, tab};
        let debug = self
            .debug
            .card
            .as_ref()
            .map(|c| self.debug.view.want(c, area));
        let band = self.console_view.want(open, area.h);
        let code = code_least(self.find.bar_height());
        let wants = [self.bench_h, debug.unwrap_or_default(), band];
        tab(area, code, crate::cards::margin(), wants)
    }

    /// O cartão do depurador, entre o código e a faixa de baixo (`None` = nenhum).
    pub(crate) fn debug_rect(&self, area: Rect, console_open: bool) -> Option<Rect> {
        self.stack(area, console_open).debug
    }

    /// Pinta o quadro em `r` (dentro do seu molde, [`crate::cards::open`]).
    pub(crate) fn paint_debug_card(
        &mut self,
        r: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let Some(c) = self.debug.card.clone() else {
            self.debug.view.painted = Default::default();
            return;
        };
        let ids = [CMD_CONTINUE, CMD_NEXT_LINE];
        self.debug
            .view
            .paint(&c, ids, r, &mut self.view, scene, ts, *theme);
    }

    /// Um clique no quadro: os botões pedem, a pilha leva à linha, um valor copia-se, ▸ e o
    /// puxador respondem; o resto engole o clique. `true` = tomado.
    pub(crate) fn debug_press(&mut self, x: f32, y: f32) -> bool {
        match self.debug.view.press(x, y) {
            None => return false,
            Some(CardClick::Ask(a)) => self.debug.ask = Some(a),
            Some(CardClick::Jump(line)) => {
                ph2d_code_buffer::goto_line(&mut self.buffer, line as usize);
                self.reveal();
            }
            Some(CardClick::Copy(v)) => self.debug.copy = Some(v),
            Some(CardClick::Taken) => {}
        }
        true
    }
}
