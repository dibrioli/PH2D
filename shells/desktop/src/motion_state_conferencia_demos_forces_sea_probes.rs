//! **AS SONDAS DO MAR** da cena `=95` — medição, não gates (doc 89, folha 02).
//!
//! ⚠️ **Ficheiro irmão de propósito.** As sondas que resolveram o
//! [Bug #7](../../../docs/Motion%20Nodes/BUGS_motion_nodes.md) são maiores que os gates que
//! delas saíram — juntas levavam `..._sea_tests.rs` a **687** linhas contra o teto de `600`
//! do shell. Aqui vive o que MEDE; ao lado vive o que AFIRMA.
//!
//! ⛔ Tudo aqui é `#[ignore]`: uma sonda **imprime**, nunca reprova. As barras dos gates saem
//! destas tabelas, e é por isso que elas ficam no repo em vez de morrerem no terminal.

use super::sea_tests::{crest_variety, submersions, surface_line};
use super::tests::{DT, registry, scene};
use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::Graph;

/// SONDA — o **mar**: a média de `y` de cada banda ao longo do tempo.
///
/// ⚠️ **A régua é a DERIVA da média, e não a dispersão.** Uma nuvem que assenta numa
/// superfície tem média estável; uma que foi LANÇADA tem média a subir para sempre, e as
/// duas podem ter a mesma dispersão.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn measure_the_sea() {
    let (doc, reg, sinks) = scene();
    let mut cook = Cook::new();
    let mut trace: Vec<Vec<f32>> = vec![Vec::new(); 2];
    // ⚠️ Mais longo que o gate de propósito: o que se procura aqui é ONDE assenta, e o
    // transiente do mergulho inicial dura mais que a janela que os gates medem.
    const LONG: usize = 900;
    for k in 0..LONG {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        for &s in &sinks {
            let _ = cook.cook(&doc.graph, &reg, s, t);
        }
        if k % 75 == 0 || k == LONG - 1 {
            for (j, &s) in sinks.iter().enumerate().skip(6) {
                let out = cook.cook(&doc.graph, &reg, s, t).expect("coze");
                if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                    let mean = p.iter().map(|q| q[1]).sum::<f32>() / p.len() as f32;
                    let wx = p.iter().map(|q| q[0]).fold(f32::MIN, f32::max)
                        - p.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
                    let d = submersions(p, j, t as f32);
                    println!(
                        "tique {k:3} banda {j}: media y {mean:8.4} · largura x {wx:6.3} · \
                         submersao mediana {:7.4} (p10 {:7.4} · p90 {:7.4})",
                        d[1], d[0], d[2]
                    );
                    trace[j - 6].push(mean);
                }
            }
        }
    }
    for (j, tr) in trace.iter().enumerate() {
        let drift = tr.windows(2).map(|w| w[1] - w[0]).fold(0.0_f32, f32::max);
        println!("banda {}: MAIOR subida entre amostras {drift:.4}", j + 6);
    }
    // ⭐ **O que cada boia FAZ**, no regime já assentado: quanto ela sobe e desce, e quanto
    // ela anda de lado. É isto que decide se o mar se VÊ a mexer.
    let mut track: Vec<Vec<Vec<[f32; 2]>>> = vec![Vec::new(); 2];
    for k in LONG..LONG + 300 {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        for (j, &s) in sinks.iter().enumerate() {
            let o = cook.cook(&doc.graph, &reg, s, t).expect("coze");
            if j >= 6
                && k % 5 == 0
                && let Some(Column::Vec2(p)) = o[0].as_stream().get("P")
            {
                track[j - 6].push(p.clone());
            }
        }
    }
    for (j, frames) in track.iter().enumerate() {
        let n = frames[0].len();
        let span = |axis: usize| {
            let mut v: Vec<f32> = (0..n)
                .map(|i| {
                    let lo = frames.iter().map(|f| f[i][axis]).fold(f32::MAX, f32::min);
                    let hi = frames.iter().map(|f| f[i][axis]).fold(f32::MIN, f32::max);
                    hi - lo
                })
                .collect();
            v.sort_by(f32::total_cmp);
            (v[n / 2], v[n - 1])
        };
        let (my, xy) = span(1);
        let (mx, xx) = span(0);
        println!(
            "banda {}: excursao VERTICAL mediana {my:.4} (max {xy:.4}) · HORIZONTAL mediana {mx:.4} (max {xx:.4})",
            j + 6
        );
    }
}

