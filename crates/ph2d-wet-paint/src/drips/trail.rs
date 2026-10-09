//! O RASTO das gotas: a cabeça que desce ([`advance`]), a tinta que sai da orla e assenta no fio, e a
//! contra-gota.

use super::*;

/// As células da coluna curta (`alcance` células) acima da orla `o`, contra a gravidade.
pub(super) fn coluna_acima(
    g: &Grid,
    o: (f64, f64),
    nx: f64,
    ny: f64,
    alcance: i32,
) -> impl Iterator<Item = usize> + '_ {
    (0..=alcance).filter_map(move |k| {
        let (cx, cy) = (
            (o.0 - nx * f64::from(k)).round() as i32,
            (o.1 - ny * f64::from(k)).round() as i32,
        );
        inside(g, cx, cy).then(|| cx as usize + cy as usize * g.s)
    })
}

/// Tira até `quanto` de pigmento da ORLA da bacia (onde ele se juntou), a mesma fração de cada célula
/// (assente e suspenso), e devolve-o com a cor dele — a massa SAI da orla (conserva-se: vai para o fio).
/// A orla inteira é a reserva: uma coluna estreita esgotava-se e o fio sumia aos 31 px (medido).
pub(super) fn tira_da_orla(g: &mut Grid, orla: &[usize], quanto: f64) -> (f64, [f64; 3]) {
    let total: f64 = orla
        .iter()
        .map(|&i| f64::from(g.sett[i]) + f64::from(g.susp[i]))
        .sum();
    if total <= 0.0 || quanto <= 0.0 {
        return (0.0, [0.0; 3]);
    }
    let fr = (quanto / total).min(1.0);
    let (mut tirado, mut rgb) = (0.0f64, [0.0f64; 3]);
    for &i in orla {
        for assente in [true, false] {
            let (massa, cor) = if assente {
                (f64::from(g.sett[i]), g.sett_rgb[i])
            } else {
                (f64::from(g.susp[i]), g.susp_rgb[i])
            };
            let dm = massa * fr;
            if dm <= 0.0 {
                continue;
            }
            let w = dm / (tirado + dm);
            for (k, ch) in rgb.iter_mut().enumerate() {
                *ch += (f64::from(cor[k]) - *ch) * w;
            }
            tirado += dm;
            if assente {
                g.sett[i] = (massa - dm) as f32;
            } else {
                g.susp[i] = (massa - dm) as f32;
            }
        }
    }
    (tirado, rgb)
}

/// A cabeça desce um passo e assenta o rasto (largura de funil: `wmax · (falta / total)²`). Quando o
/// rasto ou a água acabam, o resto do pigmento assenta ali (a contra-gota no fim do fio).
pub(super) fn advance(
    g: &mut Grid,
    p: &Params,
    c: &Cells,
    d: &mut Drip,
    nx: f64,
    ny: f64,
    mag: f64,
    paper: DripPaper,
    orla: &[usize],
) {
    let wmax = c.c(WIDTH_K * f64::from(paper.size) * f64::from(paper.size));
    let run = d.run;
    let piso = c.c(c.law.trail_min_px).max(1.0);
    let largura = |falta: f64| (wmax * (falta / run).powi(2)).max(piso).round() as i32;
    let w = largura(d.run_left);
    // O papel que segura menos (`easy`) também absorve menos o fio, na mesma proporção.
    let pelicula = if paper.easy {
        TRAIL_FILM * c.law.hold_easy / c.law.hold
    } else {
        TRAIL_FILM
    };
    if d.water < pelicula {
        // Ainda não bebeu o bastante para a próxima célula: a cabeça espera pela bacia.
        return;
    }
    let v = c.c((SPEED * mag).min(1.0));
    d.x += nx * v;
    d.y += ny * v;
    // O fio desvia-se um pouco (a água procura o caminho no papel): uma onda lenta e pequena, com a fase
    // da gota — determinista (o hash da origem), sem RNG.
    let andou = (d.run - d.run_left) * c.px;
    let fase = crate::rng::hash2(d.ox as i32, d.oy as i32, MEANDER_SEED);
    let desvio =
        c.c(MEANDER_PX) * libm::sin(2.0 * std::f64::consts::PI * (andou / MEANDER_WAVE_PX + fase));
    let (hx, hy) = (d.x - ny * desvio, d.y + nx * desvio);
    let (cx, cy) = (hx.round() as i32, hy.round() as i32);
    if !inside(g, cx, cy) {
        d.run_left = 0.0;
        return;
    }
    if (cx, cy) == d.cell {
        return;
    }
    d.cell = (cx, cy);
    let i = cx as usize + cy as usize * g.s;
    if g.film[i] as f64 > DRY_FILM {
        if d.ox.round() as i32 != cx || cy > d.oy.round() as i32 + 1 {
            // Chegou a outra água (não a da própria bacia): junta-se a ela.
            merge_into(g, i, d.water);
            d.water = 0.0;
            d.run_left = 0.0;
        }
        return;
    }
    // A mesma tinta por CÉLULA ao longo do rasto (`tint`), tirada da orla e do traço acima dela fileira a
    // fileira: quando ele se esgota o fio clareia (nunca para à espera). O pescoço: onde a gota sai da
    // orla o fio é mais largo e mais ESCURO (o funil do Rebelle: a raiz ~2× o fio) e afina em `NECK_PX`.
    let pescoco = (1.0 - andou / NECK_PX).max(0.0);
    let w = w.max(c.c(NECK_W * pescoco).round() as i32);
    // A reserva: a orla E o traço logo acima dela (com o papel que segura menos a gota nasce antes de o
    // pigmento descer à orla: só da orla, o fio saía com 16–22 de escuro contra 76–116 — medido).
    let alcance = c.c(TINT_REACH_PX).round() as i32;
    let reserva: Vec<usize> = orla
        .iter()
        .flat_map(|&i| {
            let (x, y) = ((i % g.s) as f64, (i / g.s) as f64);
            coluna_acima(g, (x, y), nx, ny, alcance).collect::<Vec<_>>()
        })
        .collect();
    let funil = 1.0 + c.law.neck_dark * pescoco;
    let (dm, rgb) = tira_da_orla(g, &reserva, funil * d.tint * (f64::from(w) + SOFT_EDGE));
    d.run_left = (d.run_left - 1.0).max(0.0);
    let sub = (hx - f64::from(cx)) * -ny + (hy - f64::from(cy)) * nx;
    settle(g, p, cx, cy, sub, w, nx, ny, dm, rgb);
    d.water -= pelicula;
    if d.run_left == 0.0 {
        // A contra-gota: uma pequena gota redonda na ponta do fio.
        let (resto, rgb) = tira_da_orla(g, &reserva, d.tint * BEAD_CELLS);
        bead(g, p, cx, cy, resto, rgb);
        d.water = 0.0;
    }
}

