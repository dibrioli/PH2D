//! **Fatia 2 do [ADR-0141]** — o orçamento do canal Position, contra a lei que a
//! Fatia 0 escreveu depois de medir.
//!
//! A lei tem duas metades, e só uma delas é sobre velocidade:
//!
//! 1. **Custo PLANO nas âncoras.** Amostrar uma trajetória de 512 âncoras não pode
//!    custar mais do que uma de 4 — as duas buscas do caminho (a key no track e o
//!    segmento na tabela de arco) são binárias, e a inversa de Newton roda numa
//!    cúbica só. É esta metade que denuncia alguém trocar a busca por uma varredura,
//!    e ela é uma **RAZÃO**: imune à deriva da máquina e ao perfil de compilação.
//! 2. **≤ 0,2 % de um frame de 60 Hz a 100 entidades** — a metade de relógio de
//!    parede, que só significa alguma coisa em `--release` e por isso é `#[ignore]`,
//!    exatamente como o [`apply_perf`] que já mora nesta pasta e argumenta o mesmo.
//!
//! Rodar a segunda:
//! ```text
//! cargo test -p ph2d-timeline --release --test motion_path_perf -- --ignored --nocapture
//! ```
//!
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
//! [`apply_perf`]: ./apply_perf.rs

use std::time::Instant;

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Transform, World};
use ph2d_timeline::{MotionPath, PropKind, TimelineDoc, apply_from_doc};

/// Quantas entidades a lei fala sobre.
const ENTITIES: usize = 100;

/// O orçamento, em microssegundos por frame: 0,2 % de 16,67 ms.
const BUDGET_US: f64 = 33.3;

/// Uma trajetória ondulada com `n` âncoras. A curvatura varia (uma reta esconderia
/// trabalho da inversa), e nada aqui é transcendental.
fn wavy(n: usize) -> MotionPath {
    MotionPath::new(
        (0..n)
            .map(|i| {
                let x = i as f32 * 7.0;
                let y = ((i % 4) as f32 - 1.5).abs() * 5.0;
                let at = [x, y];
                // Auto Bezier pela mesma porta que a autoria usa.
                MotionPath::auto_smooth(
                    (i > 0).then(|| {
                        [
                            (i - 1) as f32 * 7.0,
                            (((i - 1) % 4) as f32 - 1.5).abs() * 5.0,
                        ]
                    }),
                    at,
                    (i + 1 < n).then(|| {
                        [
                            (i + 1) as f32 * 7.0,
                            (((i + 1) % 4) as f32 - 1.5).abs() * 5.0,
                        ]
                    }),
                )
            })
            .collect(),
    )
}

/// `entities` objetos, cada um com um binding Position cujo caminho tem `anchors`
/// âncoras — e **uma key por âncora**, que é a forma real do canal (âncora `i` É key
/// `i`), não uma isolada de laboratório.
fn scene(entities: usize, anchors: usize) -> (World, TimelineDoc) {
    let path = wavy(anchors);
    let mut world = World::new();
    let mut doc = TimelineDoc::new();
    for _ in 0..entities {
        let e = world.spawn(Transform::default()).id().to_bits();
        for i in 0..anchors {
            let t = 4.0 * i as f64 / (anchors - 1) as f64;
            doc.insert_key(
                e,
                PropKind::Position,
                RationalTime::from_seconds(t),
                AnimValue::Float(path.arclen_at(i).unwrap() as f32),
                Interp::Linear,
            );
        }
        let b = doc.bindings().len() - 1;
        doc.bindings_mut()[b].path = Some(path.clone());
    }
    (world, doc)
}

/// Microssegundos por `apply_from_doc`.
fn per_frame_us(entities: usize, anchors: usize, frames: u32) -> f64 {
    let (mut world, mut doc) = scene(entities, anchors);
    apply_from_doc(&mut world, &mut doc, 0.0); // aquece buffers e captura os `rest`
    let start = Instant::now();
    for f in 0..frames {
        apply_from_doc(&mut world, &mut doc, f64::from(f) * 0.013);
    }
    start.elapsed().as_secs_f64() * 1e6 / f64::from(frames)
}

/// Microssegundos por [`MotionPath::project`] — o **controle positivo**: uma função
/// pública, honestamente `O(âncoras)`, medida com o mesmo relógio e na mesma
/// fixture. Sem ela, "a razão deu 1,0" seria uma afirmação sobre o cronômetro.
fn project_us(anchors: usize, reps: u32) -> f64 {
    let path = wavy(anchors);
    let probe = [13.0, 2.0];
    let mut sink = 0.0;
    let start = Instant::now();
    for _ in 0..reps {
        sink += path.project(probe).unwrap();
    }
    let us = start.elapsed().as_secs_f64() * 1e6 / f64::from(reps);
    assert!(sink.is_finite());
    us
}

