//! **O campo de fluxo INDEPENDENTE DE ORDEM** — o terceiro passe a sair do
//! Gauss-Seidel, pela MESMA lei do [ADR-0147](../../../../docs/architecture/decisions/0147-wet-paint-order-invariant-solver.md):
//! *toda leitura cross-célula vê o estado do INÍCIO do passe*.
//!
//! # Por que este passe era sequencial, e por que deixou de ser
//!
//! A §2 do ADR-0145 o recusou com dois mecanismos, e os dois estavam certos:
//!
//! 1. **o freio LÊ `wet[probe]`** alguns pixels adiante — e o carimbo de
//!    umidade **deste mesmo passe** pode ter escrito aquela célula;
//! 2. **o backrun ESPALHA** em `susp[nb]`/`sett[nb]`/`susp_rgb[nb]`, isto é,
//!    escreve em células de OUTRAS linhas.
//!
//! A cura não é agendamento, é **decomposição**: os dois efeitos são passes
//! próprios, e o que sobra — o núcleo, que a F1 do plano 30 mediu em **99,4%
//! do custo** — vira um **gather puro que escreve `flow_x`/`flow_y` e mais
//! nada**. Nessa forma as três condições do ADR-0109 valem por construção, sem
//! precisar de revisão:
//!
//! | passe | lê | escreve |
//! |---|---|---|
//! | [`flow_rows`] | `film`, `paper`, `vel`, `active`, `wet`, `susp`, `sett`, `bloom` | `flow_x`, `flow_y` |
//! | [`wet_stamp_rows`] | `film`, `paper`, `active` | `wet` |
//! | [`backrun_rows`] | `film`, `active`, `bflags` | `bloom`, `susp`, `sett`, `susp_rgb` |
//!
//! ⚠️ **A ORDEM dos três é a lei**, não uma conveniência: o carimbo corre
//! DEPOIS do núcleo, então todo freio lê o `wet` de antes do passo; e o
//! backrun corre por último, então o portão capilar
//! (`susp[fi] + sett[fi] < cap_gate`) e o teste `sett[nb] > 0` do empurrão
//! também leem o estado de entrada.
//!
//! # O que MUDA no desenho, e o que não muda
//!
//! **Muda:** o freio deixa de ver o carimbo de umidade que uma célula
//! anterior na varredura acabou de pôr. O efeito é confinado à FRENTE que
//! avança (numa poça assentada `wet` já vale `g(paper)` de passos anteriores,
//! e o carimbo é idempotente ⇒ diferença zero) — e é exatamente o viés
//! esquerda→direita que o `tests/solver_symmetry.rs` mede.
//!
//! **Não muda:** o backrun. A extração dele é *pure code motion* e a razão é
//! aritmética — o levante
//!
//! ```text
//!   lift = sett·0,1 ;  susp += lift ;  sett −= lift ;  cor ← mix(…, lift/(susp+lift))
//! ```
//!
//! é função **só do estado da própria célula levantada**; o vizinho que o
//! dispara não entra em nenhum termo. Aplicá-lo `n` vezes é `F^n`, e `F^n`
//! independe da ordem em que os `n` gatilhos foram descobertos. O que o
//! gather precisa é só **contar** os gatilhos, e é o que ele faz.
//!
//! ⚠️ **Por isso o `bflags` existe.** Contar os gatilhos exige perguntar
//! `sett[vizinho] > 0` e `bloom[vizinho] < 6` — dois planos que o próprio
//! passe escreve. Os dois predicados são **invariantes sob o passe** (o
//! levante multiplica `sett` por 0,9, que preserva o sinal; e `bloom` só é
//! escrito pela própria célula, logo o valor que o serial lê é sempre o de
//! entrada), então materializá-los num pré-passe não é uma aproximação: é o
//! mesmo bit, computado uma vez. **Só é alocado com o knob LIGADO** —
//! `extBackrun` é `Hidden` e nasce em `0.0`.

use crate::flow::FlowGeom;
use crate::grid::{Grid, wet_byte_from_paper};
use crate::par::{self, Rows};
use crate::rng::hash2_signed;
use crate::sim::Params;
use crate::tuning::Knob;

use super::{BLOOM_SEED, clamp_sym, js_round_i64};

/// Bit 0 — a célula carrega pigmento assentado (`sett > 0`).
const BF_HAS_SETT: u8 = 1;
/// Bit 1 — a célula ainda tem orçamento de floração (`bloom < 6`).
const BF_CAN_BLOOM: u8 = 2;

