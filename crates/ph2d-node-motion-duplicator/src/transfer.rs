//! **A TRANSFERÊNCIA DE ATRIBUTO** — o que acontece às colunas que vivem nos PONTOS.
//!
//! Doc 89, folha 08, célula do `motion.duplicator`: *"modos de transferência de atributo
//! (Copy/Mult/Add/Sub por padrão de nomes) — Houdini **Copy to Points**"*.
//!
//! ⚠️ **A célula estava escrita como omissão de knob, e a sonda mediu um DEFEITO por baixo**
//! (`measure_clone_multisource::which_point_columns_reach_the_output_at_all`, 2026-08-26): o
//! carimbo replica `shape.columns()` e soma `P`/`rot`; **toda coluna que exista só nos pontos
//! desaparece sem aviso**. Uma rampa de cor sobre o arranjo — o gesto mais comum que há — é
//! autorada nos PONTOS, e a saída não a tem:
//!
//! ```text
//! nos pontos:  Count · Index · P · tint
//! na saida:    Count · Index · P          ⚠️ tint sumiu
//! ```
//!
//! ⚠️ **O default continua a ser o de hoje** ([`Transfer::ShapeWins`]), e isso é a escolha
//! conservadora, não a certa: a referência transfere. É a mesma forma do `reindex` do
//! `motion.combine` — o param entra desligado e a pergunta vai com o smoke, porque ligá-lo
//! por omissão mudaria arte já autorada, o que nesta casa é decisão do Enio.
//!
//! ⚠️ **O `size` fica de FORA desta lei, e tem porta própria** (`point_scale`): ele é o único
//! atributo cuja composição já foi decidida e medida, e uma segunda lei a escrevê-lo seria a
//! forma clássica de as duas divergirem. *Uma grandeza, uma porta.*
//!
//! ⚠️ **E quando as duas colunas têm VARIANTES diferentes, a forma vence e nada muda.** Somar
//! um `Scalar` a um `Vec2` não tem resposta, e escolher a do ponto mudaria o tipo da coluna a
//! jusante em silêncio — o que é pior que a perda que este módulo veio curar, porque um tipo
//! trocado passa pelo cook e reaparece noutro nó.

use ph2d_nodegraph::attr::{Column, Stream};

/// O nome do param no manifesto.
pub const TRANSFER: &str = "transfer";

/// **De quem é o valor quando a coluna existe dos dois lados** — e se a coluna que só o
/// PONTO tem chega à saída.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Transfer {
    /// A forma vence, e a coluna do ponto é deitada fora. **O que sempre aconteceu.**
    ShapeWins,
    /// O ponto vence; uma coluna só-do-ponto passa a chegar.
    PointWins,
    /// Soma componente a componente; uma coluna só-do-ponto chega como está.
    Add,
    /// Produto componente a componente; uma coluna só-do-ponto chega como está.
    Multiply,
}

impl Transfer {
    /// Do param. Fora de alcance cai em [`Self::ShapeWins`] — o comportamento de sempre é a
    /// resposta honesta a um modo que ninguém pediu (a mesma lei do `Pick::of`).
    #[must_use]
    pub fn of(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::PointWins,
            2 => Self::Add,
            3 => Self::Multiply,
            _ => Self::ShapeWins,
        }
    }

    /// `true` quando este modo não toca em nada — o caminho de sempre, byte a byte.
    #[must_use]
    pub fn is_inert(self) -> bool {
        self == Self::ShapeWins
    }
}

/// Colhe `col[pi]` para cada par — o irmão do `spread`, do lado do PONTO.
fn gather(col: &Column, pairs: &[(usize, usize)]) -> Column {
    fn go<T: Copy + Default>(v: &[T], pairs: &[(usize, usize)]) -> Vec<T> {
        pairs
            .iter()
            .map(|&(_, pi)| v.get(pi).copied().unwrap_or_default())
            .collect()
    }
    match col {
        Column::Scalar(v) => Column::Scalar(go(v, pairs)),
        Column::Vec2(v) => Column::Vec2(go(v, pairs)),
        Column::Vec3(v) => Column::Vec3(go(v, pairs)),
        Column::Vec4(v) => Column::Vec4(go(v, pairs)),
    }
}

/// Combina duas colunas JÁ alinhadas (uma linha por par). `None` quando as variantes
/// discordam — o chamador mantém a da forma.
fn merge(shape: &Column, point: &Column, mode: Transfer) -> Option<Column> {
    macro_rules! zip {
        ($v:path, $a:expr, $b:expr, $n:expr) => {{
            let out = $a
                .iter()
                .zip($b.iter())
                .map(|(s, p)| {
                    let mut r = *s;
                    for k in 0..$n {
                        r[k] = match mode {
                            Transfer::Add => s[k] + p[k],
                            Transfer::Multiply => s[k] * p[k],
                            _ => p[k],
                        };
                    }
                    r
                })
                .collect();
            Some($v(out))
        }};
    }
    match (shape, point) {
        (Column::Scalar(a), Column::Scalar(b)) => {
            let out = a
                .iter()
                .zip(b.iter())
                .map(|(s, p)| match mode {
                    Transfer::Add => s + p,
                    Transfer::Multiply => s * p,
                    _ => *p,
                })
                .collect();
            Some(Column::Scalar(out))
        }
        (Column::Vec2(a), Column::Vec2(b)) => zip!(Column::Vec2, a, b, 2),
        (Column::Vec3(a), Column::Vec3(b)) => zip!(Column::Vec3, a, b, 3),
        (Column::Vec4(a), Column::Vec4(b)) => zip!(Column::Vec4, a, b, 4),
        _ => None,
    }
}

/// Escreve em `out` as colunas dos PONTOS, segundo `mode`. `reserved` são os nomes que já
/// têm lei própria (`P`/`rot` somam, `Index`/`Count` renumeram, `size` tem o `point_scale`).
///
/// No modo inerte não corre nada — nem o laço —, e é isso que mantém o caminho de sempre
/// byte a byte.
pub fn point_columns_into(
    out: &mut Stream,
    shape: &Stream,
    points: &Stream,
    pairs: &[(usize, usize)],
    mode: Transfer,
    reserved: &[&str],
) {
    if mode.is_inert() {
        return;
    }
    for (name, col) in points.columns() {
        if reserved.contains(&name.as_str()) {
            continue;
        }
        let from_point = gather(col, pairs);
        let value = match shape.get(name.as_str()) {
            // Só o ponto a tem: ela passa a chegar (é a metade que era um defeito).
            None => from_point,
            // Os dois lados a têm: o modo decide, e variantes discordantes mantêm a forma.
            Some(s) => {
                let spread = super::spread(s, pairs);
                match merge(&spread, &from_point, mode) {
                    Some(v) => v,
                    None => continue,
                }
            }
        };
        out.set(name.clone(), value);
    }
}

#[cfg(test)]
#[path = "transfer_tests.rs"]
mod transfer_tests;
