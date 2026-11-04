//! **Um núcleo == a pool, ao bit** — cada nó que ganhou `par_build` por medição (doc 98 §4).
//!
//! Numa pool de 1 trabalhador o mapa corre em ordem; numa de vários, não. Um corpo que dependa da
//! ordem (estado partilhado, redução de `f32`) só dá os mesmos bits por acaso, e aqui reprova: a
//! mutação vista vermelha foi um contador partilhado no lugar do índice.

use super::sonda_par_build::{casos, impressao, monta, registo, tique};
use ph2d_nodegraph::attr::{Column, PAR_THRESHOLD};
use ph2d_nodegraph::cook::Cook;

/// As linhas: acima do limiar, onde o caminho paralelo corre de facto.
const N: usize = 32_768;
/// Tiques por lado: o estado e o relógio têm de ser os mesmos.
const TIQUES: usize = 5;

/// Coze o caso `rotulo` nas duas pools; devolve as duas impressões e a primeira coluna `P`/`v`.
fn os_dois_lados(rotulo: &str) -> (u64, u64, Vec<f32>) {
    assert!(
        N >= PAR_THRESHOLD,
        "o gate tem de exercer o caminho paralelo"
    );
    let caso = casos()
        .into_iter()
        .find(|c| c.rotulo == rotulo)
        .expect("o caso");
    let reg = registo(N);
    let corre = |threads: usize| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("pool");
        let (mut g, no) = monta(&caso, N);
        let mut cook = Cook::new();
        let mut t = 0.0;
        pool.install(|| {
            for _ in 0..TIQUES {
                assert_eq!(tique(&mut cook, &mut g, no, &reg, &mut t), N);
            }
        });
        let s = cook.peek(no).expect("cozido")[0].as_stream().clone();
        let amostra = match (s.get("P"), s.get("v")) {
            (Some(Column::Vec2(p)), _) => p.iter().map(|q| q[0]).collect(),
            (_, Some(Column::Scalar(v))) => v.clone(),
            _ => panic!("sem coluna a comparar"),
        };
        (impressao(&s), amostra)
    };
    let (serial, amostra) = corre(1);
    let (paralelo, _) = corre(8);
    (serial, paralelo, amostra)
}

/// O nó não é a identidade nem uma constante — senão a igualdade não prova nada.
fn varia(amostra: &[f32]) -> bool {
    amostra.iter().any(|x| x.to_bits() != amostra[0].to_bits())
}

#[test]
fn o_skin_deformer_em_paralelo_e_o_serial_ao_bit() {
    let (serial, paralelo, amostra) = os_dois_lados("skin_deformer 16 ossos");
    assert!(varia(&amostra));
    assert_eq!(serial, paralelo, "rig.skin_deformer depende da ordem");
}

#[test]
fn a_expressao_em_paralelo_e_a_serial_ao_bit() {
    let (serial, paralelo, amostra) = os_dois_lados("expression");
    assert!(varia(&amostra));
    assert_eq!(serial, paralelo, "motion.expression depende da ordem");
}
