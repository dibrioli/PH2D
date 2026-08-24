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

/// A tag `name` da biblioteca desta sprite — o que o mundo de facto guarda depois da edição.
fn tag_of(sim: &SimWorld, e: Entity, name: &str) -> AnimationTag {
    sim.world()
        .get::<SpriteAnimations>(e)
        .expect("a sprite tem biblioteca")
        .get(name)
        .expect("a animacao existe")
        .clone()
}

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
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(
            &mut sim,
            1,
            0.016,
            &[],
            &mut crate::preview_drive::PreviewDrive::default(),
        );
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
    crate::render_loop::sprite_anim_tick::tick_sprite_animations(
        &mut sim,
        1,
        0.016,
        &[],
        &mut crate::preview_drive::PreviewDrive::default(),
    );
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
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(
            &mut sim,
            1,
            0.016,
            &[],
            &mut crate::preview_drive::PreviewDrive::default(),
        );
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
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(
            &mut sim,
            1,
            0.016,
            &[],
            &mut crate::preview_drive::PreviewDrive::default(),
        );
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

/// **ARRASTAR A BARRA PÕE A CÉLULA, PAUSA, e nunca sai do intervalo.**
///
/// ⚠️ **A pausa é o gesto e não um efeito colateral:** enquanto a reprodução corre, o tique também
/// escreve o `Sprite::frame`, e o dedo e o relógio disputariam o mesmo campo — a imagem piscaria
/// entre a célula arrastada e a que o relógio acabou de pôr. O gate corre o relógio DEPOIS do
/// arrasto, que é a única forma de essa disputa aparecer.
///
/// ⚠️ **E o clamp é do COMMIT.** O painel deriva a célula do snapshot, que é de um quadro atrás;
/// uma célula fora do intervalo ficaria inalcançável pelo `advance` (que só reposiciona o que já
/// está fora, e à ponta de ENTRADA — desfazendo o arrasto no tique seguinte).
///
/// **Mutação que deve sangrar:** tirar o `p.playing = false`, **ou** trocar o `clamp` por uma
/// escrita direta.
#[test]
fn dragging_the_frame_bar_sets_the_cell_pauses_and_stays_inside_the_range() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag {
        frame_ms: 10,
        ..AnimationTag::new("walk", 2, 5)
    })
    .unwrap();
    sim.world_mut().entity_mut(e).insert(lib);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("walk".into()));
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));
    assert!(
        info(&sim, e).playing,
        "a cena estava a TOCAR antes do arrasto"
    );

    edit(&mut sim, e, &reg, AnimFieldEdit::SetFrame(4));
    let after = info(&sim, e);
    assert_eq!(after.frame, 4, "a celula arrastada e' a que fica");
    assert!(
        !after.playing,
        "quem pega no volante conduz -- arrastar a barra tem de PAUSAR"
    );

    // ⚠️ E o relogio tem de deixar a celula quieta. Sem a pausa, isto anda.
    for _ in 0..20 {
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(
            &mut sim,
            1,
            0.016,
            &[],
            &mut crate::preview_drive::PreviewDrive::default(),
        );
    }
    assert_eq!(
        info(&sim, e).frame,
        4,
        "com a reproducao pausada o tique nao pode mexer na celula arrastada"
    );

    // O CLAMP, nas duas pontas: fora do intervalo da tag, o commit fixa.
    edit(&mut sim, e, &reg, AnimFieldEdit::SetFrame(0));
    assert_eq!(info(&sim, e).frame, 2, "abaixo do intervalo fixa no `from`");
    edit(&mut sim, e, &reg, AnimFieldEdit::SetFrame(99));
    assert_eq!(info(&sim, e).frame, 5, "acima do intervalo fixa no `to`");
}

