//! O **catálogo de formas** — o enum ÚNICO de toda forma paramétrica do editor, e o
//! `cook` que transforma `(kind, caixa, parâmetros)` em geometria.
//!
//! Antes, cada forma nova custava oito lugares (variante no ECS, variante na tool,
//! variante no modo de desenho, botão no painel, seção de parâmetros, ids de widget,
//! cozimento, e os braços de aplicar/semear/adotar). Com vinte e cinco formas isso
//! seria um pântano. Aqui a forma é **dado**: um `ShapeKind` + um vetor fixo de
//! [`ShapeValues`], e todo o resto do sistema é genérico sobre isso.
//!
//! **Quem sabe o quê:**
//! - *este módulo* sabe DESENHAR (kind + caixa + valores → `VecPath`);
//! - `ph2d-tool-vector::shapes` sabe APRESENTAR (rótulo de cada campo, faixa, passo,
//!   unidade) — os defaults numéricos, porém, moram aqui, para não haver duas verdades;
//! - `ph2d-ecs::VecShape` só GUARDA (o `kind` vira `u16`, primitivo puro).
//!
//! **Regras do catálogo** (o que mantém saves e undo legíveis):
//! 1. `ShapeKind` é **append-only** — o discriminante é gravado no documento.
//! 2. Os valores de uma forma são um array de tamanho fixo ([`MAX_SHAPE_FIELDS`]);
//!    campo faltante lê o default, então uma forma pode GANHAR parâmetros sem
//!    invalidar o que já foi salvo.
//! 3. Os valores são sempre em **unidades de MUNDO** (a geometria é mundo). A
//!    conversão de/para pixels é da UI, na fronteira.

use crate::VecPath;

/// Teto de parâmetros por forma. Fixo (não é `Vec`) para todo o caminho — tool,
/// snapshot do painel, componente ECS — seguir `Copy` e zero-alloc por frame.
pub const MAX_SHAPE_FIELDS: usize = 8;

/// Os parâmetros de uma forma, na ordem que o catálogo declara. Campos não usados
/// pela forma são ignorados; campos ausentes leem o default.
pub type ShapeValues = [f64; MAX_SHAPE_FIELDS];

/// Toda forma paramétrica do editor. **Append-only**: o discriminante é gravado no
/// documento (`VecShape::Param.kind`), então reordenar quebraria saves.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Hash)]
#[repr(u16)]
pub enum ShapeKind {
    #[default]
    Rectangle = 0,
    RoundRect = 1,
    Ellipse = 2,
    Polygon = 3,
    Star = 4,
    Spiral = 5,
    Line = 6,
    Arc = 7,
}

/// Todas as formas, na ordem do enum — a fonte de verdade que a UI itera.
pub const ALL_SHAPES: &[ShapeKind] = &[
    ShapeKind::Rectangle,
    ShapeKind::RoundRect,
    ShapeKind::Ellipse,
    ShapeKind::Polygon,
    ShapeKind::Star,
    ShapeKind::Spiral,
    ShapeKind::Line,
    ShapeKind::Arc,
];

impl ShapeKind {
    /// O discriminante gravado no documento.
    #[must_use]
    pub fn as_u16(self) -> u16 {
        self as u16
    }

    /// A forma de um discriminante gravado. `None` = veio de uma versão mais nova
    /// (forma desconhecida) — o chamador trata como path cru em vez de entrar em pânico.
    #[must_use]
    pub fn from_u16(v: u16) -> Option<Self> {
        ALL_SHAPES.iter().copied().find(|k| k.as_u16() == v)
    }

    /// Formas ABERTAS não têm interior — nunca recebem preenchimento.
    #[must_use]
    pub fn is_closed(self) -> bool {
        !matches!(self, ShapeKind::Line | ShapeKind::Arc | ShapeKind::Spiral)
    }

    /// Os valores default desta forma (o que uma forma nova nasce valendo). É a MESMA
    /// tabela que o catálogo de UI publica como `default` de cada campo — uma verdade só.
    #[must_use]
    pub fn defaults(self) -> ShapeValues {
        let mut v: ShapeValues = [0.0; MAX_SHAPE_FIELDS];
        match self {
            ShapeKind::RoundRect => v[0] = DEFAULT_CORNER_RADIUS,
            ShapeKind::Polygon => {
                v[0] = DEFAULT_POLYGON_SIDES;
                v[1] = 0.0; // canto vivo
            }
            ShapeKind::Star => {
                v[0] = DEFAULT_STAR_POINTS;
                v[1] = DEFAULT_STAR_INNER;
                v[2] = 0.0; // ponta viva
                v[3] = 0.0; // vale vivo
            }
            ShapeKind::Spiral => v[0] = DEFAULT_SPIRAL_TURNS,
            ShapeKind::Arc => v[0] = DEFAULT_ARC_DEGREES,
            ShapeKind::Rectangle | ShapeKind::Ellipse | ShapeKind::Line => {}
        }
        v
    }
}

