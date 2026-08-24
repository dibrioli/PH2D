//! **A ESCUTA DO INPUT MAP, no topo do teclado da shell** — irmão de [`super::keyboard`], cortado
//! por teto de LOC (HR-18) e por assunto.
//!
//! # ⛔⛔ Report do Enio (2026-08-24): *"Os atalhos de editor estão em conflito com o Bind"*
//!
//! A primeira versão pôs esta guarda como o primeiro ramo do `dispatch_key` — o primeiro **dentro
//! do editor-core**. Mas o `key_input` da shell tem **~20 `return`** antes de chegar lá: o `P` do
//! menu radial, o `W` do painel de mundo, o Espaço do transporte, o peek do Flip. Nenhuma dessas
//! teclas chegava ao editor-core, então carregar nelas durante o `Bind…` executava o atalho e **não
//! ligava nada**.
//!
//! ⚠️ E havia gate a provar a ordem — `a_captured_key_never_fires_the_editors_shortcut`, com o
//! `Tab`. Ele estava **certo e insuficiente**: media a posição dentro de um PEDAÇO da cadeia.
//! *A ordem é a feature, e ela tem de estar no topo da cadeia REAL.*
//!
//! A LEI é uma só (`ph2d_editor::interaction::capture_if_listening`); aqui está o **primeiro**
//! chamador dela, e o `dispatch_key` é o segundo — dois chamadores de uma porta, nunca duas leis.

use ph2d_host::KeyKind;
use winit::keyboard::PhysicalKey;

impl crate::App {
    /// **Consome a tecla se o `Bind…` estiver armado.** `true` ⇒ quem chamou tem de parar ali.
    ///
    /// ⚠️ **Só o `Down`**: um `Up` não escolhe tecla nenhuma, e consumi-lo deixaria o botão que a
    /// armou preso — a mesma assimetria que o `player_input` da shell sempre teve.
    ///
    /// ⚠️ **Normalizador TOTAL** (`winit_to_input_keycode`), e não o do editor: com este último, o
    /// `W`/`S`/`Z`/`Q` chegariam como `None` e cairiam para os atalhos — o defeito reportado, com
    /// outra causa e o mesmo sintoma.
    pub(crate) fn capture_binding_if_listening(
        &mut self,
        physical_key: PhysicalKey,
        kind: KeyKind,
    ) -> bool {
        if kind != KeyKind::Down {
            return false;
        }
        let PhysicalKey::Code(code) = physical_key else {
            return false;
        };
        let Some(k) = crate::keymap::winit_to_input_keycode(code) else {
            return false;
        };
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        let Some(evt) = ph2d_editor::interaction::capture_if_listening(&mut hero.store, k) else {
            return false;
        };
        hero.apply_event(evt);
        true
    }
}
