//! **COMO uma forma é pintada** — cor, gradiente e o `Paint` que os une.
//!
//! Split de `lib.rs` pelo teto de LOC (700). O corte é por ASSUNTO, e a linha é a mesma que o
//! resto da crate já usa: `lib.rs` fica com *o que uma forma É* (`VecVertex`, `VecPath`,
//! `VecScene` e a árvore) e este módulo com *com que tinta ela aparece*.
//!
//! ⚠️ Os tipos são **re-exportados na raiz** — `ph2d_vec_scene::Paint` continua a resolver, e
//! nenhum dos consumidores do workspace muda de caminho.

use ph2d_vec_pattern::{PatternMode, TileKind, TileLaw, hex_row_period};
use serde::{Deserialize, Serialize};

/// Cor de estilo (sRGB 8-bit). Fase 0: representação mínima; a cor canônica
/// OKLCH (via `ph2d-color`) é refinamento de Fase 1.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Um stop de gradiente linear/radial: cor numa posição `offset ∈ [0,1]` ao longo
/// da rampa. Ordenados por `offset` crescente pelo editor (o render assume isso).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    pub offset: f64,
    pub color: Rgba8,
}

impl GradientStop {
    #[must_use]
    pub fn new(offset: f64, color: Rgba8) -> Self {
        Self { offset, color }
    }
}

/// Um ponto de um gradiente **multi-ponto** (freeform, estilo Cavalry / Illustrator
/// Freeform Gradient): uma cor posicionada em `pos` no espaço NORMALIZADO da bbox
/// do path (`pos` em WORLD-space, mesmo espaço das âncoras, então transforma junto
/// com a shape) + uma `influence` (força/peso IDW) + `jitter` (0..1: ruído
/// determinístico por-texel na contribuição do ponto — grão estilo Cavalry). O
/// render mistura por inverse-distance weighting: `c(p) = Σ wᵢcᵢ / Σ wᵢ`,
/// `wᵢ = influenceᵢ / (dist² + ε)`, com `wᵢ` perturbado por `jitter`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientPoint {
    pub pos: [f64; 2],
    pub color: Rgba8,
    pub influence: f64,
    /// Ruído per-texel na contribuição do ponto, `0..1` (0 = blend liso, default).
    pub jitter: f64,
}

impl GradientPoint {
    /// Ponto liso (`jitter = 0`): o caminho comum. Use [`Self::with_jitter`] para o grão.
    #[must_use]
    pub fn new(pos: [f64; 2], color: Rgba8, influence: f64) -> Self {
        Self {
            pos,
            color,
            influence,
            jitter: 0.0,
        }
    }

    /// Igual a [`Self::new`] mas com `jitter` explícito (0..1).
    #[must_use]
    pub fn with_jitter(pos: [f64; 2], color: Rgba8, influence: f64, jitter: f64) -> Self {
        Self {
            pos,
            color,
            influence,
            jitter,
        }
    }
}

/// Preenchimento de um path: cor sólida ou um dos três gradientes. A geometria do
/// gradiente é armazenada em **WORLD-space** (mesmo espaço das âncoras) e
/// **transforma junto com o path** (translate/scale/rotate/flip movem os pontos do
/// gradiente igual às âncoras) — então rotacionar a shape roda o gradiente
/// rigidamente, sem "respirar" (o bug do gradiente bbox-relativo). Linear/Radial
/// rasterizam nativo no Vello; MultiPoint (freeform) por IDW num image-brush.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Paint {
    /// Cor chapada.
    Solid(Rgba8),
    /// Rampa linear do ponto `start` ao `end` (world-space).
    Linear {
        stops: Vec<GradientStop>,
        start: [f64; 2],
        end: [f64; 2],
    },
    /// Rampa radial do `center` (world-space) até `radius` (world-units).
    Radial {
        stops: Vec<GradientStop>,
        center: [f64; 2],
        radius: f64,
    },
    /// Multi-ponto freeform (Cavalry): blend IDW de pontos em world-space.
    MultiPoint { points: Vec<GradientPoint> },
    /// **Padrão de textura** — uma arte repetida num reticulado (plano 33).
    ///
    /// ⚠️ **`Box` de propósito, e o número decide.** O `Paint` mora dentro de TODO `VecPath`, e todo
    /// `VecPath` entra em TODA fotografia de undo. Medido em 2026-08-27: `size_of::<Paint>()` era
    /// **56** bytes; um [`PatternFill`] em linha levá-lo-ia a mais do dobro, e o custo seria pago
    /// por cada forma da cena a cada passo de undo — inclusive pelas que não têm padrão nenhum. Com
    /// o `Box` ele fica em 56. O postcard não vê a indirecção.
    ///
    /// ⚠️ **Apendado por último**: o postcard é posicional.
    Pattern(Box<PatternFill>),
}

impl Paint {
    /// Cor sólida (o caminho comum; `Rgba8` também converte via [`From`]).
    #[must_use]
    pub fn solid(color: Rgba8) -> Self {
        Paint::Solid(color)
    }

    /// Cor representativa (sólida / 1º stop / 1º ponto) — pra swatch de UI e para
    /// caminhos legados que esperam uma cor única. Preto opaco se um gradiente
    /// estiver (invalidamente) vazio.
    #[must_use]
    pub fn primary_color(&self) -> Rgba8 {
        match self {
            Paint::Solid(c) => *c,
            Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
                stops.first().map_or(Rgba8::new(0, 0, 0, 255), |s| s.color)
            }
            Paint::MultiPoint { points } => {
                points.first().map_or(Rgba8::new(0, 0, 0, 255), |p| p.color)
            }
            // ⭐ O padrão tem uma resposta EXACTA para esta pergunta, e não uma aproximação: a cor
            // que ele já pinta enquanto o ladrilho não resolve.
            Paint::Pattern(p) => p.fallback,
        }
    }
}

