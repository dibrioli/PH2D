#![forbid(unsafe_code)]
//! ph2d-vec-scene — modelo de documento vetorial **editor-first** (ADR-0108, Fase 0).
//!
//! O coração do Vector reposicionado: uma cena de paths editáveis, mutável e
//! clonável (undo por snapshot). É deliberadamente **puro** — zero dep de
//! vello/kurbo/ph2d-color — para não sofrer skew de versão de kurbo e ficar do
//! lado certo do gate (deferido) `vello_kurbo_only_in_ph2d_vector`: a geometria
//! mora em `[f64; 2]` cru e vira `kurbo::BezPath` só no render (`ph2d-vec-render`).
//!
//! Modela o path como o Rive modela o seu (`CubicVertex`): cada vértice é uma
//! **âncora + handle-in + handle-out** — os três skinados independentemente na
//! Fase 1, o que preserva o path **exato e editável** (não vira mesh). Rig/bones,
//! components ECS e cor OKLCH canônica entram na Fase 1.

use serde::{Deserialize, Serialize};

mod geometry;
/// Re-exported only for the crate tests (de Casteljau sampling assertions).
#[cfg(test)]
pub(crate) use geometry::cubic_at;
pub use geometry::{nearest_point_on_path, retype_vertex, split_segment};

/// Whole-path transforms (flip / rotate / translate / scale / smooth / sharpen)
/// live in a sibling module (LOC cap); the `impl VecScene` block is inherent.
mod path_ops;

#[cfg(test)]
mod tests;

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

/// Ponta do traço (mapeia p/ `kurbo::Cap` no render). Default = `Butt`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Junção entre segmentos (mapeia p/ `kurbo::Join`). Default = `Miter`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// Estilo do traço de um path: cor + largura (world-units) + ponta/junção +
/// tracejado opcional. Substitui a tupla `(Rgba8, f64)` da Fase 0.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokeSpec {
    pub color: Rgba8,
    pub width: f64,
    pub cap: LineCap,
    pub join: LineJoin,
    /// Tracejado como **múltiplos da largura**: `Some((dash, gap))` ⇒ traço de
    /// `dash·width` e vão de `gap·width`; `None` (ou `dash ≤ 0`) = contínuo.
    /// Width-aware: engrossar o traço alonga dash e vão na proporção, então a
    /// projeção da ponta nunca engole o vão.
    pub dash: Option<(f64, f64)>,
}

impl StrokeSpec {
    /// Traço sólido, ponta/junção default (Butt/Miter), sem tracejado.
    #[must_use]
    pub fn new(color: Rgba8, width: f64) -> Self {
        Self {
            color,
            width,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            dash: None,
        }
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
/// do path (`[0,1]²`, então escala com a forma) + uma `influence` (força/peso IDW).
/// O render mistura por inverse-distance weighting: `c(p) = Σ wᵢcᵢ / Σ wᵢ`,
/// `wᵢ = influenceᵢ / (dist(p,posᵢ)² + ε)`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientPoint {
    pub pos: [f64; 2],
    pub color: Rgba8,
    pub influence: f64,
}

impl GradientPoint {
    #[must_use]
    pub fn new(pos: [f64; 2], color: Rgba8, influence: f64) -> Self {
        Self {
            pos,
            color,
            influence,
        }
    }
}

/// Preenchimento de um path: cor sólida ou um dos três gradientes. Todos os
/// gradientes são **relativos à bbox** do path (auto-encaixam na forma; sem
/// coordenadas absolutas a transformar na Fase 1). Linear/Radial rasterizam nativo
/// no Vello; MultiPoint (freeform) rasteriza por IDW num image-brush.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Paint {
    /// Cor chapada.
    Solid(Rgba8),
    /// Rampa linear a `angle_deg` graus cruzando a bbox (0° = →, sentido horário).
    Linear {
        stops: Vec<GradientStop>,
        angle_deg: f64,
    },
    /// Rampa radial do centro da bbox até o canto (raio = meia-diagonal).
    Radial { stops: Vec<GradientStop> },
    /// Multi-ponto freeform (Cavalry): blend IDW de pontos no espaço da bbox.
    MultiPoint { points: Vec<GradientPoint> },
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
            Paint::Linear { stops, .. } | Paint::Radial { stops } => {
                stops.first().map_or(Rgba8::new(0, 0, 0, 255), |s| s.color)
            }
            Paint::MultiPoint { points } => {
                points.first().map_or(Rgba8::new(0, 0, 0, 255), |p| p.color)
            }
        }
    }
}

