//! **A §11 Animation a correr** — a costura entre a lei pura ([`ph2d_ecs::sprite_anim`]) e o
//! relógio do app. ⚠️ Corre no PASSO FIXO (o replay só reproduz passos iguais e contados, spec §8.4),
//! e a conversão para inteiro acontece AQUI, uma vez: a lei pura nunca vê um float. ⚠️ Um sink só: o
//! índice vai para a grelha (`SpriteGrid::frame`); a razão está em [`ph2d_ecs::sprite_anim`].

use ph2d_ecs::{SimWorld, SpriteAnimations, SpriteAnimator};

/// **Um sinal que uma animação produziu neste quadro** — o que a shell publica no outbox. Não é do
/// `ph2d-ecs`: a lei pura não conhece o `ph2d-runtime`; o tique devolve fatos, o chamador faz eventos.
pub struct AnimSignal {
    pub entity: ph2d_ecs::Entity,
    /// O nome AUTORADO na tag. Vazio nunca chega aqui — uma animação calada não produz sinal.
    pub name: String,
    /// Quantos ciclos fecharam neste tique (≥ 1).
    pub cycles: u32,
}

/// Avança toda sprite animada em `ticks` passos fixos.
///
/// `ticks × dt` numa chamada só: a [`ph2d_ecs::advance`] já recupera frame a frame (gate
/// `catching_up_in_one_call_is_the_same_as_catching_up_step_by_step`).
/// O `drive` é a metade que o undo lê: o que este tique escreve é pré-visualização, nunca autoria.
pub fn tick_sprite_animations(
    sim: &mut SimWorld,
    ticks: u32,
    fixed_dt: f64,
    tool_preview_bits: &[Option<u64>],
    drive: &mut ph2d_preview_drive::PreviewDrive,
) -> Vec<AnimSignal> {
    let mut out = Vec::new();
    if ticks == 0 {
        return out;
    }
    let dt = ph2d_ecs::timer::step_ticks(fixed_dt);
    if dt == 0 {
        return out;
    }
    // O `OnScreenEnabler` pausa («só corre no ecrã»; o `HideVisible` mora no extract). Colhido ANTES do
    // laço mutável (lê a hierarquia inteira); sem nenhum enabler na cena a lista sai vazia.
    let paused: Vec<ph2d_ecs::Entity> = {
        let w = sim.world_mut();
        let mut carriers = w.query::<&ph2d_ecs::OnScreenEnabler>();
        if carriers.iter(w).next().is_none() {
            Vec::new()
        } else {
            let mut animated = w.query::<(ph2d_ecs::Entity, &SpriteAnimator)>();
            let ids: Vec<ph2d_ecs::Entity> = animated.iter(w).map(|(e, _)| e).collect();
            let read: &ph2d_ecs::World = w;
            ids.into_iter()
                .filter(|e| ph2d_entity_visibility::on_screen_gate::processing_paused(read, *e))
                .collect()
        }
    };
    let world = sim.world_mut();
    // O índice mora na grelha (ADR-0164 F1): sprite sem grelha não tem células e nem entra no laço.
    let mut q = world.query::<(
        ph2d_ecs::Entity,
        &mut SpriteAnimator,
        &SpriteAnimations,
        &mut ph2d_ecs::SpriteGrid,
        Option<&ph2d_ecs::SpriteAnimFrames>,
        Option<&mut ph2d_ecs::SpriteAnimCursor>,
        Option<&ph2d_ecs::SpriteAnimMarkers>,
    )>();
    let mut gritos = Vec::new();
    for (entity, mut animator, tags, mut grid, lists, cursor, markers) in q.iter_mut(world) {
        // Uma folha em pintura toca mesmo com o transporte parado (Enio 23/08, «o preview não está
        // animado»), sem ligar o `playing`. A pausa do enabler vem antes, sem excepção.
        let painted = is_tool_previewed(tool_preview_bits, entity);
        if paused.contains(&entity) {
            continue;
        }
        if !animator.playing && !painted {
            continue;
        }
        let Some(tag) = tags.get(&animator.current) else {
            continue;
        };
        // A pré-visualização corre sobre uma CÓPIA que toca e repete (a lei pura pára sem `playing`);
        // dela volta só o relógio e o frame, e o documento fica intacto.
        let preview_only = painted && !animator.playing;
        let mut run = animator.clone();
        if preview_only {
            run.playing = true;
            run.loop_override = Some(true);
        }
        // O pool é a grelha lida agora: mudar `hframes` encolhe o intervalo no mesmo quadro.
        let cells = grid.cells().max(1);
        let mut frame = grid.frame;
        // O que o artista tem no documento, ANTES de este tique lhe tocar (`crate::preview_drive`).
        let before = ph2d_preview_drive::Driven::SpriteAnim {
            elapsed_ticks: animator.elapsed_ticks,
            pingpong_reverse: animator.pingpong_reverse,
            repeat_count: animator.repeat_count,
            frame,
        };
        // ⭐ A lista de quadros (plano GameEngineFuncional 07): sem ela, o intervalo de sempre.
        let list = lists.and_then(|l| l.get(&tag.name));
        let mut step = cursor.as_ref().map_or(0, |c| c.step);
        gritos.clear();
        let outcome = ph2d_ecs::advance_frames_marked(
            &mut run,
            tag,
            list,
            &mut frame,
            &mut step,
            cells,
            dt * u64::from(ticks),
            markers.filter(|_| !preview_only),
            &mut gritos,
        );
        if let Some(mut c) = cursor
            && c.step != step
        {
            c.step = step;
        }
        if preview_only {
            animator.elapsed_ticks = run.elapsed_ticks;
            animator.pingpong_reverse = run.pingpong_reverse;
            animator.repeat_count = run.repeat_count;
        } else {
            *animator = run;
        }
        // Escreve só quando MUDA: o `bevy` marca a mudança no `deref_mut` e o undo regista por diff.
        if grid.frame != frame {
            grid.frame = frame;
        }
        // A declaração lê o que ficou escrito (no `preview_only`, `run` mentiria sobre o animador).
        let after = ph2d_preview_drive::Driven::SpriteAnim {
            elapsed_ticks: animator.elapsed_ticks,
            pingpong_reverse: animator.pingpong_reverse,
            repeat_count: animator.repeat_count,
            frame,
        };
        // Só quando o tique escreveu: quem não andou não conduz (a `settle` precisa de o ver morrer).
        if before != after {
            drive.driven(entity, before, after);
        }
        // Os sinais (spec §8.10). A pré-visualização é MUDA: pegar no pincel não toca um som.
        if !preview_only {
            out.extend(gritos.drain(..).map(|name| AnimSignal {
                entity,
                name,
                cycles: 1,
            }));
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

/// **O `autoplay` ao abrir o projeto.** Não vive no tique: «começar» é uma aresta, e o bit «já comecei»
/// dessincronizava no primeiro `Ctrl+Z`. Em «Auto» começa do 1.º quadro da animação do «Starts with», mesmo que o ▶ tenha deixado outra no ecrã, ou ela
/// já a tocar a meio do ciclo. Script e Rules escolhem sozinhos: ficam como estão.
pub fn start_autoplay_animations(sim: &mut SimWorld) {
    let world = sim.world_mut();
    let mut q = world.query::<(ph2d_ecs::Entity, &SpriteAnimator, &SpriteAnimations)>();
    let inicios: Vec<(ph2d_ecs::Entity, String)> = q
        .iter(world)
        .filter(|(_, a, ..)| a.autoplay)
        .map(|(e, a, l)| (e, a.start_or_current(l).to_owned()))
        .collect();
    for (e, nome) in inicios {
        vai_para_o_inicio(world, e, &nome);
    }
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

/// Põe `e` no início de `nome`: o tocador rebobinado e a célula de entrada. Nada muda fora de «Auto», ou se a
/// lista já não tem a animação (quem a apagou cai no `current`).
fn vai_para_o_inicio(world: &mut ph2d_ecs::World, e: ph2d_ecs::Entity, nome: &str) {
    let regras = world.get::<ph2d_physics_ecs::AnimRules>(e).is_some()
        || world
            .get::<ph2d_physics_ecs::CharacterAnimations>(e)
            .is_some();
    let Some(mut a) = world.get::<SpriteAnimator>(e).cloned() else {
        return;
    };
    let Some(tag) = world
        .get::<SpriteAnimations>(e)
        .and_then(|l| l.get(nome))
        .cloned()
    else {
        return;
    };
    if regras || !a.autoplay {
        return;
    }
    a.current = nome.to_owned();
    a.rewind(Some(&tag));
    let list = world
        .get::<ph2d_ecs::SpriteAnimFrames>(e)
        .and_then(|f| f.get(nome))
        .map(<[u32]>::to_vec);
    let cells = world
        .get::<ph2d_ecs::SpriteGrid>(e)
        .map_or(1, |g| g.cells().max(1));
    let entrada = ph2d_ecs::entry_cell(&a, &tag, list.as_deref(), cells);
    if let Some((frame, step)) = entrada {
        if let Some(mut g) = world.get_mut::<ph2d_ecs::SpriteGrid>(e) {
            g.frame = frame;
        }
        if let Some(mut c) = world.get_mut::<ph2d_ecs::SpriteAnimCursor>(e) {
            c.step = step;
        }
    }
    if let Some(mut vivo) = world.get_mut::<SpriteAnimator>(e) {
        *vivo = a;
    }
}

/// A entidade está a ser pintada por uma ferramenta (o preview toca a animação sem a gravar).
fn is_tool_previewed(tool_preview_bits: &[Option<u64>], entity: ph2d_ecs::Entity) -> bool {
    tool_preview_bits
        .iter()
        .flatten()
        .any(|b| *b == entity.to_bits())
}
