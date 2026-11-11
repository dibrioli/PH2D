//! **Os cortes da advecção são AO BIT** (31/10, missão P2): o hash FNV-1a dos bits do estado que
//! [`super::advect_jacobi_rows`] escreve (`film`, `susp`, `susp_rgb`, `vel_x/y`, o rascunho `uv` e
//! `outflow`, e o `vmax`) depois de passos encadeados numa tela pequena com água nas bordas, células
//! inativas com massa, velocidade no tecto do knob (`MaxVelocity` 1 e 3), K–M ligado e desligado, e as
//! rotas serial e paralela. A constante foi gravada no `4ac6c0bf5d`, ANTES dos cortes.
use super::*;
use crate::painter::Engine;

const W: usize = 300;
const H: usize = 212;
const PASSOS: usize = 8;

struct Fnv(u64);
impl Fnv {
    fn f32s(&mut self, v: &[f32]) {
        for x in v {
            for b in x.to_bits().to_le_bytes() {
                self.0 ^= u64::from(b);
                self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
}

fn traco(e: &mut Engine, pts: &[(f64, f64)]) {
    e.pointer_down(pts[0].0, pts[0].1, None);
    let (mut x, mut y) = pts[0];
    for w in pts.windows(2) {
        let (dx, dy) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
        let len = (dx * dx + dy * dy).sqrt();
        let mut t = 0.0;
        while t < len {
            t = (t + 6.0).min(len);
            x = w[0].0 + dx / len * t;
            y = w[0].1 + dy / len * t;
            e.pointer_frame(x, y);
            if e.sim_should_run() {
                e.step_simulation();
            }
        }
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// Três traços: um em diagonal pelo meio, um colado à borda esquerda e ao topo, um no canto de baixo
/// à direita — a água toca as quatro bordas da folha.
fn cena_rf(rf: usize) -> Engine {
    let mut e = Engine::with_flow_ratio(W, H, rf);
    e.sliders.water = 1.0;
    e.sliders.size = 0.25;
    traco(&mut e, &[(30.0, 30.0), (270.0, 180.0)]);
    traco(&mut e, &[(2.0, 200.0), (2.0, 2.0), (200.0, 2.0)]);
    traco(&mut e, &[(298.0, 120.0), (298.0, 210.0), (180.0, 210.0)]);
    for _ in 0..3 {
        e.step_simulation();
    }
    e
}

/// Funde o `momentum` nos blocos enquanto vive (contador: `MOMENTO_FORA` volta ao valor do produto no fim).
struct MomentoDentro(usize);
impl MomentoDentro {
    fn new() -> Self {
        MomentoDentro(fundida::MOMENTO_FORA.swap(0, std::sync::atomic::Ordering::SeqCst))
    }
}
impl Drop for MomentoDentro {
    fn drop(&mut self) {
        fundida::MOMENTO_FORA.store(self.0, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Segura a rota de FASES enquanto vive (os gates que hasheiam o rascunho da grelha inteira).
struct Fases;
impl Fases {
    fn new() -> Self {
        ADVECT_FASES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Fases
    }
}
impl Drop for Fases {
    fn drop(&mut self) {
        ADVECT_FASES.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

fn hash_da_advecao(max_v: f64, km: bool, mode: Rows) -> (u64, f64) {
    hash_rota(max_v, km, mode, None)
}

/// `fundida = None`: a rota de fases (a faixa de UM intervalo: o rascunho no vão seco é o de antes de B13.2), rascunho incluído no hash; `Some(linhas)`: a fundida com esse
/// bloco, e o hash só da GRELHA (o rascunho da grelha inteira ela não escreve).
fn hash_rota(max_v: f64, km: bool, mode: Rows, fundida: Option<usize>) -> (u64, f64) {
    hash_rota_com(max_v, km, mode, fundida, fundida.is_none(), 1, false)
}

fn hash_rota_com(
    max_v: f64,
    km: bool,
    mode: Rows,
    fundida: Option<usize>,
    rascunho: bool,
    rf: usize,
    tiles: bool,
) -> (u64, f64) {
    let _fases = Fases::new();
    let mut e = cena_rf(rf);
    for l in &mut e.layers {
        l.grid.tiles_enabled = tiles;
    }
    e.tuning.set(Knob::MaxVelocity, max_v);
    let mut p = e.sim.gather_params(&e.tuning);
    p.km_mixing = km;
    let g = e.active_grid_mut();
    // Pigmento em suspensão de cinco tintas sob toda a água — sem ele o ramo `want > 0` (e o K–M) não corre.
    const TINTAS: [[f32; 3]; 5] = [
        [200.0, 40.0, 40.0],
        [40.0, 60.0, 200.0],
        [245.0, 205.0, 25.0],
        [255.0, 255.0, 255.0],
        [10.0, 10.0, 10.0],
    ];
    for i in 0..g.cells {
        if g.film[i] > 0.0 {
            g.susp[i] = g.film[i] * (0.15 + 0.1 * (i % 7) as f32);
            g.susp_rgb[i] = TINTAS[(i / 5) % 5];
        }
    }
    let mut h = Fnv(0xcbf2_9ce4_8422_2325);
    let mut vtop = 0.0f64;
    let mut lcg = 0x2545_f491_u32;
    for k in 0..PASSOS {
        crate::solver::rebuild_active_region(g);
        // O fluxo do produto sai do `flow_jacobi` já limitado a `±maxVelocity`; aqui ele é sorteado em
        // `[−M, M]` com 1/8 das células no tecto EXATO — o alcance máximo do retro-traço.
        for v in g.flow_x.iter_mut().chain(g.flow_y.iter_mut()) {
            lcg = lcg.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let u = f64::from(lcg >> 8) / f64::from(1u32 << 24) * 2.0 - 1.0;
            *v = match lcg & 15 {
                0 => max_v as f32,
                1 => -max_v as f32,
                _ => (u * max_v) as f32,
            };
        }
        // Gravidade forte e a rodar: as velocidades batem no tecto em x e em y, nos dois sentidos.
        let a = k as f64 * 0.9;
        let (gx, gy) = (2.5 * a.cos(), 2.5 * a.sin());
        let vmax = match fundida {
            None => advect_jacobi_rows(g, &p, gx, gy, mode),
            Some(n) => fundida::advect_fundida_rows(g, &p, gx, gy, mode, n),
        };
        vtop = vtop.max(vmax);
        // A cobertura dos ramos é a da cena GRAVADA (fases, `rf = 1`; a fundida não escreve o rascunho).
        if k == 0 && rf == 1 && fundida.is_none() {
            let s = g.s;
            let n = |f: &dyn Fn(usize) -> bool| (0..g.cells).filter(|&i| f(i)).count();
            let excesso = n(&|i| g.scratch.outflow[i] > 1.0);
            let inativa = n(&|i| g.active[i] == 0 && g.film[i] > 0.0);
            let borda = n(&|i| {
                let (x, y) = (i % s, i / s);
                g.film[i] > 0.0 && (x <= 2 || x + 3 >= g.w + 2 || y <= 2 || y + 3 >= g.h + 2)
            });
            assert!(
                excesso > 0 && inativa > 0 && borda > 0,
                "a cena não exercita os ramos: saída>1 {excesso} · inativa com água {inativa} · borda {borda}"
            );
        }
        h.f32s(&[vmax as f32]);
        h.0 ^= vmax.to_bits();
        h.f32s(&g.film);
        h.f32s(&g.susp);
        for c in &g.susp_rgb {
            h.f32s(c);
        }
        h.f32s(&g.vel_x);
        h.f32s(&g.vel_y);
        if rascunho {
            h.f32s(&g.scratch.outflow);
            for c in &g.scratch.uv {
                h.f32s(c);
            }
        }
    }
    (h.0, vtop)
}

/// **A rota FUNDIDA por blocos dá a GRELHA da rota de fases ao bit** (rodada 7): os quatro casos do gate
/// acima (tecto 1 e 3, mistura KM ou não, série e paralela), com blocos de 1, 3, 7 e 32 linhas — o halo
/// derivado do alcance tem de cobrir tudo em cada fronteira de bloco.
#[test]
fn advect_jacobi_fundida_bits_equal_the_phases() {
    let casos = [
        (1.0, false, Rows::Serial, 1),
        (1.0, true, Rows::Parallel, 1),
        (3.0, false, Rows::Parallel, 1),
        (3.0, true, Rows::Serial, 1),
        (1.0, false, Rows::Medida(0), 1),
        (3.0, true, Rows::Medida(u64::MAX), 1),
        (1.0, true, Rows::Parallel, 4),
        (3.0, false, Rows::Medida(0), 3),
    ];
    for (mv, km, mode, rf) in casos {
        let fases = hash_rota_com(mv, km, mode, None, false, rf, true).0;
        for linhas in [1, 3, 7, 32] {
            let fundida = hash_rota_com(mv, km, mode, Some(linhas), false, rf, true).0;
            assert_eq!(
                fundida, fases,
                "tecto {mv} km {km} {mode:?} bloco {linhas}: a fundida mudou bits"
            );
            let _dentro = MomentoDentro::new();
            let fundida = hash_rota_com(mv, km, mode, Some(linhas), false, rf, true).0;
            assert_eq!(
                fundida, fases,
                "tecto {mv} km {km} {mode:?} bloco {linhas}: a fundida mudou bits"
            );
        }
    }
}

#[test]
fn advect_jacobi_bits_are_the_recorded_ones() {
    let casos = [
        (1.0, false, Rows::Serial),
        (1.0, true, Rows::Parallel),
        (3.0, false, Rows::Parallel),
        (3.0, true, Rows::Serial),
    ];
    let mut got = Vec::new();
    for (mv, km, mode) in casos {
        let (h, vtop) = hash_da_advecao(mv, km, mode);
        assert!(
            vtop >= mv * 0.999,
            "a cena não chega ao tecto {mv}: vmax {vtop}"
        );
        got.push(h);
    }
    // A rota não muda a resposta (contrato da paralela).
    assert_eq!(hash_da_advecao(1.0, false, Rows::Parallel).0, got[0]);
    let s = got.iter().map(|h| format!("{h:#018x}")).collect::<Vec<_>>();
    println!("hashes {s:?}");
    assert_eq!(got, GRAVADO, "a advecção mudou de bits: {s:?}");
}

/// A cena entra no hash: a lei do DEPÓSITO move-o sem a advecção mudar. Regravado em 31/10 depois da E4
/// (`7a4bc540b7`, a receita: água sem a pressão inversa, carimbo sem `Intensity`, pigmento `φ·h`): com a lei
/// antiga do `pressure_dab` posta de volta só na cena, o HEAD `be475f3a46` dá os `8c92…/e965…/b735…/d8e7…`
/// de `4ac6c0bf5d` nos dois gates — a advecção e as duas rotas do commit estão ao bit.
/// Os dois casos K–M ([1], [3]) regravados em 01/11 pelo invólucro da média K–M (E10e2,
/// `docs/Painter/Arquitetura/E10e2_km_e_remolha.md` §1): no mesmo binário, sem o invólucro dão os antigos
/// `0x1cb2…9adf`/`0x0118…a2fd`; a tinta 10 deixa de virar 12,709 e o resto muda ≤ 4e-6.
const GRAVADO: [u64; 4] = [
    0xbe1a_63e7_3c04_445b,
    0x2bf1_4333_5a9b_f102,
    0x18d9_3872_ba02_f0ea,
    0x0418_b1ad_31b2_4a60,
];

/// **As duas rotas do commit FORÇADAS dão os bits gravados** — a rota 0 escolhe pela faixa, e a cena acima cai
/// sempre na mesma; aqui cada uma corre (1 = cópia pela faixa, 2 = troca de planos + complemento).
#[test]
fn advect_jacobi_bits_by_each_commit_route() {
    use std::sync::atomic::Ordering::Relaxed;
    for rota in [1u8, 2] {
        crate::flow::COMMIT_ROTA.store(rota, Relaxed);
        let got = [
            hash_da_advecao(1.0, false, Rows::Serial).0,
            hash_da_advecao(3.0, true, Rows::Parallel).0,
        ];
        crate::flow::COMMIT_ROTA.store(0, Relaxed);
        assert_eq!(
            got,
            [GRAVADO[0], GRAVADO[3]],
            "rota do commit {rota}: a advecção mudou de bits"
        );
    }
}

/// **O commit escreve a faixa INTEIRA, inclusive a última coluna, e nada fora dela** — célula a célula, nas duas
/// rotas: o rascunho tem um valor por célula, o grid um sentinela; faixas de larguras diferentes por linha, uma
/// linha sem faixa (sentinelas) e a faixa a tocar a borda direita da bbox.
#[test]
fn advect_jacobi_commit_writes_exactly_the_live_span() {
    use std::sync::atomic::Ordering::Relaxed;
    let (w, h) = (20, 10);
    for rota in [1u8, 2] {
        let mut g = Grid::new(w, h);
        let (s, n) = (g.s, g.cells);
        g.scratch.ensure(n);
        (g.bx0, g.bx1, g.by0, g.by1) = (2, 19, 2, 8);
        g.fill_tiles_all(); // B13.2: todo tile marcado, a faixa inteira
        for y in 2..=8usize {
            (g.row_lo[y], g.row_hi[y]) = if y == 5 {
                (i32::MAX, i32::MIN)
            } else {
                (2 + (y % 3) as i32, 21 - (y % 4) as i32)
            };
        }
        for i in 0..n {
            g.film[i] = -1.0;
            g.susp[i] = -2.0;
            g.susp_rgb[i] = [-3.0; 3];
            g.scratch.dst_film[i] = i as f32;
            g.scratch.dst_susp[i] = 0.5 * i as f32;
            g.scratch.dst_rgb[i] = [i as f32, 1.0, 2.0];
        }
        crate::flow::COMMIT_ROTA.store(rota, Relaxed);
        commit_rows(&mut g, Rows::Serial);
        crate::flow::COMMIT_ROTA.store(0, Relaxed);
        for y in 0..g.rows {
            let (lo, hi) = if (2..=8).contains(&y) {
                span_x_of(&g.row_lo, &g.row_hi, g.spans_enabled, 2, 19, y as i32)
            } else {
                (1, 0)
            };
            for x in 0..s {
                let i = y * s + x;
                let dentro = (lo..=hi).contains(&(x as i32));
                let (f, su, c) = if dentro {
                    (i as f32, 0.5 * i as f32, [i as f32, 1.0, 2.0])
                } else {
                    (-1.0, -2.0, [-3.0; 3])
                };
                assert_eq!(
                    (g.film[i], g.susp[i], g.susp_rgb[i]),
                    (f, su, c),
                    "rota {rota}: célula ({x}, {y}) faixa [{lo}, {hi}]"
                );
            }
        }
    }
}

/// **B13.2 — a advecção pelos tiles de 64 dá a GRELHA da faixa de um intervalo ao bit**, com o fluxo sorteado
/// em toda célula (também no vão seco): fases e fundida, `rf` 1 e 2, tecto 1 e 3, K–M ou não, série e paralela.
#[test]
fn advect_jacobi_tiles_bits_equal_the_one_interval_span() {
    for rf in [1, 2] {
        for (mv, km, mode) in [(1.0, false, Rows::Serial), (3.0, true, Rows::Parallel)] {
            for fundida in [None, Some(7)] {
                let um = hash_rota_com(mv, km, mode, fundida, false, rf, false).0;
                let tiles = hash_rota_com(mv, km, mode, fundida, false, rf, true).0;
                assert_eq!(
                    um, tiles,
                    "rf {rf} · tecto {mv} · km {km} · {mode:?} · fundida {fundida:?}"
                );
            }
        }
    }
}
