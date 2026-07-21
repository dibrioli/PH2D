//! **Fatia 3d do nesting + a reescrita da ENTRADA** ([ADR-0133] §5; Enio, 2026-07-20): o mapa
//! da instância — a régua do container tem de LER e ESCREVER no mesmo relógio, e o mapa vem
//! do STRIP POR ONDE SE ENTROU ([`entry_map`]), nunca de "o único strip tocando agora".
//!
//! A 1ª versão (scratch) recusava nos vãos entre instâncias (toca ZERO vezes ali) e com o
//! container instanciado 2× (toca duas) — e as duas ambiguidades eram fabricadas: entrar é
//! clicar num strip, então a instância que o animador quer dizer está nomeada pela própria
//! caminhada. Estes gates pinam o mapa novo com o MESMO oráculo dos antigos: a rampa `x = t`
//! faz a pose LER DE VOLTA o segundo-fonte amostrado, então *"buscar onde o mapa manda"* é
//! conferível olhando o que aparece.
//!
//! [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    EnterStep, StackHost, StripLoop, StripSource, TimelineDoc, apply_from_doc, entry_map,
    entry_reach,
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
/// `start`. Devolve o caminho de entrada dessa instância.
fn one_instance(start: f64) -> (World, TimelineDoc, u64, Vec<EnterStep>) {
    let (world, mut doc, e) = ramp_doc();
    let c = doc.add_container("C".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 8.0)
        .unwrap();
    let lane = doc.add_lane("doc".into()).unwrap();
    let strip = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(c).unwrap()),
            start,
            start + 8.0,
        )
        .unwrap();
    let path = vec![EnterStep {
        container: c,
        lane,
        strip: Some(strip),
    }];
    (world, doc, e, path)
}

/// **O mapa manda buscar onde o interior de fato aparece.**
///
/// O gate NÃO compara com uma fórmula: ele pede ao mapa o tempo de timeline para o segundo
/// `u` do interior, **busca lá**, e confere que o que está na tela é o segundo `u`. Era
/// exatamente isso que a régua fazia errado — ela mandava buscar `u` cru.
#[test]
fn seeking_where_the_map_says_shows_the_interior_second_you_asked_for() {
    let start = 4.0;
    let (mut world, mut doc, e, path) = one_instance(start);
    let map = entry_map(&doc, &path).expect("uma instância sem wrap tem mapa");
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

/// **O mapa vale em TODO instante do playhead — inclusive onde a instância não toca.**
///
/// É a metade que a versão de scratch não tinha (Enio, 2026-07-20: *"não consigo
/// controlar/arrastar a playhead"*): o mapa antigo era função do instante primado e sumia
/// em todo vão — a régua registrava hit num frame e não no seguinte. O mapa de entrada é
/// função do documento e da caminhada; nenhum `prime` participa, então não HÁ instante que
/// o mude — o que este gate afirma exercitando a MESMA pergunta em pontos dentro, fora e
/// depois da janela.
#[test]
fn the_entry_map_does_not_depend_on_where_the_playhead_stands() {
    let start = 4.0;
    let (mut world, mut doc, _e, path) = one_instance(start);
    let reference = entry_map(&doc, &path).expect("mapa");
    for &t in &[0.0_f64, 2.0, 5.0, 30.0] {
        apply_from_doc(&mut world, &mut doc, t); // move o mundo/scratch como o app faria
        assert_eq!(
            entry_map(&doc, &path),
            Some(reference),
            "o mapa mudou com o playhead em t={t} — voltou a depender do instante primado"
        );
    }
}

/// **Duas instâncias vivas: o mapa é o da que se ENTROU.** A versão de scratch recusava
/// (PlaysTwice); a ambiguidade era fabricada — o clique de entrada nomeia a instância.
#[test]
fn with_two_instances_the_map_is_the_entered_ones() {
    let (_world, mut doc, _e, mut path) = one_instance(0.0);
    let c = path[0].container;
    let second = doc
        .add_strip_to(
            StackHost::Document,
            path[0].lane,
            StripSource::Container(u16::try_from(c).unwrap()),
            20.0,
            28.0,
        )
        .expect("uma segunda instância");
    let first_map = entry_map(&doc, &path).expect("entrada pela 1a");
    assert!((first_map.t0 - 0.0).abs() < 1e-9 && (first_map.t1 - 8.0).abs() < 1e-9);
    path[0].strip = Some(second);
    let second_map = entry_map(&doc, &path).expect("entrada pela 2a");
    assert!(
        (second_map.t0 - 20.0).abs() < 1e-9 && (second_map.t1 - 28.0).abs() < 1e-9,
        "entrando pela 2a instância o mapa tem de ser a janela DELA, veio {second_map:?}"
    );
}

/// **A janela é onde a instância toca** — e é ela que dá o clamp.
///
/// Arrastar para fora do que a instância mostra não tem resposta honesta: o interior
/// simplesmente não está na tela ali. O mapa gruda nas bordas em vez de extrapolar para um
/// segundo em que o container não toca.
#[test]
fn the_window_is_where_the_instance_plays_and_the_map_clamps_to_it() {
    let start = 4.0;
    let (_world, doc, _e, path) = one_instance(start);
    let map = entry_map(&doc, &path).expect("mapa");
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
/// que a caminhada compõe em vez de parar no primeiro elo.
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
    let inner = doc
        .add_strip_to(
            StackHost::Container(l1),
            0,
            StripSource::Container(u16::try_from(l0).unwrap()),
            offset,
            offset + 8.0,
        )
        .unwrap();
    let lane = doc.add_lane("doc".into()).unwrap();
    let outer = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(l1).unwrap()),
            offset,
            offset + offset + 8.0,
        )
        .unwrap();

    let path = vec![
        EnterStep {
            container: l1,
            lane,
            strip: Some(outer),
        },
        EnterStep {
            container: l0,
            lane: 0,
            strip: Some(inner),
        },
    ];
    let map = entry_map(&doc, &path).expect("mapa do interior mais fundo");
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

