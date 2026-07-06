#![forbid(unsafe_code)]
//! ph2d-vec-edit — máquinas de estado de EDIÇÃO interativa da pipeline vetorial
//! nova (ADR-0108, Fase 1). Operam sobre `ph2d-vec-scene` em **world-space cru**;
//! o shell converte screen→world (via a câmera) e chama estes métodos. Puro, sem
//! vello/kurbo — igual à cena.
//!
//! Fase 1.2: `PenTool` faz **curvas de verdade** — clicar coloca uma quina;
//! clicar-e-arrastar puxa os handles Bézier (ponto suave simétrico), o gesto
//! padrão de caneta vetorial. Clicar perto do 1º vértice fecha. Editar ponto de
//! um path já pronto (Direct-select) = Fase 1.3.

use ph2d_vec_scene::{Rgba8, VecPath, VecPathId, VecScene, VecVertex, VertexKind};

/// Resultado de uma pressão do Pen (para o shell logar/reagir se quiser).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PenClick {
    /// Começou um path novo.
    Started,
    /// Anexou um vértice ao path ativo.
    Added,
    /// Fechou o path ativo (pressão perto do 1º vértice).
    Closed,
    /// Sem efeito (ex.: path ativo sumiu da cena).
    Ignored,
}

/// Ferramenta Pen: constrói um path incremental na cena. Sem chrome (roda atrás
/// de flag no shell na Fase 1; a pill do topbar entra no cutover, Fase R).
#[derive(Default)]
pub struct PenTool {
    /// Path em construção (None = a próxima pressão começa um novo).
    active: Option<VecPathId>,
    /// Arrastando o handle do vértice recém-posto (entre press e release).
    dragging: bool,
}

impl PenTool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Há um traço em progresso?
    pub fn is_drawing(&self) -> bool {
        self.active.is_some()
    }

    /// Está arrastando um handle agora (entre press e release)?
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// Pressão primária em world-space `p`: coloca uma âncora de quina (ou fecha,
    /// se perto do 1º vértice) e ARMA o arrasto de handle do vértice recém-posto.
    /// `px_to_world` = world-units por pixel (o shell deriva da câmera) → limiar
    /// de fecho (~12px) e largura de traço (~3px) constantes em pixels.
    pub fn on_press(&mut self, scene: &mut VecScene, p: [f64; 2], px_to_world: f64) -> PenClick {
        let close_dist = 12.0 * px_to_world;
        let stroke_w = 3.0 * px_to_world;
        match self.active {
            None => {
                let id = scene.push_path(VecPath {
                    id: 0,
                    verts: vec![VecVertex::corner(p)],
                    closed: false,
                    fill: None,
                    stroke: Some((PEN_STROKE, stroke_w)),
                });
                self.active = Some(id);
                self.dragging = true;
                PenClick::Started
            }
            Some(id) => {
                let Some(path) = scene.path_mut(id) else {
                    self.active = None;
                    self.dragging = false;
                    return PenClick::Ignored;
                };
                // Fecha se cair perto do 1º vértice (≥3 p/ formar uma região).
                if path.verts.len() >= 3
                    && let Some(first) = path.verts.first()
                {
                    let (dx, dy) = (p[0] - first.anchor[0], p[1] - first.anchor[1]);
                    if (dx * dx + dy * dy).sqrt() <= close_dist {
                        path.closed = true;
                        path.fill = Some(PEN_FILL);
                        self.active = None;
                        self.dragging = false;
                        return PenClick::Closed;
                    }
                }
                path.verts.push(VecVertex::corner(p));
                self.dragging = true;
                PenClick::Added
            }
        }
    }

    /// Enquanto o botão está segurado após uma âncora: puxa os handles Bézier do
    /// último vértice — `out` = cursor, `in` = espelho pela âncora (ponto suave
    /// simétrico). No-op sem arrasto ativo. Devolve `true` se consumiu.
    pub fn on_drag(&mut self, scene: &mut VecScene, p: [f64; 2]) -> bool {
        if !self.dragging {
            return false;
        }
        let Some(id) = self.active else {
            return false;
        };
        let Some(path) = scene.path_mut(id) else {
            return false;
        };
        if let Some(v) = path.verts.last_mut() {
            let a = v.anchor;
            v.out_handle = p;
            v.in_handle = [2.0 * a[0] - p[0], 2.0 * a[1] - p[1]];
            v.kind = VertexKind::Smooth;
        }
        true
    }

    /// Solta o botão: encerra o arrasto de handle. Devolve `true` se havia arrasto
    /// (o clique foi consumido pelo Pen — o shell não deve deixar cair pra pan).
    pub fn on_release(&mut self) -> bool {
        let was = self.dragging;
        self.dragging = false;
        was
    }

    /// Finaliza o traço ativo deixando-o ABERTO (clique secundário / Esc).
    pub fn finish(&mut self) {
        self.active = None;
        self.dragging = false;
    }
}

