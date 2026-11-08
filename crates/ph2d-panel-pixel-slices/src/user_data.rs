//! **O cartão User Data de um dono do Pixel** (P12 U): o [`UserData`] do documento em
//! [`UserDataView`], e cada gesto do cartão em [`UserEdit`]. Partilhado pelas janelas do Pixel que
//! têm um dono (a slice aqui; camada, cel, tag, sprite e tileset nos seus painéis).

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::user_data::{Hit, PropRow, UserDataIds, UserDataView, hit};
use ph2d_pixel_core::UserData;
use ph2d_pixel_core::user::PropMap;
use ph2d_pixel_core::user::edit::{PropKind, UserEdit, kind_of, type_name, value_text};

fn linhas(m: &PropMap) -> Vec<PropRow> {
    m.iter()
        .map(|(k, v)| PropRow {
            name: k.clone(),
            value: value_text(v),
            kind: type_name(v),
            editable: kind_of(v).is_some(),
        })
        .collect()
}

/// O que o cartão mostra de `u`.
#[must_use]
pub fn vista(u: &UserData) -> UserDataView {
    UserDataView {
        color: u.color,
        text: u.text.clone(),
        rows: u.props.get("").map(linhas).unwrap_or_default(),
        extensions: u
            .props
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, m)| (k.clone(), linhas(m)))
            .collect(),
    }
}

/// O gesto de `id` sobre `u`, lido do campo em `store` quando é um texto; `None` = não é do cartão,
/// ou nada muda. `Hit::Toggle` e `Hit::Color` são do painel (abrir, o seletor).
#[must_use]
pub fn edit_of(
    store: &WidgetStore,
    ids: &UserDataIds,
    u: &UserData,
    id: NodeId,
) -> Option<UserEdit> {
    let v = vista(u);
    let nome = |i: usize| v.rows.get(i).map(|r| r.name.clone());
    let texto = || store.text(id).map(str::to_owned);
    let mapa = u.props.get("");
    Some(match hit(ids, v.rows.len(), id)? {
        Hit::Toggle | Hit::Color => return None,
        Hit::Text => UserEdit::Text(texto()?),
        Hit::Add => UserEdit::Add,
        Hit::Remove(i) => UserEdit::Remove(nome(i)?),
        Hit::Name(i) => UserEdit::Rename(nome(i)?, texto()?),
        Hit::Value(i) => UserEdit::Value(nome(i)?, texto()?),
        Hit::Kind(i) => {
            let k = nome(i)?;
            let atual = mapa.and_then(|m| m.get(&k)).and_then(kind_of)?;
            UserEdit::Kind(k, PropKind::next(atual))
        }
    })
}
