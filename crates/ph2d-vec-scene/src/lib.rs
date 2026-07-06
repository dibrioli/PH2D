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

/// Cena vetorial — o documento editor-first. Fase 0 = container de paths com ids
/// estáveis. Rig/bones + components ECS = Fase 1.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
}

/// Círculo aproximado por 4 cúbicas (magic-number canônico de círculo-Bézier
/// `k = r·0.55228…`, não constante inventada), fechado e preenchido.
fn demo_blob() -> VecPath {
    let r = 120.0_f64;
    let k = r * 0.552_284_75;
    VecPath {
        id: 0,
        verts: vec![
            VecVertex::smooth([r, 0.0], [r, -k], [r, k]),
            VecVertex::smooth([0.0, r], [k, r], [-k, r]),
            VecVertex::smooth([-r, 0.0], [-r, k], [-r, -k]),
            VecVertex::smooth([0.0, -r], [-k, -r], [k, -r]),
        ],
        closed: true,
        fill: Some(Rgba8::new(90, 150, 230, 255)),
        stroke: None,
    }
}

/// Arco aberto (uma cúbica), traçado claro — prova o caminho de stroke.
fn demo_curve() -> VecPath {
    VecPath {
        id: 0,
        verts: vec![
            VecVertex {
                anchor: [-160.0, -150.0],
                in_handle: [-160.0, -150.0],
                out_handle: [-40.0, -280.0],
                kind: VertexKind::Corner,
            },
            VecVertex {
                anchor: [160.0, -150.0],
                in_handle: [40.0, -280.0],
                out_handle: [160.0, -150.0],
                kind: VertexKind::Corner,
            },
        ],
        closed: false,
        fill: None,
        stroke: Some((Rgba8::new(240, 240, 245, 255), 6.0)),
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
}
