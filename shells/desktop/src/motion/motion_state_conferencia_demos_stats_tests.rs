//! Gates da cena `=43` — **as estatísticas**.
//!
//! ⚠️ Esta cena julga-se PARADA (nada nela depende do relógio), então os gates
//! cozem num instante só. O que eles medem é o que o olho vai medir: **ALTURAS**
//! — e por isso as lanes de uma banda partilham a base, e o gate compara Y CRU
//! de propósito **dentro** de uma banda e nunca entre bandas.

use super::super::conferencia_demos_stats as scene;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::NodeId;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    reg
}

/// Os índices das lanes na ordem em que a cena as monta — o mesmo `LANES` do
/// produto, nomeado aqui para os gates lerem por NOME em vez de por número.
mod lane {
    pub const B1_FIELD: usize = 0;
    pub const B1_MEAN: usize = 1;
    pub const B1_MEDIAN: usize = 2;
    pub const B2_MEAN: usize = 4;
    pub const B2_MEAN_MASKED: usize = 5;
    pub const B3_MEAN: usize = 7;
    pub const B3_MEAN_GROUPED: usize = 8;
    pub const B4_FIELD: usize = 9;
    pub const B4_RANGE: usize = 10;
    pub const B4_STDDEV: usize = 11;
    pub const B5_STEP: usize = 12;
    pub const B6_BOX: usize = 13;
    pub const B7_TRIANGLE: usize = 14;
    pub const B8_SMOOTH: usize = 15;
    pub const COUNT: usize = 16;
}

/// Coza a cena e devolva o `Y` de cada lane, na ordem em que ela as monta.
fn lanes() -> Vec<Vec<f32>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = scene::build_stats_demo_document(&mut doc, &reg).expect("a cena tem de montar");
    assert_eq!(sinks.len(), lane::COUNT, "uma lane por sink");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|&s| ys(&mut cook, &doc, &reg, s))
        .collect()
}

fn ys(cook: &mut Cook, doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<f32> {
    let out = cook.cook(&doc.graph, reg, sink, 0.0).expect("cook");
    match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => panic!("sem coluna P"),
    }
}

/// A altura de uma reta — a média das peças. Para uma lane de redução sem grupo
/// isso É o valor difundido, e o gate afirma isso separadamente.
fn level(v: &[f32]) -> f32 {
    v.iter().sum::<f32>() / v.len() as f32
}

fn swing(v: &[f32]) -> f32 {
    v.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x))
        - v.iter().fold(f32::INFINITY, |m, x| m.min(*x))
}

/// **A cena monta as dezasseis cadeias e todas desenham** — o gate mais barato,
/// e o que impede as leituras abaixo de serem verdes sobre uma cena vazia.
#[test]
fn the_stats_scene_builds_every_lane() {
    let ls = lanes();
    for (i, l) in ls.iter().enumerate() {
        assert_eq!(
            l.len(),
            scene::COLS as usize,
            "a lane {i} tem contagem errada"
        );
        assert!(l.iter().all(|y| y.is_finite()), "a lane {i} tem não-finito");
    }
    assert_eq!(scene::BAND_LABELS.len(), scene::BANDS);
}

/// **A FONTE é enviesada, e é ela que torna a banda 1 legível** — num campo
/// simétrico a média e a mediana cairiam no mesmo lugar.
///
/// ⚠️ O oráculo do viés não é "média ≠ mediana" (isso é o que a banda 1 mede, e
/// usá-lo aqui seria circular): é que a **cauda é rara** — a maior parte das
/// peças fica abaixo do meio do alcance.
#[test]
fn the_source_field_is_skewed_by_construction() {
    let ls = lanes();
    let f = &ls[lane::B1_FIELD];
    let sw = swing(f);
    assert!(sw > 0.3, "o campo é chato ({sw:.4}) — não há o que reduzir");
    let lo = f.iter().fold(f32::INFINITY, |m, x| m.min(*x));
    let below_middle = f.iter().filter(|y| **y < lo + sw * 0.5).count();
    assert!(
        below_middle * 100 > f.len() * 70,
        "só {below_middle} de {} peças abaixo do meio — o campo não está enviesado",
        f.len()
    );
}

/// **Uma redução SEM grupo desenha uma RETA** — todas as peças na mesma altura.
/// É a propriedade que faz de uma banda um gráfico legível, e o gate que separa
/// *"a redução difunde"* de *"a redução passa o campo adiante"*.
#[test]
fn a_reduction_without_a_group_draws_a_straight_line() {
    let ls = lanes();
    for (i, name) in [
        (lane::B1_MEAN, "Mean"),
        (lane::B1_MEDIAN, "Median"),
        (lane::B2_MEAN_MASKED, "Mean+mask"),
        (lane::B4_RANGE, "Range"),
        (lane::B4_STDDEV, "Std Dev"),
    ] {
        let sw = swing(&ls[i]);
        assert!(sw < 1e-5, "{name} não é uma reta (amplitude {sw:e})");
    }
}

