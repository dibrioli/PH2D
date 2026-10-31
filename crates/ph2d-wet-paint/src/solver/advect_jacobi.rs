//! **O ADVECT COMO GATHER PURO** — a mesma advecção conservativa, reformulada
//! para que nenhuma célula escreva na célula de outra (doc 28 §5.45).
//!
//! ⚠️ **Isto NÃO é o [`super::advect`] mais rápido: é um SEGUNDO MODELO**, e a
//! diferença tem nome. O serial é um **Gauss-Seidel**: ele varre em ordem de
//! raster e SUBTRAI nos quatro cantos-fonte, então o canto que o destino
//! `x+1` lê já foi drenado pelo destino `x`. O resultado depende da ordem da
//! varredura — e é exatamente por isso que ele não paraleliza e não vai para
//! a GPU ([ADR-0146](../../../../docs/architecture/decisions/0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md)).
//! Aqui **todo mundo lê o estado do INÍCIO do passo**: é a forma de Jacobi, é
//! independente de ordem, e é a única que 32 núcleos (ou 10.000) podem
//! responder juntos.
//!
//! ## Como um scatter vira um gather sem atômicos
//!
//! O destino `d` puxa a fração `w_k` de cada canto `k`. Escrito como está,
//! isso é uma escrita em célula alheia. Mas a relação é **simétrica**: se `d`
//! puxa `w` de `c`, então `c` **dá** `w` a `d`. Logo
//!
//! ```text
//!   novo[c] = velho[c] · (1 − saída[c])  +  Σ_k w_k · velho[canto_k]
//!             \_______ o que SAI ______/    \____ o que ENTRA ______/
//! ```
//!
//! e `saída[c] = Σ_d w(d→c)` é ela própria um **gather** sobre a vizinhança de
//! `c`, porque a velocidade é limitada (`|u| ≤ maxVelocity`) e portanto todo
//! destino que alcança `c` está a no máximo [`stencil_radius`] células dele.
//! Nada de atômicos, nada de ordem.
//!
//! ⚠️ **A saída pode passar de 1** onde o fluxo converge (dois destinos
//! puxando forte da mesma célula), e aí a célula deve `1` de massa que não
//! tem. O serial nunca vê isso porque drena EM ORDEM: o segundo destino já
//! encontra o canto vazio. Aqui a cura é uma escala — quem sai leva
//! `min(saída, 1)`, e **quem recebe pergunta a escala da FONTE**
//! ([`take_scale`]), não a própria. É isso que mantém a soma exata: o total
//! que os destinos tomam de `c` é `saída[c] · escala[c] = min(saída[c], 1)`,
//! que é exatamente o que `c` perdeu. *Um clamp só no lado de quem sai
//! CRIARIA massa; um clamp só no lado de quem entra a DESTRUIRIA.*
//!
//! ## As quatro passadas
//!
//! 1. [`momentum_rows`] — a velocidade persistente (gravidade), na grade de
//!    FLUXO. Ela saiu do laço fino porque agora **pode**: a objeção que o
//!    serial documenta (*"o `f` que a gravidade multiplica depende de onde o
//!    laço está"*) era uma consequência da ordem, e Jacobi não tem ordem.
//! 2. [`outflow_rows`] — quanto sai de cada célula.
//! 3. [`transport_rows`] — o novo estado, no rascunho; e o `vmax`.
//! 4. [`commit_rows`] — o rascunho vira o grid.
//!
//! ⚠️ **Nenhuma delas alarga a janela varrida**, e isso não é sorte: a faixa
//! viva é a extensão da água **dilatada por [`crate::grid::SPAN_PAD`] = 5** e
//! a água anda no máximo 3 células por passo, então todo canto de um destino
//! ativo já está dentro da faixa que o serial varria. Fora dela `film` e
//! `susp` são zero por invariante (`verify_spans`), então uma `saída` velha
//! ali multiplica zero.

