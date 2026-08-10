//! **Quando as partículas nascem** — a família `emit_mode`/`burst_*`, irmã dos arquivos que
//! respondem *onde nasce*, *para onde vai* e *de que tamanho é*.
//!
//! ⚠️ **A rota que a folha do doc 89 dava como parcial está MEDIDA aqui e não serve** — ver
//! [`measure_what_a_driven_rate_actually_does`], que é sonda e não gate: ela imprime o número
//! em vez de afirmar um limiar, porque o que ela documenta é uma propriedade do nó que ninguém
//! deve *consertar* por acidente.

use super::*;
use crate::{MANIFEST, MotionEmitter, Spawn, emit, window};

fn ages_of(s: &Stream) -> Vec<f32> {
    match s.get("age").unwrap() {
        Column::Scalar(v) => v.clone(),
        other => panic!("age is Scalar, got {other:?}"),
    }
}

fn burst(count: u32, time: f32, period: f32) -> Spec {
    let mut s = spec();
    s.spawn = Spawn::Burst {
        count,
        time,
        period,
    };
    s.life = 1.0;
    s
}

// ── O que um burst É ────────────────────────────────────────────────────────

/// **Nada antes da hora, tudo de uma vez na hora, nada depois da vida.** É a definição inteira
/// de um *Spawn Burst Instantaneous*, e cada uma das três metades falha por um motivo diferente.
#[test]
fn a_burst_is_empty_then_whole_then_empty() {
    let s = burst(64, 2.0, 0.0);
    assert_eq!(emit(&s, 1.9).count(), 0, "antes: nada");
    assert_eq!(emit(&s, 2.0).count(), 64, "na hora: as 64");
    assert_eq!(emit(&s, 2.5).count(), 64, "no meio da vida: as mesmas 64");
    assert_eq!(emit(&s, 3.5).count(), 0, "passada a vida: nada");
}

/// **Todas nasceram JUNTAS**, então a idade é uma só — o que separa um burst de um jato curto,
/// e o que faz um fade por idade acender a nuvem inteira ao mesmo tempo.
#[test]
fn every_particle_of_a_burst_shares_one_age() {
    let s = burst(32, 1.0, 0.0);
    let ages = ages_of(&emit(&s, 1.4));
    assert_eq!(ages.len(), 32);
    for (i, a) in ages.iter().enumerate() {
        assert!(
            (a - 0.4).abs() < 1e-5,
            "partícula {i}: idade {a}, esperado 0.4"
        );
    }
}

/// **A identidade é estável enquanto o burst vive** — a mesma propriedade que o modo contínuo
/// tem, e a razão de os ids serem `[k·N, (k+1)·N)` e não uma renumeração por frame.
#[test]
fn a_burst_keeps_its_ids_for_its_whole_life() {
    let s = burst(16, 1.0, 0.0);
    let at = |t: f32| ids_of(&emit(&s, t));
    let (a, b) = (at(1.1), at(1.9));
    assert_eq!(a, b, "os mesmos 16 ids do começo ao fim");
    assert_eq!(a.first().copied(), Some(0.0), "numerados de zero");
}

// ── O período ───────────────────────────────────────────────────────────────

/// **Com período, os bursts se repetem — e ids de bursts diferentes NÃO se misturam.**
///
/// ⚠️ É esta a propriedade que mantém a janela CONTÍGUA: numerar o burst `k` em `[k·N, (k+1)·N)`
/// faz o conjunto vivo ser a união de um INTERVALO de `k`, cujos ids são contíguos por
/// construção — e é por isso que um `SourceWindow` (um `first`, um `count`) basta.
#[test]
fn a_periodic_burst_numbers_each_batch_after_the_last() {
    let s = burst(8, 0.0, 1.0);
    // Vida 1.0 e período 1.0 ⇒ exatamente um burst vivo por vez.
    let first = ids_of(&emit(&s, 0.5));
    let third = ids_of(&emit(&s, 2.5));
    assert_eq!(first, (0..8).map(|i| i as f32).collect::<Vec<_>>());
    assert_eq!(third, (16..24).map(|i| i as f32).collect::<Vec<_>>());
}

