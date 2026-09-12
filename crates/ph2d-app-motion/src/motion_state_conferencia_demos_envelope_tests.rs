//! Gates da cena `=46` — **o ENVELOPE**.
//!
//! Eles medem a GEOMETRIA que a cena cozinha ao longo do TEMPO, não a intenção
//! com que ela foi escrita: se a mensagem diz *"uma POPA e a outra INCHA"*, o
//! cozido tem de o mostrar tick a tick.
//!
//! ⚠️ **A SILHUETA, nunca o Y cru** — como nas quatro cenas anteriores. Aqui o que
//! se mede é o **TAMANHO**, que é onde o flash vive; cada fileira carrega o próprio
//! `offset_y` e comparar Y entre bandas mediria o `BAND_GAP`.
//!
//! ⚠️ **E toda medição AVANÇA O TICK.** Sem `advance_tick` a aresta `pre` nunca
//! carrega estado: o `pulse.beat` não bate, o envelope não decai e as dez fileiras
//! saem IDÊNTICAS e paradas — a fixture ficaria verde sobre uma cena morta. É a
//! mesma lição que a cena `=38` pagou.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// 60 ticks = um segundo. Duas batidas de `BEAT = 2 s` cabem em 240.
const DT: f64 = 1.0 / 60.0;

/// Cozinha a cena por `ticks` ticks e devolve, por banda, o **tamanho médio** da
/// fileira em cada tick.
///
/// A média sobre a fileira é o oráculo certo para as bandas 1-6 e 9-10 (todas as
/// peças fazem a mesma coisa) e é *exatamente* o que separa a banda 8 da 7 (lá o
/// interessante é QUANTAS acendem).
fn size_trace(ticks: usize) -> Vec<Vec<f32>> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_envelope_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), BANDS, "uma fileira por banda");

    let mut cook = Cook::new();
    let mut trace: Vec<Vec<f32>> = vec![Vec::new(); BANDS];
    for t in 0..ticks {
        #[expect(clippy::cast_precision_loss, reason = "poucos ticks")]
        let now = t as f64 * DT;
        for (b, &s) in sinks.iter().enumerate() {
            let out = cook.cook(&doc.graph, &reg, s, now).expect("cook");
            let Some(Column::Vec2(v)) = out[0].as_stream().get("size") else {
                panic!("a banda {b} emite size")
            };
            #[expect(clippy::cast_precision_loss, reason = "COLS e' pequeno")]
            let mean = v.iter().map(|s| s[0]).sum::<f32>() / v.len() as f32;
            trace[b].push(mean);
        }
        cook.advance_tick(&doc.graph, &reg, now).expect("tick");
    }
    trace
}

/// **O ATTACK INCHA e a ausência dele POPA** — o par 1-2.
///
/// ⚠️ **A 1ª versão deste gate foi MORTA por uma mutação, e a lição é o oráculo:**
/// ela media *"a de baixo está abaixo de 75% do pico a meio caminho da rampa"* —
/// e isso é verdade tanto para quem **ainda não chegou** como para quem **já
/// passou**. Com as duas bandas em `attack = 0` a de baixo já decaiu a 2% naquele
/// tick, e o gate ficava VERDE sobre um par idêntico. O discriminante é **QUANDO**
/// o pico acontece: a de cima no instante da batida, a de baixo `attack` ticks
/// depois.
#[test]
fn the_attack_swells_where_its_absence_pops() {
    let tr = size_trace(90);
    let (pop, swell) = (&tr[0], &tr[1]);
    let peak_at = |v: &Vec<f32>| {
        v.iter().enumerate().fold(
            (0usize, f32::MIN),
            |acc, (i, &x)| {
                if x > acc.1 { (i, x) } else { acc }
            },
        )
    };
    let (pop_t, pop_peak) = peak_at(pop);
    let (swell_t, swell_peak) = peak_at(swell);

    assert!(
        swell_t >= pop_t + (HALF_SECOND as usize) - 1,
        "a de baixo tem de chegar ao pico ~{} ticks DEPOIS: {pop_t} contra {swell_t}",
        HALF_SECOND as u32
    );
    // E ela CHEGA lá — uma rampa que nunca subisse também "atrasaria" o pico.
    assert!(
        swell_peak > DOT * 2.0 && (swell_peak - pop_peak).abs() < DOT * 0.05,
        "as duas chegam ao MESMO pico: {pop_peak:.4} contra {swell_peak:.4}"
    );
    eprintln!(
        "attack: pico da de cima no tick {pop_t} · da de baixo no {swell_t} \
         (os dois em {pop_peak:.4})"
    );
}

