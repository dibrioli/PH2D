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

/// Natureza da âncora. Fase 0 mínima; expande na Fase 1 (Smooth colinear,
/// simetria de handle, etc.).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexKind {
    /// Handles independentes (quina).
    Corner,
    /// Handles tratados como suaves na edição (Fase 1 impõe colinearidade).
    Smooth,
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
    /// Traço `(cor, largura em world-units)` (None = sem stroke).
    pub stroke: Option<(Rgba8, f64)>,
}

/// Versão do wire-format de save (postcard é posicional → bump a cada mudança de
/// schema). Fase 2: v1. (Versionamento robusto/migração = cutover, Fase R.)
pub const VEC_SCENE_SCHEMA_VERSION: u32 = 1;

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

/// Blob-círculo (4 cúbicas, magic-number canônico de círculo-Bézier
/// `k = r·0.55228…`, não constante inventada) centrado em `c`, preenchido.
fn blob(c: [f64; 2], r: f64, fill: Rgba8) -> VecPath {
    let k = r * 0.552_284_75;
    let v = |ax: f64, ay: f64, ix: f64, iy: f64, ox: f64, oy: f64| {
        VecVertex::smooth(
            [c[0] + ax, c[1] + ay],
            [c[0] + ix, c[1] + iy],
            [c[0] + ox, c[1] + oy],
        )
    };
    VecPath {
        id: 0,
        verts: vec![
            v(r, 0.0, r, -k, r, k),
            v(0.0, r, k, r, -k, r),
            v(-r, 0.0, -r, k, -r, -k),
            v(0.0, -r, -k, -r, k, -r),
        ],
        closed: true,
        fill: Some(fill),
        stroke: None,
    }
}

fn demo_blob() -> VecPath {
    blob([0.0, 0.0], 120.0 * DEMO_SCALE, Rgba8::new(90, 150, 230, 255))
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
        stroke: Some((Rgba8::new(240, 240, 245, 255), width)),
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
}
