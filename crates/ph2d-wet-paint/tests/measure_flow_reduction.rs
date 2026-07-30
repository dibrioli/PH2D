//! **F1 do plano 30 (multi-resolução): quanto custa a REDUÇÃO fino → grosso?**
//!
//! O plano §2.4 nomeia isto como o risco #1 e manda medi-lo ANTES de qualquer
//! coisa irreversível, e a razão é aritmética: rodar um passe na grade de FLUXO
//! corta o trabalho dele por `rf²`, mas exige levar os planos que ele LÊ para a
//! grade grossa, e essa redução é **`O(células finas)`** — ela **não** encolhe
//! com `rf`. O ganho é
//!
//! ```text
//!   antes:  O(finas)                        de trabalho
//!   depois: O(finas) de REDUÇÃO  +  O(finas / rf²) de trabalho
//! ```
//!
//! ⇒ se a redução custar o que o passe custava, a wave morre aqui, barato.
//!
//! ⚠️ **A LISTA DE PLANOS É O ACHADO, e ela saiu de LER o motor, não do plano.**
//! O plano §2.3 punha `vel_x`/`vel_y` na grade de fluxo e listava a redução como
//! `film`/`wet`/`susp`. Lendo os passes:
//!
//! | passe | lê |
//! |---|---|
//! | `build_flow_field` | `film` `paper` `vel_x` `vel_y` `wet` `susp` `sett` `active` `bloom` |
//! | `smooth_velocity`  | `film` `vel_x` `vel_y` `active` |
//! | `project`          | `vel_x` `vel_y` `active` |
//!
//! ⚠️ E o `advect` **ESCREVE `vel` por célula FINA** (`flow` amostrado na fonte
//! + `gravidade × film LOCAL`) — então `vel` não pode simplesmente "morar na
//! grade grossa": ou a atualização de momento migra para um passe grosso
//! próprio (o desenho do inkwash: *um campo de fluxo borrado e barato empurrando
//! um campo de tinta nítido e caro*), ou toda escrita de `advect` vira um
//! scatter com contenção. É decisão de DESENHO, e ela muda o que se reduz.
//!
//! Esta sonda, então, mede a redução em DOIS regimes, que bracketam o desenho:
//!
//! * **MÍNIMO (2 planos)** — `active` (any) + `film` (média): o que `project` e
//!   `smooth_velocity` sozinhos exigem, com `vel` já residente no grosso;
//! * **CHEIO (8 planos)** — mais `paper` `wet` `susp` `sett` `vel_x` `vel_y`:
//!   o que `build_flow_field` exige, que é o passe de 42,9%.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-wet-paint --release --test measure_flow_reduction -- --ignored --nocapture
//! ```

mod util;

use std::time::Instant;

use ph2d_wet_paint::grid::{Grid, restore_grid, snapshot_grid};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::solver;
use ph2d_wet_paint::tuning::Knob;
use util::drive_stroke;

const SIDE: usize = 4096;
const REPS: usize = 9;
const DIAG: f64 = std::f64::consts::FRAC_1_SQRT_2;

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// A MESMA poça do `measure_pass_cost::scene_big` — a da sessão do Enio (três
/// faixas largas e sobrepostas), porque um número que decide produto tem de
/// sair da cena que o produto tem.
fn scene_big() -> Engine {
    let mut e = Engine::new(SIDE, SIDE);
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let c = SIDE as f64 * 0.5;
    for lane in 0..3 {
        let off = 420.0 * f64::from(lane) - 420.0;
        drive_stroke(
            &mut e,
            c - 1500.0 * DIAG + off,
            c - 1500.0 * DIAG,
            c + 1500.0 * DIAG + off,
            c + 1500.0 * DIAG,
            120.0,
            10,
        );
    }
    e
}

/// Os buffers da grade de FLUXO para uma razão `rf`.
struct Coarse {
    cs: usize,
    film: Vec<f32>,
    cnt: Vec<u32>,
    active: Vec<u8>,
    paper: Vec<f32>,
    wet: Vec<u8>,
    susp: Vec<f32>,
    sett: Vec<f32>,
    vel_x: Vec<f32>,
    vel_y: Vec<f32>,
}

