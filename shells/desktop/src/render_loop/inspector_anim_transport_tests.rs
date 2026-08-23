//! **O TRANSPORTE da §11 Animation** — os gates de *o que toca, e a partir de onde*.
//!
//! ⚠️ **Irmão de [`super`] por CAP de FICHEIRO do shell (HR-18, 600).** O corte é por lei: a
//! AUTORIA (biblioteca, nomes, intervalos) e o snapshot ficam lá; aqui fica o tocador — rebobinar,
//! escolher, ligar, retomar. São as duas metades que a própria seção separa com dois rótulos.
//!
//! ⚠️ **Todos estes nasceram do mesmo report** (Enio, 2026-08-23: *«às vezes preciso clicar mais de
//! uma vez para checar Playing»*). A caixa era só a porta pela qual o defeito aparecia: por baixo
//! dela, *rebobinar* não movia a imagem e *ligar* uma animação já terminada era um gesto morto.

use super::*;

/// **REBOBINAR MOVE A IMAGEM** — e não só contadores que ninguém vê.
///
/// ⚠️ Enio, 2026-08-23. O botão repunha `elapsed_ticks`/`repeat_count`/o ping-pong e deixava a
/// sprite na célula onde tinha parado: carregar em «Rewind» **não fazia nada visível**.
///
/// **Mutação que deve sangrar:** trocar o `rewind_to_start` por `p.rewind(tag)`.
#[test]
fn rewinding_puts_the_picture_back_at_the_start() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 2, 6)).unwrap();
    lib.insert(AnimationTag {
        direction: AnimDirection::Reverse,
        ..AnimationTag::new("back", 2, 6)
    })
    .unwrap();
    sim.world_mut().entity_mut(e).insert(lib);
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("walk".into()));

    // A sprite parou a MEIO do intervalo — dentro dele, que é o caso que o `advance` nao toca.
    if let Some(mut s) = sim.world_mut().get_mut::<ph2d_render::Sprite>(e) {
        s.frame = 5;
    }
    edit(&mut sim, e, &reg, AnimFieldEdit::Rewind);
    assert_eq!(
        info(&sim, e).frame,
        2,
        "rebobinar tem de devolver a imagem a' primeira celula do intervalo"
    );

    // ⚠️ E a ponta certa segue a DIREÇÃO: uma tag ao contrario rebobina para o fim.
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("back".into()));
    assert_eq!(
        info(&sim, e).frame,
        6,
        "uma tag Reverse comeca no fim do intervalo"
    );
}

/// **ESCOLHER OUTRA ANIMAÇÃO COMEÇA-A DO PRINCÍPIO** — mesmo quando os intervalos se sobrepõem.
///
/// ⚠️ **É a sobreposição que faz esta lei ser precisa e não decorativa**: a tese do modelo é que as
/// animações partilham o pool de células (`idle` 0-3 e `walk` 0-7 na cena de smoke), e o `advance`
/// só reposiciona um frame que caia FORA do intervalo. Sem esta lei, sair de uma `walk` na célula
/// 2 para a `idle` começava-a a meio, e o artista via um salto que não pediu.
///
/// **Mutação que deve sangrar:** a mesma de cima, no braço do `SetCurrent`.
#[test]
fn choosing_an_overlapping_animation_still_starts_it_at_its_own_beginning() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 7)).unwrap();
    lib.insert(AnimationTag::new("idle", 3, 5)).unwrap();
    sim.world_mut().entity_mut(e).insert(lib);
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("walk".into()));
    if let Some(mut s) = sim.world_mut().get_mut::<ph2d_render::Sprite>(e) {
        s.frame = 4; // ⚠️ DENTRO do intervalo da `idle` tambem — a fixtura contem o fenomeno.
    }
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("idle".into()));
    assert_eq!(
        info(&sim, e).frame,
        3,
        "a `idle` tem de comecar na PRIMEIRA celula DELA, e nao onde a `walk` estava"
    );
}

