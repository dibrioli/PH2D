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
        Some(RecusaDoOsso::VariosEsqueletos { quantos: 2 }),
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

/// ⭐⭐⭐ **TODA recusa tem chave de i18n, e ELAS SÃO DISTINTAS.**
///
/// ⛔ Duas variantes com a mesma chave dizem a mesma frase para dois factos diferentes — e o artista
/// lê *«nada a soltar»* quando o que falta é escolher um osso.
#[test]
fn toda_recusa_tem_chave_propria() {
    let mut vistas = std::collections::BTreeSet::new();
    for r in RecusaDoOsso::TODAS {
        let k = r.chave();
        assert!(
            k.starts_with("skeleton.recusa."),
            "a chave de {r:?} nao vive no espaco desta familia: {k}"
        );
        assert!(vistas.insert(k), "duas recusas partilham a chave {k}");
    }
    assert_eq!(
        vistas.len(),
        RecusaDoOsso::TODAS.len(),
        "o censo perdeu uma recusa pelo caminho"
    );
}

/// ⚠️ **Só a recusa que NOMEIA um número o carrega** — as outras devolvem `None`, senão a frase
/// delas teria um `{quantos}` por preencher na tela.
#[test]
fn so_a_recusa_dos_esqueletos_carrega_um_numero() {
    assert_eq!(
        RecusaDoOsso::VariosEsqueletos { quantos: 4 }.quantos(),
        Some(4)
    );
    assert_eq!(RecusaDoOsso::NadaAPrender.quantos(), None);
    assert_eq!(RecusaDoOsso::NadaASoltar.quantos(), None);
}
