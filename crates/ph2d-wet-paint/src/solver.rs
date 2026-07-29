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
// §6.1 Active-region rebuild (every 2nd frame)
// ---------------------------------------------------------------------------

/// Rebuild the active mask + fresh bbox from the water map. Pass 1 marks
/// horizontal wet triples inside the previous (padded) bbox; pass 2 grows a
/// vertical "skirt" wherever a vertical triple sums to EXACTLY 1 — and the 2s
/// it writes count in later sums, which is load-bearing: an isolated front
/// gets a full skirt and can run 1 cell/frame, while a train of close stripes
/// starves its own skirt and waits to merge (keeps a wide front from
/// decomposing into permanent horizontal bands).
pub fn rebuild_active_region(g: &mut Grid) {
    let s = g.s;
    let w = g.w as i32;
    let h = g.h as i32;
    if !g.has_fluid || g.bx1 < g.bx0 {
        g.empty_bbox();
        return;
    }
    // Clear the mask over the previous bbox padded ±2. Extend the clear to
    // the pad ring when the box touches the sheet edge, so skirt 2s written
    // on the pad cannot go stale (the [1..W] clamp plus pad writes would
    // otherwise leak permanent actives on row 0 / row H+1).
    let px0 = (g.bx0 - 2).max(1);
    let px1 = (g.bx1 + 2).min(w);
    let py0 = (g.by0 - 2).max(1);
    let py1 = (g.by1 + 2).min(h);
    let cx0 = if px0 == 1 { 0 } else { px0 };
    let cx1 = if px1 == w { w + 1 } else { px1 };
    let cy0 = if py0 == 1 { 0 } else { py0 };
    let cy1 = if py1 == h { h + 1 } else { py1 };
    // Só a JANELA da faixa viva precisa ser limpa: `active ⊆ faixa` por
    // construção (invariante publicado por esta mesma função), então uma
    // célula fora dela já vale 0 e a limpeza seria escrever 0 sobre 0.
    for y in cy0..=cy1 {
        let (wl, wh) = g.span_window(y);
        let (l, hgh) = (wl.max(cx0), wh.min(cx1));
        if l > hgh {
            continue;
        }
        let base = y as usize * s;
        g.active[base + l as usize..base + hgh as usize + 1].fill(0);
    }
    // A extensão VIVA desta passada — "há algo aqui que o solver ainda tem de
    // terminar". São TRÊS coisas, e cada uma responde a um passe diferente:
    //
    //   `film > 0`  a água (o `advect`, e o próprio passe 1 abaixo)
    //   `susp > 0`  o pigmento em suspensão — o `drying_pass` é gateado em
    //               `film/susp`, NÃO na máscara ("paint dries everywhere"),
    //               então uma faixa que só cobrisse o ativo pararia de secar
    //               pigmento num pixel que a máscara já largou
    //   `vel != 0`  a VELOCIDADE sobrevivente: o ramo inativo do `advect`
    //               zera `vel`, e essa escrita É observável (o fingerprint da
    //               sessão inclui `vel_x`/`vel_y`). Uma célula que secou e
    //               ficou com velocidade tem de continuar visível até o
    //               advect zerá-la — aí ela sai sozinha na próxima passada.
    //               ⚠️ Isto foi a rede de debug que achou, na PRIMEIRA
    //               execução da suíte: sem este termo, o rastro de um drip
    //               que se afasta mais de 5 células fica com velocidade
    //               fóssil e o estado deriva do motor original.
    //
    // O anel de dreno fica FORA do termo de velocidade: o `apply_boundaries`
    // reescreve os dois componentes lá em todo frame, logo após o advect, e
    // nada lê `vel` entre um e outro — a zeragem do advect é natimorta ali.
    // ⚠️ Varre TODA linha cuja janela é não-vazia, não só as da bbox: a faixa
    // é o que lembra onde há velocidade fóssil, e a bbox de um traço NOVO não
    // tem por que cobrir o rastro de um antigo. O laço de linhas é O(altura)
    // e sai na hora nas vazias; o custo real é a janela, que é a faixa.
    g.clear_live();
    for y in 0..g.rows as i32 {
        let (wl, wh) = g.span_window(y);
        let (l, r) = (wl.max(1), wh.min(w));
        if l > r {
            continue;
        }
        let vel_rows = y >= 2 && y < h;
        let base = y as usize * s;
        let mut lo = i32::MAX;
        let mut hi = i32::MIN;
        let mut i = l as usize + base;
        for x in l..=r {
            let live = g.film[i] > 0.0
                || g.susp[i] > 0.0
                || (vel_rows && x >= 2 && x < w && (g.vel_x[i] != 0.0 || g.vel_y[i] != 0.0));
            if live {
                if x < lo {
                    lo = x;
                }
                hi = x;
            }
            i += 1;
        }
        if lo <= hi {
            g.live_lo[y as usize] = lo;
            g.live_hi[y as usize] = hi;
        }
    }

    // Pass 1 — wet cells (one row/col inside the brushable area; the drain
    // ring only ever activates via the skirt).
    let mut fx0 = w + 1;
    let mut fx1 = 0;
    let mut fy0 = h + 1;
    let mut fy1 = 0;
    let sx0 = px0.max(2);
    let sx1 = px1.min(w - 1);
    let sy0 = py0.max(3);
    let sy1 = py1.min(h - 2);
    let mut fired = false;
    for y in sy0..=sy1 {
        // A janela cobre `film ⊕ 2`, então o trio horizontal e as escritas
        // `active[i±1]` cabem nela: fora daqui o trio é 0 + 0 + 0.
        let (wl, wh) = g.span_window(y);
        let (rx0, rx1) = (wl.max(sx0), wh.min(sx1));
        if rx0 > rx1 {
            continue;
        }
        let mut i = rx0 as usize + y as usize * s;
        for x in rx0..=rx1 {
            if g.film[i - 1] as f64 + g.film[i] as f64 + g.film[i + 1] as f64 > 0.0 {
                g.active[i - 1] = 1;
                g.active[i] = 1;
                g.active[i + 1] = 1;
                fired = true;
                if x < fx0 {
                    fx0 = x;
                }
                if x > fx1 {
                    fx1 = x;
                }
                if y < fy0 {
                    fy0 = y;
                }
                if y > fy1 {
                    fy1 = y;
                }
            }
            i += 1;
        }
    }
    if !fired {
        g.empty_bbox();
        return;
    }

    // Pass 2 — the skirt, scanned top-to-down so earlier 2s shape later sums.
    let kx0 = (fx0 - 2).max(1);
    let kx1 = (fx1 + 2).min(w);
    let ky0 = (fy0 - 2).max(1);
    let ky1 = (fy1 + 2).min(h);
    let mut nx0 = w + 1;
    let mut nx1 = 0;
    let mut ny0 = h + 1;
    let mut ny1 = 0;
    let mut any_fire = false;
    for y in ky0..=ky1 {
        // O trio VERTICAL só pode somar 1 onde alguma das três células é
        // ativa, e as ativas desta passada saíram do passe 1 — dentro da
        // janela, que carrega a margem de ±2 nas linhas vizinhas também.
        let (wl, wh) = g.span_window(y);
        let (rx0, rx1) = (wl.max(kx0), wh.min(kx1));
        if rx0 > rx1 {
            continue;
        }
        let mut i = rx0 as usize + y as usize * s;
        for x in rx0..=rx1 {
            if g.active[i - s] as u32 + g.active[i] as u32 + g.active[i + s] as u32 == 1 {
                g.active[i - s] = 2;
                g.active[i] = 2;
                if g.active[i + s] == 0 {
                    g.active[i + s] = 2;
                }
                any_fire = true;
                if x < nx0 {
                    nx0 = x;
                }
                if x > nx1 {
                    nx1 = x;
                }
                if y < ny0 {
                    ny0 = y;
                }
                if y > ny1 {
                    ny1 = y; // the fire row itself
                }
            }
            i += 1;
        }
    }
    if !any_fire {
        // defensive: keep the pass-1 extent
        nx0 = fx0;
        nx1 = fx1;
        ny0 = fy0;
        ny1 = fy1;
    }
    g.bx0 = (nx0 - 5).max(1);
    g.bx1 = (nx1 + 5).min(w);
    g.by0 = (ny0 - 5).max(1);
    g.by1 = (ny1 + 5).min(h);
    g.has_fluid = true;
    // Publica a faixa por-linha com o MESMO pad ±5 que a bbox acabou de levar
    // — é o análogo por-linha do casco, e é ele que mantém `active ⊆ faixa`.
    g.publish_spans_from_live();
}

