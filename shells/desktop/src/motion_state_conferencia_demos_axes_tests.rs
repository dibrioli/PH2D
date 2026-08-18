//! Os gates da cena `=58` — os dois eixos do tamanho, e o relógio curvado.

use super::*;
use ph2d_eval_motion::MotionCookPump;

/// Coze as quatro bandas num instante e devolve `(size, world_pos)` de cada uma.
///
/// ⚠️ **Pelo `MotionCookPump` com os ESCOPOS DE TEMPO, nunca por um `Cook` cru.** O
/// `motion.time_remap` é um **passthrough**: quem reescreve o relógio é o PULLER, a
/// partir dos escopos que o `time_scopes` colhe do grafo. A primeira versão deste
/// harness cozia sem eles e mediu as duas bandas do relógio **idênticas** — a
/// fixture não continha o fenômeno, e o gate teria ficado verde sobre um remap
/// inerte.
/// O que uma banda mostra num instante: o `size` e a posição de MUNDO de cada peça.
type Band = (Vec<[f32; 2]>, Vec<[f32; 2]>);

fn cook_at(t_seconds: f64) -> Vec<Band> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_axes_demo_document(&mut doc, &reg).expect("a cena monta");
    let scopes = ph2d_node_motion_time_remap::time_scopes(&doc.graph, &reg);
    let tick = (t_seconds * 60.0).round() as u64;

    sinks
        .iter()
        .map(|s| {
            let mut pump = MotionCookPump::new();
            for k in 0..=tick {
                pump.advance_or_scrub_scoped(
                    &doc.graph,
                    &reg,
                    std::slice::from_ref(s),
                    k,
                    |k| k as f64 / 60.0,
                    [0.0, 0.0, 1.0, 1.0],
                    [1.0, 1.0],
                    &scopes,
                );
            }
            (
                pump.instances.iter().map(|i| i.size).collect(),
                pump.instances.iter().map(|i| i.world_pos).collect(),
            )
        })
        .collect()
}

/// O pior `|x − y|` de uma banda — zero significa *toda peça é quadrada*.
fn worst_aspect(size: &[[f32; 2]]) -> f32 {
    size.iter().fold(0.0f32, |m, s| m.max((s[0] - s[1]).abs()))
}

/// Quantas razões `x/y` DISTINTAS a banda tem (a duas casas). Uma anisotropia FIXA
/// tem uma só; campos independentes têm uma por peça.
fn distinct_ratios(size: &[[f32; 2]]) -> usize {
    let mut seen: Vec<i32> = size
        .iter()
        .map(|s| (s[0] / s[1].max(1e-6) * 100.0).round() as i32)
        .collect();
    seen.sort_unstable();
    seen.dedup();
    seen.len()
}

/// A altura média da fileira — o que o relógio move nas bandas 3-4.
fn mean_y(p: &[[f32; 2]]) -> f32 {
    p.iter().map(|q| q[1]).sum::<f32>() / p.len().max(1) as f32
}

/// **AS QUATRO BANDAS EXISTEM, e a mensagem tem quatro rótulos.**
#[test]
fn the_scene_builds_the_four_bands_its_message_names() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_axes_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "quatro bandas");
    assert_eq!(band_labels().count(), 4, "quatro rotulos");
}

/// **1-2 — o CONTROLE é quadrado e a banda dos eixos NÃO é.**
///
/// ⚠️ As duas metades são necessárias. Sozinha, *"a de baixo tem retângulos"* ficaria
/// verde numa cena em que a de cima também os tivesse — e aí a cena mostraria dois
/// campos de tamanho em vez de **um eixo por campo**.
#[test]
fn the_control_is_square_and_the_two_axis_band_is_not() {
    let bands = cook_at(0.0);
    let ctrl = worst_aspect(&bands[0].0);
    assert!(
        ctrl < 1e-6,
        "o CONTROLE tem de ser quadrado em toda peca, e mede {ctrl}"
    );
    let axes = worst_aspect(&bands[1].0);
    assert!(
        axes > 0.3,
        "a banda dos dois eixos tem de ter retangulos, e mede {axes}"
    );
    // E a razão MUDA de peça para peça — nenhuma anisotropia fixa reproduz isso.
    let r = distinct_ratios(&bands[1].0);
    assert!(
        r > 10,
        "as razoes x/y tem de variar entre pecas, e ha' so' {r} distintas"
    );
    assert_eq!(
        distinct_ratios(&bands[0].0),
        1,
        "o controle tem UMA razao (1:1) em toda peca"
    );
}

