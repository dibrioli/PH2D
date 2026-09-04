//! SONDA (2026-09-04) — report do Enio: *"se arrasto um ponto que esta' dentro do stroke para perto
//! ou sobre o stroke externo, algumas areas de preenchimento somem"*.
//!
//! Varre a quina de um quadrado ao longo da diagonal, atravessando a parede de um circulo, e mede
//! quantas faces limitadas a rede devolve em cada posicao.

use ph2d_vec_fill::rede;
use ph2d_vec_scene::{VecVertex, VertexKind};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

/// Um circulo de raio `r` centrado na origem, em quatro cubicas.
fn circulo(r: f64) -> (Vec<VecVertex>, bool) {
    let k = 0.552_284_749_830_793_4 * r;
    let p = [[r, 0.0], [0.0, r], [-r, 0.0], [0.0, -r]];
    let t = [[0.0, k], [-k, 0.0], [0.0, -k], [k, 0.0]];
    let mut out = Vec::new();
    for i in 0..4 {
        out.push(VecVertex {
            anchor: p[i],
            in_handle: [p[i][0] - t[i][0], p[i][1] - t[i][1]],
            out_handle: [p[i][0] + t[i][0], p[i][1] + t[i][1]],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        });
    }
    (out, true)
}

/// O quadrado cuja quina superior-direita esta' em `(c, c)`.
fn quadrado(c: f64) -> (Vec<VecVertex>, bool) {
    (vec![v(-150.0, -150.0), v(c, -150.0), v(c, c), v(-150.0, c)], true)
}

#[test]
#[ignore = "sonda de diagnostico"]
fn probe_a_quina_atravessa_a_parede() {
    let r: f64 = 100.0;
    let toque = (r * r / 2.0).sqrt(); // 70.7107 — a quina EM CIMA do circulo
    println!("quina no circulo em c = {toque:.4}");
    println!("{:>10} {:>6} {:>6} {:>7} {:>8}  areas", "c", "arcos", "faces", "stubs", "d_quina");
    for passo in -24_i32..=24 {
        let c = toque + f64::from(passo) * 0.25;
        let contornos = vec![circulo(r), quadrado(c)];
        let rd = rede(&contornos);
        let faces: Vec<_> = rd.faces().into_iter().filter(|f| f.area > 0.0).collect();
        let stubs = (0..rd.arcos.len())
            .filter(|&i| rd.arcos[i].de == rd.arcos[i].ate && rd.comprimento(i) < 1.0)
            .count();
        let mut areas: Vec<String> = faces.iter().map(|f| format!("{:.0}", f.area)).collect();
        areas.sort();
        println!(
            "{c:10.4} {:6} {:6} {stubs:7} {:8.4}  {}",
            rd.arcos.len(),
            faces.len(),
            (c * c * 2.0).sqrt() - r,
            areas.join(" ")
        );
    }
}

#[test]
#[ignore = "sonda de diagnostico"]
fn probe_detalhe_da_janela() {
    let r: f64 = 100.0;
    let toque = (r * r / 2.0_f64).sqrt();
    for passo in [-4_i32, -3, -2, -1, 0, 1, 2] {
        let c = toque + f64::from(passo) * 0.2;
        let contornos = vec![circulo(r), quadrado(c)];
        let esc = {
            let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
            for (vs, _) in &contornos {
                for w in vs {
                    lo = [lo[0].min(w.anchor[0]), lo[1].min(w.anchor[1])];
                    hi = [hi[0].max(w.anchor[0]), hi[1].max(w.anchor[1])];
                }
            }
            (hi[0] - lo[0]).hypot(hi[1] - lo[1])
        };
        let x = ph2d_vec_scene::trim_tool::crossings_all(&contornos, esc).expect("cabe");
        let rd = rede(&contornos);
        println!(
            "\nc={c:.4}  d_quina={:+.4}  escala={esc:.2}  merge={:.4}",
            (c * c * 2.0).sqrt() - r,
            esc * 1e-3
        );
        println!("  cruzamentos: circulo={:?} quadrado={:?}", x[0], x[1]);
        println!("  nos: {:?}", rd.nos.iter().map(|n| [n[0].round(), n[1].round()]).collect::<Vec<_>>());
        for (i, a) in rd.arcos.iter().enumerate() {
            println!(
                "  arco {i}: origem={} de={} ate={} faixa=({:.4},{:.4}) comp={:.4}",
                a.origem, a.de, a.ate, a.faixa.0, a.faixa.1, rd.comprimento(i)
            );
        }
        let faces: Vec<_> = rd.faces().into_iter().filter(|f| f.area > 0.0).collect();
        println!("  faces: {:?}", faces.iter().map(|f| f.area.round()).collect::<Vec<_>>());
    }
}

