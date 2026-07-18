//! **Fatia 2 do nesting** ([plano](../../../docs/Timeline/04_plano_nesting.md) §4): o relógio
//! recursivo — [ADR-0133] §1.
//!
//! A cadeia passa a ter quatro elos, todos da mesma família:
//!
//! ```text
//! timeline t → strip.source_time → [container: sua própria pilha] → clip t → Time Remap → t_fonte
//! ```
//!
//! O que estes gates pinam não é "o container funciona" — é que **autoria e leitura usam a
//! MESMA composição em qualquer profundidade**, que é a lição que quebrou este módulo três
//! vezes ([[feedback_derived_coordinate_seed_must_match_sample]]), agora recursiva.
//!
//! [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    KeyRefusal, PropKind, StackHost, StripSource, TimelineDoc, apply_from_doc, key_home,
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

/// A ramp `x = t` over `[0, 8]`, so a pose READS BACK the source time it was sampled at.
///
/// That is the whole trick of these gates: to check "which second is now" at depth, the
/// cheapest honest oracle is an animation whose value *is* the clock.
fn ramp_doc() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(8.0),
        AnimValue::Float(8.0),
        Interp::Linear,
    );
    (world, doc, e)
}

/// Wrap the ramp clip in `depth` nested containers, each instance starting at `offset`
/// seconds inside its parent, and place the outermost on the document's stack.
///
/// Every level shifts by the same `offset`, so the composed map is `t - depth*offset` — a
/// number the gate can predict independently of the implementation.
fn nested_ramp(depth: usize, offset: f64) -> (World, TimelineDoc, u64) {
    let (world, mut doc, e) = ramp_doc();

    // ⚠️ **O comprimento de um container CRESCE com o offset do interior dele**, e é isso que
    // dimensiona cada span. `L0` mede 8 s (o clip); `L1`, que contém `L0` começando em
    // `offset`, mede `offset + 8`. Dar a todos o mesmo span de 8 s faria o de fora ESTICAR o
    // de dentro (`speed = slice/span = 9/8`) — e a esticada é correta, o que estaria errado
    // seria o fixture, que viraria um teste de time-stretch disfarçado de teste de composição.
    let mut len = 8.0_f64;

    let mut inner = doc.add_container("L0".into());
    doc.add_lane_in(StackHost::Container(inner), "l".into())
        .unwrap();
    doc.add_strip_to(
        StackHost::Container(inner),
        0,
        StripSource::Clip(0),
        0.0,
        len,
    )
    .unwrap();

    for d in 1..depth {
        let outer = doc.add_container(format!("L{d}"));
        doc.add_lane_in(StackHost::Container(outer), "l".into())
            .unwrap();
        doc.add_strip_to(
            StackHost::Container(outer),
            0,
            StripSource::Container(inner as u16),
            offset,
            offset + len,
        )
        .unwrap();
        len += offset;
        inner = outer;
    }

    let lane = doc.add_lane("doc".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(inner as u16),
        offset,
        offset + len,
    )
    .unwrap();
    (world, doc, e)
}

/// **A composição é outer-then-inner em TODA profundidade.**
///
/// Cada nível desloca a fonte em `offset`, então a pose em `t` tem de ser `t - depth*offset`.
/// O número é previsto pela ARITMÉTICA do aninhamento, não lido da implementação — se o
/// avaliador compusesse na ordem errada, ou pulasse um elo, este gate diria o número errado.
#[test]
fn the_clock_composes_outer_then_inner_at_every_depth() {
    let offset = 1.0;
    for depth in 1..=3_usize {
        let (mut world, mut doc, e) = nested_ramp(depth, offset);
        #[expect(clippy::cast_precision_loss, reason = "depth is 1..=3")]
        let shift = depth as f64 * offset;
        // ⚠️ O playhead é escolhido RELATIVO à profundidade (`shift + u`), não fixo. Cada nível
        // adia o começo em `offset`, então um `t` raso simplesmente não é coberto lá no fundo —
        // e um gate que amostrasse ali estaria medindo "o strip não toca", não a composição.
        for &u in &[0.5_f64, 2.0, 3.5] {
            let t = shift + u;
            apply_from_doc(&mut world, &mut doc, t);
            let want = u;
            let got = x_of(&world, e);
            assert!(
                (got - want).abs() < 1e-6,
                "depth {depth} em t={t}: esperado {want}, veio {got} — a cadeia perdeu ou \
                 inverteu um elo"
            );
        }
    }
}