use crate::colorops::km_weighted_mean_color;
use crate::flow::{flow_at_cell, flow_at_point};
use crate::grid::{Grid, SolverScratch, span_x_of};
use crate::par::{self, Rows};
use crate::sim::Params;
use crate::tuning::Knob;
use rayon::prelude::*;

/// **A que distância pode estar um destino que puxa massa DAQUI**, em células.
///
/// ⚠️ **Não é `ceil(maxVelocity)`, e o off-by-one aqui CRIA massa.** O
/// retro-traço é `s = x − u` com `|u| ≤ M`, e os cantos de peso não-nulo são
/// `floor(s)` e `floor(s)+1`. Com `M = 1` (o **default** do knob) e `u = −1` o
/// destino em `x` puxa dos cantos `x+1` e **`x+2`** — logo a célula `c` é
/// alcançada por um destino em `c−2`. Uma vizinhança de raio `ceil(M)` o
/// perderia, a saída sairia subestimada, e a massa que os destinos tomam
/// deixaria de casar com a que a fonte perde: **o gather deixaria de
/// conservar**.
///
/// O alcance é, portanto, `ceil(M) + 1`.
#[inline]
#[must_use]
pub fn destination_reach(max_v: f64) -> i32 {
    let r = max_v.ceil() as i32;
    (if r < 1 { 1 } else { r }) + 1
}

/// O retro-traço de um destino: a coluna/linha base e a fração.
///
/// `None` quando ele deixa a folha — o serial faz `continue` ali, e a célula
/// então não puxa nada **e** não escreve velocidade (mas continua podendo ser
/// drenada por quem alcança ela).
#[inline]
fn backtrace(x: i32, y: i32, ux: f64, uy: f64, w: f64, h: f64) -> Option<(i64, f64, i64, f64)> {
    let sx = f64::from(x) - ux;
    let sy = f64::from(y) - uy;
    if sx < 1.0 || sx > w || sy < 1.0 || sy > h {
        return None;
    }
    let x0 = sx as i64; // positivo ⇒ trunc == floor (o mesmo do serial)
    let y0 = sy as i64;
    Some((x0, sx - x0 as f64, y0, sy - y0 as f64))
}

/// O peso com que um destino cujo retro-traço caiu em `(base, frac)` puxa da
/// coordenada `c` — a forma SEPARÁVEL dos quatro `w00..w11` do serial, termo
/// a termo (`w10 = fx·(1−fy)` sai de `frac · (1−frac_y)`, na mesma ordem).
#[inline]
fn corner_weight(c: i64, base: i64, frac: f64) -> f64 {
    if c == base {
        1.0 - frac
    } else if c == base + 1 {
        frac
    } else {
        0.0
    }
}

/// A escala que a FONTE impõe a quem toma dela (ver o §"a saída pode passar
/// de 1" do módulo). `1` no caso normal.
#[inline]
fn take_scale(outflow: f64) -> f64 {
    if outflow > 1.0 { 1.0 / outflow } else { 1.0 }
}

/// A rota do produto: a mesma resposta, o piso medido decide o caminho.
pub fn advect_jacobi(g: &mut Grid, p: &Params, gx: f64, gy: f64) -> f64 {
    let rows = (g.by1 - g.by0 + 1).max(0) as usize;
    let span = (g.bx1 - g.bx0 + 1).max(0) as usize;
    advect_jacobi_rows(
        g,
        p,
        gx,
        gy,
        Rows::pick_medido(rows, span, par::MIN_CELLS_ADVECT),
    )
}

/// [`advect_jacobi`] com a rota forçada — a porta dos gates de identidade.
pub fn advect_jacobi_rows(g: &mut Grid, p: &Params, gx: f64, gy: f64, mode: Rows) -> f64 {
    if g.by0 > g.by1 {
        return 0.0;
    }
    g.scratch.ensure(g.cells);
    let fundir = fundida::usa_fundida(mode, (g.by1 - g.by0 + 1) as usize * g.s);
    if !(fundir && fundida::momento_no_bloco(g)) {
        momentum_rows(g, p, gx, gy, mode);
    }
    let vmax = if fundir {
        fundida::fused_rows(
            g,
            p,
            mode,
            (gx, gy),
            BLOCO_LINHAS.load(std::sync::atomic::Ordering::Relaxed),
        )
    } else {
        let vmax = prepare_rows(g, p, mode);
        outflow_rows(g, p, mode);
        transport_rows(g, p, mode);
        vmax
    };
    commit_rows(g, mode);
    vmax
}

