//! **B13.12 cura 2, passo 0 (12/11) — fora do produto.** Antes de construir o sono EXATO por tile 64²,
//! mede se o critério é ao bit: «o tile dorme no passo k se no passo k−1 nem ele nem a ilha mudaram um
//! bit, ele não tem vel/flow ≠ 0, o passo k não seca e não houve pincel na ilha». Cena: 6 faixas de
//! 3 000 × 240 px a cada 280 px a 3840×2160 (`Lei::Cobre`, Wet Wash), 5 pintadas no começo e a 6.ª a meio.
//! Em cada passo compara-se TODO campo (fina e fluxo) antes × depois, por tile, e conta-se, por critério
//! (raio da ilha `r`, passos quietos `L`), os tile-passos candidatos e as VIOLAÇÕES (candidato que mudou).
//! Violação > 0 ⇒ dormir esse critério muda o desenho.
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-wet-paint --example sonda_sono_por_tile_4k --profile smoke -- [passos] [w] [h] [lado]
//! ```

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::Engine;

use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// O lado do tile (4.º argumento; 64 = o da `row_tiles`).
static LADO: AtomicUsize = AtomicUsize::new(64);

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

struct Foto {
    f: Vec<Vec<u32>>,
}

fn campos(g: &Grid) -> Vec<(Vec<u32>, bool)> {
    let b = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<u32>>();
    let b3 = |v: &[[f32; 3]]| {
        v.iter()
            .flat_map(|x| x.iter().map(|c| c.to_bits()))
            .collect::<Vec<u32>>()
    };
    // (bits, fluxo?) — os campos de 3 canais entram com 3 palavras por célula.
    vec![
        (b(&g.film), false),
        (b(&g.susp), false),
        (b(&g.sett), false),
        (b3(&g.susp_rgb), false),
        (b3(&g.sett_rgb), false),
        (g.wet.iter().map(|&x| u32::from(x)).collect(), false),
        (b(&g.altura), false),
        (b(&g.espessura), false),
        (b(&g.vela), false),
        (b(&g.vel_x), true),
        (b(&g.vel_y), true),
        (b(&g.flow_x), true),
        (b(&g.flow_y), true),
    ]
}

fn foto(g: &Grid) -> Foto {
    Foto {
        f: campos(g).into_iter().map(|(v, _)| v).collect(),
    }
}

fn tile_de(g: &Grid, fluxo: bool, k: usize, ntx: usize, nty: usize) -> usize {
    let (x, y) = if fluxo {
        let fg = g.flow;
        ((k % fg.s) * fg.rf, (k / fg.s) * fg.rf)
    } else {
        (k % g.s, k / g.s)
    };
    let t = LADO.load(Relaxed);
    (y / t).min(nty - 1) * ntx + (x / t).min(ntx - 1)
}

const NOMES: [&str; 13] = [
    "film",
    "susp",
    "sett",
    "susp_rgb",
    "sett_rgb",
    "wet",
    "altura",
    "espessura",
    "vela",
    "vel_x",
    "vel_y",
    "flow_x",
    "flow_y",
];

/// Os tiles com algum bit diferente entre `a` e o grid agora, e quantos tiles cada campo mudou.
fn mudou_por_campo(a: &Foto, g: &Grid, ntx: usize, nty: usize) -> (Vec<bool>, Vec<usize>) {
    let mut m = vec![false; ntx * nty];
    let mut por_campo = Vec::new();
    for (old, (new, fluxo)) in a.f.iter().zip(campos(g)) {
        let mut mc = vec![false; ntx * nty];
        if old.len() == new.len() && !new.is_empty() {
            let por = new.len() / if fluxo { g.flow.cells } else { g.cells }.max(1);
            let por = por.max(1);
            for (i, (x, y)) in old.iter().zip(&new).enumerate() {
                if x != y {
                    mc[tile_de(g, fluxo, i / por, ntx, nty)] = true;
                }
            }
        }
        por_campo.push(mc.iter().filter(|&&b| b).count());
        for (a, b) in m.iter_mut().zip(mc) {
            *a |= b;
        }
    }
    (m, por_campo)
}

fn mudou(a: &Foto, g: &Grid, ntx: usize, nty: usize) -> Vec<bool> {
    mudou_por_campo(a, g, ntx, nty).0
}

/// Tiles com vel ou flow ≠ 0, e tiles vivos (film > 0 ou fluxo ≠ 0).
fn mexe_e_vivo(g: &Grid, ntx: usize, nty: usize) -> (Vec<bool>, Vec<bool>) {
    let mut mexe = vec![false; ntx * nty];
    let mut vivo = vec![false; ntx * nty];
    for k in 0..g.flow.cells {
        if g.vel_x[k] != 0.0 || g.vel_y[k] != 0.0 || g.flow_x[k] != 0.0 || g.flow_y[k] != 0.0 {
            let t = tile_de(g, true, k, ntx, nty);
            mexe[t] = true;
            vivo[t] = true;
        }
    }
    for (k, f) in g.film.iter().enumerate() {
        if *f > 0.0 {
            vivo[tile_de(g, false, k, ntx, nty)] = true;
        }
    }
    (mexe, vivo)
}

