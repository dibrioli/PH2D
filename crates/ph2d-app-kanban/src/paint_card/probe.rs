//! ⚠️ SONDA: o que a última pintura de cada cartão mostrou, pelo título — só quando armada (os
//! testes da costura), para medir «a altura medida == a pintada» e o que a frente pintou.

use crate::docs::KanbanDocs;
use ph2d_editor_core::IconId;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::ColorToken;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// O que um cartão pintou.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Painted {
    /// A altura que a pintura percorreu (do topo ao fim do último elemento + o recuo).
    pub h: f32,
    /// Onde a imagem da capa caiu (o destino, antes do recorte) e o retângulo da capa.
    pub image: Option<(Rect, Rect)>,
    /// O alfa do véu do título da capa inteira e o contraste medido sobre a imagem.
    pub veil: Option<(f32, f32)>,
    /// O texto e o fundo do distintivo das datas.
    pub due: Option<(String, Option<ColorToken>)>,
    pub fields: Vec<String>,
    pub badges: Vec<IconId>,
}

thread_local! {
    static PAINTED: RefCell<Option<BTreeMap<String, Painted>>> = const { RefCell::new(None) };
}

/// O que o cartão `title` pintou por último (com a sonda armada).
#[cfg(test)]
pub(crate) fn last(title: &str) -> Option<Painted> {
    PAINTED.with_borrow(|m| m.as_ref()?.get(title).cloned())
}

/// Guarda o que o cartão `title` pintou (inerte sem a sonda armada).
pub(crate) fn note(title: &str, p: Painted) {
    PAINTED.with_borrow_mut(|m| {
        if let Some(m) = m {
            m.insert(title.to_owned(), p);
        }
    });
}

/// O que a frente de um cartão pintou: `(altura, destino da imagem e rect da capa, véu e
/// contraste, distintivo das datas, campos, ícones dos distintivos)`.
pub type PaintedFront = (
    f32,
    Option<(Rect, Rect)>,
    Option<(f32, f32)>,
    Option<(String, Option<ColorToken>)>,
    Vec<String>,
    Vec<IconId>,
);

impl KanbanDocs {
    /// ⚠️ SONDA: começa a guardar o que cada frente pinta (por título).
    pub fn probe_arm_fronts(&self) {
        PAINTED.with_borrow_mut(|m| *m = Some(BTreeMap::new()));
    }

    /// ⚠️ SONDA: o que o cartão `title` pintou no último quadro.
    #[must_use]
    pub fn probe_painted_front(&self, title: &str) -> Option<PaintedFront> {
        PAINTED.with_borrow(|m| {
            let p = m.as_ref()?.get(title)?.clone();
            Some((p.h, p.image, p.veil, p.due, p.fields, p.badges))
        })
    }

    /// ⚠️ SONDA: há capas de imagem a descodificar.
    #[must_use]
    pub fn probe_covers_pending(&self) -> bool {
        crate::front::cover::pending()
    }
}
