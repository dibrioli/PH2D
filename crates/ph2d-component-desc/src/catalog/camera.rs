//! ⭐⭐⭐ **A família CÂMERA** — o que a janela do jogo mostra (TOP-20 #7).
//!
//! # Porque ela é um módulo próprio, e uma categoria própria
//!
//! É a lei declarada no [`super`]: **uma família, um módulo** (isolamento, DIRETRIZ §1.5.2.1).
//!
//! ⛔ **A alternativa era pendurá-los em `Rendering`, e seria mentir**: aquela família responde
//! *«como o pixel deste objecto sai»* — blend, filtro, máscara, emissivo. A câmera não é uma
//! propriedade de um objecto que se desenha; ela é **o ponto de vista**, e o objecto que a carrega
//! muitas vezes não se desenha de todo.
//!
//! # ⚠️ São TRÊS componentes e não um com três secções, e a separação é o que o artista lê
//!
//! - **`GameCamera`** sozinho = uma câmera fixa (a de uma sala, a de um menu). É o caso mais comum
//!   e não precisa de mais nada.
//! - **`+ CameraFollow`** = ela persegue alguém. ⚠️ **A ausência do componente é a resposta** —
//!   um campo `follow_enabled` dentro da câmera daria o mesmo estado com um botão a mais e faria o
//!   painel mostrar cinco controlos mortos a quem só quer uma câmera fixa.
//! - **`+ CameraLimits`** = ela não sai da fase.
//!
//! ⚠️ **A lista está ORDENADA por `canonical_name`** — há gate (`the_catalog_is_sorted_and_unique`).

use crate::{
    ComponentCategory as C, ComponentDesc, ComponentDesc as D, FieldDesc, FieldKind as K,
    ObjectKinds as O, Propagation,
};

const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
    }
}

/// Os campos de quem persegue.
///
/// ⚠️ **`Target` é `Text` porque a referência durável desta casa é o NOME** — o undo respawna tudo
/// com bits novos, e um `Entity` gravado daria uma câmera a seguir o vazio depois do primeiro
/// `Ctrl+Z`. É a mesma decisão do `SignalActions` e do `AnchorMount`.
const FOLLOW_FIELDS: &[FieldDesc] = &[
    f(0, "component.field.follow_fields.0", K::Text),
    f(1, "component.field.follow_fields.1", K::Vec2),
    f(2, "component.field.follow_fields.2", K::Vec2),
    f(3, "component.field.follow_fields.3", K::Vec2),
    f(4, "component.field.follow_fields.4", K::Vec2),
];

/// Os campos da câmera.
const CAMERA_FIELDS: &[FieldDesc] = &[
    f(0, "component.field.camera_fields.0", K::Scalar),
    f(1, "component.field.camera_fields.1", K::Vec2),
    f(2, "component.field.camera_fields.2", K::Int),
    f(3, "component.field.camera_fields.3", K::Toggle),
    f(4, "component.field.camera_fields.4", K::Int),
];

/// Os limites da fase — a caixa de que a JANELA não sai.
const LIMITS_FIELDS: &[FieldDesc] = &[
    f(0, "component.field.limits_fields.0", K::Vec2),
    f(1, "component.field.limits_fields.1", K::Vec2),
];

/// Os descritores da família.
pub const DESCS: &[ComponentDesc] = &[
    // ⚠️ **`O::ANY` nos três, e é a decisão**: uma câmera é quase sempre um objecto VAZIO — o ponto
    // de vista que o artista põe onde quer. Restringir a `DRAWABLE` tornaria o caso canónico
    // inexprimível, que é o mesmo argumento que o ouvinte de áudio já pagou.
    D::authored(
        "ph2d::ecs::CameraFollow",
        "component.camera_follow.name",
        C::Camera,
        O::ANY,
        FOLLOW_FIELDS,
    ),
    D::authored(
        "ph2d::ecs::CameraLimits",
        "component.camera_limits.name",
        C::Camera,
        O::ANY,
        LIMITS_FIELDS,
    ),
    D::authored(
        "ph2d::ecs::GameCamera",
        "component.game_camera.name",
        C::Camera,
        O::ANY,
        CAMERA_FIELDS,
    ),
];
