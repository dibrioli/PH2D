//! **Fatia 3d do nesting** ([plano](../../../docs/Timeline/04_plano_nesting.md) §5): o mapa da
//! instância — a régua do container tem de LER e ESCREVER no mesmo relógio ([ADR-0133] §5).
//!
//! A 3c consertou a leitura (a marca do playhead) e deixou a escrita: arrastar a régua lá
//! dentro mandava um segundo LOCAL para o playhead da TIMELINE. Com a instância começando em
//! 4 s, arrastar até o segundo 1 do interior buscava o segundo 1 da cena — três segundos fora
//! do container, e sem nada na tela dizendo que você saiu.
//!
//! O oráculo destes gates é sempre a POSE, nunca um número interno: a rampa `x = t` faz o
//! objeto ler de volta o segundo em que foi amostrado, então *"buscar onde o mapa manda"* pode
//! ser conferido **olhando o que aparece**.
//!
//! [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    StackHost, StripLoop, StripSource, TimelineDoc, apply_from_doc, container_map,
    container_playhead,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn x_of(world: &World, e: u64) -> f64 {
    f64::from(
        world
            .get::<Transform>(Entity::try_from_bits(e).unwrap())
            .unwrap()
            .translation
            .x,
    )
}

/// A rampa `x = t` sobre `[0, 8]` — a pose LÊ DE VOLTA o tempo-fonte amostrado.
fn ramp_doc() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0), (8.0, 8.0)] {
        doc.insert_key(
            e,
            ph2d_timeline::PropKind::TranslationX,
            s(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    (world, doc, e)
}

/// Um container "C" com a rampa dentro, instanciado UMA vez no documento começando em
/// `start`. Devolve o índice do container.
fn one_instance(start: f64) -> (World, TimelineDoc, u64, usize) {
    let (world, mut doc, e) = ramp_doc();
    let c = doc.add_container("C".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 8.0)
        .unwrap();
    let lane = doc.add_lane("doc".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(u16::try_from(c).unwrap()),
        start,
        start + 8.0,
    )
    .unwrap();
    (world, doc, e, c)
}

/// **O mapa manda buscar onde o interior de fato aparece.**
///
/// O gate NÃO compara com uma fórmula: ele pede ao mapa o tempo de timeline para o segundo
/// `u` do interior, **busca lá**, e confere que o que está na tela é o segundo `u`. Era
/// exatamente isso que a régua fazia errado — ela mandava buscar `u` cru.
#[test]
fn seeking_where_the_map_says_shows_the_interior_second_you_asked_for() {
    let start = 4.0;
    let (mut world, mut doc, e, c) = one_instance(start);
    apply_from_doc(&mut world, &mut doc, start + 1.0); // prima o scratch dentro da instância
    let map = container_map(&doc, c).expect("uma instância única e sem wrap tem mapa");
    for &u in &[0.0_f64, 1.5, 4.0, 7.5] {
        let t = map.host_time(u);
        apply_from_doc(&mut world, &mut doc, t);
        assert!(
            (x_of(&world, e) - u).abs() < 1e-6,
            "pedir o segundo {u} do interior levou a t={t}, e lá a tela mostra {} — \
             o mapa mandou buscar no lugar errado",
            x_of(&world, e)
        );
        // ⚠️ E o número CRU (o bug da 3c) tem de ser o lugar errado, senão a fixture está
        // num ponto degenerado onde os dois relógios coincidem e o gate não prova nada.
        assert!(
            (t - u).abs() > 1e-9 || u == 0.0 && start == 0.0,
            "com a instância em {start} s o segundo local {u} não pode ser o segundo {t} da \
             timeline — a fixture perdeu o deslocamento que ela existe para conter"
        );
    }
}

/// **As duas portas concordam**: o mapa é o inverso exato do relógio que a régua DESENHA.
///
/// `container_playhead` responde "que segundo o interior está lendo"; `map.local_time` tem de
/// dar o mesmo número, e `host_time` tem de desfazê-lo. Duas portas para a mesma pergunta
/// divergem ([[feedback_two_doors_to_the_same_question_diverge]]) — aqui a divergência seria
/// a marca do playhead num segundo e o arrasto noutro.
#[test]
fn the_map_is_the_inverse_of_the_playhead_it_draws() {
    let start = 4.0;
    let (mut world, mut doc, _e, c) = one_instance(start);
    for &t in &[4.0_f64, 5.25, 8.0, 11.9] {
        apply_from_doc(&mut world, &mut doc, t);
        let map = container_map(&doc, c).expect("mapa");
        let drawn = container_playhead(&doc, c, t).expect("tocando");
        assert!(
            (map.local_time(t) - drawn).abs() < 1e-9,
            "em t={t} a régua desenha {drawn} e o mapa lê {} — as portas divergiram",
            map.local_time(t)
        );
        assert!(
            (map.host_time(drawn) - t).abs() < 1e-9,
            "ida e volta não fecha em t={t}"
        );
    }
}

/// **A janela é onde a instância toca** — e é ela que dá o clamp.
///
/// Arrastar para fora do que a instância mostra não tem resposta honesta: o interior
/// simplesmente não está na tela ali. O mapa gruda nas bordas em vez de extrapolar para um
/// segundo em que o container não toca.
#[test]
fn the_window_is_where_the_instance_plays_and_the_map_clamps_to_it() {
    let start = 4.0;
    let (mut world, mut doc, _e, c) = one_instance(start);
    apply_from_doc(&mut world, &mut doc, start + 1.0);
    let map = container_map(&doc, c).expect("mapa");
    assert!((map.t0 - start).abs() < 1e-9 && (map.t1 - (start + 8.0)).abs() < 1e-9);
    assert!((map.u0 - 0.0).abs() < 1e-9 && (map.u1 - 8.0).abs() < 1e-9);
    assert!(
        (map.host_time(-5.0) - map.t0).abs() < 1e-9,
        "clampa embaixo"
    );
    assert!(
        (map.host_time(99.0) - map.t1).abs() < 1e-9,
        "clampa em cima"
    );
    assert!(
        (map.local_time(0.0) - map.u0).abs() < 1e-9,
        "o inverso também"
    );
}

/// **A composição vale em profundidade**: o mapa do interior mais fundo sai na timeline.
///
/// Dois níveis, cada um deslocando 1 s: pedir o segundo `u` do container mais interno tem de
/// levar a um `t` em que a tela mostra `u`. É o mesmo oráculo do gate raso — e é o que prova
/// que a caminhada para fora compõe em vez de parar no primeiro elo.
#[test]
fn the_map_composes_out_to_the_timeline_at_depth() {
    let offset = 1.0;
    let (mut world, mut doc, e) = ramp_doc();
    // L0 contém o clip; L1 contém L0 deslocado de `offset`; o documento contém L1 deslocado
    // de `offset`. ⚠️ L1 mede `offset + 8` — o comprimento CRESCE com o deslocamento do
    // interior, e dar-lhe 8 s faria o de fora esticar o de dentro.
    let l0 = doc.add_container("L0".into());
    doc.add_lane_in(StackHost::Container(l0), "l".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(l0), 0, StripSource::Clip(0), 0.0, 8.0)
        .unwrap();
    let l1 = doc.add_container("L1".into());
    doc.add_lane_in(StackHost::Container(l1), "l".into())
        .unwrap();
    doc.add_strip_to(
        StackHost::Container(l1),
        0,
        StripSource::Container(u16::try_from(l0).unwrap()),
        offset,
        offset + 8.0,
    )
    .unwrap();
    let lane = doc.add_lane("doc".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(u16::try_from(l1).unwrap()),
        offset,
        offset + offset + 8.0,
    )
    .unwrap();

    apply_from_doc(&mut world, &mut doc, 2.0 * offset + 1.0);
    let map = container_map(&doc, l0).expect("mapa do interior mais fundo");
    assert!(
        (map.t0 - 2.0 * offset).abs() < 1e-6,
        "dois níveis de 1 s deviam abrir o interior em 2 s da timeline, veio {}",
        map.t0
    );
    for &u in &[0.5_f64, 3.0, 6.0] {
        let t = map.host_time(u);
        apply_from_doc(&mut world, &mut doc, t);
        assert!(
            (x_of(&world, e) - u).abs() < 1e-6,
            "em profundidade 2, pedir o segundo {u} levou a t={t} e a tela mostra {}",
            x_of(&world, e)
        );
    }
}

/// **Tocando duas vezes não há mapa** — "aqui" nomeia dois lugares.
///
/// É a MESMA recusa que `container_playhead` faz para desenhar, feita do lado da escrita.
/// Escolher uma das duas em silêncio é a classe de palpite que este módulo recusa em todo
/// lugar; a régua simplesmente não arrasta.
#[test]
fn a_container_playing_twice_has_no_map() {
    let (mut world, mut doc, _e, c) = one_instance(0.0);
    doc.add_strip_to(
        StackHost::Document,
        0,
        StripSource::Container(u16::try_from(c).unwrap()),
        0.0,
        8.0,
    )
    .expect("uma segunda instância, sobreposta");
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        container_map(&doc, c).is_none(),
        "duas instâncias vivas no mesmo instante não podem oferecer UM mapa"
    );
}

/// **Uma instância que DÁ A VOLTA não tem inverso** — um segundo do interior acontece em
/// vários segundos da timeline.
///
/// O gate move só o `loop_mode` e o span; tudo mais é o fixture que TEM mapa logo acima, então
/// o que ele mede é a volta e nada além dela.
#[test]
fn a_wrapping_instance_has_no_map() {
    let (mut world, mut doc, _e, c) = one_instance(0.0);
    {
        let lane = &mut doc.stack_mut()[0];
        let st = &mut lane.strips[0];
        st.t_end = st.t_start + 20.0; // mais longo que uma passada
        st.speed = 1.0;
        st.src_out = 8.0;
        st.loop_mode = StripLoop::Loop;
    }
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        container_map(&doc, c).is_none(),
        "sob wrap o segundo 1 do interior está em t=1, 9 e 17 — não há UM lugar para buscar"
    );
    // E o controle POSITIVO: o MESMO strip, sem dar a volta, tem mapa. Sem isto o gate
    // ficaria verde com `container_map` devolvendo `None` para tudo.
    doc.stack_mut()[0].strips[0].t_end = 8.0;
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        container_map(&doc, c).is_some(),
        "sem dar a volta o mesmo strip tem de ter mapa — senão o gate acima não mede o wrap"
    );
}
