//! **PULAR** (W4) — a altura é AUTORADA, e a gravidade tem fases.
//!
//! # ⚠️ Parametrizado por ALTURA, nunca por força
//!
//! É a lição que o `bevy-tnua` documenta e a razão de o campo se chamar
//! `jump_height`: uma força (ou um impulso) dá **alturas diferentes** conforme a
//! velocidade vertical no instante do toque e conforme onde na mola o
//! personagem estava. O artista não pensa em newtons — ele pensa *"este pulo
//! alcança aquela plataforma"*, e o número que ele digita tem de ser essa
//! frase. A conversão é `v₀ = √(2·|g|·h)`, feita **uma vez**, no salto.
//!
//! ⚠️ **E a altura é a de gravidade NEUTRA.** Os multiplicadores abaixo existem
//! precisamente para dobrá-la, então prometer que ela é invariante seria mentir:
//! `peak_gravity < 1` sobe MAIS (é o que alongar significa) e `cut_gravity`
//! corta. O gate da altura autorada roda com os multiplicadores em `1.0`, que é
//! o controle, e os outros medem quanto cada um a move.
//!
//! # ⚠️ O ÁPICE tem duas escolas, e elas são OPOSTAS
//!
//! O Celeste **ALONGA** o topo (gravidade reduzida — é *forgiveness*: dá ao
//! jogador a janela para mirar) e o `bevy-tnua` **ENCURTA** (`peak_prevention`,
//! é *snappiness*). **Decisão: alongar** — o pedido é um platformer PRECISO, e a
//! precisão vive na janela em que o jogador ainda pode decidir. Quem quiser o
//! oposto põe `peak_gravity > 1`: é o mesmo knob, do outro lado do 1, em vez de
//! um segundo controle dizendo o contrário do primeiro.
//!
//! # ⚠️ A mola CALA enquanto o pulo sobe
//!
//! É o defeito que este módulo existiria para ter: no instante da decolagem o
//! raio **ainda vê o chão** (o personagem não saiu do `cling_distance`), então
//! uma mola viva puxaria de volta o que o boost acabou de dar. Enquanto
//! [`JumpState::airborne`], a perna se cala; ela volta quando o personagem está
//! DESCENDO e o sensor acha chão — que é também o que faz bater a cabeça num
//! teto pousar corretamente.
//!
//! # ⚠️ A gravidade extra é `(escala − 1)·g`, e o `1.0` é BYTE-IDÊNTICO
//!
//! O solver já aplica a gravidade; esta lei só acrescenta a diferença. Com todos
//! os multiplicadores em `1.0` o termo é exatamente zero e a queda é a do mundo
//! — o que torna cada knob uma escolha, e não um imposto.

use crate::{GroundSample, Motor, Vec2};

/// Como o personagem PULA.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JumpConfig {
    /// A altura de um pulo COMPLETO, metros acima do ponto de decolagem, **com
    /// gravidade neutra** (ver o aviso do módulo).
    pub jump_height: f32,

    /// Multiplicador de gravidade na saída, enquanto a velocidade de subida
    /// passa de [`takeoff_speed`](Self::takeoff_speed).
    ///
    /// Acima de `1` o personagem sai rápido e desacelera rápido — o *snappy* que
    /// o `bevy-tnua` chama de cura do *"painfully slow"*. `1.0` é inerte.
    pub takeoff_gravity: f32,
    /// A velocidade acima da qual a gravidade de saída age.
    pub takeoff_speed: f32,

    /// Multiplicador perto do ápice — ⚠️ **abaixo de `1` ALONGA** (a decisão do
    /// módulo), acima de `1` encurta.
    pub peak_gravity: f32,
    /// A janela do ápice: `|v_subida| ≤ isto` conta como topo.
    pub peak_speed: f32,

    /// Multiplicador na QUEDA. Acima de `1` é o padrão de todo platformer —
    /// descer mais rápido do que se sobe.
    pub fall_gravity: f32,
    /// Multiplicador enquanto SOBE com o botão já SOLTO — a altura variável.
    ///
    /// É o que faz um toque curto dar um pulo curto. `1.0` desliga a altura
    /// variável (todo pulo vai à altura cheia).
    pub cut_gravity: f32,

    /// **COYOTE TIME** (W8) — por quantos segundos depois de sair do chão o pulo
    /// ainda sai.
    ///
    /// ⚠️ **É perdão para um erro de TEMPO, não uma segunda chance:** ele é
    /// CONSUMIDO pela decolagem e só volta a encher com o pé no chão, então nada
    /// aqui dá um pulo duplo. `0.0` desliga.
    pub coyote_time: f32,

    /// **JUMP BUFFER** (W8) — por quantos segundos um aperto *cedo demais*
    /// sobrevive, esperando o pé tocar o chão.
    ///
    /// O erro simétrico do coyote: um apertou tarde, o outro cedo. `0.0`
    /// desliga.
    pub jump_buffer: f32,
}

