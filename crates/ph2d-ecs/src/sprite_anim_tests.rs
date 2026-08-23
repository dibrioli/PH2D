//! **Os gates da §11 Animation** — irmão de [`super::sprite_anim`] por CAP de LOC.
//!
//! ⚠️ **A lei desta seção é o [`super::advance`]**, e ela é subtil: quatro direções, repetição
//! finita, o *hold* do último frame, o atraso entre ciclos e uma velocidade que pode ser negativa.
//! Cada um destes é uma decisão que compila estando errada — por isso a suíte segue **sequências
//! de frames**, e não estados isolados: a ordem é o que o artista vê.

use super::*;

const MS: u64 = 1_000;

/// Corre a lei em passos de `frame_ms/2` e devolve a sequência de frames por que ela passou.
///
/// ⚠️ **Meio frame por passo de propósito**: um passo igual à duração exata esconderia um erro de
/// fronteira (`>=` contra `>`), e um passo muito pequeno faria a suíte medir o acumulador em vez
/// da lei.
fn walk(tag: &AnimationTag, animator: &mut SpriteAnimator, cells: u32, steps: usize) -> Vec<u32> {
    let mut frame = u32::MAX; // fora do intervalo: a lei entra pela ponta certa
    let dt = u64::from(tag.frame_ms) * MS / 2;
    let mut seen = Vec::new();
    for _ in 0..steps {
        advance(animator, tag, &mut frame, cells, dt);
        if seen.last() != Some(&frame) {
            seen.push(frame);
        }
    }
    seen
}

fn playing(tag: &AnimationTag) -> SpriteAnimator {
    let mut a = SpriteAnimator::new(&tag.name);
    a.playing = true;
    a.rewind(Some(tag));
    a
}

fn tag_dir(name: &str, from: u32, to: u32, dir: AnimDirection) -> AnimationTag {
    AnimationTag {
        direction: dir,
        ..AnimationTag::new(name, from, to)
    }
}

/// **Forward percorre o intervalo e volta ao princípio.**
#[test]
fn the_forward_cycle_walks_the_range_and_wraps() {
    let t = tag_dir("walk", 0, 2, AnimDirection::Forward);
    let mut a = playing(&t);
    assert_eq!(walk(&t, &mut a, 3, 12), vec![0, 1, 2, 0, 1, 2, 0]);
    assert!(a.repeat_count >= 2, "os ciclos tem de contar");
}

/// **Reverse entra pela ponta LONGE e anda para trás.**
///
/// ⚠️ A entrada é metade do comportamento: uma `Reverse` que entrasse no `from` tocaria o primeiro
/// frame e só depois saltaria — um salto que o artista lê como defeito.
#[test]
fn reverse_enters_at_the_far_end_and_walks_back() {
    let t = tag_dir("back", 0, 2, AnimDirection::Reverse);
    let mut a = playing(&t);
    assert_eq!(walk(&t, &mut a, 3, 12), vec![2, 1, 0, 2, 1, 0, 2]);
}

/// **Ping-pong NÃO repete as pontas.**
///
/// ⚠️ É o erro clássico desta lei: bater no fim e mandar o mesmo frame outra vez dá um «soluço»
/// de dois quadros iguais em cada volta. A sequência certa é `0 1 2 1 0 1 2`.
#[test]
fn ping_pong_does_not_repeat_the_endpoints() {
    let t = tag_dir("bob", 0, 2, AnimDirection::PingPong);
    let mut a = playing(&t);
    assert_eq!(walk(&t, &mut a, 3, 16), vec![0, 1, 2, 1, 0, 1, 2, 1, 0]);
}

/// E o irmão dele começa pela outra ponta.
#[test]
fn ping_pong_reverse_starts_at_the_far_end() {
    let t = tag_dir("bob_r", 0, 2, AnimDirection::PingPongReverse);
    let mut a = playing(&t);
    assert_eq!(walk(&t, &mut a, 3, 16), vec![2, 1, 0, 1, 2, 1, 0, 1, 2]);
}