/// Os knobs do passe, colhidos UMA vez (o corpo da célula é o laço mais quente
/// do motor depois do advect).
struct FlowConst {
    max_v: f64,
    level_k: f64,
    level_clamp: f64,
    cap_k: f64,
    cap_gate: f64,
    visc_threshold: f64,
    brake_bias: f64,
    brake_reach: f64,
    dif: f64,
    fingering: bool,
    gnx: f64,
    gny: f64,
    wave_len: f64,
    t_axis_x: f64,
    t_axis_y: f64,
    ext_fingering: f64,
    backrun: bool,
    ext_backrun: f64,
    mix: crate::colorops::ColorMix,
}

impl FlowConst {
    fn new(p: &Params, gx: f64, gy: f64, ext_bypass: bool, geom: &FlowGeom) -> Self {
        let g_mag = (gx * gx + gy * gy).sqrt();
        let fingering = !ext_bypass && p.k(Knob::ExtFingering) > 0.0 && g_mag > 0.0;
        let (mut gnx, mut gny, mut wave_len, mut t_axis_x, mut t_axis_y) =
            (0.0, 0.0, 1.0, 0.0, 0.0);
        if fingering {
            gnx = gx / g_mag;
            gny = gy / g_mag;
            // A largura de onda é uma FRAÇÃO da folha, então ela mede a folha
            // na grade em que o padrão é desenhado.
            wave_len = (geom.w as f64 / p.k(Knob::ExtRivulets)).max(4.0);
            t_axis_x = -gny;
            t_axis_y = gnx;
        }
        FlowConst {
            max_v: p.k(Knob::MaxVelocity),
            level_k: p.k(Knob::Leveling),
            level_clamp: p.k(Knob::LevelClamp),
            cap_k: p.k(Knob::Capillary),
            cap_gate: p.k(Knob::CapillaryGate),
            visc_threshold: p.k(Knob::Viscosity),
            brake_bias: p.k(Knob::Brake),
            brake_reach: p.k(Knob::BrakeReach),
            dif: crate::flow::diff_scale(geom.rf),
            fingering,
            gnx,
            gny,
            wave_len,
            t_axis_x,
            t_axis_y,
            ext_fingering: p.k(Knob::ExtFingering),
            backrun: !ext_bypass && p.k(Knob::ExtBackrun) > 0.0,
            ext_backrun: p.k(Knob::ExtBackrun),
            mix: p.mix,
        }
    }
}

/// **O campo de fluxo que o produto roda.**
pub fn build_flow_field_jacobi(g: &mut Grid, p: &Params, gx: f64, gy: f64, ext_bypass: bool) {
    let (bx0, by0, bx1, by1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), g.flow.rf);
    let rows = (by1 - by0 + 1).max(0) as usize;
    let span = (bx1 - bx0 + 1).max(0) as usize;
    let mode = Rows::pick(rows, span, par::MIN_CELLS_FLOW);
    build_flow_field_jacobi_rows(g, p, gx, gy, ext_bypass, mode);
}

/// [`build_flow_field_jacobi`] com a rota de caminhada FORÇADA — a porta dos
/// gates de identidade (ADR-0145). O produto chama sempre a irmã acima.
pub fn build_flow_field_jacobi_rows(
    g: &mut Grid,
    p: &Params,
    gx: f64,
    gy: f64,
    ext_bypass: bool,
    mode: Rows,
) {
    let c = FlowConst::new(p, gx, gy, ext_bypass, &g.flow);
    g.refresh_flow_spans();
    // ⚠️ A ORDEM é a lei do passe — ver o cabeçalho do módulo.
    flow_rows(g, &c, mode);
    wet_stamp_rows(g, mode);
    if c.backrun {
        backrun_rows(g, &c, mode);
    }
}