/// **Vida maior que o período ⇒ bursts SE SOBREPÕEM, e as idades se separam por período.**
/// É onde o `age_step` ganha sentido: dentro de um lote o passo é zero, e salta um período na
/// fronteira entre lotes.
#[test]
fn overlapping_bursts_are_one_contiguous_window_with_stepped_ages() {
    let mut s = burst(4, 0.0, 1.0);
    s.life = 2.5; // 3 lotes vivos a t = 2.2
    let out = emit(&s, 2.2);
    let ids = ids_of(&out);
    let ages = ages_of(&out);
    assert_eq!(ids.len(), 12, "três lotes de quatro");
    // Contígua: sem buracos.
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(*id, i as f32, "ids contíguos a partir de zero");
    }
    // O lote mais velho nasceu em t=0, o do meio em 1, o novo em 2.
    for (i, a) in ages.iter().enumerate() {
        let want = 2.2 - (i / 4) as f32;
        assert!(
            (a - want).abs() < 1e-5,
            "id {i}: idade {a}, esperado {want}"
        );
    }
}

/// **O teto guarda os lotes MAIS NOVOS**, a mesma regra do modo contínuo — uma nuvem que estoura
/// o orçamento deve parecer o disparo recente, não um fantasma antigo congelado.
#[test]
fn the_cap_keeps_the_newest_bursts() {
    let mut s = burst(10, 0.0, 1.0);
    s.life = 10.0;
    s.max = 25;
    let out = emit(&s, 4.5);
    let ids = ids_of(&out);
    assert_eq!(ids.len(), 25, "capado");
    assert_eq!(
        ids.last().copied(),
        Some(49.0),
        "o topo é o lote mais novo (k=4 ⇒ ids 40..49)"
    );
    // ⚠️ E a idade reportada é a da partícula que SOBREVIVEU, não a do lote que o teto cortou.
    let ages = ages_of(&out);
    assert!(
        (ages[0] - 2.5).abs() < 1e-5,
        "a mais velha viva nasceu em t=2 (id 25 ⇒ lote 2): idade {}",
        ages[0]
    );
}

// ── As fronteiras ───────────────────────────────────────────────────────────

/// Um burst de contagem zero não existe, e um período negativo é um burst só — degenerados que
/// a lei resolve sozinha, sem um ramo que alguém tenha de lembrar de escrever.
#[test]
fn a_degenerate_burst_is_empty_or_single() {
    assert_eq!(emit(&burst(0, 0.0, 1.0), 1.0).count(), 0, "contagem zero");
    let s = burst(5, 0.0, -3.0);
    assert_eq!(emit(&s, 0.5).count(), 5, "período negativo ⇒ um burst só");
    assert_eq!(emit(&s, 5.0).count(), 0, "e ele acaba");
}

/// **O modo contínuo é BYTE-idêntico ao que sempre shipou** — a refatoração que trouxe o segundo
/// modo (`rate` virou `Spawn`) não tem licença para mover um bit do primeiro.
///
/// ⚠️ O oráculo é a lei de contagem CRUA, campo a campo por bits: `count`/`first` são inteiros e
/// `age_first` é o `f32` que a coluna `age` inteira herda.
#[test]
fn the_continuous_mode_is_the_law_that_always_shipped() {
    for &(rate, life, max, t) in &[
        (40.0f32, 3.0f32, 512usize, 7.25f32),
        (10.0, 1.0, 1024, 3.0),
        (4_000_000.0, 1.0, 4096, 3_600.0),
    ] {
        let w = window(Spawn::Continuous { rate }, life, max, t);
        // A forma fechada, escrita aqui à mão: `k/rate`, `[ceil((t−life)·rate), floor(t·rate)]`.
        let (td, rd, ld) = (f64::from(t), f64::from(rate), f64::from(life));
        let newest = (td * rd).floor();
        let oldest = ((td - ld) * rd).ceil().max(0.0);
        let span = (newest - oldest) as u64 + 1;
        let count = span.min(max as u64);
        let first = newest as u64 + 1 - count;
        assert_eq!(w.count, count as usize, "count @ rate {rate}");
        assert_eq!(
            w.first,
            (first % u64::from(ph2d_nodegraph::gpu::ID_WRAP)) as u32,
            "first @ rate {rate}"
        );
        assert_eq!(
            w.age_first.to_bits(),
            ((td - first as f64 / rd) as f32).to_bits(),
            "age_first @ rate {rate}"
        );
    }
}