/// **⭐ Uma animação que toca UMA VEZ pára no ÚLTIMO frame** — não volta ao primeiro.
///
/// ⚠️ **A primeira versão desta lei voltava**, e o defeito é de produto, não de aritmética: a pose
/// final de um *attack* é o resultado do gesto. Parar no início deixaria a sprite em repouso, que
/// é o oposto do que o gesto acabou de fazer. É o que o Godot e o Phaser fazem.
#[test]
fn a_play_once_animation_stops_on_the_last_frame() {
    let t = AnimationTag {
        repeat: Some(1),
        ..tag_dir("attack", 0, 2, AnimDirection::Forward)
    };
    let mut a = playing(&t);
    let seq = walk(&t, &mut a, 3, 20);
    assert_eq!(seq, vec![0, 1, 2], "toca uma vez e fica no fim");
    assert!(!a.playing, "tem de parar");
    assert_eq!(a.repeat_count, 1);

    // E o mesmo pela outra ponta: uma `Reverse` de uma volta pára no `from`.
    let t = AnimationTag {
        repeat: Some(1),
        ..tag_dir("attack_r", 0, 2, AnimDirection::Reverse)
    };
    let mut a = playing(&t);
    assert_eq!(walk(&t, &mut a, 3, 20), vec![2, 1, 0]);
    assert!(!a.playing);
}

/// **O `hold_ms` alonga SÓ o último frame do ciclo** — é a «respiração» do idle.
#[test]
fn the_hold_only_lengthens_the_last_frame() {
    let t = AnimationTag {
        frame_ms: 100,
        hold_ms: 200,
        ..tag_dir("idle", 0, 1, AnimDirection::Forward)
    };
    let mut a = playing(&t);
    let mut frame = u32::MAX;

    advance(&mut a, &t, &mut frame, 2, 99 * MS);
    assert_eq!(frame, 0, "o primeiro frame dura o normal");
    advance(&mut a, &t, &mut frame, 2, 2 * MS);
    assert_eq!(frame, 1);
    // ⚠️ Sem o hold, isto já teria voltado a 0 — o teste falharia aqui.
    advance(&mut a, &t, &mut frame, 2, 250 * MS);
    assert_eq!(
        frame, 1,
        "o ultimo frame do ciclo segura {}ms a mais",
        t.hold_ms
    );
    advance(&mut a, &t, &mut frame, 2, 60 * MS);
    assert_eq!(frame, 0, "e depois volta");
}

/// **O `repeat_delay_ms` só conta quando ainda VEM outro ciclo.**
///
/// ⚠️ É o que o distingue do `hold`, e a diferença é observável: cobrá-lo na última volta faria a
/// animação ficar meio segundo parada **depois de já ter acabado**.
#[test]
fn the_repeat_delay_only_counts_when_another_cycle_follows() {
    let t = AnimationTag {
        frame_ms: 100,
        repeat_delay_ms: 500,
        repeat: Some(2),
        ..tag_dir("twice", 0, 1, AnimDirection::Forward)
    };
    let mut a = playing(&t);
    let mut frame = u32::MAX;

    advance(&mut a, &t, &mut frame, 2, 101 * MS);
    assert_eq!(frame, 1, "ciclo 1, ultimo frame");
    advance(&mut a, &t, &mut frame, 2, 400 * MS);
    assert_eq!(frame, 1, "o atraso entre ciclos ainda corre");
    advance(&mut a, &t, &mut frame, 2, 250 * MS);
    assert_eq!(frame, 0, "ciclo 2 comeca");
    assert_eq!(a.repeat_count, 1);

    // Última volta: o atraso NÃO se cobra, e ela pára no último frame.
    advance(&mut a, &t, &mut frame, 2, 101 * MS);
    assert_eq!(frame, 1);
    advance(&mut a, &t, &mut frame, 2, 101 * MS);
    assert_eq!(frame, 1, "parou no fim");
    assert!(!a.playing);
    assert_eq!(a.repeat_count, 2);
}

