//! **O CHROME LEGADO** — o interruptor que esconde os clusters da barra de topo e o trilho
//! lateral enquanto a barra de menus nova (D2) não existe.
//!
//! ⚠️ **É um interruptor e não uma remoção, e a razão está MEDIDA:** nenhum atalho de teclado
//! alcança as pílulas de módulo (Vector, Motion, Flip, Play…) e a paleta de
//! comandos é um widget genérico, não um catálogo global. Apagá-las deixaria o app sem forma de
//! abrir um módulo, e bloquearia os smokes de todas as outras linhas.
//!
//! ⏳ **Condição de saída:** quando a barra de menus existir, o campo `view.legacy_chrome` morre e
//! o que ele esconde é apagado de vez. Enquanto ele existir, é dívida **nomeada**.
//!
//! Cortado do `input_dispatch/keyboard.rs` pelo tecto de LOC (614/600) — e o corte é por
//! responsabilidade: aquele ficheiro roteia teclas, este sabe o que é o chrome legado.

impl crate::App {
    /// Alterna o chrome legado (a porta é do hero) e diz no terminal em que estado ficou. `false`
    /// quando não há `HeroScreen` ou a apresentação do quadro a recusa (a tecla não é consumida).
    pub(crate) fn toggle_legacy_chrome(&mut self) -> bool {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        let Some(on) = hero.toggle_legacy_chrome() else {
            return false;
        };
        let state = if on { "VISIVEL" } else { "fora" };
        eprintln!("[ui] chrome legado (barra de topo + trilho): {state}");
        true
    }
}
