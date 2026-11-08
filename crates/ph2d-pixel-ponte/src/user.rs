//! **O cartão User Data ↔ o documento** (P12 U): o [`UserData`] de um dono em [`UserDataView`], e o
//! gesto do cartão ([`CardEdit`]) no [`Op::UserData`] da porta única. Partilhado pela slice, camada,
//! cel e tag (as janelas só vêem a vista; a família só traduz).

use ph2d_editor_core::panel::cards::user_data::{CardEdit, PropRow, UserDataView};
use ph2d_editor_core::pixel_panel::{PixelOwner, PixelUserInfo};
use ph2d_pixel_core::user::edit::{PropKind, UserEdit, apply_edit, kind_of, type_name, value_text};
use ph2d_pixel_core::user::{Owner, PropMap};
use ph2d_pixel_core::{FrameId, LayerId, Op, Sprite, UserData};

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

/// O gesto do cartão sobre `u`, no vocabulário do documento (`Kind` roda o tipo actual; uma linha
/// só de leitura não roda).
#[must_use]
pub fn traduz(u: &UserData, e: &CardEdit) -> Option<UserEdit> {
    Some(match e.clone() {
        CardEdit::Color(c) => UserEdit::Color(c),
        CardEdit::Text(t) => UserEdit::Text(t),
        CardEdit::Add => UserEdit::Add,
        CardEdit::Remove(k) => UserEdit::Remove(k),
        CardEdit::Rename(a, b) => UserEdit::Rename(a, b),
        CardEdit::Value(k, v) => UserEdit::Value(k, v),
        CardEdit::Kind(k) => {
            let atual = u.props.get("").and_then(|m| m.get(&k)).and_then(kind_of)?;
            UserEdit::Kind(k, PropKind::next(atual))
        }
    })
}

/// O dono do chrome no documento.
#[must_use]
pub const fn dono(o: PixelOwner) -> Owner {
    match o {
        PixelOwner::Sprite => Owner::Sprite,
        PixelOwner::Layer(l) => Owner::Layer(LayerId(l)),
        PixelOwner::Cel { layer, frame } => Owner::Cel((LayerId(layer), FrameId(frame))),
        PixelOwner::Tag(i) => Owner::Tag(i),
        PixelOwner::Tileset(t) => Owner::Tileset(ph2d_pixel_core::TilesetId(t)),
    }
}

/// O passo do gesto `e` sobre `owner`; `None` = nada muda.
#[must_use]
pub fn op(s: &Sprite, owner: PixelOwner, e: &CardEdit) -> Option<Op> {
    let owner = dono(owner);
    let u = s.user_of(owner)?;
    let data = apply_edit(u, &traduz(u, e)?)?;
    Some(Op::UserData {
        owner,
        data: Box::new(data),
    })
}

/// O user data da camada, do cel (`layer × frame`) e da tag que cobre o frame activo (a 1.ª, como
/// as Tag Properties da linha do tempo).
#[must_use]
pub fn info(s: &Sprite, layer: LayerId, frame: FrameId) -> PixelUserInfo {
    let v = |o: Option<PixelOwner>| o.and_then(|o| s.user_of(dono(o))).map(vista);
    let pos = s.frame_index(frame).and_then(|p| u32::try_from(p).ok());
    let tag = pos.and_then(|p| s.tags().iter().position(|t| t.from <= p && p <= t.to));
    let (layer, frame) = (Some(layer.0), Some(frame.0));
    PixelUserInfo {
        layer: v(layer.map(PixelOwner::Layer)),
        cel: v(layer
            .zip(frame)
            .map(|(layer, frame)| PixelOwner::Cel { layer, frame })),
        tag: v(tag.map(PixelOwner::Tag)),
        sprite: v(Some(PixelOwner::Sprite)),
    }
}