impl From<Rgba8> for Paint {
    fn from(c: Rgba8) -> Self {
        Paint::Solid(c)
    }
}

/// Natureza da âncora — o trio canônico de editor vetorial (Inkscape/Illustrator).
/// Governa como a EDIÇÃO de um handle trata o handle oposto ([`retype_vertex`]
/// aplica a restrição geométrica ao trocar de tipo).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexKind {
    /// Quina / cusp: os dois handles são independentes (arrastar um não move o
    /// outro). Reto quando os handles coincidem com a âncora.
    Corner,
    /// Suave: handles **colineares** (tangente contínua), comprimentos
    /// **independentes** — arrastar um gira o oposto para manter a tangente, mas
    /// preserva o comprimento dele.
    Smooth,
    /// Simétrico: colinear **e** comprimentos iguais — arrastar um espelha o
    /// outro (curvatura contínua). É o que o Pen cria ao arrastar.
    Symmetric,
}

/// Vértice cúbico: âncora + dois handles, em coordenadas **absolutas** de
/// world-space (como o `CubicVertex` do Rive — cada ponto é skinado à parte).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecVertex {
    pub anchor: [f64; 2],
    pub in_handle: [f64; 2],
    pub out_handle: [f64; 2],
    pub kind: VertexKind,
}

impl VecVertex {
    /// Vértice de quina reto: handles coincidentes com a âncora (sem curvatura).
    pub fn corner(anchor: [f64; 2]) -> Self {
        Self {
            anchor,
            in_handle: anchor,
            out_handle: anchor,
            kind: VertexKind::Corner,
        }
    }

    /// Vértice suave com handles absolutos explícitos.
    pub fn smooth(anchor: [f64; 2], in_handle: [f64; 2], out_handle: [f64; 2]) -> Self {
        Self {
            anchor,
            in_handle,
            out_handle,
            kind: VertexKind::Smooth,
        }
    }
}

/// Identificador estável de path dentro de uma cena.
pub type VecPathId = u64;

/// Path vetorial editável: sequência de vértices cúbicos + estilo. Mutável e
/// clonável — o undo da Fase 1 é snapshot da `VecScene` inteira.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecPath {
    pub id: VecPathId,
    pub verts: Vec<VecVertex>,
    pub closed: bool,
    /// Preenchimento (None = sem fill). Ver [`Paint`] (sólido ou gradiente).
    pub fill: Option<Paint>,
    /// Traço (None = sem stroke). Ver [`StrokeSpec`].
    pub stroke: Option<StrokeSpec>,
}

/// Versão do wire-format de save (postcard é posicional → bump a cada mudança de
/// schema). v2: `VertexKind` ganhou `Symmetric`. v3: `stroke` virou
/// [`StrokeSpec`] (cap/join/dash). v4: `fill` virou [`Paint`] (sólido + gradientes
/// Linear/Radial/MultiPoint). (Migração robusta = cutover, Fase R.)
pub const VEC_SCENE_SCHEMA_VERSION: u32 = 4;

/// Reordenação na pilha de render (índice `0` = fundo, último = frente). Uma
/// operação de documento, mapeada pela shell a partir dos botões Arrange (mirror
/// de [`crate`]'s BoolOp no vetor). `Raise`/`Lower` movem um passo; `ToFront`/
/// `ToBack` vão ao extremo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ZOrder {
    /// Um passo à frente (troca com o vizinho de cima).
    Raise,
    /// Um passo atrás (troca com o vizinho de baixo).
    Lower,
    /// Ao topo da pilha (renderiza por último, sobre todos).
    ToFront,
    /// Ao fundo da pilha (renderiza primeiro, sob todos).
    ToBack,
}