/// **Uma velocidade NEGATIVA toca ao contrário — ela não faz o tempo andar para trás.**
///
/// ⚠️ A alternativa (somar um delta negativo ao acumulador) obriga a «desavançar» frames com o
/// resto do frame anterior, que é um segundo modelo de tempo dentro do mesmo estado.
#[test]
fn a_negative_speed_plays_backwards_without_running_time_backwards() {
    let t = tag_dir("walk", 0, 2, AnimDirection::Forward);
    let mut a = playing(&t);
    a.speed_q16 = -SPEED_ONE_Q16;
    assert_eq!(walk(&t, &mut a, 3, 12), vec![0, 2, 1, 0, 2, 1, 0]);
}

/// Uma velocidade **dupla** anda o dobro dos frames no mesmo tempo.
#[test]
fn the_speed_scale_is_a_rate_not_a_frame_skip() {
    let t = tag_dir("walk", 0, 3, AnimDirection::Forward);
    let mut a = playing(&t);
    a.speed_q16 = 2 * SPEED_ONE_Q16;
    let mut frame = u32::MAX;
    advance(&mut a, &t, &mut frame, 4, 100 * MS); // 100 ms a 2× = 2 frames
    assert_eq!(frame, 2);
}

/// **Velocidade zero é PAUSA, e não perde o estado.**
#[test]
fn zero_speed_pauses_without_losing_the_position() {
    let t = tag_dir("walk", 0, 2, AnimDirection::Forward);
    let mut a = playing(&t);
    let mut frame = u32::MAX;
    advance(&mut a, &t, &mut frame, 3, 150 * MS);
    assert_eq!(frame, 1);
    let elapsed = a.elapsed_ticks;
    a.speed_q16 = 0;
    advance(&mut a, &t, &mut frame, 3, 10_000 * MS);
    assert_eq!(frame, 1, "pausado nao anda");
    assert_eq!(a.elapsed_ticks, elapsed, "e nao acumula por baixo");
}

/// **Um intervalo de UMA célula não avança e não conta ciclos.**
///
/// ⚠️ Sem esta guarda, um `repeat: Some(3)` sobre uma célula terminaria no primeiro tique — os
/// ciclos correriam à velocidade do relógio em vez da do frame.
#[test]
fn a_single_cell_range_never_advances_and_never_loops() {
    let t = AnimationTag {
        repeat: Some(3),
        ..tag_dir("still", 1, 1, AnimDirection::Forward)
    };
    let mut a = playing(&t);
    let mut frame = u32::MAX;
    advance(&mut a, &t, &mut frame, 4, 10_000 * MS);
    assert_eq!(frame, 1);
    assert_eq!(a.repeat_count, 0);
    assert!(a.playing);
}

/// **O intervalo é DERIVADO contra a grelha de hoje** — encolher `hframes` não deixa uma tag a
/// apontar para fora.
#[test]
fn the_range_is_resolved_against_the_grid_that_exists_now() {
    let t = tag_dir("walk", 2, 7, AnimDirection::Forward);
    assert_eq!(t.resolve(8), Some((2, 7)));
    assert_eq!(t.resolve(5), Some((2, 4)), "a grelha encolheu: o fim recua");
    assert_eq!(t.resolve(2), None, "a grelha encolheu abaixo do inicio");
    assert_eq!(t.resolve(0), None);
    // ⚠️ `from > to` é um estado que a UI pode produzir a meio de uma edição: lê-se ordenado.
    assert_eq!(
        tag_dir("x", 5, 1, AnimDirection::Forward).resolve(8),
        Some((1, 5))
    );

    // E a lei não mexe em nada quando não há intervalo.
    let mut a = playing(&t);
    let mut frame = 3;
    advance(&mut a, &t, &mut frame, 1, 10_000 * MS);
    assert_eq!(frame, 3, "sem intervalo, nada se escreve");
}

