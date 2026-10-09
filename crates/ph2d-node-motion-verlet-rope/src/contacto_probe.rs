//! ⭐ **A RODADA do Bug #16** (doc 125 §1.2) — a fixtura `rope_thickness` (`24` nós, raio `0,5`,
//! a cabeça a `3,0·sin(2πt/0,6)`, `240` quadros a `60 Hz`) com as peças a declarar o colisor, em
//! cada variante de ONDE o contacto corre, no MESMO processo. As réguas são posições (nenhum
//! relógio, salvo o custo no tecto, que imprime a carga ao lado).
//!
//! `cargo test -p ph2d-node-motion-verlet-rope --release --lib -- --ignored --nocapture sonda_`

use super::contacto::{Contacto, Ritmo, sub_passos};
use super::{Params, simulate};
use ph2d_nodegraph::attr::{Column, Stream};

const NOS: usize = 24;
const RAIO: f32 = 0.5;
const TIQUES: usize = 240;
/// Pares a menos disto na corrente são vizinhos de repouso (`4 · 6/23 > 1,0`).
const VIZINHOS: usize = 3;

fn params(count: usize, length: f32, substeps: usize) -> Params {
    Params {
        count,
        seg_rest: length / (count as f32 - 1.0),
        rest: None,
        gravity: 9.8,
        iterations: 16,
        damping: 0.02,
        pin_tail: false,
        bend: 0.0,
        substeps,
    }
}

fn forma(raio: f32) -> Stream {
    Stream::new(1).with("collider", Column::Scalar(vec![raio]))
}

