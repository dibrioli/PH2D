//! **A LEI do relógio** da §11 Animation — [`advance`] e o passo inteiro por trás dela —, ficheiro
//! irmão de `sprite_anim.rs` pelo tecto de 700 LOC (P14). Corte mecânico: cada item saiu inteiro,
//! verbatim, de lá, e é re-exportado no mesmo endereço (`crate::sprite_anim::advance`, …).

use super::*;

/// O que um passo de [`advance`] produziu — o que a shell publica como sinal (spec §8.10).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct AnimOutcome {
    /// O frame mudou neste passo.
    pub frame_changed: bool,
    /// Quantos ciclos fecharam neste passo (normalmente 0 ou 1).
    pub looped: u32,
    /// A animação chegou ao fim e parou.
    pub finished: bool,
}

/// **A LEI.** Avança `frame` e o estado interno em `dt_ticks` microssegundos.
///
/// `frame` é o índice **absoluto** de célula, e é o que a shell escreve em `Sprite::frame`.
/// Devolve o que aconteceu, para os sinais.
///
/// ⚠️ **Tudo inteiro.** Ver o doc do módulo: um acumulador em vírgula flutuante diverge entre
/// sistemas e leva o hash de replay com ele.
///
/// ⚠️ **O `hold_ms` e o `repeat_delay_ms` são DURAÇÃO do último frame**, e não uma fase à parte.
/// A spec §8.4 desenha um `in_hold_phase` no estado; medido, ele é um estado a mais para o mesmo
/// resultado observável — durante os dois é o último frame que está no ecrã. Dobrá-los na duração
/// dá a mesma imagem com menos uma coisa que pode ficar dessincronizada. O que os distingue
/// continua a existir: o `repeat_delay_ms` **só conta quando ainda vem outro ciclo**.
pub fn advance(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    frame: &mut u32,
    cells: u32,
    dt_ticks: u64,
) -> AnimOutcome {
    let Some((lo, hi)) = tag.resolve(cells) else {
        return AnimOutcome::default();
    };
    advance_span(animator, tag, frame, lo, hi, dt_ticks)
}

/// [`advance`] sobre um curso `lo..=hi` já resolvido. A lista de quadros
/// ([`crate::SpriteAnimFrames`]) chama-a com o PASSO (`0..n`) no lugar da célula: o `per_frame_ms`
/// indexa-se por `frame - lo`, que nos dois casos é o passo.
pub(crate) fn advance_span(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    frame: &mut u32,
    lo: u32,
    hi: u32,
    dt_ticks: u64,
) -> AnimOutcome {
    advance_span_visiting(animator, tag, frame, lo, hi, dt_ticks, &mut |_| {})
}

/// [`advance_span`] dizendo cada passo em que entra (`frame - lo`): é o que um marcador em quadro lê. Um tique
/// atrasado atravessa vários passos, e cada um é dito.
pub(crate) fn advance_span_visiting(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    frame: &mut u32,
    lo: u32,
    hi: u32,
    dt_ticks: u64,
    visit: &mut dyn FnMut(u32),
) -> AnimOutcome {
    let mut out = AnimOutcome::default();
    // Um `frame` fora do intervalo é o estado normal ao trocar de tag: entra pela ponta certa.
    if *frame < lo || *frame > hi {
        *frame = if animator.pingpong_reverse { hi } else { lo };
        out.frame_changed = true;
        visit(*frame - lo);
    }
    if !animator.playing || animator.speed_q16 == 0 || dt_ticks == 0 {
        return out;
    }
    // UMA célula sem fim marcado não tem o que fazer: ciclos sem imagem que muda só gritariam On Loop.
    // Com fim marcado (Once) ela dura o tempo dela e acaba, como no Godot: senão o On Finish nunca sai.
    if lo == hi && (animator.cycles_allowed(tag).is_none() || animator.direction(tag).bounces()) {
        return out;
    }

    // ⚠️ **A magnitude e o SENTIDO viajam separados.** Uma velocidade negativa toca ao contrário —
    // ela não faz o tempo andar para trás. Somar um delta negativo ao `elapsed` obrigaria a
    // «desavançar» frames com o resto do frame anterior, que é um segundo modelo de tempo.
    let speed = animator.speed_q16.clamp(-SPEED_MAX_Q16, SPEED_MAX_Q16);
    let magnitude = (dt_ticks as u128 * u128::from(speed.unsigned_abs()) / 65_536) as u64;
    animator.elapsed_ticks = animator.elapsed_ticks.saturating_add(magnitude);

    let mut guard = 0u32;
    loop {
        let step = step_ticks(animator, tag, *frame, lo, hi);
        if animator.elapsed_ticks < step {
            break;
        }
        animator.elapsed_ticks -= step;
        let antes = *frame;
        step_one(animator, tag, frame, lo, hi, speed < 0, &mut out);
        if *frame != antes {
            visit(*frame - lo);
        }
        if !animator.playing {
            animator.elapsed_ticks = 0;
            break;
        }
        // Rede: `frame_ticks` é ≥ 1 ms e o passo fixo tem teto de 8 sub-passos, então este laço
        // corre poucas vezes. O contador existe para que uma mudança futura no relógio não possa
        // transformá-lo num congelamento silencioso.
        guard += 1;
        if guard >= 1024 {
            animator.elapsed_ticks = 0;
            break;
        }
    }
    out
}

