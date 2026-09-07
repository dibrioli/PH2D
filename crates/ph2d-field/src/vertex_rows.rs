//! ⭐⭐⭐ **QUE FORMAS TÊM VÉRTICES, E ONDE ELES ESTÃO NA LISTA DE LINHAS** (W133) — a porta que o
//! canvas usa para desenhar uma alça por vértice.
//!
//! # Por que a pergunta é feita AQUI, e não no shell
//!
//! Enio, 2026-09-07: *«para esse tipo de objeto e todos os outros que dependem de posição de vertex,
//! os vertex devem aparecer no canvas em tempo real»*. ⚠️ **«E todos os outros» é o pedido inteiro**:
//! uma lista escrita à mão no shell responderia hoje e ficaria para trás na primeira forma nova —
//! que é exactamente o defeito que a W103 pagou com quatro constantes `SHAPES.len() − N`.
//!
//! Aqui a lista é um `match` **exaustivo** sobre a primitiva: uma forma nova com vértices é **erro
//! de compilação** neste arquivo, e quem a escrever tem de dizer onde os pontos dela vivem.
//!
//! ⭐ **E há um censo do outro lado** (`the_vertex_door_knows_every_shape_with_free_rows`): toda
//! forma cuja tabela de linhas declara um PAR de [`Span::Free`] tem de ser conhecida por esta porta.
//! *O `match` impede o esquecimento ao compilar; o censo impede a resposta errada.*
//!
//! # ⚠️ O índice é da LISTA DE LINHAS, e não do vector de pontos
//!
//! O painel escreve por índice ([`crate::Param::Dim`]), então é isso que o canvas precisa de
//! devolver — e é isso que faz o arrasto de uma alça atravessar **a mesma porta** que arrastar a
//! linha do painel: a validação, a coerção e o undo são os mesmos, sem uma segunda lei.

use crate::Primitive;

/// **Os vértices de uma forma, e onde a primeira coordenada deles está na lista de linhas.**
#[derive(Clone, Debug, PartialEq)]
pub struct VertexRows {
    /// O índice, em [`crate::dims`], do `x` do **primeiro** vértice. O `y` dele é o seguinte, e cada
    /// vértice ocupa **duas** linhas consecutivas.
    pub first_row: usize,
    /// Os pontos, em coordenadas **locais** do nó — os mesmos números que o painel mostra.
    pub points: Vec<[f32; 2]>,
}

/// **Esta forma tem vértices autorados?** — e, se tem, quais e onde.
///
/// `None` para toda forma cujo contorno é decidido por uma fórmula (o prisma, a estrela, o coração):
/// ali não há ponto nenhum que o artista possa agarrar, e uma alça sobre um número derivado seria um
/// controlo que promete o que o modelo não tem.
///
/// ⚠️ **O `Extrude` e o `Revolve` também devolvem `None`, e não é esquecimento:** os pontos deles são
/// do **editor vetorial**, e o vínculo re-coze o perfil a cada quadro — uma alça que os arrastasse
/// seria escrita e apagada no quadro seguinte. *É a mesma razão que fez o polígono nascer primitiva
/// própria na W132.*
#[must_use]
pub fn vertex_rows(p: &Primitive) -> Option<VertexRows> {
    match p {
        // ─────────────────────────── W131 ───────────────────────────
        // As seis coordenadas são as primeiras linhas da tabela dele, nesta ordem.
        Primitive::Triangle { a, b, c, .. } => Some(VertexRows {
            first_row: 0,
            points: vec![*a, *b, *c],
        }),
        // ─────────────────────────── W132 ───────────────────────────
        // ⚠️ **A contagem é a linha `0`**, e por isso o primeiro `x` está na `1` — ver
        // [`crate::dims_table_polygon::ROWS_BEFORE_VERTICES`], que é de onde este número sai.
        Primitive::Polygon { profile, .. } => Some(VertexRows {
            first_row: crate::dims::ROWS_BEFORE_VERTICES,
            points: profile.contours().first().cloned().unwrap_or_default(),
        }),
        // ⚠️ **Lista FECHADA**: uma primitiva nova é erro de compilação aqui, e quem a escrever tem
        // de dizer se ela tem pontos que o artista digita. ⛔ Um `_ => None` faria a forma seguinte
        // nascer sem alças no canvas **sem um aviso**, que é a família de defeitos que este módulo
        // já pagou três vezes.
        Primitive::Box { .. }
        | Primitive::Sphere { .. }
        | Primitive::Cylinder { .. }
        | Primitive::Torus { .. }
        | Primitive::Extrude { .. }
        | Primitive::Revolve { .. }
        | Primitive::Cone { .. }
        | Primitive::Capsule { .. }
        | Primitive::Prism { .. }
        | Primitive::Wedge { .. }
        | Primitive::TorusArc { .. }
        | Primitive::Star { .. }
        | Primitive::BoxFrame { .. }
        | Primitive::Ellipsoid { .. }
        | Primitive::Octahedron { .. }
        | Primitive::RoundCone { .. }
        | Primitive::CutSphere { .. }
        | Primitive::HollowDome { .. }
        | Primitive::Link { .. }
        | Primitive::SolidAngle { .. }
        | Primitive::Gear { .. }
        | Primitive::Cross { .. }
        | Primitive::Heart { .. }
        | Primitive::Moon { .. }
        | Primitive::Drop { .. }
        | Primitive::Pie { .. }
        | Primitive::Trapezoid { .. }
        | Primitive::Vesica { .. }
        | Primitive::Arrow { .. }
        | Primitive::Chevron { .. }
        | Primitive::BentArrow { .. }
        | Primitive::Rhombus { .. }
        | Primitive::Tube { .. }
        | Primitive::CircleSegment { .. }
        | Primitive::SpeechRect { .. }
        | Primitive::SpeechOval { .. }
        | Primitive::Cloud { .. }
        | Primitive::Bolt { .. }
        | Primitive::Shield { .. }
        | Primitive::Tag { .. }
        | Primitive::Check { .. }
        | Primitive::Banner { .. }
        | Primitive::Brace { .. }
        | Primitive::Parallelogram { .. }
        | Primitive::Delay { .. }
        | Primitive::Display { .. }
        | Primitive::OffPage { .. }
        | Primitive::Spiral { .. }
        | Primitive::Document { .. }
        | Primitive::Helix { .. }
        | Primitive::Gyroid { .. }
        | Primitive::RoundedCylinder { .. }
        | Primitive::Superquadric { .. }
        | Primitive::Superformula { .. }
        | Primitive::TorusKnot { .. } => None,
    }
}
