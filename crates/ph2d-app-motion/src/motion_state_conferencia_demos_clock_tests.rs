//! Os gates da cena `=59` — o relógio como campo.

use super::*;
use ph2d_eval_motion::MotionCookPump;

/// O `y` de MUNDO de cada peça de uma banda, cozido até `tick`.
///
/// ⚠️ Pelo `MotionCookPump`, não por um `Cook` cru: o oscilador é `Effect::Temporal`
/// e a bomba é a porta que o app usa. (Aqui não há `motion.time_remap`, logo não há
/// escopos — mas usar a mesma porta é o que mantém a fixture a conter o fenômeno.)
fn band_y(band: usize, tick: u64) -> Vec<f32> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_clock_demo_document(&mut doc, &reg).expect("a cena monta");

    let mut pump = MotionCookPump::new();
    for k in 0..=tick {
        pump.advance_or_scrub_scoped(
            &doc.graph,
            &reg,
            std::slice::from_ref(&sinks[band]),
            k,
            |k| k as f64 / 60.0,
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0],
            &Default::default(),
        );
    }
    pump.instances.iter().map(|i| i.world_pos[1]).collect()
}

/// A mesma banda, mas devolvendo `(x, y)` — a banda 3 precisa do raio.
fn band_xy(band: usize, tick: u64) -> Vec<[f32; 2]> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_clock_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut pump = MotionCookPump::new();
    for k in 0..=tick {
        pump.advance_or_scrub_scoped(
            &doc.graph,
            &reg,
            std::slice::from_ref(&sinks[band]),
            k,
            |k| k as f64 / 60.0,
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0],
            &Default::default(),
        );
    }
    pump.instances.iter().map(|i| i.world_pos).collect()
}

/// As posições do bloco da banda 3 **sem nada em cima** — a régua do raio.
///
/// ⚠️ Monta-se pela MESMA função `grid` da cena, e não por uma cópia dos params: uma
/// segunda escrita de `BLOCK`/`GAP` aqui seria um número que envelhece sozinho.
fn block_grid_positions() -> Vec<[f32; 2]> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut g = Graph::new();
    let side = block_side() as f32;
    let node = grid(&mut g, side, side, 0.0, 0.0);
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let out = cook.cook(&g, &reg, node, 0.0).expect("coza");
    let ph2d_nodegraph::value::CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    match s.get("P") {
        Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// `(resíduo a 1 volta, a 10 voltas, deriva do CONTROLE sem espelho)` da banda 4 —
/// uma função só porque o gate os **afirma** e a mensagem da cena os **cita**.
fn mirror_residues() -> (f32, f32, f32) {
    let w = f64::from(wrap_seconds());
    let period = (2.0 * w * 60.0).round() as u64;
    let base = band_y(3, 40);
    let worst = |k: u64| {
        band_y(3, 40 + k * period)
            .iter()
            .zip(&base)
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()))
    };
    let drift = band_y(1, 40 + period)
        .iter()
        .zip(band_y(1, 40))
        .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
    (worst(1), worst(10), drift)
}

/// Quantas alturas DISTINTAS (a 4 casas) a banda tem. `1` = toda peça no mesmo sítio.
fn distinct(y: &[f32]) -> usize {
    let mut k: Vec<i64> = y.iter().map(|v| (v * 1e4).round() as i64).collect();
    k.sort_unstable();
    k.dedup();
    k.len()
}

/// **AS QUATRO BANDAS EXISTEM, e a mensagem tem quatro rótulos.**
#[test]
fn the_scene_builds_the_four_bands_its_message_names() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_clock_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "quatro bandas");
    assert_eq!(band_labels().count(), 4, "quatro rotulos");
}

/// **1-2 — o controle é uma BARRA e a porta o parte numa ONDA.**
///
/// ⚠️ As duas metades são necessárias, e a primeira é a que custa: com
/// `phase_stagger = 0` a fileira sem porta tem de ter **UMA** altura. Sem esse
/// controle, *"a de baixo tem alturas diferentes"* ficaria verde sobre o knob que o
/// nó já tinha desde sempre, e a cena provaria o `phase_stagger` em vez da porta.
#[test]
fn the_control_moves_as_one_bar_and_the_time_port_breaks_it_into_a_wave() {
    let t = 37; // um instante qualquer FORA de um zero da onda
    let bar = band_y(0, t);
    assert_eq!(
        distinct(&bar),
        1,
        "sem porta e com phase_stagger = 0 a fileira tem de ser UMA barra, e tem {} alturas",
        distinct(&bar)
    );
    let wave = band_y(1, t);
    assert!(
        distinct(&wave) > 10,
        "com um relogio por peca a fileira tem de ser uma onda, e tem so' {} alturas",
        distinct(&wave)
    );
}

