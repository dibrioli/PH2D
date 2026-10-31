//! **O que cada número do painel significa para um osso** — a tradução `id → campo` e a aplicação
//! dela, na FAMÍLIA e não na fase do quadro (CLAUDE.md §2).

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

impl BoneKnob {
    /// **Todos os números do osso** — a população do censo de knobs (o `match` exaustivo de
    /// `a_lista_todos_cobre_o_enum` guarda-a).
    pub const TODOS: [Self; 7] = [
        Self::Length,
        Self::Strength,
        Self::Segments,
        Self::CurveInX,
        Self::CurveInY,
        Self::CurveOutX,
        Self::CurveOutY,
    ];
}

/// ⭐⭐ **Que número do osso este id é.**
/// `None` para um id que não é do osso (nunca um `_ => Length`).
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

/// O piso do comprimento: em `0` o osso não tem eixo e o empurrão dos filhos não tem de onde voltar
/// (`empurra_filhos` recusa `l0 <= 0`) — arrastar Length até 0 colapsava-os na raiz para sempre
/// (smoke 30/10). `1e-3` é invisível e mantém a ida-e-volta do `f32` (o filho a meio volta ao bit
/// de `1e-4`, `comprimento_tests`).
pub const COMPRIMENTO_MINIMO: f64 = 1e-3;

/// ⭐⭐ **Escreve o número no osso**, com as cercas de cada campo.
pub fn apply(bone: &mut Bone, knob: BoneKnob, v: f64) {
    match knob {
        // PISOS, não tetos: negativo viraria o osso do avesso / o peso negativo.
        BoneKnob::Length => bone.length = v.max(COMPRIMENTO_MINIMO),
        BoneKnob::Strength => bone.strength = v.max(0.0),
        // Os segmentos: o `as u8` satura e o `segments_of` é a porta do tecto MEDIDO (`MAX_SEGMENTS`).
        BoneKnob::Segments => {
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

#[cfg(test)]
#[path = "censo_dos_knobs_do_osso_tests.rs"]
mod censo_dos_knobs_do_osso_tests;

#[cfg(test)]
mod todos_tests {
    use super::BoneKnob;

    /// ⭐ **A lista `TODOS` cobre o enum** — o `match` é exaustivo, logo uma variante nova **não
    /// compila** até vir à lista.
    #[test]
    fn a_lista_todos_cobre_o_enum() {
        for k in BoneKnob::TODOS {
            // ⚠️ O `match` sem `_ =>` é o gate: uma variante nova **não compila** aqui.
            match k {
                BoneKnob::Length
                | BoneKnob::Strength
                | BoneKnob::Segments
                | BoneKnob::CurveInX
                | BoneKnob::CurveInY
                | BoneKnob::CurveOutX
                | BoneKnob::CurveOutY => {}
            }
        }
        // DISTINTOS, e não só a contagem: uma cópia de outro item manteria os `7` (mutação).
        let distintos: std::collections::BTreeSet<String> = BoneKnob::TODOS
            .into_iter()
            .map(|k| format!("{k:?}"))
            .collect();
        assert_eq!(
            distintos.len(),
            BoneKnob::TODOS.len(),
            "a lista TODOS tem uma variante REPETIDA: outra ficou de fora, e o censo mede-a zero \
             vezes sem reprovar"
        );
        assert_eq!(
            BoneKnob::TODOS.len(),
            7,
            "a populacao do censo mudou sem o numero mudar — um knob novo entrou e o censo \
             continua a medir sete"
        );
    }
}
