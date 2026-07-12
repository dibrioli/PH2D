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
pub use geometry::{ghost_handle, nearest_point_on_path, retype_vertex, split_segment};

/// Compound paths (contornos extras + fill rule) + o índice plano de vértice.
mod compound;
pub use compound::{Contour, FillRule};

/// Pilha de z + recorte de copy/paste. A ÁRVORE de objetos é a Hierarchy do
/// editor (ADR-0110): nome/visibilidade/trava/parentesco são da entidade ECS.
mod structure;
pub use structure::{VecClip, VecViewState};

/// ADR-0111: a geometria do path é LOCAL. O afim que a leva ao mundo vem da
/// entidade (`Transform` ∘ cadeia de pais) e é publicado pela shell a cada frame.
mod xform;
pub use xform::{VecXforms, Xform, xform_of};

/// Whole-path transforms (flip / rotate / translate / scale / bbox) live in a
/// sibling module (LOC cap); the `impl VecScene` block is inherent.
mod path_ops;
pub use path_ops::bake_xform;

/// **O ponto está dentro da forma?** — módulo irmão de `path_ops` (teto de LOC). A ponta de
/// seta é um [`VecPath`] que **não vive na cena**, então o hit-test precisa poder perguntar por
/// CAMINHO, e não só por id — senão a parte gorda da seta fica invisível para o mouse.
mod inside;
pub use inside::contains_point;

/// Reshape ops (smooth / sharpen / simplify / subdivide), likewise a sibling.
mod reshape;

/// Shape primitives (rectangle / ellipse / polygon / star / spiral / round-rect)
/// + the demo scene — a sibling module (LOC cap).
mod shapes;
pub use shapes::{
    MAX_ARC_DEGREES, MAX_POLYGON_SIDES, MAX_SPIRAL_TURNS, arc, ellipse, line, rectangle,
    regular_polygon, regular_polygon_rounded, round_rect_radii, rounded_rect, rounded_rect_corners,
    spiral, star, star_rounded,
};

/// Arredondamento de quinas com raio por-vértice (o motor do canto redondo do polígono
/// e dos dois raios da estrela) — módulo irmão de `shapes` (LOC cap).
pub mod corners;

/// **Suavização de quina** (o *squircle*): o arco de círculo vira `asa + arco curto + asa`,
/// e a curvatura sobe em rampa a partir do lado reto em vez de saltar. Motor de
/// `corners::round_closed_corners_smooth` — módulo irmão de `corners` (LOC cap).
pub(crate) mod smooth;

/// **O espaço de autoria do catálogo**: caixa unitária com `v` para BAIXO (convenção
/// SVG/OOXML), mapeada para o mundo (Y-para-CIMA) num lugar só. As referências de onde as
/// formas vêm são todas Y-para-baixo; sem esta fronteira, cada porte é um espelhamento
/// silencioso esperando para acontecer — e foi (as assimétricas nasceram de cabeça para
/// baixo em 2026-07-12).
pub(crate) mod space;

/// A família do CÍRCULO (elipse · arco · pizza · rosquinha · segmento) — pontos do
/// mesmo espaço de três parâmetros, não cinco primitivas. Módulo irmão de `shapes`.
pub(crate) mod round;
pub use round::{FULL_TURN, arc_open, ellipse_chord, ellipse_sweep};

/// As setas em BLOCO (forma preenchível) — não confundir com as pontas de traço
/// (arrowheads), que são propriedade do stroke. Módulo irmão de `shapes`.
mod arrows;
pub use arrows::{arrow_bent, arrow_block, arrow_double, chevron};

/// O vocabulário de FLUXOGRAMA (ANSI/ISO 5807) — losango, pílula, cilindro, documento…
/// Módulo irmão de `shapes`.
mod flow;
pub use flow::{
    cylinder, delay, diamond, display, document, hexagon_flat, junction, note_bracket, offpage,
    parallelogram, pill, predefined_process, trapezoid,
};

