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

/// As teclas que estão SEGURADAS agora.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PlayerKeys {
    left: bool,
    right: bool,
    jump: bool,
}

impl PlayerKeys {
    /// O eixo de caminhada em `[-1, 1]`, derivado dos dois botões.
    ///
    /// As duas seguradas dão **zero** — a resposta que o jogador espera, e a que
    /// um acumulador não daria.
    pub(crate) fn drive(self) -> f32 {
        f32::from(u8::from(self.right)) - f32::from(u8::from(self.left))
    }

    /// **O botão de pulo está pressionado agora.**
    ///
    /// ⚠️ Isto é o ESTADO, não a borda — a lei do pulo deriva a borda sozinha
    /// ([`ph2d_physics_ecs::JumpState`]), e por bom motivo: quem a derivasse
    /// aqui precisaria de uma segunda memória do mesmo fato, e as duas
    /// divergiriam no primeiro dispatch que deve mais de um tick.
    pub(crate) fn jump(self) -> bool {
        self.jump
    }

    /// **O dedo do jogador, inteiro.**
    ///
    /// ⚠️ Porta ÚNICA de propósito: entregar `drive` e `jump` por caminhos
    /// separados convidaria um deles a ser esquecido num sítio de despacho, e o
    /// modo de falha disso é **metade do controle morta em silêncio** — a
    /// família que esta linha já pagou nos knobs de zona. Um botão novo entra
    /// aqui e chega a todo mundo.
    pub(crate) fn input(self) -> ph2d_physics_ecs::PlayerInput {
        ph2d_physics_ecs::PlayerInput {
            drive: self.drive(),
            jump: self.jump(),
        }
    }

    /// Registra um press/release. Devolve `true` se a tecla é do player —
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
            // ⚠️ **Seta para CIMA e `Z`, e o Espaço NÃO** — ele é o Play/Pause do
            // transporte, e um platformer cujo botão de pulo também pausa a cena
            // é uma tecla com dois donos. `W` também não: ela abre o painel de
            // MUNDO da física. As duas escolhas são por CONFLITO medido, não por
            // gosto — as outras teclas do repo foram varridas antes.
            KeyCode::ArrowUp | KeyCode::KeyZ => {
                self.jump = pressed;
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

    /// Teclas que não são do player não são reclamadas.
    ///
    /// ⚠️ **O ESPAÇO está nesta lista de propósito** — ele é o Play/Pause do
    /// transporte, e o gate é o que impede alguém de o adotar como botão de
    /// pulo por parecer natural.
    #[test]
    fn other_keys_are_not_claimed() {
        let mut k = PlayerKeys::default();
        assert!(!k.key(KeyCode::Space, true));
        assert!(!k.key(KeyCode::KeyW, true), "W abre o painel de MUNDO");
        assert_eq!(k.drive(), 0.0);
        assert!(!k.jump());
    }

    /// **O pulo é um botão SEGURADO**, e as duas teclas são a mesma coisa.
    ///
    /// Segurar é o que a altura variável lê (soltar durante a subida corta o
    /// pulo), então guardar só a borda aqui perderia a metade que importa.
    #[test]
    fn the_jump_button_is_held_and_released() {
        let mut k = PlayerKeys::default();
        assert!(!k.jump());
        assert!(k.key(KeyCode::ArrowUp, true));
        assert!(k.jump());
        k.key(KeyCode::ArrowUp, false);
        assert!(!k.jump());
        assert!(k.key(KeyCode::KeyZ, true));
        assert!(k.jump(), "Z e a seta para cima sao o MESMO botao");
    }

    /// E perder o foco solta o pulo junto com as setas.
    #[test]
    fn losing_focus_releases_the_jump_button_too() {
        let mut k = PlayerKeys::default();
        k.key(KeyCode::KeyZ, true);
        k.release_all();
        assert!(!k.jump());
    }
}