/// **Um frame fora do intervalo entra pela ponta CERTA**, mesmo pausado.
///
/// ⚠️ Pausado também: trocar de tag com a animação parada tem de mostrar já o primeiro frame dela,
/// senão o artista escolhe `attack` e continua a ver a pose de `idle`.
#[test]
fn a_frame_outside_the_range_enters_by_the_right_end_even_when_paused() {
    for (dir, want) in [
        (AnimDirection::Forward, 4),
        (AnimDirection::Reverse, 6),
        (AnimDirection::PingPong, 4),
        (AnimDirection::PingPongReverse, 6),
    ] {
        let t = tag_dir("x", 4, 6, dir);
        let mut a = SpriteAnimator::new("x");
        a.rewind(Some(&t));
        let mut frame = 0;
        let out = advance(&mut a, &t, &mut frame, 8, 5_000 * MS);
        assert_eq!(frame, want, "{dir:?} entrou pela ponta errada");
        assert!(out.frame_changed);
    }
}

/// **Os sinais dizem o que aconteceu** (spec §8.10): frame mudou, ciclo fechou, acabou.
#[test]
fn the_outcome_reports_what_the_step_did() {
    let t = AnimationTag {
        repeat: Some(1),
        ..tag_dir("once", 0, 1, AnimDirection::Forward)
    };
    let mut a = playing(&t);
    let mut frame = u32::MAX;

    let out = advance(&mut a, &t, &mut frame, 2, 50 * MS);
    assert!(out.frame_changed, "a entrada no intervalo e' uma mudanca");
    assert_eq!((out.looped, out.finished), (0, false));

    let out = advance(&mut a, &t, &mut frame, 2, 60 * MS);
    assert!(out.frame_changed);
    assert_eq!((out.looped, out.finished), (0, false));

    let out = advance(&mut a, &t, &mut frame, 2, 110 * MS);
    assert_eq!(
        (out.looped, out.finished),
        (1, true),
        "o fim do unico ciclo e' um LOOP e um FINISHED"
    );
    assert!(!out.frame_changed, "e ela ficou no ultimo frame");
}

/// **Os três estados do animador são distinguíveis** — a mesma lei do `MountState`.
#[test]
fn the_three_animator_states_are_told_apart() {
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 3)).unwrap();

    let none = SpriteAnimator::default();
    assert_eq!(animator_state(&none, Some(&lib), 4), AnimatorState::NoTag);

    let ok = SpriteAnimator::new("walk");
    assert_eq!(animator_state(&ok, Some(&lib), 4), AnimatorState::Ready);

    let gone = SpriteAnimator::new("run");
    assert_eq!(
        animator_state(&gone, Some(&lib), 4),
        AnimatorState::Dangling
    );
    // ⚠️ **E a tag pode existir e mesmo assim estar pendurada** — a grelha encolheu para debaixo
    // dela. Um estado que só olhasse o nome diria «pronta» sobre uma animação que não toca.
    assert_eq!(
        animator_state(&ok, Some(&lib), 0),
        AnimatorState::Dangling,
        "a grelha vazia deixa toda tag pendurada"
    );
    assert_eq!(animator_state(&ok, None, 4), AnimatorState::Dangling);
}

/// Trocar de tag **repõe** o ciclo — senão a primeira volta da tag nova começa a meio.
#[test]
fn rewinding_resets_the_cycle_state() {
    let t = tag_dir("bob", 0, 2, AnimDirection::PingPong);
    let mut a = playing(&t);
    let mut frame = u32::MAX;
    advance(&mut a, &t, &mut frame, 3, 1_000 * MS);
    assert!(a.repeat_count > 0);

    let other = tag_dir("back", 0, 2, AnimDirection::Reverse);
    a.current = other.name.clone();
    a.rewind(Some(&other));
    assert_eq!(a.repeat_count, 0);
    assert_eq!(a.elapsed_ticks, 0);
    assert!(a.pingpong_reverse, "uma Reverse parte da ponta longe");
}

