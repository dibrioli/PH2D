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
    /// Preenchimento (None = sem fill).
    pub fill: Option<Rgba8>,
    /// Traço (None = sem stroke). Ver [`StrokeSpec`].
    pub stroke: Option<StrokeSpec>,
}

/// Versão do wire-format de save (postcard é posicional → bump a cada mudança de
/// schema). v2: `VertexKind` ganhou `Symmetric`. v3: `stroke` virou
/// [`StrokeSpec`] (cap/join/dash). (Migração robusta = cutover, Fase R.)
pub const VEC_SCENE_SCHEMA_VERSION: u32 = 3;

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

/// Fração do vão ao vizinho usada como comprimento de handle no auto-smooth
/// (Corner→Smooth com handles degenerados). 1/3 é o default de facto (Inkscape).
const AUTO_SMOOTH_FRAC: f64 = 1.0 / 3.0;

/// Retipa o vértice `i` de `path` para `kind`, ajustando os handles conforme a
/// restrição do tipo (o núcleo da edição rica de handles, ADR-0108 Fase 1):
///
/// - **Corner**: mantém as posições dos handles (vira cusp de handles
///   independentes; se colineares antes, continuam onde estão até o próximo drag).
/// - **Smooth**: torna os handles **colineares** preservando cada comprimento.
/// - **Symmetric**: colineares **e** comprimento igual (média).
///
/// A tangente vem dos handles atuais (`out_rel − in_rel`); se ambos forem
/// degenerados (cusp reto), é **sintetizada dos vizinhos** (auto-smooth):
/// tangente = direção `prev→next`, comprimento = [`AUTO_SMOOTH_FRAC`] do vão.
/// Retorna `true` se algo mudou. Puro; sem trig além de `sqrt` (normalização).
#[must_use]
pub fn retype_vertex(path: &mut VecPath, i: usize, kind: VertexKind) -> bool {
    let n = path.verts.len();
    if i >= n {
        return false;
    }
    let before = path.verts[i];
    let a = before.anchor;

    if kind == VertexKind::Corner {
        // Cusp: só marca o tipo; posições preservadas (independentes a partir daqui).
        path.verts[i].kind = VertexKind::Corner;
        return path.verts[i] != before;
    }

    // Handles atuais relativos à âncora + comprimentos.
    let in_rel = [before.in_handle[0] - a[0], before.in_handle[1] - a[1]];
    let out_rel = [before.out_handle[0] - a[0], before.out_handle[1] - a[1]];
    let li = (in_rel[0] * in_rel[0] + in_rel[1] * in_rel[1]).sqrt();
    let lo = (out_rel[0] * out_rel[0] + out_rel[1] * out_rel[1]).sqrt();
    let degenerate = li < 1e-9 && lo < 1e-9;

    // Tangente dos vizinhos (para auto-smooth quando degenerado / sem direção).
    let neighbor = neighbor_tangent(path, i);

    // Direção da tangente unitária.
    let tan = if degenerate {
        match neighbor {
            Some((t, _)) => t,
            None => return false, // nada de que sintetizar (path minúsculo)
        }
    } else {
        // out_rel − in_rel aponta ao longo da tangente (out no +t, in no −t).
        let d = [out_rel[0] - in_rel[0], out_rel[1] - in_rel[1]];
        match normalize(d).or_else(|| neighbor.map(|(t, _)| t)) {
            Some(t) => t,
            None => return false,
        }
    };

    let (len_in, len_out) = if degenerate {
        let base = neighbor.map(|(_, b)| b).unwrap_or(0.0);
        (base, base)
    } else if kind == VertexKind::Symmetric {
        let m = (li + lo) * 0.5;
        (m, m)
    } else {
        (li, lo) // Smooth preserva comprimentos
    };

    path.verts[i].out_handle = [a[0] + tan[0] * len_out, a[1] + tan[1] * len_out];
    path.verts[i].in_handle = [a[0] - tan[0] * len_in, a[1] - tan[1] * len_in];
    path.verts[i].kind = kind;
    path.verts[i] != before
}

