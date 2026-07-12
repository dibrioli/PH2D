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

/// Teto de parâmetros de uma forma. **Espelha `ph2d_vec_scene::MAX_SHAPE_FIELDS`** — o
/// ECS não depende do vetor (usa só primitivos), então o número é duplicado aqui e um
/// gate na shell prova que os dois nunca divergem.
pub const MAX_SHAPE_VALUES: usize = 8;

/// Os parâmetros de uma forma vetorial viva. A geometria LOCAL do `VecPath` é uma
/// função pura destes parâmetros (a pose fica no `Transform` da entidade, ADR-0111).
/// Ausência do componente = path cru (desenhado com a caneta, ou já convertido).
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VecShape {
    /// Forma paramétrica (retângulo, elipse, polígono, estrela, seta, balão, …) —
    /// **uma variante só**, porque a forma é DADO:
    ///
    /// - `kind` = o discriminante do catálogo (`ph2d_vec_scene::ShapeKind as u16`;
    ///   append-only, é o que fica gravado no save);
    /// - `w`/`h` = a caixa AUTORADA pelo gesto, **com sinal** (a reta precisa da direção);
    /// - `values` = os parâmetros da forma em unidades de **MUNDO**, na ordem que o
    ///   catálogo declara. Campos não usados ficam em zero.
    ///
    /// Uma forma nova não acrescenta variante aqui: entra no catálogo e já é salva,
    /// desfeita e re-cozida de graça.
    Param {
        kind: u16,
        w: f64,
        h: f64,
        values: [f64; MAX_SHAPE_VALUES],
    },
    /// O texto é a exceção: os parâmetros dele não são números (string, família, eixos).
    Text(VecTextParams),
}

impl SimComponent for VecShape {}
