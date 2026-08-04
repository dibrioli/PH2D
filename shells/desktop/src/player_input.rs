//! **O dedo do jogador** (W3) — a política pura que transforma teclas seguradas
//! num `drive`.
//!
//! Módulo próprio, e puro, pelo motivo do `flip_peek`: uma política que mora
//! dentro do `key_input` é uma política que **nenhum teste alcança** (o
//! `winit::KeyEvent` tem campo privado e não pode ser construído fora do winit —
//! foi essa exata parede que fez o corpo do `on_keyboard_input` virar `key_input`).
//!
//! # ⚠️ Duas teclas, não um número
//!
//! O estado guardado são os dois BOTÕES, e o `drive` é derivado deles. Guardar o
//! número já somado pareceria mais simples e erraria no caso que acontece o
//! tempo todo: segurar as duas setas ao mesmo tempo tem de dar **zero**, e
//! soltar uma delas tem de devolver a direção da outra — o que um acumulador
//! `+1`/`−1` só acerta se nenhum evento de release se perder. Um `Up` perdido
//! (troca de janela, foco roubado) deixaria o personagem andando sozinho para
//! sempre.
//!
//! # ⚠️ E ele NÃO consome a tecla
//!
//! A seta já tem dono na shell (o nudge de nó do Vector, gateado em
//! `vector_tool_active`). Observar sem consumir é o que garante que esta wave
//! seja **zero-regressão** para quem nunca criou um player: o evento segue o
//! caminho de sempre, e o que muda é só um par de bools que ninguém lê numa cena
//! sem player.

use winit::keyboard::KeyCode;

/// As teclas de caminhada que estão SEGURADAS agora.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PlayerKeys {
    left: bool,
    right: bool,
}

impl PlayerKeys {
    /// O eixo de caminhada em `[-1, 1]`, derivado dos dois botões.
    ///
    /// As duas seguradas dão **zero** — a resposta que o jogador espera, e a que
    /// um acumulador não daria.
    pub(crate) fn drive(self) -> f32 {
        f32::from(u8::from(self.right)) - f32::from(u8::from(self.left))
    }

    /// Registra um press/release. Devolve `true` se a tecla é de caminhada —
    /// informativo, **nunca** consumo (ver o aviso do módulo).
    pub(crate) fn key(&mut self, code: KeyCode, pressed: bool) -> bool {
        match code {
            KeyCode::ArrowLeft | KeyCode::KeyA => {
                self.left = pressed;
                true
            }
            KeyCode::ArrowRight | KeyCode::KeyD => {
                self.right = pressed;
                true
            }
            _ => false,
        }
    }

    /// Solta tudo. Chamado quando a janela perde o foco: uma tecla cujo `Up`
    /// nunca chega é um personagem que anda sozinho até alguém tocá-la de novo.
    pub(crate) fn release_all(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_held_arrow_drives_and_releasing_it_stops() {
        let mut k = PlayerKeys::default();
        assert_eq!(k.drive(), 0.0);
        assert!(k.key(KeyCode::ArrowRight, true));
        assert_eq!(k.drive(), 1.0);
        k.key(KeyCode::ArrowRight, false);
        assert_eq!(k.drive(), 0.0);
        k.key(KeyCode::ArrowLeft, true);
        assert_eq!(k.drive(), -1.0);
    }

    /// ⚠️ **As duas ao mesmo tempo dão ZERO, e soltar uma devolve a outra.**
    ///
    /// É o gate que o acumulador `+1`/`−1` não passa sem um contador perfeito de
    /// releases — e é por isso que o estado guardado são os BOTÕES.
    #[test]
    fn both_at_once_cancel_and_letting_one_go_gives_the_other() {
        let mut k = PlayerKeys::default();
        k.key(KeyCode::ArrowLeft, true);
        k.key(KeyCode::ArrowRight, true);
        assert_eq!(k.drive(), 0.0, "as duas seguradas: parado");
        k.key(KeyCode::ArrowLeft, false);
        assert_eq!(k.drive(), 1.0, "soltar a esquerda devolve a direita");
    }

    /// Um press repetido (auto-repeat) não empilha nada.
    #[test]
    fn autorepeat_is_idempotent() {
        let mut k = PlayerKeys::default();
        for _ in 0..10 {
            k.key(KeyCode::KeyD, true);
        }
        assert_eq!(k.drive(), 1.0);
        k.key(KeyCode::KeyD, false);
        assert_eq!(k.drive(), 0.0);
    }

    /// Perder o foco solta tudo — senão a tecla cujo `Up` não chega deixa o
    /// personagem andando sozinho.
    #[test]
    fn losing_focus_releases_everything() {
        let mut k = PlayerKeys::default();
        k.key(KeyCode::ArrowRight, true);
        k.release_all();
        assert_eq!(k.drive(), 0.0);
    }

    /// Teclas que não são de caminhada não são reclamadas.
    #[test]
    fn other_keys_are_not_claimed() {
        let mut k = PlayerKeys::default();
        assert!(!k.key(KeyCode::Space, true));
        assert!(!k.key(KeyCode::ArrowUp, true));
        assert_eq!(k.drive(), 0.0);
    }
}