/// Uma banda de mar sozinha, com os números que se quiserem — o arnês da varredura.
///
/// Devolve `(excursão vertical mediana, excursão horizontal mediana, submersão mediana)` no
/// regime já assentado.
/// Monta e corre UMA banda de mar com os números que se quiserem, e devolve
/// `(as poses do regime já assentado, a última pose)`.
///
/// ⚠️ **O calado é ARGUMENTO** — ele é a alavanca que o [`Bug #7`](../../../docs/Motion%20Nodes/BUGS_motion_nodes.md)
/// nomeia, e uma varredura que o lesse do autorado mediria sempre o mesmo sítio.
fn run_sea(
    density: f32,
    grav: f32,
    drag: f32,
    speed: f32,
    waves: f32,
    draft: f32,
) -> (Vec<Vec<[f32; 2]>>, Vec<[f32; 2]>) {
    let reg = registry();
    let mut g = Graph::new();
    let (_, lambda, ..) = sea_authored();
    let amp = lambda * 0.1;
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", 2.0);
    g.set_param(src, "cols", 128.0);
    g.set_param(src, "gap_x", 7.0 / 127.0);
    g.set_param(src, "gap_y", 0.3);
    let up = g.add_node("motion.move");
    g.set_param(up, "dy", 0.6);
    let integ = g.add_node("motion.integrate");
    let w = g.add_node("force.wind");
    g.set_param(w, "angle", 270.0);
    g.set_param(w, "strength", grav);
    g.set_param(w, "gust", 0.0);
    let b = g.add_node("force.buoyancy");
    g.set_param(b, "level", 0.0);
    g.set_param(b, "density", density);
    g.set_param(b, "depth", draft);
    g.set_param(b, "drag", drag);
    g.set_param(b, "wave_amplitude", amp);
    g.set_param(b, "wave_length", lambda);
    g.set_param(b, "wave_speed", speed);
    g.set_param(b, ph2d_node_force_buoyancy::WAVES, waves);
    for (from, to, port, delayed) in [
        (src, up, 0, false),
        (up, integ, 0, false),
        (integ, w, 0, true),
        (w, b, 0, false),
        (b, integ, 1, false),
    ] {
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (from, 0),
            to: (to, port),
            delayed,
        })
        .expect("liga");
    }
    g.validate(&reg).expect("bem-tipada");

    let mut cook = Cook::new();
    let mut frames: Vec<Vec<[f32; 2]>> = Vec::new();
    let mut last = Vec::new();
    for k in 0..1200 {
        let t = f64::from(k) / 60.0;
        cook.advance_tick(&g, &reg, t).expect("avanca");
        let o = cook.cook(&g, &reg, integ, t).expect("coze");
        if let Some(Column::Vec2(p)) = o[0].as_stream().get("P") {
            if k >= 900 && k % 5 == 0 {
                frames.push(p.clone());
            }
            last = p.clone();
        }
    }
    (frames, last)
}