/// **E a coluna `age` do modo contínuo é a MESMA EXPRESSÃO, não uma equivalente.**
///
/// ⚠️ **Este gate existe porque uma mutação sobreviveu, e ela sobreviveu DUAS vezes** — a
/// segunda ensinando onde medir. Derivar o passo de `born_at` (`(first+k)/rate − first/rate`,
/// em `f64`, rebaixado) é aritmética equivalente e concorda com `k/rate` em quase todo lugar.
///
/// A primeira cena que escolhi (`rate 187`) diverge no PASSO e **não na idade**: o passo só é
/// consumido como `age_first − passo`, e uma diferença de um ulp num número ~0,048 é absorvida
/// ao ser subtraída de ~0,5. *Medir a grandeza intermediária respondeu a pergunta errada — o que
/// shipa é a coluna.* Varrendo a IDADE sobre **18,6 milhões de cenas**, ela diverge **18.269
/// vezes**, e a primeira é a fixture abaixo: `rate 252`, `life 0,5 s`, playhead de 18 horas,
/// `first = 16.518.474` — colado no penhasco de `2²⁴` que este nó já documenta, que é a única
/// região onde `first` é grande o bastante para a f64 e a f32 discordarem.
///
/// O oráculo é a expressão que sempre shipou, escrita à mão: um passo `f32` sobre `k`, e nada
/// mais.
#[test]
fn the_continuous_age_column_is_the_expression_that_always_shipped() {
    const RATE: f32 = 252.0;
    let mut s = spec();
    s.spawn = Spawn::Continuous { rate: RATE };
    s.life = 0.5;
    s.max = 4096;
    let t = 65_550.0f32;
    let out = emit(&s, t);
    let w = window(s.spawn, s.life, s.max, t);
    let ages = ages_of(&out);
    assert!(w.count > 90, "a janela do caso medido: {}", w.count);
    let mut differing = 0usize;
    for (k, a) in ages.iter().enumerate() {
        let want = w.age_first - k as f32 / RATE;
        if a.to_bits() != want.to_bits() {
            differing += 1;
        }
    }
    assert_eq!(
        differing,
        0,
        "{differing} de {} idades não são o passo `k/rate` em f32",
        ages.len()
    );
}

/// **A costura:** o `emit_mode` autorado CHEGA à lei de contagem. Todos os gates acima montam o
/// `Spawn` a mão e ficariam verdes com os cinco params sem leitor.
#[test]
fn the_authored_burst_reaches_the_count_law() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionEmitter as &dyn NodeOp)
        }
    }
    let cook = |burst: bool, t: f64| -> (usize, f32) {
        let mut g = Graph::new();
        let em = g.add_node("motion.emitter");
        g.set_param(em, "rate", 40.0);
        g.set_param(em, "life", 1.0);
        if burst {
            g.set_param(em, "emit_mode", 1.0);
            g.set_param(em, "burst_count", 77.0);
            g.set_param(em, "burst_time", 2.0);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, em, t).unwrap();
        let s = out[0].as_stream();
        // ⚠️ Um stream vazio não tem coluna nenhuma, então a idade só é perguntada quando há
        // alguém para tê-la — a metade *antes da hora* deste gate emite exatamente zero.
        let age = if s.count() == 0 {
            f32::NAN
        } else {
            ages_of(s)[0]
        };
        (s.count(), age)
    };
    // ⚠️ A metade que nomeia o DEFAULT não o menciona: um grafo que nunca tocou o param é o
    // jato contínuo de sempre — 40/s por 1 s de vida.
    assert_eq!(cook(false, 3.0).0, 41, "intocado: o jato");
    assert_eq!(cook(true, 1.9).0, 0, "antes da hora: nada");
    let (n, age) = cook(true, 2.4);
    assert_eq!(n, 77, "as 77 autoradas");
    assert!((age - 0.4).abs() < 1e-5, "nascidas em t=2: idade {age}");
}

/// O kernel é informado dos três números de que precisa — e ⚠️ **`burst_time` NÃO é um deles**,
/// de propósito: ele entra só pela lei de contagem (que a CPU roda e cujo resultado o cook
/// ESCREVE em `window_age`), então passá-lo ao kernel seria um uniforme que nada lê.
#[test]
fn the_kernel_is_handed_what_the_burst_age_needs() {
    let k = crate::GPU_KERNEL;
    for p in ["emit_mode", "burst_count", "burst_period"] {
        assert!(k.params.contains(&p), "falta {p} em {:?}", k.params);
    }
    assert!(
        !k.params.contains(&"burst_time"),
        "burst_time não é lido pelo kernel"
    );
    assert!(k.wgsl.contains("params.burst_period"), "o corpo o lê");
}

