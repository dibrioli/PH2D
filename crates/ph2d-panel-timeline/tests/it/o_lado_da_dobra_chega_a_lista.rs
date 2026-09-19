//! ⭐⭐⭐ **O CANAL DO LADO DA DOBRA É ALCANÇÁVEL PELO ARTISTA** (pedido do dono, 2026-09-18).
//!
//! ⛔⛔ **Sem a linha na lista do *+ Track*, um canal tem resolver, rótulo e chave de i18n e SÓ UM
//! TESTE lhe chega.** Foi exactamente o que aconteceu ao `PropKind::Morph`, e a nota que o regista
//! vive ao lado da tabela em `ids/timeline.rs`. *O canal existia e o artista não lhe chegava.*

use ph2d_timeline::PropKind;

/// ⭐ Ele está na lista, e o **id dele é próprio** (dois botões com o mesmo id abrem a mesma track).
#[test]
fn o_lado_da_dobra_esta_na_lista_do_mais_track() {
    let tabela = ph2d_panel_timeline::ids::ADDPROP_BUTTONS;
    assert!(
        tabela.iter().any(|(_, p)| *p == PropKind::IkBendSide),
        "o lado da dobra nao esta' na lista do «+ Track»: o canal existe e o artista nao lhe chega"
    );
    let ids: std::collections::BTreeSet<_> = tabela.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        ids.len(),
        tabela.len(),
        "dois botoes do «+ Track» partilham o mesmo id: um deles abre a track do outro, e o \
         artista nao consegue criar a que pediu"
    );
    // ⚠️ E o CONTROLO de que a lista é a POPULAÇÃO e não um subconjunto de conveniência: os quatro
    // canais do osso que vieram antes continuam lá. *Uma lista que encolhe em silêncio é como o
    // Morph ficou de fora.*
    for p in [
        PropKind::BoneBendInX,
        PropKind::BoneBendInY,
        PropKind::BoneBendOutX,
        PropKind::BoneBendOutY,
    ] {
        assert!(
            tabela.iter().any(|(_, q)| *q == p),
            "{p:?} saiu da lista do «+ Track»: um canal do osso deixou de ser alcançavel"
        );
    }
}