/// Eixo de espelhamento de um path ([`VecScene::flip_path`]). `Horizontal` =
/// esquerda↔direita (espelha X); `Vertical` = cima↔baixo (espelha Y). Ambos em
/// torno do CENTRO da bbox dos pontos de controle do próprio path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FlipAxis {
    Horizontal,
    Vertical,
}

/// Sentido de uma rotação de 90° ([`VecScene::rotate_path`]), em torno do centro
/// da bbox do path. `Cw` = horário, `Ccw` = anti-horário (na convenção de tela,
/// Y para baixo). Quarto-de-volta é transcendental-free (só troca de eixo +
/// sinal — HR-5).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Rotate90 {
    Cw,
    Ccw,
}

/// Cena vetorial — o documento editor-first. `PartialEq` para o undo detectar
/// mudança real (só vira passo de histórico se a cena mudou de fato).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VecScene {
    paths: Vec<VecPath>,
    next_id: VecPathId,
}

impl VecScene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn paths(&self) -> &[VecPath] {
        &self.paths
    }

    pub fn paths_mut(&mut self) -> &mut [VecPath] {
        &mut self.paths
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Adiciona um path (o `id` recebido é sobrescrito pelo próximo id da cena)
    /// e devolve o id atribuído.
    pub fn push_path(&mut self, mut path: VecPath) -> VecPathId {
        let id = self.next_id;
        self.next_id += 1;
        path.id = id;
        self.paths.push(path);
        id
    }

    /// Acesso mutável ao path de id `id` (para edição incremental — ex.: o Pen
    /// anexando vértices ao traço em progresso).
    pub fn path_mut(&mut self, id: VecPathId) -> Option<&mut VecPath> {
        self.paths.iter_mut().find(|p| p.id == id)
    }

    /// Remove o path de id `id`; devolve `true` se existia (delete / consumo por
    /// booleana).
    pub fn remove_path(&mut self, id: VecPathId) -> bool {
        if let Some(i) = self.paths.iter().position(|p| p.id == id) {
            self.paths.remove(i);
            true
        } else {
            false
        }
    }

    /// Duplica o path `id`, deslocando o clone por `(dx, dy)` world-units (âncora
    /// **e** os dois handles de cada vértice), e devolve o id NOVO (empilhado no
    /// topo). `None` se o id não existe. O clone herda estilo/fill/closed.
    pub fn duplicate_path(&mut self, id: VecPathId, dx: f64, dy: f64) -> Option<VecPathId> {
        let mut clone = self.paths.iter().find(|p| p.id == id)?.clone();
        for v in &mut clone.verts {
            v.anchor[0] += dx;
            v.anchor[1] += dy;
            v.in_handle[0] += dx;
            v.in_handle[1] += dy;
            v.out_handle[0] += dx;
            v.out_handle[1] += dy;
        }
        Some(self.push_path(clone))
    }

    /// Reordena o path `id` na pilha de render ([`ZOrder`]). Devolve `true` se a
    /// posição mudou (`false` se o id sumiu ou já estava no extremo pedido).
    pub fn reorder_path(&mut self, id: VecPathId, order: ZOrder) -> bool {
        let Some(i) = self.paths.iter().position(|p| p.id == id) else {
            return false;
        };
        let last = self.paths.len() - 1;
        let j = match order {
            ZOrder::Raise => (i + 1).min(last),
            ZOrder::Lower => i.saturating_sub(1),
            ZOrder::ToFront => last,
            ZOrder::ToBack => 0,
        };
        if i == j {
            return false;
        }
        let p = self.paths.remove(i);
        self.paths.insert(j, p);
        true
    }

    /// Serializa a cena (postcard), prefixada pela versão de schema.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(&(VEC_SCENE_SCHEMA_VERSION, self)).map_err(|e| e.to_string())
    }

    /// Desserializa uma cena salva por [`Self::to_bytes`]; rejeita schema alheio.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let (ver, scene): (u32, VecScene) =
            postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
        if ver != VEC_SCENE_SCHEMA_VERSION {
            return Err(format!(
                "versão de schema {ver} != {VEC_SCENE_SCHEMA_VERSION}"
            ));
        }
        Ok(scene)
    }

    /// Cena de demonstração da Fase 0: um blob fechado **preenchido** + uma curva
    /// aberta **traçada** — prova fill + stroke + curvatura Bézier ponta-a-ponta
    /// pela pipeline nova. Sai de cena quando as ferramentas de desenho entrarem
    /// (Fase 1); não é conteúdo persistido.
    pub fn demo() -> Self {
        let mut scene = Self::new();
        scene.push_path(demo_blob());
        scene.push_path(demo_curve());
        scene
    }

    /// Spike de escala (ADR-0108 §5): `n` blobs numa grade quadrada. Cada frame o
    /// dispatch re-encoda TUDO (sem dirty-tracking ainda) → mede o custo de
    /// re-encode **naive** e fixa o N do kill-criterion. Não é conteúdo real
    /// (dirigido por `PH2D_VEC_DEMO_N` no shell).
    pub fn demo_grid(n: usize) -> Self {
        let mut scene = Self::new();
        if n == 0 {
            return scene;
        }
        let cols = (n as f64).sqrt().ceil() as usize;
        // Empacota a grade num quadrado que CABE na viewport-default (~±4.8 world,
        // aferido no smoke) → todos os N ficam visíveis (a grade fica mais densa
        // conforme N sobe) e o teste de GPU é justo (nada é culled off-screen).
        let extent = 7.0_f64;
        let spacing = extent / cols as f64;
        let r = spacing * 0.4; // raio proporcional ao passo → sem overlap
        let half = extent * 0.5 - spacing * 0.5;
        for i in 0..n {
            let cx = (i % cols) as f64 * spacing - half;
            let cy = (i / cols) as f64 * spacing - half;
            scene.push_path(blob([cx, cy], r, Rgba8::new(90, 150, 230, 255)));
        }
        scene
    }
}