/// Tangente unitária `prev→next` no vértice `i` (wrap se fechado) + comprimento
/// de handle sugerido ([`AUTO_SMOOTH_FRAC`] do meio-vão). `None` se não houver
/// vizinhos utilizáveis (path degenerado) ou os vizinhos coincidirem.
fn neighbor_tangent(path: &VecPath, i: usize) -> Option<([f64; 2], f64)> {
    let n = path.verts.len();
    let a = path.verts[i].anchor;
    let prev = if i > 0 {
        Some(path.verts[i - 1].anchor)
    } else if path.closed {
        Some(path.verts[n - 1].anchor)
    } else {
        None
    };
    let next = if i + 1 < n {
        Some(path.verts[i + 1].anchor)
    } else if path.closed {
        Some(path.verts[0].anchor)
    } else {
        None
    };
    // Endpoints de path aberto usam a própria âncora como o vizinho ausente.
    let (p, q) = match (prev, next) {
        (Some(p), Some(q)) => (p, q),
        (Some(p), None) => (p, a),
        (None, Some(q)) => (a, q),
        (None, None) => return None,
    };
    let d = [q[0] - p[0], q[1] - p[1]];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if len < 1e-12 {
        return None;
    }
    Some(([d[0] / len, d[1] / len], len * 0.5 * AUTO_SMOOTH_FRAC))
}

/// Normaliza `v`; `None` se ~zero.
fn normalize(v: [f64; 2]) -> Option<[f64; 2]> {
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if l < 1e-9 {
        None
    } else {
        Some([v[0] / l, v[1] / l])
    }
}

/// Interpolação linear.
fn lerp(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// Avalia a cúbica de Bézier (P0,P1,P2,P3) em `t` (base de Bernstein).
fn cubic_at(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    ]
}

/// Nº de segmentos de `path` (fechado inclui o segmento de fechamento).
fn segment_count(path: &VecPath) -> usize {
    let n = path.verts.len();
    if n < 2 {
        0
    } else if path.closed {
        n
    } else {
        n - 1
    }
}

/// Divide o segmento cúbico `seg` (de `verts[seg]` a `verts[(seg+1) % n]`) no
/// parâmetro `t ∈ [0,1]` via **de Casteljau**, inserindo um vértice **Smooth**
/// novo — a FORMA é preservada exatamente (as duas cúbicas resultantes somam a
/// original). Ajusta o out-handle do vértice anterior e o in-handle do seguinte.
/// Devolve o índice do vértice novo (`seg + 1`), ou `None` se o segmento não
/// existe. É o núcleo do "inserir vértice num segmento" (ADR-0108 Fase 1).
pub fn split_segment(path: &mut VecPath, seg: usize, t: f64) -> Option<usize> {
    let n = path.verts.len();
    if seg >= segment_count(path) {
        return None;
    }
    let a = seg;
    let b = (seg + 1) % n;
    let t = t.clamp(0.0, 1.0);
    let (p0, p1) = (path.verts[a].anchor, path.verts[a].out_handle);
    let (p2, p3) = (path.verts[b].in_handle, path.verts[b].anchor);
    let q0 = lerp(p0, p1, t);
    let q1 = lerp(p1, p2, t);
    let q2 = lerp(p2, p3, t);
    let r0 = lerp(q0, q1, t);
    let r1 = lerp(q1, q2, t);
    let s = lerp(r0, r1, t);
    // Ajusta os vizinhos ANTES de inserir (índices ainda válidos).
    path.verts[a].out_handle = q0;
    path.verts[b].in_handle = q2;
    path.verts.insert(
        a + 1,
        VecVertex {
            anchor: s,
            in_handle: r0,
            out_handle: r1,
            kind: VertexKind::Smooth,
        },
    );
    Some(a + 1)
}

