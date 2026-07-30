//! Fluid solver passes (port of `solver.js`; SPEC §6.1, §6.3–6.7, plus the
//! flow-side gated extensions of §17). All passes iterate only the active
//! bounding box.
//!
//! The velocity design in one paragraph: gravity accumulates in the
//! PERSISTENT field (vel_x/vel_y); every frame the TRANSIENT flow
//! (flow_x/flow_y) is rebuilt from it — with leveling + capillary + the
//! look-ahead absorbency brake on one frame in four, and as a cheap unbraked
//! smoothing on the other three. Mass then advects along the transient flow.
//! Because the brake only bites 1-in-4, a drip that would stall under
//! constant braking keeps advancing on the free frames: that asymmetry is
//! why drips run.

use crate::grid::{Grid, wet_byte_from_paper};
use crate::par::{self, Rows};
use crate::rng::hash2_signed;
use crate::sim::Params;
use crate::tuning::Knob;

const BLOOM_SEED: u32 = 0x600d;

#[inline]
fn clamp_sym(v: f64, m: f64) -> f64 {
    if v > m {
        m
    } else if v < -m {
        -m
    } else {
        v
    }
}

// ---------------------------------------------------------------------------
// §6.3 Flow-field build (every 4th frame — the braked frame)
// ---------------------------------------------------------------------------