/// **O HOLD é um PLATÔ, não uma queda mais lenta** — o par 3-4.
///
/// ⚠️ A metade que importa: a de baixo tem de ficar **no pico** durante a janela
/// autorada. Uma queda mais lenta também estaria "mais alta", e é por isso que o
/// oráculo compara contra o PRÓPRIO pico dela e não contra a vizinha.
#[test]
fn the_hold_holds_at_the_peak_and_only_then_falls() {
    let tr = size_trace(90);
    let (plain, held) = (&tr[2], &tr[3]);
    let peak = held.iter().copied().fold(f32::MIN, f32::max);
    let beat_at = held
        .iter()
        .position(|&s| s >= peak * 0.99)
        .expect("a batida acontece");
    let window = HALF_SECOND as usize;

    for (t, &v) in held.iter().enumerate().skip(beat_at).take(window) {
        assert!(
            v >= peak * 0.999,
            "o platô tem de ser CRAVADO no tick {t}: {v:.4} contra {peak:.4}"
        );
    }
    assert!(
        held[beat_at + window + 8] < peak * 0.9,
        "e depois dele CAI: {:.4}",
        held[beat_at + window + 8]
    );
    // CONTROLE: a vizinha sem hold já caiu bem dentro da mesma janela.
    assert!(
        plain[beat_at + window / 2] < peak * 0.5,
        "a de cima cai desde logo: {:.4}",
        plain[beat_at + window / 2]
    );
}

/// **O DEGRAU CORTA onde a exponencial desvanece** — o par 5-6.
///
/// ⚠️ O discriminante é a queda ser **abrupta**, não ser mais rápida: mede-se o
/// maior salto de um tick para o seguinte, que numa exponencial é sempre pequeno
/// e num degrau é o pico inteiro.
#[test]
fn the_drawn_cliff_cuts_where_the_exponential_fades() {
    let tr = size_trace(90);
    let biggest_drop = |v: &Vec<f32>| v.windows(2).map(|w| w[0] - w[1]).fold(f32::MIN, f32::max);
    let (fade, cliff) = (biggest_drop(&tr[4]), biggest_drop(&tr[5]));
    let peak = tr[5].iter().copied().fold(f32::MIN, f32::max) - DOT;
    assert!(
        cliff > peak * 0.5,
        "o degrau larga MEIO flash num tick: {cliff:.4} de {peak:.4}"
    );
    // ⚠️ **A 2ª metade é uma RAZÃO, e a 1ª versão dela era uma barra absoluta que
    // reprovava produto correto:** o maior passo de uma exponencial é o PRIMEIRO,
    // e uma cauda de 20 ticks larga 25% logo ali (medido: 0,1379 contra uma barra
    // de 0,114 que eu tinha escolhido). Comparar as duas quedas entre si não tem
    // número a calibrar — e é a comparação que a cena de facto mostra.
    assert!(
        cliff > fade * 2.5,
        "o degrau larga MUITO mais que a exponencial: {cliff:.4} contra {fade:.4}"
    );
    eprintln!("shape: maior queda por tick — exponencial {fade:.4} · degrau {cliff:.4}");
}