// ---------------------------------------------------------------------------
// §6.3 Flow-field build (every 4th frame — the braked frame)
// ---------------------------------------------------------------------------

pub fn build_flow_field(g: &mut Grid, p: &Params, gx: f64, gy: f64, ext_bypass: bool) {
    let s = g.s;
    let cells = g.cells as i64;
    let max_v = p.k(Knob::MaxVelocity);
    let g_mag = (gx * gx + gy * gy).sqrt();
    let fingering = !ext_bypass && p.k(Knob::ExtFingering) > 0.0 && g_mag > 0.0;
    let (mut gnx, mut gny, mut wave_len, mut t_axis_x, mut t_axis_y) = (0.0, 0.0, 1.0, 0.0, 0.0);
    if fingering {
        gnx = gx / g_mag;
        gny = gy / g_mag;
        wave_len = (g.w as f64 / p.k(Knob::ExtRivulets)).max(4.0);
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
    let (gbx0, gbx1, by0, by1) = (g.bx0, g.bx1, g.by0, g.by1);
    let spans_on = g.spans_enabled;
    // Split-borrow the fields once — the loop body reads/writes through
    // locals instead of re-projecting `g.` per access.
    let Grid {
        row_lo,
        row_hi,
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
        let off = (bx0 - 1) as usize; // bx0 >= 1
        let span = (bx1 - bx0 + 3) as usize; // stencil reach 1 on both sides
        let base = y as usize * s;
        let row = base + off;
        // Middle / up / down row windows, all exactly `span` long.
        let film_m = &film_full[row..row + span];
        let film_u = &film_full[row - s..row - s + span];
        let film_d = &film_full[row + s..row + s + span];
        let paper_m = &paper_full[row..row + span];
        let paper_u = &paper_full[row - s..row - s + span];
        let paper_d = &paper_full[row + s..row + s + span];
        let velx_m = &velx_full[row..row + span];
        let velx_u = &velx_full[row - s..row - s + span];
        let velx_d = &velx_full[row + s..row + s + span];
        let vely_m = &vely_full[row..row + span];
        let vely_u = &vely_full[row - s..row - s + span];
        let vely_d = &vely_full[row + s..row + s + span];
        let active_m = &active_full[row..row + span];
        let mut i = bx0 as usize + base;
        for x in bx0..=bx1 {
            let k = (x - bx0) as usize + 1; // 1..=span-2
            if active_m[k] == 0 {
                i += 1;
                continue;
            }
            let f = film_m[k] as f64;
            let mut ex = velx_m[k] as f64;
            let mut ey = vely_m[k] as f64;

            // Leveling: water flows thick -> thin, clamped per axis.
            let mut lx = (film_m[k - 1] as f64 - film_m[k + 1] as f64) * level_k;
            if lx > level_clamp {
                lx = level_clamp;
            } else if lx < -level_clamp {
                lx = -level_clamp;
            }
            let mut ly = (film_u[k] as f64 - film_d[k] as f64) * level_k;
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
            // grain rivulets.
            if (susp[i] as f64 + sett[i] as f64) < cap_gate {
                let pc = paper_m[k] as f64;
                let pl = paper_m[k - 1] as f64;
                let pr = paper_m[k + 1] as f64;
                ex += if pl > pc {
                    if pc < pr { pl - pr } else { pc - pr }
                } else if pc < pr {
                    pl - pc
                } else {
                    pl - pr
                } * cap_k;
                let pu = paper_u[k] as f64;
                let pd = paper_d[k] as f64;
                ey += if pu > pc {
                    if pc < pd { pu - pd } else { pc - pd }
                } else if pc < pd {
                    pu - pc
                } else {
                    pu - pd
                } * cap_k;
            }

            // Viscosity: deep water drags its persistent-field neighbours
            // along. (The knob is the film THRESHOLD; the 0.2 blend weights
            // are fixed.)
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
                wet[i] = wet_byte_from_paper(paper_m[k] as f64);
            }

            // Fingering (extension, PRE-brake): at a drip's leading edge, add
            // a transverse sinusoidal ripple — push down at the peaks plus a
            // slight sideways component; the brake then self-selects rivulet
            // columns.
            if fingering && f > 0.1 {
                let j = i as i64 + js_round_i64(gnx) + js_round_i64(gny) * s as i64;
                if j >= 0 && j < cells && film_full[j as usize] as f64 <= 0.01 {
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
            let s_len = brake_reach / ((ex * ex + ey * ey).sqrt() + 0.01);
            let probe =
                i as i64 + (ex * s_len).trunc() as i64 + (ey * s_len).trunc() as i64 * s as i64;
            if probe >= 0 && probe < cells {
                let pu = probe as usize;
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
                let can_bloom = bloom[i] < 6;
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
                    let nb = (i as i64 + dxn + dyn_ * s as i64) as usize;
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
                        bloom[i] += 1;
                    }
                } else {
                    bloom[i] = 0;
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
    let s = g.s;
    let max_v = p.k(Knob::MaxVelocity);
    let (by0, by1) = (g.by0, g.by1);
    for y in by0..=by1 {
        // Faixa viva: fora dela `active` é 0 e o corpo já era um `continue`.
        let (bx0, bx1) = g.span_x(y);
        if bx0 > bx1 {
            continue;
        }
        let mut i = bx0 as usize + y as usize * s;
        for _x in bx0..=bx1 {
            if g.active[i] == 0 {
                i += 1;
                continue;
            }
            let (mut fx, mut fy);
            if g.film[i] as f64 > 0.05 {
                fx = 0.2 * g.vel_x[i] as f64
                    + 0.2
                        * (g.vel_x[i - 1] as f64
                            + g.vel_x[i + 1] as f64
                            + g.vel_x[i - s] as f64
                            + g.vel_x[i + s] as f64);
                fy = 0.2 * g.vel_y[i] as f64
                    + 0.2
                        * (g.vel_y[i - 1] as f64
                            + g.vel_y[i + 1] as f64
                            + g.vel_y[i - s] as f64
                            + g.vel_y[i + s] as f64);
            } else {
                // Whatever gravity the persistent field carries passes
                // straight through.
                fx = g.vel_x[i] as f64;
                fy = g.vel_y[i] as f64;
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
            g.flow_x[i] = fx as f32;
            g.flow_y[i] = fy as f32;
            i += 1;
        }
    }
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

mod advect;
mod project;
pub use advect::advect;
pub use project::{apply_boundaries, project};
