//! **O PISCA-PISCA DO DRAGON** — report do Enio, 2026-08-30: *"em dragon enquanto cresce
//! (aumentando Generations) parece piscar"*.
//!
//! # A régua que faltava
//!
//! A lei do crescimento normaliza o tamanho por [`turtle::span`] = **o maior lado da caixa
//! alinhada aos eixos**, `max(w, h)`. Essa grandeza **não é invariante à rotação**, e a curva
//! do dragão **roda `45°` por geração** por construção (`F -> F+G` a `90°`). Quando a caixa
//! troca de lado longo, a lei passa a fixar a OUTRA dimensão: o tamanho verdadeiro estagna e
//! depois arranca.
//!
//! # O OBSERVADOR é a largura média de Cauchy, e a escolha custou duas tentativas
//!
//! Para acusar o produto é preciso uma régua que ele não use. As duas primeiras foram
//! **rejeitadas por medição**, e as duas pela MESMA causa:
//!
//! | tentativa | por que caiu |
//! |---|---|
//! | raio de giração (RMS) | é uma medida de DISTRIBUIÇÃO: na travessia de uma geração inteira a contagem de elementos DUPLICA e os novos nascem coincidentes com os pais ⇒ salto puro de amostragem (Tree: passo `−7 991 %` do médio) |
//! | maior distância ao centroide | o **centroide** salta pela mesma razão (Tree: ondulação `151×`, Wild `395×`) |
//!
//! ⇒ a régua tem de ser um EXTENSO sem centroide: `largura(u) = max⟨P,u⟩ − min⟨P,u⟩`, e a
//! MÉDIA dela sobre `K` direções uniformes no semicírculo. É invariante à translação por
//! construção, invariante à rotação a menos de `O(1/K²)`, e pontos coincidentes não a movem.
//! (Para um convexo ela é o `perímetro/π` — a fórmula de Cauchy.)

use ph2d_node_source_lsystem::{PRESETS, Preset, probe_build};
use ph2d_nodegraph::attr::Column;

/// O observador: `K` alto de propósito — ele não paga o orçamento do produto.
const K_OBS: usize = 64;

fn mean_width(p: &ph2d_nodegraph::attr::Stream) -> f32 {
    let Some(Column::Vec2(v)) = p.get("P") else {
        return 0.0;
    };
    if v.is_empty() {
        return 0.0;
    }
    let mut acc = 0.0f64;
    for k in 0..K_OBS {
        let a = std::f32::consts::PI * k as f32 / K_OBS as f32;
        let (c, s) = (a.cos(), a.sin());
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for q in v {
            let t = q[0] * c + q[1] * s;
            lo = lo.min(t);
            hi = hi.max(t);
        }
        acc += f64::from(hi - lo);
    }
    (acc / K_OBS as f64) as f32
}

fn axis_span(p: &ph2d_nodegraph::attr::Stream) -> f32 {
    let Some(Column::Vec2(v)) = p.get("P") else {
        return 0.0;
    };
    if v.is_empty() {
        return 0.0;
    }
    let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
    let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
    let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
    let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
    (x1 - x0).max(y1 - y0)
}

/// `(desvio da rampa recta em % do arrasto, pior passo em % do passo médio)`.
///
/// ⚠️ Duas colunas porque elas apanham defeitos DIFERENTES: o desvio vê uma curvatura lenta,
/// e o pior passo vê a estagnação seguida de arranque (que é o que se lê como «piscar»).
fn straightness(series: &[f32]) -> (f32, f32) {
    let n = series.len();
    if n < 3 {
        return (0.0, 100.0);
    }
    let (a, b) = (series[0], series[n - 1]);
    if (b - a).abs() < 1e-9 {
        return (0.0, 0.0);
    }
    let u: Vec<f32> = series.iter().map(|s| (s - a) / (b - a)).collect();
    let dev = u
        .iter()
        .enumerate()
        .map(|(k, v)| (v - k as f32 / (n - 1) as f32).abs())
        .fold(0.0f32, f32::max);
    let d: Vec<f32> = u.windows(2).map(|w| w[1] - w[0]).collect();
    let mean = 1.0 / (n - 1) as f32;
    let worst = d.iter().copied().fold(f32::MAX, f32::min) / mean;
    (dev * 100.0, worst * 100.0)
}

const STEP: f32 = 0.02;

fn sweep(p: &Preset, trace: bool) {
    let ov = [("angle", p.angle), ("step", p.step), ("width", p.width)];
    // As DUAS últimas gerações do molde: é onde o arrasto do Enio vive.
    let (from, to) = ((p.generations - 2.0).max(0.0), p.generations);
    let steps = ((to - from) / STEP).round() as i32;
    let (mut wid, mut ax) = (vec![], vec![]);
    if trace {
        println!("\n=== {} ({from} .. {to}) ===", p.label);
        println!("  gens      n    span(eixo)  largura média");
    }
    for k in 0..=steps {
        let g = from + k as f32 * STEP;
        let s = probe_build(p.axiom, p.rules, g, &ov);
        let (w, a) = (mean_width(&s), axis_span(&s));
        if trace {
            println!(
                "{g:6.2} {:6}    {a:9.4}      {w:9.4}",
                match s.get("P") {
                    Some(Column::Vec2(v)) => v.len(),
                    _ => 0,
                }
            );
        }
        wid.push(w);
        ax.push(a);
    }
    let (dw, sw) = straightness(&wid);
    let (da, sa) = straightness(&ax);
    println!(
        "{:8} largura média: desvio {dw:5.1} %  pior passo {sw:7.1} %   |   span de eixo: desvio {da:5.1} %  pior passo {sa:7.1} %",
        p.label
    );
}

fn main() {
    let want: Vec<String> = std::env::args().skip(1).collect();
    println!(
        "arrasto das DUAS últimas gerações · desvio = maior afastamento da rampa recta (% do arrasto)\n\
         pior passo = menor passo em % do passo médio (100 % = recta · 0 % = parado · <0 = anda para TRÁS)\n"
    );
    for p in PRESETS {
        if want.is_empty() || want.iter().any(|w| w == p.label) {
            sweep(p, !want.is_empty());
        }
    }
}