impl JumpConfig {
    /// Um perfil de partida — ⚠️ **NÃO são defaults de produto** (a mesma nota
    /// dos irmãos [`crate::RideConfig::STARTING_POINT`] e
    /// [`crate::WalkConfig::STARTING_POINT`]).
    pub const STARTING_POINT: Self = Self {
        jump_height: 2.0,
        // Inerte: a saída é a da gravidade do mundo até alguém decidir o
        // contrário. Duas rows neutras, não duas rows mortas — mexer nelas faz
        // coisa, e é essa a diferença.
        takeoff_gravity: 1.0,
        takeoff_speed: 0.0,
        // ⚠️ ABAIXO de 1: o topo ALONGA. Ver o aviso do módulo.
        peak_gravity: 0.5,
        peak_speed: 1.5,
        fall_gravity: 2.0,
        cut_gravity: 4.0,
        // ⚠️ **0,1 s é a janela do Celeste** (5-6 quadros a 60 Hz), e o número
        // que a torna julgável é a DISTÂNCIA: a 5 m/s de caminhada são **0,5 m
        // além da beirada**, e a queda dentro dela é `½·g·t² = 4,9 cm` — menos
        // que um passo, e um vigésimo da altura do personagem. É por isso que
        // ela lê como *"eu ainda estava na borda"* em vez de *"pulei do ar"*.
        coyote_time: 0.1,
        jump_buffer: 0.1,
    };
}

/// O estado VIVO de um pulo — o que o tick anterior deixou.
///
/// ⚠️ **Mora na PONTE, nunca no componente**: um campo que muda por tick dentro
/// de um componente faria o `canonicalize` do undo ver cada frame como um passo
/// (a mesma lei que manteve velocidade e sono fora do `RigidBody`). E a partir
/// da W7 ele deixa de ser guardado e passa a ser **derivado da fita de entrada**,
/// que é o que o torna reproduzível num scrub.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct JumpState {
    /// Em voo por causa de um pulo AUTORADO — não por ter andado para fora de
    /// uma borda.
    ///
    /// É ele que **cala a mola** (ver o aviso do módulo) e é ele que impede um
    /// segundo pulo no ar.
    pub airborne: bool,
    /// O botão foi solto desde a decolagem: o corte já está armado.
    ///
    /// ⚠️ Uma vez armado ele NÃO desarma segurando de novo. Soltar é uma
    /// decisão sobre ESTE pulo, e deixar o jogador desfazê-la no ar tornaria a
    /// altura função de quantas vezes ele tamborilou o dedo.
    pub cut: bool,
    /// O botão estava segurado no tick anterior — a BORDA sai daqui.
    ///
    /// Sem ela, segurar a tecla no chão re-pularia a cada tick.
    pub was_held: bool,

    /// **Segundos de coyote restantes** (W8) — enche com o pé no chão, escorre
    /// no ar, e a decolagem o ZERA.
    ///
    /// ⚠️ **É um CONTADOR, e é ele que torna o push incondicional do estado
    /// load-bearing** (ver o aviso em `bridge/player.rs`): todo campo acima é
    /// função pura da entrada deste tick, então perdê-los num tick sem motor não
    /// custava nada. Um contador congelado é uma tolerância que deixa de
    /// existir, **sem sintoma** — o pulo continua saindo.
    pub coyote: f32,

    /// **Segundos de aperto guardado** (W8) — enche na BORDA do botão, escorre
    /// sempre, e a decolagem o ZERA.
    ///
    /// ⚠️ Zerar na decolagem não é higiene: sem isso um aperto só dispararia o
    /// pulo e, no tique seguinte ainda em contato, dispararia de novo.
    pub buffer: f32,
}

