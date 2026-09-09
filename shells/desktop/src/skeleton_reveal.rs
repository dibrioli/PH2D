//! ⭐⭐⭐ **REVELAR-AO-FOCAR: a lei de quando a secção SKELETON vem à vista.**
//!
//! ⛔⛔ **Report do dono (2026-09-08): *«selecionar o bone nem sempre abre a secção de skeleton no
//! painel»*.** Medido no painel, com um viewport de `900 px` de altura:
//!
//! | o que está seleccionado | `y` do cabeçalho |
//! |---|---|
//! | só um osso (modo Osso) | **1316 px** |
//! | só um osso (modo Select) | 1316 px |
//! | osso + forma presa | 1316 px |
//! | \+ forma com traço | **1978 px** |
//!
//! ⇒ *ela **nunca** cabe na tela por si* — está sempre pelo menos `416 px` abaixo da dobra. O
//! *«nem sempre»* do report é o painel já estar rolado até lá por outra razão.
//!
//! ⚠️ **É a mesma lei que a timeline já segue** — *«seleccionar um objecto NOVO leva a timeline à
//! aba Keys»* (Enio, 2026-07-22).
//!
//! ⚠️ **Esta porta responde SÓ *«pedir ou não?»*.** *Se* o painel de facto rola é decisão dele, que
//! é o único sítio onde a faixa visível e o `y` do cabeçalho existem — um cabeçalho já à vista fica
//! onde está.

/// **Um osso NOVO entrou em foco?** Actualiza a memória e diz se a secção deve ser pedida.
///
/// ⚠️ **A ARESTA, nunca o estado.** Pedir a revelação em todo quadro em que um osso está em foco
/// tiraria ao artista o sítio onde ele está a ler: ele rolaria o painel e o quadro seguinte
/// puxava-o de volta, para sempre, enquanto o osso estivesse escolhido.
///
/// ⚠️ **Largar o osso ESQUECE-O** (`memoria` volta a `None`), e é de propósito: re-escolher o mesmo
/// osso revela outra vez, que é exactamente o gesto do report.
pub(crate) fn on_focus(memoria: &mut Option<u64>, foco: Option<u64>) -> bool {
    if *memoria == foco {
        return false;
    }
    *memoria = foco;
    foco.is_some()
}

#[cfg(test)]
mod tests {
    use super::on_focus;

    /// Um osso novo pede a revelação; o MESMO osso, quadro após quadro, não.
    #[test]
    fn only_a_new_bone_asks_for_the_section() {
        let mut m = None;
        assert!(on_focus(&mut m, Some(7)), "o 1o osso tem de pedir");
        for _ in 0..600 {
            assert!(
                !on_focus(&mut m, Some(7)),
                "o MESMO osso pediu outra vez — o painel ficaria preso e o artista nao conseguiria \
                 rolar para lado nenhum"
            );
        }
        assert!(on_focus(&mut m, Some(9)), "outro osso tem de pedir");
    }

    /// Largar o osso não pede nada — e faz o MESMO osso voltar a pedir quando for re-escolhido.
    #[test]
    fn letting_go_asks_for_nothing_and_arms_the_same_bone_again() {
        let mut m = None;
        assert!(on_focus(&mut m, Some(7)));
        assert!(
            !on_focus(&mut m, None),
            "largar o osso nao e' um pedido de revelacao"
        );
        assert_eq!(
            m, None,
            "largar tem de ESQUECER, senao o gesto do report nao repete"
        );
        assert!(
            on_focus(&mut m, Some(7)),
            "re-escolher o mesmo osso tem de revelar — e' literalmente o gesto do report"
        );
    }
}
