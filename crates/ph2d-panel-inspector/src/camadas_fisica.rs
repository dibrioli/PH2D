//! **O que as camadas de colisão se chamam e em quem tocam** (ficha f1, onda 2): a app publica a tabela do projeto e as
//! linhas da matriz do mundo a cada quadro; o cartão Shape e quem mais mostra uma camada leem-nas daqui.

use ph2d_i18n::{tr, tr_with};
use std::cell::RefCell;

const CAMADAS: usize = 8;

struct Camadas {
    nomes: [String; CAMADAS],
    linhas: [u8; CAMADAS],
}

thread_local! {
    static AGORA: RefCell<Camadas> = RefCell::new(Camadas {
        nomes: Default::default(),
        linhas: [u8::MAX; CAMADAS],
    });
}

/// As fotos: o Player com os cartões de cima dobrados, para Body e Shape caberem.
pub fn dobra_os_cartoes_de_cima(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    for id in [
        crate::CARD_IMAGE,
        crate::CARD_LOOK_ANIMADA,
        ph2d_editor_core::ids::INSP_LIVE_ANIM_SECTION,
    ] {
        store.set_collapsed(id, true);
    }
}

/// As fotos: o campo do «Rename…» aberto, como depois do clique.
pub fn abre_o_rename(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    store.set_collapsed(crate::ids::INSP_PHYS_LAYER_RENAMING, false);
}

/// As fotos: a lista da camada aberta, como depois do clique.
pub fn abre_a_lista(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    use ph2d_editor_core::interaction::InteractiveState;
    if let Some(InteractiveState::Dropdown { open, .. }) =
        store.get_mut(crate::ids::INSP_PHYS_LAYER_PICK)
    {
        *open = true;
    }
}

/// Uma vez por quadro: os nomes dados (vazio = sem nome) e a matriz do mundo.
pub fn set_current(nomes: [String; CAMADAS], linhas: [u8; CAMADAS]) {
    AGORA.with(|a| *a.borrow_mut() = Camadas { nomes, linhas });
}

/// O nome dado à camada, vazio se não tem.
#[must_use]
pub(crate) fn nome(i: usize) -> String {
    AGORA.with(|a| a.borrow().nomes.get(i).cloned().unwrap_or_default())
}

/// O que se lê: o nome, ou o número sem nome dado.
#[must_use]
pub(crate) fn rotulo(i: usize) -> String {
    let n = nome(i);
    if n.is_empty() { i.to_string() } else { n }
}

/// «Hits: Ground, Enemy», pela matriz do mundo; «Hits: nothing» sem nenhuma.
#[must_use]
pub fn hits(camada: usize) -> String {
    let linha = AGORA.with(|a| a.borrow().linhas.get(camada).copied().unwrap_or(0));
    let nomes: Vec<String> = (0..CAMADAS)
        .filter(|j| linha & (1 << j) != 0)
        .map(rotulo)
        .collect();
    if nomes.is_empty() {
        tr("card.insp.phys.hits_nothing").to_owned()
    } else {
        tr_with("card.insp.phys.hits", &[("names", &nomes.join(", "))])
    }
}
