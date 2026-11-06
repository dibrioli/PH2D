//! A ponte com a família: o instantâneo que ela publica antes de pintar e a fila de intenções que ela drena.

use std::cell::RefCell;
use std::rc::Rc;

use ph2d_a11y::NodeId;
use ph2d_sfx_recipe::{LayerId, Recipe, TakeId};

/// Um material do projeto, com o nome que a tela mostra.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub take: TakeId,
    pub segundos: f32,
}

/// O que o painel mostra, da aba ativa.
#[derive(Clone, Debug, PartialEq)]
pub struct Instantaneo {
    pub receita: Recipe,
    pub sel: Option<LayerId>,
    pub materiais: Vec<Material>,
}

impl Instantaneo {
    #[must_use]
    pub fn camada(&self) -> Option<&ph2d_sfx_recipe::Layer> {
        self.receita.layer(self.sel?)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intencao {
    Valor(NodeId, f64),
    Opcao(NodeId),
    Material(TakeId),
    /// Troca a curva da camada por uma de Attack · Decay · Close.
    FormaHit,
}

thread_local! {
    static ATUAL: RefCell<Option<Rc<Instantaneo>>> = const { RefCell::new(None) };
    static FILA: RefCell<Vec<Intencao>> = const { RefCell::new(Vec::new()) };
}

/// A família, uma vez por imagem; `None` = não há aba do som ativa.
pub fn publica(i: Option<Instantaneo>) {
    ATUAL.with(|a| *a.borrow_mut() = i.map(Rc::new));
}

pub(crate) fn atual() -> Option<Rc<Instantaneo>> {
    ATUAL.with(|a| a.borrow().clone())
}

pub(crate) fn pede(i: Intencao) {
    FILA.with(|f| f.borrow_mut().push(i));
}

/// A família drena a fila, na ordem em que a mão pediu.
#[must_use]
pub fn tomar() -> Vec<Intencao> {
    FILA.with(|f| std::mem::take(&mut *f.borrow_mut()))
}

/// O material de uma opção da lista suspensa.
pub(crate) fn material_de(id: NodeId) -> Option<TakeId> {
    let snap = atual()?;
    snap.materiais
        .iter()
        .enumerate()
        .find(|(i, _)| crate::ids::opcao_de_material(*i) == id)
        .map(|(_, m)| m.take)
}
