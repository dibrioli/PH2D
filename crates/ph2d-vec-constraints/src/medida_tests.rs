//! ⏱ **Onde vai o tempo de juntar UMA regra** ([`a_mais`]) a 200 pontos — W6 ponta 3
//! (`docs/BlenderConstraints/07_W6_pontas.md` §2): componente · `Sistema::monta` · resíduos+linhas · posto, e o
//! posto denso (a referência, Gram–Schmidt) × o esparso (Householder) intercalados no MESMO processo, a ordem
//! rodada; resumo = o MÍNIMO. Sonda, não gate: `bash scripts/ph2d-run.sh cargo test --profile smoke -p
//! ph2d-vec-constraints --lib -- --ignored --nocapture where_the_join_of_one_rule_spends_its_time`.

use std::time::Instant;

use ph2d_vec_scene::cad::{Lei, RegraCad, SegCad};

use super::posto::{Base, Posto, PostoDenso};
use super::{Sistema, a_mais, a_mais_com, componente, posto};
use crate::posto_tests::cena;

const RODADAS: usize = 9;

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn minimo(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::INFINITY, f64::min)
}

#[test]
#[ignore = "sonda de velocidade: corre em --profile smoke, à mão"]
fn where_the_join_of_one_rule_spends_its_time() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", carga.trim());
    println!(
        "| cena | a_mais (denso) | a_mais (esparso) | componente | monta | linhas | posto denso | posto esparso |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    for nome in ["escada_202", "misto_200"] {
        let c = cena::cena(nome);
        let folha = c.folha();
        let nova = RegraCad {
            lei: Lei::Horizontal(SegCad(c.ponto[0], c.ponto[c.ponto.len() / 2])),
            ..c.regras[0]
        };
        let mut t = [const { Vec::new() }; 7];
        for rodada in 0..RODADAS {
            for k in 0..2 {
                let t0 = Instant::now();
                if (rodada + k) % 2 == 0 {
                    let _ = a_mais_com(&folha, &c.regras, nova, PostoDenso::new);
                    t[0].push(ms(t0));
                } else {
                    let _ = a_mais(&folha, &c.regras, nova);
                    t[1].push(ms(t0));
                }
            }
            let t0 = Instant::now();
            let todas = componente(&c.regras, nova);
            t[2].push(ms(t0));
            let t0 = Instant::now();
            let s = Sistema::monta(&folha, &todas, None, None);
            t[3].push(ms(t0));
            let t0 = Instant::now();
            let f = &s.valores;
            let valor = |v: &posto::Var| f[s.global(v)];
            let global = |v: &posto::Var| s.global(v);
            let linhas: Vec<Vec<posto::Linha>> = s
                .ativas
                .iter()
                .map(|&i| {
                    let (vars, res) = posto::residuos(&todas[i].lei, &valor);
                    posto::linhas(&vars, &res, &global)
                })
                .collect();
            t[4].push(ms(t0));
            let mut fora = [0, 0];
            for k in 0..2 {
                let esparso = (rodada + k) % 2 == 1;
                let t0 = Instant::now();
                let mut denso = PostoDenso::new(f.len());
                let mut esp = Posto::new(f.len());
                let n = linhas
                    .iter()
                    .filter(|l| {
                        !if esparso {
                            esp.aceita((*l).clone())
                        } else {
                            denso.aceita((*l).clone())
                        }
                    })
                    .count();
                t[5 + usize::from(esparso)].push(ms(t0));
                fora[usize::from(esparso)] = n;
            }
            assert_eq!(fora[0], fora[1], "as duas bases dão o mesmo posto");
        }
        let m: Vec<String> = t.iter().map(|v| format!("{:.3}", minimo(v))).collect();
        println!("| {nome} | {} |", m.join(" | "));
    }
}