/// **O NÚCLEO — gather puro.** Escreve `flow_x`/`flow_y` e mais nada.
#[allow(clippy::too_many_lines)]
fn flow_rows(g: &mut Grid, c: &FlowConst, mode: Rows) {
    let fine_s = g.s;
    let (gw, gh) = (g.w as i32, g.h as i32);
    let geom = g.flow;
    let s = geom.s;
    let rf = geom.rf as i64;
    let cells = geom.cells as i64;
    let fine_cells = g.cells as i64;
    let (gbx0, by0, gbx1, by1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), geom.rf);
    if by0 > by1 || gbx0 > gbx1 {
        return;
    }
    let spans_on = g.spans_enabled;
    let Grid {
        frow_lo: row_lo,
        frow_hi: row_hi,
        film,
        susp,
        sett,
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
    // Tudo abaixo é LEITURA — é isto que faz do passe um gather, e o
    // compilador o verifica em vez de um comentário o prometer.
    let film_full: &[f32] = film;
    let paper_full: &[f32] = paper;
    let velx_full: &[f32] = vel_x;
    let vely_full: &[f32] = vel_y;
    let active_full: &[u8] = active;
    let wet_full: &[u8] = wet;
    let susp_full: &[f32] = susp;
    let sett_full: &[f32] = sett;
    let bloom_full: &[u8] = bloom;

    let band = by0 as usize * s..(by1 as usize + 1) * s;
    par::walk_rows2(
        mode,
        &mut flow_x[band.clone()],
        &mut flow_y[band],
        s,
        |ri, fxr, fyr| {
            let y = by0 + ri as i32;
            let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
            if bx0 > bx1 {
                return;
            }
            // As janelas de FLUXO (velocidade): linhas `y-1 / y / y+1`.
            let off = (bx0 - 1) as usize; // bx0 >= 1
            let span = (bx1 - bx0 + 3) as usize; // stencil reach 1 dos dois lados
            let base = y as usize * s;
            let row = base + off;
            let velx_m = &velx_full[row..row + span];
            let velx_u = &velx_full[row - s..row - s + span];
            let velx_d = &velx_full[row + s..row + s + span];
            let vely_m = &vely_full[row..row + span];
            let vely_u = &vely_full[row - s..row - s + span];
            let vely_d = &vely_full[row + s..row + s + span];
            // As janelas FINAS: as linhas AMOSTRADAS, e a coluna anda de `rf`
            // em `rf`. Em `rf = 1` `fy_* == y∓1` e `fx_off == off`.
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
                let k = (x - bx0) as usize + 1; // 1..=span-2
                let px = crate::flow::flow_probe(x, gw, pr);
                let kf = (px - fx_off) as usize;
                let kfl = (crate::flow::flow_probe(x - 1, gw, pr) - fx_off) as usize;
                let kfr = (crate::flow::flow_probe(x + 1, gw, pr) - fx_off) as usize;
                let fi = fy_m * fine_s + px as usize;
                if active_m[kf] == 0 {
                    i += 1;
                    continue;
                }
                let f = film_m[kf] as f64;
                let mut ex = velx_m[k] as f64;
                let mut ey = vely_m[k] as f64;

                // Leveling: a água corre do grosso para o fino, clampada por
                // eixo. (Diferença finita ⇒ leva o `dif`.)
                let mut lx = (film_m[kfl] as f64 - film_m[kfr] as f64) * c.level_k * c.dif;
                if lx > c.level_clamp {
                    lx = c.level_clamp;
                } else if lx < -c.level_clamp {
                    lx = -c.level_clamp;
                }
                let mut ly = (film_u[kf] as f64 - film_d[kf] as f64) * c.level_k * c.dif;
                if ly > c.level_clamp {
                    ly = c.level_clamp;
                } else if ly < -c.level_clamp {
                    ly = -c.level_clamp;
                }
                ex += lx;
                ey += ly;

                // Capilar: só tinta fina segue o dente do papel.
                if (susp_full[fi] as f64 + sett_full[fi] as f64) < c.cap_gate {
                    let pc = paper_m[kf] as f64;
                    let pl = paper_m[kfl] as f64;
                    let prr = paper_m[kfr] as f64;
                    ex += if pl > pc {
                        if pc < prr { pl - prr } else { pc - prr }
                    } else if pc < prr {
                        pl - pc
                    } else {
                        pl - prr
                    } * c.cap_k
                        * c.dif;
                    let pu = paper_u[kf] as f64;
                    let pd = paper_d[kf] as f64;
                    ey += if pu > pc {
                        if pc < pd { pu - pd } else { pc - pd }
                    } else if pc < pd {
                        pu - pc
                    } else {
                        pu - pd
                    } * c.cap_k
                        * c.dif;
                }

                // Viscosidade: água funda arrasta os vizinhos do campo
                // persistente. É uma MÉDIA — adimensional, sem `dif`.
                if f > c.visc_threshold {
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

                // ⚠️ O CARIMBO DE UMIDADE não mora aqui — ver `wet_stamp_rows`.

                // Fingering (extensão, PRÉ-freio).
                if c.fingering && f > 0.1 {
                    let dxs = js_round_i64(c.gnx);
                    let dys = js_round_i64(c.gny);
                    let j = i as i64 + dxs + dys * s as i64;
                    let jf = fi as i64 + dxs * rf + dys * rf * fine_s as i64;
                    if j >= 0
                        && j < cells
                        && jf >= 0
                        && jf < fine_cells
                        && film_full[jf as usize] as f64 <= 0.01
                    {
                        let phase = (2.0
                            * std::f64::consts::PI
                            * (x as f64 * c.t_axis_x + y as f64 * c.t_axis_y))
                            / c.wave_len;
                        let push = c.ext_fingering * f.min(2.0);
                        let sn = libm::sin(phase);
                        let cs = libm::cos(phase);
                        ex += c.gnx * push * sn + c.t_axis_x * push * 0.3 * cs;
                        ey += c.gny * push * sn + c.t_axis_y * push * 0.3 * cs;
                    }
                }

                // Freio de absorvência por look-ahead.
                //
                // ⚠️ **`wet_full` é o `wet` do INÍCIO do passo** — é aqui que a
                // independência de ordem é ganha, e é a única diferença de
                // desenho entre este passe e o Gauss-Seidel.
                let s_len = c.brake_reach * c.dif / ((ex * ex + ey * ey).sqrt() + 0.01);
                let dxs = (ex * s_len).trunc() as i64;
                let dys = (ey * s_len).trunc() as i64;
                let probe = i as i64 + dxs + dys * s as i64;
                let probe_f = fi as i64 + dxs * rf + dys * rf * fine_s as i64;
                if probe >= 0 && probe < cells && probe_f >= 0 && probe_f < fine_cells {
                    let pu = probe_f as usize;
                    let mut brake =
                        film_full[pu] as f64 + (3.0 / 255.0) * wet_full[pu] as f64 - c.brake_bias;
                    if brake < 0.05 {
                        brake = 0.05;
                    } else if brake > 1.0 {
                        brake = 1.0;
                    }
                    ex *= brake;
                    ey *= brake;
                }

                // Backrun (extensão, PÓS-freio) — **só o EMPURRÃO**. O levante
                // e o orçamento de floração são de `backrun_rows`, e é isso
                // que tira a escrita cross-linha daqui.
                if c.backrun && bloom_full[fi] < 6 {
                    let thr = 0.8 + 0.2 * hash2_signed(x, y, BLOOM_SEED);
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
                        let nb = (fi as i64 + dxn * rf + dyn_ * rf * fine_s as i64) as usize;
                        let gap = f - film_full[nb] as f64;
                        if gap > thr && sett_full[nb] as f64 > 0.0 {
                            let push = c.ext_backrun * gap.min(1.5) * 0.5;
                            if dxn != 0 {
                                ex += dxn as f64 * push;
                            } else {
                                ey += dyn_ as f64 * push;
                            }
                        }
                    }
                }

                fxr[x as usize] = clamp_sym(ex, c.max_v) as f32;
                fyr[x as usize] = clamp_sym(ey, c.max_v) as f32;
                i += 1;
            }
        },
    );
}

