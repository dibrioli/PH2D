use super::*;

#[test]
fn a_entrada_fotografa_uma_vez_e_a_saida_devolve_a_fotografia() {
    let mut s = Sessao::<u32>::default();
    assert_eq!(s.quadro(false), Passo::Nada);
    assert_eq!(s.quadro(true), Passo::Fotografa);
    s.guarda(7);
    assert!(s.a_correr());
    // A correr (e em pausa, que continua a ser a corrida): nada.
    assert_eq!(s.quadro(true), Passo::Nada);
    assert_eq!(s.quadro(true), Passo::Nada);
    assert_eq!(s.quadro(false), Passo::Repoe(7));
    assert!(!s.a_correr());
    // A fotografia sai com a reposição: uma segunda saída não repõe outra vez.
    assert_eq!(s.quadro(false), Passo::Nada);
}

#[test]
fn cada_corrida_tem_a_sua_fotografia() {
    let mut s = Sessao::<u32>::default();
    assert_eq!(s.quadro(true), Passo::Fotografa);
    s.guarda(1);
    assert_eq!(s.quadro(false), Passo::Repoe(1));
    assert_eq!(s.quadro(true), Passo::Fotografa);
    s.guarda(2);
    assert_eq!(s.quadro(false), Passo::Repoe(2));
}

#[test]
fn uma_fotografia_fora_da_corrida_nao_conta() {
    let mut s = Sessao::<u32>::default();
    s.guarda(9);
    assert_eq!(s.quadro(true), Passo::Fotografa);
    assert_eq!(s.quadro(false), Passo::Nada);
}

#[test]
fn abrir_outro_documento_a_meio_esquece_a_fotografia() {
    let mut s = Sessao::<u32>::default();
    assert_eq!(s.quadro(true), Passo::Fotografa);
    s.guarda(3);
    s.esquece();
    assert_eq!(s.quadro(false), Passo::Nada);
}