/// Os caps da spec §8.11 são impostos, e a lista tem uma porta só.
#[test]
fn the_library_enforces_the_caps_it_declares() {
    assert_eq!(validate_tag_name(""), Err(AnimTagError::Empty));
    assert_eq!(
        validate_tag_name(&"a".repeat(ANIM_NAME_MAX_BYTES + 1)),
        Err(AnimTagError::TooLong)
    );
    assert_eq!(
        validate_tag_name("bad\u{1b}[0m"),
        Err(AnimTagError::ControlChar)
    );

    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 1)).unwrap();
    assert_eq!(
        lib.insert(AnimationTag::new("walk", 2, 3)),
        Err(AnimTagError::Duplicate),
        "dois nomes iguais tornam `get` ambiguo"
    );
    while lib.len() < ANIM_TAGS_MAX {
        let n = lib.next_free_name();
        lib.insert(AnimationTag::new(n, 0, 1))
            .expect("dentro do cap");
    }
    assert_eq!(
        lib.insert(AnimationTag::new("one_too_many", 0, 1)),
        Err(AnimTagError::ListFull)
    );
}

/// O nome oferecido está sempre livre — **mesmo com buracos na sequência**.
#[test]
fn the_offered_tag_name_is_always_free() {
    let mut lib = SpriteAnimations::new();
    for n in [0, 1, 3] {
        lib.insert(AnimationTag::new(format!("anim_{n}"), 0, 1))
            .unwrap();
    }
    assert_eq!(lib.next_free_name(), "anim_2", "saltou o buraco");
    assert!(lib.remove("anim_1"));
    assert_eq!(lib.next_free_name(), "anim_1");
    assert!(!lib.remove("nope"));
}

/// **Um `frame_ms` de ZERO vindo do disco corre ao PISO de 1 ms, não à velocidade do relógio.**
///
/// ⚠️ **A primeira versão deste teste era VAZIA, e foi a mutação que o disse:** ela afirmava só
/// que o laço termina — e quem o termina é a rede de 1024 passos, não o clamp. Tirar o clamp
/// deixava-a verde enquanto a animação queimava 256 ciclos num tique.
///
/// Agora ele mede o que o clamp **faz**: um passo de 1 ms avança **um** frame e fecha **zero**
/// ciclos. Sem o clamp seriam 1024 passos sobre 4 células — 256 ciclos, e o `repeat_count` diria.
#[test]
fn a_zero_frame_ms_from_disk_runs_at_the_one_millisecond_floor() {
    let t = AnimationTag {
        frame_ms: 0,
        ..tag_dir("bad", 0, 3, AnimDirection::Forward)
    };
    let mut a = playing(&t);
    let mut frame = u32::MAX;
    advance(&mut a, &t, &mut frame, 4, MS);
    assert_eq!(frame, 1, "um milissegundo tem de avancar UM frame");
    assert_eq!(
        a.repeat_count, 0,
        "sem o clamp isto seriam 256 ciclos num unico tique"
    );
}

/// Round-trip pelo postcard — é como os dois componentes viajam no save e no undo.
#[test]
fn both_components_round_trip_through_postcard() {
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag {
        frame_ms: 42,
        direction: AnimDirection::PingPongReverse,
        repeat: Some(3),
        hold_ms: 120,
        repeat_delay_ms: 480,
        ..AnimationTag::new("attack", 4, 11)
    })
    .unwrap();
    let bytes = postcard::to_allocvec(&lib).unwrap();
    assert_eq!(
        postcard::from_bytes::<SpriteAnimations>(&bytes).unwrap(),
        lib
    );

    let a = SpriteAnimator {
        current: "attack".into(),
        playing: true,
        autoplay: true,
        speed_q16: -2 * SPEED_ONE_Q16,
        direction_override: Some(AnimDirection::Reverse),
        loop_override: Some(false),
        elapsed_ticks: 12_345,
        pingpong_reverse: true,
        repeat_count: 7,
    };
    let bytes = postcard::to_allocvec(&a).unwrap();
    assert_eq!(postcard::from_bytes::<SpriteAnimator>(&bytes).unwrap(), a);
}