/// As fatias e constantes que as fases por linha leem — todas do estado do INÍCIO do passo (ADR-0147).
pub(super) struct Ctx<'a> {
    pub(super) s: usize,
    wf: f64,
    hf: f64,
    pub(super) gh: i32,
    pub(super) reach: i32,
    km_mean: bool,
    pub(super) by0: i32,
    pub(super) by1: i32,
    geom: crate::flow::FlowGeom,
    flow_x: &'a [f32],
    flow_y: &'a [f32],
    film: &'a [f32],
    susp: &'a [f32],
    susp_rgb: &'a [[f32; 3]],
    active: &'a [u8],
    row_lo: &'a [i32],
    row_hi: &'a [i32],
    spans_enabled: bool,
    bx0: i32,
    bx1: i32,
    gw: i32,
    max_v: f64,
    /// A bbox na grade de FLUXO ([`crate::flow::flow_bbox`]).
    fb: (i32, i32, i32, i32),
}

impl Ctx<'_> {
    #[inline]
    fn span(&self, y: i32) -> (i32, i32) {
        span_x_of(
            self.row_lo,
            self.row_hi,
            self.spans_enabled,
            self.bx0,
            self.bx1,
            y,
        )
    }
}

/// O grid partido em leitura ([`Ctx`]) e rascunho (escrita).
pub(super) fn split<'a>(g: &'a mut Grid, p: &Params) -> (Ctx<'a>, &'a mut SolverScratch) {
    let (c, scratch, _, _) = split_vel(g, p);
    (c, scratch)
}

/// [`split`] com os planos da velocidade persistente (o que o [`momentum_row`] escreve).
pub(super) fn split_vel<'a>(
    g: &'a mut Grid,
    p: &Params,
) -> (Ctx<'a>, &'a mut SolverScratch, &'a mut [f32], &'a mut [f32]) {
    let (s, gh, wf, hf) = (g.s, g.h as i32, g.w as f64, g.h as f64);
    let gw = g.w as i32;
    let fb = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), g.flow.rf);
    let (by0, by1, bx0, bx1) = (g.by0, g.by1, g.bx0, g.bx1);
    let (spans_enabled, geom) = (g.spans_enabled, g.flow);
    let Grid {
        scratch,
        film,
        susp,
        susp_rgb,
        active,
        flow_x,
        flow_y,
        row_lo,
        row_hi,
        vel_x,
        vel_y,
        ..
    } = g;
    let c = Ctx {
        s,
        wf,
        hf,
        gh,
        reach: destination_reach(p.k(Knob::MaxVelocity)),
        km_mean: p.km_mixing,
        by0,
        by1,
        geom,
        flow_x,
        flow_y,
        film,
        susp,
        susp_rgb,
        active,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        gw,
        max_v: p.k(Knob::MaxVelocity),
        fb,
    };
    (c, scratch, vel_x, vel_y)
}

/// **O fluxo fino, materializado uma vez** — e o `vmax` de carona.
///
/// ⚠️ Esta passada é o que separa um gather viável de um brinquedo: sem ela a
/// vizinhança 3×3 do [`outflow_rows`] re-amostra a grade de fluxo **nove
/// vezes por célula**, e a rota serial do gather custava **180,8 ms contra
/// 36,0 do Gauss-Seidel** (medido, `Flow 4`). O amostrador é o mesmo
/// [`crate::flow::FlowRowSampler`] que o serial usa — uma resposta só.
fn prepare_rows(g: &mut Grid, p: &Params, mode: Rows) -> f64 {
    let (c, scratch) = split(g, p);
    let band = c.by0 as usize * c.s..(c.by1 as usize + 1) * c.s;
    walk_rows2_reduce(
        mode,
        &mut scratch.uv[band.clone()],
        &mut scratch.bt[band],
        c.s,
        0.0f64,
        |ri, row, bt| prepare_row(&c, c.by0 + ri as i32, row, bt),
        f64::max,
    )
}

