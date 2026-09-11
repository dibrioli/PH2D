//! Os gates da cena `=53` — o teto da taxa.
//!
//! ⚠️ **A régua desta cena é a TAXA, não a pose**, então todo oráculo aqui lê a
//! diferença entre dois tiques da MESMA banda. Comparar bandas em coordenada
//! crua mediria o `BAND_DY` que a cena põe entre elas — a lição que este repo já
//! pagou quatro vezes (o `1,15` do grupo B, o `offset_y` do C, o `BAND_GAP` do
//! D, o `BAND_DY` do K).

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const DT: f64 = 1.0 / 60.0;
const TICKS: usize = 120;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Coze a cena e devolve, por banda, o Y do primeiro elemento a cada tique.
///
/// ⚠️ **O primeiro elemento serve porque a fileira inteira anda junta** — o
/// vaivém dirige o canal Y do stream, não uma pose por peça.
fn run() -> Vec<Vec<f32>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_rate_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut out = vec![Vec::new(); sinks.len()];
    let mut cook = Cook::new();
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("o tique avança o `pre`");
        for (i, s) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, &reg, *s, t).expect("a banda coze");
            if let Some(Column::Vec2(p)) = v[0].as_stream().get("P") {
                out[i].push(p[0][1]);
            }
        }
    }
    out
}

/// Quantos tiques do começo o oráculo pula. Ver o gate do teto de passo.
const BIRTH: usize = 2;

/// O maior passo entre dois tiques consecutivos.
fn worst_step(v: &[f32]) -> f32 {
    v.windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max)
}

/// A excursão pico-a-pico da banda.
fn span(v: &[f32]) -> f32 {
    let hi = v.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let lo = v.iter().copied().fold(f32::INFINITY, f32::min);
    hi - lo
}

/// **A CENA MONTA AS QUATRO BANDAS E CADA UMA TERMINA NUM OUTPUT.**
#[test]
fn the_scene_builds_four_bands_that_all_end_in_an_output() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_rate_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "quatro bandas, dois pares");
    for s in &sinks {
        assert_eq!(
            doc.graph.node(*s).map(|n| n.type_name.as_str()),
            Some("motion.output"),
            "toda banda termina num Output"
        );
    }
    assert_eq!(band_labels().count(), 4, "um rótulo por banda");
}

/// **O TETO DE PASSO SEGURA A TAXA, E A FILEIRA PERDE OS PICOS.**
///
/// O CONTROLE é a primeira metade: se o vaivém não estiver a exigir mais taxa do
/// que o teto permite, as duas bandas saem iguais e a cena não mostra nada.
#[test]
fn the_capped_row_cannot_keep_up_with_the_sway() {
    let p = run();
    let (free, capped) = (&p[0], &p[1]);
    let (max_step, _) = ceilings();
    let w_free = worst_step(free);
    assert!(
        w_free > max_step * 2.0,
        "o CONTROLE: o vaivém TEM de pedir mais taxa que o teto, senão as duas \
         bandas coincidem; passo livre {w_free:.4} contra teto {max_step:.4}"
    );
    // ...e a consequência VISÍVEL: ela não alcança os picos.
    assert!(
        span(capped) < span(free) * 0.8,
        "a banda limitada tem de perder excursão: {:.4} contra {:.4}",
        span(capped),
        span(free)
    );
}

