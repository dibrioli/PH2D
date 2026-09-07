//! ⭐⭐ **AS LINHAS DO POLÍGONO** (W132) — a contagem, os `2N` números dos vértices, e o rodapé de
//! sempre (altura, chanfro, filete).
//!
//! # Por que um arquivo irmão
//!
//! Ele é a única forma desta casa cujo **número de linhas depende de uma linha**, e a tabela de
//! chaves que isso exige tem tamanho fixo por construção. O [`super::dims_table`] responde por
//! *que números cada forma tem*; separar esta mantém a tabela de chaves ao lado da lei que a lê.
//!
//! # ⚠️ A ordem É a identidade, e a contagem vem PRIMEIRO
//!
//! O painel escreve por **índice** ([`super::Param::Dim`]). A linha da contagem está no índice `0`,
//! que é o único que **não se mexe** quando a contagem muda — as que deslizam são as de baixo, e
//! ninguém as está a arrastar enquanto arrasta a de cima. *Se a contagem fosse a última, subir um
//! vértice renumeraria a linha debaixo do dedo.*

use super::{Dim, Span};
use crate::polygon::{MAX_POLYGON_VERTICES, MIN_POLYGON_VERTICES};
use crate::{Primitive, round_limit};

/// ⭐ **As chaves de rótulo dos vértices, uma por número.**
///
/// ⚠️ **Estáticas e escritas por extenso, e não montadas em tempo de execução.** O `ph2d_i18n::tr`
/// recebe `&str` e devolve `&'static str`; uma chave montada (`format!("field.dim.v{i}x")`) não teria
/// onde viver, e a rota de chave desconhecida **vaza uma string por quadro** ao pintar o
/// identificador cru. *A tabela é o preço de o rótulo ser uma constante.*
///
/// ⚠️ O comprimento é [`MAX_POLYGON_VERTICES`] por construção — subir o teto sem estender esta
/// tabela é **erro de compilação**, que é o que impede uma linha de nascer sem nome.
pub(crate) const VERTEX_KEYS: [[&str; 2]; MAX_POLYGON_VERTICES as usize] = [
    ["field.dim.v1x", "field.dim.v1y"],
    ["field.dim.v2x", "field.dim.v2y"],
    ["field.dim.v3x", "field.dim.v3y"],
    ["field.dim.v4x", "field.dim.v4y"],
    ["field.dim.v5x", "field.dim.v5y"],
    ["field.dim.v6x", "field.dim.v6y"],
    ["field.dim.v7x", "field.dim.v7y"],
    ["field.dim.v8x", "field.dim.v8y"],
    ["field.dim.v9x", "field.dim.v9y"],
    ["field.dim.v10x", "field.dim.v10y"],
    ["field.dim.v11x", "field.dim.v11y"],
    ["field.dim.v12x", "field.dim.v12y"],
    ["field.dim.v13x", "field.dim.v13y"],
    ["field.dim.v14x", "field.dim.v14y"],
    ["field.dim.v15x", "field.dim.v15y"],
    ["field.dim.v16x", "field.dim.v16y"],
    ["field.dim.v17x", "field.dim.v17y"],
    ["field.dim.v18x", "field.dim.v18y"],
    ["field.dim.v19x", "field.dim.v19y"],
    ["field.dim.v20x", "field.dim.v20y"],
    ["field.dim.v21x", "field.dim.v21y"],
    ["field.dim.v22x", "field.dim.v22y"],
    ["field.dim.v23x", "field.dim.v23y"],
    ["field.dim.v24x", "field.dim.v24y"],
    ["field.dim.v25x", "field.dim.v25y"],
    ["field.dim.v26x", "field.dim.v26y"],
    ["field.dim.v27x", "field.dim.v27y"],
];

/// Quantas linhas vêm ANTES do primeiro vértice — a contagem, e só ela.
pub(crate) const ROWS_BEFORE_VERTICES: usize = 1;

/// As linhas de um polígono.
///
/// ⚠️ **Inalcançável com outra primitiva** — quem chega aqui já foi nomeado pelo braço do
/// [`super::dims_table::dims`], que continua exaustivo.
#[must_use]
pub(crate) fn dims_polygon(p: &Primitive) -> Vec<Dim> {
    let Primitive::Polygon {
        profile,
        half_height,
        round,
        chamfer,
    } = p
    else {
        return Vec::new();
    };
    let pontos: &[[f32; 2]] = profile.contours().first().map_or(&[], Vec::as_slice);
    let n = pontos.len().min(MAX_POLYGON_VERTICES as usize);
    let mut linhas = Vec::with_capacity(ROWS_BEFORE_VERTICES + 2 * n + 3);
    #[allow(clippy::cast_precision_loss)]
    linhas.push(Dim {
        key: "field.dim.vertices",
        value: pontos.len() as f32,
        span: Span::Count {
            min: MIN_POLYGON_VERTICES,
            max: MAX_POLYGON_VERTICES,
        },
    });
    for (i, v) in pontos.iter().take(n).enumerate() {
        // ⚠️ **`Free` nos dois eixos** — um vértice é uma POSIÇÃO, e o piso dela é negativo. Foi
        // exactamente esta a linha que o triângulo pagou na W131.
        linhas.push(Dim {
            key: VERTEX_KEYS[i][0],
            value: v[0],
            span: Span::Free,
        });
        linhas.push(Dim {
            key: VERTEX_KEYS[i][1],
            value: v[1],
            span: Span::Free,
        });
    }
    linhas.push(Dim {
        key: "field.dim.height",
        value: half_height * 2.0,
        span: Span::Positive,
    });
    // ⚠️ **A mesma parede do [`Primitive::Extrude`], e ela é a meia-altura E SÓ ELA** — porque este
    // filete é do **ARO** (a aresta entre a parede e a tampa), e não das quinas do contorno. ⛔ É
    // por isso que o teto NÃO é o inraio, ao contrário do [`Primitive::Triangle`]: lá o filete come
    // o interior da chapa e passar do inraio parte o campo; aqui a largura do contorno não entra na
    // conta de todo. Medição e tabela: `ph2d_field_eval::profile::sd_extrude`.
    let parede = round_limit(p).map_or(Span::FromZero, Span::WallFromZero);
    linhas.push(Dim {
        key: "field.dim.chamfer",
        value: *chamfer,
        span: parede,
    });
    linhas.push(Dim {
        key: "field.dim.round",
        value: *round,
        span: parede,
    });
    linhas
}
