//! ⭐ **As dicas dos cartões**: todo ícone sem texto pintado num cartão regista-se aqui (o censo
//! `every_icon_in_a_card_has_a_tooltip` exige-lhe uma dica), e o campo numérico ganha a dica do
//! arrasto com o rótulo dele (maquete B: «Length — drag the icon · Shift ×10 · Alt fine»).

use ph2d_a11y::NodeId;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

thread_local! {
    static LISTA: Cell<Option<NodeId>> = const { Cell::new(None) };
    static ICONES: RefCell<BTreeMap<NodeId, BTreeSet<NodeId>>> =
        const { RefCell::new(BTreeMap::new()) };
    static ROTULOS: RefCell<BTreeMap<NodeId, String>> = const { RefCell::new(BTreeMap::new()) };
}

pub(super) fn comeca(panel: NodeId) {
    LISTA.with(|l| l.set(Some(panel)));
    ICONES.with(|i| i.borrow_mut().remove(&panel));
}

pub(super) fn icone(id: NodeId) {
    if let Some(p) = LISTA.with(Cell::get) {
        ICONES.with(|i| {
            i.borrow_mut().entry(p).or_default().insert(id);
        });
    }
}

pub(super) fn rotulo(id: NodeId, l: &str) {
    ROTULOS.with(|r| {
        let mut r = r.borrow_mut();
        if r.get(&id).is_none_or(|v| v != l) {
            r.insert(id, l.to_string());
        }
    });
}

/// Os ícones sem texto que a lista de `panel` pintou no último quadro.
#[must_use]
pub fn icon_only_ids(panel: NodeId) -> Vec<NodeId> {
    ICONES.with(|i| {
        i.borrow()
            .get(&panel)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    })
}

/// A dica do campo numérico de cartão `id` (o arrasto, com o rótulo dele).
#[must_use]
pub fn field_hint(store: &crate::interaction::WidgetStore, id: NodeId) -> Option<String> {
    store.card_field(id)?;
    let dica = ph2d_i18n::tr("card.field.drag_hint");
    Some(match ROTULOS.with(|r| r.borrow().get(&id).cloned()) {
        Some(l) => ph2d_i18n::tr("card.field.drag_hint_named")
            .replace("{label}", &l)
            .replace("{hint}", dica),
        None => dica.to_string(),
    })
}