/// Uma linha do [`prepare_rows`]: `row`/`bt` são a linha `y` inteira (índice = coluna).
pub(super) fn prepare_row(c: &Ctx, y: i32, row: &mut [[f32; 2]], bt: &mut [i8]) -> f64 {
    let (lo, hi) = c.span(y);
    if lo > hi {
        return 0.0;
    }
    let mut sampler = crate::flow::FlowRowSampler::new(&c.geom, y);
    let mut vmax = 0.0f64;
    let mut i = lo as usize + y as usize * c.s;
    for x in lo..=hi {
        let (ux, uy) = sampler.at(c.flow_x, c.flow_y, x, i, c.geom.s);
        let uvf = [ux as f32, uy as f32];
        row[x as usize] = uvf;
        // A linha-base do retro-traço, do `uv` JÁ arredondado — o que o `outflow_rows` lê.
        bt[x as usize] = match backtrace(x, y, f64::from(uvf[0]), f64::from(uvf[1]), c.wf, c.hf) {
            Some((_, _, y0, _)) if c.active[i] != 0 => {
                i8::try_from(i64::from(y) - y0).unwrap_or(SolverScratch::BT_NADA)
            }
            _ => SolverScratch::BT_NADA,
        };
        // O `vmax` é o do SERIAL: só célula ativa, e antes da checagem de sair da folha.
        if c.active[i] != 0 {
            let axv = ux.abs();
            let ayv = uy.abs();
            if axv > vmax {
                vmax = axv;
            }
            if ayv > vmax {
                vmax = ayv;
            }
        }
        i += 1;
    }
    vmax
}

/// **A velocidade persistente**, na grade de FLUXO: o fluxo amostrado na
/// origem do retro-traço mais a gravidade, escalada pela água daqui.
///
/// Quem escreve uma célula de fluxo é a célula fina **PROBE** dela (plano 30),
/// e cada linha de fluxo tem exatamente uma linha fina probe ⇒ as linhas são
/// disjuntas.
fn momentum_rows(g: &mut Grid, p: &Params, gx: f64, gy: f64, mode: Rows) {
    let (c, _, vel_x, vel_y) = split_vel(g, p);
    let (_, fby0, _, fby1) = c.fb;
    if fby0 > fby1 || c.fb.0 > c.fb.2 {
        return;
    }
    let band = fby0 as usize * c.geom.s..(fby1 as usize + 1) * c.geom.s;
    par::walk_rows2(
        mode,
        &mut vel_x[band.clone()],
        &mut vel_y[band],
        c.geom.s,
        |r, vx, vy| momentum_row(&c, fby0 + r as i32, vx, vy, gx, gy),
    );
}

