//! **B13.12 cura 3 (12/11) — fora do produto.** O passo do Wet Wash a 4K com 1..6 traços, ms por
//! estágio 0..6 (mínimo e mediana), sim-s/s = min(1, 25/mediana) e o hash de todos os campos no fim.
//! Cena: as 6 faixas de 3 000 × 240 px a cada 280 px de `sonda_sono_por_tile_4k` (`Lei::Cobre`).
//! Variantes A×B: `PH2D_SONDA_ROTAS` = lista de rotas separadas por vírgula (`a` = o produto, `b` = a cura
//! da vez), corridas intercaladas sobre o MESMO estado inicial (clone do grid).
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-wet-paint --example sonda_estagios_4k --profile smoke -- [traços] [passos] [rondas]
//! ```

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::Engine;
use std::hash::{Hash, Hasher};
use std::time::Instant;

fn faixa(e: &mut Engine, y0: f64, x0: f64, x1: f64) {
    let mut y = y0;
    while y < y0 + 240.0 {
        e.pointer_down(x0, y, None);
        let mut x = x0;
        while x < x1 {
            x = (x + 16.0).min(x1);
            e.dispatch_pressure_dab(x, y, 6.0, 60.0);
        }
        e.pointer_up();
        while e.release_frame(x, y) {}
        y += 12.0;
    }
}

fn hash(g: &Grid) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for v in [
        &g.film,
        &g.susp,
        &g.sett,
        &g.altura,
        &g.espessura,
        &g.vela,
        &g.vel_x,
        &g.vel_y,
        &g.flow_x,
        &g.flow_y,
    ] {
        v.iter().for_each(|x| x.to_bits().hash(&mut h));
    }
    for v in [&g.susp_rgb, &g.sett_rgb] {
        v.iter().flatten().for_each(|x| x.to_bits().hash(&mut h));
    }
    g.wet.hash(&mut h);
    h.finish()
}

/// A rota da variante: `a` = o produto · `fNN` = `BLOCO_LINHAS` · `sNN` = `SAIDA_BLOCO` · `x` = fases ·
/// `m` = o momento no bloco · `cN` = `COMMIT_ROTA` N (1 = cópia, o de antes da cura 3; 2 = troca).
fn rota(r: &str) {
    use ph2d_wet_paint::solver::{ADVECT_FASES, MOMENTO_FORA};
    use ph2d_wet_paint::solver::{BLOCO_LINHAS, SAIDA_BLOCO};
    use std::sync::atomic::Ordering::Relaxed;
    BLOCO_LINHAS.store(16, Relaxed);
    SAIDA_BLOCO.store(16, Relaxed);
    ADVECT_FASES.store(0, Relaxed);
    MOMENTO_FORA.store(1, Relaxed);
    ph2d_wet_paint::flow::COMMIT_ROTA.store(0, Relaxed);
    let n = r.get(1..).and_then(|v| v.parse().ok());
    match (r.as_bytes().first(), n) {
        (Some(b'f'), Some(n)) => BLOCO_LINHAS.store(n, Relaxed),
        (Some(b's'), Some(n)) => SAIDA_BLOCO.store(n, Relaxed),
        (Some(b'x'), _) => ADVECT_FASES.store(1, Relaxed),
        (Some(b'm'), _) => MOMENTO_FORA.store(0, Relaxed),
        (Some(b'c'), Some(n)) => {
            ph2d_wet_paint::flow::COMMIT_ROTA.store(u8::try_from(n).unwrap_or(0), Relaxed)
        }
        _ => {}
    }
}