/// Ponto mais próximo de `p` sobre os segmentos de `path` (amostragem uniforme,
/// `samples` por segmento). Devolve `(seg, t, dist²)` — o alvo do "clicar perto
/// do traço pra inserir um vértice" — ou `None` se não há segmentos.
#[must_use]
pub fn nearest_point_on_path(path: &VecPath, p: [f64; 2], samples: u32) -> Option<(usize, f64, f64)> {
    let n = path.verts.len();
    let segs = segment_count(path);
    if segs == 0 {
        return None;
    }
    let s = samples.max(2);
    let mut best: Option<(usize, f64, f64)> = None;
    for seg in 0..segs {
        let (a, b) = (seg, (seg + 1) % n);
        let (p0, p1) = (path.verts[a].anchor, path.verts[a].out_handle);
        let (p2, p3) = (path.verts[b].in_handle, path.verts[b].anchor);
        for k in 0..=s {
            let t = f64::from(k) / f64::from(s);
            let c = cubic_at(p0, p1, p2, p3, t);
            let (dx, dy) = (c[0] - p[0], c[1] - p[1]);
            let d2 = dx * dx + dy * dy;
            let better = match best {
                None => true,
                Some((_, _, bd)) => d2 < bd,
            };
            if better {
                best = Some((seg, t, d2));
            }
        }
    }
    best
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
    p.fill = Some(fill);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_has_fill_and_stroke_paths() {
        let scene = VecScene::demo();
        assert_eq!(scene.paths().len(), 2);
        assert!(scene.paths()[0].fill.is_some() && scene.paths()[0].closed);
        assert!(scene.paths()[1].stroke.is_some() && !scene.paths()[1].closed);
    }

    #[test]
    fn push_path_assigns_monotonic_ids() {
        let mut scene = VecScene::new();
        let a = scene.push_path(VecPath {
            id: 999,
            verts: vec![VecVertex::corner([0.0, 0.0])],
            closed: false,
            fill: None,
            stroke: None,
        });
        let b = scene.push_path(VecPath {
            id: 999,
            verts: vec![VecVertex::corner([1.0, 1.0])],
            closed: false,
            fill: None,
            stroke: None,
        });
        assert_eq!((a, b), (0, 1));
        assert_eq!(scene.paths()[0].id, 0);
    }

    #[test]
    fn demo_grid_count() {
        assert_eq!(VecScene::demo_grid(50).paths().len(), 50);
        assert!(VecScene::demo_grid(0).is_empty());
    }

    #[test]
    fn postcard_roundtrip_is_identity() {
        let scene = VecScene::demo();
        let bytes = scene.to_bytes().unwrap();
        let back = VecScene::from_bytes(&bytes).unwrap();
        assert_eq!(scene, back);
    }

    #[test]
    fn from_bytes_rejects_garbage() {
        assert!(VecScene::from_bytes(&[0xFF, 0xFF, 0xFF]).is_err());
    }

    fn anchor_bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
        let mut mn = [f64::MAX; 2];
        let mut mx = [f64::MIN; 2];
        for v in &p.verts {
            mn[0] = mn[0].min(v.anchor[0]);
            mn[1] = mn[1].min(v.anchor[1]);
            mx[0] = mx[0].max(v.anchor[0]);
            mx[1] = mx[1].max(v.anchor[1]);
        }
        (mn, mx)
    }

    #[test]
    fn rectangle_is_closed_four_corners_spanning_the_bbox() {
        // Corners passed in arbitrary order → normalized bbox.
        let r = rectangle([3.0, 5.0], [-1.0, -2.0]);
        assert!(r.closed && r.fill.is_none() && r.stroke.is_none());
        assert_eq!(r.verts.len(), 4);
        assert!(r.verts.iter().all(|v| v.kind == VertexKind::Corner));
        let (mn, mx) = anchor_bbox(&r);
        assert_eq!((mn, mx), ([-1.0, -2.0], [3.0, 5.0]));
    }

    #[test]
    fn ellipse_matches_blob_when_radii_equal() {
        // `blob` now delegates to `ellipse`; the demo circle must be byte-identical
        // (guards the postcard/demo determinism after the refactor).
        let mut e = ellipse([0.0, 0.0], 1.2, 1.2);
        e.fill = Some(Rgba8::new(90, 150, 230, 255));
        assert_eq!(e, blob([0.0, 0.0], 1.2, Rgba8::new(90, 150, 230, 255)));
        assert!(e.verts.iter().all(|v| v.kind == VertexKind::Smooth));
    }

    #[test]
    fn ellipse_anchors_touch_the_bbox_extents() {
        let e = ellipse([2.0, 3.0], 4.0, 1.0);
        let (mn, mx) = anchor_bbox(&e);
        assert_eq!((mn, mx), ([-2.0, 2.0], [6.0, 4.0]));
    }

    #[test]
    fn regular_polygon_has_sides_corner_verts_and_clamps() {
        let p = regular_polygon([0.0, 0.0], 2.0, 2.0, 5);
        assert!(p.closed);
        assert_eq!(p.verts.len(), 5);
        assert!(p.verts.iter().all(|v| v.kind == VertexKind::Corner));
        // Clamp: sides < 3 → 3.
        assert_eq!(regular_polygon([0.0, 0.0], 1.0, 1.0, 0).verts.len(), 3);
        assert_eq!(
            regular_polygon([0.0, 0.0], 1.0, 1.0, MAX_POLYGON_SIDES + 99)
                .verts
                .len(),
            MAX_POLYGON_SIDES as usize
        );
    }

    #[test]
    fn regular_polygon_first_vertex_is_at_top() {
        // Angle 0 = top (−Y): first anchor sits at (cx, cy − ry).
        let p = regular_polygon([1.0, 1.0], 3.0, 2.0, 6);
        let a = p.verts[0].anchor;
        assert!((a[0] - 1.0).abs() < 1e-9, "x centered");
        assert!((a[1] - (1.0 - 2.0)).abs() < 1e-9, "y at top of bbox");
    }

    /// A closed triangle of straight corners (degenerate handles).
    fn corner_triangle() -> VecPath {
        VecPath {
            id: 0,
            verts: vec![
                VecVertex::corner([0.0, 0.0]),
                VecVertex::corner([4.0, 0.0]),
                VecVertex::corner([2.0, 3.0]),
            ],
            closed: true,
            fill: None,
            stroke: None,
        }
    }

    fn handles_rel(v: &VecVertex) -> ([f64; 2], [f64; 2]) {
        (
            [v.in_handle[0] - v.anchor[0], v.in_handle[1] - v.anchor[1]],
            [v.out_handle[0] - v.anchor[0], v.out_handle[1] - v.anchor[1]],
        )
    }

    fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
        a[0] * b[1] - a[1] * b[0]
    }
    fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
        a[0] * b[0] + a[1] * b[1]
    }
    fn norm(v: [f64; 2]) -> f64 {
        (v[0] * v[0] + v[1] * v[1]).sqrt()
    }

    #[test]
    fn retype_corner_to_smooth_auto_synthesizes_colinear_handles_from_neighbors() {
        let mut p = corner_triangle();
        // Vertex 1 (bbox apex on the base) had degenerate handles → synthesized.
        assert!(retype_vertex(&mut p, 1, VertexKind::Smooth));
        let v = p.verts[1];
        assert_eq!(v.kind, VertexKind::Smooth);
        let (in_rel, out_rel) = handles_rel(&v);
        assert!(norm(in_rel) > 1e-6 && norm(out_rel) > 1e-6, "handles grew");
        // Colinear + opposite (tangent continuous).
        assert!(cross(in_rel, out_rel).abs() < 1e-9, "colinear");
        assert!(dot(in_rel, out_rel) < 0.0, "opposite sides of the anchor");
    }

    #[test]
    fn retype_to_symmetric_equalizes_handle_lengths() {
        let mut p = VecPath {
            id: 0,
            verts: vec![
                VecVertex::corner([0.0, 0.0]),
                // Asymmetric colinear-ish handles on the middle vertex.
                VecVertex::smooth([4.0, 0.0], [3.0, 0.0], [5.0, 0.5]),
                VecVertex::corner([8.0, 0.0]),
            ],
            closed: false,
            fill: None,
            stroke: None,
        };
        assert!(retype_vertex(&mut p, 1, VertexKind::Symmetric));
        let v = p.verts[1];
        let (in_rel, out_rel) = handles_rel(&v);
        assert!((norm(in_rel) - norm(out_rel)).abs() < 1e-9, "equal length");
        assert!(cross(in_rel, out_rel).abs() < 1e-9, "colinear");
        assert!(dot(in_rel, out_rel) < 0.0, "opposite");
    }

    #[test]
    fn retype_to_corner_keeps_handle_positions_as_a_cusp() {
        let mut p = corner_triangle();
        let _ = retype_vertex(&mut p, 1, VertexKind::Symmetric); // grow handles
        let grown = p.verts[1];
        assert!(retype_vertex(&mut p, 1, VertexKind::Corner));
        let cusp = p.verts[1];
        assert_eq!(cusp.kind, VertexKind::Corner);
        // Handles unchanged — Corner just releases the colinear constraint.
        assert_eq!(cusp.in_handle, grown.in_handle);
        assert_eq!(cusp.out_handle, grown.out_handle);
    }

    #[test]
    fn retype_is_noop_when_kind_and_geometry_already_match() {
        let mut p = corner_triangle();
        // Already Corner with degenerate handles → Corner is a true no-op.
        assert!(!retype_vertex(&mut p, 1, VertexKind::Corner));
        // Out-of-bounds index.
        assert!(!retype_vertex(&mut p, 99, VertexKind::Smooth));
    }

    /// A curved open segment for split tests.
    fn curved_segment() -> VecPath {
        VecPath {
            id: 0,
            verts: vec![
                VecVertex::smooth([0.0, 0.0], [0.0, 0.0], [1.0, 2.0]),
                VecVertex::smooth([3.0, 0.0], [2.0, 2.0], [3.0, 0.0]),
            ],
            closed: false,
            fill: None,
            stroke: None,
        }
    }

    #[test]
    fn split_segment_inserts_a_smooth_vertex_on_the_curve() {
        let mut p = curved_segment();
        let (p0, p1, p2, p3) = ([0.0, 0.0], [1.0, 2.0], [2.0, 2.0], [3.0, 0.0]);
        let ni = split_segment(&mut p, 0, 0.4).unwrap();
        assert_eq!(ni, 1);
        assert_eq!(p.verts.len(), 3);
        assert_eq!(p.verts[1].kind, VertexKind::Smooth);
        // The new anchor lies exactly on the original curve at t = 0.4.
        let want = cubic_at(p0, p1, p2, p3, 0.4);
        assert!((p.verts[1].anchor[0] - want[0]).abs() < 1e-9);
        assert!((p.verts[1].anchor[1] - want[1]).abs() < 1e-9);
    }

    #[test]
    fn split_preserves_the_shape_exactly() {
        let mut p = curved_segment();
        let (p0, p1, p2, p3) = ([0.0, 0.0], [1.0, 2.0], [2.0, 2.0], [3.0, 0.0]);
        split_segment(&mut p, 0, 0.4).unwrap();
        // Left sub-cubic spans original t∈[0,0.4]; its local u maps to orig 0.4·u.
        let (la0, la1) = (p.verts[0].anchor, p.verts[0].out_handle);
        let (la2, la3) = (p.verts[1].in_handle, p.verts[1].anchor);
        for &u in &[0.0_f64, 0.5, 1.0] {
            let got = cubic_at(la0, la1, la2, la3, u);
            let want = cubic_at(p0, p1, p2, p3, 0.4 * u);
            assert!((got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9);
        }
        // Right sub-cubic spans original t∈[0.4,1].
        let (ra0, ra1) = (p.verts[1].anchor, p.verts[1].out_handle);
        let (ra2, ra3) = (p.verts[2].in_handle, p.verts[2].anchor);
        for &u in &[0.0_f64, 0.5, 1.0] {
            let got = cubic_at(ra0, ra1, ra2, ra3, u);
            let want = cubic_at(p0, p1, p2, p3, 0.4 + 0.6 * u);
            assert!((got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9);
        }
    }

    #[test]
    fn split_handles_the_closing_segment_of_a_closed_path() {
        let mut p = corner_triangle(); // 3 verts, closed → 3 segments (incl. closing)
        // Closing segment is index 2 (v2 → v0).
        let ni = split_segment(&mut p, 2, 0.5).unwrap();
        assert_eq!(ni, 3); // appended at the end
        assert_eq!(p.verts.len(), 4);
        // Out of range segment → None.
        assert!(split_segment(&mut p, 99, 0.5).is_none());
    }

    #[test]
    fn star_has_2n_alternating_corner_verts() {
        let s = star([0.0, 0.0], 4.0, 4.0, 5, 0.5);
        assert!(s.closed);
        assert_eq!(s.verts.len(), 10); // 2 · 5
        assert!(s.verts.iter().all(|v| v.kind == VertexKind::Corner));
        // Even indices are outer (radius 4), odd are inner (radius 2).
        let rad = |v: &VecVertex| (v.anchor[0].powi(2) + v.anchor[1].powi(2)).sqrt();
        assert!((rad(&s.verts[0]) - 4.0).abs() < 1e-9, "outer");
        assert!((rad(&s.verts[1]) - 2.0).abs() < 1e-9, "inner = 4·0.5");
        // First point at the top (−Y).
        assert!(s.verts[0].anchor[0].abs() < 1e-9 && (s.verts[0].anchor[1] + 4.0).abs() < 1e-9);
        // Clamps.
        assert_eq!(star([0.0, 0.0], 1.0, 1.0, 2, 0.5).verts.len(), 6); // points → 3
    }

    #[test]
    fn rounded_rect_is_eight_corners_within_the_bbox() {
        let rr = rounded_rect([0.0, 0.0], [10.0, 6.0], 2.0);
        assert!(rr.closed);
        assert_eq!(rr.verts.len(), 8);
        assert!(rr.verts.iter().all(|v| v.kind == VertexKind::Corner));
        // Every anchor sits inside the bbox.
        assert!(
            rr.verts
                .iter()
                .all(|v| (0.0..=10.0).contains(&v.anchor[0])
                    && (0.0..=6.0).contains(&v.anchor[1]))
        );
        // At least one handle is offset (the arcs) — the shape is rounded.
        assert!(
            rr.verts
                .iter()
                .any(|v| v.out_handle != v.anchor || v.in_handle != v.anchor)
        );
    }

    #[test]
    fn rounded_rect_degenerates_to_rectangle_at_zero_radius() {
        let rr = rounded_rect([0.0, 0.0], [4.0, 3.0], 0.0);
        assert_eq!(rr.verts.len(), 4);
        assert_eq!(rr, rectangle([0.0, 0.0], [4.0, 3.0]));
    }

    #[test]
    fn rounded_rect_clamps_radius_to_half_the_smaller_side() {
        // 4×10 rect, huge radius → clamps to 2 (half of 4). Anchors still valid.
        let rr = rounded_rect([0.0, 0.0], [4.0, 10.0], 999.0);
        assert_eq!(rr.verts.len(), 8);
        assert!(
            rr.verts
                .iter()
                .all(|v| (0.0..=4.0).contains(&v.anchor[0])
                    && (0.0..=10.0).contains(&v.anchor[1]))
        );
    }

    #[test]
    fn nearest_point_on_path_finds_the_click_segment_and_t() {
        let p = curved_segment();
        // A point near the midpoint of the (only) segment.
        let mid = cubic_at([0.0, 0.0], [1.0, 2.0], [2.0, 2.0], [3.0, 0.0], 0.5);
        let probe = [mid[0], mid[1] + 0.05];
        let (seg, t, d2) = nearest_point_on_path(&p, probe, 64).unwrap();
        assert_eq!(seg, 0);
        assert!((t - 0.5).abs() < 0.1, "t near the middle");
        assert!(d2.sqrt() < 0.1, "close to the curve");
        // A single-vertex path has no segments.
        let dot = VecPath {
            id: 0,
            verts: vec![VecVertex::corner([0.0, 0.0])],
            closed: false,
            fill: None,
            stroke: None,
        };
        assert!(nearest_point_on_path(&dot, [0.0, 0.0], 8).is_none());
    }
}