/// ⚠️ **Roda na grade de FLUXO** (plano 30), e é o passe que a wave existe para
/// mover: 42,9% do tick não-amortizado, e a F1 mediu que **99,4% dele é o
/// NÚCLEO** (nivelamento · capilar · viscosidade · freio) — o backrun custa
/// 0,6% e o fingering 0,4%, então não há o que fatorar.
///
/// Os planos FINOS que ele lê (`film`, `paper`, `susp`, `sett`, `wet`) entram
/// por **AMOSTRA** ([`crate::flow::probe_idx`]) — uma célula fina por bloco.
/// Mediar seria `O(finas)` e custaria 12,7× mais (F1).
///
/// **As diferenças finitas levam o [`crate::flow::diff_scale`]** e nada mais
/// leva: o nivelamento e o capilar são gradientes; a viscosidade é uma média;
/// os empurrões do fingering e do backrun já são velocidades.
pub fn build_flow_field(g: &mut Grid, p: &Params, gx: f64, gy: f64, ext_bypass: bool) {
    let fine_s = g.s;
    let (gw, gh) = (g.w as i32, g.h as i32);
    let geom = g.flow;
    let s = geom.s;
    let rf = geom.rf as i64;
    let dif = crate::flow::diff_scale(geom.rf);
    let cells = geom.cells as i64;
    let fine_cells = g.cells as i64;
    let max_v = p.k(Knob::MaxVelocity);
    let g_mag = (gx * gx + gy * gy).sqrt();
    let fingering = !ext_bypass && p.k(Knob::ExtFingering) > 0.0 && g_mag > 0.0;
    let (mut gnx, mut gny, mut wave_len, mut t_axis_x, mut t_axis_y) = (0.0, 0.0, 1.0, 0.0, 0.0);
    if fingering {
        gnx = gx / g_mag;
        gny = gy / g_mag;
        // A largura de onda é uma FRAÇÃO da folha, então ela mede a folha na
        // grade em que o padrão é desenhado.
        wave_len = (geom.w as f64 / p.k(Knob::ExtRivulets)).max(4.0);
        t_axis_x = -gny;
        t_axis_y = gnx; // transverse axis
    }
    let backrun = !ext_bypass && p.k(Knob::ExtBackrun) > 0.0;
    let ext_backrun = p.k(Knob::ExtBackrun);
    let ext_fingering = p.k(Knob::ExtFingering);
    let mix = p.mix;
    let mut out = [0.0f64; 3];
    // Hoist every knob out of the hot loop.
    let level_k = p.k(Knob::Leveling);
    let level_clamp = p.k(Knob::LevelClamp);
    let cap_k = p.k(Knob::Capillary);
    let cap_gate = p.k(Knob::CapillaryGate);
    let visc_threshold = p.k(Knob::Viscosity);
    let brake_bias = p.k(Knob::Brake);
    let brake_reach = p.k(Knob::BrakeReach);
    let (gbx0, by0, gbx1, by1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), geom.rf);
    g.refresh_flow_spans();
    let spans_on = g.spans_enabled;
    // Split-borrow the fields once — the loop body reads/writes through
    // locals instead of re-projecting `g.` per access.
    let Grid {
        frow_lo: row_lo,
        frow_hi: row_hi,
        film,
        susp,
        susp_rgb,
        sett,
        sett_rgb,
        vel_x,
        vel_y,
        flow_x,
        flow_y,
        wet,
        paper,
        active,
        bloom,
        ..
    } = g;

    // Per-row subslices of the READ-ONLY arrays (film/paper/vel/active are
    // never written by this pass on any path) let the compiler prove the
    // stencil indices in-range and drop the bounds checks — the measured
    // cost of this loop was dominated by checked loads. The mutable arrays
    // (susp/sett/colors via backrun, wet, flow) stay checked full-array:
    // one door, no duplicated physics.
    let film_full: &[f32] = film;
    let paper_full: &[f32] = paper;
    let velx_full: &[f32] = vel_x;
    let vely_full: &[f32] = vel_y;
    let active_full: &[u8] = active;
    for y in by0..=by1 {
        // A faixa viva desta linha (ver `Grid::row_lo`): fora dela `active` é
        // 0, e o corpo do laço já era um `continue`.
        let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
        if bx0 > bx1 {
            continue;
        }
        // As janelas de FLUXO (velocidade): linhas `y-1 / y / y+1`.
        let off = (bx0 - 1) as usize; // bx0 >= 1
        let span = (bx1 - bx0 + 3) as usize; // stencil reach 1 on both sides
        let base = y as usize * s;
        let row = base + off;
        let velx_m = &velx_full[row..row + span];
        let velx_u = &velx_full[row - s..row - s + span];
        let velx_d = &velx_full[row + s..row + s + span];
        let vely_m = &vely_full[row..row + span];
        let vely_u = &vely_full[row - s..row - s + span];
        let vely_d = &vely_full[row + s..row + s + span];
        // As janelas FINAS: as linhas AMOSTRADAS, e a coluna anda de `rf` em
        // `rf`. Em `rf = 1` `fy_* == y∓1` e `fx_off == off`, então as fatias
        // são as mesmas que o motor sempre recortou.
        let pr = geom.rf;
        let fy_m = crate::flow::flow_probe(y, gh, pr) as usize;
        let fy_u = crate::flow::flow_probe(y - 1, gh, pr) as usize;
        let fy_d = crate::flow::flow_probe(y + 1, gh, pr) as usize;
        let fx_off = crate::flow::flow_probe(bx0 - 1, gw, pr);
        let fx_end = crate::flow::flow_probe(bx1 + 1, gw, pr);
        let fspan = (fx_end - fx_off + 1) as usize;
        let frow = fy_m * fine_s + fx_off as usize;
        let film_m = &film_full[frow..frow + fspan];
        let film_u = &film_full[fy_u * fine_s + fx_off as usize..][..fspan];
        let film_d = &film_full[fy_d * fine_s + fx_off as usize..][..fspan];
        let paper_m = &paper_full[frow..frow + fspan];
        let paper_u = &paper_full[fy_u * fine_s + fx_off as usize..][..fspan];
        let paper_d = &paper_full[fy_d * fine_s + fx_off as usize..][..fspan];
        let active_m = &active_full[frow..frow + fspan];
        let mut i = bx0 as usize + base;
        for x in bx0..=bx1 {
            // `k` percorre a janela de FLUXO; `kf` a FINA (que anda de `rf` em
            // `rf`, e cujos vizinhos são as células amostradas dos blocos ao
            // lado — não as células finas adjacentes).
            let k = (x - bx0) as usize + 1; // 1..=span-2
            let px = crate::flow::flow_probe(x, gw, pr);
            let kf = (px - fx_off) as usize;
            let kfl = (crate::flow::flow_probe(x - 1, gw, pr) - fx_off) as usize;
            let kfr = (crate::flow::flow_probe(x + 1, gw, pr) - fx_off) as usize;
            // O índice FINO desta célula de fluxo (a porta única do probe).
            let fi = fy_m * fine_s + px as usize;
            if active_m[kf] == 0 {
                i += 1;
                continue;
            }
            let f = film_m[kf] as f64;
            let mut ex = velx_m[k] as f64;
            let mut ey = vely_m[k] as f64;

            // Leveling: water flows thick -> thin, clamped per axis.
            // (Diferença finita ⇒ leva o `dif`.)
            let mut lx = (film_m[kfl] as f64 - film_m[kfr] as f64) * level_k * dif;
            if lx > level_clamp {
                lx = level_clamp;
            } else if lx < -level_clamp {
                lx = -level_clamp;
            }
            let mut ly = (film_u[kf] as f64 - film_d[kf] as f64) * level_k * dif;
            if ly > level_clamp {
                ly = level_clamp;
            } else if ly < -level_clamp {
                ly = -level_clamp;
            }
            ex += lx;
            ey += ly;

            // Capillary: only thin paint follows the tooth. Steepest-descent
            // pull — the asymmetric form picks the drop ACROSS the cell in
            // the direction of the steeper fall, which channels water into
            // grain rivulets. (Diferença finita ⇒ leva o `dif`.)
            if (susp[fi] as f64 + sett[fi] as f64) < cap_gate {
                let pc = paper_m[kf] as f64;
                let pl = paper_m[kfl] as f64;
                let prr = paper_m[kfr] as f64;
                ex += if pl > pc {
                    if pc < prr { pl - prr } else { pc - prr }
                } else if pc < prr {
                    pl - pc
                } else {
                    pl - prr
                } * cap_k
                    * dif;
                let pu = paper_u[kf] as f64;
                let pd = paper_d[kf] as f64;
                ey += if pu > pc {
                    if pc < pd { pu - pd } else { pc - pd }
                } else if pc < pd {
                    pu - pc
                } else {
                    pu - pd
                } * cap_k
                    * dif;
            }

            // Viscosity: deep water drags its persistent-field neighbours
            // along. (The knob is the film THRESHOLD; the 0.2 blend weights
            // are fixed.) É uma MÉDIA — adimensional, sem `dif`.
            if f > visc_threshold {
                ex = 0.2 * ex
                    + 0.2
                        * (velx_m[k - 1] as f64
                            + velx_m[k + 1] as f64
                            + velx_u[k] as f64
                            + velx_d[k] as f64);
                ey = 0.2 * ey
                    + 0.2
                        * (vely_m[k - 1] as f64
                            + vely_m[k + 1] as f64
                            + vely_u[k] as f64
                            + vely_d[k] as f64);
            }

            // Wetness stamp: deep film marks the sheet damp, valleys damper —
            // future flow keeps running along established wet channels.
            if f > 3.0 {
                wet[fi] = wet_byte_from_paper(paper_m[kf] as f64);
            }

            // Fingering (extension, PRE-brake): at a drip's leading edge, add
            // a transverse sinusoidal ripple — push down at the peaks plus a
            // slight sideways component; the brake then self-selects rivulet
            // columns.
            if fingering && f > 0.1 {
                let dxs = js_round_i64(gnx);
                let dys = js_round_i64(gny);
                let j = i as i64 + dxs + dys * s as i64;
                let jf = fi as i64 + dxs * rf + dys * rf * fine_s as i64;
                if j >= 0
                    && j < cells
                    && jf >= 0
                    && jf < fine_cells
                    && film_full[jf as usize] as f64 <= 0.01
                {
                    let phase =
                        (2.0 * std::f64::consts::PI * (x as f64 * t_axis_x + y as f64 * t_axis_y))
                            / wave_len;
                    let push = ext_fingering * f.min(2.0);
                    let sn = libm::sin(phase);
                    let cs = libm::cos(phase);
                    ex += gnx * push * sn + t_axis_x * push * 0.3 * cs;
                    ey += gny * push * sn + t_axis_y * push * 0.3 * cs;
                }
            }

            // Look-ahead absorbency brake: probe a few px downstream; wet or
            // flooded ground ahead lets flow keep running, dry ground stalls
            // it. Linear index arithmetic, no 2-D clamp: a probe past the
            // array end skips the brake entirely (flow at the sheet edge runs
            // off it).
            //
            // ⚠️ O alcance é uma DISTÂNCIA na folha, então em passos de índice
            // ele leva o `dif`: sondar `brake_reach` células FINAS custa
            // `brake_reach / rf` passos de fluxo.
            let s_len = brake_reach * dif / ((ex * ex + ey * ey).sqrt() + 0.01);
            let dxs = (ex * s_len).trunc() as i64;
            let dys = (ey * s_len).trunc() as i64;
            let probe = i as i64 + dxs + dys * s as i64;
            let probe_f = fi as i64 + dxs * rf + dys * rf * fine_s as i64;
            if probe >= 0 && probe < cells && probe_f >= 0 && probe_f < fine_cells {
                let pu = probe_f as usize;
                let mut brake = film_full[pu] as f64 + (3.0 / 255.0) * wet[pu] as f64 - brake_bias;
                if brake < 0.05 {
                    brake = 0.05;
                } else if brake > 1.0 {
                    brake = 1.0;
                }
                ex *= brake;
                ey *= brake;
            }

            // Backrun / bloom (extension, POST-brake): where this cell is
            // much wetter than a neighbour holding settled pigment, shove
            // flow toward it and lift some of its settled mass back into
            // suspension. A per-cell budget (max 6 blooming build-frames per
            // fresh front) stops sloshing from pumping thin lines;
            // integer-hash jitter crenellates the rim.
            if backrun {
                let thr = 0.8 + 0.2 * hash2_signed(x, y, BLOOM_SEED);
                let mut met = false;
                let can_bloom = bloom[fi] < 6;
                for n_idx in 0..4 {
                    let dxn: i64 = match n_idx {
                        0 => -1,
                        1 => 1,
                        _ => 0,
                    };
                    let dyn_: i64 = match n_idx {
                        2 => -1,
                        3 => 1,
                        _ => 0,
                    };
                    // ⚠️ O vizinho é de FLUXO, mas o pigmento que ele levanta é
                    // FINO: a célula amostrada daquele bloco. Em `rf > 1` o
                    // backrun fica ESPARSO (um sítio de nucleação por bloco) —
                    // mudança de APARÊNCIA, para o smoke decidir.
                    let nb = (fi as i64 + dxn * rf + dyn_ * rf * fine_s as i64) as usize;
                    let gap = f - film_full[nb] as f64;
                    if gap > thr && sett[nb] as f64 > 0.0 {
                        met = true;
                        if can_bloom {
                            let push = ext_backrun * gap.min(1.5) * 0.5;
                            if dxn != 0 {
                                ex += dxn as f64 * push;
                            } else {
                                ey += dyn_ as f64 * push;
                            }
                            let lift = sett[nb] as f64 * 0.1;
                            let w = lift / (susp[nb] as f64 + lift);
                            let uc = susp_rgb[nb];
                            let sc = sett_rgb[nb];
                            mix.mix(
                                uc[0] as f64,
                                uc[1] as f64,
                                uc[2] as f64,
                                sc[0] as f64,
                                sc[1] as f64,
                                sc[2] as f64,
                                w,
                                &mut out,
                            );
                            susp_rgb[nb] = [out[0] as f32, out[1] as f32, out[2] as f32];
                            susp[nb] = (susp[nb] as f64 + lift) as f32;
                            sett[nb] = (sett[nb] as f64 - lift) as f32;
                        }
                    }
                }
                if met {
                    if can_bloom {
                        bloom[fi] += 1;
                    }
                } else {
                    bloom[fi] = 0;
                }
            }

            flow_x[i] = clamp_sym(ex, max_v) as f32;
            flow_y[i] = clamp_sym(ey, max_v) as f32;
            i += 1;
        }
    }
}