impl From<Rgba8> for Paint {
    fn from(c: Rgba8) -> Self {
        Paint::Solid(c)
    }
}

/// **De onde vem a arte de um padrão.** Dois produtores, um consumidor — o molde do `FxImage`.
///
/// ⚠️ Quem consome **nunca ramifica por origem**: a shell resolve os dois no MESMO ladrilho de
/// pixels (um `StableImage` memoizado), e o render só sabe desenhar um ladrilho.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternSource {
    /// Uma imagem do projecto, pela identidade de CONTEÚDO (blake3, HR-6) — dois padrões com os
    /// mesmos pixels custam uma entrada no ficheiro.
    Image(ph2d_asset_id::AssetId),
    /// ⭐ **Uma forma do próprio documento** — o modelo do Figma (*"the pattern's source references
    /// another object on the canvas in the same file"*), e o que torna um padrão **vivo**: editar a
    /// forma-fonte re-assa o ladrilho em toda forma que a usa.
    Shape(crate::VecPathId),
}

/// **Um padrão de textura AUTORADO** — o que o artista mexeu, em unidades de MUNDO.
///
/// ⚠️ **Isto não é a [`TileLaw`]**, e a diferença é o ponto: a `TileLaw` é a lei *assada*, em pixels
/// da arte; esta é a lei *autorada*, no espaço das âncoras. A conversão entre as duas vive numa
/// porta só ([`Self::law`]), alimentada por outra ([`Self::period`]) — escritas duas vezes, a
/// colmeia ficaria com o desenho num sítio e o espaçamento noutro.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatternFill {
    /// De onde vem a arte.
    pub source: PatternSource,
    /// O reticulado.
    pub kind: TileKind,
    /// O desfasamento é `1/offset_denom` de uma célula (`0`/`1` = nenhum).
    pub offset_denom: u8,
    /// O tamanho da ARTE no mundo — o rasto de UMA cópia, nas mesmas unidades das âncoras.
    pub size: [f64; 2],
    /// O vão acrescentado a cada célula, no mundo. **Negativo é sobreposição** (o *Overlap* do
    /// Illustrator).
    pub gap: [f64; 2],
    /// O canto do ladrilho, no espaço das âncoras.
    pub origin: [f64; 2],
    /// A rotação do padrão, em radianos.
    pub angle: f64,
    /// Como ele preenche o que sobra da forma.
    pub mode: PatternMode,
    /// A opacidade do padrão, `0..=1`.
    ///
    /// ⚠️ A fileira *Fill Opacity* que o painel já tem escreve a alfa da **cor do estilo da
    /// ferramenta**, que um padrão não tem — por isso ela não alcança isto.
    pub alpha: f32,
    /// ⭐ **A cor pintada enquanto o ladrilho não resolve** (a arte ainda não carregou, a
    /// forma-fonte desapareceu, o assado recusou por tamanho).
    ///
    /// ⚠️ **A alternativa era desenhar NADA, e ela é pior**: uma forma invisível lê-se como *"a
    /// ferramenta está partida"*, e o artista não tem como distinguir isso de um preenchimento
    /// vazio. O precedente é o `fallback` do `ProceduralFill` (ADR-0056-amendment-3), que existe
    /// pela mesma razão — *graceful degradation* quando o `id` não resolve.
    pub fallback: Rgba8,
}

impl PatternFill {
    /// Um padrão mínimo: a grade encostada, um ladrilho de uma unidade, sem rotação.
    #[must_use]
    pub fn new(source: PatternSource, size: [f64; 2], fallback: Rgba8) -> Self {
        Self {
            source,
            kind: TileKind::Grid,
            offset_denom: 1,
            size,
            gap: [0.0, 0.0],
            origin: [0.0, 0.0],
            angle: 0.0,
            mode: PatternMode::Tile,
            alpha: 1.0,
            fallback,
        }
    }

    /// **O passo da repetição, em mundo** — `arte + vão`, com UMA excepção.
    ///
    /// ⭐ **A colmeia deriva o passo vertical do horizontal** ([`hex_row_period`], `√3/2`): é o único
    /// valor que põe os seis vizinhos à mesma distância, e é o que separa uma colmeia de um tijolo.
    /// ⚠️ Nessa lei o `gap[1]` autorado é **ignorado** — não há dois sítios a decidir o mesmo passo.
    #[must_use]
    pub fn period(&self) -> [f64; 2] {
        let x = self.size[0] + self.gap[0];
        let y = if matches!(self.kind, TileKind::Hex) {
            hex_row_period(x)
        } else {
            self.size[1] + self.gap[1]
        };
        [x, y]
    }

    /// A lei ASSADA, em pixels da arte — a porta única de mundo para pixel.
    ///
    /// O vão em pixels sai do `período − arte`, e **não** do `gap` autorado: assim a colmeia (cujo
    /// passo vertical é derivado) atravessa a mesma conta que todo o resto.
    #[must_use]
    pub fn law(&self, art_px: [u32; 2]) -> TileLaw {
        let p = self.period();
        let gap = [p[0] - self.size[0], p[1] - self.size[1]];
        TileLaw {
            kind: self.kind,
            offset_denom: self.offset_denom,
            gap_px: ph2d_vec_pattern::gap_px_from_world(gap, self.size, art_px),
        }
    }

    /// O afim **pixels do ladrilho -> espaço das âncoras**, para o `brush_transform` do render.
    #[must_use]
    pub fn placement(&self, cells: [u32; 2], tile_px: [u32; 2]) -> [f64; 6] {
        ph2d_vec_pattern::placement(self.period(), cells, self.origin, self.angle, tile_px)
    }
}