#[test]
#[ignore = "sonda de diagnostico"]
fn probe_ancoras_no_toque() {
    let inicial = rede(&[circulo(100.0), quadrado(70.2107)]);
    let face = inicial.face_em([0.0, 0.0]).expect("a lente");
    println!("ANCORAS gravadas em c=70.2107 (face area {:.0}):", face.area);
    let mut ancoras = Vec::new();
    for &(i, frente) in &face.arcos {
        let a = &inicial.arcos[i];
        println!("  arco {i} origem={} faixa=({:.4},{:.4}) frente={frente}", a.origem, a.faixa.0, a.faixa.1);
        for t in [0.1, 0.35, 0.65, 0.9] {
            ancoras.push((a.origem, a.em(t), frente));
        }
    }
    for c in [70.2107_f64, 70.7107, 70.9607] {
        let r = rede(&[circulo(100.0), quadrado(c)]);
        let faces: Vec<_> = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
        println!("\nc={c:.4}  faces={:?}", faces.iter().map(|f| f.area.round()).collect::<Vec<_>>());
        for (i, a) in r.arcos.iter().enumerate() {
            println!("  arco {i}: origem={} de={} ate={} faixa=({:.4},{:.4})", a.origem, a.de, a.ate, a.faixa.0, a.faixa.1);
        }
        for (k, f) in faces.iter().enumerate() {
            println!("  face {k} area={:.0} ciclo={:?}", f.area, f.arcos);
        }
        for &(origem, frac, frente) in &ancoras {
            let arco = r.arco_em(origem, frac);
            let fi = arco.and_then(|a| r.face_de(&faces, a, frente));
            println!("  ancora(origem={origem}, frac={frac:.4}, frente={frente}) -> arco={arco:?} face={fi:?}");
        }
    }
}

#[test]
#[ignore = "sonda de diagnostico"]
fn probe_passeio_no_toque() {
    let exato = (100.0_f64 * 100.0 / 2.0).sqrt();
    for c in [exato, 70.7107, exato + 1e-4, exato + 1e-2, exato + 0.1, exato + 0.2] {
        let r = rede(&[circulo(100.0), quadrado(c)]);
        println!("\nc={c:.10}  d={:+.3e}  arcos={}", (c * c * 2.0).sqrt() - 100.0, r.arcos.len());
        println!("  nos={:?}", r.nos.iter().map(|n| [(n[0]*1e4).round()/1e4, (n[1]*1e4).round()/1e4]).collect::<Vec<_>>());
        for (i, a) in r.arcos.iter().enumerate() {
            println!("   arco {i}: origem={} de={} ate={} faixa=({:.6},{:.6})", a.origem, a.de, a.ate, a.faixa.0, a.faixa.1);
        }
        for (k, f) in r.faces().iter().enumerate() {
            println!("   ciclo {k}: area={:12.4} {:?}", f.area, f.arcos);
        }
    }
}

/// Duas paredes que cortam uma terceira quase no MESMO sítio: o pedaço entre os dois cortes é um
/// arco-ponto. A fusão de travessias é POR PAR, logo ela não o evita.
#[test]
#[ignore = "sonda de diagnostico"]
fn probe_arco_ponto_por_travessias_de_pares_diferentes() {
    for gap in [1e-2_f64, 1e-6, 1e-10, 1e-14] {
        let contornos = vec![
            (vec![v(0.0, -60.0), v(0.0, 60.0)], false),      // a parede cortada duas vezes
            (vec![v(-60.0, 0.0), v(60.0, 0.0)], false),
            (vec![v(-60.0, gap), v(60.0, gap)], false),
            (vec![v(-60.0, -40.0), v(60.0, -40.0)], false),  // fecha uma regiao la' em baixo
            (vec![v(-40.0, -60.0), v(-40.0, 60.0)], false),
        ];
        let r = rede(&contornos);
        let curtos: Vec<(usize, f64, bool)> = (0..r.arcos.len())
            .map(|i| (i, r.comprimento(i), r.arcos[i].de == r.arcos[i].ate))
            .filter(|(_, c, _)| *c < 1e-3)
            .collect();
        let faces: Vec<_> = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
        println!(
            "gap={gap:.0e}  arcos={}  curtos={curtos:?}  faces={:?}  regiao_baixo={:?}",
            r.arcos.len(),
            faces.iter().map(|f| (f.area * 100.0).round() / 100.0).collect::<Vec<_>>(),
            r.face_em([-20.0, -20.0]).map(|f| (f.area * 100.0).round() / 100.0),
        );
    }
}

/// Custo: a folga apertada faz a rede crescer? (arcos e relogio, N circulos que se cruzam)
#[test]
#[ignore = "sonda de diagnostico"]
fn probe_custo_da_folga_apertada() {
    println!("{:>4} {:>7} {:>8} {:>9}  load", "N", "arcos", "faces", "ms");
    for n in [4_usize, 8, 16, 24, 32] {
        let mut cs = Vec::new();
        for i in 0..n {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            let (c, s) = (a.cos() * 60.0, a.sin() * 60.0);
            let (verts, closed) = circulo(80.0);
            cs.push((
                verts
                    .into_iter()
                    .map(|mut v| {
                        for p in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
                            p[0] += c;
                            p[1] += s;
                        }
                        v
                    })
                    .collect::<Vec<_>>(),
                closed,
            ));
        }
        let t = std::time::Instant::now();
        let r = rede(&cs);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        let faces = r.faces().into_iter().filter(|f| f.area > 0.0).count();
        let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
        println!(
            "{n:>4} {:>7} {faces:>8} {ms:>9.2}  {}",
            r.arcos.len(),
            load.split_whitespace().next().unwrap_or("?")
        );
    }
}
