#![forbid(unsafe_code)]
//! ph2d-vec-edit — máquinas de estado de EDIÇÃO interativa da pipeline vetorial
//! nova (ADR-0108, Fase 1). Operam sobre `ph2d-vec-scene` em **world-space cru**;
//! o shell converte screen→world (via a câmera) e chama estes métodos. Puro, sem
//! vello/kurbo — igual à cena.
//!
//! Fase 1.1: `PenTool` — clicar constrói um path (vértices de quina); clicar perto
//! do início fecha. Handles Bézier + edição de ponto = Fase 1.2.

use ph2d_vec_scene::{Rgba8, VecPath, VecPathId, VecScene, VecVertex};

/// Resultado de um clique do Pen (para o shell logar/reagir se quiser).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PenClick {
    /// Começou um path novo.
    Started,
    /// Anexou um vértice ao path ativo.
    Added,
    /// Fechou o path ativo (clique perto do 1º vértice).
    Closed,
    /// Sem efeito (ex.: path ativo sumiu da cena).
    Ignored,
}

/// Ferramenta Pen: constrói um path incremental na cena. Sem chrome (Fase 1.1
/// roda atrás de flag no shell; a pill do topbar entra no cutover, Fase R).
#[derive(Default)]
pub struct PenTool {
    /// Path em construção (None = próximo clique começa um novo).
    active: Option<VecPathId>,
}

impl PenTool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Há um traço em progresso?
    pub fn is_drawing(&self) -> bool {
        self.active.is_some()
    }

    /// Clique primário em world-space `p`. `px_to_world` = world-units por pixel
    /// de tela (o shell deriva da câmera) — dá limiar de fecho (~12px) e largura
    /// de traço (~3px) constantes em pixels, independentes do zoom.
    pub fn on_click(&mut self, scene: &mut VecScene, p: [f64; 2], px_to_world: f64) -> PenClick {
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
                PenClick::Started
            }
            Some(id) => {
                let Some(path) = scene.path_mut(id) else {
                    self.active = None;
                    return PenClick::Ignored;
                };
                // Fecha se o clique cair perto do 1º vértice (precisa de ≥3 p/
                // formar uma região, senão um clique-duplo no início fecharia nada).
                if path.verts.len() >= 3
                    && let Some(first) = path.verts.first()
                {
                    let (dx, dy) = (p[0] - first.anchor[0], p[1] - first.anchor[1]);
                    if (dx * dx + dy * dy).sqrt() <= close_dist {
                        path.closed = true;
                        path.fill = Some(PEN_FILL);
                        self.active = None;
                        return PenClick::Closed;
                    }
                }
                path.verts.push(VecVertex::corner(p));
                PenClick::Added
            }
        }
    }

    /// Finaliza o traço ativo deixando-o ABERTO (clique secundário / Esc).
    pub fn finish(&mut self) {
        self.active = None;
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
    fn pen_builds_then_closes_a_path() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        assert_eq!(pen.on_click(&mut scene, [0.0, 0.0], PTW), PenClick::Started);
        assert!(pen.is_drawing());
        assert_eq!(pen.on_click(&mut scene, [2.0, 0.0], PTW), PenClick::Added);
        assert_eq!(pen.on_click(&mut scene, [2.0, 2.0], PTW), PenClick::Added);
        assert_eq!(scene.paths().len(), 1);
        assert_eq!(scene.paths()[0].verts.len(), 3);
        // clique a 0.05 world do início (< 12·PTW = 0.12) → fecha
        assert_eq!(pen.on_click(&mut scene, [0.05, 0.0], PTW), PenClick::Closed);
        assert!(!pen.is_drawing());
        assert!(scene.paths()[0].closed);
        assert!(scene.paths()[0].fill.is_some());
    }

    #[test]
    fn near_start_with_two_verts_does_not_close() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_click(&mut scene, [0.0, 0.0], PTW);
        // 2º clique perto do início ainda ADICIONA (não fecha — precisa de ≥3).
        assert_eq!(pen.on_click(&mut scene, [0.05, 0.0], PTW), PenClick::Added);
        assert!(!scene.paths()[0].closed);
    }

    #[test]
    fn finish_leaves_open_path() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_click(&mut scene, [0.0, 0.0], PTW);
        pen.on_click(&mut scene, [2.0, 0.0], PTW);
        pen.finish();
        assert!(!pen.is_drawing());
        assert!(!scene.paths()[0].closed);
        assert_eq!(scene.paths()[0].verts.len(), 2);
    }
}