/// Quanto o frame ATUAL dura, com o hold e o atraso entre ciclos dobrados nele.
fn step_ticks(animator: &SpriteAnimator, tag: &AnimationTag, frame: u32, lo: u32, hi: u32) -> u64 {
    let base = tag.frame_ticks_at(frame, lo);
    if !is_cycle_end(animator, tag, frame, lo, hi) {
        return base;
    }
    let hold = u64::from(tag.hold_ms) * 1_000;
    let more_to_come = animator
        .cycles_allowed(tag)
        .is_none_or(|n| animator.repeat_count + 1 < n);
    let delay = if more_to_come {
        u64::from(tag.repeat_delay_ms) * 1_000
    } else {
        0
    };
    base + hold + delay
}

/// Este frame é o ÚLTIMO do ciclo atual?
fn is_cycle_end(
    animator: &SpriteAnimator,
    tag: &AnimationTag,
    frame: u32,
    lo: u32,
    hi: u32,
) -> bool {
    let dir = animator.direction(tag);
    if dir.bounces() {
        // Num ping-pong o ciclo fecha ao voltar à ponta de PARTIDA.
        let start_at_hi = dir.starts_reversed();
        if animator.pingpong_reverse == start_at_hi {
            // A ir na direção de partida: só o meio ciclo final fecha na ponta de chegada.
            meio_ciclo_final(animator, tag) && frame == if start_at_hi { lo } else { hi }
        } else {
            frame == if start_at_hi { hi } else { lo }
        }
    } else if dir.starts_reversed() {
        frame == lo
    } else {
        frame == hi
    }
}

/// Um passo de frame, na direção efetiva. Fecha o ciclo quando chega à ponta.
fn step_one(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    frame: &mut u32,
    lo: u32,
    hi: u32,
    play_backwards: bool,
    out: &mut AnimOutcome,
) {
    let dir = animator.direction(tag);
    // O sentido de marcha: o da tag, invertido pela volta do ping-pong e pela velocidade negativa.
    let mut going_back = dir.starts_reversed();
    if dir.bounces() && animator.pingpong_reverse != dir.starts_reversed() {
        going_back = !going_back;
    }
    if play_backwards {
        going_back = !going_back;
    }

    let at_end = if going_back {
        *frame == lo
    } else {
        *frame == hi
    };
    if !at_end {
        *frame = if going_back { *frame - 1 } else { *frame + 1 };
        out.frame_changed = true;
        return;
    }

    // Chegou à ponta.
    //
    // ⚠️ **Uma animação que NÃO repete pára no ÚLTIMO frame, e não volta ao primeiro.** É o que o
    // Godot e o Phaser fazem, e é o que um *attack* precisa: a pose final é o resultado do gesto.
    // Saltar para o início e só então parar deixaria a sprite na pose de repouso — o oposto.
    let will_continue = animator
        .cycles_allowed(tag)
        .is_none_or(|n| animator.repeat_count + 1 < n);

    if dir.bounces() {
        let start_at_hi = dir.starts_reversed();
        let closes_here = animator.pingpong_reverse != start_at_hi;
        if (closes_here || meio_ciclo_final(animator, tag)) && !will_continue {
            close_cycle(animator, tag, out);
            return;
        }
        animator.pingpong_reverse = !animator.pingpong_reverse;
        *frame = if going_back { lo + 1 } else { *frame - 1 };
        out.frame_changed = true;
        if closes_here {
            close_cycle(animator, tag, out);
        }
        return;
    }
    if will_continue {
        *frame = if going_back { hi } else { lo };
        out.frame_changed = true;
    }
    close_cycle(animator, tag, out);
}

/// O ciclo a correr é o ÚLTIMO e é meio ([`AnimationTag::acaba_na_ponta_de_chegada`]).
fn meio_ciclo_final(animator: &SpriteAnimator, tag: &AnimationTag) -> bool {
    tag.acaba_na_ponta_de_chegada
        && animator.loop_override.is_none()
        && tag.repeat.is_some_and(|n| animator.repeat_count + 1 >= n)
}

/// Fecha um ciclo: conta-o, e pára se já não há mais.
fn close_cycle(animator: &mut SpriteAnimator, tag: &AnimationTag, out: &mut AnimOutcome) {
    animator.repeat_count = animator.repeat_count.saturating_add(1);
    out.looped += 1;
    if let Some(n) = animator.cycles_allowed(tag)
        && animator.repeat_count >= n
    {
        animator.playing = false;
        out.finished = true;
    }
}
