//! **O RICOCHETE: o orçamento sobrevive, a direcção é o ESPELHO.**
//!
//! É a lei irmã do deslize da `ph2d-topdown` — o mesmo orçamento, outra re-emissão —, e ela foi
//! **medida no oráculo** (Godot 4.7.2 MIT, `move_and_collide` + `get_remainder` + `Vector2.bounce`,
//! corrido sem interface pelo `godot_projectile_probe.gd`). O corpus está em
//! `tests/fixtures/godot_bounce.txt`, com cabeçalho.
//!
//! # As três cláusulas, cada uma com o número que a deu
//!
//! 1. **O orçamento parte-se e SOMA.** Medido: `andou 3,984375 + resto 6,015625 = 10,000000` num
//!    orçamento de `10` px. Nada evapora no toque.
//! 2. **A direcção nova é o ESPELHO**, `v − 2(v·n)n`, e o oráculo acerta com ela a **`0,00000°`**
//!    nos seis ângulos do corpus — e `|saída| = |resto|`: reflectir não muda o comprimento, logo o
//!    orçamento atravessa o ricochete **inteiro**.
//! 3. **O tecto é obrigatório.** Medido: **dois** ricochetes cabem num tique quando o orçamento
//!    chega para isso (`resto_final 339,76` de `600`), e numa quina fechada um corpo rápido
//!    ping-pongaria sem fim.
//!
//! # ⚠️ A `bounciness` é NOSSA, e ela toca nas DUAS grandezas
//!
//! O `Vector2.bounce` do Godot não tem coeficiente — o corpus mede o espelho perfeito. A perda por
//! salto é desenho nosso (o *Bounciness* do Unreal), e ela multiplica **a velocidade e o resto do
//! orçamento pelo MESMO número**, porque eles são a mesma grandeza (`orçamento = |v|·dt_restante`).
//! ⛔ Escalar só um faria o resto deste tique e a rapidez do seguinte discordarem.

use crate::{ProjectileLaw, SweepStep, Vec2, dot, mirror, normalize};

/// O que um ricochete produz — **as duas metades por UMA porta**.
///
/// ⚠️⚠️ **Elas não podem sair por portas separadas.** O chamador precisa de continuar a varredura
/// (o `step`) **e** de guardar a velocidade nova (a memória do tique seguinte), e as duas são o
/// mesmo espelho com o mesmo factor. Duas chamadas seriam duas oportunidades de escrever a lei
/// pela metade — o defeito que esta linha pagou duas vezes em 2026-09-15.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounced {
    /// O passo seguinte da varredura deste tique.
    pub step: SweepStep,
    /// A velocidade depois do salto, m/s.
    pub velocity: Vec2,
}

/// **O passo seguinte**, dado o que o mundo deixou andar e em que bateu.
///
/// - `moved` — o comprimento que de facto andou neste passo;
/// - `normal` — a normal da superfície **que se opõe ao movimento**. ⚠️ A ponte normaliza o sinal
///   antes de chamar: a lei não pode adivinhar de que lado o `normal1` da `rapier` aponta, e uma
///   lei que adivinhasse mandaria o projéctil **para dentro** da parede metade das vezes.
///
/// Devolve `None` quando o orçamento acabou, quando o tecto fechou, quando o corpo não está a
/// entrar na superfície, ou quando a normal degenera.
#[must_use]
pub fn next_step(
    step: SweepStep,
    velocity: Vec2,
    moved: f32,
    normal: Vec2,
    law: &ProjectileLaw,
) -> Option<Bounced> {
    // ⭐ O RESTO e o TECTO são a metade partilhada: a folha responde-a para as duas leis.
    let resto = ph2d_sweep::remaining(step, moved)?;
    let n = normalize(normal)?;

    // ⚠️ Não está a entrar na superfície — não há o que reflectir. (A normal já vem virada contra
    // o movimento, logo `dot(dir, n)` tem de ser negativo.)
    if dot(step.dir, n) >= 0.0 {
        return None;
    }

    let dir = normalize(mirror(step.dir, n))?;
    // ⚠️ **O MESMO factor nas duas grandezas** — ver o cabeçalho.
    let k = law.bounciness.clamp(0.0, 1.0);
    let budget = resto * k;
    if budget < crate::RESTO_MINIMO {
        return None;
    }
    let v = mirror(velocity, n);
    Some(Bounced {
        step: SweepStep {
            dir,
            budget,
            steps_left: step.steps_left - 1,
        },
        velocity: [v[0] * k, v[1] * k],
    })
}

#[cfg(test)]
#[path = "bounce_tests.rs"]
mod tests;