/// **UMA FOLHA EM PINTURA TOCA, mesmo com o transporte parado** (Enio, 2026-08-23: *«o preview não
/// está animado»*).
///
/// ⚠️ Enquanto uma ferramenta pré-visualiza um sprite com grelha, o quad dele **desdobra-se** e o
/// `Sprite::frame` deixa de ter efeito no que se pinta — o único sítio onde ele ainda se vê é a
/// célula de pré-visualização ao lado. Se ela dependesse do `playing`, bastaria o artista ter
/// pausado uma vez (arrastar a barra de frames **pausa**, por desenho) para ela nascer parada. *Uma
/// pré-visualização que existe para mostrar o movimento não pode depender de um interruptor de
/// cena.*
///
/// ⚠️ **E ela NÃO liga o transporte** — a terceira asserção. Ligar `playing` faria a pintura deixar
/// a cena a tocar depois de sair da ferramenta, que é uma edição que ninguém pediu.
///
/// **Mutação que deve sangrar:** tirar o `|| painted` do guarda do tique, **ou** trocá-lo por
/// `animator.playing = true`.
#[test]
fn a_sheet_under_a_tool_preview_plays_even_when_the_transport_is_paused() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag {
        frame_ms: 10,
        ..AnimationTag::new("walk", 0, 7)
    })
    .unwrap();
    sim.world_mut().entity_mut(e).insert(lib);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("walk".into()));
    // ⚠️ PARADO de propósito -- é o estado em que o report caiu.
    assert!(!info(&sim, e).playing, "a fixtura tem de estar PAUSADA");

    // 1. Sem ferramenta a pintar, uma cena pausada não anda.
    let before = info(&sim, e).frame;
    for _ in 0..20 {
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(
            &mut sim,
            1,
            0.016,
            &[],
            &mut crate::preview_drive::PreviewDrive::default(),
        );
    }
    assert_eq!(
        info(&sim, e).frame,
        before,
        "pausada e sem pintura, o tique nao pode mexer no frame"
    );

    // 2. Sob pré-visualização de ferramenta, ela ANDA.
    //
    // ⚠️ **A afirmação é «passou por VÁRIOS quadros», e não «acabou noutro»** — e a diferença
    // custou uma corrida: a 1.ª versão comparava o frame final com o inicial, e depois de 20
    // tiques a `walk` tinha dado a volta e **voltado ao 0**. *Um gate que mede o ponto final de um
    // ciclo é verde ou vermelho por acidente.*
    let painted = [Some(e.to_bits())];
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..20 {
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(
            &mut sim,
            1,
            0.016,
            &painted,
            &mut crate::preview_drive::PreviewDrive::default(),
        );
        seen.insert(info(&sim, e).frame);
    }
    let after = info(&sim, e);
    assert!(
        seen.len() > 3,
        "a folha em pintura tem de tocar -- a pre-visualizacao existe para mostrar o movimento \
         (passou por {:?})",
        seen
    );

    // 3. ⚠️ E o TRANSPORTE continua parado: sair da ferramenta devolve a cena como ela estava.
    assert!(
        !after.playing,
        "a pintura nao pode LIGAR o transporte -- isso e' uma edicao que ninguem pediu"
    );

    // 4. E a pré-visualização de OUTRA entidade não põe esta a tocar.
    let other = [Some(e.to_bits() ^ 0xFFFF)];
    let held = info(&sim, e).frame;
    for _ in 0..20 {
        crate::render_loop::sprite_anim_tick::tick_sprite_animations(
            &mut sim,
            1,
            0.016,
            &other,
            &mut crate::preview_drive::PreviewDrive::default(),
        );
        assert_eq!(
            info(&sim, e).frame,
            held,
            "a pintura do vizinho nao toca esta -- e a asserção é DENTRO do laço, senão ela \
             mediria outra vez o fim de um ciclo"
        );
    }
}

