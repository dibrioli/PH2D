//! Os gates da cena `=54` — as oitavas e o laço do `motion.wiggle`.
//!
//! ⚠️ **A régua desta cena é a FORMA da trajetória de uma peça ao longo do
//! tempo**, então todo oráculo lê a MESMA banda em tiques diferentes. Comparar
//! bandas em coordenada crua mediria o `BAND_DY` que a cena põe entre elas — a
//! lição que este repo já pagou cinco vezes.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const DT: f64 = 1.0 / 60.0;
const TICKS: usize = 240;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Coze a cena e devolve, por banda, o Y do primeiro elemento a cada tique.
fn run() -> Vec<Vec<f32>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_octave_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut out = vec![Vec::new(); sinks.len()];
    let mut cook = Cook::new();
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("o tique avança");
        for (i, s) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, &reg, *s, t).expect("a banda coze");
            if let Some(Column::Vec2(p)) = v[0].as_stream().get("P") {
                out[i].push(p[0][1]);
            }
        }
    }
    out
}

/// A rugosidade da trajetória: a média de `|segunda diferença|`. Uma onda lisa
/// tem curvatura pequena; um tremor de várias oitavas tem curvatura grande —
/// e ela é **invariante ao deslocamento da banda**, que é o ponto.
fn roughness(v: &[f32]) -> f32 {
    let s: f32 = v.windows(3).map(|w| (w[2] - 2.0 * w[1] + w[0]).abs()).sum();
    s / (v.len() - 2) as f32
}

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
    let sinks = build_octave_demo_document(&mut doc, &reg).expect("a cena monta");
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

/// **AS OITAVAS ACRESCENTAM TEXTURA, E NÃO TAMANHO.**
///
/// As duas metades são o gate: a rugosidade tem de SUBIR (é a feature) e a
/// excursão tem de ficar da mesma ordem (senão a cena estaria a mostrar *"a de
/// baixo mexe mais"*, que qualquer mudança de amplitude produziria).
#[test]
fn the_octaves_add_texture_not_size() {
    let p = run();
    let (flat, rough) = (&p[0], &p[1]);
    let (rf, rr) = (roughness(flat), roughness(rough));
    assert!(
        rr > rf * 2.0,
        "cinco oitavas TÊM de enrugar a trajetória: {rr:.5} contra {rf:.5}"
    );
    let (sf, sr) = (span(flat), span(rough));
    assert!(
        sr < sf * 1.6 && sr > sf * 0.4,
        "e a excursão fica da MESMA ordem — {sr:.3} contra {sf:.3}"
    );
}

/// **O LAÇO FECHA NA TELA, E O CONTROLE NÃO.**
///
/// ⚠️ O oráculo é a banda comparada CONSIGO MESMA, um período depois — é o que
/// torna a leitura *"ela repete"* mensurável sem olhar para a vizinha.
#[test]
fn the_looped_band_returns_and_the_open_one_does_not() {
    let p = run();
    let period = (LOOP_LEN as f64 / DT).round() as usize;
    let (open, looped) = (&p[2], &p[3]);
    let d = |v: &Vec<f32>| {
        (0..v.len() - period)
            .map(|k| (v[k + period] - v[k]).abs())
            .fold(0.0f32, f32::max)
    };
    let (od, ld) = (d(open), d(looped));
    assert!(
        od > 0.05,
        "o CONTROLE: sem laço a banda NÃO volta ao mesmo lugar ({od:.4})"
    );
    assert!(
        ld < 1e-3,
        "com laço ela volta, e a volta tem de fechar: {ld:.6}"
    );
}

/// **A SONDA** — imprime os números que a mensagem do smoke cita.
#[test]
#[ignore = "sonda"]
fn measure_what_the_scene_shows() {
    let p = run();
    let (oct, amp_mult, loop_len) = knobs();
    println!("cena 54 ({TICKS} tiques, oitavas {oct}, amp_mult {amp_mult}, laço {loop_len}s):");
    for (i, label) in band_labels() {
        println!(
            "  {}. {label}\n       rugosidade {:.5}   excursão {:.3}",
            i + 1,
            roughness(&p[i]),
            span(&p[i])
        );
    }
    let period = (loop_len as f64 / DT).round() as usize;
    for (i, name) in [(2usize, "controle"), (3, "com laço")] {
        let d = (0..p[i].len() - period)
            .map(|k| (p[i][k + period] - p[i][k]).abs())
            .fold(0.0f32, f32::max);
        println!("  volta de {loop_len}s na banda {} ({name}): {d:.6}", i + 1);
    }
}
