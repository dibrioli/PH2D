//! **A água ARRASTA o que levantou** ([`apply_water_drag`], [`Knob::WaterDrag`], 31/10, o Water Brush) — filho de
//! [`super`] por LOC.

use super::*;

/// ⭐ **A água leva o SUSPENSO pelo traço** ([`Knob::WaterDrag`]): cada célula do carimbo manda a fracção
/// `u = min(1, carimbo × WaterDrag)` do seu pigmento suspenso para `+ (dab − prev)`, repartida bilinear — a cor esbate
/// na direcção do gesto e a origem clareia (o rasto decai como `(1 − u)` por dab). TRANSPORTE, nunca mistura: a massa
/// conserva-se (o Smear interpola e duplica a borda de trás — com o `WetLift` a repor o suspenso da tinta seca, `+35 %`
/// de pigmento a `0,8`, medido). E nenhuma célula passa do pigmento total (suspenso + assentado) que ela ou a origem
/// já tinham: o resto fica para trás — sem o anel escuro de água a mais (R1 §2). Só o suspenso e nunca o papel: o
/// assentado chega pela porta do `WetLift`. `0` = nada a fazer, ao bit.
pub fn apply_water_drag(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    prev_x: f64,
    prev_y: f64,
) -> Option<TouchedRect> {
    // F4-2 E6b: a soma segue a massa (a caixa cobre a origem e o destino `+ (dab − prev)`).
    let raio = dab.r + (dab.x - prev_x).hypot(dab.y - prev_y) + 2.0;
    super::tinta::com_foto(g, p, (dab.x, dab.y, raio), |g| {
        arrasta(g, p, tex, dab, prev_x, prev_y)
    })
}

/// O corpo de [`apply_water_drag`].
fn arrasta(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    prev_x: f64,
    prev_y: f64,
) -> Option<TouchedRect> {
    let force = p.k(Knob::WaterDrag);
    let (dx, dy) = (dab.x - prev_x, dab.y - prev_y);
    if force <= 0.0 || dx.hypot(dy) < 1e-6 || dab.r <= 0.0 {
        return None;
    }
    let clamp_i = dab.intensity.min(3.0);
    let (s, w, h) = (g.s, g.w, g.h);
    // O total de cada célula ANTES do dab: o tecto de quem recebe.
    let total = |g: &Grid, i: usize| f64::from(g.susp[i]) + f64::from(g.sett[i]);
    let mut leva: Vec<(usize, [(usize, f64); 4])> = Vec::new();
    let mut parte: Vec<f64> = Vec::new();
    let mut frente: Vec<f64> = Vec::new();
    for_each_stamp_pixel(
        s,
        w,
        h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        |i, x, y, fall, texv| {
            let u = (fall * texv * clamp_i).min(1.0) * force;
            let m = f64::from(g.susp[i]) * u.min(1.0);
            if m <= 0.0 {
                return;
            }
            let (fx, fy) = (f64::from(x) + dx, f64::from(y) + dy);
            let (x0, y0) = (fx.floor(), fy.floor());
            let (ax, ay) = (fx - x0, fy - y0);
            let alvo = |qx: f64, qy: f64, peso: f64| -> (usize, f64) {
                if qx < 1.0 || qy < 1.0 || qx > w as f64 || qy > h as f64 || peso <= 0.0 {
                    (i, 0.0)
                } else {
                    (qx as usize + qy as usize * s, peso)
                }
            };
            leva.push((
                i,
                [
                    alvo(x0, y0, (1.0 - ax) * (1.0 - ay)),
                    alvo(x0 + 1.0, y0, ax * (1.0 - ay)),
                    alvo(x0, y0 + 1.0, (1.0 - ax) * ay),
                    alvo(x0 + 1.0, y0 + 1.0, ax * ay),
                ],
            ));
            parte.push(m);
            frente.push(f64::from(x) * dx + f64::from(y) * dy);
        },
    );
    if leva.is_empty() {
        return None;
    }
    // O tecto de cada célula é o total dela ANTES do dab (ou o da origem, o maior); tirar TUDO primeiro e só depois
    // pousar deixa a tinta lisa lisa (cada célula recebe o que mandou) — pousar à medida fechava a porta a quem vem atrás.
    let mut antes_do_dab: std::collections::BTreeMap<usize, f64> =
        std::collections::BTreeMap::new();
    for (i, alvos) in &leva {
        antes_do_dab.entry(*i).or_insert_with(|| total(g, *i));
        for (j, _) in alvos {
            antes_do_dab.entry(*j).or_insert_with(|| total(g, *j));
        }
    }
    let cores: Vec<[f32; 3]> = leva.iter().map(|(i, _)| g.susp_rgb[*i]).collect();
    for (k, (i, _)) in leva.iter().enumerate() {
        parte[k] = parte[k].min(f64::from(g.susp[*i]));
        g.susp[*i] = (f64::from(g.susp[*i]) - parte[k]).max(0.0) as f32;
    }
    let mix = p.mix;
    let mut out = [0.0f64; 3];
    let mut pousa = |g: &mut Grid, j: usize, q: f64, rgb: [f32; 3]| {
        let dest = f64::from(g.susp[j]);
        let c = g.susp_rgb[j];
        mix.mix(
            f64::from(c[0]),
            f64::from(c[1]),
            f64::from(c[2]),
            f64::from(rgb[0]),
            f64::from(rgb[1]),
            f64::from(rgb[2]),
            q / (dest + q),
            &mut out,
        );
        g.susp_rgb[j] = [out[0] as f32, out[1] as f32, out[2] as f32];
        g.susp[j] = (dest + q) as f32;
    };
    // Da FRENTE para trás: quando uma célula pousa (e devolve o que não coube), quem a tem por alvo ainda não pousou
    // — o tecto vê a devolução, e nenhuma célula passa dele.
    let mut ordem: Vec<usize> = (0..leva.len()).collect();
    ordem.sort_by(|&a, &b| frente[b].total_cmp(&frente[a]));
    for k in ordem {
        let (i, alvos) = &leva[k];
        let origem = antes_do_dab[i];
        let mut fica = parte[k];
        for &(j, peso) in alvos {
            if peso <= 0.0 || j == *i {
                continue;
            }
            let cabe = (antes_do_dab[&j].max(origem) - total(g, j)).max(0.0);
            let q = (parte[k] * peso).min(cabe);
            if q > 0.0 {
                pousa(g, j, q, cores[k]);
                fica -= q;
            }
        }
        if fica > 0.0 {
            pousa(g, *i, fica, cores[k]);
        }
    }
    let rr = rect_around(g, dab.x, dab.y, dab.r + dx.abs().max(dy.abs()) + 1.0);
    expand_to(g, &rr);
    Some(rr)
}
