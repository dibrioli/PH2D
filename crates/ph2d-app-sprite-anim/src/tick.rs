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
    // ⭐⭐ **O `OnScreenEnabler` a PAUSAR de facto** (2026-08-30) — *«só corre quando está no
    // ecrã»*, que é metade do que aqueles cinco campos da §8 prometem. A outra metade (o
    // `HideVisible`) mora no extract.
    //
    // ⚠️ **Colhido ANTES do laço mutável, e não dentro dele**: a decisão precisa de ler a
    // hierarquia inteira (`world_transform` + os ancestrais) enquanto o laço já tem `&mut` sobre o
    // mundo. ⚠️ **E a lista sai vazia numa cena sem um único enabler**, então o caminho comum é
    // uma varredura de componente e o quadro é byte-idêntico ao de antes desta cura.
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
    // ⭐ **`&mut SpriteGrid` e já não `&mut Sprite`** (ADR-0164 F1 passo 6): o índice de célula
    // mudou-se para a grelha. E o filtro passa a ser uma LEI — uma sprite sem grelha não tem
    // células que uma animação possa percorrer, então ela nem entra no laço, em vez de ser
    // tiquetaqueada sobre um pool de uma célula só.
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
        let painted = is_tool_previewed(tool_preview_bits, entity);
        // ⚠️ **A pausa do enabler vem ANTES da pré-visualização**, e é deliberado: uma sprite que
        // saiu do rect que ela própria declara não corre — nem sequer para a célula que a
        // ferramenta mostra ao lado —, senão o «só corre quando está no ecrã» teria uma excepção
        // que ninguém escreveu em lado nenhum.
        if paused.contains(&entity) {
            continue;
        }
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
        // ⚠️ **Escreve só quando MUDA.** O `Sprite` é `SimComponent` e o undo regista por DIFF:
        // tocar-lhe todo o quadro faria uma sprite pausada num frame só produzir um passo de
        // undo por quadro. `bevy` marca a mudança no `deref_mut`, não na atribuição.
        if grid.frame != frame {
            grid.frame = frame;
        }
        // **A DECLARAÇÃO**, com o depois lido do que ficou de facto escrito (o `preview_only`
        // devolve só o relógio, então `run` sozinho mentiria sobre o animador).
        let after = ph2d_preview_drive::Driven::SpriteAnim {
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

/// **O `autoplay` a fazer o que promete**, no único momento em que ele tem significado no editor:
/// o projeto acabou de abrir.
///
/// ⚠️ Ele **não** pode viver no tique: «começar a tocar» é uma aresta, e o tique só vê estados —
/// para a detetar ali seria preciso um bit a mais que diz «já comecei», e um bit desses fica
/// dessincronizado no primeiro `Ctrl+Z`.
///
/// Em «Auto» começa do 1.º quadro da animação do «Starts with», mesmo que o ▶ tenha deixado outra no ecrã, ou ela
/// já a tocar a meio do ciclo. Script e Rules escolhem sozinhos: ficam como estão.
pub fn start_autoplay_animations(sim: &mut SimWorld) {
    let world = sim.world_mut();
    let mut q = world.query::<(
        ph2d_ecs::Entity,
        &SpriteAnimator,
        &SpriteAnimations,
        Option<&ph2d_ecs::SpriteAnimStart>,
    )>();
    let inicios: Vec<(ph2d_ecs::Entity, String)> = q
        .iter(world)
        .filter(|(_, a, ..)| a.autoplay)
        .map(|(e, a, l, s)| (e, ph2d_ecs::SpriteAnimStart::ou_current(s, a, l).to_owned()))
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
