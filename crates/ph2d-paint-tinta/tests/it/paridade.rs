//! Paridade com o oráculo `lei_unica.py` (fixture gravada com cabeçalho; regerar com o comando do
//! cabeçalho). Tolerância ABSOLUTA por banda `TOL` sobre `R` e `C` (ambos em `[0,1]`).
//!
//! Porquê `2e-6`: a crate corre em `f32` (ε = 1,2e-7) e o oráculo em `f64`. Medido 31/10: max
//! |ΔR| `1,5e-7`, |ΔC| `1,8e-7`, chamadas cruas `1,5e-7` (≈ 1–2 ulp perto de 1). `TOL` ≈ 10× o
//! medido (margem para outra ordem de `libm`/CPU) e ~3 ordens abaixo de um passo de 8 bits
//! (1/255 linear no escuro ≈ 3e-4). A forma com cancelação perdia `4e-6` — este `TOL` reprova-a.

use ph2d_paint_tinta::{Bandas, Pigmento, camada, camada_banda, cobertura, mistura, s_fator};

const FIXTURE: &str = include_str!("../fixtures/lei_unica.tsv");
const TOL: f64 = 2.0e-6;

fn num(x: &str) -> f64 {
    x.parse()
        .unwrap_or_else(|_| panic!("número inválido na fixture: {x}"))
}

#[test]
fn a_crate_casa_com_o_oraculo_por_banda() {
    assert!(
        FIXTURE.starts_with("# fixture da lei única"),
        "fixture sem cabeçalho"
    );
    let mut pig: Vec<(String, Pigmento<3>)> = Vec::new();
    let (mut casos, mut crus, mut defeito_oraculo) = (0usize, 0usize, 0usize);
    let (mut max_r, mut max_c, mut max_k) = (0.0f64, 0.0f64, 0.0f64);
    for linha in FIXTURE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let c: Vec<&str> = linha.split('\t').collect();
        match c[0] {
            "P" => {
                let v: Vec<f32> = c[2..8].iter().map(|x| num(x) as f32).collect();
                pig.push((
                    c[1].to_owned(),
                    Pigmento::de_rinf(Bandas([v[0], v[1], v[2]]), Bandas([v[3], v[4], v[5]])),
                ));
            }
            "C" => {
                let azul = &pig.iter().find(|p| p.0 == "AZUL").expect("AZUL").1;
                let amarelo = &pig.iter().find(|p| p.0 == "AMARELO").expect("AMARELO").1;
                let (va, vb, m) = (num(c[1]) as f32, num(c[2]) as f32, num(c[3]) as f32);
                let o = mistura(&[(azul, va), (amarelo, vb)], s_fator(m, c[4] == "1"));
                let rg = Bandas([num(c[5]) as f32, num(c[6]) as f32, num(c[7]) as f32]);
                let r = camada(&o, &rg);
                let cob = cobertura(&o);
                for b in 0..3 {
                    let er = (f64::from(r.0[b]) - num(c[8 + b])).abs();
                    let ec = (f64::from(cob.0[b]) - num(c[11 + b])).abs();
                    assert!(
                        er <= TOL,
                        "R banda {b}: {} vs {} em «{linha}»",
                        r.0[b],
                        c[8 + b]
                    );
                    assert!(
                        ec <= TOL,
                        "C banda {b}: {} vs {} em «{linha}»",
                        cob.0[b],
                        c[11 + b]
                    );
                    max_r = max_r.max(er);
                    max_c = max_c.max(ec);
                }
                casos += 1;
            }
            "K" => {
                let (kt, st, rg) = (num(c[1]), num(c[2]), num(c[3]));
                let r = f64::from(camada_banda(kt as f32, st as f32, rg as f32));
                if kt == 0.0 && st > 1e-9 {
                    // Ramo do oráculo com b = 0: `0·coth(1e-12)` devolve 1 (ou 0/0). A verdade é o
                    // limite fechado K = 0: R = (S(1−R_g)+R_g)/(S(1−R_g)+1). Ver F4_1 §oráculo.
                    let verdade = (st * (1.0 - rg) + rg) / (st * (1.0 - rg) + 1.0);
                    assert!((r - verdade).abs() <= TOL, "K=0: {r} vs {verdade}");
                    defeito_oraculo += 1;
                    continue;
                }
                let er = (r - num(c[4])).abs();
                assert!(er <= TOL, "km_camada({kt},{st},{rg}) = {r} vs {}", c[4]);
                max_k = max_k.max(er);
                crus += 1;
            }
            outro => panic!("linha desconhecida: {outro}"),
        }
    }
    println!(
        "paridade: {casos} casos × 3 bandas (max |ΔR| {max_r:.2e}, max |ΔC| {max_c:.2e}) · \
         {crus} chamadas cruas (max {max_k:.2e}) · {defeito_oraculo} no ramo K=0 contra a forma fechada"
    );
    assert_eq!(pig.len(), 2);
    assert!(
        casos >= 1500 && crus >= 140,
        "fixture encolheu: {casos} casos, {crus} crus"
    );
}