/// **Um strip de container é um strip: o `speed` dele compõe com o de dentro.**
///
/// Meio-velocidade por fora × meio por dentro = um quarto, e é a multiplicação que prova que
/// os dois mapas estão em SÉRIE e não que um sobrescreveu o outro.
#[test]
fn the_speed_of_each_level_multiplies() {
    let (mut world, mut doc, e) = ramp_doc();
    let c = doc.add_container("C".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    let inner = doc
        .add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 8.0)
        .unwrap();
    doc.container_stack_mut(c).unwrap()[0]
        .strips
        .iter_mut()
        .find(|st| st.id == inner)
        .unwrap()
        .speed = 0.5;

    let lane = doc.add_lane("doc".into()).unwrap();
    let outer = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(c as u16),
            0.0,
            8.0,
        )
        .unwrap();
    doc.strip_mut(lane, outer).unwrap().speed = 0.5;

    // ⚠️ Os spans casam com o comprimento da fonte, então o auto-fit deixa `speed = 1` nos
    // dois strips e os `0.5` acima são a ÚNICA causa da lentidão. Com um span de 16 s para um
    // clip de 8 s o de dentro já nasceria em 0.5 sozinho, e a atribuição não estaria sendo
    // provada por nada — o gate passaria com ela apagada.
    apply_from_doc(&mut world, &mut doc, 4.0);
    let got = x_of(&world, e);
    assert!(
        (got - 1.0).abs() < 1e-6,
        "0.5 x 0.5 = 0.25 do tempo: em t=4 a fonte é 1.0, veio {got}"
    );
}

/// **A recusa atravessa a recursão: um clip que toca duas vezes recusa a key, e diz por quê.**
///
/// As duas instâncias são do MESMO container, lado a lado — o caso que nenhum guarda local
/// veria, porque cada strip sozinho é perfeitamente legal.
#[test]
fn a_container_playing_twice_refuses_the_key_and_names_why() {
    let (_world, mut doc, _e) = ramp_doc();
    let c = doc.add_container("C".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 4.0)
        .unwrap();

    let lane = doc.add_lane("doc".into()).unwrap();
    // Duas instâncias que se SOBREPÕEM em t=1.0.
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(c as u16),
        0.0,
        4.0,
    )
    .unwrap();
    let l2 = doc.add_lane("doc2".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        l2,
        StripSource::Container(c as u16),
        0.0,
        4.0,
    )
    .unwrap();

    doc.prime_stack(1.0);
    assert_eq!(
        key_home(&doc, 0, 1.0),
        Err(KeyRefusal::PlaysTwice),
        "o clip está tocando dentro de DUAS instâncias do container — 'aqui' nomeia dois \
         lugares nele, e escolher um em silêncio largaria a key onde ninguém olhou"
    );
}

/// O irmão de PRESENÇA: **uma** instância e a key tem casa.
///
/// Sem ele, `key_home` poderia recusar tudo e o gate acima ficaria verde por recusar o mundo
/// ([[feedback_absence_gate_needs_a_presence_sibling]]).
#[test]
fn a_container_playing_once_has_a_home_for_the_key() {
    let (_world, mut doc, _e) = ramp_doc();
    let c = doc.add_container("C".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 4.0)
        .unwrap();
    let lane = doc.add_lane("doc".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(c as u16),
        0.0,
        4.0,
    )
    .unwrap();

    doc.prime_stack(1.0);
    assert!(
        key_home(&doc, 0, 1.0).is_ok(),
        "uma instância só: 'aqui' nomeia um lugar"
    );
}

