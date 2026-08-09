//! **O BOTÃO que o `CanvasPointer` não carrega** — o botão direito do Grid Stamp apaga a célula.
//!
//! Irmão de [`super::painter_canvas_mods`], que responde à MESMA pergunta para as TECLAS: o
//! [`ph2d_editor_core::tool::CanvasPaintTool`] é contrato congelado (§6) e o ponteiro dele não leva
//! nem modificador nem botão, então os dois viajam **fora de banda** — o shell os amostra e os empurra
//! ao tool logo antes da entrega. Módulo próprio, e não mais um bloco no `painter_canvas_input`, por
//! duas razões: aquele arquivo bateu o teto de LOC, e um botão não é uma tecla — dizer que é faria a
//! prosa daquele header mentir sobre o próprio conteúdo.

use crate::App;
use ph2d_tool_painter::PainterTool;

impl App {
    /// **Grid Stamp — o botão direito APAGA.** Um Down secundário arma o gesto de apagar e o entrega
    /// pelo caminho NORMAL de traço, então o arrasto (via `CursorMoved`), a coalescência, o undo de um
    /// passo e o carimbo célula-a-célula vêm todos de graça — a única diferença entre pintar e apagar
    /// é o `blend` que o carimbo escolhe.
    ///
    /// ⚠️ **Só reivindica o clique no Grid Stamp** (`stamps_on_a_grid`). Em todo outro método o botão
    /// direito continua sendo do menu de contexto e dos consumidores que já existiam (o conta-gotas, a
    /// borracha de proteção, os menus de ponto da curva, o fim da polilinha do Line) — roubá-lo em
    /// geral seria tirar comportamento em troca de nada.
    ///
    /// ⚠️ **E desarma se o traço não abriu**: um Down fora da pegada da sprite não vira gesto, e deixar
    /// o flag ligado faria o PRÓXIMO traço — de botão esquerdo — apagar.
    pub(crate) fn painter_grid_erase_down(&mut self, px: f32, py: f32) -> bool {
        if !self.painter_stamps_on_a_grid() {
            return false;
        }
        self.set_painter_grid_erase(true);
        let started = self.painter_canvas_down(px, py, 1.0);
        if !started {
            self.set_painter_grid_erase(false);
        }
        started
    }

    /// Fecha um gesto de apagar do Grid Stamp.
    ///
    /// ⚠️ **A ordem é load-bearing:** o `painter_canvas_up` entrega a fase `Up`, que ainda carimba a
    /// cauda do traço — desarmar antes dele faria o ÚLTIMO carimbo pintar em vez de apagar.
    pub(crate) fn painter_grid_erase_up(&mut self) {
        self.painter_canvas_up();
        self.set_painter_grid_erase(false);
    }

    /// O Painter está ativo com o método que carimba numa grade? (downcast escopado, como o irmão
    /// `painter_coalesces_motion` logo abaixo.)
    fn painter_stamps_on_a_grid(&mut self) -> bool {
        self.gfx
            .as_mut()
            .and_then(|g| g.tools.active_mut())
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
            .is_some_and(|p| p.stamps_on_a_grid())
    }

    /// Empurra o botão para o tool — fora de banda, porque o `CanvasPointer` é contrato congelado e
    /// não o carrega (o precedente exato do `set_line_constrain`, em `painter_canvas_mods`).
    fn set_painter_grid_erase(&mut self, on: bool) {
        if let Some(p) = self
            .gfx
            .as_mut()
            .and_then(|g| g.tools.active_mut())
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        {
            p.set_grid_stamp_erase(on);
        }
    }
}