/// **3-4 — o relógio curvado PARA no meio, e o de sempre não.**
///
/// ⚠️ O oráculo é o **movimento entre dois instantes DENTRO da pausa**, não a posição:
/// as duas bandas oscilam com a mesma amplitude de propósito (o remap reescreve o
/// relógio, nunca a amplitude), então comparar alturas não distinguiria nada.
#[test]
fn the_curved_clock_pauses_in_the_middle_and_the_plain_one_does_not() {
    let w = f64::from(window_seconds());
    // A pausa desenhada vai de 0,40 a 0,60 da janela; medimos DENTRO dela.
    let (a, b) = (0.45 * w, 0.55 * w);
    let (ba, bb) = (cook_at(a), cook_at(b));

    let plain = (mean_y(&bb[2].1) - mean_y(&ba[2].1)).abs();
    let curved = (mean_y(&bb[3].1) - mean_y(&ba[3].1)).abs();
    assert!(
        plain > 0.05,
        "o CONTROLE tem de se mover na janela medida, e move {plain}"
    );
    assert!(
        curved < plain * 0.1,
        "o relogio curvado tem de PARAR: move {curved} contra {plain} do controle"
    );
    eprintln!("[=58] na pausa: de sempre move {plain:.4}, curvado move {curved:.4}");
}

/// **A sonda que a mensagem cita** — ela imprime, não afirma.
#[test]
#[ignore = "sonda: imprime os numeros que a mensagem da cena cita"]
fn measure_what_the_scene_shows() {
    let bands = cook_at(0.0);
    eprintln!("\n[=58] o que a cena monta");
    for (i, (size, _)) in bands.iter().enumerate().take(2) {
        eprintln!(
            "  banda {}: pior |x-y| {:.4}  razoes distintas {}  primeiras {}",
            i + 1,
            worst_aspect(size),
            distinct_ratios(size),
            size.iter()
                .take(4)
                .map(|s| format!("({:.2},{:.2})", s[0], s[1]))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    let w = f64::from(window_seconds());
    eprintln!("  -- o relogio, altura MEDIA da fileira ao longo da janela --");
    for k in 0..=10 {
        let t = k as f64 * w / 10.0;
        let b = cook_at(t);
        eprintln!(
            "  t = {t:>5.2} s | de sempre {:>8.4} | curvado {:>8.4}",
            mean_y(&b[2].1),
            mean_y(&b[3].1)
        );
    }
}

/// **A MENSAGEM e a CENA não podem divergir.**
///
/// Os números vivem em dois lugares por natureza (o que a cena PRODUZ e o que ela
/// DIZ), e é exactamente o par que apodrece quando alguém afina um e esquece o
/// outro — a lição que a wave do ESPAÇAMENTO pagou. Aqui o gate lê o FONTE do
/// irmão que narra a cena e prende os quatro números que ele cita.
#[test]
fn the_printed_numbers_are_the_ones_the_scene_produces() {
    const NARRATION: &str = include_str!("motion_state_demo_conferencia_animadores.rs");
    // Controle positivo: a varredura enxerga o arquivo certo.
    assert!(
        NARRATION.contains("axes-demo"),
        "a varredura nao achou a narracao da cena =58"
    );

    let bands = cook_at(0.0);
    let axes = worst_aspect(&bands[1].0);
    let ratios = distinct_ratios(&bands[1].0);
    let w = f64::from(window_seconds());
    let (ba, bb) = (cook_at(0.45 * w), cook_at(0.55 * w));
    let plain = (mean_y(&bb[2].1) - mean_y(&ba[2].1)).abs();

    for claim in [
        format!("const AXES_WORST: f32 = {axes:.4};"),
        format!("const AXES_RATIOS: usize = {ratios};"),
        format!("const PLAIN_MOVE: f32 = {plain:.4};"),
        format!("const PIECES_PER_ROW: usize = {};", bands[1].0.len()),
    ] {
        assert!(
            NARRATION.contains(&claim),
            "a mensagem perdeu a linha {claim:?} -- ela cita um numero que a cena nao produz"
        );
    }
}