/// **Uma caminhada OBSOLETA não tem mapa** — o strip de entrada foi deletado (ou já não
/// toca aquele container). Devolver a janela de OUTRO strip seria a régua buscando num
/// lugar que o animador nunca nomeou.
#[test]
fn a_stale_walk_has_no_map() {
    let (_world, mut doc, _e, path) = one_instance(0.0);
    // Controle positivo primeiro: a caminhada viva TEM mapa.
    assert!(entry_map(&doc, &path).is_some(), "controle positivo");

    // RETARGET: o strip ainda existe, mas já não toca aquele container (virou clip). A
    // janela dele descreveria outra coisa — e é esta metade que torna a checagem de fonte
    // do `entry_strip` load-bearing (deletar o strip já falha no lookup por id).
    doc.strip_in_mut(StackHost::Document, path[0].lane, path[0].strip.unwrap())
        .unwrap()
        .source = StripSource::Clip(0);
    assert!(
        entry_map(&doc, &path).is_none(),
        "um strip retargetado não é mais a instância que se entrou"
    );
    doc.strip_in_mut(StackHost::Document, path[0].lane, path[0].strip.unwrap())
        .unwrap()
        .source = StripSource::Container(u16::try_from(path[0].container).unwrap());
    assert!(entry_map(&doc, &path).is_some(), "restaurado, volta");

    doc.remove_strip_in(StackHost::Document, path[0].lane, path[0].strip.unwrap());
    assert!(
        entry_map(&doc, &path).is_none(),
        "sem o strip de entrada não há janela para oferecer"
    );
}

/// **Uma instância que DÁ A VOLTA não tem inverso** — um segundo do interior acontece em
/// vários segundos da timeline. O gate move só o `loop_mode` e o span; tudo mais é o
/// fixture que TEM mapa logo acima, então o que ele mede é a volta e nada além dela.
#[test]
fn a_wrapping_instance_has_no_map() {
    let (_world, mut doc, _e, path) = one_instance(0.0);
    {
        let lane = &mut doc.stack_mut()[0];
        let st = &mut lane.strips[0];
        st.t_end = st.t_start + 20.0; // mais longo que uma passada
        st.speed = 1.0;
        st.src_out = 8.0;
        st.loop_mode = StripLoop::Loop;
    }
    assert!(
        entry_map(&doc, &path).is_none(),
        "sob wrap o segundo 1 do interior está em t=1, 9 e 17 — não há UM lugar para buscar"
    );
    // E o controle POSITIVO: o MESMO strip, sem dar a volta, tem mapa. Sem isto o gate
    // ficaria verde com `entry_map` devolvendo `None` para tudo.
    doc.stack_mut()[0].strips[0].t_end = 8.0;
    assert!(
        entry_map(&doc, &path).is_some(),
        "sem dar a volta o mesmo strip tem de ter mapa — senão o gate acima não mede o wrap"
    );
}

/// **O alcance da instância inclui os leads** — é a janela que o loop do transporte deve
/// abraçar ao entrar ([`entry_reach`]): só a janela que SE MOVE cortaria do ciclo as
/// próprias fades que o artista está ajustando (o bug do `stack_end_seconds`, um nível
/// abaixo).
#[test]
fn the_entry_reach_wraps_the_instances_leads() {
    let (_world, mut doc, _e, path) = one_instance(4.0);
    assert_eq!(
        entry_reach(&doc, &path),
        Some((4.0, 12.0)),
        "sem leads o alcance é a própria janela"
    );
    {
        let st = doc
            .strip_in_mut(StackHost::Document, path[0].lane, path[0].strip.unwrap())
            .unwrap();
        st.lead_in = 1.0;
        st.lead_out = 0.5;
    }
    assert_eq!(
        entry_reach(&doc, &path),
        Some((3.0, 12.5)),
        "com leads o alcance estende por eles dos dois lados"
    );
}
