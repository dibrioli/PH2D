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

/// **O falloff temporal** — a curva do GP, achatada no essencial.
///
/// A referência descreve UMA curva com o quadro ativo em `x = 0.5`, os anteriores em
/// `[0, 0.5)` e os posteriores em `(0.5, 1]` — o que dá **atenuação assimétrica de graça**
/// (o passado e o futuro podem cair em ritmos diferentes). Aqui a curva é a tenda linear
/// normalizada por cada LADO da seleção, o que preserva essa assimetria: dois quadros para
/// trás e dez para a frente não fazem o vizinho de trás cair dez vezes mais rápido.
///
/// `delta` = distância em quadros do alvo ao quadro ATIVO (negativo = antes).
#[must_use]
pub(crate) fn falloff_at(delta: i32, span_before: i32, span_after: i32) -> f32 {
    // (Sem caso especial para `delta == 0`: a aritmética abaixo já devolve `1.0` ali —
    // `t = 0` ⇒ influência cheia. Um early-return redundante mentiria sobre onde a
    // invariante mora, e a mutação que o removia não derrubava gate nenhum. Quem guarda a
    // propriedade é o teste `falloff_at(0, …) == 1.0`.)
    let span = if delta < 0 { span_before } else { span_after };
    if span <= 0 {
        return 1.0;
    }
    let t = (delta.abs() as f32 / span as f32).clamp(0.0, 1.0);
    (1.0 - t).mul_add(1.0 - MIN_FALLOFF, MIN_FALLOFF)
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

    // Os dois lados da seleção, medidos a partir do quadro ATIVO — é o que dá a
    // assimetria da curva (ver `falloff_at`).
    let (mut span_before, mut span_after) = (0, 0);
    for &k in selection {
        let d = k - active_frame;
        span_before = span_before.max(-d);
        span_after = span_after.max(d);
    }

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
            falloff_at(k - active_frame, span_before, span_after)
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
