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

/// ⭐⭐⭐ **COMO esta câmera treme** (suplente #25) — os cinco números da lei do abanão.
///
/// ⚠️ **«Punch» e não «Exponent»:** o artista escolhe o CARÁCTER do abanão, e a potência a que o
/// trauma é elevado é como a lei o exprime, não como ele o pensa.
const SHAKE_FIELDS: &[FieldDesc] = &[
    f(0, "Amplitude", K::Scalar),
    f(1, "Frequency", K::Scalar),
    f(2, "Decay", K::Scalar),
    f(3, "Punch", K::Int),
    // ⭐ `Seed` e nao `Int`: ela nao tem valor CERTO, so tem de ser DIFERENTE.
    f(4, "Seed", K::Seed),
];

/// ⭐⭐⭐ **Uma FONTE de abanão** (suplente #25) — como a vigia e o gatilho, o componente é uma LISTA
/// e isto descreve a LINHA.
const SHAKE_SOURCE_FIELDS: &[FieldDesc] = &[
    // ⚠️ **Vazio = calada** — a lei da vigia, do gatilho e da §11.
    f(0, "On", K::Text),
    f(1, "From", K::Enum),
    f(2, "Strength", K::Scalar),
    f(3, "Full Within", K::Scalar),
    f(4, "Nothing Beyond", K::Scalar),
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
    // ⭐⭐⭐ **O ABANÃO DA VISTA** (suplente #25) — *como* esta câmera treme.
    //
    // ⚠️⚠️ **Entre o `CameraLimits` e a `GameCamera`, e isso NÃO é estilo — o GATE apanhou-me:** a
    // família é procurada por busca BINÁRIA, e fora de ordem o descritor devolve `None` para um tipo
    // que existe. *Um descritor que não é encontrado lê-se exactamente como um que não existe*, e a
    // 1.ª redacção pôs este bloco no topo do ficheiro.
    //
    // ⛔ **Ele NÃO requer a `GameCamera`, e a ausência é a decisão:** um abanão numa entidade sem
    // câmera é inerte e o painel DI-LO (*«This is not the camera in command»*), enquanto exigi-lo
    // impediria o artista de o preparar antes de anexar a câmera.
    D::authored(
        "ph2d::ecs::CameraShake",
        "Camera Shake",
        C::Camera,
        O::ANY,
        SHAKE_FIELDS,
    ),
    D::authored(
        "ph2d::ecs::GameCamera",
        "component.game_camera.name",
        C::Camera,
        O::ANY,
        CAMERA_FIELDS,
    ),
    // ⭐⭐⭐ **QUEM EXPLODE** (suplente #25) — e ele mora na família da CÂMERA apesar de nunca viver
    // numa: *o assunto é o abanão*, e pô-lo na família LÓGICA separaria as duas metades de uma lei
    // que só se lê inteira. ⚠️ `O::ANY` pela razão das irmãs: quem explode é tantas vezes um objecto
    // VAZIO quanto uma sprite.
    D::authored(
        "ph2d::ecs::ShakeEmitter",
        "Shake Emitter",
        C::Camera,
        O::ANY,
        SHAKE_SOURCE_FIELDS,
    ),
];
