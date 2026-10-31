//! **O fator de borda GRADUADO** (`edgeSoftMass`, 31/10): com `k > 0` cada vizinho conta
//! `min(susp / (10·(1+k)), 1)` em vez de `susp > 10`, e a borda que esbate devagar deixa de ter a linha onde o
//! escurecimento salta (o contorno fino na tampa do «Wet Wash» que o Rebelle não tem). `k = 0` = o limiar de
//! sempre, ao bit — o hash abaixo foi gravado no código de antes da lei.

use ph2d_wet_paint::drying::{drying_pass, drying_pass_jacobi};
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::sim::{Params, Sim};
use ph2d_wet_paint::solver::rebuild_active_region;
use ph2d_wet_paint::tuning::{Knob, Tuning};

const W: usize = 48;
const H: usize = 7;

/// Uma tampa: `susp` chato a 40 até `x = 4`, depois a descer em RAMPA LENTA até 0 em 20 células (cruza o limiar
/// 10 a meio), com água por todo o lado. `dente` dá estrutura em `y` à cena do hash (0 = linhas iguais).
fn rampa(dente: f32) -> Grid {
    let mut g = Grid::new(W, H);
    let s = g.s;
    for y in 1..=H {
        for x in 1..=W {
            let i = x + y * s;
            let u = ((x as f32 - 4.0) / 20.0).clamp(0.0, 1.0);
            let m = 40.0 * (1.0 - u) * (1.0 + dente * ((x * 3 + y * 5) % 7) as f32 / 7.0);
            g.film[i] = if x <= 30 { 0.5 } else { 0.0 };
            g.susp[i] = m;
            g.sett[i] = dente * 2.0 * (y % 3) as f32;
            g.susp_rgb[i] = [200.0, 40.0, 90.0];
            g.sett_rgb[i] = [30.0, 90.0, 200.0];
        }
    }
    g.expand_bbox(1, 1, W as i32, H as i32);
    rebuild_active_region(&mut g);
    g
}

fn params(k: f64) -> Params {
    let mut t = Tuning::default();
    t.set(Knob::EdgeSoftMass, k);
    Sim::default().gather_params(&t)
}

fn fnv(g: &Grid) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut come = |v: f32| {
        for b in v.to_bits().to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for i in 0..g.cells {
        come(g.film[i]);
        come(g.susp[i]);
        come(g.sett[i]);
        g.susp_rgb[i].into_iter().for_each(&mut come);
        g.sett_rgb[i].into_iter().for_each(&mut come);
    }
    h
}

/// As duas rotas (Gauss-Seidel e Jacobi), 6 passes cada, sobre a cena com estrutura.
fn hash_das_rotas(p: &Params) -> (u64, u64) {
    let (mut a, mut b) = (rampa(0.6), rampa(0.6));
    for _ in 0..6 {
        drying_pass(&mut a, p, 0.002, 0.0002, false);
        drying_pass_jacobi(&mut b, p, 0.002, 0.0002, false);
    }
    (fnv(&a), fnv(&b))
}

/// **`edgeSoftMass = 0` é a secagem de antes, AO BIT** — nas duas rotas. Os números foram gravados com o código
/// anterior à lei (a janela binária sozinha).
#[test]
fn a_borda_graduada_a_zero_e_a_secagem_de_antes_ao_bit() {
    let (gs, jacobi) = hash_das_rotas(&params(0.0));
    println!("  hash: gauss-seidel {gs:#018x}  jacobi {jacobi:#018x}");
    assert_eq!(gs, GS_ANTES, "a rota Gauss-Seidel mudou com o knob a 0");
    assert_eq!(jacobi, JACOBI_ANTES, "a rota Jacobi mudou com o knob a 0");
}

const GS_ANTES: u64 = 0x20ba_9f7b_2369_30f9;
const JACOBI_ANTES: u64 = 0xe8e3_174d_523a_6d13;

/// O pior salto da 2.ª diferença do `sett` ao longo da rampa (linha do meio), relativo ao `sett` médio da rampa.
fn degrau(p: &Params) -> f64 {
    let mut g = rampa(0.0);
    for _ in 0..3 {
        drying_pass_jacobi(&mut g, p, 0.0005, 0.0, false);
    }
    let s = g.s;
    let y = H / 2 + 1;
    // Até x = 22 (susp 4): a ponta da rampa (susp → 0 em x = 24) é um canto da FIXTURA, igual a todo `k`.
    let v: Vec<f64> = (4..=22).map(|x| f64::from(g.sett[x + y * s])).collect();
    let media = v.iter().sum::<f64>() / v.len() as f64;
    let pior = v
        .windows(3)
        .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
        .fold(0.0f64, f64::max);
    pior / media
}

/// **A LEI: na rampa lenta o escurecimento não salta.** Com o limiar binário o fator de borda cai de 1 para 0 em
/// três células onde `susp` cruza 10, e o `sett` ganha um degrau isolado ali; graduado, o mesmo perfil é suave.
/// O controlo: a `k = 0` o degrau EXISTE (senão a lei passava por vácuo).
#[test]
fn a_borda_graduada_esbate_a_rampa_sem_degrau() {
    let (duro, mole) = (degrau(&params(0.0)), degrau(&params(2.0)));
    println!("  pior 2.ª diferença / sett médio: k=0 {duro:.4}  k=2 {mole:.4}");
    assert!(
        duro > 0.05,
        "a fixture não mostra o degrau do limiar binário: {duro}"
    );
    assert!(
        mole * 4.0 < duro,
        "a borda graduada não tirou o degrau: k=0 {duro}, k=2 {mole}"
    );
}

/// **Com `k > 0` o valor é ABSOLUTO, nas duas rotas** (gravado no código de 31/10, depois da lei). A `k = 1`
/// (abaixo de 1,5: a porta da rota Gauss-Seidel é `k > 0`, não um limiar maior) e a `k = 3`. O gate da janela
/// compara a janela com a forma direta, que divide pelo mesmo 9: só um número gravado vê o divisor.
#[test]
fn a_borda_graduada_com_k_positivo_da_os_bits_gravados() {
    let zero = hash_das_rotas(&params(0.0));
    let mut got = Vec::new();
    for k in [1.0, 3.0] {
        let (gs, jacobi) = hash_das_rotas(&params(k));
        assert!(
            gs != zero.0 && jacobi != zero.1,
            "k={k}: o knob não mudou a secagem — a lei não foi exercida"
        );
        got.push((gs, jacobi));
    }
    println!("  hashes k>0: {got:#018x?}");
    assert_eq!(got, GRADUADO, "a secagem graduada mudou de bits");
}

const GRADUADO: [(u64, u64); 2] = [
    (0x56fc_2695_f66e_b14b, 0xda68_3999_d3d7_8e61),
    (0x2886_d9ff_064e_4cde, 0x6cd2_62a6_0859_df55),
];