/// Nanossegundos por [`MotionPath::at`] — a amostragem SOZINHA, sem o
/// `Track::sample` nem a inversa de Newton por cima.
fn at_ns(anchors: usize, reps: u32) -> f64 {
    let path = wavy(anchors);
    let total = path.length();
    let mut sink = 0.0f32;
    let start = Instant::now();
    for k in 0..reps {
        // Percorre o caminho inteiro para que a busca de segmento não caia sempre no
        // mesmo lugar — uma sonda fixa deixaria o preditor de ramo esconder a
        // varredura.
        let s = total * f64::from(k) / f64::from(reps);
        sink += path.at(s).unwrap().point[0];
    }
    let ns = start.elapsed().as_secs_f64() * 1e9 / f64::from(reps);
    assert!(sink.is_finite());
    ns
}

/// **A camada AFIADA: a busca de segmento é binária.**
///
/// ⚠️ Este gate existe porque o de ponta a ponta abaixo **não bastou**: trocar o
/// `partition_point` por uma varredura sobre 512 âncoras moveu a razão de lá para
/// apenas **1,77×** — sob a barra de 2,0 — porque `Track::sample` e a inversa de
/// Newton dominam o frame e diluem o defeito. Duas camadas, dois gates
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
///
/// Aqui a amostragem é medida sozinha e o contraste é 1024×, onde uma varredura é
/// inconfundível: `log2(8192/8) = 10` passos de busca binária a mais contra 8192
/// comparações.
#[test]
fn the_segment_lookup_is_a_binary_search_not_a_scan() {
    let (few, many) = (8usize, 8192usize);
    let small = at_ns(few, 20_000);
    let big = at_ns(many, 20_000);
    let ratio = big / small;
    println!("MEDIDO  at(): {small:.0} ns ({few} âncoras) -> {big:.0} ns ({many}) = {ratio:.2}x");
    assert!(
        ratio < 3.0,
        "amostrar um caminho de {many} âncoras custou {ratio:.2}x o de {few} \
         ({big:.0} vs {small:.0} ns): a busca de segmento virou varredura"
    );
}

/// **A rede de ponta a ponta.** Amostrar é duas buscas binárias e uma
/// inversa numa cúbica: 128× mais âncoras não podem custar 128× mais.
///
/// Uma RAZÃO, não um relógio — o `ci-test` compila em `opt-level=1` e um limite de
/// wall-clock mediria o PERFIL. E com o controle ao lado, porque uma razão de 1,0
/// sobre um cronômetro cego também dá 1,0.
#[test]
fn the_cost_of_sampling_a_path_is_flat_in_its_anchors() {
    let (few, many) = (4usize, 512usize);
    let small = per_frame_us(ENTITIES, few, 60);
    let big = per_frame_us(ENTITIES, many, 60);
    let ratio = big / small;

    // O controle: a MESMA fixture, o MESMO relógio, numa função que de fato varre
    // todo segmento. Se ele não enxergar o crescimento, esta suíte não enxerga nada.
    let control = project_us(many, 200) / project_us(few, 200);

    println!(
        "MEDIDO  apply: {small:.2} us ({few} âncoras) -> {big:.2} us ({many}) = {ratio:.2}x  |  \
         controle project: {control:.1}x"
    );
    assert!(
        control > 8.0,
        "o controle cresceu só {control:.1}x para 128x o trabalho — o cronômetro desta \
         fixture não resolve O(n), então a razão de 1x abaixo não prova nada"
    );
    assert!(
        ratio < 2.0,
        "amostrar {many} âncoras custou {ratio:.2}x o de {few} ({big:.2} vs {small:.2} us/frame): \
         alguma busca virou varredura"
    );
    // ⚠️ Barra FROUXA de propósito, e medida: com uma varredura no lugar da busca de
    // segmento este número vai a 1,77× e PASSA. Quem pega essa regressão é o gate
    // afiado acima; este aqui guarda o resto do passe (o track, a inversa, o laço de
    // bindings), onde uma regressão canvas-proporcional apareceria.
}

/// **A metade de relógio de parede**, em release: a lei do ADR, com o número impresso
/// ao lado dela.
#[test]
#[ignore = "wall-clock: só significa alguma coisa em --release (veja o cabeçalho)"]
fn a_hundred_objects_on_paths_fit_in_a_fifth_of_a_percent_of_a_frame() {
    let us = per_frame_us(ENTITIES, 64, 400);
    println!(
        "MEDIDO  {ENTITIES} entidades em modo Path: {us:.2} us/frame \
         ({:.3} % de um frame de 60 Hz; orçamento {BUDGET_US} us)",
        us / 166.67
    );
    assert!(
        us < BUDGET_US,
        "{us:.2} us/frame passa o orçamento de {BUDGET_US} us (ADR-0141, Fatia 0)"
    );
}
