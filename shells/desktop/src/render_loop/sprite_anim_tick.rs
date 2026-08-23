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

/// **Um sinal que uma animação produziu neste quadro** — o que a shell publica no outbox.
///
/// ⚠️ Um tipo do SHELL e não do `ph2d-ecs`: a lei pura não conhece o `ph2d-runtime`, e não pode —
/// ela é o que o replay reproduz, e o outbox é o que o app faz com isso. O tique devolve fatos; o
/// chamador é que os transforma em eventos.
pub(crate) struct AnimSignal {
    pub(crate) entity: ph2d_ecs::Entity,
    /// O nome AUTORADO na tag. Vazio nunca chega aqui — uma animação calada não produz sinal.
    pub(crate) name: String,
    /// Quantos ciclos fecharam neste tique (≥ 1).
    pub(crate) cycles: u32,
}
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
/// ⚠️ **O `drive` é a metade que o undo lê.** Tudo o que este tique escreve — os três campos do
/// relógio e o `Sprite::frame` — é **pré-visualização**, nunca autoria: declará-lo aqui é o que
/// impede que cada clique dado enquanto a animação toca empilhe um passo de Ctrl+Z vazio
/// ([`crate::preview_drive`], e a auditoria 21 §4).
pub(crate) fn tick_sprite_animations(
    sim: &mut SimWorld,
    ticks: u32,
    fixed_dt: f64,
    tool_preview_bits: &[Option<u64>],
    drive: &mut crate::preview_drive::PreviewDrive,
) -> Vec<AnimSignal> {
    let mut out = Vec::new();
    if ticks == 0 {
        return out;
    }
    let dt = step_ticks(fixed_dt);
    if dt == 0 {
        return out;
    }
    let world = sim.world_mut();
    let mut q = world.query::<(
        ph2d_ecs::Entity,
        &mut SpriteAnimator,
        &SpriteAnimations,
        &mut Sprite,
    )>();
    for (entity, mut animator, tags, mut sprite) in q.iter_mut(world) {
        // ⚠️ **UMA FOLHA EM PINTURA TOCA, mesmo com o transporte parado** (Enio, 2026-08-23:
        // *«o preview não está animado»*).
        //
        // Enquanto uma ferramenta pré-visualiza um sprite com grelha, o quad dele **desdobra-se** e
        // passa a mostrar a imagem inteira — o `Sprite::frame` deixa de ter efeito no que se pinta,
        // e o único sítio onde ele ainda se vê é a célula de pré-visualização ao lado. Se ela
        // dependesse do `playing`, bastaria o artista ter pausado uma vez (arrastar a barra de
        // frames pausa, por desenho) para a pré-visualização nascer **parada** — e ela existe
        // exactamente para mostrar o movimento.
        //
        // ⚠️ Isto **não** liga o transporte: o `playing` fica como estava, e ao sair da ferramenta
        // a sprite volta a obedecer-lhe. O que corre aqui é a pré-visualização.
        let painted =
            crate::render_loop::sim_extract_sheet::is_tool_previewed(tool_preview_bits, entity);
        if !animator.playing && !painted {
            continue;
        }
        let Some(tag) = tags.get(&animator.current) else {
            continue;
        };
        // ⚠️ **A pré-visualização corre sobre uma CÓPIA, e a lei pura é a razão:** a
        // [`ph2d_ecs::advance`] também desiste quando `playing` é falso (é ela que define o que
        // «tocar» quer dizer), então bastava-me abrir o guarda acima e o frame continuava parado.
        //
        // ⚠️ **A cópia leva `loop_override = Some(true)`**: uma animação de uma volta já esgotada
        // congelaria a pré-visualização na última célula — e uma pré-visualização que existe para
        // mostrar o movimento não pode acabar. *Ela repete porque é uma pré-visualização, não
        // porque a cena o pede.*
        //
        // ⇒ Do que a cópia produz volta o **relógio** (é ele que faz o tempo passar entre tiques)
        // e o frame; o `playing` e os overrides do documento ficam intactos, e sair da ferramenta
        // devolve a cena exactamente como ela estava.
        let preview_only = painted && !animator.playing;
        let mut run = animator.clone();
        if preview_only {
            run.playing = true;
            run.loop_override = Some(true);
        }
        // O pool é a grelha desta sprite, lida **agora** — mexer em `hframes` a meio de uma
        // animação encolhe o intervalo no mesmo quadro, sem estado intermédio a envelhecer.
        let cells = sprite.hframes.saturating_mul(sprite.vframes).max(1);
        let mut frame = sprite.frame;
        // O que o artista tem no documento, ANTES de este tique lhe tocar (`crate::preview_drive`).
        let before = crate::preview_drive::Driven::SpriteAnim {
            elapsed_ticks: animator.elapsed_ticks,
            pingpong_reverse: animator.pingpong_reverse,
            repeat_count: animator.repeat_count,
            frame,
        };
        let outcome = ph2d_ecs::advance(&mut run, tag, &mut frame, cells, dt * u64::from(ticks));
        if preview_only {
            animator.elapsed_ticks = run.elapsed_ticks;
            animator.pingpong_reverse = run.pingpong_reverse;
            animator.repeat_count = run.repeat_count;
        } else {
            *animator = run;
        }
        // ⚠️ **Escreve só quando MUDA.** O `Sprite` é `SimComponent` e o undo regista por DIFF:
        // tocar-lhe todo o quadro faria uma sprite pausada num frame só produzir um passo de
        // undo por quadro. `bevy` marca a mudança no `deref_mut`, não na atribuição.
        if sprite.frame != frame {
            sprite.frame = frame;
        }
        // **A DECLARAÇÃO**, com o depois lido do que ficou de facto escrito (o `preview_only`
        // devolve só o relógio, então `run` sozinho mentiria sobre o animador).
        let after = crate::preview_drive::Driven::SpriteAnim {
            elapsed_ticks: animator.elapsed_ticks,
            pingpong_reverse: animator.pingpong_reverse,
            repeat_count: animator.repeat_count,
            frame,
        };
        // ⚠️ **Só quando este tique de facto escreveu.** Um animador que está a tocar e não andou
        // nada (`dt·ticks` engolido por uma velocidade zero) não está a conduzir ninguém, e
        // declará-lo manteria viva uma condução que a `settle` precisa de ver morrer.
        if before != after {
            drive.driven(entity, before, after);
        }
        // **OS SINAIS** (spec §8.10). ⚠️ **A pré-visualização é MUDA**: uma folha em pintura corre
        // sobre uma cópia do animador, e ela existe para mostrar o movimento — não para fazer
        // acontecer coisas na cena. Publicar dali faria um som tocar porque alguém pegou no
        // pincel.
        if !preview_only {
            if outcome.looped > 0 && !tag.signal_on_loop.is_empty() {
                out.push(AnimSignal {
                    entity,
                    name: tag.signal_on_loop.clone(),
                    cycles: outcome.looped,
                });
            }
            if outcome.finished && !tag.signal_on_finish.is_empty() {
                out.push(AnimSignal {
                    entity,
                    name: tag.signal_on_finish.clone(),
                    cycles: outcome.looped.max(1),
                });
            }
        }
    }
    out
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

#[cfg(test)]
#[path = "sprite_anim_signal_tests.rs"]
mod signal_tests;