/// Os quatro params são APENDADOS, e o neutro reproduz o emissor que sempre shipou.
#[test]
fn the_burst_params_are_appended_with_a_neutral_default() {
    let names: Vec<&str> = MANIFEST.params.iter().map(|p| p.name).collect();
    for p in ["emit_mode", "burst_count", "burst_time", "burst_period"] {
        assert!(names.contains(&p), "falta {p}");
    }
    let mode = MANIFEST
        .params
        .iter()
        .find(|p| p.name == "emit_mode")
        .expect("declarado");
    assert_eq!(mode.default, 0.0, "o neutro é o jato contínuo");
}

// ── A SONDA que re-precifica a folha ────────────────────────────────────────

/// **SONDA — o que um `rate` DIRIGIDO de fato faz** (doc 89 fam. 1, linha 55).
///
/// A folha marcava o burst como **PARCIAL**, dizendo que `pulse.* → rate` como param dirigido
/// *"dá o pulso"*. Não dá, e isto imprime o porquê: a lei de contagem é
/// `newest = floor(t · rate)`, uma forma fechada que trata a taxa ATUAL como se sempre tivesse
/// sido a taxa. O id de uma partícula nascida em `s` é `∫₀ˢ rate(u) du`; com taxa constante isso
/// é `s·rate`, e dirigir a taxa torna a forma fechada uma mentira.
///
/// Medido: um pulso de 40 → 1000 em t=2 **substitui a população inteira** (a janela salta de
/// `[40, 79]` para `[1000, 2000]`, disjunta) e as partículas do pulso somem quando ele baixa, em
/// vez de viverem `life`. Não é um burst parcial — é outro fenômeno, sem nenhuma das propriedades
/// de um burst.
///
/// ⚠️ E o achado maior é sobre TODO `rate` animado, não só sobre bursts: a coerência degrada
/// LINEARMENTE com o playhead, porque `Δnewest = t · Δrate`. Medido com Δrate = **1**:
/// 100% dos ids sobrevivem a t=1, 90% a t=5, 78% a t=10, 29% a t=30 e **0% a t=60**.
///
/// É sonda e não gate porque o número descreve a lei — o dia em que ele mudar é o dia em que
/// alguém trocou a forma fechada por um integral, que é uma decisão, não uma regressão.
#[test]
#[ignore = "sonda de diagnóstico: cargo test -p ph2d-node-motion-emitter -- --ignored --nocapture"]
fn measure_what_a_driven_rate_actually_does() {
    let mut s = spec();
    s.life = 1.0;
    s.max = 100_000;
    eprintln!("  UM PULSO 40 -> 1000 em t=2");
    eprintln!("      t    rate      n   primeiro    ultimo   idade[0]");
    for i in 0..10 {
        let t = 1.94 + i as f32 * 0.02;
        let rate = if (2.0..2.05).contains(&t) {
            1000.0
        } else {
            40.0
        };
        s.spawn = Spawn::Continuous { rate };
        let out = emit(&s, t);
        let ids = ids_of(&out);
        eprintln!(
            "{t:7.2} {rate:7.0} {:6} {:10.0} {:9.0}   {:8.3}",
            ids.len(),
            ids.first().copied().unwrap_or(-1.0),
            ids.last().copied().unwrap_or(-1.0),
            ages_of(&out).first().copied().unwrap_or(f32::NAN),
        );
    }
    eprintln!("\n  UMA RAMPA DE UMA UNIDADE (40 -> 41): quanto da nuvem sobrevive?");
    eprintln!("      t   n(40)  n(41)   ids em comum   sobrevivem");
    for &t in &[0.5f32, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0] {
        s.spawn = Spawn::Continuous { rate: 40.0 };
        let a = ids_of(&emit(&s, t));
        s.spawn = Spawn::Continuous { rate: 41.0 };
        let b = ids_of(&emit(&s, t));
        let shared = a.iter().filter(|i| b.contains(i)).count();
        eprintln!(
            "{t:7.1} {:7} {:6} {:14} {:11.0}%",
            a.len(),
            b.len(),
            shared,
            100.0 * shared as f32 / a.len() as f32
        );
    }
}