/// Escala da cena-demo em world-units. A câmera-default enquadra uma região
/// pequena → 1% do tamanho original (Enio smoke 2026-07-05: "objetos gigantes,
/// canvas todo azul"). Um knob só, trivial de re-tunar.
const DEMO_SCALE: f64 = 0.01;

/// Constante canônica do círculo-Bézier (`k = r·0.55228…`): o comprimento de
/// handle que aproxima um quarto de círculo com uma cúbica. Não é constante
/// inventada — é o valor publicado (4/3·(√2−1)).
const KAPPA: f64 = 0.552_284_75;

/// Teto de lados de um polígono regular (defensivo contra `sides` absurdo vindo
/// da UI; o slider real fica em 3..12). Independente do `MAX_POLYGON_SIDES=128`
/// congelado do modelo *antigo* (`ph2d-vector-doc`); aqui é só um clamp local.
pub const MAX_POLYGON_SIDES: u32 = 128;

/// Retângulo eixo-alinhado a partir de dois cantos opostos (ordem livre): quatro
/// vértices de quina, fechado, **sem estilo** (o chamador aplica fill/stroke).
/// Base das ferramentas de forma (ADR-0108 Fase 1).
#[must_use]
pub fn rectangle(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let (x0, x1) = (a[0].min(b[0]), a[0].max(b[0]));
    let (y0, y1) = (a[1].min(b[1]), a[1].max(b[1]));
    VecPath {
        id: 0,
        verts: vec![
            VecVertex::corner([x0, y0]),
            VecVertex::corner([x1, y0]),
            VecVertex::corner([x1, y1]),
            VecVertex::corner([x0, y1]),
        ],
        closed: true,
        fill: None,
        stroke: None,
    }
}

/// Elipse centrada em `center` com semi-eixos `rx`/`ry`: quatro vértices suaves
/// com os handles canônicos de círculo-Bézier ([`KAPPA`]). Fechada, sem estilo.
#[must_use]
pub fn ellipse(center: [f64; 2], rx: f64, ry: f64) -> VecPath {
    let (cx, cy) = (center[0], center[1]);
    let (kx, ky) = (rx * KAPPA, ry * KAPPA);
    let v = |ax: f64, ay: f64, ix: f64, iy: f64, ox: f64, oy: f64| {
        VecVertex::smooth([cx + ax, cy + ay], [cx + ix, cy + iy], [cx + ox, cy + oy])
    };
    VecPath {
        id: 0,
        verts: vec![
            v(rx, 0.0, rx, -ky, rx, ky),
            v(0.0, ry, kx, ry, -kx, ry),
            v(-rx, 0.0, -rx, ky, -rx, -ky),
            v(0.0, -ry, -kx, -ry, kx, -ry),
        ],
        closed: true,
        fill: None,
        stroke: None,
    }
}

