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
//! # ⚠️ E o CHÃO absorve a componente que aponta para ele — mas a RÉGUA importa
//!
//! Sem a absorção um personagem parado numa rampa **DESLIZA**: a gravidade entra
//! no deslocamento pedido, o controlador a projeta ao longo da superfície
//! (`slide`, que é o que faz uma parede não travar o movimento) e o resultado é
//! deriva morro abaixo. Medido, numa rampa de 30°: **−0,0279 m em 10 s**, e
//! insensível ao limite de rampa — não é o *auto-slide* do rapier, é o slide
//! genérico. É o `floor_stop_on_slope` do Godot, que shipa **ligado**.
//!
//! ⚠️ **A primeira versão desta lei absorvia com a régua ERRADA, e a medição a
//! pegou:** ela perguntava à [`crate::footing`], cujo alcance é o da PERNA
//! (`float_height + cling_distance`) — calibrado para uma cápsula que **paira**.
//! Sob Snap não há perna, e absorver a esse alcance congelava o personagem a
//! **0,4 m no ar**, exatamente onde nasceu, com todos os outros gates verdes (a
//! caminhada andava, a rampa não derivava).
//!
//! A cura não foi tirar a absorção — foi **corrigir a régua**: sob Snap o
//! `float_height` que a lei recebe é o
//! [`PhysicsWorld::body_foot_distance`](../../ph2d_physics/struct.PhysicsWorld.html#method.body_foot_distance),
//! *onde os pés deste corpo de facto ficam*. Aí *"estou no chão"* volta a
//! significar o que a palavra diz, e a [`crate::footing`] continua a porta ÚNICA
//! nos dois modos (K4).
//!
//! ⚠️ **Só a componente que aponta PARA o chão é absorvida**: zerar o eixo
//! inteiro mataria o pulo no tique da decolagem, em que o raio ainda vê o chão e
//! a subida já começou.

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
    /// **O mundo segurou-me no tique anterior?**
    ///
    /// ⚠️ **Esta NÃO é a resposta da lei sobre chão** (K4) — essa é a
    /// [`crate::footing`], e é ela que decide pulo, perdão do coyote, caminhada
    /// e agachar, nos DOIS modos. Esta é a pergunta do INTEGRADOR, que é outra:
    /// *"há alguma coisa a segurar-me AGORA?"*.
    ///
    /// As duas medem coisas diferentes de propósito, e colapsá-las foi medido:
    /// a `footing` tem uma faixa de tolerância (`cling_distance`) para o gesto
    /// não morrer num degrau, e absorver a gravidade dentro dessa faixa deixa o
    /// personagem **pendurado no ar** na borda dela — 1,237 m onde o chão está a
    /// 1,000, para sempre, com todos os outros gates verdes.
    ///
    /// Vem do controlador, que é quem tocou no mundo; e mora aqui — e não num
    /// mapa da ponte — pelo mesmo motivo que a velocidade: é o ring de tiques
    /// âncora que guarda este tipo.
    pub grounded: bool,
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
    ground_velocity: Vec2,
    gravity: Vec2,
    up: Vec2,
    dt: f32,
) -> (KinematicState, Vec2) {
    let mut v = [
        state.velocity[0] + (gravity[0] + motor.accel[0]) * dt + motor.boost[0],
        state.velocity[1] + (gravity[1] + motor.accel[1]) * dt + motor.boost[1],
    ];

    // Ver [`KinematicState::grounded`]: a pergunta do INTEGRADOR, não a da lei.
    if state.grounded {
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
    (
        KinematicState {
            velocity: v,
            ..state
        },
        wanted,
    )
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
    grounded: bool,
    dt: f32,
) -> KinematicState {
    if dt <= 0.0 {
        return KinematicState { grounded, ..state };
    }
    let mut v = state.velocity;
    for i in 0..2 {
        let lost = (wanted[i] - effective[i]) / dt;
        // `clamp` entre 0 e a própria velocidade cobre as duas metades de uma
        // vez: fora do sentido de `v` o intervalo colapsa em zero, e dentro dele
        // o teto é `v` — o resultado nunca troca de sinal.
        v[i] -= lost.clamp(v[i].min(0.0), v[i].max(0.0));
    }
    KinematicState {
        velocity: v,
        grounded,
    }
}

#[cfg(test)]
#[path = "kinematic_tests.rs"]
mod tests;