/// Defaults de geometria (unidades de MUNDO onde couber). O catálogo de UI referencia
/// estes mesmos números — nunca redeclare um default lá.
pub const DEFAULT_CORNER_RADIUS: f64 = 0.12;
pub const DEFAULT_POLYGON_SIDES: f64 = 5.0;
pub const DEFAULT_STAR_POINTS: f64 = 5.0;
pub const DEFAULT_STAR_INNER: f64 = 0.5;
pub const DEFAULT_SPIRAL_TURNS: f64 = 3.0;
pub const DEFAULT_ARC_DEGREES: f64 = 180.0;

/// Lê o campo `i` (0 se não veio) — tolerante a arrays curtos, que é o que permite uma
/// forma ganhar parâmetros sem invalidar o que já está salvo.
fn f(v: &[f64], i: usize) -> f64 {
    v.get(i).copied().unwrap_or(0.0)
}

/// **O cozimento.** `(kind, caixa do gesto, valores em MUNDO) → geometria`, sem estilo.
/// A caixa vai de `a` a `b` (cantos opostos); as formas radiais usam o centro + os
/// semi-eixos dela. É a ÚNICA porta de geometria paramétrica: o `ShapeTool` (preview do
/// arrasto) e o re-cook da forma viva passam os dois por aqui, então o que se desenha e
/// o que se guarda nunca divergem.
#[must_use]
pub fn cook(kind: ShapeKind, a: [f64; 2], b: [f64; 2], v: &[f64]) -> VecPath {
    let c = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let rx = (b[0] - a[0]).abs() * 0.5;
    let ry = (b[1] - a[1]).abs() * 0.5;
    match kind {
        ShapeKind::Rectangle => crate::rectangle(a, b),
        ShapeKind::RoundRect => crate::rounded_rect(a, b, f(v, 0)),
        ShapeKind::Ellipse => crate::ellipse(c, rx, ry),
        ShapeKind::Polygon => {
            crate::regular_polygon_rounded(c, rx, ry, f(v, 0).round().max(3.0) as u32, f(v, 1))
        }
        ShapeKind::Star => crate::star_rounded(
            c,
            rx,
            ry,
            f(v, 0).round().max(3.0) as u32,
            f(v, 1),
            f(v, 2),
            f(v, 3),
        ),
        ShapeKind::Spiral => crate::spiral(c, rx, ry, f(v, 0).round().max(1.0) as u32),
        // A reta guarda a DIREÇÃO (a caixa com sinal), não a bbox.
        ShapeKind::Line => crate::line(a, b),
        ShapeKind::Arc => crate::arc(c, rx, ry, f(v, 0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O discriminante é gravado no documento: reordenar o enum reescreveria o
    /// significado de todo save. Este teste PRENDE os números — um `git diff` que os
    /// mude fica vermelho, e formas novas só podem ser APENDADAS.
    #[test]
    fn the_shape_discriminants_are_frozen_append_only() {
        assert_eq!(ShapeKind::Rectangle.as_u16(), 0);
        assert_eq!(ShapeKind::RoundRect.as_u16(), 1);
        assert_eq!(ShapeKind::Ellipse.as_u16(), 2);
        assert_eq!(ShapeKind::Polygon.as_u16(), 3);
        assert_eq!(ShapeKind::Star.as_u16(), 4);
        assert_eq!(ShapeKind::Spiral.as_u16(), 5);
        assert_eq!(ShapeKind::Line.as_u16(), 6);
        assert_eq!(ShapeKind::Arc.as_u16(), 7);
        // E o ida-e-volta fecha para TODA forma do catálogo.
        for &k in ALL_SHAPES {
            assert_eq!(ShapeKind::from_u16(k.as_u16()), Some(k));
        }
        assert_eq!(
            ShapeKind::from_u16(9_999),
            None,
            "forma desconhecida != panic"
        );
    }

    /// Toda forma do catálogo COZINHA (nenhuma fica sem braço no `cook`) e produz
    /// geometria dentro da caixa do gesto. É o gate anti-forma-morta: um `ShapeKind`
    /// novo sem cozimento seria selecionável no painel e desenharia o vazio.
    #[test]
    fn every_shape_in_the_catalog_cooks_inside_its_box() {
        let (a, b) = ([-2.0, -1.0], [2.0, 1.0]);
        for &k in ALL_SHAPES {
            let p = cook(k, a, b, &k.defaults());
            assert!(!p.verts.is_empty(), "{k:?} cozinhou vazio");
            for vtx in p.verts_all() {
                assert!(
                    vtx.anchor[0] >= a[0] - 1e-6
                        && vtx.anchor[0] <= b[0] + 1e-6
                        && vtx.anchor[1] >= a[1] - 1e-6
                        && vtx.anchor[1] <= b[1] + 1e-6,
                    "{k:?}: âncora {:?} fora da caixa do gesto",
                    vtx.anchor
                );
            }
            assert_eq!(p.closed, k.is_closed(), "{k:?}: fechamento inconsistente");
        }
    }

    /// Um array de valores CURTO (um save antigo, feito antes de a forma ganhar um
    /// parâmetro) cozinha com os campos que faltam em zero — nunca entra em pânico.
    #[test]
    fn a_short_value_array_cooks_instead_of_panicking() {
        let p = cook(ShapeKind::Star, [-1.0, -1.0], [1.0, 1.0], &[5.0]);
        assert!(!p.verts.is_empty());
    }
}
