//! **Os cortes do fluxo são AO BIT** (31/10): o hash FNV-1a dos bits do estado inteiro da água
//! (`film`, `susp`, `susp_rgb`, `sett`, `sett_rgb`, `vel_x/y`, `flow_x/y`, `wet`, `active`, `bloom`)
//! depois de passos COMPLETOS de `step_simulation` numa tela com água nas quatro bordas, pigmento em
//! suspensão e gravidade a rodar. A constante foi gravada no `cfa0a413c3`, ANTES dos cortes.
use crate::painter::Engine;

const W: usize = 300;
const H: usize = 212;
const PASSOS: usize = 14;

struct Fnv(u64);
impl Fnv {
    fn bytes(&mut self, v: impl Iterator<Item = u8>) {
        for b in v {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }
    fn f32s(&mut self, v: &[f32]) {
        self.bytes(v.iter().flat_map(|x| x.to_bits().to_le_bytes()));
    }
    fn rgb(&mut self, v: &[[f32; 3]]) {
        self.bytes(v.iter().flatten().flat_map(|x| x.to_bits().to_le_bytes()));
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

fn hash_do_estado(gravidade: bool) -> u64 {
    let mut e = Engine::new(W, H);
    e.sliders.water = 1.0;
    e.sliders.size = 0.25;
    traco(&mut e, &[(30.0, 30.0), (270.0, 180.0)]);
    traco(&mut e, &[(2.0, 200.0), (2.0, 2.0), (200.0, 2.0)]);
    traco(&mut e, &[(298.0, 120.0), (298.0, 210.0), (180.0, 210.0)]);
    if gravidade {
        e.sim.gravity_override = Some([0.4, 0.7]);
    }
    let mut h = Fnv(0xcbf2_9ce4_8422_2325);
    for _ in 0..PASSOS {
        e.step_simulation();
        let g = e.active_grid();
        h.f32s(&g.film);
        h.f32s(&g.susp);
        h.rgb(&g.susp_rgb);
        h.f32s(&g.sett);
        h.rgb(&g.sett_rgb);
        h.f32s(&g.vel_x);
        h.f32s(&g.vel_y);
        h.f32s(&g.flow_x);
        h.f32s(&g.flow_y);
        h.bytes(g.wet.iter().copied());
        h.bytes(g.active.iter().copied());
        h.bytes(g.bloom.iter().copied());
    }
    let g = e.active_grid();
    assert!(
        g.film.iter().any(|&f| f > 0.0) && g.flow_x.iter().any(|&f| f != 0.0),
        "a cena não tem água nem fluxo"
    );
    h.0
}

#[test]
fn flow_step_bits_are_the_recorded_ones() {
    for rota in [1u8, 2] {
        crate::flow::COMMIT_ROTA.store(rota, std::sync::atomic::Ordering::Relaxed);
        let got = [hash_do_estado(false), hash_do_estado(true)];
        assert_eq!(
            got, GRAVADO,
            "rota do commit {rota}: o passo da água mudou de bits"
        );
    }
    crate::flow::COMMIT_ROTA.store(0, std::sync::atomic::Ordering::Relaxed);
    let got = [hash_do_estado(false), hash_do_estado(true)];
    let s = got.iter().map(|h| format!("{h:#018x}")).collect::<Vec<_>>();
    println!("hashes {s:?}");
    assert_eq!(got, GRAVADO, "o passo da água mudou de bits: {s:?}");
}

const GRAVADO: [u64; 2] = [0xf959_f6a5_7bbf_c1b2, 0x4650_940d_d0f2_d7fd];
