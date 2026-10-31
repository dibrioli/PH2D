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
fn cena() -> Engine {
    let mut e = Engine::new(W, H);
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

fn hash_da_advecao(max_v: f64, km: bool, mode: Rows) -> (u64, f64) {
    let mut e = cena();
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
        let vmax = advect_jacobi_rows(g, &p, 2.5 * a.cos(), 2.5 * a.sin(), mode);
        vtop = vtop.max(vmax);
        if k == 0 {
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
        h.f32s(&g.scratch.outflow);
        for c in &g.scratch.uv {
            h.f32s(c);
        }
    }
    (h.0, vtop)
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
        assert!(vtop >= mv * 0.999, "a cena não chega ao tecto {mv}: vmax {vtop}");
        got.push(h);
    }
    // A rota não muda a resposta (contrato da paralela).
    assert_eq!(hash_da_advecao(1.0, false, Rows::Parallel).0, got[0]);
    let s = got.iter().map(|h| format!("{h:#018x}")).collect::<Vec<_>>();
    println!("hashes {s:?}");
    assert_eq!(got, GRAVADO, "a advecção mudou de bits: {s:?}");
}

const GRAVADO: [u64; 4] = [
    0x8c92_596a_5485_ccbc,
    0xe965_2f81_1a49_dff5,
    0xb735_c68b_229b_7a08,
    0xd8e7_013b_c075_8c6a,
];