/// Cor do traço do Pen (claro, sobre o canvas escuro).
const PEN_STROKE: Rgba8 = Rgba8::new(240, 240, 245, 255);
/// Preenchimento leve aplicado ao fechar o path.
const PEN_FILL: Rgba8 = Rgba8::new(90, 150, 230, 120);

#[cfg(test)]
mod tests {
    use super::*;

    const PTW: f64 = 0.01; // world-units por pixel (câmera fictícia)

    #[test]
    fn press_builds_then_closes_a_path() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        assert_eq!(pen.on_press(&mut scene, [0.0, 0.0], PTW), PenClick::Started);
        assert!(pen.is_drawing());
        pen.on_release();
        assert_eq!(pen.on_press(&mut scene, [2.0, 0.0], PTW), PenClick::Added);
        pen.on_release();
        assert_eq!(pen.on_press(&mut scene, [2.0, 2.0], PTW), PenClick::Added);
        pen.on_release();
        assert_eq!(scene.paths().len(), 1);
        assert_eq!(scene.paths()[0].verts.len(), 3);
        // pressão a 0.05 world do início (< 12·PTW = 0.12) → fecha
        assert_eq!(pen.on_press(&mut scene, [0.05, 0.0], PTW), PenClick::Closed);
        assert!(!pen.is_drawing());
        assert!(scene.paths()[0].closed);
        assert!(scene.paths()[0].fill.is_some());
    }

    #[test]
    fn drag_makes_a_smooth_vertex_with_mirrored_handles() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW);
        assert!(pen.is_dragging());
        assert!(pen.on_drag(&mut scene, [1.0, 0.5]));
        let v = scene.paths()[0].verts[0];
        assert_eq!(v.kind, VertexKind::Smooth);
        assert_eq!(v.out_handle, [1.0, 0.5]);
        assert_eq!(v.in_handle, [-1.0, -0.5]); // espelho pela âncora (0,0)
        // solta → arrasto encerra; move posterior não mexe mais no handle
        assert!(pen.on_release());
        assert!(!pen.is_dragging());
        assert!(!pen.on_drag(&mut scene, [9.0, 9.0]));
        assert_eq!(scene.paths()[0].verts[0].out_handle, [1.0, 0.5]);
    }

    #[test]
    fn plain_click_stays_a_corner() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW);
        pen.on_release(); // sem drag entre press e release
        let v = scene.paths()[0].verts[0];
        assert_eq!(v.kind, VertexKind::Corner);
        assert_eq!(v.out_handle, v.anchor);
    }

    #[test]
    fn finish_leaves_open_path() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW);
        pen.on_release();
        pen.on_press(&mut scene, [2.0, 0.0], PTW);
        pen.finish();
        assert!(!pen.is_drawing());
        assert!(!pen.is_dragging());
        assert!(!scene.paths()[0].closed);
        assert_eq!(scene.paths()[0].verts.len(), 2);
    }
}