fn ilha(m: &[bool], tx: usize, ty: usize, r: usize, ntx: usize, nty: usize) -> bool {
    let (x0, x1) = (tx.saturating_sub(r), (tx + r).min(ntx - 1));
    let (y0, y1) = (ty.saturating_sub(r), (ty + r).min(nty - 1));
    (y0..=y1).any(|y| (x0..=x1).any(|x| m[y * ntx + x]))
}

fn main() {
    let arg = |i: usize, d: usize| {
        std::env::args()
            .nth(i)
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (passos, w, h) = (arg(1, 120), arg(2, 3840), arg(3, 2160));
    LADO.store(arg(4, 64).max(1), Relaxed);
    let mut e = Engine::new(w, h);
    e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
    e.sim.glaze_filme = ph2d_wet_paint::colorops::VELATURA_DO_FILME;
    e.sim.pickup_assentado = 0.0;
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let esc = w as f64 / 3840.0;
    let (x0, x1) = (420.0 * esc, 3420.0 * esc);
    let cores = [
        [40.0, 70.0, 200.0],
        [200.0, 40.0, 40.0],
        [230.0, 200.0, 30.0],
        [30.0, 150.0, 60.0],
        [120.0, 40.0, 160.0],
        [90.0, 60.0, 40.0],
    ];
    for (i, c) in cores.iter().enumerate().take(5) {
        e.color = *c;
        faixa(&mut e, (60.0 + 280.0 * i as f64) * esc, x0, x1);
    }
    let (ntx, nty) = {
        let g = e.active_grid();
        let t = LADO.load(Relaxed);
        (g.s.div_ceil(t), g.rows.div_ceil(t))
    };
    // Critérios: (raio da ilha, passos quietos seguidos).
    let crit: [(usize, usize); 6] = [(1, 1), (1, 2), (1, 12), (3, 1), (3, 2), (3, 12)];
    let mut cand = [0u64; 6];
    let mut viol = [0u64; 6];
    let mut viol_mexe_vizinho = [0u64; 6];
    let mut vivos = 0u64;
    let mut hist: Vec<Vec<bool>> = Vec::new(); // mudou (passo + pincel) dos passos anteriores
    let meio = passos / 3;
    println!(
        "sono por tile {w}x{h} · {ntx}x{nty} tiles · {passos} passos · 6.ª faixa no passo {meio} · {} fios",
        rayon::current_num_threads()
    );
    for k in 0..passos {
        let antes_pincel = foto(e.active_grid());
        if k == meio {
            e.color = cores[5];
            faixa(&mut e, (60.0 + 280.0 * 5.0) * esc, x0, x1);
        }
        let g = e.active_grid();
        let pincel = mudou(&antes_pincel, g, ntx, nty);
        let (mexe, vivo) = mexe_e_vivo(g, ntx, nty);
        let n = e.sim.frame + 1;
        let seca = n.is_multiple_of(e.sim.dry_every);
        let antes = foto(g);
        e.step_simulation();
        let g = e.active_grid();
        let (m, pc) = mudou_por_campo(&antes, g, ntx, nty);
        if k % 10 == 0 || k % 10 == 1 {
            let txt: Vec<String> = NOMES
                .iter()
                .zip(&pc)
                .filter(|(_, c)| **c > 0)
                .map(|(n, c)| format!("{n} {c}"))
                .collect();
            let nmexe = mexe.iter().filter(|&&b| b).count();
            println!(
                "    tiles mudados por campo: {} · tiles com vel/flow≠0 {nmexe}",
                txt.join(" · ")
            );
        }
        let mut tot = m.clone();
        for (t, p) in tot.iter_mut().zip(&pincel) {
            *t |= p;
        }
        let nm = m.iter().filter(|&&b| b).count();
        let nv = vivo.iter().filter(|&&b| b).count();
        vivos += nv as u64;
        let mut linha = String::new();
        for (c, &(r, l)) in crit.iter().enumerate() {
            if seca || hist.len() < l {
                continue;
            }
            let mut nc = 0;
            for ty in 0..nty {
                for tx in 0..ntx {
                    let t = ty * ntx + tx;
                    if !vivo[t] || mexe[t] || ilha(&pincel, tx, ty, r, ntx, nty) {
                        continue;
                    }
                    if hist[hist.len() - l..]
                        .iter()
                        .any(|hm| ilha(hm, tx, ty, r, ntx, nty))
                    {
                        continue;
                    }
                    nc += 1;
                    cand[c] += 1;
                    if m[t] {
                        viol[c] += 1;
                        if ilha(&mexe, tx, ty, r, ntx, nty) {
                            viol_mexe_vizinho[c] += 1;
                        }
                    }
                }
            }
            linha.push_str(&format!(" r{r}L{l} {nc}"));
        }
        println!(
            "  passo {n:3}{} · vivos {nv} · mudaram {nm} · candidatos{linha}",
            if seca { " seca" } else { "     " }
        );
        hist.push(tot);
    }
    println!("total vivos (tile-passo) {vivos}");
    for (c, &(r, l)) in crit.iter().enumerate() {
        println!(
            "  ilha {}x{} · {l:2} passo(s) quieto(s): candidatos {} ({:.3} dos vivos) · VIOLAÇÕES {} (com vizinho a mexer {})",
            2 * r + 1,
            2 * r + 1,
            cand[c],
            cand[c] as f64 / vivos.max(1) as f64,
            viol[c],
            viol_mexe_vizinho[c]
        );
    }
}
