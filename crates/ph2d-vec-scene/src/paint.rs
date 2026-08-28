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

    /// **O lado MAIOR de uma cópia** — o número que o slider *Size* e a alça de escala autoram.
    ///
    /// ⭐ **O painel autora UM número e o documento guarda DOIS**, e a diferença é o aspecto da
    /// arte: oferecer os dois lados deixaria o artista esmagar a imagem sem querer.
    #[must_use]
    pub fn longer_side(&self) -> f64 {
        self.size[0].max(self.size[1])
    }

    /// Reescala `size` preservando o aspecto, para que o lado maior meça `longer`.
    ///
    /// ⚠️⚠️ **A PORTA ÚNICA do tamanho.** O slider e a alça de canvas escrevem por aqui, e é isso
    /// que os impede de divergir — a lei escrita nos dois sítios é a lei que um dia muda só num.
    ///
    /// ⚠️ `is_finite()` ANTES da comparação: um `NaN` reprova toda desigualdade e escorregaria pela
    /// porta de trás, deixando um `size` de `NaN` que apaga a forma sem erro nenhum.
    pub fn set_longer_side(&mut self, longer: f64) {
        let cur = self.longer_side();
        if !cur.is_finite() || cur <= 0.0 || !longer.is_finite() || longer <= 0.0 {
            let v = longer.max(f64::EPSILON);
            self.size = [v, v];
            return;
        }
        let s = longer / cur;
        self.size = [self.size[0] * s, self.size[1] * s];
    }

    /// **A FASE do padrão dentro de UMA repetição**, `0..1` por eixo, medida do canto `base`.
    ///
    /// ⭐ **É uma faixa FECHADA que exprime toda a aparência distinta**, porque deslocar o padrão
    /// por um período inteiro é a identidade — o modelo do `background-position` em percentagem e do
    /// arrasto de origem do Illustrator. Um par de coordenadas de mundo cruas seria ilimitado (a
    /// forma vive onde quiser) e quase todo o curso seria repetição.
    ///
    /// ⚠️ **Os eixos são os do PADRÃO**, não os do mundo: é ao longo deles que a repetição é
    /// periódica, e é por isso que rodar o padrão roda também o que o *Shift* desliza.
    ///
    /// ⚠️ Um período não-positivo não tem fase nenhuma e devolve `0` — o `gap` é **bipolar**, então
    /// `size + gap` pode ser negativo, e `rem_euclid` de um período negativo daria uma fase que anda
    /// para trás.
    #[must_use]
    pub fn shift(&self, base: [f64; 2]) -> [f64; 2] {
        let p = self.period();
        let along = self.along_axes(base);
        [0usize, 1].map(|k| {
            if p[k].is_finite() && p[k] > 0.0 && along[k].is_finite() {
                (along[k] / p[k]).rem_euclid(1.0)
            } else {
                0.0
            }
        })
    }

    /// **Põe a fase do eixo `axis`** (`0` = o eixo X do padrão, `1` = o Y) — a **porta ÚNICA** da
    /// posição, e a única escrita do `origin` depois do nascimento.
    ///
    /// ⚠️ **Só a parte FRACCIONÁRIA se mexe.** Reescrever a posição inteira teleportaria a origem
    /// para junto de `base`; no `Tile` isso seria invisível (um período é a identidade), mas no
    /// `Mirror` a identidade são DOIS períodos e o reflexo trocaria de fase sozinho.
    ///
    /// ⚠️⚠️ **E ela é IDEMPOTENTE por construção**: o deslocamento é aplicado ao `origin` que já
    /// existe, e um delta abaixo de um bilionésimo do período não escreve nada. Sem isso, a ida e
    /// volta `origin -> fase -> origin` (que o painel faz a cada quadro em que o slider está
    /// agarrado) devolveria um `origin` diferente no último bit e cada quadro viraria um passo de
    /// undo — o defeito que o `canonicalize` do editor curou para o mundo inteiro.
    pub fn set_shift_axis(&mut self, base: [f64; 2], axis: usize, shift: f64) {
        let Some(&per) = self.period().get(axis) else {
            return;
        };
        if !per.is_finite() || per <= 0.0 || !shift.is_finite() {
            return;
        }
        let along = self.along_axes(base);
        let Some(&cur) = along.get(axis) else { return };
        if !cur.is_finite() {
            return;
        }
        let delta = (cur / per)
            .div_euclid(1.0)
            .mul_add(per, shift.rem_euclid(1.0) * per)
            - cur;
        // ⚠️ Uma tolerância RELATIVA ao período, e não um epsilon absoluto: o que se compara é
        // "isto é uma fracção autorável de uma repetição?". O chip anda de 1% e o track é `f32`
        // (~1e-7 relativo), então `1e-12` está muito abaixo de qualquer gesto e muito acima do ruído
        // de vírgula flutuante da ida e volta pela base ortonormal (~1e-16).
        if delta.abs() <= per * 1e-12 {
            return;
        }
        let (sin, cos) = self.angle.sin_cos();
        let dir = if axis == 0 { [cos, sin] } else { [-sin, cos] };
        self.origin = [
            delta.mul_add(dir[0], self.origin[0]),
            delta.mul_add(dir[1], self.origin[1]),
        ];
    }

    /// `origin − base`, projectado nos eixos do padrão. A base ortonormal de [`Self::shift`] e de
    /// [`Self::set_shift_axis`], escrita **uma vez** — as duas têm de concordar bit a bit, senão a
    /// fase que o painel mostra não é a fase que ele escreve.
    fn along_axes(&self, base: [f64; 2]) -> [f64; 2] {
        let (sin, cos) = self.angle.sin_cos();
        let d = [self.origin[0] - base[0], self.origin[1] - base[1]];
        [
            d[0].mul_add(cos, d[1] * sin),
            (-d[0]).mul_add(sin, d[1] * cos),
        ]
    }

    /// O afim **pixels do ladrilho -> espaço das âncoras**, para o `brush_transform` do render.
    #[must_use]
    pub fn placement(&self, cells: [u32; 2], tile_px: [u32; 2]) -> [f64; 6] {
        ph2d_vec_pattern::placement(self.period(), cells, self.origin, self.angle, tile_px)
    }

    /// A colocação EFECTIVA dentro da caixa `bbox` da forma.
    ///
    /// ⭐⭐ **No [`PatternMode::Clamp`] ela ENQUADRA a cópia na forma; em qualquer outro modo é a
    /// autorada, ao bit.** O `Clamp` desenha uma cópia só e estica a borda dela pelo resto — com a
    /// cópia do tamanho de um ladrilho, no canto, o artista vê quase só borda esticada (report do
    /// Enio, 2026-08-27: *"clamp deixa tudo em branco"*). Enquadrar é o que o modo promete
    /// (*"mostra a imagem uma vez"*) e é o *Fill* do Figma.
    ///
    /// ⚠️⚠️ **DERIVADA, nunca escrita.** A 1.ª cura gravava `size`/`origin` ao entrar no modo, e o
    /// report seguinte apanhou-a: *"quando volta para tile o aspecto fica de clamp até mudar o
    /// parâmetro Size"*. Escrever destruía a lei que o artista tinha afinado, e voltar não a
    /// devolvia. *Um modo de APRESENTAÇÃO não consome o documento.*
    ///
    /// ⚠️ **Enquadra COBRINDO, não cabendo** (o `max` das duas razões): a forma fica toda pintada e
    /// o aspecto da arte é preservado — a arte transborda em vez de deixar faixa vazia, que num
    /// modo assim se leria como erro de enquadramento.
    #[must_use]
    pub fn placement_in(
        &self,
        cells: [u32; 2],
        tile_px: [u32; 2],
        bbox: ([f64; 2], [f64; 2]),
    ) -> [f64; 6] {
        if !matches!(self.mode, ph2d_vec_pattern::PatternMode::Clamp) {
            return self.placement(cells, tile_px);
        }
        let (lo, hi) = bbox;
        let box_wh = [hi[0] - lo[0], hi[1] - lo[1]];
        let ok = |v: f64| v.is_finite() && v > 0.0;
        if !ok(self.size[0]) || !ok(self.size[1]) || !ok(box_wh[0]) || !ok(box_wh[1]) {
            return self.placement(cells, tile_px);
        }
        let s = (box_wh[0] / self.size[0]).max(box_wh[1] / self.size[1]);
        let framed = [self.size[0] * s, self.size[1] * s];
        ph2d_vec_pattern::placement(framed, cells, lo, self.angle, tile_px)
    }
}