impl Coarse {
    fn new(w: usize, h: usize, rf: usize) -> Self {
        let cw = w.div_ceil(rf);
        let ch = h.div_ceil(rf);
        let cs = cw + 2;
        let n = cs * (ch + 2);
        Coarse {
            cs,
            film: vec![0.0; n],
            cnt: vec![0; n],
            active: vec![0; n],
            paper: vec![0.0; n],
            wet: vec![0; n],
            susp: vec![0.0; n],
            sett: vec![0.0; n],
            vel_x: vec![0.0; n],
            vel_y: vec![0.0; n],
        }
    }
}

/// Percorre a faixa viva em BLOCOS de `rf` colunas, entregando ao corpo
/// `(intervalo fino de índices, índice grosso)`.
///
/// ⚠️ **A divisão inteira por CÉLULA é o que mata um redutor ingênuo** — a 1ª
/// versão desta sonda a tinha, e mediu 1,46 ns/célula contra os 0,27 do passe
/// que ela deveria substituir. O índice grosso anda UMA vez por bloco.
#[inline]
fn walk_blocks(g: &Grid, cs: usize, rf: usize, mut body: impl FnMut(usize, usize, usize)) {
    let s = g.s;
    for y in g.by0..=g.by1 {
        let (bx0, bx1) = g.span_x(y);
        if bx0 > bx1 {
            continue;
        }
        let cbase = ((y as usize - 1) / rf + 1) * cs;
        let base = y as usize * s;
        let mut x = bx0 as usize;
        let hi = bx1 as usize;
        let mut ci = cbase + (x - 1) / rf + 1;
        while x <= hi {
            // O fim do bloco grosso a que `x` pertence.
            let end = (((x - 1) / rf) * rf + rf).min(hi);
            body(base + x, base + end, ci);
            x = end + 1;
            ci += 1;
        }
    }
}

/// **A redução MÍNIMA** — `active` (any) + `film` (média). É o que `project` e
/// `smooth_velocity` pedem, e só eles.
fn reduce_min(g: &Grid, c: &mut Coarse, rf: usize) {
    let (film, active, cnt) = (&mut c.film, &mut c.active, &mut c.cnt);
    walk_blocks(g, c.cs, rf, |i0, i1, ci| {
        let mut sum = 0.0f32;
        let mut act = 0u8;
        for i in i0..=i1 {
            sum += g.film[i];
            act |= g.active[i];
        }
        film[ci] += sum;
        active[ci] |= act;
        cnt[ci] += (i1 - i0 + 1) as u32;
    });
    // Normalização: O(grosso), isto é O(finas / rf²).
    for (f, n) in c.film.iter_mut().zip(c.cnt.iter_mut()) {
        if *n > 0 {
            *f /= *n as f32;
            *n = 0;
        }
    }
}