fn vec2(s: &Stream, nome: &str) -> Vec<[f32; 2]> {
    match s.get(nome) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn par_distante(p: &[[f32; 2]]) -> f32 {
    let mut m = f32::MAX;
    for i in 0..p.len() {
        for j in (i + VIZINHOS + 1)..p.len() {
            m = m.min((p[i][0] - p[j][0]).hypot(p[i][1] - p[j][1]));
        }
    }
    m
}

fn arco(p: &[[f32; 2]]) -> f32 {
    p.windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .sum()
}

/// `(pior distante na 2.ª metade, no fim, arco no fim, maior arco na 2.ª metade, maior |x| da
/// cauda na 2.ª metade, sub-passos corridos)`.
fn marcha(p: &Params, ritmo: Option<Ritmo>) -> [f32; 6] {
    marcha_com(p, ritmo, RAIO).0
}

/// [`marcha`] com o raio dado; devolve também `(o par pior, o maior pedido de sub-passos SEM
/// tecto)`. Os pares ligados no repouso (`< 2r`) ficam de fora, como no filtro.
fn marcha_com(p: &Params, ritmo: Option<Ritmo>, raio: f32) -> ([f32; 6], (usize, usize), usize) {
    let rest = super::seed([0.0, 0.0], p).0;
    let ligado = |i: usize, j: usize| {
        (rest[i][0] - rest[j][0]).hypot(rest[i][1] - rest[j][1]) < 2.0 * raio - 1e-4
    };
    let (mut par, mut pedido_max) = ((0, 0), 0usize);
    let contacto = ritmo.and_then(|r| {
        let mut c = Contacto::de(&forma(raio), p.count, rest.clone(), 1.0)?;
        c.ritmo = r;
        Some(c)
    });
    let mut state = Stream::new(0);
    let mut r = [f32::MAX, 0.0, 0.0, 0.0, 0.0, 0.0];
    for k in 0..TIQUES {
        let t = k as f32 / 60.0;
        let anchor = [3.0 * (std::f32::consts::TAU * t / 0.6).sin(), 0.0];
        let (pos, prev) = (vec2(&state, "P"), vec2(&state, "rope_prev"));
        if pos.len() == p.count {
            r[5] += sub_passos(&pos, &prev, anchor, p, contacto.as_ref()) as f32;
            if let Some(c) = &contacto {
                let mut d = (anchor[0] - pos[0][0]).hypot(anchor[1] - pos[0][1]);
                for (a, b) in pos.iter().zip(&prev) {
                    d = d.max((a[0] - b[0]).hypot(a[1] - b[1]));
                }
                pedido_max = pedido_max.max(c.sub_passos(1, d, 1 << 20));
            }
        }
        state = simulate(anchor, &state, t, p, contacto.as_ref());
        let pos = vec2(&state, "P");
        if k >= TIQUES / 2 {
            for i in 0..pos.len() {
                for j in (i + 1)..pos.len() {
                    let d = (pos[i][0] - pos[j][0]).hypot(pos[i][1] - pos[j][1]);
                    if !ligado(i, j) && d < r[0] {
                        r[0] = d;
                        par = (i, j);
                    }
                }
            }
            r[3] = r[3].max(arco(&pos));
            r[4] = r[4].max(pos[p.count - 1][0].abs());
        }
        r[1] = par_distante(&pos);
        r[2] = arco(&pos);
    }
    (r, par, pedido_max)
}

#[test]
#[ignore = "sonda: a rodada do Bug #16 — corra à mão com --nocapture"]
fn sonda_onde_o_contacto_corre() {
    println!(
        "variante                         | 2a metade | fim    | arco fim | arco max | cauda |x| | sub-passos"
    );
    let linha = |nome: &str, r: [f32; 6]| {
        println!(
            "{nome:32} | {:9.4} | {:6.4} | {:8.3} | {:8.3} | {:9.3} | {:5.0}",
            r[0], r[1], r[2], r[3], r[4], r[5]
        );
    };
    for n in [1usize, 2, 4, 8] {
        let p = params(NOS, 6.0, n);
        linha(&format!("nua n={n}"), marcha(&p, None));
        linha(
            &format!("por sub-passo n={n}"),
            marcha(&p, Some(Ritmo::PorSubPasso)),
        );
        linha(
            &format!("por iteracao n={n}"),
            marcha(&p, Some(Ritmo::PorIteracao)),
        );
    }
    let p = params(NOS, 6.0, 1);
    for k in [1.0f32, 0.5, 0.25] {
        linha(&format!("CFL k={k}"), marcha(&p, Some(Ritmo::Cfl(k))));
    }
    for k in [1.0f32, 0.5] {
        linha(
            &format!("maxSpeed k={k}"),
            marcha(&p, Some(Ritmo::VelocidadeMax(k))),
        );
    }
    // ⭐ A corda FINA (doc 125 §1.3): o raio abaixo de meio segmento, e a âncora a saltar o tique
    // inteiro ou a andar ao longo dos sub-passos.
    for raio in [0.5f32, 0.2, 0.12] {
        for (nome, ritmo) in [
            ("nua", None),
            ("CFL ancora inteira", Some(Ritmo::CflAncoraInteira(0.5))),
            ("CFL ancora anda", Some(Ritmo::Cfl(0.5))),
        ] {
            let (r, par, pedido) = marcha_com(&p, ritmo, raio);
            println!(
                "raio {raio:.2} {nome:20} | pior solto 2a metade {:.4} (par {par:?}) · D {:.2} · arco max {:.3} · sub-passos {:.0} · pedido max {pedido}",
                r[0],
                2.0 * raio,
                r[3],
                r[5]
            );
        }
    }
}

/// O custo de UM tique no tecto, nua contra com contacto (o mínimo de cinco, intercalados). A
/// geometria por nó é a da fixtura (segmento `6/23`, raio `0,5`) e a DENSA (o comprimento `6,0`
/// com todos os nós — cada nó sobrepõe milhares de vizinhos de repouso).
#[test]
#[ignore = "sonda: o custo no tecto — corra em --release"]
fn sonda_o_custo_no_tecto() {
    println!(
        "load: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for (nome, n, comprimento) in [
        ("fixtura", 32_768usize, 6.0 / 23.0 * 32_767.0),
        ("densa 1024", 1_024, 6.0),
        ("densa 4096", 4_096, 6.0),
    ] {
        let p = params(n, comprimento, 1);
        let rest = super::seed([0.0, 0.0], &p).0;
        let mut por_iteracao = Contacto::de(&forma(RAIO), n, rest.clone(), 1.0).expect("declarado");
        por_iteracao.ritmo = Ritmo::PorIteracao;
        let lei = Contacto::de(&forma(RAIO), n, rest, 1.0).expect("declarado");
        let semente = simulate([0.0, 0.0], &Stream::new(0), 0.0, &p, None);
        // A âncora a `0,3` (devagar) e a `2,0` (um puxão): o passo seguro pede `2` e `8` sub-passos.
        let variantes: [(&str, Option<&Contacto>, f32); 4] = [
            ("nua", None, 0.3),
            ("por iteracao, 1 sub-passo", Some(&por_iteracao), 0.3),
            ("a lei, ancora a 0,3", Some(&lei), 0.3),
            ("a lei, ancora a 2,0", Some(&lei), 2.0),
        ];
        let mut melhor = [f64::MAX; 4];
        for _ in 0..5 {
            for (v, (_, cc, x)) in variantes.iter().enumerate() {
                let t0 = std::time::Instant::now();
                std::hint::black_box(simulate([*x, 0.0], &semente, 1.0 / 60.0, &p, *cc));
                melhor[v] = melhor[v].min(t0.elapsed().as_secs_f64() * 1000.0);
            }
        }
        for (v, (nome_v, _, _)) in variantes.iter().enumerate() {
            println!("{nome:12} n={n:6} {nome_v:28}: {:10.3} ms", melhor[v]);
        }
    }
}