/// A contra-gota: `mass` num disco de raio [`BEAD_R`] em volta de `(cx, cy)` (mais no centro).
pub(super) fn bead(g: &mut Grid, p: &Params, cx: i32, cy: i32, mass: f64, rgb: [f64; 3]) {
    if mass <= 0.0 {
        return;
    }
    let r = BEAD_R;
    let pesos: Vec<(i32, i32, f64)> = (-r..=r)
        .flat_map(|dy| (-r..=r).map(move |dx| (dx, dy)))
        .filter_map(|(dx, dy)| {
            let q = f64::from(dx * dx + dy * dy) / f64::from(r * r + 1);
            (q <= 1.0).then_some((dx, dy, 1.0 - q))
        })
        .collect();
    let soma: f64 = pesos.iter().map(|w| w.2).sum();
    for (dx, dy, w) in pesos {
        settle_cell(g, p, cx + dx, cy + dy, mass * w / soma, rgb);
    }
}

/// Assenta `mass` numa célula (a mistura de cor do assente, e o sujo).
pub(super) fn settle_cell(g: &mut Grid, p: &Params, x: i32, y: i32, mass: f64, rgb: [f64; 3]) {
    if mass <= 0.0 || !inside(g, x, y) {
        return;
    }
    let i = x as usize + y as usize * g.s;
    let old = g.sett[i] as f64;
    let c = g.sett_rgb[i];
    let mut out = [0.0f64; 3];
    p.mix.mix(
        f64::from(c[0]),
        f64::from(c[1]),
        f64::from(c[2]),
        rgb[0],
        rgb[1],
        rgb[2],
        mass / (old + mass),
        &mut out,
    );
    g.sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
    g.sett[i] = (old + mass) as f32;
    note_dirty(g, x, y);
}

/// Assenta `mass` de pigmento numa fileira transversal centrada em `(cx, cy) + sub`, `w` células cheias
/// e a descida suave de [`SOFT_EDGE`] (a borda do fio é macia e anti-serrilhada, não um risco de régua).
pub(super) fn settle(
    g: &mut Grid,
    p: &Params,
    cx: i32,
    cy: i32,
    sub: f64,
    w: i32,
    nx: f64,
    ny: f64,
    mass: f64,
    rgb: [f64; 3],
) {
    if mass <= 0.0 {
        return;
    }
    // O perfil: 1 no miolo (meia-largura `w/2`), a descer suavemente a 0 em `SOFT_EDGE` células, centrado
    // em `sub` (a posição transversal da cabeça dentro da célula). Normalizado ao total `mass`.
    let half = f64::from(w) / 2.0;
    let reach = (half + SOFT_EDGE).ceil() as i32 + 1;
    let peso = |k: i32| -> f64 {
        let dist = (f64::from(k) - sub).abs();
        let r = (dist - half + SOFT_EDGE / 2.0) / SOFT_EDGE;
        if r <= 0.0 {
            1.0
        } else if r >= 1.0 {
            0.0
        } else {
            1.0 - r * r * (3.0 - 2.0 * r)
        }
    };
    let soma: f64 = (-reach..=reach).map(peso).sum();
    if soma <= 0.0 {
        return;
    }
    for k in -reach..=reach {
        let wk = peso(k);
        if wk <= 0.0 {
            continue;
        }
        let t = f64::from(k);
        let (x, y) = (
            (f64::from(cx) - ny * t).round() as i32,
            (f64::from(cy) + nx * t).round() as i32,
        );
        settle_cell(g, p, x, y, mass * wk / soma, rgb);
    }
}

pub(super) fn merge_into(g: &mut Grid, i: usize, water: f64) {
    g.film[i] = (g.film[i] as f64 + water) as f32;
}

pub(super) fn note_dirty(g: &mut Grid, x: i32, y: i32) {
    g.drip_dirty = Some(match g.drip_dirty {
        None => [x, y, x, y],
        Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x), y1.max(y)],
    });
}
