//! **O ARRASTO da janela do Input Map** — irmão exacto do `fill_drag::arm_fill_modal_drag_*`.
//!
//! ⚠️ **Máquina de estados do SHELL, e não um evento de widget**, pela mesma razão que a do Fill: um
//! `Click` na faixa do título é *"largou sem mover"*; o **arrasto** precisa de ver cada `CursorMoved`
//! entre o Down e o Up, e isso não é uma coisa que um widget observe. O handler de chrome consome o
//! clique nu (para nunca vazar), e este ficheiro faz o movimento.

use std::cell::Cell;

use ph2d_editor::ids;

thread_local! {
    /// O último ponto do cursor enquanto a janela está a ser arrastada. `None` = não há arrasto.
    static INPUT_MAP_DRAG: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

impl crate::App {
    /// Um Primary Down sobre a faixa do título arma o arrasto. Devolve `true` (consome o Down) para
    /// que a janela **se mova em vez de o Down fazer outra coisa** — e para que ela nunca feche a
    /// meio do movimento.
    pub(crate) fn arm_input_map_drag_if_on_handle(&mut self, px: f32, py: f32) -> bool {
        let on_handle = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.hit_index.hit(px, py))
            == Some(ids::INPUT_MAP_HANDLE);
        if !on_handle {
            return false;
        }
        INPUT_MAP_DRAG.with(|c| c.set(Some((px, py))));
        true
    }

    /// `CursorMoved` durante o arrasto: desloca a janela pelo delta do cursor. Devolve `true`
    /// (consome o movimento) enquanto arrasta, para não fazer pan nem conduzir um gizmo por baixo.
    pub(crate) fn input_map_drag_move(&mut self, px: f32, py: f32) -> bool {
        let Some((lx, ly)) = INPUT_MAP_DRAG.with(Cell::get) else {
            return false;
        };
        INPUT_MAP_DRAG.with(|c| c.set(Some((px, py))));
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.move_input_map(px - lx, py - ly);
        }
        true
    }

    /// Primary Up: termina o arrasto. No-op quando não há nenhum.
    pub(crate) fn input_map_drag_up(&mut self) {
        INPUT_MAP_DRAG.with(|c| c.set(None));
    }
}

impl crate::App {
    /// **A ESCUTA TAMBÉM APANHA UM BOTÃO DE COMANDO** (plano 30 §0.1: *"qualquer objecto do game"*).
    ///
    /// ⚠️ **Aqui e não no despacho de teclado**, porque um gamepad não tem despacho: o adaptador do
    /// `gilrs` bombeia eventos para o retrato de dispositivos e ninguém os "encaminha". A pergunta
    /// certa é feita **uma vez por quadro**, sobre a **BORDA** (`pressed`, não `held`) — com `held`,
    /// um botão que já estivesse em baixo quando o artista carregou em `Bind…` ligar-se-ia sozinho,
    /// sem ele ter feito nada.
    ///
    /// ⚠️ **Só corre com a escuta armada**, e sai cedo caso contrário: é o mesmo custo de um `if`
    /// num quadro normal.
    pub(crate) fn poll_input_map_pad_binding(&mut self) {
        let Some(armed) = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.input_map_listening())
        else {
            return;
        };
        let Some(hit) = ph2d_input::GamepadButton::ALL
            .iter()
            .copied()
            .find(|b| self.input.gamepad.pressed(*b))
        else {
            return;
        };
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        hero.store.stop_listening();
        if let Some(a) = hero.input_map.get_mut(armed) {
            let b = ph2d_input::Binding::PadButton(hit);
            // ⚠️ Não duplica, pela razão do teclado: duas linhas iguais no painel seriam
            // indistinguíveis ao apagar.
            if !a.bindings.contains(&b) {
                a.bindings.push(b);
            }
        }
    }
}
