//! **A INTEGRAÇÃO CINEMÁTICA** — o motor de intenção vira um deslocamento
//! (W-KinMove, K5/K7 do plano 07).
//!
//! # ⚠️ Por que isto é LEI e não ponte
//!
//! Um corpo cinemático não tem velocidade que o solver possua, então alguém tem
//! de a possuir — e o aviso já estava escrito no próprio [`crate::PlayerState`]:
//! *"um estado de player que vivesse num segundo mapa da ponte teria de ser
//! acrescentado àquele ring à mão, e esquecê-lo é um scrub que devolve o mundo
//! de um tique e a memória de outro"*. Estando aqui, ela entra no ring de
//! checkpoints e sobrevive a um scrub de graça.
//!
//! E o que esta função faz é **aritmética sobre a intenção**, que é exactamente
//! o que esta crate é. O que fica na ponte é a única coisa que precisa do mundo:
//! perguntar ao `move_shape` **quanto do deslocamento coube**.
//!
//! # ⚠️ A gravidade é aplicada AQUI, e é a assimetria central dos dois modos
//!
//! No modo dinâmico o solver integra a gravidade e a mola a cancela; num corpo
//! cinemático **ninguém a aplica**, então esta lei a soma ao motor. É por isso
//! que a [`crate::PlayerStep::gravity_hold`] não é consultada neste caminho: ela
//! existe para a ponte dinâmica dividir o impulso entre sub-passos, e aqui não
//! há sub-passo nenhum a dividir.
//!
//! # ⚠️ E é por isso que o CHÃO tem de absorver a componente que aponta para ele
//!
//! Sem a absorção o personagem parado acumula velocidade para baixo para sempre
//! (o snap corrige a POSIÇÃO todo tique e a velocidade continua a crescer), e o
//! primeiro degrau que ele descer vira uma queda de mil metros por segundo. É o
//! mesmo passo que o `CharacterController` da Unity e o `move_and_slide` do
//! Godot dão, e a pergunta *"estou no chão?"* vem da [`crate::footing`] — **não**
//! do `grounded` do `move_shape` (K4).

use crate::{Motor, Vec2};

/// **A velocidade que o modo cinemático possui** — o que o solver possuiria se
/// o corpo fosse dinâmico.
///
/// ⚠️ Um tipo e não um `Vec2` solto: ele mora no [`crate::PlayerState`], que é
/// o valor que a fita guarda, e um par de `f32` anônimo ali seria indistinguível
/// da memória do chão que já vive ao lado.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct KinematicState {
    /// Metros por segundo, em MUNDO.
    pub velocity: Vec2,
}

/// **O deslocamento que este tique PEDE** — e o estado que ele deixa.
///
/// `ground_velocity` entra aqui e não na ponte (K7): ela já viaja no
/// [`crate::GroundSample`] desde a W3, e somá-la fora seria uma segunda resposta
/// para *"quanto o chão me leva?"*.
///
/// ⚠️ **A velocidade do chão NÃO entra na velocidade guardada**, e a distinção é
/// o que faz um personagem sair de uma plataforma móvel com o impulso dela em
/// vez de colado a ela: o que ele *possui* é a velocidade dele; o que a
/// plataforma acrescenta é deslocamento **deste** tique. Somá-la ao estado a
/// tornaria permanente, e ele continuaria a voar para o lado depois de saltar
/// para o chão firme.
#[must_use]
pub fn kinematic_advance(
    state: KinematicState,
    motor: Motor,
    grounded: bool,
    ground_velocity: Vec2,
    gravity: Vec2,
    up: Vec2,
    dt: f32,
) -> (KinematicState, Vec2) {
    let mut v = [
        state.velocity[0] + (gravity[0] + motor.accel[0]) * dt + motor.boost[0],
        state.velocity[1] + (gravity[1] + motor.accel[1]) * dt + motor.boost[1],
    ];

    // ⚠️ **Só a componente que aponta PARA o chão é absorvida.** Zerar o eixo
    // inteiro mataria o pulo no tique da decolagem — o raio ainda vê o chão ali
    // (o personagem não saiu do `cling_distance`), que é a mesma armadilha que a
    // `JumpState::airborne` existe para cobrir do outro lado.
    if grounded {
        let into = v[0] * up[0] + v[1] * up[1];
        if into < 0.0 {
            v[0] -= up[0] * into;
            v[1] -= up[1] * into;
        }
    }

    let wanted = [
        (v[0] + ground_velocity[0]) * dt,
        (v[1] + ground_velocity[1]) * dt,
    ];
    (KinematicState { velocity: v }, wanted)
}

/// **O que o mundo NÃO deixou acontecer** — a diferença entre o pedido e o
/// efetivo, devolvida como velocidade.
///
/// # ⚠️ Sem isto o personagem acelera contra uma parede para sempre
///
/// A velocidade que esta lei possui é uma ficção: nada no mundo a corrige. Um
/// personagem encostado num teto continuaria a somar `+v` para cima tique após
/// tique, e no instante em que o teto acabasse ele sairia disparado — o defeito
/// clássico de todo controlador cinemático escrito sem este passo.
///
/// # ⚠️ Só o que FREIA a velocidade PRÓPRIA é absorvido
///
/// A regra ingênua (`v −= (pedido − efetivo)/dt`) tem um artefato que o caso
/// mais comum de plataforma expõe: um personagem **parado** sobre um vagão,
/// prensado contra uma parede, tem o deslocamento do VAGÃO bloqueado — e a
/// subtração cega deixa-o com `v = −velocidade_do_vagão`. Ele fica quieto
/// enquanto o vagão o empurra e **dispara para trás** no instante em que sai
/// dele.
///
/// A lei é por componente e tem duas metades, ambas load-bearing:
///
/// 1. **o sinal** — só se absorve o que aponta no mesmo sentido da velocidade
///    própria (um bloqueio que impede a plataforma de me levar não é uma
///    velocidade minha a corrigir);
/// 2. **o teto** — nunca se remove mais do que existe, senão parar contra uma
///    parede vira um empurrão para o outro lado.
///
/// ⚠️ **E o que sobra é deslizar:** numa rampa, parte do pedido vira movimento
/// noutra direção sem nada ter sido *bloqueado* ali — a componente onde a
/// velocidade própria é zero passa intacta, que é o que mantém o deslize vivo.
#[must_use]
pub fn kinematic_settle(
    state: KinematicState,
    wanted: Vec2,
    effective: Vec2,
    dt: f32,
) -> KinematicState {
    if dt <= 0.0 {
        return state;
    }
    let mut v = state.velocity;
    for i in 0..2 {
        let lost = (wanted[i] - effective[i]) / dt;
        // `clamp` entre 0 e a própria velocidade cobre as duas metades de uma
        // vez: fora do sentido de `v` o intervalo colapsa em zero, e dentro dele
        // o teto é `v` — o resultado nunca troca de sinal.
        v[i] -= lost.clamp(v[i].min(0.0), v[i].max(0.0));
    }
    KinematicState { velocity: v }
}

#[cfg(test)]
#[path = "kinematic_tests.rs"]
mod tests;
