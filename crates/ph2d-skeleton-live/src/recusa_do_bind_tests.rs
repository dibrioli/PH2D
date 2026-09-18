//! Os gates da recusa do *Bind*.

use super::*;
use crate::esqueletos_tests_support::cadeias;

/// ⭐⭐⭐ **A metade NEGATIVA, e ela é metade do valor:** com UM esqueleto na cena o *Bind* sem osso
/// escolhido é o caminho de sempre, e a porta cala-se.
///
/// ⛔ Sem isto, a cura partiria o fluxo que o artista já aprendeu para tratar um caso que só existe
/// quando há ambiguidade — *um botão que se queixa sempre é ruído*.
#[test]
fn com_um_esqueleto_so_a_porta_cala_se() {
    let mut sim = SimWorld::default();
    cadeias(&mut sim, 1);
    assert_eq!(
        recusa_do_bind(&sim, None),
        None,
        "com UM esqueleto na cena o bind sem osso escolhido passou a ser recusado — isto parte o \
         fluxo de sempre para curar um caso que ali nao existe"
    );
}

/// ⭐⭐⭐ **COM DOIS, ela recusa — e diz QUANTOS.**
#[test]
fn com_dois_esqueletos_e_sem_osso_a_porta_recusa() {
    let mut sim = SimWorld::default();
    cadeias(&mut sim, 2);
    assert_eq!(
        recusa_do_bind(&sim, None),
        Some(RecusaDoBind::VariosEsqueletos { quantos: 2 }),
        "com DOIS esqueletos e nenhum osso escolhido o bind seguiu em frente — a forma ficaria \
         presa aos seis ossos das duas cadeias, que e' o defeito medido na sonda do rig partilhado"
    );
}

/// ⚠️ **E com o osso escolhido ela cala-se, mesmo com dois esqueletos** — a ambiguidade acabou.
#[test]
fn com_o_osso_escolhido_a_porta_cala_se() {
    let mut sim = SimWorld::default();
    let raizes = cadeias(&mut sim, 2);
    assert_eq!(
        recusa_do_bind(&sim, Some(raizes[0])),
        None,
        "com o osso escolhido a porta ainda recusa: ela deixou de ler a semente, e o gesto que \
         desambigua passou a ser inutil"
    );
}

/// ⛔ **A frase diz o problema E o gesto que o cura.**
///
/// ⚠️ Uma recusa que só diz o problema manda o artista adivinhar, que é o mesmo que não dizer nada —
/// e ela carrega o NÚMERO, senão não se distingue de uma queixa genérica.
#[test]
fn a_frase_da_recusa_nomeia_a_cura_e_o_numero() {
    let f = RecusaDoBind::VariosEsqueletos { quantos: 3 }.frase();
    assert!(
        f.contains('3'),
        "a frase nao diz QUANTOS esqueletos ha': {f}"
    );
    assert!(
        f.contains("osso") && f.contains("Hierarquia"),
        "a frase nao diz o gesto que cura (escolher tambem um osso na Hierarquia): {f}"
    );
}
