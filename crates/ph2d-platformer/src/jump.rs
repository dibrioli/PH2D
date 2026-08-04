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
}

/// **PULAR.** Ver a lei no topo do módulo.
///
/// - `footing`: o chão que a lei aceita ([`crate::footing`]), `None` no ar.
/// - `rel_up`: a velocidade de subida **relativa ao chão** — é ela que faz
///   pular de um elevador que sobe levar a velocidade dele junto.
/// - `held`: o botão de pulo está pressionado AGORA.
#[must_use]
pub fn jump_step(
    cfg: &JumpConfig,
    state: JumpState,
    footing: Option<&GroundSample>,
    rel_up: f32,
    held: bool,
    gravity: Vec2,
    up: Vec2,
) -> JumpStep {
    let pressed = held && !state.was_held;
    let mut next = JumpState {
        was_held: held,
        ..state
    };

    // ── DECOLAGEM ────────────────────────────────────────────────────────────
    // Só do chão, e só na BORDA: segurar a tecla não re-pula.
    if footing.is_some() && !state.airborne && pressed {
        let g = (gravity[0] * gravity[0] + gravity[1] * gravity[1]).sqrt();
        let v0 = (2.0 * g * cfg.jump_height.max(0.0)).sqrt();
        next.airborne = true;
        next.cut = false;
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
        };
    }

    // ── POUSO ────────────────────────────────────────────────────────────────
    // O pulo acaba quando ele está DESCENDO e o sensor acha chão. As duas
    // metades importam: só "achou chão" pousaria no tick da decolagem (o raio
    // ainda vê o chão), e só "descendo" nunca pousaria.
    if state.airborne && rel_up <= 0.0 && footing.is_some() {
        next.airborne = false;
        next.cut = false;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP: Vec2 = [0.0, 1.0];
    const G: Vec2 = [0.0, -9.81];

    fn ground() -> GroundSample {
        GroundSample {
            distance: 0.9,
            normal: [0.0, 1.0],
            ground_velocity: [0.0, 0.0],
        }
    }

    /// **A altura vira velocidade UMA vez, pela fórmula.**
    ///
    /// `v₀ = √(2·g·h)`: com `h = 2` e `g = 9,81` são 6,264 m/s.
    #[test]
    fn the_takeoff_speed_comes_from_the_authored_height() {
        let cfg = JumpConfig::STARTING_POINT;
        let s = jump_step(
            &cfg,
            JumpState::default(),
            Some(&ground()),
            0.0,
            true,
            G,
            UP,
        );
        let expected = (2.0 * 9.81 * 2.0_f32).sqrt();
        assert!(
            (s.motor.boost[1] - expected).abs() < 1.0e-3,
            "o boost tem de ser v0 = sqrt(2gh) = {expected:.4}: {:?}",
            s.motor.boost
        );
        assert!(s.state.airborne, "decolou");
        assert!(!s.spring_armed, "e a perna CALA");
    }

    /// ⚠️ **Pular já subindo dá a MESMA altura** — o boost leva AO valor, não
    /// soma a ele.
    ///
    /// É a frase inteira do *"parametrizado por altura"*: sem isto, pular de uma
    /// plataforma que sobe daria um pulo mais alto do que o autorado, e o número
    /// na row deixaria de descrever o que acontece.
    #[test]
    fn jumping_while_already_rising_reaches_the_same_height() {
        let cfg = JumpConfig::STARTING_POINT;
        let v0 = (2.0 * 9.81 * 2.0_f32).sqrt();
        for start in [0.0_f32, 2.0, -1.0] {
            let s = jump_step(
                &cfg,
                JumpState::default(),
                Some(&ground()),
                start,
                true,
                G,
                UP,
            );
            let after = start + s.motor.boost[1];
            assert!(
                (after - v0).abs() < 1.0e-3,
                "partindo de {start}: a velocidade final tem de ser {v0:.4}, deu {after:.4}"
            );
        }
    }

    /// ⚠️ **Segurar a tecla NÃO re-pula** — a decolagem é na BORDA.
    ///
    /// Sem isto, um dedo apoiado no botão daria um impulso por tick e o
    /// personagem subiria para sempre.
    #[test]
    fn holding_the_button_does_not_re_jump() {
        let cfg = JumpConfig::STARTING_POINT;
        let first = jump_step(
            &cfg,
            JumpState::default(),
            Some(&ground()),
            0.0,
            true,
            G,
            UP,
        );
        assert!(first.motor.boost[1] > 0.0);
        // O tick seguinte, ainda com a tecla presa e ainda vendo o chão.
        let second = jump_step(&cfg, first.state, Some(&ground()), 3.0, true, G, UP);
        assert_eq!(second.motor.boost, [0.0, 0.0], "nada de segundo impulso");
    }

    /// **Nem pular no AR** — o pulo duplo é uma feature, não um acidente.
    #[test]
    fn a_second_press_in_mid_air_does_nothing() {
        let cfg = JumpConfig::STARTING_POINT;
        let flying = JumpState {
            airborne: true,
            cut: false,
            was_held: false,
        };
        let s = jump_step(&cfg, flying, None, 3.0, true, G, UP);
        assert_eq!(s.motor.boost, [0.0, 0.0]);
    }

    /// **Soltar durante a subida ARMA o corte, e ele não desarma.**
    ///
    /// Segurar de novo no ar não devolve a altura — senão ela seria função de
    /// quantas vezes o jogador tamborilou o dedo.
    #[test]
    fn releasing_arms_the_cut_and_re_holding_does_not_undo_it() {
        let cfg = JumpConfig::STARTING_POINT;
        let flying = JumpState {
            airborne: true,
            cut: false,
            was_held: true,
        };
        let released = jump_step(&cfg, flying, None, 4.0, false, G, UP);
        assert!(released.state.cut, "soltar arma o corte");
        assert!(
            (released.motor.accel[1] - G[1] * (cfg.cut_gravity - 1.0)).abs() < 1.0e-4,
            "e o corte pesa: {:?}",
            released.motor.accel
        );

        let re_held = jump_step(&cfg, released.state, None, 3.0, true, G, UP);
        assert!(re_held.state.cut, "segurar de novo NAO desfaz o corte");
    }

    /// **As quatro fases dão quatro gravidades**, e a do ápice é a única ABAIXO
    /// de 1 — a decisão do módulo.
    #[test]
    fn each_phase_gets_its_own_gravity() {
        let cfg = JumpConfig::STARTING_POINT;
        let flying = JumpState {
            airborne: true,
            cut: false,
            was_held: true,
        };
        let scale = |rel: f32, st: JumpState, held: bool| {
            let s = jump_step(&cfg, st, None, rel, held, G, UP);
            s.motor.accel[1] / G[1] + 1.0
        };
        let rising = scale(5.0, flying, true);
        let peak = scale(0.0, flying, true);
        let falling = scale(-5.0, flying, true);
        let cut = scale(
            5.0,
            JumpState {
                cut: true,
                ..flying
            },
            true,
        );
        assert!(
            (rising - 1.0).abs() < 1.0e-4,
            "subindo (takeoff inerte): {rising}"
        );
        assert!((peak - cfg.peak_gravity).abs() < 1.0e-4, "apice: {peak}");
        assert!(
            (falling - cfg.fall_gravity).abs() < 1.0e-4,
            "queda: {falling}"
        );
        assert!((cut - cfg.cut_gravity).abs() < 1.0e-4, "corte: {cut}");
        assert!(
            peak < 1.0,
            "⚠️ o apice ALONGA (Celeste), nao encurta (tnua): {peak}"
        );
    }

    /// ⚠️ **Multiplicadores em `1.0` são gravidade extra ZERO** — o mundo sem
    /// pulo, byte a byte. É o que torna cada knob uma escolha e não um imposto.
    #[test]
    fn neutral_multipliers_add_exactly_nothing() {
        let cfg = JumpConfig {
            takeoff_gravity: 1.0,
            peak_gravity: 1.0,
            fall_gravity: 1.0,
            cut_gravity: 1.0,
            ..JumpConfig::STARTING_POINT
        };
        let flying = JumpState {
            airborne: true,
            cut: true,
            was_held: false,
        };
        for rel in [-9.0_f32, -1.0, 0.0, 1.0, 9.0] {
            let s = jump_step(&cfg, flying, None, rel, false, G, UP);
            assert_eq!(s.motor.accel, [0.0, 0.0], "a {rel} m/s");
        }
    }

    /// **O pouso pede as DUAS metades** — descendo E com chão ao alcance.
    ///
    /// Só "achou chão" pousaria no tick da decolagem (o raio ainda vê o chão) e
    /// a mola puxaria de volta o pulo inteiro; só "descendo" nunca pousaria.
    #[test]
    fn landing_needs_both_halves() {
        let cfg = JumpConfig::STARTING_POINT;
        let flying = JumpState {
            airborne: true,
            cut: false,
            was_held: false,
        };
        // Subindo COM chão ao alcance (o tick logo após a decolagem): não pousa.
        let rising = jump_step(&cfg, flying, Some(&ground()), 5.0, false, G, UP);
        assert!(rising.state.airborne, "ainda subindo, nao pousou");
        assert!(!rising.spring_armed, "e a perna segue CALADA");

        // Descendo SEM chão: também não.
        let falling = jump_step(&cfg, flying, None, -5.0, false, G, UP);
        assert!(falling.state.airborne);

        // Descendo COM chão: pousou, e a perna volta.
        let landed = jump_step(&cfg, flying, Some(&ground()), -5.0, false, G, UP);
        assert!(!landed.state.airborne, "pousou");
        assert!(landed.spring_armed, "e a perna volta a agir");
    }

    /// ⚠️ **O pouso limpa o CORTE também, e ele é alcançável.**
    ///
    /// O corte é lido num sítio só — subindo acima do `peak_speed` — e no chão a
    /// perna arma primeiro, então um corte esquecido parece inofensivo. Não é: se
    /// o personagem **sai do chão SUBINDO sem pular** (andar para fora da borda de
    /// uma plataforma kinematic que ASCENDE, que este produto já tem desde o W4),
    /// não há decolagem para zerá-lo — e a subida ganha `cut_gravity` (4×) onde
    /// devia ganhar `takeoff_gravity` (1×), puxando o personagem para baixo por um
    /// corte de um pulo que acabou.
    ///
    /// ⚠️ Este gate nasceu de uma mutação que **sobreviveu à suíte inteira** (a de
    /// comportamento inclusive): tirar o `next.cut = false` do pouso não move um
    /// milímetro em nenhuma cena que só pula e cai.
    #[test]
    fn landing_clears_the_cut_so_the_next_rise_is_not_punished() {
        let cfg = JumpConfig::STARTING_POINT;
        let cut_in_flight = JumpState {
            airborne: true,
            cut: true,
            was_held: false,
        };

        // Pousa com o corte armado.
        let landed = jump_step(&cfg, cut_in_flight, Some(&ground()), -5.0, false, G, UP);
        assert!(!landed.state.airborne, "pousou");
        assert!(!landed.state.cut, "e o corte MORREU com o pouso");

        // Agora sai do chão SUBINDO sem pular (a plataforma o levou).
        let rising = jump_step(&cfg, landed.state, None, 5.0, false, G, UP);
        let scale = rising.motor.accel[1] / G[1] + 1.0;
        assert!(
            (scale - cfg.takeoff_gravity).abs() < 1.0e-4,
            "subir sem pular usa a gravidade de SAIDA, nao a do corte: {scale}"
        );
    }

    /// Sem pulo nenhum, andar para fora de uma borda ainda ganha a gravidade de
    /// QUEDA — ela é sobre cair, não sobre pular.
    #[test]
    fn walking_off_a_ledge_still_falls_faster() {
        let cfg = JumpConfig::STARTING_POINT;
        let s = jump_step(&cfg, JumpState::default(), None, -5.0, false, G, UP);
        assert!(!s.state.airborne, "ele nao pulou");
        assert!(
            (s.motor.accel[1] - G[1] * (cfg.fall_gravity - 1.0)).abs() < 1.0e-4,
            "mas cai com a gravidade de queda: {:?}",
            s.motor.accel
        );
    }
}
