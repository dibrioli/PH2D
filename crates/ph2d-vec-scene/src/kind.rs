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

use crate::{FULL_TURN, VecPath};

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
    /// Pizza / rosquinha / anel parcial — a mesma forma da elipse, com `sweep` e
    /// `inner` já abertos por default (o atalho para quem quer a fatia direto).
    Pie = 8,
    /// Segmento circular: o arco fechado pela CORDA (não pelo centro).
    Segment = 9,
    // ── Setas (formas preenchíveis; a ponta de TRAÇO é outra coisa) ──────────
    ArrowRight = 10,
    ArrowDouble = 11,
    ArrowBent = 12,
    Chevron = 13,
    ArrowCurved = 14,
    // ── Fluxograma (ANSI/ISO 5807) ───────────────────────────────────────────
    Diamond = 15,
    Pill = 16,
    Parallelogram = 17,
    Trapezoid = 18,
    TrapezoidFlip = 19,
    HexagonFlat = 20,
    Cylinder = 21,
    Document = 22,
    Delay = 23,
    Display = 24,
    PredefinedProcess = 25,
    OffPage = 26,
    Junction = 27,
    NoteBracket = 28,
    // ── Balões ───────────────────────────────────────────────────────────────
    SpeechRect = 29,
    SpeechOval = 30,
    Thought = 31,
    Burst = 32,
    Brace = 33,
    // ── Símbolos ─────────────────────────────────────────────────────────────
    Heart = 34,
    Cloud = 35,
    Bolt = 36,
    Moon = 37,
    Drop = 38,
    Shield = 39,
    Tag = 40,
    Gear = 41,
    Cross = 42,
    Check = 43,
    Banner = 44,
    // ── Isométricas (2.5D) ───────────────────────────────────────────────────
    IsoCube = 45,
    IsoCone = 46,
    IsoPyramid = 47,
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
    ShapeKind::Pie,
    ShapeKind::Segment,
    ShapeKind::ArrowRight,
    ShapeKind::ArrowDouble,
    ShapeKind::ArrowBent,
    ShapeKind::Chevron,
    ShapeKind::ArrowCurved,
    ShapeKind::Diamond,
    ShapeKind::Pill,
    ShapeKind::Parallelogram,
    ShapeKind::Trapezoid,
    ShapeKind::TrapezoidFlip,
    ShapeKind::HexagonFlat,
    ShapeKind::Cylinder,
    ShapeKind::Document,
    ShapeKind::Delay,
    ShapeKind::Display,
    ShapeKind::PredefinedProcess,
    ShapeKind::OffPage,
    ShapeKind::Junction,
    ShapeKind::NoteBracket,
    ShapeKind::SpeechRect,
    ShapeKind::SpeechOval,
    ShapeKind::Thought,
    ShapeKind::Burst,
    ShapeKind::Brace,
    ShapeKind::Heart,
    ShapeKind::Cloud,
    ShapeKind::Bolt,
    ShapeKind::Moon,
    ShapeKind::Drop,
    ShapeKind::Shield,
    ShapeKind::Tag,
    ShapeKind::Gear,
    ShapeKind::Cross,
    ShapeKind::Check,
    ShapeKind::Banner,
    ShapeKind::IsoCube,
    ShapeKind::IsoCone,
    ShapeKind::IsoPyramid,
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
        !matches!(
            self,
            ShapeKind::Line
                | ShapeKind::Arc
                | ShapeKind::Spiral
                | ShapeKind::NoteBracket
                | ShapeKind::Brace
        )
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
            // A elipse nasce INTEIRA e sem furo; abrir `sweep`/`inner` a leva a pizza,
            // rosquinha ou anel parcial sem trocar de forma.
            ShapeKind::Ellipse => {
                v[0] = FULL_TURN;
                v[1] = 0.0;
                v[2] = 0.0;
            }
            // A pizza é a mesma forma, já aberta (o atalho do catálogo).
            ShapeKind::Pie => {
                v[0] = DEFAULT_ARC_DEGREES;
                v[1] = 0.0;
                v[2] = 0.0;
            }
            ShapeKind::Arc | ShapeKind::Segment => {
                v[0] = DEFAULT_ARC_DEGREES;
                v[1] = 0.0;
            }
            // Setas: as medidas são FRAÇÕES da caixa (a seta guarda a proporção ao
            // redimensionar, que é o que se espera de uma forma paramétrica).
            ShapeKind::ArrowRight | ShapeKind::ArrowDouble => {
                v[0] = DEFAULT_ARROW_TAIL;
                v[1] = DEFAULT_ARROW_HEAD_LEN;
                v[2] = DEFAULT_ARROW_HEAD_W;
            }
            ShapeKind::ArrowBent => {
                v[0] = 0.25; // espessura do corpo (fração do lado curto)
                v[1] = 0.3; // comprimento da cabeça
                v[2] = 0.6; // largura cheia da cabeça
                v[3] = 0.35; // raio EXTERNO do cotovelo (0 = quina quadrada)
            }
            ShapeKind::Chevron => {
                v[0] = 0.3; // profundidade da ponta
                v[1] = 0.2; // entalhe
            }
            // A seta circular é a `circularArrow` do ECMA-376 em forma fechada. O sweep é
            // COM SINAL (negativo = canhota), e a largura da cabeça governa o raio da
            // faixa — é o transbordo dela que a caixa precisa acomodar.
            ShapeKind::ArrowCurved => {
                v[0] = 270.0; // sweep total, cauda->ponta (com sinal)
                v[1] = 180.0; // onde a cauda começa (0 = 3 horas)
                v[2] = 0.125; // espessura radial da faixa
                v[3] = 0.25; // largura cheia da cabeça
                v[4] = 0.12; // quanto do sweep a cabeça consome
            }
            // Fluxograma: as medidas são frações da caixa.
            ShapeKind::Parallelogram => v[0] = 0.2,
            ShapeKind::Trapezoid | ShapeKind::TrapezoidFlip => v[0] = 0.2,
            ShapeKind::HexagonFlat => v[0] = 0.2,
            ShapeKind::Cylinder => v[0] = 0.2,
            ShapeKind::Document => v[0] = 0.15,
            ShapeKind::Display => v[0] = 0.2,
            ShapeKind::PredefinedProcess => v[0] = 0.12,
            ShapeKind::OffPage => v[0] = 0.3,
            ShapeKind::NoteBracket => v[0] = 0.15,
            // Balões: o BICO é um ponto 2D livre (fração da caixa) — é o que os apps de
            // verdade fazem, e o que o `wedgeRectCallout` do OOXML não tem (lá a base salta
            // numa grade de 1/12). `room` é a faixa que o corpo cede ao rabo: ele vive
            // DENTRO dela, então não tem como vazar a caixa.
            ShapeKind::SpeechRect => {
                v[0] = DEFAULT_CORNER_RADIUS;
                v[1] = 0.30; // bico: u
                v[2] = 0.97; // bico: v (embaixo — o rabo aponta para quem fala)
                v[3] = 0.26; // largura da base
                v[4] = 0.30; // faixa reservada ao rabo
            }
            ShapeKind::SpeechOval => {
                v[0] = 0.30;
                v[1] = 0.97;
                v[2] = 13.0; // meia-abertura da base na elipse (o spec usa 11°)
                v[3] = 0.26;
            }
            ShapeKind::Thought => {
                v[0] = 0.28;
                v[1] = 0.95;
                v[2] = 3.0; // bolhas da corrente (o `cloudCallout` usa 3)
                v[3] = 11.0; // bolhas da nuvem (o spec usa 11 arcos)
            }
            ShapeKind::Burst => {
                v[0] = 12.0; // pontas (o `irregularSeal1` tem 12)
                v[1] = 0.565; // razão interna, medida do spec
                v[2] = 0.28;
                v[3] = 0.55;
            }
            ShapeKind::Brace => {
                v[0] = 1.0 / 12.0; // o `adj1 = 8333` do spec
                v[1] = 0.5;
                v[2] = 0.5;
            }
            // Símbolos.
            ShapeKind::Heart => v[0] = 0.2,
            ShapeKind::Cloud => {
                v[0] = 11.0;
                v[1] = 0.62; // raio da bolha ÷ passo da espinha (≤ 0,5 = sem sobreposição)
                v[2] = 0.0; // vale: 0 = cúspide (OOXML), > 0 = contorno G1
                v[3] = 1.0; // irregularidade (a tabela medida do spec, em escala cheia)
            }
            ShapeKind::Bolt => v[0] = 0.4,
            ShapeKind::Moon => v[0] = 0.45,
            ShapeKind::Drop => v[0] = 0.5,
            ShapeKind::Shield => v[0] = 0.6,
            ShapeKind::Tag => {
                v[0] = 0.25;
                v[1] = 0.18;
            }
            ShapeKind::Gear => {
                v[0] = 8.0;
                v[1] = 0.2;
                v[2] = 0.35;
            }
            ShapeKind::Cross => v[0] = 0.35,
            ShapeKind::Check => v[0] = 0.25,
            ShapeKind::Banner => v[0] = 0.15,
            // `rise` = a altura do vértice interno (inclina os eixos da projeção) e
            // `skew` = a repartição largura/profundidade. Em 0,5/0,5 numa caixa quadrada
            // sai o dimétrico 2:1 clássico.
            ShapeKind::IsoCube | ShapeKind::IsoPyramid => {
                v[0] = 0.5;
                v[1] = 0.5;
            }
            ShapeKind::IsoCone => v[0] = 0.2,
            ShapeKind::Rectangle
            | ShapeKind::Line
            | ShapeKind::Diamond
            | ShapeKind::Pill
            | ShapeKind::Delay
            | ShapeKind::Junction => {}
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
/// Setas: espessura da haste, comprimento e largura da cabeça (frações da caixa).
pub const DEFAULT_ARROW_TAIL: f64 = 0.4;
pub const DEFAULT_ARROW_HEAD_LEN: f64 = 0.4;
pub const DEFAULT_ARROW_HEAD_W: f64 = 1.0;

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
        // A família do círculo: `sweep` + `start` + `inner` dão elipse, pizza, rosquinha
        // e anel parcial (crate::round). Pie é a mesma forma com outros defaults.
        ShapeKind::Ellipse | ShapeKind::Pie => {
            crate::ellipse_sweep(c, rx, ry, f(v, 1), f(v, 0), f(v, 2))
        }
        ShapeKind::Segment => crate::ellipse_chord(c, rx, ry, f(v, 1), f(v, 0)),
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
        ShapeKind::Arc => crate::arc_open(c, rx, ry, f(v, 1), f(v, 0)),
        ShapeKind::ArrowRight => crate::arrow_block(a, b, f(v, 0), f(v, 1), f(v, 2)),
        ShapeKind::ArrowDouble => crate::arrow_double(a, b, f(v, 0), f(v, 1), f(v, 2)),
        ShapeKind::ArrowBent => crate::arrow_bent(a, b, f(v, 0), f(v, 1), f(v, 2), f(v, 3)),
        ShapeKind::Chevron => crate::chevron(a, b, f(v, 0), f(v, 1)),
        ShapeKind::ArrowCurved => {
            crate::arrow_curved(a, b, f(v, 0), f(v, 1), f(v, 2), f(v, 3), f(v, 4))
        }
        ShapeKind::Diamond => crate::diamond(a, b),
        ShapeKind::Pill => crate::pill(a, b),
        ShapeKind::Parallelogram => crate::parallelogram(a, b, f(v, 0)),
        ShapeKind::Trapezoid => crate::trapezoid(a, b, f(v, 0), false),
        ShapeKind::TrapezoidFlip => crate::trapezoid(a, b, f(v, 0), true),
        ShapeKind::HexagonFlat => crate::hexagon_flat(a, b, f(v, 0)),
        ShapeKind::Cylinder => crate::cylinder(a, b, f(v, 0)),
        ShapeKind::Document => crate::document(a, b, f(v, 0)),
        ShapeKind::Delay => crate::delay(a, b),
        ShapeKind::Display => crate::display(a, b, f(v, 0)),
        ShapeKind::PredefinedProcess => crate::predefined_process(a, b, f(v, 0)),
        ShapeKind::OffPage => crate::offpage(a, b, f(v, 0)),
        ShapeKind::Junction => crate::junction(a, b),
        ShapeKind::NoteBracket => crate::note_bracket(a, b, f(v, 0)),
        ShapeKind::SpeechRect => {
            crate::speech_rect(a, b, f(v, 0), f(v, 1), f(v, 2), f(v, 3), f(v, 4))
        }
        ShapeKind::SpeechOval => crate::speech_oval(a, b, f(v, 0), f(v, 1), f(v, 2), f(v, 3)),
        ShapeKind::Thought => crate::thought(a, b, f(v, 0), f(v, 1), f(v, 2), f(v, 3)),
        ShapeKind::Burst => crate::burst(a, b, f(v, 0), f(v, 1), f(v, 2), f(v, 3)),
        ShapeKind::Brace => crate::brace(a, b, f(v, 0), f(v, 1), f(v, 2)),
        ShapeKind::Heart => crate::heart(a, b, f(v, 0)),
        ShapeKind::Cloud => crate::cloud(a, b, f(v, 0), f(v, 1), f(v, 2), f(v, 3)),
        ShapeKind::Bolt => crate::bolt(a, b, f(v, 0)),
        ShapeKind::Moon => crate::moon(a, b, f(v, 0)),
        ShapeKind::Drop => crate::drop(a, b, f(v, 0)),
        ShapeKind::Shield => crate::shield(a, b, f(v, 0)),
        ShapeKind::Tag => crate::tag(a, b, f(v, 0), f(v, 1)),
        ShapeKind::Gear => crate::gear(a, b, f(v, 0), f(v, 1), f(v, 2)),
        ShapeKind::Cross => crate::cross(a, b, f(v, 0)),
        ShapeKind::Check => crate::check(a, b, f(v, 0)),
        ShapeKind::Banner => crate::banner(a, b, f(v, 0)),
        // O 3º valor (ou o 2º, no cone) e o PONTO DE VISTA: >= 0,5 = visto de baixo. Ele
        // muda quais arestas EXISTEM, nao so a aparencia.
        ShapeKind::IsoCube => crate::iso_cube(a, b, f(v, 0), f(v, 1), f(v, 2) >= 0.5),
        ShapeKind::IsoCone => crate::iso_cone(a, b, f(v, 0), f(v, 1) >= 0.5),
        ShapeKind::IsoPyramid => crate::iso_pyramid(a, b, f(v, 0), f(v, 1), f(v, 2) >= 0.5),
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
        assert_eq!(ShapeKind::Pie.as_u16(), 8);
        assert_eq!(ShapeKind::Segment.as_u16(), 9);
        assert_eq!(ShapeKind::ArrowRight.as_u16(), 10);
        assert_eq!(ShapeKind::ArrowCurved.as_u16(), 14);
        assert_eq!(ShapeKind::Diamond.as_u16(), 15);
        assert_eq!(ShapeKind::NoteBracket.as_u16(), 28);
        assert_eq!(ShapeKind::SpeechRect.as_u16(), 29);
        assert_eq!(ShapeKind::Banner.as_u16(), 44);
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