/// **A PROBABILIDADE acende UMA FRAÇÃO, e peças DIFERENTES a cada batida** — o
/// par 7-8.
///
/// ⚠️ As duas metades são independentes e as duas são necessárias. *"Uma fração"*
/// sozinho passaria com um sorteio TRAVADO (as mesmas peças, sempre); *"peças
/// diferentes"* sozinho passaria com um sorteio que acende tudo. Juntas elas são a
/// feature.
#[test]
fn the_probability_lights_a_fraction_and_a_different_one_each_beat() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_envelope_demo_document(&mut doc, &reg).expect("a cena monta");
    let (all, some) = (sinks[6], sinks[7]);

    let mut cook = Cook::new();
    let mut shots: Vec<Vec<bool>> = Vec::new();
    let mut all_counts = Vec::new();
    let mut some_counts = Vec::new();
    // ⚠️ **A batida é a BORDA, não o estado de estar aceso** — e a 1ª versão
    // deste laço confundiu os dois. Com uma cauda de 20 ticks a fileira fica
    // "toda acesa" por ~seis ticks depois de cada batida, então gravar todo tick
    // aceso recolhia SEIS retratos da MESMA batida e o gate acusava o sorteio de
    // ter travado. A fixture não continha o fenômeno que ela media.
    let mut was_all = false;
    for t in 0..400usize {
        #[expect(clippy::cast_precision_loss, reason = "poucos ticks")]
        let now = t as f64 * DT;
        let a = lit_at(&mut cook, &doc, &reg, all, now);
        let s = lit_at(&mut cook, &doc, &reg, some, now);
        let now_all = a.iter().all(|&b| b);
        if now_all && !was_all {
            all_counts.push(a.iter().filter(|b| **b).count());
            some_counts.push(s.iter().filter(|b| **b).count());
            shots.push(s);
        }
        was_all = now_all;
        cook.advance_tick(&doc.graph, &reg, now).expect("tick");
    }

    assert!(
        shots.len() >= 3,
        "a janela tem de conter 3 batidas: {}",
        shots.len()
    );
    assert!(
        all_counts.iter().all(|&c| c == COLS as usize),
        "a banda 7 acende a fileira INTEIRA: {all_counts:?}"
    );
    #[expect(clippy::cast_precision_loss, reason = "contagens pequenas")]
    let mean = some_counts.iter().sum::<usize>() as f32 / some_counts.len() as f32;
    let want = SOME * COLS;
    assert!(
        (mean - want).abs() < COLS * 0.2,
        "a banda 8 acende ~{want:.1} pecas, mediu {mean:.1}: {some_counts:?}"
    );
    // E o CONJUNTO muda de batida para batida — a metade que um sorteio travado
    // falha.
    assert!(
        shots[0] != shots[1] || shots[1] != shots[2],
        "as MESMAS pecas em toda batida = o sorteio travou"
    );
    eprintln!(
        "probability: fileira inteira {} · sorteada {mean:.1} de {} em {} batidas",
        all_counts[0],
        COLS as u32,
        shots.len()
    );
}

/// Quais peças da banda estão acesas neste instante.
fn lit_at(
    cook: &mut Cook,
    doc: &MotionDoc,
    reg: &NodeRegistry,
    sink: NodeId,
    now: f64,
) -> Vec<bool> {
    let out = cook.cook(&doc.graph, reg, sink, now).expect("cook");
    let Some(Column::Vec2(v)) = out[0].as_stream().get("size") else {
        panic!("size")
    };
    v.iter().map(|s| s[0] > DOT * 1.5).collect()
}

/// **O SMOOTHER SALTA E ESCORRE** — o par 9-10.
///
/// ⚠️ As duas metades outra vez: a SUBIDA tem de ser a mesma (a régua da descida
/// não pode tocá-la) e a DESCIDA do assimétrico tem de ficar muito mais alta. Um
/// `ticks_down` que simplesmente atrasasse tudo falharia a primeira.
#[test]
fn the_smoother_snaps_up_and_oozes_down() {
    let tr = size_trace(200);
    let (sym, asym) = (&tr[8], &tr[9]);
    let peak = sym.iter().copied().fold(f32::MIN, f32::max);

    // O topo do degrau: o primeiro tick em que o simétrico chega ao pico.
    let up = sym
        .iter()
        .position(|&s| s >= peak * 0.99)
        .expect("o degrau sobe");
    assert!(
        (asym[up] - sym[up]).abs() < peak * 0.02,
        "a SUBIDA e' a mesma nos dois: {:.4} contra {:.4}",
        sym[up],
        asym[up]
    );
    // A borda de descida: o primeiro tick depois do topo em que o simétrico já
    // largou metade.
    let down = (up..sym.len())
        .find(|&t| sym[t] < DOT + (peak - DOT) * 0.5)
        .expect("o degrau desce");
    assert!(
        asym[down] > DOT + (peak - DOT) * 0.8,
        "o assimetrico ainda esta' la' em cima em {down}: {:.4} contra {:.4}",
        asym[down],
        sym[down]
    );
    eprintln!(
        "smoother: subida {:.4}/{:.4} · descida no tick {down} {:.4} contra {:.4}",
        sym[up], asym[up], sym[down], asym[down]
    );
}