/// **O CARIMBO DE UMIDADE** — film fundo marca a folha úmida, os vales mais
/// ainda, e o fluxo futuro segue os canais já molhados.
///
/// ⚠️ **É um passe próprio porque a condição e o valor dependem SÓ de planos
/// que ninguém escreve** (`film`, `paper`, `active`): extraí-lo custa uma
/// varredura e devolve um `wet` que o núcleo pode ler sem ordem.
fn wet_stamp_rows(g: &mut Grid, mode: Rows) {
    let fine_s = g.s;
    let (gw, gh) = (g.w as i32, g.h as i32);
    let geom = g.flow;
    let (gbx0, by0, gbx1, by1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), geom.rf);
    if by0 > by1 || gbx0 > gbx1 {
        return;
    }
    let spans_on = g.spans_enabled;
    let Grid {
        frow_lo: row_lo,
        frow_hi: row_hi,
        film,
        paper,
        active,
        wet,
        ..
    } = g;
    let film_full: &[f32] = film;
    let paper_full: &[f32] = paper;
    let active_full: &[u8] = active;
    // ⚠️ A faixa é a das linhas FINAS que os probes tocam. Linhas de fluxo
    // distintas amostram linhas finas distintas (o `flow_probe` é injetivo por
    // eixo), então as fatias seguem disjuntas.
    let fy_lo = crate::flow::flow_probe(by0, gh, geom.rf) as usize;
    let fy_hi = crate::flow::flow_probe(by1, gh, geom.rf) as usize;
    let band = fy_lo * fine_s..(fy_hi + 1) * fine_s;
    par::walk_rows(mode, &mut wet[band], fine_s, |ri, wr| {
        let fy = fy_lo + ri;
        // Esta linha fina é amostrada por alguma linha de fluxo?
        let y = crate::flow::fine_to_flow(fy as i32, geom.rf);
        if crate::flow::flow_probe(y, gh, geom.rf) as usize != fy || y < by0 || y > by1 {
            return;
        }
        let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
        if bx0 > bx1 {
            return;
        }
        for x in bx0..=bx1 {
            let px = crate::flow::flow_probe(x, gw, geom.rf) as usize;
            let fi = fy * fine_s + px;
            if active_full[fi] == 0 {
                continue;
            }
            if film_full[fi] as f64 > 3.0 {
                wr[px] = wet_byte_from_paper(paper_full[fi] as f64);
            }
        }
    });
}

