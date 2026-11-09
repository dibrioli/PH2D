//! As slices entre o documento e o `.ase` (P11).

use ph2d_aseprite::{AseExternalFile, AseFile, AseSlice, AseSliceKey, AseUserData};
use ph2d_pixel_core::{Rect, Slice, SliceId, SliceKey, SliceShape, Sprite};

/// As slices do ficheiro, na ordem dele, com ids a partir de `primeiro`. Nada se corta (F3).
pub(super) fn slices_de(file: &AseFile, primeiro: u32) -> Vec<Slice> {
    let ext = &file.external_files;
    file.slices
        .iter()
        .enumerate()
        .map(|(i, s)| Slice {
            id: SliceId(primeiro + i as u32),
            name: s.name.clone(),
            user: super::user::de(
                Some(&AseUserData {
                    text: Some(s.text.clone()),
                    color: s.color,
                    maps: s.maps.clone(),
                }),
                ext,
            ),
            keys: s
                .keys
                .iter()
                .map(|k| SliceKey {
                    frame: k.frame,
                    shape: SliceShape {
                        bounds: Rect::new(k.x, k.y, k.w, k.h),
                        center: k.center.map(|(x, y, w, h)| Rect::new(x, y, w, h)),
                        pivot: k.pivot,
                    },
                })
                .collect(),
        })
        .collect()
}

pub(super) fn slices_saida(s: &Sprite, ext: &[AseExternalFile]) -> Vec<AseSlice> {
    s.slices()
        .iter()
        .map(|c| AseSlice {
            name: c.name.clone(),
            color: (c.user.color != [0; 4]).then_some(c.user.color),
            text: c.user.text.clone(),
            maps: super::user::saida(&com_pivo(s, c), ext).map_or_else(Vec::new, |u| u.maps),
            keys: c
                .keys
                .iter()
                .map(|k| {
                    let b = k.shape.bounds;
                    AseSliceKey {
                        frame: k.frame,
                        x: b.x,
                        y: b.y,
                        w: b.w,
                        h: b.h,
                        center: k.shape.center.map(|r| (r.x, r.y, r.w, r.h)),
                        pivot: k.shape.pivot,
                    }
                })
                .collect(),
        })
        .collect()
}

/// P14e: a propriedade do mapa do artista que diz «esta slice é a do pivô do sprite».
pub const PIVOT_PROP: &str = "ph2d.pivot";

/// O user data da slice, com `ph2d.pivot` se ela é a `pivot_slice`.
fn com_pivo(s: &Sprite, c: &Slice) -> ph2d_pixel_core::user::UserData {
    let mut u = c.user.clone();
    if s.pivot_slice() == Some(c.id) {
        u.props.entry(String::new()).or_default().insert(
            PIVOT_PROP.to_owned(),
            ph2d_pixel_core::user::PropValue::Bool(true),
        );
    }
    u
}

/// Tira `ph2d.pivot` das slices lidas: a 1.ª que a tinha é a `pivot_slice`.
pub(super) fn tira_pivo(slices: &mut [Slice]) -> Option<SliceId> {
    let mut pivo = None;
    for c in slices {
        let Some(m) = c.user.props.get_mut("") else {
            continue;
        };
        let v = m.remove(PIVOT_PROP);
        if m.is_empty() {
            c.user.props.remove("");
        }
        if v == Some(ph2d_pixel_core::user::PropValue::Bool(true)) && pivo.is_none() {
            pivo = Some(c.id);
        }
    }
    pivo
}

/// P14e: a propriedade do mapa do artista que diz «esta slice é um collider». Ao ler sai do user data
/// (o collider é do mundo); ao gravar entra pela cópia que o app passa (`com_colliders`).
pub const COLLIDER_PROP: &str = "ph2d.collider";

/// Os ids das slices marcadas collider.
pub type Colliders = std::collections::BTreeSet<u32>;

/// Tira `ph2d.collider` das slices lidas e devolve as que o tinham a `true`.
pub(super) fn tira_collider(slices: &mut [Slice]) -> Colliders {
    let mut ids = Colliders::new();
    for c in slices {
        let Some(m) = c.user.props.get_mut("") else {
            continue;
        };
        if m.remove(COLLIDER_PROP) == Some(ph2d_pixel_core::user::PropValue::Bool(true)) {
            ids.insert(c.id.0);
        }
        if m.is_empty() {
            c.user.props.remove("");
        }
    }
    ids
}
