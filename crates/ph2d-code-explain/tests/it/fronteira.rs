//! Smoke do dono 11/11: cortes de `&str` por contagem de bytes caíam a meio de um caráter. A mensagem
//! do motor carrega texto do utilizador; o corte da posição (`strip_position`, 120 bytes) não pode partir.

#[test]
fn a_mensagem_com_caracter_de_varios_bytes_no_corte_nao_parte() {
    for pad in 115..125 {
        for c in ['«', 'é', '—', '🎮'] {
            let raw = format!("{}{c} bob.luau:12: attempt to index nil", "a".repeat(pad));
            let _ = ph2d_code_explain::explain(&raw);
        }
    }
}
