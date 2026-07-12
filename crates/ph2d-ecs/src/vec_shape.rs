//! `VecShape` — os PARÂMETROS de uma forma vetorial VIVA (Live Shape, estilo
//! Figma/Illustrator).
//!
//! Toda forma paramétrica (retângulo, elipse, polígono, estrela, texto…) é UM objeto
//! cuja geometria é DERIVADA destes parâmetros: a shell re-cozinha o `VecPath` da
//! entidade quando os parâmetros mudam. A entidade que tem este componente é uma
//! forma viva e editável no painel; a que NÃO tem é um path cru (desenhado com a
//! caneta, ou já "convertido em curvas").
//!
//! **"Convert to Curves"** = descartar este componente: a geometria já assada na cena
//! vira um path comum, não-paramétrico (para texto, ainda explode num grupo de paths
//! por-letra — o texto vivo é UM compound de todos os glyphs).
//!
//! Como o [`crate::VecPathRef`], mora aqui (não no `ph2d-vec-scene`) e usa só
//! PRIMITIVOS — `ph2d-ecs` não depende do documento vetorial nem da tipografia. A
//! shell converte de/para os seus tipos ricos (`ShapeKind`, `AxisTag`, `TextAlign`).
//! Registrado no `ComponentRegistry`, então sobrevive a undo + save sem código extra.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Parâmetros de um bloco de texto vivo. Espelho de `shells/desktop`'s
/// `VecTextEdit` em primitivos: `align` é `0=Left / 1=Center / 2=Right`; `axes` são
/// os eixos de variação extras (`wght` fica em `weight`), com o tag OT cru de 4 bytes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecTextParams {
    pub text: String,
    /// Baseline da 1ª linha em MUNDO (a geometria do texto é assada aqui; a pose
    /// adicional fica no `Transform`). Guardado para "Convert to Curves" re-cozinhar
    /// os glyphs no lugar certo.
    pub origin: [f64; 2],
    /// Família escolhida (`None` = a fonte embutida).
    pub family: Option<String>,
    /// Tamanho do glyph em unidades de mundo.
    pub size: f64,
    /// Peso (`wght`).
    pub weight: f32,
    /// Entrelinha como múltiplo do tamanho.
    pub line_height: f64,
    /// Espaçamento entre glyphs como fração do tamanho (em).
    pub tracking: f64,
    /// Alinhamento: `0=Left`, `1=Center`, `2=Right`.
    pub align: u8,
    /// Eixos de variação extras (fora `wght`): `(tag OT de 4 bytes, valor)`.
    pub axes: Vec<([u8; 4], f32)>,
}

/// Os parâmetros de uma forma vetorial viva. A geometria LOCAL do `VecPath` é uma
/// função pura destes parâmetros (a pose fica no `Transform` da entidade, ADR-0111);
/// `w`/`h` são as extensões locais em mundo. Ausência do componente = path cru.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VecShape {
    Rectangle {
        w: f64,
        h: f64,
    },
    RoundRect {
        w: f64,
        h: f64,
        radius: f64,
    },
    Ellipse {
        w: f64,
        h: f64,
    },
    Polygon {
        w: f64,
        h: f64,
        sides: u32,
    },
    Star {
        w: f64,
        h: f64,
        points: u32,
        inner_ratio: f64,
    },
    Spiral {
        w: f64,
        h: f64,
        turns: u32,
    },
    Line {
        w: f64,
        h: f64,
    },
    Arc {
        w: f64,
        h: f64,
        degrees: f64,
    },
    Text(VecTextParams),
}

impl SimComponent for VecShape {}