/// Uma linha de FLUXO `cy` do [`momentum_rows`]; `vx`/`vy` são a linha inteira. Lê `film`/`active` só na
/// linha fina probe dela e o fluxo no retro-traço — nada do que as outras fases escrevem.
pub(super) fn momentum_row(c: &Ctx, cy: i32, vx: &mut [f32], vy: &mut [f32], gx: f64, gy: f64) {
    let (fbx0, _, fbx1, _) = c.fb;
    let (s, geom, max_v) = (c.s, &c.geom, c.max_v);
    let py = crate::flow::flow_probe(cy, c.gh, geom.rf);
    if py < c.by0 || py > c.by1 {
        return;
    }
    let (lo, hi) = c.span(py);
    if lo > hi {
        return;
    }
    for cx in fbx0..=fbx1 {
        let px = crate::flow::flow_probe(cx, c.gw, geom.rf);
        if px < lo || px > hi {
            continue;
        }
        let i = px as usize + py as usize * s;
        let col = cx as usize;
        if c.active[i] == 0 {
            vx[col] = 0.0;
            vy[col] = 0.0;
            continue;
        }
        let (ux, uy) = flow_at_cell(c.flow_x, c.flow_y, geom, px, py, i);
        // Sai da folha ⇒ o serial faz `continue` ANTES de escrever a
        // velocidade: uma gota que alcança a borda mantém o momento.
        if backtrace(px, py, ux, uy, c.wf, c.hf).is_none() {
            continue;
        }
        let sx = f64::from(px) - ux;
        let sy = f64::from(py) - uy;
        let f = c.film[i] as f64;
        let (sfx, sfy) = flow_at_point(c.flow_x, c.flow_y, geom, sx, sy);
        let mut nvx = sfx + gx * f;
        let mut nvy = sfy + gy * f;
        if nvx > max_v {
            nvx = max_v;
        } else if nvx < -max_v {
            nvx = -max_v;
        }
        if nvy > max_v {
            nvy = max_v;
        } else if nvy < -max_v {
            nvy = -max_v;
        }
        vx[col] = nvx as f32;
        vy[col] = nvy as f32;
    }
}

/// **Quanto sai de cada célula** — a soma dos pesos com que os destinos da
/// vizinhança a puxam.
///
/// ⚠️ **É um SCATTER, e ele é seguro porque o alvo é PRIVADO da linha.** A
/// forma óbvia — cada célula varrer a própria vizinhança e somar — custa
/// `(2·alcance+1)²` retro-traços por célula, isto é **25** no default (ver
/// [`destination_reach`]); medida, ela punha a rota serial do gather em ~200
/// ms. Virando o laço do avesso — cada DESTINO deposita os seus dois pesos na
/// linha de saída — o custo cai para `2·alcance+1` retro-traços, **5**, e cada
/// um deposita em duas colunas de um acumulador que é a própria fatia
/// `&mut [f32]` desta linha. Nenhuma outra tarefa a enxerga.
///
/// ⚠️ **A ordem da soma é fixa** (linha-fonte crescente, depois coluna), então
/// as rotas serial e paralela produzem o MESMO `f64` — e é isso que faz o gate
/// de identidade valer apesar de a soma ser em ponto flutuante.
fn outflow_rows(g: &mut Grid, p: &Params, mode: Rows) {
    let (c, scratch) = split(g, p);
    let band = c.by0 as usize * c.s..(c.by1 as usize + 1) * c.s;
    let SolverScratch {
        uv, bt, outflow, ..
    } = scratch;
    let (uv, bt): (&[[f32; 2]], &[i8]) = (uv, bt);
    par::walk_rows(mode, &mut outflow[band], c.s, |ri, row| {
        outflow_row(&c, c.by0 + ri as i32, row, uv, bt, 0, 1, c.gh);
    });
}