fn one_sea(density: f32, grav: f32, drag: f32, speed: f32, waves: f32) -> (f32, f32, f32) {
    let (_, lambda, _, draft, _) = sea_authored();
    let amp = lambda * 0.1;
    let (frames, last) = run_sea(density, grav, drag, speed, waves, draft);
    let n = frames[0].len();
    let median_span = |axis: usize| {
        let mut v: Vec<f32> = (0..n)
            .map(|i| {
                let lo = frames.iter().map(|f| f[i][axis]).fold(f32::MAX, f32::min);
                let hi = frames.iter().map(|f| f[i][axis]).fold(f32::MIN, f32::max);
                hi - lo
            })
            .collect();
        v.sort_by(f32::total_cmp);
        v[n / 2]
    };
    let t = 1199.0_f32 / 60.0;
    let mut d: Vec<f32> = last
        .iter()
        .map(|q| {
            ph2d_node_force_buoyancy::surface_at(q[0], t, 0.0, amp, lambda, speed, waves) - q[1]
        })
        .collect();
    d.sort_by(f32::total_cmp);
    // ⚠️ **A excursão horizontal não distingue ORBITAR de PARTIR.** Uma boia que vai e vem
    // meia onda tem a mesma excursão de uma que anda meia onda e nunca volta; o que separa
    // as duas é a deriva LÍQUIDA da banda.
    let mean_x = |f: &Vec<[f32; 2]>| f.iter().map(|q| q[0]).sum::<f32>() / f.len() as f32;
    let net = mean_x(frames.last().expect("frames")) - mean_x(&frames[0]);
    (median_span(1), net, d[d.len() / 2])
}

/// SONDA — **[Bug #7](../../../docs/Motion%20Nodes/BUGS_motion_nodes.md): as cristas parecem
/// diferentes?** A régua que o report do Enio pedia, e que nenhuma outra tinha.
///
/// ⚠️ **Mede as DUAS**: a variedade que a superfície TEM e a que as boias DESENHAM. O
/// defeito é a diferença entre elas — e uma régua só da superfície diria que está tudo bem.
///
/// ⛔ **A saída barata é medida PRIMEIRO** (menos camadas), antes de mexer no calado: a 4.ª
/// camada tem `1/8` da amplitude e pode ser invisível de qualquer maneira.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn measure_the_crest_variety() {
    let (amp, lambda, ..) = sea_authored();
    let t = 1199.0_f32 / 60.0;
    let (cn, cv) = crest_variety(&surface_line(t, 1.0), lambda / 24.0, amp);
    println!("CONTROLE — a senoide PURA tem de dar ~0: {cn} cristas, variedade {cv:.4}");
    println!(
        "ondas calado |  superficie: n  variedade  |  boias: n  variedade  | fraccao desenhada"
    );
    for waves in [1.0_f32, 2.0, 3.0, 4.0] {
        // A janela segue a onda mais fina DESTE espectro, não uma constante.
        let win = ph2d_node_force_buoyancy::finest_wavelength(lambda, waves) / 3.0;
        let (sn, sv) = crest_variety(&surface_line(t, waves), win, amp);
        let (_, last) = run_sea(6.0, 2.0, sea_drag(), 1.0, waves, 0.5);
        let (fnum, fv) = crest_variety(&last, win, amp);
        let frac = if sv > 1e-6 { fv / sv } else { 0.0 };
        println!(
            "{waves:5.0} {:6.2} |  {sn:12}  {sv:9.4}  |  {fnum:7}  {fv:9.4}  | {frac:16.2}",
            0.5
        );
    }
    // ⭐ **O eixo que faltava: o ARRASTO.** Encolher o calado sobe a frequência própria da
    // boia (`√(densidade/calado)`) e ao mesmo tempo BAIXA o amortecimento
    // (`ζ = arrasto·sub / 2ω_n`) — por isso ela passa a inventar cristas em vez de as
    // seguir. Manter `ζ` obriga o arrasto a subir com `ω_n`, ou seja com `1/√calado`.
    // ⚠️ A CONTAGEM de cristas é a régua que separa FIEL de RUIDOSO: a superfície tem 8.
    println!();
    println!("waves=4 · veloc arrasto | boias: n  variedade  fraccao | deriva liquida");
    let win4 = ph2d_node_force_buoyancy::finest_wavelength(lambda, 4.0) / 3.0;
    let (sn4, sv4) = crest_variety(&surface_line(t, 4.0), win4, amp);
    println!("  (a superficie: {sn4} cristas, variedade {sv4:.4})");
    println!("calado veloc arrasto ondas | limiar | n  variedade | balanco (x vaga) | deriva");
    let height = 2.0 * amp;
    for (draft, speed, drag) in [
        (0.20_f32, 0.50_f32, 12.0_f32),
        (0.20, 0.50, 14.0),
        (0.20, 0.50, 16.0),
        (0.20, 0.50, 18.0),
        (0.20, 0.50, 20.0),
    ] {
        for waves in [1.0_f32, 4.0] {
            let win = ph2d_node_force_buoyancy::finest_wavelength(lambda, waves) / 3.0;
            let (sn, sv) = crest_variety(&surface_line(t, waves), win, amp);
            let bar = 6.0 * (4.0 * std::f32::consts::TAU * 0.1)
                / (1.0 + (4.0 * std::f32::consts::TAU * 0.1).powi(2)).sqrt()
                / speed;
            let (frames, last) = run_sea(6.0, 2.0, drag, speed, waves, draft);
            let (fnum, fv) = crest_variety(&last, win, amp);
            let n = frames[0].len();
            let mut ex: Vec<f32> = (0..n)
                .map(|i| {
                    let lo = frames.iter().map(|f| f[i][1]).fold(f32::MAX, f32::min);
                    let hi = frames.iter().map(|f| f[i][1]).fold(f32::MIN, f32::max);
                    hi - lo
                })
                .collect();
            ex.sort_by(f32::total_cmp);
            let mx = |f: &Vec<[f32; 2]>| f.iter().map(|q| q[0]).sum::<f32>() / f.len() as f32;
            let net = mx(frames.last().expect("frames")) - mx(&frames[0]);
            println!(
                "{draft:6.2} {speed:5.2} {drag:7.2} {waves:5.0} | {bar:6.2} | {fnum:2} {fv:9.4} (sup {sn:2} {sv:6.3}) | {:6.3} ({:4.2}) | {net:7.3}",
                ex[n / 2],
                ex[n / 2] / height
            );
        }
    }
}

