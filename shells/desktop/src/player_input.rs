//! **O DEDO DO JOGADOR** — e ele deixou de ser um punhado de teclas cravadas.
//!
//! # O que esta wave mudou (plano 30 W5)
//!
//! Até 2026-08-24 este módulo tinha a lista de teclas **escrita à mão** (`←/A`, `→/D`, `↑/Z`,
//! `↓/S`, `Q`, `R`) e derivava o `drive` delas. Agora ele **resolve o `InputMap` do projecto**: o
//! artista liga as teclas na janela flutuante, e o mesmo `PlayerInput` sai do outro lado.
//!
//! ⭐⭐ **O `PlayerKeys` foi REMOVIDO, e não mantido ao lado.** Ele e o retrato de dispositivos do
//! `ph2d-input` seriam **duas memórias da mesma mão**, e elas divergiriam no primeiro `Up` que uma
//! recebesse e a outra não — com o sintoma a ser *"às vezes o boneco anda sozinho"*, que é o mais
//! caro de diagnosticar. *Uma lei escrita em dois sítios ainda não é uma lei.*
//!
//! # ⭐ A fita não muda de significado, e isso não é sorte — é o desenho de origem
//!
//! O plano tinha escrito uma **LEI Nº 1** (*"a fita grava a acção resolvida, nunca a tecla"*) como
//! um risco a guardar. A medição do `TapeWire` mostrou que ele **já era assim**: ela grava
//! `(drive: f32, botões: u8)` — semântico, nunca um keycode. ⇒ remapear `jump` de `Z` para `Espaço`
//! **não toca** em gravação nenhuma, e o hash de replay do `physics_ecs_c9` fica onde estava. *Uma
//! nota de risco escrita sem medir descreve um perigo que o desenho já tinha evitado.*
//!
//! # ⚠️ O que continua igual, de propósito
//!
//! - **Duas teclas, não um número**: o `drive` sai da SUBTRACÇÃO de duas acções
//!   ([`ph2d_input::Input::axis`]), então as duas seguradas dão **zero** e soltar uma devolve a
//!   direcção da outra. Um acumulador `+1`/`−1` erraria no primeiro `Up` perdido.
//! - **Estado, não borda**: `jump`/`down`/`dash`/`grab` entregam o que está premido AGORA. A lei do
//!   controlador deriva as bordas sozinha, e quem as derivasse aqui precisaria de uma segunda
//!   memória do mesmo facto — as duas divergiriam no primeiro dispatch que devesse mais de um tique.
//! - **Uma porta só**: [`App::resolve_player_input`] entrega o dedo INTEIRO. Entregar `drive` e
//!   `jump` por caminhos separados convidaria um deles a ser esquecido num sítio de despacho, e o
//!   modo de falha disso é **metade do controlo morta em silêncio**.

use ph2d_input::{
    ActionState, Input, PLAYER_DASH, PLAYER_DOWN, PLAYER_GRAB, PLAYER_JUMP, PLAYER_MOVE_LEFT,
    PLAYER_MOVE_RIGHT,
};

impl crate::App {
    /// **Resolve o mapa e devolve o dedo do jogador deste quadro.**
    ///
    /// ⚠️ **O `mem::take` não é um truque — é a forma de não ter dois donos.** O mapa vive no
    /// `HeroScreen` (dentro de `self.gfx`) e o estado resolvido vive na `App`; tocar nos dois ao
    /// mesmo tempo seria emprestar `self` duas vezes. Tirar o estado, resolver contra o mapa, e
    /// repô-lo mantém **um** dono para cada coisa em vez de clonar o mapa a cada quadro.
    ///
    /// ⚠️ **Sem `HeroScreen` (a GPU ainda não subiu) o dedo é o NEUTRO**, e não a última leitura:
    /// um personagem que continuasse a andar durante o arranque seria movimento que ninguém pediu.
    pub(crate) fn resolve_player_input(&mut self) -> ph2d_physics_ecs::PlayerInput {
        let mut actions: ActionState = std::mem::take(&mut self.input_actions);
        let resolved = match self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) {
            Some(hero) => {
                actions.tick(&hero.input_map, &self.input);
                let input = Input::new(&hero.input_map, &actions);
                ph2d_physics_ecs::PlayerInput {
                    drive: input.axis(PLAYER_MOVE_LEFT, PLAYER_MOVE_RIGHT),
                    jump: input.pressed(PLAYER_JUMP),
                    down: input.pressed(PLAYER_DOWN),
                    dash: input.pressed(PLAYER_DASH),
                    grab: input.pressed(PLAYER_GRAB),
                }
            }
            None => ph2d_physics_ecs::PlayerInput::default(),
        };
        self.input_actions = actions;
        resolved
    }
}

#[cfg(test)]
#[path = "player_input_tests.rs"]
mod tests;
