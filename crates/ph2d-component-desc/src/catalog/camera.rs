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
    // ⭐⭐⭐ **O DOLLY** (plano 24, W5) — a profundidade, em fracções da distância focal.
    f(5, "component.field.camera_fields.5", K::Scalar),
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
    f(0, "component.field.shake_fields.0", K::Scalar),
    f(1, "component.field.shake_fields.1", K::Scalar),
    f(2, "component.field.shake_fields.2", K::Scalar),
    f(3, "component.field.shake_fields.3", K::Int),
    // ⭐ `Seed` e nao `Int`: ela nao tem valor CERTO, so tem de ser DIFERENTE.
    f(4, "component.field.shake_fields.4", K::Seed),
];

/// ⭐⭐⭐ **Uma FONTE de abanão** (suplente #25) — como a vigia e o gatilho, o componente é uma LISTA
/// e isto descreve a LINHA.
const SHAKE_SOURCE_FIELDS: &[FieldDesc] = &[
    // ⚠️ **Vazio = calada** — a lei da vigia, do gatilho e da §11.
    f(0, "component.field.shake_source_fields.0", K::Text),
    f(1, "component.field.shake_source_fields.1", K::Enum),
    f(2, "component.field.shake_source_fields.2", K::Scalar),
    f(3, "component.field.shake_source_fields.3", K::Scalar),
    f(4, "component.field.shake_source_fields.4", K::Scalar),
];

/// ⭐⭐⭐ **A PARALAXE** (plano 24, W1) — **UM campo**, e é isso a wave inteira.
///
/// ⚠️ **`K::Vec2` e não dois `Scalar`**, e a razão é a que o [`K::Vec4`] já escreve ao lado: o
/// componente guarda **um** `[f32; 2]`, logo descrevê-lo como dois daria dois `field_id` ao mesmo
/// campo e um override gravado sobre metade dele não teria onde ser aplicado. *O descritor espelha
/// a ESTRUTURA, nunca o layout do painel* — que aqui até é mesmo uma fileira com dois números.
const SCROLL_FIELDS: &[FieldDesc] = &[f(0, "component.field.scroll_fields.0", K::Vec2)];
/// ⚠️ Em METROS e não em pixels: o `Transform` desta casa já é métrico, e a única px→m é a do
/// projecto (`pixels_per_meter`).
const REPEAT_FIELDS: &[FieldDesc] = &[f(0, "component.field.repeat_fields.0", K::Vec2)];
/// ⚠️ Metros por SEGUNDO — o eixo do tempo é o playhead, e é isso que a faz sobreviver a um scrub.
const MOTION_FIELDS: &[FieldDesc] = &[f(0, "component.field.motion_fields.0", K::Vec2)];

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
        "component.camera_shake.name",
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
    // ⭐⭐⭐ **A PARALAXE** (plano 24, W1) — *quanto do movimento do mundo este objecto guarda*.
    // ⚠️ Ele mora na família da CÂMERA e nunca vive numa: o número inteiro dele é sobre o movimento
    // DELA, e é ali que o artista o procura (a mesma fronteira do `ShakeEmitter`, abaixo).
    // ⚠️ `O::ANY` porque a lei só lê o `Transform`: um fundo é uma sprite, um vector ou um objecto
    // vazio com filhos, e nenhum deles é mais «paralaxe» que o outro.
    D::authored(
        "ph2d::ecs::ScrollFactor",
        "component.scroll_factor.name",
        C::Camera,
        O::ANY,
        SCROLL_FIELDS,
    ),
    // ⛔⛔ **As três irmãs REQUEREM o `ScrollFactor`** (auditoria 26, §2.5): a ponte só percorre quem
    // o tem e a secção só existe com ele ⇒ anexadas sozinhas ficavam inertes, invisíveis e sem gesto
    // de remoção. O `requires` faz a paleta trazer o `Parallax` com elas.
    // ⭐⭐ **O CONFINAMENTO** (plano 24, W3) — *a borda do fundo nunca entra em cena*. ⚠️ Ele
    // reaproveita os `LIMITS_FIELDS` do `CameraLimits`, e isso é a decisão certa: são a MESMA
    // grandeza (uma região `min`/`max` em metros), e uma segunda tabela com os mesmos dois campos
    // daria dois rótulos para um conceito. ⛔ E não é `largura`/`altura`: o joelho da lei mede a
    // borda da VISTA contra a borda da REGIÃO, e uma largura sem origem não a nomeia.
    D::authored_requiring(
        "ph2d::ecs::ScrollLimits",
        "component.scroll_limits.name",
        C::Camera,
        O::ANY,
        LIMITS_FIELDS,
        &["ph2d::ecs::ScrollFactor"],
    ),
    // ⭐⭐ **O MOVIMENTO PRÓPRIO** (plano 24, W4) — *nuvens que andam sozinhas*, e ele é uma
    // função PURA do playhead. ⚠️ Antes do `ScrollRepeat` porque a lista é ORDENADA por
    // `canonical_name` e há gate.
    D::authored_requiring(
        "ph2d::ecs::ScrollMotion",
        "component.scroll_motion.name",
        C::Camera,
        O::ANY,
        MOTION_FIELDS,
        &["ph2d::ecs::ScrollFactor"],
    ),
    // ⭐⭐ **A REPETIÇÃO** (plano 24, W2) — *quanto mede um ladrilho deste fundo*. ⛔ Irmã das três
    // de cima e não um campo delas: quase todo objecto com paralaxe não repete, e um campo ali
    // seria um knob morto em todos eles. *Quatro componentes porque são quatro populações.*
    D::authored_requiring(
        "ph2d::ecs::ScrollRepeat",
        "component.scroll_repeat.name",
        C::Camera,
        O::ANY,
        REPEAT_FIELDS,
        &["ph2d::ecs::ScrollFactor"],
    ),
    // ⭐⭐⭐ **QUEM EXPLODE** (suplente #25) — e ele mora na família da CÂMERA apesar de nunca viver
    // numa: *o assunto é o abanão*, e pô-lo na família LÓGICA separaria as duas metades de uma lei
    // que só se lê inteira. ⚠️ `O::ANY` pela razão das irmãs: quem explode é tantas vezes um objecto
    // VAZIO quanto uma sprite.
    D::authored(
        "ph2d::ecs::ShakeEmitter",
        "component.shake_emitter.name",
        C::Camera,
        O::ANY,
        SHAKE_SOURCE_FIELDS,
    ),
];

#[cfg(test)]
mod tests {
    /// ⛔⛔ **As três irmãs da paralaxe REQUEREM o `ScrollFactor`** (auditoria 26, §2.5) — sem ele a
    /// ponte não as percorre e a secção não existe: anexadas sozinhas ficavam inertes, invisíveis e
    /// sem gesto de remoção. O CONTROLO: o próprio factor não requer nada.
    #[test]
    fn as_irmas_da_paralaxe_requerem_o_factor() {
        for nome in [
            "ph2d::ecs::ScrollRepeat",
            "ph2d::ecs::ScrollLimits",
            "ph2d::ecs::ScrollMotion",
        ] {
            let d = super::super::desc_for(nome).expect(nome);
            assert_eq!(d.requires, &["ph2d::ecs::ScrollFactor"], "{nome}");
        }
        let f = super::super::desc_for("ph2d::ecs::ScrollFactor").expect("factor");
        assert!(f.requires.is_empty());
    }
}