/// SONDA — **a boia ENCAIXA na cava?** A varredura que escolhe o arrasto do mar.
///
/// ⚠️ **A lei da armadilha:** a boia escorrega para a cava até o empurrão em declive igualar
/// o arrasto. Ela ENCAIXA se existir um declive onde isso acontece à velocidade da onda, ou
/// seja se `densidade · declive_máximo ≥ arrasto · velocidade`. Encaixada, a excursão
/// vertical dela é ZERO e a horizontal é a onda inteira — que foi o que se mediu na 1.ª
/// versão (`0,0056` contra `4,92`).
///
/// ⚠️ **E o espectro multiplica o declive pelo número de camadas** (cada oitava tem metade da
/// amplitude e metade do comprimento ⇒ o MESMO declive), então a fileira de 4 ondas precisa
/// de ~4× o arrasto da de 1 — e é ela que manda.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn measure_the_trapping_sweep() {
    // A altura da vaga, que é o que a excursão vertical tem de reproduzir.
    let (_, lambda, ..) = sea_authored();
    let height = 2.0 * lambda * 0.1;
    println!("altura da vaga = {height:.4}");
    println!(
        "densidade grav arrasto ondas | limiar | vertical (x altura) deriva_liquida submersao"
    );
    for (dens, grav) in [(12.0_f32, 4.0_f32), (6.0, 2.0)] {
        for drag in [6.0_f32, 11.0, 12.8, 16.7, 20.0] {
            for waves in [1.0_f32, 4.0] {
                // O limiar da armadilha, pela lei: `densidade · declive_max · inv_len / vel`.
                let slope = waves * std::f32::consts::TAU * 0.1;
                let bar = dens * slope / (1.0 + slope * slope).sqrt();
                let (v, net, s) = one_sea(dens, grav, drag, 1.0, waves);
                println!(
                    "{dens:8.1} {grav:4.1} {drag:7.1} {waves:5.1} | {bar:6.2} | {v:8.4} ({:4.2}) {net:13.4} {s:10.4}",
                    v / height
                );
            }
        }
    }
}
