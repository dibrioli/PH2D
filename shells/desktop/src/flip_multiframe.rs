//! ADR-0114 W7 — **multiframe**: o mesmo gesto edita N quadros.
//!
//! A feature-assinatura do Grease Pencil para animação (`02_referencia §11`): com chaves
//! selecionadas na tira, esculpir/preencher age em **todas elas de uma vez**, com um
//! **falloff** por distância temporal.
//!
//! O alvo é resolvido **ANTES** do gesto, e os consumidores só iteram `(drawing, frame,
//! falloff)` — é o que mantém o Sculpt e o balde ignorantes do multiframe.
//!
//! ## As três regras da referência, e por que cada uma existe
//!
//! **1. Dedup por `DrawingId` (a que vem com exclamação no doc).** Um MESMO desenho pode
//! ser referenciado por VÁRIAS chaves — é o "duplicate as instance", e é como um ciclo
//! reusa arte (`FlipDrawing::users`). Sem o dedup, um gesto de escultura aplicaria o pincel
//! **duas vezes no mesmo buffer**: a linha andaria o dobro num quadro e o animador veria a
//! arte se deformar sozinha, só nos quadros instanciados. É o tipo de bug que ninguém
//! atribui ao multiframe.
//!
//! **2. O falloff só multiplica influência de PINCEL.** Ops discretas (o balde, o delete)
//! usam `1.0`: meio-preenchimento não existe.
//!
//! **3. Inserir uma chave ao desenhar LIMPA a seleção** (`flip_strip`) — senão o próximo
//! gesto de escultura sairia esculpindo quadros que o usuário esqueceu de desmarcar.

use ph2d_core::Playhead;
use ph2d_flip::{DrawingId, FlipDoc, FlipObjectId, Frame, LayerId};

/// Um quadro-alvo do gesto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Target {
    pub(crate) did: DrawingId,
    /// A chave (a 1ª, se o desenho é instanciado por várias — ver o dedup).
    pub(crate) frame: Frame,
    /// `1.0` no quadro ativo; menor nos vizinhos quando o falloff está ligado.
    pub(crate) falloff: f32,
}

/// Piso do falloff. O quadro mais distante da seleção não pode receber influência ZERO —
/// isso o tornaria um alvo que o usuário selecionou e que não se mexe (a ferramenta
/// pareceria ignorá-lo). O mesmo raciocínio — e o mesmo remédio — do `GHOST_MIN_ALPHA`
/// dos fantasmas.
const MIN_FALLOFF: f32 = 0.15; // LITERAL-OK: piso de influencia, medido pelo mesmo criterio do ghost

/// **O falloff temporal** — meia-vida geométrica: **50% por quadro de distância**.
///
/// `presença = 0.5^|delta|`, com piso em `MIN_FALLOFF`. É **SIMÉTRICO** (só depende de
/// `|delta|` — não do lado nem de quão espalhada está a seleção) e independente do span:
/// *"cada quadro de distância, metade da influência"* (Enio 2026-07-15, *"50% mais claro a
/// cada frame"* + *"por que não gradua simetricamente?"*). Substituiu a tenda linear
/// normalizada-por-lado da referência do GP — que era assimétrica de propósito, mas fazia o
/// mesmo `|delta|` pesar diferente nos dois lados quando o ativo não estava centrado, e o
/// animador leu isso como bug. O piso impede que um quadro marcado fique totalmente inerte
/// (o mesmo raciocínio do `GHOST_MIN_ALPHA`); e é por ele — não pela meia-vida — que a
/// mutação de retirar o `max(MIN_FALLOFF)` sangra.
///
/// `delta` = distância em quadros do alvo ao quadro ATIVO (o SINAL não importa: simétrico).
#[must_use]
pub(crate) fn falloff_at(delta: i32) -> f32 {
    // `delta == 0` ⇒ `0.5^0 = 1.0` (influência cheia), sem caso especial. `powi` é
    // multiplicação repetida — determinístico e transcendental-free (HR-5).
    0.5f32.powi(delta.abs()).max(MIN_FALLOFF)
}

/// **O peso do multiframe que a chave `k` recebe** — a PRÉVIA que a tira pinta em cada
/// célula selecionada. `1.0` fora do multiframe (0/1 chave marcada) ou com o falloff
/// desligado; senão a meia-vida do falloff. É a MESMA `falloff_at` que a escultura usa,
/// então a cor da célula não pode mentir sobre a força do gesto.
#[must_use]
pub(crate) fn cell_weight(
    selection: &[Frame],
    active_frame: Frame,
    k: Frame,
    falloff_on: bool,
) -> f32 {
    if selection.len() < 2 || !falloff_on {
        return 1.0;
    }
    falloff_at(k - active_frame)
}

/// **Os quadros que este gesto edita.**
///
/// - Sem seleção múltipla (0 ou 1 chave marcada): devolve **só o quadro ativo**, com
///   `falloff = 1.0`. É o caminho de sempre, byte a byte — quem nunca usou o multiframe não
///   vê diferença nenhuma.
/// - Com N chaves: todas elas, **deduplicadas por `DrawingId`**, mais o quadro ativo (que
///   entra sempre — é o `+ frame atual como fallback` da referência) com falloff `1.0`.
///
/// `active_did` é o desenho que o gesto já resolveu pelo caminho normal (com autokey, que
/// PODE ter criado uma chave). Ele é a âncora: o Δ temporal e o falloff são medidos dele.
///
/// Nenhuma chave é CRIADA aqui: as selecionadas já existem (é o que "chave" significa na
/// tira), e o alvo ativo veio pronto. Multiframe **não inventa quadro**.
#[must_use]
pub(crate) fn targets(
    flip: &FlipDoc,
    oid: FlipObjectId,
    lid: LayerId,
    playhead: &Playhead,
    selection: &[Frame],
    active: (DrawingId, Frame),
    falloff_on: bool,
) -> Vec<Target> {
    let (active_did, active_frame) = active;
    let mut out = vec![Target {
        did: active_did,
        frame: active_frame,
        falloff: 1.0,
    }];
    if selection.len() < 2 {
        return out; // o caminho de sempre
    }
    let _ = playhead;
    let Some(layer) = flip.object(oid).and_then(|o| o.layer(lid)) else {
        return out;
    };
    // chave → desenho (a API canônica da camada; sentinelas de fim já filtradas).
    let cells = layer.cells();

    for &k in selection {
        let Some(&(_, did, _)) = cells.iter().find(|(f, _, _)| *f == k) else {
            continue; // a chave sumiu (apagada) — a seleção é estado de UI, o doc manda
        };
        // **O DEDUP.** Duas chaves que compartilham o desenho (instância / ciclo) são UM
        // alvo: aplicar o pincel duas vezes no mesmo buffer dobraria o efeito nelas.
        if out.iter().any(|t| t.did == did) {
            continue;
        }
        let falloff = if falloff_on {
            falloff_at(k - active_frame)
        } else {
            1.0
        };
        out.push(Target {
            did,
            frame: k,
            falloff,
        });
    }
    out
}

#[cfg(test)]
#[path = "flip_multiframe_tests.rs"]
mod tests;
