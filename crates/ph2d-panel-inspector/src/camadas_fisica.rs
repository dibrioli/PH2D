//! **O que as camadas de colisão se chamam e em quem tocam** (ficha f1, onda 2): a app publica a tabela do projeto e as
//! linhas da matriz do mundo a cada quadro; o cartão Collision Layers e quem mais mostra uma camada leem-nas daqui.

use std::cell::RefCell;

const CAMADAS: usize = crate::ids::INSP_PHYS_LAYERS;

struct Camadas {
    nomes: [String; CAMADAS],
    linhas: [u32; CAMADAS],
    /// As que se mostram, em bits (`ph2d_physics_ecs::visible_layers`).
    visiveis: u32,
}

thread_local! {
    static AGORA: RefCell<Camadas> = RefCell::new(Camadas {
        nomes: Default::default(),
        linhas: [u32::MAX; CAMADAS],
        visiveis: 0xFF,
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

/// Uma vez por quadro: os nomes dados (vazio = sem nome), a matriz do mundo e as camadas visíveis.
pub fn set_current(nomes: [String; CAMADAS], linhas: [u32; CAMADAS], visiveis: u32) {
    AGORA.with(|a| {
        *a.borrow_mut() = Camadas {
            nomes,
            linhas,
            visiveis,
        };
    });
}

/// A camada aparece nas listas?
#[must_use]
pub(crate) fn visivel(i: usize) -> bool {
    AGORA.with(|a| a.borrow().visiveis & (1u32 << i) != 0)
}

/// O nome dado à camada, vazio se não tem.
#[must_use]
pub(crate) fn nome(i: usize) -> String {
    AGORA.with(|a| a.borrow().nomes.get(i).cloned().unwrap_or_default())
}

/// O que se lê: o nome, ou o padrão («Default», «Layer n») sem nome dado (dono 07/11; o projeto não grava o «Layer n»).
#[must_use]
pub fn rotulo(i: usize) -> String {
    ph2d_editor_core::nome_curto::nome_de_camada(&rotulo_inteiro(i))
}

/// O mesmo sem o corte «…»: o que as dicas mostram (onda 9d).
#[must_use]
pub(crate) fn rotulo_inteiro(i: usize) -> String {
    let n = nome(i);
    if n.is_empty() { rotulo_padrao(i) } else { n }
}

/// O que mostra a camada sem nome: a 0 é «Default» (onda 9a, maquete B), as outras «Layer n».
#[must_use]
pub(crate) fn rotulo_padrao(i: usize) -> String {
    if i == 0 {
        return ph2d_i18n::tr("card.physics.layer_zero").to_owned();
    }
    ph2d_i18n::tr_with("card.physics.layer_default", &[("n", &i.to_string())])
}

/// A linha `a` da matriz viva.
#[must_use]
pub(crate) fn linha(a: usize) -> u32 {
    AGORA.with(|c| c.borrow().linhas.get(a).copied().unwrap_or(0))
}

/// A próxima camada por mostrar, a mesma conta de `LayerNames::next_free`; `None` nas 32.
#[must_use]
pub fn proxima_livre() -> Option<u8> {
    let livre = (!AGORA.with(|c| c.borrow().visiveis)).trailing_zeros();
    u8::try_from(livre)
        .ok()
        .filter(|&l| usize::from(l) < CAMADAS)
}