/// Os dois predicados que o backrun pergunta aos VIZINHOS, materializados
/// antes de qualquer escrita — ver o cabeçalho do módulo para a prova de que
/// os dois são invariantes sob o passe.
fn bflag_rows(g: &mut Grid, mode: Rows) {
    let s = g.s;
    // A janela é a bbox FINA dilatada de um bloco: o levante alcança vizinhos
    // fora da região ativa.
    let rf = g.flow.rf as i32;
    let (y0, y1) = (
        (g.by0 - rf).max(0) as usize,
        (g.by1 + rf).min(g.h as i32 + 1) as usize,
    );
    if y0 > y1 {
        return;
    }
    let Grid {
        scratch,
        sett,
        bloom,
        ..
    } = g;
    let sett_full: &[f32] = sett;
    let bloom_full: &[u8] = bloom;
    let band = y0 * s..(y1 + 1) * s;
    par::walk_rows(mode, &mut scratch.bflags[band], s, |ri, row| {
        let base = (y0 + ri) * s;
        for (x, cell) in row.iter_mut().enumerate() {
            let i = base + x;
            *cell = u8::from(sett_full[i] > 0.0) * BF_HAS_SETT
                + u8::from(bloom_full[i] < 6) * BF_CAN_BLOOM;
        }
    });
}