/// **«Acabou» e «foi pausada» leem-se igual no `playing`, e não são a mesma coisa.**
///
/// ⚠️ É esta distinção que faz o interruptor *Playing* ser um gesto vivo: sem ela, voltar a ligar
/// uma animação de uma volta é pedir a [`advance`] que feche o ciclo outra vez no primeiro passo.
#[test]
fn a_finished_animation_is_told_apart_from_a_paused_one() {
    let once = AnimationTag {
        repeat: Some(1),
        ..AnimationTag::new("attack", 0, 3)
    };
    let forever = AnimationTag::new("walk", 0, 3);

    // Recem-rebobinada: nenhuma das duas acabou.
    let a = playing(&once);
    assert!(!a.is_finished(&once));
    assert!(!playing(&forever).is_finished(&forever));

    // Corrida ate' parar: a de uma volta ACABOU.
    let mut a = playing(&once);
    let _ = walk(&once, &mut a, 4, 20);
    assert!(!a.playing, "a de uma volta tem de parar");
    assert!(
        a.is_finished(&once),
        "e o motivo de ela ter parado e' o FIM"
    );

    // A que repete para sempre corre o mesmo tanto e NUNCA acaba.
    let mut b = playing(&forever);
    let _ = walk(&forever, &mut b, 4, 20);
    assert!(b.playing);
    assert!(
        !b.is_finished(&forever),
        "uma tag sem teto de ciclos nunca acaba, por mais que corra"
    );

    // ⚠️ O `loop_override` MANDA nos dois sentidos — a pergunta e' sobre a lei efetiva.
    let mut c = playing(&forever);
    c.loop_override = Some(false);
    let _ = walk(&forever, &mut c, 4, 20);
    assert!(
        c.is_finished(&forever),
        "com o loop forcado a UMA volta, a tag infinita acaba"
    );
    let mut d = playing(&once);
    d.loop_override = Some(true);
    let _ = walk(&once, &mut d, 4, 20);
    assert!(
        !d.is_finished(&once),
        "com o loop forcado a SEMPRE, a de uma volta nao acaba"
    );
}

/// **A célula de entrada é a ponta por onde a direção efetiva começa** — e é da grelha de hoje.
///
/// ⚠️ **Ela é o que faltava ao rebobinar.** Repor os contadores e deixar a imagem onde estava dá
/// um botão que não faz nada — e, com um `repeat` finito, um botão que não faz nada **duas** vezes
/// (o primeiro passo volta a fechar o ciclo a partir da ponta).
#[test]
fn the_entry_cell_is_the_end_the_effective_direction_starts_from() {
    let fwd = tag_dir("a", 2, 5, AnimDirection::Forward);
    let rev = tag_dir("b", 2, 5, AnimDirection::Reverse);
    let pp = tag_dir("c", 2, 5, AnimDirection::PingPong);
    let ppr = tag_dir("d", 2, 5, AnimDirection::PingPongReverse);
    let a = SpriteAnimator::new("a");
    assert_eq!(entry_frame(&a, &fwd, 8), Some(2));
    assert_eq!(entry_frame(&a, &rev, 8), Some(5));
    assert_eq!(entry_frame(&a, &pp, 8), Some(2), "o ping-pong parte do lo");
    assert_eq!(entry_frame(&a, &ppr, 8), Some(5), "e o reverso do hi");

    // ⚠️ A DIREÇÃO EFETIVA, e nao a da tag: o override manda aqui como manda em `advance`.
    let mut o = SpriteAnimator::new("a");
    o.direction_override = Some(AnimDirection::Reverse);
    assert_eq!(entry_frame(&o, &fwd, 8), Some(5));

    // ⚠️ A grelha de HOJE: se ela encolheu, o `hi` recua com ela.
    assert_eq!(entry_frame(&a, &rev, 4), Some(3));
    assert_eq!(
        entry_frame(&a, &rev, 2),
        None,
        "a tag nao alcanca celula nenhuma"
    );

    // E e' EXATAMENTE por onde `advance` faz uma reproducao rebobinada entrar.
    for t in [&fwd, &rev, &pp, &ppr] {
        let mut anim = playing(t);
        let mut frame = u32::MAX;
        advance(&mut anim, t, &mut frame, 8, 0);
        assert_eq!(
            Some(frame),
            entry_frame(&anim, t, 8),
            "a entrada de `advance` e a celula de rebobinar tem de ser a MESMA para {}",
            t.name
        );
    }
}