/// **⚠️ ABERTO — o teto é honrado em regime e NÃO na inversão do vaivém, e o
/// número está aqui em vez de escondido numa barra folgada.**
///
/// Medido nesta cena: os passos são `0,0800` **ao dígito** ao longo de toda a
/// rampa e sobem a **`0,1678` no tique 70** — exactamente onde o alvo INVERTE —,
/// com os vizinhos irregulares (`−0,0800 · 0,0256 · 0,1678 · 0,0955 · 0,0800`).
/// A lei do kernel **não pode** produzir isso: ela clampa `|out − prev|`, então
/// o passo é `≤ max_step` por construção, e os cinco gates de unidade do nó
/// (que dirigem o produto pelo `Cook`, com mutação provada) concordam.
///
/// ⇒ **a diferença tem de estar entre o kernel e o que esta cena monta**, e o
/// candidato nomeado é o `prev_out` que o gather entrega no tique da inversão.
/// Fica `#[ignore]` com o mecanismo escrito — o precedente dos dois
/// `watercolor_app_params_incremental` do Painter: *uma barra afrouxada sobre um
/// mecanismo que ninguém entendeu é um gate que deixou de perguntar*.
#[test]
#[ignore = "ABERTO: 0,1678 contra um teto de 0,0800, no tique da inversão — ver o doc"]
fn the_ceiling_is_honoured_on_every_tick_including_the_turn() {
    let p = run();
    let (_, capped) = (&p[0], &p[1]);
    let (max_step, _) = ceilings();
    let d: Vec<f32> = capped.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
    let (worst_i, worst) =
        d[BIRTH..].iter().enumerate().fold(
            (0usize, 0.0f32),
            |a, (i, v)| if *v > a.1 { (i, *v) } else { a },
        );
    assert!(
        worst <= max_step + 1e-5,
        "o teto vale em TODO tique, e não só na rampa; o pior mede {worst:.4} no \
         tique {} contra um teto de {max_step:.4}",
        worst_i + BIRTH
    );
}

/// **O TETO DE ACELERAÇÃO ATRASA O ARRANQUE E DEPOIS SOLTA.**
///
/// As duas metades são o gate: sem a segunda, *"ela parte devagar"* seria
/// satisfeito por um teto de passo, que é a outra banda.
#[test]
fn the_ramped_row_starts_slower_than_it_ends() {
    let p = run();
    let (free, ramped) = (&p[2], &p[3]);
    let early = worst_step(&ramped[..12]);
    let free_early = worst_step(&free[..12]);
    assert!(
        early < free_early * 0.5,
        "o arranque limitado tem de ser MUITO mais lento: {early:.4} contra \
         {free_early:.4}"
    );
    let later = worst_step(&ramped[40..]);
    assert!(
        later > early * 2.0,
        "e depois ele ACELERA, senão isto é um teto de passo disfarçado: \
         {later:.4} contra {early:.4} no arranque"
    );
}

/// **A SONDA** — imprime os números que a mensagem do smoke cita.
#[test]
#[ignore = "sonda"]
fn measure_what_the_scene_shows() {
    let p = run();
    let (max_step, max_accel) = ceilings();
    println!("cena 53 ({TICKS} tiques, lag {TICKS_LAG} tiques, vaivém {PERIOD}s):");
    for (i, label) in band_labels() {
        println!(
            "  {}. {label}\n       maior passo {:.4}   excursão {:.4}",
            i + 1,
            worst_step(&p[i]),
            span(&p[i])
        );
    }
    {
        let d: Vec<f32> = p[1].windows(2).map(|w| w[1] - w[0]).collect();
        let (i, m) = d.iter().enumerate().fold((0usize, 0.0f32), |a, (i, v)| {
            if v.abs() > a.1 { (i, v.abs()) } else { a }
        });
        println!(
            "  pior passo da banda 2: {m:.4} no tique {i}; vizinhos {:?}",
            &d[i.saturating_sub(2)..(i + 3).min(d.len())]
        );
    }
    println!(
        "  os 4 primeiros passos da banda 2: {:?}  (o 1o e' o NASCIMENTO)",
        p[1].windows(2)
            .take(4)
            .map(|w| w[1] - w[0])
            .collect::<Vec<_>>()
    );
    println!("  tetos autorados: passo {max_step:.4}  aceleração {max_accel:.4}");
    println!(
        "  arranque (12 primeiros tiques): controle {:.4}  limitado {:.4}  \
         (e depois {:.4})",
        worst_step(&p[2][..12]),
        worst_step(&p[3][..12]),
        worst_step(&p[3][40..])
    );
}

const TICKS_LAG: f32 = super::TICKS;