fn main() {
    let arg = |i: usize, d: usize| {
        std::env::args()
            .nth(i)
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (tracos, passos, rondas) = (arg(1, 6).clamp(1, 6), arg(2, 24), arg(3, 3));
    let rotas: Vec<String> = std::env::var("PH2D_SONDA_ROTAS")
        .unwrap_or_else(|_| "a".into())
        .split(',')
        .map(str::to_owned)
        .collect();
    let (w, h) = (3840, 2160);
    let mut e = Engine::new(w, h);
    e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
    e.sim.glaze_filme = ph2d_wet_paint::colorops::VELATURA_DO_FILME;
    e.sim.pickup_assentado = 0.0;
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let cores = [
        [40.0, 70.0, 200.0],
        [200.0, 40.0, 40.0],
        [230.0, 200.0, 30.0],
        [30.0, 150.0, 60.0],
        [120.0, 40.0, 160.0],
        [90.0, 60.0, 40.0],
    ];
    for (i, c) in cores.iter().enumerate().take(tracos) {
        e.color = *c;
        faixa(&mut e, 60.0 + 280.0 * i as f64, 420.0, 3420.0);
    }
    // Aquece: a água começa a correr e mistura (o estado do defeito).
    for _ in 0..12 {
        e.step_simulation();
    }
    let g0 = e.active_grid().clone();
    let (sim0, frame0) = (e.sim.dry_every, e.sim.frame);
    println!(
        "estágios 4K · {tracos} traço(s) · {passos} passos × {rondas} rondas · {} fios · rotas {rotas:?}",
        rayon::current_num_threads()
    );
    let mut ms: Vec<Vec<f64>> = vec![Vec::new(); rotas.len()];
    let mut st: Vec<Vec<[f64; 7]>> = vec![Vec::new(); rotas.len()];
    let mut hs: Vec<u64> = vec![0; rotas.len()];
    for ronda in 0..rondas {
        for k in 0..rotas.len() {
            let j = (k + ronda) % rotas.len();
            rota(&rotas[j]);
            *e.active_grid_mut() = g0.clone();
            e.sim.dry_every = sim0;
            e.sim.frame = frame0;
            for _ in 0..passos {
                let mut por = [0.0f64; 7];
                let t0 = Instant::now();
                let mut s = 0usize;
                loop {
                    let t = Instant::now();
                    let fim = e.step_stage();
                    por[s.min(6)] += t.elapsed().as_secs_f64() * 1e3;
                    s += 1;
                    if fim.is_some() {
                        break;
                    }
                }
                ms[j].push(t0.elapsed().as_secs_f64() * 1e3);
                st[j].push(por);
            }
            hs[j] = hash(e.active_grid());
        }
    }
    rota("a");
    // O estágio 3 por partes, sobre clones do estado inicial (mín de `rondas`).
    let p = e.sim.gather_params(&e.tuning);
    let [gx, gy] = e.sim.gravity(&e.tuning);
    let (mut ta, mut ts, mut tr) = (f64::MAX, f64::MAX, f64::MAX);
    for _ in 0..rondas.max(3) {
        let mut g = g0.clone();
        let t = Instant::now();
        ph2d_wet_paint::solver::advect_jacobi(&mut g, &p, gx, gy);
        ta = ta.min(t.elapsed().as_secs_f64() * 1e3);
        let t = Instant::now();
        if p.sedimenta > 0.0 {
            let a = p.sedimenta * (gx * gx + gy * gy).sqrt();
            ph2d_wet_paint::solver::sedimenta(&mut g, a, gx, gy, p.km_mixing);
        }
        ts = ts.min(t.elapsed().as_secs_f64() * 1e3);
        let t = Instant::now();
        ph2d_wet_paint::estreito::arredonda(&mut g);
        tr = tr.min(t.elapsed().as_secs_f64() * 1e3);
    }
    println!(
        "  estágio 3 por partes (mín): advect {ta:.1} · sedimenta {ts:.1} (a={}) · arredonda {tr:.1} ms",
        p.sedimenta
    );
    for (j, r) in rotas.iter().enumerate() {
        let mut v = ms[j].clone();
        v.sort_by(f64::total_cmp);
        let (mn, md) = (v[0], v[v.len() / 2]);
        let est: Vec<String> = (0..7)
            .map(|s| {
                let mut x: Vec<f64> = st[j].iter().map(|p| p[s]).collect();
                x.sort_by(f64::total_cmp);
                format!("{:.1}/{:.1}", x[0], x[x.len() / 2])
            })
            .collect();
        println!(
            "  rota {r}: passo mín {mn:.1} · mediana {md:.1} ms · sim-s/s {:.2} · estágios mín/med [{}] · hash {:016x}",
            (25.0 / md).min(1.0),
            est.join(" · "),
            hs[j]
        );
    }
}