/// Polígono regular inscrito na elipse de raios `rx`/`ry`: `sides` vértices de
/// quina (arestas retas), primeiro vértice no topo (12h). `sides` clampado a
/// `[3, MAX_POLYGON_SIDES]`. Fechado, sem estilo. Usa cos/sin (geometria de
/// editor — não é sim determinística; kurbo/vello já usam trig internamente).
#[must_use]
pub fn regular_polygon(center: [f64; 2], rx: f64, ry: f64, sides: u32) -> VecPath {
    let n = sides.clamp(3, MAX_POLYGON_SIDES) as usize;
    let (cx, cy) = (center[0], center[1]);
    let step = std::f64::consts::TAU / n as f64;
    // Ângulo 0 = topo (−Y): começa em 12h e caminha em torno da elipse.
    let start = -std::f64::consts::FRAC_PI_2;
    let verts = (0..n)
        .map(|i| {
            let a = start + step * i as f64;
            VecVertex::corner([cx + rx * a.cos(), cy + ry * a.sin()])
        })
        .collect();
    VecPath {
        id: 0,
        verts,
        closed: true,
        fill: None,
        stroke: None,
    }
}

/// Teto de pontas de uma estrela (clamp defensivo; o slider real fica em 3..12).
pub const MAX_STAR_POINTS: u32 = 60;

/// Estrela de `points` pontas inscrita na elipse de raios `rx`/`ry`: `2·points`
/// vértices de quina alternando raio externo (`rx`,`ry`) e interno
/// (`rx·inner_ratio`,`ry·inner_ratio`), primeira ponta no topo. `points` clampado
/// a `[3, MAX_STAR_POINTS]`, `inner_ratio` a `[0.05, 0.95]`. Fechada, sem estilo.
#[must_use]
pub fn star(center: [f64; 2], rx: f64, ry: f64, points: u32, inner_ratio: f64) -> VecPath {
    let n = points.clamp(3, MAX_STAR_POINTS) as usize;
    let ratio = inner_ratio.clamp(0.05, 0.95);
    let (cx, cy) = (center[0], center[1]);
    let step = std::f64::consts::PI / n as f64; // meio passo (2n vértices em 2π)
    let start = -std::f64::consts::FRAC_PI_2;
    let verts = (0..2 * n)
        .map(|i| {
            let a = start + step * i as f64;
            let (sx, sy) = if i % 2 == 0 {
                (rx, ry)
            } else {
                (rx * ratio, ry * ratio)
            };
            VecVertex::corner([cx + sx * a.cos(), cy + sy * a.sin()])
        })
        .collect();
    VecPath {
        id: 0,
        verts,
        closed: true,
        fill: None,
        stroke: None,
    }
}

/// Teto de voltas de uma espiral (clamp defensivo; o slider real fica em 1..8).
pub const MAX_SPIRAL_TURNS: u32 = 8;

/// Espiral de Arquimedes ABERTA inscrita na elipse de raios `rx`/`ry`, com
/// `turns` voltas (clampado a `[1, MAX_SPIRAL_TURNS]`). Cresce do centro
/// (`f = 0`) até a borda (`f = 1`), amostrada a 24 vértices de quina por volta,
/// primeira amostra no topo. Aberta, sem estilo.
#[must_use]
pub fn spiral(center: [f64; 2], rx: f64, ry: f64, turns: u32) -> VecPath {
    let t = turns.clamp(1, MAX_SPIRAL_TURNS);
    let (cx, cy) = (center[0], center[1]);
    let total = std::f64::consts::TAU * f64::from(t);
    let start = -std::f64::consts::FRAC_PI_2;
    let steps = t as usize * 24;
    let verts = (0..=steps)
        .map(|i| {
            let f = i as f64 / steps as f64; // 0..1 (fração do raio E do ângulo)
            let a = start + total * f;
            VecVertex::corner([cx + rx * f * a.cos(), cy + ry * f * a.sin()])
        })
        .collect();
    VecPath {
        id: 0,
        verts,
        closed: false,
        fill: None,
        stroke: None,
    }
}

