//! ⭐⭐ **O COLISOR QUE A FORMA DECLARA** — doc 109, ordem do dono (2026-09-13): *«colidem
//! sozinhas»*.
//!
//! ⚠️ **Só quem desenha sabe o tamanho do que desenha.** Com o mesmo `size = 1` um `Circle` desta
//! forma desenha raio `1` e uma sprite desenha raio inscrito `0,5`; nenhum consumidor a jusante
//! sabe que mídia recebeu. Por isso a declaração nasce AQUI e viaja como coluna
//! ([`ph2d_nodegraph::attr::COLLIDER_COLUMN`]).
//!
//! ⚠️ **O raio sai da GEOMETRIA, e a geometria é do shell** — este nó não alcança a biblioteca
//! vectorial, por desenho. O shell mede os dois raios uma vez por geometria e publica-os com ela
//! ([`param::COLLIDER_AROUND_COL`] · [`param::COLLIDER_INSIDE_COL`]); o nó ESCOLHE pelos params
//! dele e **retira sempre** as duas colunas. Elas não podem ser a declaração: a chave de conteúdo
//! é partilhada por toda forma com a mesma geometria, e duas formas iguais podem declarar coisas
//! diferentes.

use super::param;
use ph2d_nodegraph::attr::{COLLIDER_COLUMN, Column, Stream};

/// Os rótulos do `Collider Fit`. ⚠️ O índice é formato de arquivo — APPEND ONLY.
pub(crate) static FIT_LABELS: &[&str] = &["Around", "Inside"];

/// O índice de `Inside` em [`FIT_LABELS`].
const FIT_INSIDE: i32 = 1;

/// O stream publicado → o stream que o nó emite: as colunas de raio **retiradas**, e o
/// `collider` escrito só com o `Collide` ligado.
///
/// ⚠️ **Desligado, nenhuma coluna nova** — a lei estrutural do `fill`/`rotation` deste nó: o
/// default não é *«escrever o mesmo valor»*, é *«não escrever»*.
///
/// ⚠️ Ligado mas sem os raios publicados (um cozimento adiantado, antes do `publish` do shell)
/// ⇒ também nada: um colisor inventado seria pior que nenhum.
pub(crate) fn declare(published: Stream, param: impl Fn(&str) -> f32) -> Stream {
    let n = published.count();
    let raio = |name: &str| match published.get(name) {
        Some(Column::Scalar(v)) if v.len() == n => Some(v.clone()),
        _ => None,
    };
    let (around, inside) = (
        raio(param::COLLIDER_AROUND_COL),
        raio(param::COLLIDER_INSIDE_COL),
    );
    let mut out = if around.is_none() && inside.is_none() {
        published
    } else {
        let mut s = Stream::new(n);
        for (name, col) in published.columns() {
            if name != param::COLLIDER_AROUND_COL && name != param::COLLIDER_INSIDE_COL {
                s.set(name.clone(), col.clone());
            }
        }
        s
    };
    if n == 0 || param(param::COLLIDE) < 0.5 {
        return out;
    }
    let escolhido = if param(param::COLLIDER_FIT).round() as i32 == FIT_INSIDE {
        inside
    } else {
        around
    };
    if let Some(r) = escolhido {
        // Um multiplicador negativo ou não-finito não é um pedido: lê como a identidade.
        let k = param(param::COLLIDER_SCALE);
        let k = if k.is_finite() { k.max(0.0) } else { 1.0 };
        out.set(
            COLLIDER_COLUMN,
            Column::Scalar(r.iter().map(|x| x * k).collect()),
        );
    }
    out
}

#[cfg(test)]
#[path = "collider_tests.rs"]
mod tests;
