//! **A §11 Animation a correr** — a costura entre a lei pura ([`ph2d_ecs::sprite_anim`]) e o
//! relógio do app.
//!
//! # ⚠️ Corre no PASSO FIXO, e não no relógio de parede
//!
//! A spec §8.4 exige que o `SpriteAnimator` seja `SimComponent` reproduzível pelo replay, e um
//! replay só reproduz o que anda em passos **iguais e contados**. O `wall_dt` mede quanto o último
//! quadro demorou — ele muda com a carga da máquina, e uma animação presa a ele avançaria
//! diferente em duas corridas do mesmo ficheiro.
//!
//! ⚠️ **A conversão para inteiro acontece AQUI, uma vez.** A lei pura nunca vê um float
//! (`vfmadd` em x86_64 e não em aarch64 ⇒ divergência de um ULP ⇒ o frame avança noutro tique ⇒
//! o UV do atlas diverge ⇒ o hash de replay quebra). O `fixed_dt` é uma **constante**, por isso
//! `dt_ticks` é a mesma em toda máquina.
//!
//! # ⚠️ UM sink: `Sprite::frame`
//!
//! O índice que a lei produz é escrito no `Sprite::frame` e em mais nada. A folha autorada
//! (`SpriteSheetRef`) é *proveniência de autoria* e não um índice vivo — a razão está no doc de
//! [`ph2d_ecs::sprite_anim`].

use ph2d_ecs::{SimWorld, SpriteAnimations, SpriteAnimator};
use ph2d_render::Sprite;

/// Quantos microssegundos vale um passo fixo. Convertido **uma vez**, a partir de uma constante.
///
/// ⚠️ `round`, e não `as u64`: a 60 Hz o passo é `16666,66…` µs, e truncar perderia 0,66 µs por
/// tique — 40 µs por segundo, 2,4 ms por minuto. Invisível num quadro, e uma animação longa
/// dessincronizaria de um relógio que não trunca.
#[must_use]
pub(crate) fn step_ticks(fixed_dt: f64) -> u64 {
    (fixed_dt * 1_000_000.0).round().max(0.0) as u64
}

/// Avança toda sprite animada em `ticks` passos fixos.
///
/// ⚠️ **`ticks × dt` numa chamada só, e a razão é uma MEDIÇÃO.** A primeira versão disto corria um
/// laço de `ticks` chamadas, com a justificação de que «um passo grande atravessaria o fim de um
/// ciclo sem o fechar». **Era falso**, e foi uma mutação que o disse: a própria
/// [`ph2d_ecs::advance`] tem um laço de recuperação que avança **um frame de cada vez** até gastar
/// o acumulado, por isso os dois caminhos fecham exatamente os mesmos ciclos. O gate
/// `catching_up_in_one_call_is_the_same_as_catching_up_step_by_step` prende a equivalência.
///
/// ⇒ Fica a forma simples. *Um laço a mais que a mutação não distingue é uma justificação que
/// ninguém voltou a verificar.*
pub(crate) fn tick_sprite_animations(sim: &mut SimWorld, ticks: u32, fixed_dt: f64) {
    if ticks == 0 {
        return;
    }
    let dt = step_ticks(fixed_dt);
    if dt == 0 {
        return;
    }
    let world = sim.world_mut();
    let mut q = world.query::<(&mut SpriteAnimator, &SpriteAnimations, &mut Sprite)>();
    for (mut animator, tags, mut sprite) in q.iter_mut(world) {
        if !animator.playing {
            continue;
        }
        let Some(tag) = tags.get(&animator.current) else {
            continue;
        };
        // O pool é a grelha desta sprite, lida **agora** — mexer em `hframes` a meio de uma
        // animação encolhe o intervalo no mesmo quadro, sem estado intermédio a envelhecer.
        let cells = sprite.hframes.saturating_mul(sprite.vframes).max(1);
        let mut frame = sprite.frame;
        ph2d_ecs::advance(&mut animator, tag, &mut frame, cells, dt * u64::from(ticks));
        // ⚠️ **Escreve só quando MUDA.** O `Sprite` é `SimComponent` e o undo regista por DIFF:
        // tocar-lhe todo o quadro faria uma sprite pausada num frame só produzir um passo de
        // undo por quadro. `bevy` marca a mudança no `deref_mut`, não na atribuição.
        if sprite.frame != frame {
            sprite.frame = frame;
        }
    }
}

/// **O `autoplay` a fazer o que promete**, no único momento em que ele tem significado no editor:
/// o projeto acabou de abrir.
///
/// ⚠️ Ele **não** pode viver no tique: «começar a tocar» é uma aresta, e o tique só vê estados —
/// para a detetar ali seria preciso um bit a mais que diz «já comecei», e um bit desses fica
/// dessincronizado no primeiro `Ctrl+Z`.
pub(crate) fn start_autoplay_animations(sim: &mut SimWorld) {
    let world = sim.world_mut();
    let mut q = world.query::<(&mut SpriteAnimator, &SpriteAnimations)>();
    for (mut animator, tags) in q.iter_mut(world) {
        if !animator.autoplay || animator.playing {
            continue;
        }
        let tag = tags.get(&animator.current).cloned();
        animator.playing = true;
        animator.rewind(tag.as_ref());
    }
}
