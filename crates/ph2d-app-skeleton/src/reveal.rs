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
pub fn on_focus(memoria: &mut Option<u64>, foco: Option<u64>) -> bool {
    if *memoria == foco {
        return false;
    }
    *memoria = foco;
    foco.is_some()
}

/// ⭐⭐⭐ **UM OSSO ACABADO DE NASCER NÃO É UM OSSO ESCOLHIDO** — absorve-o sem revelar nada.
///
/// ⛔⛔ **Report do dono (2026-09-09): *«cada vez que se cria um osso o modo Transform é
/// selecionado»*.** O osso novo fica aceso (é assim que o artista vê qual é), e no quadro seguinte
/// o [`on_focus`] lia essa mudança como *«o artista escolheu um osso»* — logo abria o painel,
/// trazia a aba e **armava *Transform***, arrancando-o do verbo em que ele estava a trabalhar.
///
/// ⚠️ **A aresta não tinha erro nenhum; o que faltava era a outra metade da história.** *«O foco
/// mudou»* tem duas causas que se leem iguais no fim do quadro — *o artista apontou* e *o gesto
/// produziu* —, e só quem produziu sabe distinguir. ⇒ quem cria o osso alimenta a memória, e a
/// aresta seguinte não tem nada a relatar.
///
/// ⛔ **Não é uma isenção nem um sinalizador**: a memória continua a ser o único estado desta lei, e
/// esta porta escreve exactamente o que o [`on_focus`] escreveria se o osso tivesse sido apontado.
/// Um `bool` *«ignora a próxima aresta»* ao lado dela seria um segundo estado a divergir do
/// primeiro no primeiro clique.
pub fn on_birth(memoria: &mut Option<u64>, novo: u64) {
    *memoria = Some(novo);
}

#[cfg(test)]
mod tests {
    use super::{on_birth, on_focus};

    /// ⭐⭐⭐ **CRIAR UM OSSO NÃO PEDE REVELAÇÃO NENHUMA** — o report de 2026-09-09, dito como lei.
    ///
    /// ⛔ Sem a absorção, o quadro seguinte à criação vê um foco NOVO e arma *Transform*: o artista
    /// larga o rato a fazer um osso e a ferramenta troca-lhe de verbo debaixo da mão.
    ///
    /// (Mutação: o `on_birth` não escrever a memória ⇒ RED na 1ª asserção.)
    #[test]
    fn a_newborn_bone_asks_for_nothing() {
        let mut m = None;
        on_birth(&mut m, 7);
        assert!(
            !on_focus(&mut m, Some(7)),
            "o osso RECEM-NASCIDO pediu a revelacao — o modo Transform seria armado a cada osso \
             criado, que e' literalmente o report do dono"
        );
        // ⚠️ E a lei geral continua inteira: OUTRO osso, escolhido a seguir, revela.
        assert!(
            on_focus(&mut m, Some(9)),
            "absorver o recem-nascido nao pode DESLIGAR a aresta — escolher outro osso ainda revela"
        );
        // ⚠️ E o mesmo osso, RE-escolhido depois de largado, também.
        assert!(!on_focus(&mut m, None));
        assert!(
            on_focus(&mut m, Some(7)),
            "re-escolher o osso criado tem de revelar — nascer nao o isenta para sempre"
        );
    }

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
