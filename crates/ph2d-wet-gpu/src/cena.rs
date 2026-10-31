//! Cenas de teste e de medida — partilhadas pelo gate de paridade e pelo `mede_advect`.

use ph2d_wet_paint::grid::Grid;

/// Uma poça com três manchas e um redemoinho + cisalhamento no fluxo, cobrindo a folha
/// inteira (`active = 1`, bbox = folha, faixas desligadas). `|uv| ≤ amp`.
#[must_use]
pub fn poca(w: usize, h: usize, rf: usize, amp: f32) -> Grid {
    let mut g = Grid::with_flow_ratio(w, h, rf);
    let (wf, hf) = (w as f32, h as f32);
    let manchas = [
        (0.3f32, 0.4f32, 0.18f32),
        (0.62, 0.55, 0.22),
        (0.45, 0.75, 0.12),
    ];
    for y in 1..=h {
        for x in 1..=w {
            let i = x + y * g.s;
            let (nx, ny) = (x as f32 / wf, y as f32 / hf);
            let mut f = 0.05f32;
            for (cx, cy, r) in manchas {
                let d2 = ((nx - cx).powi(2) + (ny - cy).powi(2)) / (r * r);
                f += (-d2).exp();
            }
            g.film[i] = f;
            g.susp[i] = 0.3 * f * (0.5 + 0.5 * (7.0 * nx).sin().abs());
            g.susp_rgb[i] = [nx, ny, 1.0 - 0.5 * (nx + ny)];
            g.active[i] = 1;
        }
    }
    let fl = g.flow;
    for cy in 1..=fl.h {
        for cx in 1..=fl.w {
            let (nx, ny) = (cx as f32 / fl.w as f32, cy as f32 / fl.h as f32);
            let (dx, dy) = (nx - 0.5, ny - 0.5);
            let i = cx + cy * fl.s;
            g.flow_x[i] = amp * (-dy * 1.6 + 0.35 * (9.0 * ny).sin()).clamp(-1.0, 1.0);
            g.flow_y[i] = amp * (dx * 1.6 + 0.35 * (11.0 * nx).cos()).clamp(-1.0, 1.0);
        }
    }
    g.vel_x.clone_from(&g.flow_x);
    g.vel_y.clone_from(&g.flow_y);
    g.bx0 = 1;
    g.by0 = 1;
    g.bx1 = w as i32;
    g.by1 = h as i32;
    g.has_fluid = true;
    g.spans_enabled = false;
    g
}

/// [`poca`] com papel de dente, pigmento assentado, `wet` semeado e uma mancha FUNDA
/// (`film > 3`: o carimbo de umidade dispara) — exercita capilar, freio e carimbo.
#[must_use]
pub fn poca_papel(w: usize, h: usize, rf: usize, amp: f32) -> Grid {
    let mut g = poca(w, h, rf, amp);
    let (wf, hf) = (w as f32, h as f32);
    for y in 1..=h {
        for x in 1..=w {
            let i = x + y * g.s;
            let (nx, ny) = (x as f32 / wf, y as f32 / hf);
            g.paper[i] = 0.5 + 0.25 * (31.0 * nx).sin() * (23.0 * ny).cos();
            g.sett[i] = 0.02 * (1.0 + (13.0 * nx + 5.0 * ny).sin());
            let d2 = ((nx - 0.3).powi(2) + (ny - 0.4).powi(2)) / 0.01;
            g.film[i] += 3.5 * (-d2).exp();
            if (x + 2 * y) % 3 == 0 {
                g.wet[i] = ((x * 7 + y * 13) % 256) as u8;
            }
        }
    }
    g
}

/// A grade espelhada em x: célula `x → w+1−x`, fluxo `ux → −ux` (exige `rf = 1`).
#[must_use]
pub fn espelha_x(g: &Grid) -> Grid {
    assert_eq!(
        g.flow.rf, 1,
        "o espelho exato da amostragem do fluxo exige rf = 1"
    );
    let mut m = g.clone();
    for y in 0..g.rows {
        for x in 1..=g.w {
            let (i, j) = (x + y * g.s, (g.w + 1 - x) + y * g.s);
            m.film[j] = g.film[i];
            m.susp[j] = g.susp[i];
            m.susp_rgb[j] = g.susp_rgb[i];
            m.active[j] = g.active[i];
            m.flow_x[j] = -g.flow_x[i];
            m.flow_y[j] = g.flow_y[i];
            m.vel_x[j] = -g.vel_x[i];
            m.vel_y[j] = g.vel_y[i];
            m.paper[j] = g.paper[i];
            m.sett[j] = g.sett[i];
            m.wet[j] = g.wet[i];
        }
    }
    m
}