/// **A MEDIANA não cai onde a MÉDIA cai** — a leitura da banda 1, e a razão de
/// existir do modo. Num campo com cauda a média cede a ela e o rank não.
#[test]
fn the_median_does_not_land_where_the_mean_lands() {
    let ls = lanes();
    let mean = level(&ls[lane::B1_MEAN]);
    let median = level(&ls[lane::B1_MEDIAN]);
    let gap = mean - median;
    let dot = 0.26; // o tamanho da peça: duas retas mais próximas que isto encostam
    assert!(
        gap > dot,
        "a média ({mean:.4}) e a mediana ({median:.4}) distam {gap:.4} — \
         mais perto que uma peça, as duas retas encostam e a banda 1 não se lê"
    );
}

/// **Ligar a MÁSCARA sobe a reta da média** — a leitura da banda 2. E a metade
/// que a torna honesta: a reta continua a ser desenhada por TODAS as peças (a
/// máscara escolhe quem é CONTADO, nunca quem é RESPONDIDO), o que o gate da
/// reta acima já afirma para esta lane.
#[test]
fn wiring_the_mask_raises_the_mean() {
    let ls = lanes();
    let plain = level(&ls[lane::B2_MEAN]);
    let masked = level(&ls[lane::B2_MEAN_MASKED]);
    assert!(
        masked - plain > 0.26,
        "a média mascarada ({masked:.4}) tinha de subir bem acima da média \
         ({plain:.4}) — o degrau da máscara não está a cortar a cauda"
    );
    assert_eq!(
        ls[lane::B2_MEAN_MASKED].len(),
        scene::COLS as usize,
        "e a reta é desenhada por todas as peças, não só pelas contadas"
    );
}

/// **Ligar o GRUPO transforma a reta numa ESCADA** — a leitura da banda 3, e o
/// item que nenhuma composição de nós alcançava.
///
/// O oráculo é a CONTAGEM de degraus: a lane tem de tomar exactamente
/// `GROUP_BINS` alturas distintas, e nenhuma outra contagem serve (uma a menos e
/// dois bins colapsaram; uma a mais e a quantização não partiu onde devia).
#[test]
fn wiring_the_group_turns_the_line_into_a_staircase() {
    let ls = lanes();
    let plain = &ls[lane::B3_MEAN];
    let grouped = &ls[lane::B3_MEAN_GROUPED];
    assert!(
        swing(plain) < 1e-5,
        "a lane de controle tem de ser uma reta"
    );
    assert!(
        swing(grouped) > 0.1,
        "a lane agrupada é uma reta ({:.6}) — o grupo não chegou ao nó",
        swing(grouped)
    );
    let mut levels: Vec<f32> = grouped.clone();
    levels.sort_by(|a, b| a.partial_cmp(b).unwrap());
    levels.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    assert_eq!(
        levels.len(),
        scene::group_bins() as usize,
        "a escada tem {} degraus e devia ter {}",
        levels.len(),
        scene::group_bins() as usize
    );
}

/// **O Range mede o vão inteiro e o desvio a dispersão** — a leitura da banda 4.
/// As duas são grandezas, não níveis, e num campo com cauda rara o desvio fica
/// bem abaixo do vão.
///
/// ⚠️ **A altura de uma lane inclui o `offset_y` da BANDA dela**, e a primeira
/// versão deste gate comparou a altura CRUA do Range com o vão do campo — o erro
/// que mede o LAYOUT em vez do número (o mesmo que o gate de lock-step do grupo
/// B pagou como `1,15`). A base da banda é o CHÃO do campo dela, que é zero por
/// construção (a fonte é um `value.step`, que bottoma em 0).
#[test]
fn the_range_is_the_whole_span_and_the_deviation_is_much_smaller() {
    let ls = lanes();
    let base = ls[lane::B4_FIELD]
        .iter()
        .fold(f32::INFINITY, |m, x| m.min(*x));
    let range = level(&ls[lane::B4_RANGE]) - base;
    let sd = level(&ls[lane::B4_STDDEV]) - base;
    assert!(
        range > sd * 2.0,
        "o vão ({range:.4}) tinha de ser bem maior que a dispersão ({sd:.4})"
    );
    // E o Range é de facto o vão do campo — a mesma escala, medida na fonte.
    let field_span = swing(&ls[lane::B4_FIELD]);
    assert!(
        (range - field_span).abs() < field_span * 0.02,
        "o Range ({range:.4}) tinha de valer o vão do campo ({field_span:.4})"
    );
}

