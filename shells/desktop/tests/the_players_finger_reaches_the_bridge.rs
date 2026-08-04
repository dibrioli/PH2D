//! **ARCH-GATE do fio da entrada do player** (W3).
//!
//! A política de teclas é pura e tem gates próprios (`crate::player_input`), e a
//! lei tem os dela na `ph2d-platformer`. O que **nenhum dos dois alcança** é o
//! FIO: um `winit::KeyEvent` não pode ser construído fora do winit (a parede que
//! fez o corpo do `on_keyboard_input` virar `key_input`), e o `render_loop` exige
//! janela. Com as duas pontas certas e o meio desligado, tudo fica verde e o
//! personagem não anda.
//!
//! Três afirmações, e cada uma é uma forma diferente de o fio nascer morto.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// **(1) A tecla é OBSERVADA, e o bloco que a observa não CONSOME.**
///
/// A seta já tem dono (o nudge de nó do Vector), então roubá-la aqui regrediria
/// uma ferramenta que ninguém pediu para mexer. O gate afirma o corpo exato do
/// bloco — três linhas, sem `return` dentro —, porque *"observa"* e *"observa e
/// engole"* são indistinguíveis em qualquer teste que não tenha uma janela.
#[test]
fn the_walk_keys_are_observed_without_being_consumed() {
    let src = read("src/input_dispatch/keyboard.rs");
    let block = "if let PhysicalKey::Code(code) = physical_key {\n\
         \x20           self.player_keys.key(code, state == ElementState::Pressed);\n\
         \x20       }";
    assert!(
        src.contains(block),
        "o bloco que observa as teclas de caminhada tem de ser exatamente estas tres \
         linhas — um `return` ali dentro rouba a seta do nudge do Vector"
    );
}

/// **(2) O dedo INTEIRO chega ao dispatch da física.**
///
/// Sem esta linha o `PlayerKeys` seria estado que ninguém lê: os gates dele
/// continuariam verdes e o personagem ficaria parado.
///
/// ⚠️ A âncora é a **porta única** (`input()`), não `drive()` — e a diferença não
/// é cosmética: com dois getters, um dispatch que entrega só metade do dedo é
/// um build que compila, e o pulo seria a metade esquecida.
#[test]
fn the_whole_finger_is_handed_to_the_physics_dispatch() {
    let src = read("src/render_loop/mod.rs");
    assert!(
        src.contains("self.player_keys.input()"),
        "o `render_loop` tem de entregar a entrada INTEIRA (a porta unica \
         `PlayerKeys::input`) ao `physics_bridge::dispatch`"
    );
}

/// **(3) A entrega acontece ANTES da decisão de HOLD, não depois.**
///
/// ⚠️ Esta é a que não se adivinha. Com a simulação desarmada o mundo é
/// **segurado** (`PhysicsBridge::hold`) e o `dispatch` retorna cedo; entregar a
/// entrada depois desse `return` faria a tecla que o artista já estava segurando
/// ser **engolida** no instante em que ele arma o Physics — o personagem só
/// andaria depois de soltar e apertar de novo.
///
/// A afirmação é POSICIONAL sobre duas âncoras que descrevem o que o código faz
/// (a entrega e o early-out), nunca uma distância em bytes.
///
/// ⚠️ **E a âncora é a CHAMADA, não a lista de argumentos** — a versão original
/// pinava `hand_input_to_players(bridge, sim, drive);` inteiro e nasceu VERMELHA
/// na W4, quando o argumento virou `input`: o fio estava intacto e o gate
/// reprovava um rename. *Uma lista de argumentos é um proxy que expira*, e o que
/// este gate afirma é ORDEM.
///
/// ⚠️ Procurar só o NOME também não serve: ele casa com a **definição** do `fn`,
/// e no dia em que ela subisse para cima do `dispatch` o gate compararia a
/// posição errada e ficaria verde sobre o fio invertido.
#[test]
fn the_input_is_handed_over_before_the_hold_early_out() {
    let src = read("src/render_loop/physics_bridge.rs");
    let hand = src
        .match_indices("hand_input_to_players(")
        .find(|(at, _)| !src[..*at].ends_with("fn "))
        .map(|(at, _)| at)
        .expect("o dispatch tem de CHAMAR a entrega da entrada aos players");
    let hold = src
        .find("if !simulate {")
        .expect("o dispatch tem de ter o early-out do hold");
    assert!(
        hand < hold,
        "a entrada tem de ser entregue ANTES do early-out do hold: \
         entrega em {hand}, early-out em {hold}"
    );
}

/// **Controle positivo** — as três âncoras existem no arquivo que o gate lê.
///
/// Sem ele, renomear um arquivo (ou um gate que procura no lugar errado) deixaria
/// as três asserções acima verdes por vácuo, que é o modo de falha canônico de um
/// gate que lê fonte.
#[test]
fn the_files_the_gate_reads_are_the_ones_that_carry_the_wire() {
    assert!(read("src/input_dispatch/keyboard.rs").contains("player_keys"));
    assert!(read("src/render_loop/mod.rs").contains("physics_bridge::dispatch("));
    assert!(read("src/render_loop/physics_bridge.rs").contains("fn hand_input_to_players"));
}
