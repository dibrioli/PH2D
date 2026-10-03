//! Gates do canal de opacidade.

use crate::Tinta;

fn tinta() -> Tinta {
    let faces: Vec<[u32; 3]> = vec![[0, 1, 2]];
    Tinta::nova(3, faces.iter().map(|f| &f[..]), 1)
}

/// ⭐⭐ Sem o canal o plano é opaco, e ele RECUSA ganhar transparência por uma
/// escrita solta; com o canal, a escrita entra e a contagem errada é recusada.
#[test]
fn um_plano_sem_canal_e_opaco_e_nao_aceita_transparencia_solta() {
    let mut t = tinta();
    let n = t.amostras().len();
    assert!(!t.tem_alfa());
    assert_eq!(t.opacidade(0), 1.0);
    assert!(
        t.define_opacidade(0, 1.0),
        "escrever 1 num plano opaco é a mesma coisa"
    );
    assert!(
        !t.define_opacidade(0, 0.5),
        "transparência solta num plano opaco"
    );
    assert!(!t.com_alfa(Some(vec![0.5; n + 1])), "contagem errada");
    assert!(t.com_alfa(Some(vec![0.25; n])));
    assert!(t.define_opacidade(1, 0.75));
    assert_eq!((t.opacidade(0), t.opacidade(1)), (0.25, 0.75));
    let antes = t.footprint_bytes();
    assert_eq!(t.tira_alfa().map(|a| a.len()), Some(n));
    assert_eq!(
        antes - t.footprint_bytes(),
        n * 4,
        "o canal conta no orçamento"
    );
}
