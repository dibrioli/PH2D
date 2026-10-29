//! O **Smear** ([`apply_smear`]) — filho de [`super`] por LOC.

use super::*;

/// smear: drag suspended pigment + water from the previous-dab offset (the
/// reach grows with speed). The source is a mass-weighted bilinear sample
/// of a pre-dab snapshot (weighting each corner by its mass keeps edges
/// from dragging toward black).
///
/// Doc 23 P2 — the settled layer is no longer untouchable (the old note
/// said "matching real smudge behavior"; measured against Rebelle's Smudge
/// and real re-soluble watercolor, it wasn't): on a WET cell the pass
/// dissolves settled pigment through the lift door (the next dab's
/// snapshot then drags what this one freed — smudging through wet paint IS
/// dissolve-then-drag); on a DRY cell the settled layer itself is dragged,
/// like every raster smudge. Both are resisted by `1 − staining`, so
/// `staining = 1` restores the suspended-only model to the byte.
pub fn apply_smear(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    prev_x: f64,
    prev_y: f64,
) -> Option<TouchedRect> {
    let force = p.k(Knob::Smear);
    // A água do Smear ([`Knob::SmearWater`]): até ao meio, quanto da água o rasto arrasta; acima, quanto molha.
    let agua = p.k(Knob::SmearWater);
    let arrasta_agua = (2.0 * agua).min(1.0);
    let molha = (2.0 * agua - 1.0).max(0.0);
    let cap = p.k(Knob::WaterCap);
    let sett_gain = 1.0 - p.k(Knob::ExtStaining);
    let clamp_i = dab.intensity.min(3.0);
    let dx = dab.x - prev_x;
    let dy = dab.y - prev_y;
    let reach = dx.abs().max(dy.abs()).ceil();
    // Snapshot the affected region (+10 px pad): sampling live arrays would
    // compound the drag within one dab.
    let pad = 10.0 + reach;
    let r = rect_around_nao_vazio(g, dab.x, dab.y, dab.r + pad)?;
    let snap_w = (r.x1 - r.x0 + 1) as usize;
    let snap_h = (r.y1 - r.y0 + 1) as usize;
    let n = snap_w * snap_h;
    let mut s_mass = vec![0.0f32; n];
    let mut s_rgb = vec![[0.0f32; 3]; n];
    let mut s_f = vec![0.0f32; n];
    let mut s_sett = vec![0.0f32; n];
    let mut s_sett_rgb = vec![[0.0f32; 3]; n];
    // O material (W2 E3) é borrado com a cor, pela mesma amostra pesada pela massa.
    let s_mat = crate::grid::material::recorte(g, r.x0, r.y0, r.x1, r.y1);
    // A marca do nivelado (K7/K8) viaja como o material; vazia = a sessão nunca nivelou ⇒ nada muda.
    let s_marca = marca::recorte(g, r.x0, r.y0, r.x1, r.y1);
    for y in r.y0..=r.y1 {
        for x in r.x0..=r.x1 {
            let i = x as usize + y as usize * g.s;
            let l = (x - r.x0) as usize + (y - r.y0) as usize * snap_w;
            s_mass[l] = g.susp[i];
            s_rgb[l] = g.susp_rgb[i];
            s_f[l] = g.film[i];
            s_sett[l] = g.sett[i];
            s_sett_rgb[l] = g.sett_rgb[i];
        }
    }
    let mut out = [0.0f64; 3];
    let mix = p.mix;
    let mut touched = false;
    let Grid {
        s,
        w,
        h,
        susp,
        susp_rgb,
        sett,
        sett_rgb,
        film,
        vela,
        aparencia,
        material,
        nivelado,
        wet,
        paper,
        ..
    } = g;
    let borra = |m: &mut [crate::material::MaterialCelula],
                 niv: &mut [f32],
                 i: usize,
                 ls: [usize; 4],
                 ws: [f64; 4],
                 t: f64| {
        crate::grid::material::borra(m, &s_mat, i, ls, ws, t);
        marca::borra(niv, &s_marca, i, ls, ws, t);
    };
    for_each_stamp_pixel(
        *s,
        *w,
        *h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        |i, x, y, fall, texv| {
            let mut stamp = fall * texv * clamp_i;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            let u = (stamp * force).min(1.0);
            if u <= 0.0 {
                return;
            }
            // Clamp the sample offset to the snapshot pad.
            let mut fx = x as f64 - dx;
            let mut fy = y as f64 - dy;
            if fx < r.x0 as f64 {
                fx = r.x0 as f64;
            }
            if fx > (r.x1 - 1) as f64 {
                fx = (r.x1 - 1) as f64;
            }
            if fy < r.y0 as f64 {
                fy = r.y0 as f64;
            }
            if fy > (r.y1 - 1) as f64 {
                fy = (r.y1 - 1) as f64;
            }
            let x0 = fx.floor() as i32;
            let y0 = fy.floor() as i32;
            let tx = fx - x0 as f64;
            let ty = fy - y0 as f64;
            let l00 = (x0 - r.x0) as usize + (y0 - r.y0) as usize * snap_w;
            let l10 = l00 + 1;
            let l01 = l00 + snap_w;
            let l11 = l01 + 1;
            let w00 = (1.0 - tx) * (1.0 - ty);
            let w10 = tx * (1.0 - ty);
            let w01 = (1.0 - tx) * ty;
            let w11 = tx * ty;
            // Mass-weighted color sample.
            let m00 = s_mass[l00] as f64 * w00;
            let m10 = s_mass[l10] as f64 * w10;
            let m01 = s_mass[l01] as f64 * w01;
            let m11 = s_mass[l11] as f64 * w11;
            let src_mass = m00 + m10 + m01 + m11;
            if src_mass > 0.0 {
                let inv = 1.0 / src_mass;
                let k00 = s_rgb[l00];
                let k10 = s_rgb[l10];
                let k01 = s_rgb[l01];
                let k11 = s_rgb[l11];
                let cr = (k00[0] as f64 * m00
                    + k10[0] as f64 * m10
                    + k01[0] as f64 * m01
                    + k11[0] as f64 * m11)
                    * inv;
                let cg = (k00[1] as f64 * m00
                    + k10[1] as f64 * m10
                    + k01[1] as f64 * m01
                    + k11[1] as f64 * m11)
                    * inv;
                let cbv = (k00[2] as f64 * m00
                    + k10[2] as f64 * m10
                    + k01[2] as f64 * m01
                    + k11[2] as f64 * m11)
                    * inv;
                let in_c = alpha_of_mass(src_mass) * u;
                let res = alpha_of_mass(susp[i] as f64) * (1.0 - in_c);
                if in_c + res > 0.0 {
                    let wgt = in_c / (in_c + res);
                    let uc = susp_rgb[i];
                    mix.mix(
                        uc[0] as f64,
                        uc[1] as f64,
                        uc[2] as f64,
                        cr,
                        cg,
                        cbv,
                        wgt,
                        &mut out,
                    );
                    susp_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
                    borra(
                        material,
                        nivelado,
                        i,
                        [l00, l10, l01, l11],
                        [m00, m10, m01, m11],
                        wgt,
                    );
                }
                susp[i] = (susp[i] as f64 + (src_mass - susp[i] as f64) * u) as f32;
            }
            // Water lerps (plain bilinear), scaled by the Smear's water; above the middle it also wets the trail.
            let src_film = s_f[l00] as f64 * w00
                + s_f[l10] as f64 * w10
                + s_f[l01] as f64 * w01
                + s_f[l11] as f64 * w11;
            film[i] = (film[i] as f64 + (src_film - film[i] as f64) * u * arrasta_agua) as f32;
            if molha > 0.0 && (film[i] as f64) < cap {
                film[i] = (film[i] as f64 + (cap - film[i] as f64) * u * molha) as f32;
                wet[i] = wet_byte_from_paper(paper[i] as f64);
            }
            // Doc 23 P2 — the settled layer, resisted by staining.
            let u_sett = u * sett_gain;
            if u_sett > 0.0 {
                if film[i] as f64 > 0.001 {
                    // WET cell: dissolve through the lift door; the susp
                    // drag above carries it from the next dab on.
                    if sett[i] > 0.0 {
                        let b = clamp01(u_sett);
                        lift_settled(
                            b,
                            &mut susp[i],
                            &mut sett[i],
                            &mut susp_rgb[i],
                            sett_rgb[i],
                            mix,
                            p.glaze,
                            p.ecra
                                .com_vela(vela[i])
                                .com_vidrado(crate::grid::aparencia::vidrado(aparencia, i)),
                            &mut out,
                        );
                    }
                } else {
                    // DRY cell: drag the settled layer itself (same
                    // mass-weighted bilinear sample as the suspended pass).
                    let sm00 = s_sett[l00] as f64 * w00;
                    let sm10 = s_sett[l10] as f64 * w10;
                    let sm01 = s_sett[l01] as f64 * w01;
                    let sm11 = s_sett[l11] as f64 * w11;
                    let src_sett = sm00 + sm10 + sm01 + sm11;
                    if src_sett > 0.0 {
                        let inv = 1.0 / src_sett;
                        let k00 = s_sett_rgb[l00];
                        let k10 = s_sett_rgb[l10];
                        let k01 = s_sett_rgb[l01];
                        let k11 = s_sett_rgb[l11];
                        let cr = (k00[0] as f64 * sm00
                            + k10[0] as f64 * sm10
                            + k01[0] as f64 * sm01
                            + k11[0] as f64 * sm11)
                            * inv;
                        let cg = (k00[1] as f64 * sm00
                            + k10[1] as f64 * sm10
                            + k01[1] as f64 * sm01
                            + k11[1] as f64 * sm11)
                            * inv;
                        let cbv = (k00[2] as f64 * sm00
                            + k10[2] as f64 * sm10
                            + k01[2] as f64 * sm01
                            + k11[2] as f64 * sm11)
                            * inv;
                        let in_c = alpha_of_mass(src_sett) * u_sett;
                        let res = alpha_of_mass(sett[i] as f64) * (1.0 - in_c);
                        if in_c + res > 0.0 {
                            let wgt = in_c / (in_c + res);
                            let sc = sett_rgb[i];
                            mix.mix(
                                sc[0] as f64,
                                sc[1] as f64,
                                sc[2] as f64,
                                cr,
                                cg,
                                cbv,
                                wgt,
                                &mut out,
                            );
                            sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
                            borra(
                                material,
                                nivelado,
                                i,
                                [l00, l10, l01, l11],
                                [sm00, sm10, sm01, sm11],
                                wgt,
                            );
                        }
                    }
                    if src_sett > 0.0 || sett[i] > 0.0 {
                        sett[i] = (sett[i] as f64 + (src_sett - sett[i] as f64) * u_sett) as f32;
                    }
                }
            }
            touched = true;
        },
    );
    if touched {
        let rr = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &rr);
        Some(rr)
    } else {
        None
    }
}