/// **3 — o relógio é função do ESPAÇO: mesma distância ⇒ mesmo instante.**
///
/// ⚠️ Esta é a afirmação que separa *"um relógio por elemento"* de *"um relógio por
/// ÍNDICE"*, e é por isso que o oráculo é o RAIO e não a ordem: num bloco 9×9 as
/// quatro quinas têm índices bem distantes e o **mesmo** `|P|`, então um relógio
/// indexado as poria em alturas diferentes. O gate pede as duas coisas ao mesmo
/// tempo — raios diferentes divergem, raios iguais coincidem.
#[test]
fn the_spatial_clock_is_a_function_of_the_radius_not_of_the_index() {
    let side = block_side();
    let base = block_grid_positions();
    assert_eq!(base.len(), side * side, "o bloco tem {side}x{side} pecas");

    // ⚠️ **A DIFERENÇA entre dois instantes, não a altura.** A altura de mundo é
    // `grade + oscilação`, e a parcela da grade não tem nada a ver com o relógio;
    // subtrair dois instantes cancela-a **exactamente** e deixa só o que a porta faz.
    let (a, b) = (band_xy(2, 20), band_xy(2, 50));
    let delta: Vec<f32> = a.iter().zip(&b).map(|(p, q)| q[1] - p[1]).collect();

    // Agrupadas pelo RAIO da grade. Num 9×9 as quatro quinas têm índices bem
    // distantes e o MESMO `|P|`: é isso que separa *um relógio por elemento* de
    // *um relógio por ÍNDICE*, que é o que o `phase_stagger` sempre deu.
    let mut by_radius: std::collections::BTreeMap<i64, Vec<f32>> =
        std::collections::BTreeMap::new();
    for (p, d) in base.iter().zip(&delta) {
        let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        by_radius
            .entry((r * 1e4).round() as i64)
            .or_default()
            .push(*d);
    }
    let shared = by_radius.values().filter(|v| v.len() > 1).count();
    assert!(
        shared >= 4,
        "a fixture tem de CONTER o fenomeno: so' {shared} raios com mais de uma peca"
    );
    let worst = by_radius
        .values()
        .map(|v| {
            let (lo, hi) = v
                .iter()
                .fold((f32::MAX, f32::MIN), |(l, h), x| (l.min(*x), h.max(*x)));
            hi - lo
        })
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-5,
        "pecas 'a mesma distancia tem de partilhar o instante, e divergem {worst}"
    );

    // E a metade OPOSTA: raios DIFERENTES não andam juntos — senão o "campo" seria um
    // relógio global e o gate acima ficaria verde de graça.
    let across = distinct(&delta);
    assert!(
        across > 3,
        "raios diferentes tem de dar instantes diferentes, e ha' so' {across}"
    );
}

/// **4 — o ciclo fecha por CONSTRUÇÃO, e o resíduo NÃO CRESCE.**
///
/// ⚠️ **A barra não é zero, e a primeira versão deste gate pediu `==` e reprovou sobre
/// produto correto.** O relógio é um `f32` e `t` cresce sem parar, então a parte
/// fraccionária de `t/período` perde bits com a magnitude — uma volta depois o quadro
/// repete-se a ~1e-6 de unidade de mundo, e isso é a resolução do número, não o
/// mecanismo. O que se afirma aqui é a coisa que um cross-fade **não** consegue: que
/// o resíduo à décima volta é o MESMO da primeira, em vez de crescer.
///
/// ⚠️ O período do espelho é `2·WRAP_S` (sobe e desce), não `WRAP_S`.
#[test]
fn the_mirrored_clock_repeats_and_the_residue_does_not_grow() {
    let (one, ten, drift) = mirror_residues();
    assert!(
        one < 1e-4,
        "uma volta depois tem de ser o mesmo quadro, e difere {one}"
    );
    assert!(
        ten < 10.0 * one.max(1e-7),
        "o residuo tem de NAO CRESCER: 1 volta {one}, 10 voltas {ten}"
    );
    // CONTROLE: a banda 2 (o mesmo relógio, SEM o espelho) tem de ter derivado — e por
    // uma ordem de grandeza que não se confunde com o resíduo do float.
    assert!(
        drift > 1e3 * ten.max(1e-7),
        "o controle sem wrap tinha de derivar: {drift} contra {ten} do espelhado"
    );
    eprintln!("[=59] espelho: 1 volta {one:.2e}, 10 voltas {ten:.2e}, controle {drift:.4}");
}

/// **A sonda que a mensagem cita** — ela imprime, não afirma.
#[test]
#[ignore = "sonda: imprime os numeros que a mensagem da cena cita"]
fn measure_what_the_scene_shows() {
    eprintln!("\n[=59] o que a cena monta");
    for (i, label) in band_labels() {
        let y = band_y(i, 37);
        eprintln!(
            "  banda {}: {} alturas distintas em {} pecas  ({label})",
            i + 1,
            distinct(&y),
            y.len()
        );
    }
    let w = f64::from(wrap_seconds());
    let period = (2.0 * w * 60.0).round() as u64;
    eprintln!("  -- a banda 4, um periodo ({period} tiques) --");
    for k in [40u64, 40 + period, 40 + 2 * period] {
        let y = band_y(3, k);
        eprintln!("  tique {k:>4} | primeiras alturas {:?}", &y[..3]);
    }
}

/// **A MENSAGEM e a CENA não podem divergir** — o par que apodrece quando alguém
/// afina um e esquece o outro (a lição que a wave do ESPAÇAMENTO pagou).
#[test]
fn the_printed_numbers_are_the_ones_the_scene_produces() {
    const NARRATION: &str = include_str!("motion_state_demo_conferencia_animadores.rs");
    assert!(
        NARRATION.contains("clock-demo"),
        "a varredura nao achou a narracao da cena =59"
    );
    let bar = distinct(&band_y(0, 37));
    let wave = distinct(&band_y(1, 37));
    let block = band_y(2, 37).len();
    let (one, ten, drift) = mirror_residues();
    for claim in [
        format!("const BAR_HEIGHTS: usize = {bar};"),
        format!("const WAVE_HEIGHTS: usize = {wave};"),
        format!("const BLOCK_PIECES: usize = {block};"),
        format!("const MIRROR_ONE: f32 = {one};"),
        format!("const MIRROR_TEN: f32 = {ten};"),
        format!("const MIRROR_DRIFT: f32 = {drift:.4};"),
    ] {
        assert!(
            NARRATION.contains(&claim),
            "a mensagem perdeu a linha {claim:?} -- ela cita um numero que a cena nao produz"
        );
    }
}
