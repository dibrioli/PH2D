//! ⭐ **Quando o Done devolve a cena** (doc 17 item 9): nunca a meio de um gesto (botão em baixo) nem
//! com o jogo a correr — nesses quadros o fim da sessão NÃO se consome e fica armado até largar /
//! parar. A shell, ao devolver, regista primeiro o diff pendente como passo (as chaves ficam).

/// `true` = devolver a cena AGORA. `fim` é o `take_ended` do painel: só é chamado (e consumido)
/// quando nada impede a devolução.
#[must_use]
pub fn devolve_agora(
    botao_em_baixo: bool,
    jogo_a_correr: bool,
    fim: impl FnOnce() -> bool,
) -> bool {
    !botao_em_baixo && !jogo_a_correr && fim()
}

#[cfg(test)]
mod tests {
    use super::devolve_agora;
    use std::cell::Cell;

    /// *Mutação: chamar `fim()` primeiro ⇒ o fim consome-se a meio do arrasto e nunca volta.*
    #[test]
    fn done_held_button_or_game_keeps_the_end_armed() {
        let armado = Cell::new(true);
        let fim = || armado.replace(false);
        assert!(
            !devolve_agora(true, false, fim),
            "botão em baixo: não repõe"
        );
        assert!(
            !devolve_agora(false, true, fim),
            "o jogo a correr: não repõe"
        );
        assert!(armado.get(), "o fim ficou armado");
        assert!(devolve_agora(false, false, fim), "largou: repõe");
        assert!(
            !armado.get() && !devolve_agora(false, false, fim),
            "uma vez só"
        );
    }
}