/// Os SÍMBOLOS (coração, lua, gota, escudo, engrenagem, tag, raio, cruz, check, banner).
/// Módulo irmão de `shapes`.
mod symbols;
pub use symbols::{banner, bolt, check, cross, drop, gear, heart, moon, shield, tag};

/// Os BALÕES (fala · pensamento · nuvem · explosão · chave). Família própria: o que os
/// define é o RABO (a ponta que aponta para quem fala) e a costura de arcos tangentes da
/// nuvem — assunto bem diferente do dos símbolos. Módulo irmão de `shapes`.
mod bubbles;
pub use bubbles::{brace, burst, cloud, speech_oval, speech_rect, thought};

/// O motor comum dos balões (anel de vértices · costura de arcos · enxerto do rabo ·
/// reajuste de bbox). Módulo irmão de `bubbles` (teto de LOC).
mod bubbles_ring;

/// Os sólidos ISOMÉTRICOS (cubo · cone · pirâmide). Módulo próprio porque a projeção é um
/// assunto seu — dois parâmetros (`rise`/`skew`) governam os três, e as arestas internas
/// são o que os faz ler como volume. Módulo irmão de `shapes`.
mod iso;
pub use iso::{iso_cone, iso_cube, iso_pyramid};

/// **As pontas de traço** (setas, losangos, bolinhas). NÃO são formas do catálogo: são
/// propriedade do STROKE — nascem na ponta de qualquer caminho aberto, herdam a cor e a
/// largura dele, e giram sozinhas com a tangente da curva.
mod marker;
pub use marker::{ALL_MARKERS, Marker, end_tangent, stroke_head, trim_path};

/// **Onde uma linha ENCOSTA numa forma** — a borda real (o contorno achatado), não a caixa
/// envolvente. É o que faz um conector parar na estrela em vez de por baixo dela.
mod boundary;
pub use boundary::{boundary_hit, outline};

/// **O filete de quina de uma polilinha** — o que transforma o cotovelo duro de um
/// fluxograma numa linha que dobra suave. Arco exato em qualquer ângulo, sem transcendental.
mod polyline;
pub use polyline::{round_polyline, smooth_polyline};

/// O CATÁLOGO de formas paramétricas: o enum único (`ShapeKind`), os valores de cada
/// forma e o `cook` que os transforma em geometria. A forma é DADO — é o que faz uma
/// forma nova custar uma linha de tabela em vez de oito lugares.
mod kind;
pub use kind::{
    ALL_SHAPES, DEFAULT_ARC_DEGREES, DEFAULT_CORNER_RADIUS, DEFAULT_POLYGON_SIDES,
    DEFAULT_SPIRAL_TURNS, DEFAULT_STAR_INNER, DEFAULT_STAR_POINTS, MAX_SHAPE_FIELDS, ShapeKind,
    ShapeValues, cook,
};

#[cfg(test)]
mod tests;

/// **O gate de ORIENTAÇÃO** (`match` exaustivo sobre `ShapeKind`, sem braço `_`): toda
/// forma do catálogo declara se tem um "cima" — e, se tem, prova para que lado, com um
/// predicado que o gate verifica NÃO ser vazio (isto é: que ele reprova a forma
/// espelhada). Arquivo irmão por causa do teto de LOC.
#[cfg(test)]
#[path = "orient_tests.rs"]
mod orient_tests;