/// Uma linha do [`outflow_rows`]. `uv`/`bt` guardam as linhas `ny_lo..=ny_hi` a partir do índice global
/// `ubase` (`0` = a grelha inteira); só elas são candidatas.
#[allow(clippy::too_many_arguments)]
pub(super) fn outflow_row(
    c: &Ctx,
    y: i32,
    row: &mut [f32],
    uv: &[[f32; 2]],
    bt: &[i8],
    ubase: usize,
    ny_lo: i32,
    ny_hi: i32,
) {
    let (lo, hi) = c.span(y);
    if lo > hi {
        return;
    }
    let (lo_u, hi_u) = (lo as usize, hi as usize);
    row[lo_u..=hi_u].fill(0.0);
    for ny in (y - c.reach).max(ny_lo)..=(y + c.reach).min(ny_hi) {
        let (slo, shi) = c.span(ny);
        if slo > shi {
            continue;
        }
        // O destino puxa desta linha se a base dele é `y` (`ny − y0 = ny − y`) ou `y − 1`.
        let (d_base, d_cima) = ((ny - y) as i8, (ny - y + 1) as i8);
        let rowbase = ny as usize * c.s - ubase;
        for nx in slo..=shi {
            let di = nx as usize + rowbase;
            let d = bt[di];
            if d != d_base && d != d_cima {
                continue;
            }
            let uvd = uv[di];
            let Some((x0, fx, y0, fy)) =
                backtrace(nx, ny, f64::from(uvd[0]), f64::from(uvd[1]), c.wf, c.hf)
            else {
                continue;
            };
            let wy = corner_weight(i64::from(y), y0, fy);
            if wy == 0.0 {
                continue;
            }
            // As duas colunas-fonte deste destino. Fora da faixa desta
            // linha não há massa (invariante), então o peso perdido
            // multiplicaria zero.
            let c0 = x0 as usize;
            if x0 >= lo as i64 && x0 <= hi as i64 {
                row[c0] += ((1.0 - fx) * wy) as f32;
            }
            if x0 >= lo as i64 - 1 && x0 < hi as i64 {
                row[c0 + 1] += (fx * wy) as f32;
            }
        }
    }
}

/// **O novo estado**, no rascunho.
fn transport_rows(g: &mut Grid, p: &Params, mode: Rows) {
    let (c, scratch) = split(g, p);
    let band = c.by0 as usize * c.s..(c.by1 as usize + 1) * c.s;
    let SolverScratch {
        uv,
        outflow,
        dst_film,
        dst_susp,
        dst_rgb,
        ..
    } = scratch;
    let (uv, outflow): (&[[f32; 2]], &[f32]) = (uv, outflow);
    par::walk_rows3(
        mode,
        &mut dst_film[band.clone()],
        &mut dst_susp[band.clone()],
        &mut dst_rgb[band],
        c.s,
        |ri, fr, sr, cr| transport_row(&c, c.by0 + ri as i32, fr, sr, cr, outflow, 0, uv, 0),
    );
}