/// **O degrau ATRAVESSA a banda 5** — a fonte das três janelas. Sem esta metade
/// os três perfis abaixo seriam três retas concordantes.
#[test]
fn the_step_source_actually_steps() {
    let ls = lanes();
    let sw = swing(&ls[lane::B5_STEP]);
    assert!(sw > 1.0, "o degrau é raso ({sw:.4})");
}

/// **Os três pesos filtram o MESMO degrau de três maneiras** — a leitura das
/// bandas 6-8.
///
/// ⚠️ **O oráculo é o ARRANQUE da rampa, e o que eu ia usar estava errado.** A
/// primeira versão media CURVATURA e afirmava `Box > Triangle > Smooth`; medido,
/// a ordem é `Box 0,100 · Smooth 0,040 · Triangle 0,027`, porque um S **tem** de
/// curvar mais no meio para ser chato nas pontas. *Uma barra que código correto
/// não consegue satisfazer não é rigor.*
///
/// O que ordena os três — e é o que o olho lê — é **quão de repente a rampa
/// começa**. O incremento de uma amostra para a seguinte é exactamente o peso do
/// tap que acabou de cruzar o degrau, então `primeiro ÷ maior` **É** o perfil de
/// peso visto de lado: `1,00` para o Box (todos iguais), `1/(r+1)` para o
/// Triangle, e menos ainda para o Smooth.
#[test]
fn the_three_weights_filter_the_same_step_three_ways() {
    let ls = lanes();
    let abruptness = |v: &[f32]| {
        let d: Vec<f32> = v.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
        let mx = d.iter().fold(0.0f32, |a, b| a.max(*b));
        assert!(mx > 1e-4, "o perfil é chato — não há rampa a medir");
        let first = d.iter().copied().find(|x| *x > mx * 1e-3).unwrap_or(mx);
        first / mx
    };
    let (b, t, s) = (
        abruptness(&ls[lane::B6_BOX]),
        abruptness(&ls[lane::B7_TRIANGLE]),
        abruptness(&ls[lane::B8_SMOOTH]),
    );
    assert!(
        b > 0.9,
        "o Box tem incrementos IGUAIS: o arranque é o máximo ({b:.4})"
    );
    assert!(
        t < b * 0.3,
        "o Triangle tinha de arrancar bem mais devagar (box {b:.4} vs triangle {t:.4})"
    );
    assert!(
        s < t,
        "e o Smooth mais devagar ainda (triangle {t:.4} vs smooth {s:.4})"
    );
    // E os três TÊM de filtrar: a rampa do degrau cru é UMA amostra.
    let ramp = |v: &[f32]| {
        let mx = v
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0f32, f32::max);
        v.windows(2)
            .filter(|w| (w[1] - w[0]).abs() > mx * 1e-3)
            .count()
    };
    assert_eq!(ramp(&ls[lane::B5_STEP]), 1, "o degrau cru sobe de uma vez");
    for (i, name) in [
        (lane::B6_BOX, "Box"),
        (lane::B7_TRIANGLE, "Triangle"),
        (lane::B8_SMOOTH, "Smooth"),
    ] {
        assert!(
            ramp(&ls[i]) > 5,
            "{name}: a rampa tem {} amostras — não está a filtrar",
            ramp(&ls[i])
        );
    }
}

/// **A sonda que produziu os números do anúncio** — as alturas de cada banda.
#[test]
#[ignore = "sonda: cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture stats_scene"]
fn measure_the_stats_scene() {
    let ls = lanes();
    let f = &ls[lane::B1_FIELD];
    println!(
        "campo: vao {:.4}  min {:.4}",
        swing(f),
        f.iter().fold(f32::INFINITY, |m, x| m.min(*x))
    );
    for (i, name) in [
        (lane::B1_MEAN, "b1 Mean"),
        (lane::B1_MEDIAN, "b1 Median"),
        (lane::B2_MEAN, "b2 Mean"),
        (lane::B2_MEAN_MASKED, "b2 Mean+mask"),
        (lane::B3_MEAN_GROUPED, "b3 Mean+group"),
        (lane::B4_RANGE, "b4 Range"),
        (lane::B4_STDDEV, "b4 Std Dev"),
    ] {
        println!(
            "{name:<16} nivel {:.4}  amplitude {:.4}",
            level(&ls[i]),
            swing(&ls[i])
        );
    }
    for (i, name) in [
        (lane::B5_STEP, "b5 degrau"),
        (lane::B6_BOX, "b6 Box"),
        (lane::B7_TRIANGLE, "b7 Triangle"),
        (lane::B8_SMOOTH, "b8 Smooth"),
    ] {
        let v = &ls[i];
        let d: Vec<f32> = v.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
        let mx = d.iter().fold(0.0f32, |a, b| a.max(*b));
        let first = d.iter().copied().find(|x| *x > mx * 1e-3).unwrap_or(mx);
        println!(
            "{name:<16} vao {:.4}  arranque/maior {:.4}",
            swing(v),
            first / mx
        );
    }
}