/// **LIGAR UMA ANIMAÇÃO QUE JÁ ACABOU REVÊ-A** — o defeito irmão do da caixa.
///
/// ⚠️ **Sem isto, o interruptor é MORTO e não «lento»:** numa tag de uma volta já terminada, pôr
/// `playing = true` deixa a imagem na ponta com o contador cheio, e o primeiro passo de `advance`
/// fecha o ciclo outra vez e volta a parar — no mesmo tique. O gate corre o relógio DEPOIS de
/// ligar, que é a única forma de a metade morta aparecer.
///
/// ⚠️ E a segunda metade é a que impede a cura de se tornar *«ligar rebobina sempre»*: quem pausou
/// a meio continua de onde estava.
///
/// **Mutação que deve sangrar:** tirar o `if ... is_finished` do braço `Playing`.
#[test]
fn turning_playing_back_on_replays_an_animation_that_had_finished() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag {
        repeat: Some(1),
        frame_ms: 10,
        ..AnimationTag::new("attack", 4, 7)
    })
    .unwrap();
    sim.world_mut().entity_mut(e).insert(lib);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::SetCurrent("attack".into()),
    );
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));

    // Corre ate' ela se parar sozinha, na ULTIMA celula.
    for _ in 0..40 {
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(&mut sim, 1, 0.016);
    }
    assert!(!info(&sim, e).playing, "a de uma volta para-se sozinha");
    assert_eq!(info(&sim, e).frame, 7, "e para na ULTIMA celula");

    // O artista marca a caixa outra vez. ⚠️ E o relogio corre — sem isto o gate ficaria verde
    // sobre um `playing = true` que o proximo tique apaga.
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));
    assert_eq!(
        info(&sim, e).frame,
        4,
        "ligar de novo devolve a imagem ao inicio"
    );
    crate::render_loop::sprite_anim_tick::tick_sprite_animations(&mut sim, 1, 0.016);
    let after = info(&sim, e);
    assert!(
        after.playing,
        "e ela tem de CONTINUAR a tocar depois do primeiro tique -- \
         um `playing` que dura um quadro e' um interruptor morto"
    );
    assert!(
        after.frame > 4,
        "e a imagem tem de andar (viu frame {})",
        after.frame
    );

    // ⚠️ A OUTRA METADE: quem PAUSOU a meio continua de onde estava.
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(false));
    let paused = info(&sim, e).frame;
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));
    assert_eq!(
        info(&sim, e).frame,
        paused,
        "retomar uma pausa nao pode rebobinar -- so' o FIM rebobina"
    );
}

/// **ESCOLHER OUTRA ANIMAÇÃO RETOMA A QUE TINHA ACABADO — e respeita uma PAUSA.**
///
/// ⚠️ É a segunda porta da mesma lei do `turning_playing_back_on_replays…`: quem parou a
/// reprodução foi o fim do ciclo, e não o artista. Sem isto, a sequência que a cena de smoke
/// descreve — *tocar `attack` uma vez, depois clicar em `walk` para voltar ao ciclo* — deixava a
/// sprite parada, e o artista ia procurar a caixa de `Playing` para reparar um estado que ele
/// nunca criou (Enio, 2026-08-23).
///
/// ⚠️ **As duas metades**, e a segunda é a que impede a cura de virar *«clicar na lista toca
/// sempre»*: quem desmarcou `Playing` a meio de um ciclo continua a poder folhear a lista em
/// silêncio.
///
/// **Mutação que deve sangrar:** tirar o `if !p.playing && … is_finished` do braço `SetCurrent`,
/// **ou** trocá-lo por `if !p.playing`.
#[test]
fn choosing_another_animation_resumes_one_that_had_run_itself_out_but_not_a_pause() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag {
        repeat: Some(1),
        frame_ms: 10,
        ..AnimationTag::new("attack", 4, 7)
    })
    .unwrap();
    lib.insert(AnimationTag {
        frame_ms: 10,
        ..AnimationTag::new("walk", 0, 7)
    })
    .unwrap();
    sim.world_mut().entity_mut(e).insert(lib);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);

    // 1. A `attack` corre ate' se esgotar sozinha.
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::SetCurrent("attack".into()),
    );
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));
    for _ in 0..40 {
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(&mut sim, 1, 0.016);
    }
    assert!(!info(&sim, e).playing, "a de uma volta esgotou-se");

    // 2. Escolher a `walk` volta a tocar — e ANDA no tique seguinte.
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("walk".into()));
    assert!(
        info(&sim, e).playing,
        "escolher outra animacao depois de uma se ESGOTAR tem de a tocar"
    );
    let start = info(&sim, e).frame;
    for _ in 0..3 {
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(&mut sim, 1, 0.016);
    }
    assert!(
        info(&sim, e).frame != start,
        "e a imagem tem de andar de verdade"
    );

    // 3. ⚠️ A PAUSA EXPLICITA e' respeitada: desmarcar a meio e folhear a lista fica em silencio.
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(false));
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::SetCurrent("attack".into()),
    );
    assert!(
        !info(&sim, e).playing,
        "quem PAUSOU a meio de um ciclo continua a poder folhear a lista em silencio"
    );
}