/// Retângulo de cantos arredondados a partir de dois cantos opostos + raio
/// `radius` (world-units), clampado a metade do menor lado. Oito vértices de
/// quina: arestas retas + quartos-de-círculo (handles `KAPPA`). `radius ≈ 0` →
/// [`rectangle`]. Fechado, sem estilo.
#[must_use]
pub fn rounded_rect(a: [f64; 2], b: [f64; 2], radius: f64) -> VecPath {
    let (x0, x1) = (a[0].min(b[0]), a[0].max(b[0]));
    let (y0, y1) = (a[1].min(b[1]), a[1].max(b[1]));
    let r = radius.max(0.0).min((x1 - x0) * 0.5).min((y1 - y0) * 0.5);
    if r < 1e-9 {
        return rectangle(a, b);
    }
    let k = r * KAPPA;
    // 8 âncoras (sentido horário a partir da aresta de cima) com o handle do arco
    // no lado curvo e handle nulo (na âncora) no lado reto → quinas independentes.
    let corner = |anchor: [f64; 2], in_h: [f64; 2], out_h: [f64; 2]| VecVertex {
        anchor,
        in_handle: in_h,
        out_handle: out_h,
        kind: VertexKind::Corner,
    };
    let verts = vec![
        // v1: fim do arco sup-esq, início da aresta de cima.
        corner([x0 + r, y0], [x0 + r - k, y0], [x0 + r, y0]),
        // v2: fim da aresta de cima, início do arco sup-dir.
        corner([x1 - r, y0], [x1 - r, y0], [x1 - r + k, y0]),
        // v3: fim do arco sup-dir, início da aresta direita.
        corner([x1, y0 + r], [x1, y0 + r - k], [x1, y0 + r]),
        // v4: fim da aresta direita, início do arco inf-dir.
        corner([x1, y1 - r], [x1, y1 - r], [x1, y1 - r + k]),
        // v5: fim do arco inf-dir, início da aresta de baixo.
        corner([x1 - r, y1], [x1 - r + k, y1], [x1 - r, y1]),
        // v6: fim da aresta de baixo, início do arco inf-esq.
        corner([x0 + r, y1], [x0 + r, y1], [x0 + r - k, y1]),
        // v7: fim do arco inf-esq, início da aresta esquerda.
        corner([x0, y1 - r], [x0, y1 - r + k], [x0, y1 - r]),
        // v8: fim da aresta esquerda, início do arco sup-esq.
        corner([x0, y0 + r], [x0, y0 + r], [x0, y0 + r - k]),
    ];
    VecPath {
        id: 0,
        verts,
        closed: true,
        fill: None,
        stroke: None,
    }
}

/// Blob-círculo preenchido (usa [`ellipse`] com `rx = ry`), para a cena-demo.
fn blob(c: [f64; 2], r: f64, fill: Rgba8) -> VecPath {
    let mut p = ellipse(c, r, r);
    p.fill = Some(Paint::solid(fill));
    p
}

fn demo_blob() -> VecPath {
    blob(
        [0.0, 0.0],
        120.0 * DEMO_SCALE,
        Rgba8::new(90, 150, 230, 255),
    )
}

/// Arco aberto (uma cúbica), traçado claro — prova o caminho de stroke. Largura
/// proporcional ao raio (30%) para ficar visível em qualquer `DEMO_SCALE`.
fn demo_curve() -> VecPath {
    let p = |x: f64, y: f64| [x * DEMO_SCALE, y * DEMO_SCALE];
    let width = 120.0 * DEMO_SCALE * 0.3;
    VecPath {
        id: 0,
        verts: vec![
            VecVertex {
                anchor: p(-160.0, -150.0),
                in_handle: p(-160.0, -150.0),
                out_handle: p(-40.0, -280.0),
                kind: VertexKind::Corner,
            },
            VecVertex {
                anchor: p(160.0, -150.0),
                in_handle: p(40.0, -280.0),
                out_handle: p(160.0, -150.0),
                kind: VertexKind::Corner,
            },
        ],
        closed: false,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(240, 240, 245, 255), width)),
    }
}