/// **DECLARAR A DURAÇÃO DE UM QUADRO NÃO ESCREVE OS OUTROS** (spec §8.12, pedido do Enio:
/// *«se não tiver um parâmetro de duração para cada quadro, crie»*).
///
/// ⚠️ O vetor cresce **só até à célula tocada**, e o resto fica a `0` — que quer dizer *herda o
/// `Frame ms`*. Materializar as oito células ao tocar numa gravaria sete números que dizem o que o
/// `frame_ms` já diz, e o primeiro que o artista mudasse deixaria os outros seis congelados no
/// valor de então.
///
/// **Mutação que deve sangrar:** preencher o vetor com o `frame_ms` em vez de `0`.
#[test]
fn setting_one_frames_duration_does_not_write_the_others() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 7)).unwrap();
    sim.world_mut().entity_mut(e).insert(lib);

    edit(&mut sim, e, &reg, AnimFieldEdit::FrameMsAt(0, 2, 400));
    let tag = tag_of(&sim, e, "walk");
    assert_eq!(
        tag.per_frame_ms,
        vec![0, 0, 400],
        "o vetor cresce ate' a celula tocada e o resto HERDA"
    );
    assert!(tag.has_per_frame_timing());
}

/// **E LIMPAR O ÚLTIMO VALOR ENCOLHE O VETOR** — sem a poda, ele ficaria cheio de zeros: invisível
/// no ecrã, gravado no ficheiro, e a fazer o aviso *«this animation has per-frame timing»* mentir
/// para sempre.
///
/// ⛔ *Um estado que só existe como resíduo é um estado que ninguém volta a explicar.*
///
/// **Mutação que deve sangrar:** tirar o `while … pop()`.
#[test]
fn clearing_the_last_value_shrinks_the_vector_back_to_nothing() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 7)).unwrap();
    sim.world_mut().entity_mut(e).insert(lib);

    edit(&mut sim, e, &reg, AnimFieldEdit::FrameMsAt(0, 1, 200));
    edit(&mut sim, e, &reg, AnimFieldEdit::FrameMsAt(0, 3, 300));
    assert_eq!(tag_of(&sim, e, "walk").per_frame_ms, vec![0, 200, 0, 300]);

    // Limpa o último: o vetor encolhe até ao que ainda declara alguma coisa.
    edit(&mut sim, e, &reg, AnimFieldEdit::FrameMsAt(0, 3, 0));
    assert_eq!(tag_of(&sim, e, "walk").per_frame_ms, vec![0, 200]);
    // E limpar o que resta deixa-o VAZIO — a animação volta a ser de ritmo uniforme.
    edit(&mut sim, e, &reg, AnimFieldEdit::FrameMsAt(0, 1, 0));
    let tag = tag_of(&sim, e, "walk");
    assert!(
        tag.per_frame_ms.is_empty(),
        "sobrou residuo: {:?}",
        tag.per_frame_ms
    );
    assert!(
        !tag.has_per_frame_timing(),
        "e o aviso do painel tem de sumir"
    );
}

/// **Limpar uma célula que nunca foi declarada não escreve nada** — sem isto, pôr `0` num campo
/// que já mostrava `0` faria o vetor crescer só para guardar zeros.
#[test]
fn clearing_a_cell_that_was_never_declared_writes_nothing() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 7)).unwrap();
    sim.world_mut().entity_mut(e).insert(lib);

    edit(&mut sim, e, &reg, AnimFieldEdit::FrameMsAt(0, 5, 0));
    assert!(tag_of(&sim, e, "walk").per_frame_ms.is_empty());
}

/// **O cap da spec vale nesta porta também** — a escrita cobre o gesto, e a lei pura volta a
/// impô-lo na leitura (é a mesma dupla do `FrameMs`).
#[test]
fn the_spec_cap_holds_at_this_door_too() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 7)).unwrap();
    sim.world_mut().entity_mut(e).insert(lib);

    edit(&mut sim, e, &reg, AnimFieldEdit::FrameMsAt(0, 0, u32::MAX));
    assert_eq!(
        tag_of(&sim, e, "walk").per_frame_ms[0],
        ph2d_ecs::FRAME_MS_MAX
    );
}
