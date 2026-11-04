//! A cena da camada das listas e a porta de quem pinta: [`abre`] e [`por_cima`]. Mora abaixo dos
//! pintores de lista; os alvos e o fecho, que falam com o índice de cliques, ficam em
//! [`crate::panel::camada_das_listas`].

use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_vector::VectorScene;
use std::cell::RefCell;

/// O que a camada juntou no quadro.
pub(crate) struct Camada {
    pub(crate) cena: VectorScene,
    pub(crate) alvos: Vec<(NodeId, Rect)>,
}

thread_local! {
    static CAMADA: RefCell<Option<Camada>> = const { RefCell::new(None) };
}

/// Abre a camada do quadro; o que sobrou de um quadro interrompido perde-se.
pub fn abre(cena_do_quadro: &VectorScene) {
    let mut cena = VectorScene::new();
    if cena_do_quadro.is_recording() {
        cena.record();
    }
    CAMADA.with(|c| {
        *c.borrow_mut() = Some(Camada {
            cena,
            alvos: Vec::new(),
        });
    });
}

/// Pinta `f` na camada, se houver; senão em `scene`.
pub fn por_cima<R>(scene: &mut VectorScene, f: impl FnOnce(&mut VectorScene) -> R) -> R {
    match CAMADA.with(|c| c.borrow_mut().take()) {
        None => f(scene),
        Some(mut camada) => {
            let r = f(&mut camada.cena);
            CAMADA.with(|c| *c.borrow_mut() = Some(camada));
            r
        }
    }
}

/// Guarda um alvo para o fecho, se há camada aberta.
pub(crate) fn guarda_alvo(id: NodeId, rect: Rect) {
    CAMADA.with(|c| {
        if let Some(camada) = c.borrow_mut().as_mut() {
            camada.alvos.push((id, rect));
        }
    });
}

/// Fecha a camada e entrega o que ela juntou.
pub(crate) fn tira() -> Option<Camada> {
    CAMADA.with(|c| c.borrow_mut().take())
}