/// **O BACKRUN** — o levante do pigmento assentado e o orçamento de floração.
///
/// ⚠️ **Gather:** onde o serial fazia a célula-gatilho ESCREVER nos quatro
/// vizinhos, aqui cada célula CONTA quantos vizinhos a disparam e aplica o
/// levante esse número de vezes. É o mesmo número — ver o cabeçalho.
fn backrun_rows(g: &mut Grid, c: &FlowConst, mode: Rows) {
    let fine_s = g.s;
    let (gw, gh) = (g.w as i32, g.h as i32);
    let geom = g.flow;
    let (gbx0, by0, gbx1, by1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), geom.rf);
    if by0 > by1 || gbx0 > gbx1 {
        return;
    }
    g.scratch.ensure_backrun(g.cells);
    bflag_rows(g, mode);

    let spans_on = g.spans_enabled;
    // ⚠️ O EMPURRÃO (`ext_backrun`) é do núcleo — ele escreve `flow`, e escrever
    // `flow` aqui seria a segunda porta do mesmo número. Aqui só o levante e o
    // orçamento de floração, que são as duas escritas cross-linha.
    let mix = c.mix;
    let Grid {
        scratch,
        frow_lo: row_lo,
        frow_hi: row_hi,
        film,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        active,
        bloom,
        ..
    } = g;
    let film_full: &[f32] = film;
    let active_full: &[u8] = active;
    let sett_rgb_full: &[[f32; 3]] = sett_rgb;
    let bflags: &[u8] = &scratch.bflags;

    // ⚠️ **Tudo aqui caminha em espaço de FLUXO**, e não por conveniência: os
    // probes de duas células de fluxo vizinhas distam exatamente `rf` células
    // finas, então *o vizinho fino do backrun É a célula de fluxo ao lado*. É
    // isso que deixa o limiar do gatilho sair das coordenadas dele sem
    // nenhuma conversão de volta.
    let probe_row = |y: i32| crate::flow::flow_probe(y, gh, geom.rf) as usize;

    // (a) O ORÇAMENTO DE FLORAÇÃO — cada célula decide o próprio `bloom` a
    //     partir dos SEUS quatro vizinhos, exatamente como o serial fazia.
    let (bl_lo, bl_hi) = (probe_row(by0), probe_row(by1));
    par::walk_rows(
        mode,
        &mut bloom[bl_lo * fine_s..(bl_hi + 1) * fine_s],
        fine_s,
        |ri, br| {
            let fy = bl_lo + ri;
            let y = crate::flow::fine_to_flow(fy as i32, geom.rf);
            if probe_row(y) != fy || y < by0 || y > by1 {
                return;
            }
            let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
            if bx0 > bx1 {
                return;
            }
            for x in bx0..=bx1 {
                let px = crate::flow::flow_probe(x, gw, geom.rf) as usize;
                let fi = fy * fine_s + px;
                if active_full[fi] == 0 {
                    continue;
                }
                let f = film_full[fi] as f64;
                let thr = 0.8 + 0.2 * hash2_signed(x, y, BLOOM_SEED);
                let can_bloom = bflags[fi] & BF_CAN_BLOOM != 0;
                let mut met = false;
                for (dxn, dyn_) in NB4 {
                    let nb = probe_row(y + dyn_ as i32) * fine_s
                        + crate::flow::flow_probe(x + dxn as i32, gw, geom.rf) as usize;
                    if f - film_full[nb] as f64 > thr && bflags[nb] & BF_HAS_SETT != 0 {
                        met = true;
                    }
                }
                if met {
                    if can_bloom {
                        br[px] += 1;
                    }
                } else {
                    br[px] = 0;
                }
            }
        },
    );

    // (b) O LEVANTE — cada célula CONTA quantos vizinhos a disparam e aplica
    //     `F` esse número de vezes. A janela é a das células de fluxo
    //     dilatada de UMA, porque o alvo de um levante não precisa estar vivo.
    let (lo, hi) = (
        probe_row((by0 - 1).max(0)),
        probe_row((by1 + 1).min(gh + 1)),
    );
    let band = lo * fine_s..(hi + 1) * fine_s;
    par::walk_rows3(
        mode,
        &mut susp[band.clone()],
        &mut sett[band.clone()],
        &mut susp_rgb[band],
        fine_s,
        |ri, ur, tr, cr| {
            let fy = lo + ri;
            let y = crate::flow::fine_to_flow(fy as i32, geom.rf);
            if probe_row(y) != fy {
                return;
            }
            let mut out = [0.0f64; 3];
            for x in (gbx0 - 1).max(0)..=(gbx1 + 1).min(geom.w as i32 + 1) {
                let px = crate::flow::flow_probe(x, gw, geom.rf) as usize;
                let fi = fy * fine_s + px;
                if bflags[fi] & BF_HAS_SETT == 0 {
                    continue;
                }
                let mut n = 0u32;
                for (dxn, dyn_) in NB4 {
                    // O GATILHO é o VIZINHO: ele é quem tem de estar vivo, ter
                    // orçamento, e ver o degrau de film contra ESTA célula.
                    let (dx, dy) = (x + dxn as i32, y + dyn_ as i32);
                    if dx < 0 || dy < 0 || dx > geom.w as i32 + 1 || dy > geom.h as i32 + 1 {
                        continue;
                    }
                    let du =
                        probe_row(dy) * fine_s + crate::flow::flow_probe(dx, gw, geom.rf) as usize;
                    if active_full[du] == 0 || bflags[du] & BF_CAN_BLOOM == 0 {
                        continue;
                    }
                    let thr = 0.8 + 0.2 * hash2_signed(dx, dy, BLOOM_SEED);
                    if film_full[du] as f64 - film_full[fi] as f64 > thr {
                        n += 1;
                    }
                }
                // `F^n` — e `F` não conhece o gatilho, então a ordem em que os
                // `n` foram descobertos não entra em nenhum termo.
                let sc = sett_rgb_full[fi];
                for _ in 0..n {
                    let lift = tr[px] as f64 * 0.1;
                    let w = lift / (ur[px] as f64 + lift);
                    let uc = cr[px];
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
                    cr[px] = [out[0] as f32, out[1] as f32, out[2] as f32];
                    ur[px] = (ur[px] as f64 + lift) as f32;
                    tr[px] = (tr[px] as f64 - lift) as f32;
                }
            }
        },
    );
}

/// Os quatro vizinhos, na ORDEM do serial (esquerda, direita, cima, baixo).
const NB4: [(i64, i64); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
