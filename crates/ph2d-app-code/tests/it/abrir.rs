//! ⭐ **E3 — o «Edit script» abre a aba CERTA com o texto da biblioteca**, e o segundo pedido não a
//! duplica. (O clique real no cartão até ao pedido é provado em `ph2d-panel-inspector` e em
//! `ph2d-app-components`; aqui o pedido no mundo até à aba activa.)

use crate::Mesa;
use ph2d_editor_core::doc_surface::DocumentSurface;

const BOB: &str = "-- bob\nfunction update(self, dt)\nend\n";
const ANA: &str = "-- ana\n";

#[test]
fn o_pedido_abre_a_aba_com_o_texto_da_biblioteca() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB), ("ana.luau", ANA)]);
    // Controlo: sem pedido não há aba.
    m.quadro();
    assert!(m.docs().tabs().is_empty());
    assert!(m.hero.documents.scene_active());

    m.pede(ids[1]);
    let tabs = m.docs().tabs();
    assert_eq!(tabs.len(), 1);
    assert_eq!(
        (tabs[0].id, tabs[0].title.as_str(), tabs[0].dirty),
        (ids[1].0, "ana.luau", false)
    );
    assert_eq!(m.docs().text_of(ids[1]).as_deref(), Some(ANA));
    assert_eq!(
        m.hero.documents.surface_active().map(|t| t.id),
        Some(ids[1].0),
        "a aba aberta fica activa"
    );

    m.pede(ids[0]);
    assert_eq!(m.docs().text_of(ids[0]).as_deref(), Some(BOB));
    assert_eq!(
        m.hero.documents.surface_active().map(|t| t.id),
        Some(ids[0].0)
    );
}

#[test]
fn o_segundo_pedido_activa_a_aba_que_ja_existe() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB), ("ana.luau", ANA)]);
    m.pede(ids[0]);
    m.pede(ids[1]);
    m.pede(ids[0]);
    assert_eq!(
        m.docs().tabs().len(),
        2,
        "o mesmo script não abre duas abas"
    );
    assert_eq!(
        m.hero.documents.surface_active().map(|t| t.id),
        Some(ids[0].0)
    );
}

/// ⛔ Fechar com alterações por salvar NUNCA descarta: o documento estaciona e volta como estava.
#[test]
fn fechar_com_alteracoes_guarda_o_texto_e_reabre_com_ele() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    m.pede(ids[0]);
    m.escreve(ids[0], "x");
    let escrito = m.docs().text_of(ids[0]).expect("aberto");
    assert_ne!(escrito, BOB);
    m.docs().close(ids[0].0);
    m.quadro();
    assert!(m.docs().tabs().is_empty());
    assert!(m.hero.documents.scene_active(), "a aba fechada cai na Cena");
    assert!(m.docs().is_parked(ids[0]) && m.docs().is_dirty(ids[0]));
    assert_eq!(m.texto_salvo(ids[0]), BOB, "fechar não salva");
    m.pede(ids[0]);
    assert_eq!(
        m.docs().text_of(ids[0]),
        Some(escrito),
        "reabre com o texto por salvar"
    );
    assert!(m.docs().tabs()[0].dirty, "e ainda com o ●");
    // Controlo: sem alterações, fechar larga o documento (o texto está na biblioteca).
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    m.pede(ids[0]);
    m.docs().close(ids[0].0);
    assert!(!m.docs().is_parked(ids[0]));
}