/// **A redução CHEIA** — os oito planos que `build_flow_field` lê.
fn reduce_full(g: &Grid, c: &mut Coarse, rf: usize) {
    let Coarse {
        film,
        paper,
        susp,
        sett,
        vel_x,
        vel_y,
        wet,
        active,
        cnt,
        cs,
    } = c;
    let cs = *cs;
    walk_blocks(g, cs, rf, |i0, i1, ci| {
        let (mut sf, mut sp, mut ss, mut se, mut svx, mut svy) = (0.0f32, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut act = 0u8;
        let mut wt = 0u8;
        for i in i0..=i1 {
            sf += g.film[i];
            sp += g.paper[i];
            ss += g.susp[i];
            se += g.sett[i];
            svx += g.vel_x[i];
            svy += g.vel_y[i];
            act |= g.active[i];
            if g.wet[i] > wt {
                wt = g.wet[i];
            }
        }
        film[ci] += sf;
        paper[ci] += sp;
        susp[ci] += ss;
        sett[ci] += se;
        vel_x[ci] += svx;
        vel_y[ci] += svy;
        active[ci] |= act;
        if wt > wet[ci] {
            wet[ci] = wt;
        }
        cnt[ci] += (i1 - i0 + 1) as u32;
    });
    for (k, n) in c.cnt.iter_mut().enumerate() {
        if *n > 0 {
            let inv = 1.0 / *n as f32;
            c.film[k] *= inv;
            c.paper[k] *= inv;
            c.susp[k] *= inv;
            c.sett[k] *= inv;
            c.vel_x[k] *= inv;
            c.vel_y[k] *= inv;
            *n = 0;
        }
    }
}

/// **A ALTERNATIVA que a redução esconde: AMOSTRAR em vez de MEDIAR.**
///
/// Uma redução é `O(finas)` porque LÊ toda célula fina. Mas o passe de fluxo
/// não precisa da média — ele precisa de *um número que descreva o bloco*, e o
/// campo é suave por física (é a premissa inteira do inkwash). Ler UMA célula
/// fina por bloco custa **`O(grossas)`**, com passo `rf` na memória.
///
/// ⚠️ Não é a mesma resposta: uma gota de 1 px de largura pode cair ENTRE dois
/// pontos de amostra e o fluxo não a vê. Isso é pergunta de APARÊNCIA, decidida
/// por render-and-look — mas o CUSTO se decide aqui.
fn sample_full(g: &Grid, c: &mut Coarse, rf: usize) {
    let s = g.s;
    let cs = c.cs;
    let mut y = g.by0;
    while y <= g.by1 {
        let (bx0, bx1) = g.span_x(y);
        if bx0 <= bx1 {
            let cbase = ((y as usize - 1) / rf + 1) * cs;
            let base = y as usize * s;
            let mut x = bx0 as usize;
            let mut ci = cbase + (x - 1) / rf + 1;
            while x <= bx1 as usize {
                let i = base + x;
                c.film[ci] = g.film[i];
                c.paper[ci] = g.paper[i];
                c.susp[ci] = g.susp[i];
                c.sett[ci] = g.sett[i];
                c.vel_x[ci] = g.vel_x[i];
                c.vel_y[ci] = g.vel_y[i];
                c.wet[ci] = g.wet[i];
                c.active[ci] = g.active[i];
                x += rf;
                ci += 1;
            }
        }
        y += rf as i32;
    }
}

fn time(mut f: impl FnMut()) -> f64 {
    let mut v = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let t = Instant::now();
        f();
        v.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    median(&mut v)
}

fn time_pass(
    g: &mut Grid,
    snap: &ph2d_wet_paint::grid::GridSnapshot,
    mut f: impl FnMut(&mut Grid),
) -> f64 {
    let mut v = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        restore_grid(g, snap);
        let t = Instant::now();
        f(g);
        v.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    median(&mut v)
}

#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_what_the_flow_grid_would_cost_to_feed() {
    println!("\n=== F1: a REDUCAO fino -> grosso paga o proprio preco? ===");
    let mut e = scene_big();
    let p: Params = e.sim.gather_params(&e.tuning);
    let grav = e.sim.gravity(&e.tuning);
    let bypass = e.sim.ext_bypass;
    let g = e.active_grid_mut();
    let (w, h) = (g.w, g.h);
    let snap = snapshot_grid(g);

    // --- o que os tres passes candidatos custam HOJE, na grade fina ---
    let t_build = time_pass(g, &snap, |g| {
        solver::build_flow_field(g, &p, grav[0], grav[1], bypass);
    });
    let t_smooth = time_pass(g, &snap, |g| solver::smooth_velocity(g, &p));
    let t_project = time_pass(g, &snap, |g| solver::project(g, &p));
    restore_grid(g, &snap);

    let mut span_cells = 0u64;
    let mut active = 0u64;
    for y in g.by0..=g.by1 {
        let (bx0, bx1) = g.span_x(y);
        if bx0 > bx1 {
            continue;
        }
        span_cells += (bx1 - bx0 + 1) as u64;
        let base = y as usize * g.s;
        for x in bx0..=bx1 {
            if g.active[x as usize + base] != 0 {
                active += 1;
            }
        }
    }
    println!(
        "\n  poca: faixa viva {span_cells} celulas | ativas {active} ({:.1}%)",
        100.0 * active as f64 / span_cells.max(1) as f64
    );
    println!("\n  CUSTO DE HOJE (grade fina, 1 celula por px):");
    println!("    build_flow_field   {t_build:7.3} ms");
    println!("    smooth_velocity    {t_smooth:7.3} ms");
    println!("    project            {t_project:7.3} ms");

    println!("\n  ALIMENTAR a grade de fluxo, por rota:");
    println!("    rf   MEDIA min(2pl)   MEDIA cheia(8pl)   AMOSTRA cheia(8pl)");
    for rf in [2usize, 4, 8] {
        let mut c = Coarse::new(w, h, rf);
        let t_min = time(|| reduce_min(g, &mut c, rf));
        let mut c2 = Coarse::new(w, h, rf);
        let t_full = time(|| reduce_full(g, &mut c2, rf));
        let mut c3 = Coarse::new(w, h, rf);
        let t_smp = time(|| sample_full(g, &mut c3, rf));
        println!("    {rf:<4} {t_min:10.3} ms      {t_full:10.3} ms       {t_smp:10.3} ms");
    }
    println!(
        "    (MEDIA e O(finas) e NAO encolhe com rf; AMOSTRA e O(grossas) e encolhe por rf^2)"
    );

    // --- De que o build_flow_field e feito: ABLACAO POR ENTRADA (knobs), nunca
    //     instrumentacao (uma sonda que re-implementa o laco fica CEGA a porta).
    //     §2.5 do plano: o backrun espalha PIGMENTO e o fingering e da borda —
    //     os dois ficam FINOS, entao o que eles custam NAO encolhe com rf.
    println!("\n  De que o build_flow_field e feito (ablacao por KNOB):");
    let mut probe = |label: &str, backrun: f64, fingering: f64| {
        let mut t = e.tuning.clone();
        t.set(Knob::ExtBackrun, backrun);
        t.set(Knob::ExtFingering, fingering);
        let pp = e.sim.gather_params(&t);
        let gg = e.active_grid_mut();
        let ms = time_pass(gg, &snap, |g| {
            solver::build_flow_field(g, &pp, grav[0], grav[1], false);
        });
        restore_grid(gg, &snap);
        println!("    {label:<34} {ms:7.3} ms");
        ms
    };
    let (k_back, k_fing) = (p.k(Knob::ExtBackrun), p.k(Knob::ExtFingering));
    let b_all = probe("tudo ligado (o produto)", k_back, k_fing);
    let b_no_back = probe("sem backrun", 0.0, k_fing);
    let b_no_fing = probe("sem fingering", k_back, 0.0);
    let b_bare = probe("sem os dois (so o campo)", 0.0, 0.0);
    println!(
        "    => backrun {:+.3} ms | fingering {:+.3} ms | nucleo {b_bare:.3} ms",
        b_all - b_no_back,
        b_all - b_no_fing
    );

    let g = e.active_grid_mut();
    // --- o veredito, com a aritmetica escrita ---
    let mut c = Coarse::new(w, h, 4);
    let t_min4 = time(|| reduce_min(g, &mut c, 4));
    let mut c2 = Coarse::new(w, h, 4);
    let t_full4 = time(|| reduce_full(g, &mut c2, 4));

    println!("\n  VEREDITO a rf=4 (o ponto de operacao do plano):");
    let cheap = t_smooth + t_project;
    let cheap_after = cheap / 16.0 + t_min4;
    println!(
        "    [A] project + smooth SOZINHOS:  {cheap:.3} -> {:.3} + {t_min4:.3} = {cheap_after:.3} ms   ({:+.3} ms)",
        cheap / 16.0,
        cheap_after - cheap
    );
    let all = cheap + t_build;
    let all_after = all / 16.0 + t_full4;
    println!(
        "    [B] + build_flow_field:         {all:.3} -> {:.3} + {t_full4:.3} = {all_after:.3} ms   ({:+.3} ms)",
        all / 16.0,
        all_after - all
    );
    println!(
        "\n  ⚠️ [B] supoe o build INTEIRO grosso, o que o plano §2.5 diz ser FALSO\n  \
         (o backrun espalha PIGMENTO e fica fino). O numero real de [B] fica entre\n  \
         os dois, e a fatoracao do passe e a Fase 3."
    );
}