/// `Math.round` of a unit-vector component, as the JS fingering does.
#[inline]
fn js_round_i64(v: f64) -> i64 {
    (v + 0.5).floor() as i64
}

// ---------------------------------------------------------------------------
// §6.4 Velocity smoothing (the other 3 frames — never braked)
// ---------------------------------------------------------------------------

pub fn smooth_velocity(g: &mut Grid, p: &Params) {
    let (fbx0, fby0, fbx1, fby1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), g.flow.rf);
    let rows = (fby1 - fby0 + 1).max(0) as usize;
    let span = (fbx1 - fbx0 + 1).max(0) as usize;
    smooth_velocity_rows(g, p, Rows::pick(rows, span, par::MIN_CELLS_GATHER));
}

/// [`smooth_velocity`] com a rota de caminhada FORÇADA — a porta dos gates de
/// identidade (ADR-0145). O produto chama sempre o [`smooth_velocity`].
///
/// **É um GATHER puro:** escreve `flow_x`/`flow_y` no próprio índice e lê
/// `vel_x`/`vel_y` (inclusive das linhas vizinhas), `film` e `active` — nenhum
/// deles tocado por este passe. Sem redução, sem transcendental, sem RNG.
///
/// ⚠️ **Roda na grade de FLUXO** (plano 30): os quatro planos de velocidade são
/// indexados pelo stride de fluxo, e `film`/`active` — que são FINOS — entram
/// pela porta [`crate::flow::probe_idx`], isto é, **por amostra e não por
/// média** (a F1 mediu a média em 12,7× o custo da amostra).
///
/// **Nenhuma diferença finita aqui:** as duas expressões são MÉDIAS de
/// vizinhos, adimensionais, então elas não levam o [`crate::flow::diff_scale`].
pub fn smooth_velocity_rows(g: &mut Grid, p: &Params, mode: Rows) {
    let s = g.s;
    let geom = g.flow;
    let fs = geom.s;
    let (gw, gh) = (g.w as i32, g.h as i32);
    let max_v = p.k(Knob::MaxVelocity);
    let (fbx0, fby0, fbx1, fby1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), geom.rf);
    if fby1 < fby0 {
        return;
    }
    g.refresh_flow_spans();
    let spans_on = g.spans_enabled;
    let Grid {
        frow_lo,
        frow_hi,
        film,
        vel_x,
        vel_y,
        flow_x,
        flow_y,
        active,
        ..
    } = g;
    let row_lo: &[i32] = frow_lo;
    let row_hi: &[i32] = frow_hi;
    let film: &[f32] = film;
    let velx: &[f32] = vel_x;
    let vely: &[f32] = vel_y;
    let active: &[u8] = active;
    let b = fby0 as usize * fs..(fby1 as usize + 1) * fs;
    par::walk_rows2(
        mode,
        &mut flow_x[b.clone()],
        &mut flow_y[b],
        fs,
        |r, fxr, fyr| {
            let y = fby0 + r as i32;
            // Faixa viva: fora dela `active` é 0 e o corpo já era um `continue`.
            let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, fbx0, fbx1, y);
            if bx0 > bx1 {
                return;
            }
            let base = y as usize * fs;
            for x in bx0..=bx1 {
                let i = x as usize + base;
                let fi = crate::flow::probe_idx(x, y, gw, gh, s, geom.rf);
                if active[fi] == 0 {
                    continue;
                }
                let (mut fx, mut fy);
                if film[fi] as f64 > 0.05 {
                    fx = 0.2 * velx[i] as f64
                        + 0.2
                            * (velx[i - 1] as f64
                                + velx[i + 1] as f64
                                + velx[i - fs] as f64
                                + velx[i + fs] as f64);
                    fy = 0.2 * vely[i] as f64
                        + 0.2
                            * (vely[i - 1] as f64
                                + vely[i + 1] as f64
                                + vely[i - fs] as f64
                                + vely[i + fs] as f64);
                } else {
                    // Whatever gravity the persistent field carries passes
                    // straight through.
                    fx = velx[i] as f64;
                    fy = vely[i] as f64;
                }
                if fx > max_v {
                    fx = max_v;
                } else if fx < -max_v {
                    fx = -max_v;
                }
                if fy > max_v {
                    fy = max_v;
                } else if fy < -max_v {
                    fy = -max_v;
                }
                fxr[x as usize] = fx as f32;
                fyr[x as usize] = fy as f32;
            }
        },
    );
}

