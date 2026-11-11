//! A cola do quadro do depurador ([`crate::debug`]): a altura, a pintura e o clique moram no
//! `ph2d_panel_code::debug_card` (plano 14, as três aparências); aqui o quadro liga-se à aba.

use crate::commands::{CMD_CONTINUE, CMD_NEXT_LINE};
use crate::doc::CodeDoc;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::debug_card::{CardClick, DebugCardView};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

impl CodeDoc {
    /// As alturas da Bancada, do depurador e da faixa, encolhidas pela ordem até o código guardar as
    /// suas linhas ([`ph2d_panel_code::stack`]).
    pub(crate) fn stack(&self, area: Rect, open: bool) -> [f32; 3] {
        use ph2d_panel_code::stack::{Want, code_least, fit};
        let (v, g) = (&self.debug.view, crate::cards::margin());
        let debug = self.debug.card.as_ref().map_or(Want::default(), |c| Want {
            h: v.height(c, area),
            least: DebugCardView::least(c),
        });
        let (band, code) = (
            self.console_view.want(open, area.h),
            code_least(self.find.bar_height()),
        );
        fit(area.h, code, g, [self.bench_h, debug, band])
    }

    /// O cartão do depurador, entre o código e a faixa de baixo (`None` = nenhum).
    pub(crate) fn debug_rect(&self, area: Rect, console_open: bool) -> Option<Rect> {
        let h = self.stack(area, console_open)[1];
        if h <= 0.0 {
            return None;
        }
        let (code, _) = self.cards(area, console_open);
        Some(Rect::new(
            area.x,
            code.y + code.h + crate::cards::margin(),
            area.w,
            h,
        ))
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
