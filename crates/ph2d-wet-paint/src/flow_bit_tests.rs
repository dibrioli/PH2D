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
    hash_dos_passos(e, gravidade)
}

fn hash_dos_passos(mut e: Engine, gravidade: bool) -> u64 {
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

/// Regravado 31/10 (E4, `7a4bc540b7`): a receita × quantidade recalcula o pigmento e a água de cada dab por
/// `painter/receita.rs` (φ·h) em vez de `PigmentPerDab`/`water²÷pressão` — iguais à pressão de referência até ao
/// arredondamento (`receita_tests`, < 1e-6 por dab); o passo da água não mudou (as rotas 1 e 2 dão o mesmo par).
/// Antes (a umidade de hoje, 03/11): `[0x0abe_64da_e119_8a5b, 0xabd1_73df_e10f_26da]`. Doc: `docs/Painter/Arquitetura/FECHO_31_10.md`.
/// Regravado 14/11 (B13.10): o freio da água nas bordas esquerda/direita deixa de sondar a linha vizinha. Com
/// `flow::FREIO_LINEAR` ligado a cena dá ao bit o par de antes, `[0x43ec_b6d1_707d_bdb8, 0x8410_5e51_a74c_3aa6]`.
const GRAVADO: [u64; 2] = [0x2215_5309_2cd2_0899, 0xfde6_3aec_bd3d_86be];

/// Duas faixas largas separadas por linhas SECAS dentro da bbox: as linhas sem faixa viva (`row_lo > row_hi`,
/// sentinelas `i32::MAX`/`MIN`) são contadas pelo `commit_rows` — `hi − lo + 1` transbordava (30/10, o
/// `parallel_rows`). As três rotas do commit dão o mesmo estado.
#[test]
fn flow_step_bits_with_dry_rows_inside_the_bbox() {
    let cena = || {
        let mut e = Engine::new(W, H);
        e.sliders.water = 1.0;
        e.sliders.size = 1.0;
        traco(&mut e, &[(20.0, 20.0), (280.0, 24.0)]);
        traco(&mut e, &[(20.0, 190.0), (280.0, 186.0)]);
        e
    };
    let mut got = Vec::new();
    for rota in [1u8, 2, 0] {
        crate::flow::COMMIT_ROTA.store(rota, std::sync::atomic::Ordering::Relaxed);
        got.push(hash_dos_passos(cena(), false));
    }
    crate::flow::COMMIT_ROTA.store(0, std::sync::atomic::Ordering::Relaxed);
    assert!(
        got.iter().all(|&h| h == got[0]),
        "as rotas do commit divergem: {got:x?}"
    );
}