/// **O gate de ROBUSTEZ do `cook`**: o usuário arrasta sliders, então o teste arrasta
/// também. Varre milhares de vetores de parâmetros × caixas de gesto (inclusive as
/// degeneradas) e exige que nada exploda nem saia `NaN`. Arquivo irmão (teto de LOC).
#[cfg(test)]
#[path = "robust_tests.rs"]
mod robust_tests;

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
    /// **Ponta no COMEÇO** do caminho (arrowhead). Só vale em caminho aberto.
    /// Apendado por ÚLTIMO: o postcard é posicional, então um save anterior a este campo
    /// segue legível e lê `Marker::None` (que é o default — uma linha é uma linha).
    #[serde(default)]
    pub marker_start: Marker,
    /// **Ponta no FIM** do caminho. Idem.
    #[serde(default)]
    pub marker_end: Marker,
    /// **Tamanho da ponta**, como múltiplo do tamanho que a largura do traço já dita.
    /// `1.0` = o default.
    ///
    /// Por que um MÚLTIPLO e não um tamanho absoluto: a ponta tem de crescer com o traço,
    /// senão engrossar a linha faz a seta encolher visualmente até virar um cotoco. O
    /// multiplicador dá o ajuste fino sem quebrar essa proporção.
    #[serde(default = "unit_scale")]
    pub marker_scale: f64,
    /// **Arredondamento das quinas da ponta**: `0` = afiada, `1` = o máximo que a geometria
    /// da ponta comporta sem se descaracterizar.
    #[serde(default)]
    pub marker_round: f64,
}

/// O default do [`StrokeSpec::marker_scale`]. Precisa ser uma função porque o default de
/// `serde` para um `f64` é **zero** — e uma ponta de tamanho zero é uma ponta invisível.
fn unit_scale() -> f64 {
    1.0
}

impl StrokeSpec {
    /// Traço sólido, ponta/junção default (Butt/Miter), sem tracejado, sem setas.
    #[must_use]
    pub fn new(color: Rgba8, width: f64) -> Self {
        Self {
            color,
            width,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            dash: None,
            marker_start: Marker::None,
            marker_end: Marker::None,
            marker_scale: 1.0,
            marker_round: 0.0,
        }
    }

    /// `true` se o traço tem alguma ponta — o render então precisa recuar a linha.
    #[must_use]
    pub fn has_markers(&self) -> bool {
        self.marker_start != Marker::None || self.marker_end != Marker::None
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
///
/// `verts`/`closed` são o contorno **primário**. Um path pode ser **compound**
/// (buraco, ilha) carregando contornos extras em `subpaths`; [`FillRule`] decide
/// o que é vazado. Ver o módulo `compound` para o índice plano de vértice que
/// endereça todos os contornos de uma vez.
///
/// **Não** carrega nome, visibilidade, trava nem pai: isso é da entidade ECS que o
/// representa na Hierarquia (ADR-0110). Aqui só geometria e estilo.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VecPath {
    pub id: VecPathId,
    pub verts: Vec<VecVertex>,
    pub closed: bool,
    /// Preenchimento (None = sem fill). Ver [`Paint`] (sólido ou gradiente).
    pub fill: Option<Paint>,
    /// Traço (None = sem stroke). Ver [`StrokeSpec`].
    pub stroke: Option<StrokeSpec>,
    /// Contornos ADICIONAIS (compound path). Vazio = path de contorno único.
    pub subpaths: Vec<Contour>,
    /// Regra de preenchimento entre os contornos. Irrelevante (as duas coincidem)
    /// quando `subpaths` está vazio.
    pub fill_rule: FillRule,
}

/// Versão do wire-format de save (postcard é posicional → bump a cada mudança de
/// schema). v2: `VertexKind` ganhou `Symmetric`. v3: `stroke` virou
/// [`StrokeSpec`] (cap/join/dash). v4: `fill` virou [`Paint`] (sólido + gradientes
/// Linear/Radial/MultiPoint). v5: [`GradientPoint`] ganhou `jitter`. v6: `VecPath`
/// ganhou `subpaths` + `fill_rule` (compound paths). (Migração robusta = cutover,
/// Fase R.)
pub const VEC_SCENE_SCHEMA_VERSION: u32 = 7;

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
    /// A pilha de z (fundo → topo). É uma **projeção** da árvore da Hierarquia,
    /// re-sincronizada pela shell — ver [`VecScene::reorder_to`] (ADR-0110).
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
}
