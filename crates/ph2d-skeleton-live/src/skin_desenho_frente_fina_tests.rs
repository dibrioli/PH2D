//! Gates da malha fina ([`super`]).

use super::ix;

#[test]
fn o_indice_da_grelha_triangular_e_a_ordem_de_construcao() {
    for d in 1..=8 {
        let mut n = 0;
        for j in 0..=d {
            for i in 0..=d - j {
                assert_eq!(ix(d, i, j), n, "d={d} i={i} j={j}");
                n += 1;
            }
        }
    }
}
