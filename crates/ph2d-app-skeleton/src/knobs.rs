//! ⭐⭐⭐ **O QUE CADA NÚMERO DO PAINEL SIGNIFICA PARA UM OSSO** — a tradução `id → campo` e a
//! aplicação dela.
//!
//! ⚠️ **Ela mora na FAMÍLIA e não na fase do quadro**, pela lei do CLAUDE.md §2: a shell decide a
//! ORDEM em que as coisas acontecem; *o que* um controlo faz é conhecimento de quem possui o
//! componente. ⛔ Escrita na fase, ela cresceria com cada campo novo dentro da unidade de compilação
//! mais cara do repo.
//!
//! ⚠️ **O `BoneKnob` era um `bool`** (`true` = força), e os cinco campos da F8 não cabem num bit —
//! *um booleano que distingue dois casos é uma enumeração à espera do terceiro*.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids;
use ph2d_skeleton_ecs::Bone;

/// Qual número do osso um campo do painel escreveu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoneKnob {
    /// O comprimento, em unidades locais.
    Length,
    /// O alcance, em comprimentos deste osso.
    Strength,
    /// ⭐ Em quantos sub-ossos ele dobra (F8).
    Segments,
    /// A alça da raiz **ao longo** do eixo (o *ease*).
    CurveInX,
    /// ⭐ A alça da raiz **atravessada** ao eixo — é esta que arqueia.
    CurveInY,
    /// A alça da ponta ao longo do eixo.
    CurveOutX,
    /// ⭐ A alça da ponta atravessada ao eixo.
    CurveOutY,
}

/// ⭐⭐ **Que número do osso este id é.**
///
/// ⚠️ **Devolve `Option` e é isso que a mantém honesta:** um id que não seja do osso cai fora e
/// segue a cadeia do despacho. ⛔ Um braço `_ => Length` mandaria todo campo desconhecido escrever o
/// comprimento — a forma mais cara de um controlo parecer vivo.
#[must_use]
pub fn of_id(id: NodeId) -> Option<BoneKnob> {
    Some(match id {
        x if x == ids::VECTOR_BONE_LENGTH => BoneKnob::Length,
        x if x == ids::VECTOR_BONE_STRENGTH => BoneKnob::Strength,
        x if x == ids::VECTOR_BONE_SEGMENTS => BoneKnob::Segments,
        x if x == ids::VECTOR_BONE_CURVE_IN_X => BoneKnob::CurveInX,
        x if x == ids::VECTOR_BONE_CURVE_IN_Y => BoneKnob::CurveInY,
        x if x == ids::VECTOR_BONE_CURVE_OUT_X => BoneKnob::CurveOutX,
        x if x == ids::VECTOR_BONE_CURVE_OUT_Y => BoneKnob::CurveOutY,
        _ => return None,
    })
}

/// ⭐⭐ **Escreve o número no osso**, com as cercas de cada campo.
pub fn apply(bone: &mut Bone, knob: BoneKnob, v: f64) {
    match knob {
        // ⛔ Os dois são PISOS, não tetos: um comprimento negativo viraria o osso do avesso e uma
        // força negativa daria peso negativo. O tecto é o do documento — §0.0: um limite legítimo
        // diz de que recurso é, e não há recurso nenhum a limitar aqui.
        BoneKnob::Length => bone.length = v.max(0.0),
        BoneKnob::Strength => bone.strength = v.max(0.0),
        // ⭐ Os SEGMENTOS têm tecto, e ele é MEDIDO: a `Skin::weights_at` percorre todos os ossos
        // por ponto, então `N` segmentos multiplicam por `N` o custo deste osso. A tabela do custo
        // vive no doc do `MAX_SEGMENTS`. ⚠️ Guarda-se o número **já saturado** para o campo do
        // painel não mostrar um valor que a lei nunca honra.
        BoneKnob::Segments => {
            // ⚠️⚠️ **UMA porta e não duas.** A 1.ª redacção fazia `v.clamp(1, MAX)` ANTES do cast, e
            // a prova de mutação mostrou que aquele `clamp` é **redundante**: em Rust o `as u8`
            // SATURA (`1000.0 → 255`, `−5.0 → 0`) e o `segments_of` fecha os dois lados a seguir.
            // *Duas respostas à mesma pergunta divergem no dia em que alguém mexe numa delas* — e a
            // que manda é a da lei, que é onde o tecto é medido.
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "o `as u8` satura de propósito, e o `segments_of` é a porta do tecto"
            )]
            let n = v as u8;
            bone.segments = ph2d_skeleton::bend::segments_of(n);
        }
        // ⚠️ As quatro alças NÃO têm piso nem tecto, e a ausência é a decisão: uma alça negativa
        // arqueia para o outro lado, que é metade das poses que um rabo faz.
        BoneKnob::CurveInX => bone.curve.inn[0] = v,
        BoneKnob::CurveInY => bone.curve.inn[1] = v,
        BoneKnob::CurveOutX => bone.curve.out[0] = v,
        BoneKnob::CurveOutY => bone.curve.out[1] = v,
    }
}

#[cfg(test)]
#[path = "knobs_tests.rs"]
mod knobs_tests;