/// Diffusion (extension, smoothing frames): Fickian spread of suspended
/// pigment through a still wet film. Symmetric flux to the +x/+y neighbours
/// only (each edge visited once => mass-conserving); color rides
/// mass-weighted.
pub fn diffusion_pass(g: &mut Grid, p: &Params) {
    let s = g.s;
    let mut out = [0.0f64; 3];
    let rate_knob = p.k(Knob::ExtDiffusion);
    for y in g.by0..=g.by1 {
        // Faixa viva: fora dela `active` é 0 e o corpo já era um `continue`.
        let (bx0, bx1) = g.span_x(y);
        if bx0 > bx1 {
            continue;
        }
        let mut i = bx0 as usize + y as usize * s;
        for _x in bx0..=bx1 {
            if g.active[i] == 0 || g.film[i] as f64 <= 0.1 {
                i += 1;
                continue;
            }
            let rate = rate_knob * (g.film[i] as f64 / 1.5).min(1.0);
            for e in 0..2 {
                let nb = if e == 0 { i + 1 } else { i + s };
                let flux = rate * (g.susp[i] as f64 - g.susp[nb] as f64);
                let (from, to) = if flux > 0.0 { (i, nb) } else { (nb, i) };
                let mut dm = flux.abs();
                if dm <= 0.0 {
                    continue;
                }
                if dm > g.susp[from] as f64 {
                    dm = g.susp[from] as f64;
                }
                let w = dm / (g.susp[to] as f64 + dm);
                let tc = g.susp_rgb[to];
                let fc = g.susp_rgb[from];
                p.mix.mix(
                    tc[0] as f64,
                    tc[1] as f64,
                    tc[2] as f64,
                    fc[0] as f64,
                    fc[1] as f64,
                    fc[2] as f64,
                    w,
                    &mut out,
                );
                g.susp_rgb[to] = [out[0] as f32, out[1] as f32, out[2] as f32];
                g.susp[from] = (g.susp[from] as f64 - dm) as f32;
                g.susp[to] = (g.susp[to] as f64 + dm) as f32;
            }
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// §6.5 advection and §6.6/§6.7 projection + boundaries live in child modules
// (workspace file-LOC cap); the `solver::` paths are re-exported unchanged.
// ---------------------------------------------------------------------------

mod active_region;
mod advect;
mod advect_jacobi;
mod project;
pub use active_region::{rebuild_active_region, rebuild_active_region_rows};
pub use advect::advect;
pub use advect_jacobi::{advect_jacobi, advect_jacobi_rows, destination_reach};
pub use project::{apply_boundaries, project, project_rows};