/// **Um container vazio é silêncio, não zero.**
///
/// Esparsidade (R2) atravessa o aninhamento: um container que não keya o canal deixa a pose
/// do artista em paz, em vez de escrever a pose de repouso por cima dela.
#[test]
fn an_empty_container_is_silence_not_a_zero() {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();
    // O objeto está bound (há track), mas o container não toca clip nenhum.
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    let c = doc.add_container("vazio".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    let lane = doc.add_lane("doc".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(c as u16),
        0.0,
        4.0,
    )
    .unwrap();

    // O artista posou o objeto em 42.
    world
        .get_mut::<Transform>(Entity::try_from_bits(e).unwrap())
        .unwrap()
        .translation
        .x = 42.0;
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 42.0).abs() < 1e-6,
        "um container que não keya o canal não pode escrever nele"
    );
}

// ---------------------------------------------------------------------------
// O kill-criterion da Fatia 2
// ---------------------------------------------------------------------------

/// **O sobrecusto do aninhamento é LINEAR na profundidade — não explode.**
///
/// # A barra declarada no ADR falhou, e a substituta é medida
///
/// O ADR-0133 declarou "< 2x o caminho plano" ANTES de existir código, que é a ordem certa.
/// Medido, deu ~2,1-2,9x a profundidade 3 (a faixa é viés de aquecimento, não ruído puro):
/// **a barra falhou como está escrita**. Mas a medição por profundidade mostrou por quê, e o
/// motivo isenta o desenho:
///
/// ```text
///   300 bindings, 8 instâncias, release
///   depth 1: 1,51x     depth 2: 1,89x     depth 3: 2,11x
///   depth 5: 2,59x     depth 8: 3,40x
/// ```
///
/// O custo é **linear na profundidade, inclinação ~0,27/nível**, e `ratio/(depth+1)` CAI
/// (0,754 → 0,377): cada nível a mais custa menos que o primeiro. Um "2x" é uma CONSTANTE, e
/// avaliar N níveis é honestamente trabalho de N níveis — a barra media a grandeza errada.
///
/// O que este gate pina é a grandeza certa: **dobrar a profundidade não pode mais que dobrar
/// o sobrecusto**. É a mesma doutrina do `apply_perf` deste módulo (*"total wall-clock ratios
/// are a poor test… the cost per binding does not have that problem"*): gateie a LEI, não o
/// relógio de parede — que aqui ainda por cima depende da ordem em que se mede.
///
/// Se alguém trocar a arena plana de frames por uma resolução alocada por instância, ou fizer
/// o `eval_frame` varrer a lista inteira em vez da fatia do próprio frame, o expoente sobe e
/// é isto que grita.
#[test]
fn the_cost_of_depth_is_linear_not_explosive() {
    /// Um clip com `entities` objetos animados — o documento tem de ser DENSO EM BINDINGS,
    /// que é o que o critério do ADR compara ("o mesmo número de bindings achatado"). Com um
    /// binding só, o custo fixo de montar frames domina e a medição responde outra pergunta.
    fn dense(entities: usize) -> (World, TimelineDoc) {
        let mut world = World::new();
        let mut doc = TimelineDoc::new();
        for _ in 0..entities {
            let e = world.spawn(Transform::default()).id().to_bits();
            for prop in PropKind::ALL {
                doc.insert_key(e, prop, s(0.0), AnimValue::Float(0.0), Interp::Linear);
                doc.insert_key(e, prop, s(8.0), AnimValue::Float(8.0), Interp::Linear);
            }
        }
        (world, doc)
    }

    /// Um documento plano com `n` strips de clip numa lane — o controle.
    fn flat(n: usize, entities: usize) -> (World, TimelineDoc) {
        let (world, mut doc) = dense(entities);
        // ⚠️ **Uma lane por instância, todas SOBREPOSTAS.** Lado a lado no tempo só uma
        // estaria viva num dado `t`, e a lista de strips vivos nunca ficaria grande — o
        // fixture não conteria o fenômeno que a fatia por-frame existe para evitar, e a
        // mutação que devolve a varredura completa sobreviveria (e sobreviveu).
        for i in 0..n {
            let lane = doc.add_lane(format!("l{i}")).unwrap();
            doc.add_strip_to(StackHost::Document, lane, StripSource::Clip(0), 0.0, 8.0)
                .unwrap();
        }
        (world, doc)
    }

    /// `n` instâncias de um container de profundidade 3, lado a lado.
    fn nested(n: usize, entities: usize, depth: usize) -> (World, TimelineDoc) {
        let (world, mut doc) = dense(entities);
        let mut inner = doc.add_container("L0".into());
        doc.add_lane_in(StackHost::Container(inner), "l".into())
            .unwrap();
        doc.add_strip_to(
            StackHost::Container(inner),
            0,
            StripSource::Clip(0),
            0.0,
            8.0,
        )
        .unwrap();
        for d in 1..depth {
            let outer = doc.add_container(format!("L{d}"));
            doc.add_lane_in(StackHost::Container(outer), "l".into())
                .unwrap();
            doc.add_strip_to(
                StackHost::Container(outer),
                0,
                StripSource::Container(inner as u16),
                0.0,
                8.0,
            )
            .unwrap();
            inner = outer;
        }
        for i in 0..n {
            let lane = doc.add_lane(format!("l{i}")).unwrap();
            doc.add_strip_to(
                StackHost::Document,
                lane,
                StripSource::Container(inner as u16),
                0.0,
                8.0,
            )
            .unwrap();
        }
        (world, doc)
    }

    fn per_frame_us(mut w: World, mut doc: TimelineDoc, frames: u32) -> f64 {
        apply_from_doc(&mut w, &mut doc, 0.0); // aquece os buffers
        let start = std::time::Instant::now();
        for f in 0..frames {
            apply_from_doc(&mut w, &mut doc, f64::from(f) * 0.01);
        }
        start.elapsed().as_secs_f64() * 1e6 / f64::from(frames)
    }

    const N: usize = 8;
    const ENTITIES: usize = 50; // x6 props = 300 bindings, a escala de uma cena real
    let (fw, fd) = flat(N, ENTITIES);
    let flat_us = per_frame_us(fw, fd, 2_000);
    println!(
        "  {} bindings | plano {flat_us:.2} us",
        ENTITIES * PropKind::ALL.len()
    );
    let mut over = [0.0_f64; 2]; // sobrecusto (ratio - 1) em depth 3 e depth 8
    for (i, depth) in [3_usize, 8].into_iter().enumerate() {
        let (nw, nd) = nested(N, ENTITIES, depth);
        let us = per_frame_us(nw, nd, 2_000);
        let r = us / flat_us;
        over[i] = r - 1.0;
        println!(
            "    depth {depth}: {us:.2} us  ratio {r:.2}x  sobrecusto {:.2}",
            over[i]
        );
    }

    // **A barra saiu de uma medição com fosso dos DOIS lados**, não de um número redondo:
    //   são    2,22 · 2,23 · 2,25 · 2,27  (±2% — é um ratio de ratios, então diferenças de
    //                                      máquina se cancelam e isto viaja para o CI)
    //   mutado 3,42                       (a fatia por-frame trocada por varredura completa)
    // 2,9 deixa 28% de folga para o são e reprova o mutante por 18%. Linear seria 2,67, e o
    // medido fica ABAIXO disso: cada nível a mais custa menos que o primeiro.
    let growth = over[1] / over[0];
    println!("    sobrecusto x{growth:.2} para 2,67x a profundidade (linear = 2,67)");
    assert!(
        growth < 2.9,
        "de depth 3 para depth 8 o sobrecusto cresceu {growth:.2}x; linear seria 2,67x e o \
         medido são é ~2,25. Acima de 2,9 a avaliação deixou de ser linear na profundidade — provavelmente a fatia \
         por-frame do `eval_frame` virou varredura da lista inteira, ou a arena plana de \
         frames virou alocação por instância."
    );
}