/// Uma linha do [`transport_rows`]: `fr`/`sr`/`cr` são a linha `y` inteira; `outflow` e `uv` começam no
/// índice global `obase`/`ubase`.
#[allow(clippy::too_many_arguments)]
pub(super) fn transport_row(
    c: &Ctx,
    y: i32,
    fr: &mut [f32],
    sr: &mut [f32],
    cr: &mut [[f32; 3]],
    outflow: &[f32],
    obase: usize,
    uv: &[[f32; 2]],
    ubase: usize,
) {
    let s = c.s;
    let (film, susp, susp_rgb) = (c.film, c.susp, c.susp_rgb);
    let (lo, hi) = c.span(y);
    if lo > hi {
        return;
    }
    let mut km_colors = [0.0f64; 12];
    let mut km_weights = [0.0f64; 4];
    let mut km_out = [0.0f64; 3];
    let mut i = lo as usize + y as usize * s;
    for x in lo..=hi {
        let f0 = film[i] as f64;
        let m0 = susp[i] as f64;
        // O que SAI (vale para toda célula com massa, ativa ou não —
        // um canto inativo é drenado por quem o alcança).
        let of = outflow[i - obase] as f64;
        let leave = if of > 1.0 { 1.0 } else { of };
        let mut new_film = f0 - f0 * leave;
        let mut new_susp = m0 - m0 * leave;
        let mut new_rgb = susp_rgb[i];
        // O que ENTRA (só quem é ativo puxa).
        if c.active[i] != 0 {
            let uvc = uv[i - ubase];
            let (ux, uy) = (f64::from(uvc[0]), f64::from(uvc[1]));
            if let Some((x0, fx, y0, fy)) = backtrace(x, y, ux, uy, c.wf, c.hf) {
                let i00 = x0 as usize + y0 as usize * s;
                let i10 = i00 + 1;
                let i01 = i00 + s;
                let i11 = i01 + 1;
                let w00 = (1.0 - fx) * (1.0 - fy);
                let w10 = fx * (1.0 - fy);
                let w01 = (1.0 - fx) * fy;
                let w11 = fx * fy;
                // ⚠️ A escala é a da FONTE — ver o módulo. Com as quatro saídas ≤ 1 (o caso
                // comum) a escala é `1` e `w · 1 == w` ao bit: um teste em vez de quatro.
                let (o00, o10, o01, o11) = (
                    outflow[i00 - obase],
                    outflow[i10 - obase],
                    outflow[i01 - obase],
                    outflow[i11 - obase],
                );
                let (k00, k10, k01, k11) = if o00.max(o10).max(o01.max(o11)) <= 1.0 {
                    (w00, w10, w01, w11)
                } else {
                    (
                        w00 * take_scale(o00 as f64),
                        w10 * take_scale(o10 as f64),
                        w01 * take_scale(o01 as f64),
                        w11 * take_scale(o11 as f64),
                    )
                };
                let p00 = susp[i00] as f64 * k00;
                let p10 = susp[i10] as f64 * k10;
                let p01 = susp[i01] as f64 * k01;
                let p11 = susp[i11] as f64 * k11;
                let want = p00 + p10 + p01 + p11;
                if want > 0.0 {
                    let inv = 1.0 / want;
                    let c00 = susp_rgb[i00];
                    let c10 = susp_rgb[i10];
                    let c01 = susp_rgb[i01];
                    let c11 = susp_rgb[i11];
                    let (r_in, g_in, b_in);
                    if c.km_mean {
                        km_colors[0] = c00[0] as f64;
                        km_colors[1] = c00[1] as f64;
                        km_colors[2] = c00[2] as f64;
                        km_colors[3] = c10[0] as f64;
                        km_colors[4] = c10[1] as f64;
                        km_colors[5] = c10[2] as f64;
                        km_colors[6] = c01[0] as f64;
                        km_colors[7] = c01[1] as f64;
                        km_colors[8] = c01[2] as f64;
                        km_colors[9] = c11[0] as f64;
                        km_colors[10] = c11[1] as f64;
                        km_colors[11] = c11[2] as f64;
                        km_weights[0] = p00;
                        km_weights[1] = p10;
                        km_weights[2] = p01;
                        km_weights[3] = p11;
                        km_weighted_mean_color(&km_colors, &km_weights, 4, inv, &mut km_out);
                        r_in = km_out[0];
                        g_in = km_out[1];
                        b_in = km_out[2];
                    } else {
                        r_in = (c00[0] as f64 * p00
                            + c10[0] as f64 * p10
                            + c01[0] as f64 * p01
                            + c11[0] as f64 * p11)
                            * inv;
                        g_in = (c00[1] as f64 * p00
                            + c10[1] as f64 * p10
                            + c01[1] as f64 * p01
                            + c11[1] as f64 * p11)
                            * inv;
                        b_in = (c00[2] as f64 * p00
                            + c10[2] as f64 * p10
                            + c01[2] as f64 * p01
                            + c11[2] as f64 * p11)
                            * inv;
                    }
                    new_susp += want;
                    new_rgb = [r_in as f32, g_in as f32, b_in as f32]; // REPLACE
                }
                new_film += film[i00] as f64 * k00
                    + film[i10] as f64 * k10
                    + film[i01] as f64 * k01
                    + film[i11] as f64 * k11;
            }
        }
        fr[x as usize] = new_film as f32;
        sr[x as usize] = new_susp as f32;
        cr[x as usize] = new_rgb;
        i += 1;
    }
}

#[path = "advect_commit.rs"]
mod commit;
use commit::{commit_rows, walk_rows2_reduce};

#[path = "advect_fundida.rs"]
mod fundida;
#[doc(hidden)]
pub use fundida::{ADVECT_FASES, BLOCO_LINHAS, MOMENTO_FORA};

#[cfg(test)]
#[path = "advect_jacobi_measure.rs"]
mod advect_jacobi_measure;

#[cfg(test)]
#[path = "advect_bit_tests.rs"]
mod advect_bit_tests;