/// O que a lei do pulo decidiu neste tick.
pub struct JumpStep {
    /// O termo do motor: o impulso da decolagem (boost) e a gravidade extra
    /// (accel).
    pub motor: Motor,
    /// O estado a guardar para o próximo tick.
    pub state: JumpState,
    /// **A perna pode agir?** Falso enquanto o pulo sobe — ver o aviso do
    /// módulo.
    pub spring_armed: bool,
    /// ⚠️ **Este motor é um EMPURRÃO no chão** — verdadeiro só no tick da
    /// decolagem.
    ///
    /// A reação da 3ª lei ([`crate::react`]) precisa distinguir o empurrão da
    /// decolagem (contato: pular de uma jangada a afunda) da **gravidade de
    /// FASE** do arco (ficção aplicada ao personagem no ar, que devolvida ao
    /// chão seria ele empurrar uma plataforma que não está tocando).
    ///
    /// ⚠️ Hoje os dois são distinguíveis pelo acidente de um usar `boost` e o
    /// outro `accel` — e é exatamente por ser acidente, e não contrato, que este
    /// campo existe.
    pub takeoff: bool,
}

/// **PULAR.** Ver a lei no topo do módulo.
///
/// - `footing`: o chão que a lei aceita ([`crate::footing`]), `None` no ar.
/// - `rel_up`: a velocidade de subida **relativa ao chão** — é ela que faz
///   pular de um elevador que sobe levar a velocidade dele junto.
/// - `held`: o botão de pulo está pressionado AGORA.
#[must_use]
// ⚠️ **Oito argumentos, e o 8º é o `dt`** (W8) — os dois relógios do perdão
// escorrem em segundos, nunca em contagem de tiques. Agrupar `(footing, rel_up,
// held)` num "o que o mundo diz" seria inventar um tipo com um chamador só; o
// precedente é o `body_desc` do W-LockRot, que levou o mesmo `allow`.
#[allow(clippy::too_many_arguments)]
pub fn jump_step(
    cfg: &JumpConfig,
    state: JumpState,
    footing: Option<&GroundSample>,
    rel_up: f32,
    held: bool,
    gravity: Vec2,
    up: Vec2,
    dt: f32,
) -> JumpStep {
    let pressed = held && !state.was_held;
    let mut next = JumpState {
        was_held: held,
        ..state
    };

    // ── OS DOIS RELÓGIOS DO PERDÃO (W8) ──────────────────────────────────────
    // ⚠️ Eles correm ANTES da decolagem, e a ordem é a lei: um aperto feito
    // NESTE tique tem de poder disparar NESTE tique. Enchendo depois, um aperto
    // com o pé no chão só sairia no tique seguinte — a assistência atrasaria o
    // que ela existe para adiantar.
    //
    // ⚠️ E os dois escorrem por `dt` de RELÓGIO, nunca por contagem de tiques:
    // uma tolerância medida em quadros muda de tamanho quando o `fixed_dt` muda,
    // e "0,1 s" é uma frase sobre o dedo do jogador, não sobre a taxa da sim.
    if footing.is_some() && !state.airborne {
        // No chão o coyote está CHEIO — ele é o que sobra depois de sair, não um
        // tempo acumulado por ficar parado.
        next.coyote = cfg.coyote_time.max(0.0);
    } else {
        next.coyote = (state.coyote - dt).max(0.0);
    }
    next.buffer = if pressed {
        cfg.jump_buffer.max(0.0)
    } else {
        (state.buffer - dt).max(0.0)
    };

    // ── POUSO ────────────────────────────────────────────────────────────────
    // ⚠️ **ANTES da decolagem, e a ordem é o JUMP BUFFER** (W8): um aperto
    // guardado tem de disparar NO tique em que o pé toca. Com a decolagem
    // primeiro, ela lê o `airborne` que ainda não caiu, recusa, e o pulo sai um
    // tique depois — 16 ms de atraso exatamente no gesto que o buffer existe
    // para adiantar. Ordenado assim, o mesmo tique pousa e decola.
    //
    // ⚠️ E isto **não** faz o pouso disparar no tique da decolagem: ali
    // `state.airborne` é falso na entrada, e é ele (não o `next`) que este ramo
    // pergunta.
    // O pulo acaba quando ele está DESCENDO e o sensor acha chão. As duas
    // metades importam: só "achou chão" pousaria no tick da decolagem (o raio
    // ainda vê o chão), e só "descendo" nunca pousaria.
    if state.airborne && rel_up <= 0.0 && footing.is_some() {
        next.airborne = false;
        next.cut = false;
    }

    // ── DECOLAGEM ────────────────────────────────────────────────────────────
    // ⚠️ **As duas metades são agora RELÓGIOS, e a borda vive DENTRO deles.**
    // "Estou no chão" virou *"estou no chão ou saí dele há pouco"* (coyote) e
    // "apertei agora" virou *"apertei agora ou há pouco"* (buffer) — e o `>= 0`
    // do buffer é o que mantém `jump_buffer = 0` sendo o comportamento de antes
    // desta wave AO BIT: com a janela em zero, `next.buffer > 0.0` é falso e só
    // o aperto DESTE tique passa.
    let can_reach_ground = footing.is_some() || next.coyote > 0.0;
    let wants_to_jump = pressed || next.buffer > 0.0;
    if can_reach_ground && !next.airborne && wants_to_jump {
        let g = (gravity[0] * gravity[0] + gravity[1] * gravity[1]).sqrt();
        let v0 = (2.0 * g * cfg.jump_height.max(0.0)).sqrt();
        next.airborne = true;
        next.cut = false;
        // ⚠️ **Os dois são CONSUMIDOS**, e por razões diferentes: o coyote
        // porque perdoar um erro de tempo não é dar uma segunda chance (sem
        // isto, sair da borda e pular deixaria a janela viva no ar), e o buffer
        // porque um aperto guardado que sobrevive à própria decolagem re-dispara
        // no tique seguinte, enquanto o pé ainda toca.
        next.coyote = 0.0;
        next.buffer = 0.0;
        // ⚠️ O boost leva a velocidade AO valor, não SOMA a ele: pular já
        // subindo (numa plataforma que sobe, ou logo depois de um degrau) tem de
        // dar a mesma altura que pular parado — que é a frase inteira do
        // "parametrizado por altura".
        let delta = v0 - rel_up;
        return JumpStep {
            motor: Motor {
                accel: [0.0, 0.0],
                boost: [up[0] * delta, up[1] * delta],
            },
            state: next,
            spring_armed: false,
            takeoff: true,
        };
    }

    // ── O CORTE ──────────────────────────────────────────────────────────────
    // Soltar durante a subida arma o corte, e ele não desarma (ver `JumpState`).
    if next.airborne && !held {
        next.cut = true;
    }

    let spring_armed = !next.airborne;

    // ── A GRAVIDADE POR FASE ─────────────────────────────────────────────────
    // No chão a perna manda, e uma gravidade extra ali brigaria com ela.
    if footing.is_some() && spring_armed {
        return JumpStep {
            motor: Motor::default(),
            state: next,
            spring_armed,
            takeoff: false,
        };
    }

    let scale = if rel_up > cfg.peak_speed {
        // SUBINDO.
        if next.cut {
            cfg.cut_gravity
        } else if rel_up > cfg.takeoff_speed {
            cfg.takeoff_gravity
        } else {
            1.0
        }
    } else if rel_up < -cfg.peak_speed {
        cfg.fall_gravity
    } else {
        // O ÁPICE — e também o instante em que se anda para fora de uma borda,
        // que é o mesmo `|v| ≈ 0`. Ganhar ali a mesma flutuação é o *coyote
        // feel* do Celeste, e é de propósito.
        cfg.peak_gravity
    };

    // ⚠️ `(escala − 1)·g`, então `1.0` é EXATAMENTE zero — a queda do mundo,
    // byte a byte.
    let extra = scale - 1.0;
    JumpStep {
        motor: Motor {
            accel: [gravity[0] * extra, gravity[1] * extra],
            boost: [0.0, 0.0],
        },
        state: next,
        spring_armed,
        takeoff: false,
    }
}

/// Os gates desta lei — irmão por `#[path]` (o `mod tests` continua FILHO, então
/// `use super::*` alcança os privados) pelo cap de 700 LOC.
#[cfg(test)]
#[path = "jump_tests.rs"]
mod tests;
