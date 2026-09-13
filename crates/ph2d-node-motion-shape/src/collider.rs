//! ⭐⭐ **O COLISOR QUE A FORMA DECLARA** — doc 109, ordem do dono (2026-09-13): *«colidem
//! sozinhas»*.
//!
//! ⚠️ **Só quem desenha sabe o tamanho do que desenha.** Com o mesmo `size = 1` um `Circle` desta
//! forma desenha raio `1` e uma sprite desenha raio inscrito `0,5`; nenhum consumidor a jusante
//! sabe que mídia recebeu. Por isso a declaração nasce AQUI e viaja como colunas
//! ([`COLLIDER_BOX_COLUMN`] · [`COLLIDER_COLUMN`] · [`COLLIDER_OFFSET_COLUMN`]).
//!
//! ## A forma do colisor: `Box` ou `Circle`, as duas ADAPTADAS à forma (doc 109 §5)
//!
//! Report do dono (13/09, com foto): *«collider impreciso, o collider não é gerado conforme a forma
//! da Shape. No mínimo precisamos de colliders circulares e retangulares que tentam se adaptar às
//! dimensões da shape e que tenham ajustes de tamanho com gizmo visível»*.
//!
//! - **`Box`** (o default) — a **caixa envolvente** do contorno: um quadrado declara o próprio
//!   quadrado, um retângulo o próprio retângulo. `Collider Width`/`Collider Height` multiplicam-na.
//! - **`Circle`** — o círculo que toca os lados MAIORES dessa caixa: num círculo é o próprio
//!   círculo, num quadrado o inscrito. `Collider Radius` multiplica-o.
//!
//! As duas centram-se no MEIO da caixa, e o [`COLLIDER_OFFSET_COLUMN`] leva esse centro quando ele
//! não é a origem da peça (uma estrela de cinco pontas é mais alta em cima que em baixo).
//!
//! ⛔⛔ **O círculo À VOLTA do contorno (a 1.ª redacção, `Around`) é o defeito do report:** um
//! quadrado de meio-lado `0,11` declarava raio `√2 · 0,11 = 0,156`, e a pilha assentava com `0,31`
//! entre centros onde o lado é `0,22` — `41 %` de ar entre peças que o olho lê como caixas.
//!
//! ⚠️ **A caixa sai da GEOMETRIA, e a geometria é do shell** — este nó não alcança a biblioteca
//! vectorial, por desenho. O shell mede a caixa envolvente uma vez por geometria e publica-a com
//! ela ([`param::COLLIDER_FIT_CENTER_COL`] · [`param::COLLIDER_FIT_HALF_COL`]); o nó DECLARA pelos
//! params dele e **retira sempre** as duas colunas. Elas não podem ser a declaração: a chave de
//! conteúdo é partilhada por toda forma com a mesma geometria, e duas formas iguais podem declarar
//! coisas diferentes.

use super::param;
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, INV_INERTIA_COLUMN,
    Stream,
};

/// Os rótulos do `Collider Shape`. ⚠️ O índice é formato de arquivo — APPEND ONLY.
pub(crate) static SHAPE_LABELS: &[&str] = &["Box", "Circle"];

/// O índice de `Box` em [`SHAPE_LABELS`] — o default.
pub(crate) const SHAPE_BOX: i32 = 0;
/// O índice de `Circle` em [`SHAPE_LABELS`].
pub(crate) const SHAPE_CIRCLE: i32 = 1;

/// Um multiplicador autorado, lido: um negativo é zero, e um não-finito não é um pedido — lê como a
/// identidade.
fn multiplicador(k: f32) -> f32 {
    if k.is_finite() { k.max(0.0) } else { 1.0 }
}

/// Uma coluna `Vec2` do stream publicado, se tiver o comprimento dele.
fn vec2(s: &Stream, name: &str) -> Option<Vec<[f32; 2]>> {
    match s.get(name) {
        Some(Column::Vec2(v)) if v.len() == s.count() => Some(v.clone()),
        _ => None,
    }
}

/// O stream publicado → o stream que o nó emite: as colunas da caixa envolvente **retiradas**, e o
/// colisor escrito só com o `Collide` ligado.
///
/// ⚠️ **Desligado, nenhuma coluna nova** — a lei estrutural do `fill`/`rotation` deste nó: o
/// default não é *«escrever o mesmo valor»*, é *«não escrever»*.
///
/// ⚠️ Ligado mas sem a caixa publicada (um cozimento adiantado, antes do `publish` do shell)
/// ⇒ também nada: um colisor inventado seria pior que nenhum.
pub(crate) fn declare(published: Stream, param: impl Fn(&str) -> f32) -> Stream {
    let n = published.count();
    let (centro, meia) = (
        vec2(&published, param::COLLIDER_FIT_CENTER_COL),
        vec2(&published, param::COLLIDER_FIT_HALF_COL),
    );
    let publicou = published.get(param::COLLIDER_FIT_CENTER_COL).is_some()
        || published.get(param::COLLIDER_FIT_HALF_COL).is_some();
    let mut out = if publicou {
        let mut s = Stream::new(n);
        for (name, col) in published.columns() {
            if name != param::COLLIDER_FIT_CENTER_COL && name != param::COLLIDER_FIT_HALF_COL {
                s.set(name.clone(), col.clone());
            }
        }
        s
    } else {
        published
    };
    if n == 0 || param(param::COLLIDE) < 0.5 {
        return out;
    }
    let (Some(centro), Some(meia)) = (centro, meia) else {
        return out;
    };
    if param(param::COLLIDER_SHAPE).round() as i32 == SHAPE_CIRCLE {
        let k = multiplicador(param(param::COLLIDER_RADIUS));
        out.set(
            COLLIDER_COLUMN,
            Column::Scalar(meia.iter().map(|m| m[0].max(m[1]) * k).collect()),
        );
    } else {
        let (w, h) = (
            multiplicador(param(param::COLLIDER_WIDTH)),
            multiplicador(param(param::COLLIDER_HEIGHT)),
        );
        out.set(
            COLLIDER_BOX_COLUMN,
            Column::Vec2(meia.iter().map(|m| [m[0] * w, m[1] * h]).collect()),
        );
    }
    // O centro só se escreve FORA da origem — a mesma lei estrutural: a arte centrada declara o
    // que toda declaração anterior a esta coluna já queria dizer.
    if centro.iter().any(|c| *c != [0.0, 0.0]) {
        out.set(COLLIDER_OFFSET_COLUMN, Column::Vec2(centro));
    }
    // ⭐⭐ **TRAVAR a rotação** (doc 109 §6): a coluna a `0` diz ao solver que esta peça não roda.
    // ⚠️ Destravada NÃO se escreve nada — a ausência quer dizer *«deriva da forma»*, e escrever o
    // valor derivado aqui seria a segunda resposta à mesma pergunta.
    if param(param::LOCK_ROTATION) >= 0.5 {
        out.set(INV_INERTIA_COLUMN, Column::Scalar(vec![0.0; n]));
    }
    out
}

#[cfg(test)]
#[path = "collider_tests.rs"]
mod tests;
