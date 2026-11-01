//! Paridade com o oráculo `lei_unica.py` (fixture gravada com cabeçalho; regerar com o comando do
//! cabeçalho). Tolerâncias ABSOLUTAS por banda, medidas 31/10 (crate `f32` × oráculo `f64`):
//!
//! - `TOL_LAB = 2e-6` sobre Oklab (`L ∈ [0,1]`, `a,b ∈ ±0,4`): a ida lin→Oklab em `f32` é 2 matrizes
//!   3×3 + `cbrtf` (~4 ulp). Medido: ver `println!`; `TOL` ≈ 10× o medido e ~4 ordens abaixo de um
//!   passo de 8 bits em Oklab (~4e-3).
//! - `TOL_R = 5e-6` sobre `R` e `C` (em `[0,1]`): soma a ida-volta Oklab (cubo amplifica 3× o erro
//!   relativo de `L`) e o `K/S` perto do piso `R_INF_MIN` (q ≈ 500). ~2 ordens abaixo de 1/255
//!   linear no escuro (≈ 3e-4).
//! - `TOL_K = 2e-6` nas chamadas cruas de `camada_banda` (só a eq. 2; a forma com cancelação perdia
//!   `4e-6` — este `TOL` reprova-a).

use ph2d_paint_tinta::oklab::linear_para_oklab;
use ph2d_paint_tinta::{
    Bandas, LeiDeMistura, Soma, Tinta, camada, camada_banda, cobertura, mistura, optica,
};

const FIXTURE: &str = include_str!("../fixtures/lei_unica.tsv");
const TOL_LAB: f64 = 2.0e-6;
const TOL_R: f64 = 5.0e-6;
const TOL_K: f64 = 2.0e-6;

fn num(x: &str) -> f64 {
    x.parse()
        .unwrap_or_else(|_| panic!("número inválido na fixture: {x}"))
}

#[test]
fn a_crate_casa_com_o_oraculo_por_banda() {
    assert!(
        FIXTURE.starts_with("# fixture da lei única — mistura Oklab"),
        "fixture sem cabeçalho"
    );
    let mut tintas: Vec<(String, Tinta)> = Vec::new();
    let tinta = |ts: &[(String, Tinta)], n: &str| ts.iter().find(|p| p.0 == n).expect(n).1;
    let (mut n_o, mut n_m, mut n_c, mut n_k, mut defeito_oraculo) = (0, 0, 0, 0, 0);
    let (mut max_o, mut max_m, mut max_r, mut max_c, mut max_k) = (0f64, 0f64, 0f64, 0f64, 0f64);
    for linha in FIXTURE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let c: Vec<&str> = linha.split('\t').collect();
        match c[0] {
            "T" => {
                let v: Vec<f32> = c[2..6].iter().map(|x| num(x) as f32).collect();
                tintas.push((c[1].to_owned(), Tinta::de_linear([v[0], v[1], v[2]], v[3])));
            }
            "O" => {
                let lin = [num(c[1]) as f32, num(c[2]) as f32, num(c[3]) as f32];
                let o = linear_para_oklab(lin);
                for i in 0..3 {
                    let e = (f64::from(o[i]) - num(c[4 + i])).abs();
                    assert!(e <= TOL_LAB, "Oklab {i}: {o:?} em «{linha}»");
                    max_o = max_o.max(e);
                }
                n_o += 1;
            }
            "M" => {
                let (a, b) = (tinta(&tintas, c[1]), tinta(&tintas, c[3]));
                let m = mistura(
                    &[(a, num(c[2]) as f32), (b, num(c[4]) as f32)],
                    LeiDeMistura::Oklab,
                )
                .expect("volume > 0");
                for i in 0..3 {
                    let e = (f64::from(m.lab[i]) - num(c[5 + i])).abs();
                    assert!(e <= TOL_LAB, "mistura Lab {i}: {m:?} em «{linha}»");
                    max_m = max_m.max(e);
                }
                let e = (f64::from(m.transparencia) - num(c[8])).abs();
                assert!(e <= TOL_LAB, "mistura t: {m:?} em «{linha}»");
                n_m += 1;
            }
            "C" => {
                let (a, b) = (tinta(&tintas, c[1]), tinta(&tintas, c[3]));
                let soma = Soma::de(a, num(c[2]) as f32).junta(Soma::de(b, num(c[4]) as f32));
                let o = optica(&soma, num(c[5]) as f32, c[6] == "1");
                let rg = Bandas([num(c[7]) as f32, num(c[8]) as f32, num(c[9]) as f32]);
                let r = camada(&o, &rg);
                let cob = cobertura(&o);
                for b in 0..3 {
                    let er = (f64::from(r.0[b]) - num(c[10 + b])).abs();
                    let ec = (f64::from(cob.0[b]) - num(c[13 + b])).abs();
                    assert!(er <= TOL_R, "R banda {b}: {} em «{linha}»", r.0[b]);
                    assert!(ec <= TOL_R, "C banda {b}: {} em «{linha}»", cob.0[b]);
                    max_r = max_r.max(er);
                    max_c = max_c.max(ec);
                }
                n_c += 1;
            }
            "K" => {
                let (kt, st, rg) = (num(c[1]), num(c[2]), num(c[3]));
                let r = f64::from(camada_banda(kt as f32, st as f32, rg as f32));
                if kt == 0.0 && st > 1e-9 {
                    // Ramo do oráculo com b = 0: `0·coth(1e-12)`. A verdade é o limite fechado K = 0:
                    // R = (S(1−R_g)+R_g)/(S(1−R_g)+1). Ver F4_1 §oráculo.
                    let verdade = (st * (1.0 - rg) + rg) / (st * (1.0 - rg) + 1.0);
                    assert!((r - verdade).abs() <= TOL_K, "K=0: {r} vs {verdade}");
                    defeito_oraculo += 1;
                    continue;
                }
                let er = (r - num(c[4])).abs();
                assert!(er <= TOL_K, "km_camada({kt},{st},{rg}) = {r} vs {}", c[4]);
                max_k = max_k.max(er);
                n_k += 1;
            }
            outro => panic!("linha desconhecida: {outro}"),
        }
    }
    println!(
        "paridade: {n_o} Oklab (max {max_o:.2e}) · {n_m} misturas (max {max_m:.2e}) · {n_c} camadas × 3 \
         bandas (max |ΔR| {max_r:.2e}, |ΔC| {max_c:.2e}) · {n_k} cruas (max {max_k:.2e}) · \
         {defeito_oraculo} no ramo K=0 contra a forma fechada"
    );
    assert_eq!(tintas.len(), 6);
    assert!(
        n_o >= 10 && n_m >= 140 && n_c >= 4700 && n_k >= 140,
        "fixture encolheu"
    );
}
