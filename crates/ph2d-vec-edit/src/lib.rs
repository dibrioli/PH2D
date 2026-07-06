#![forbid(unsafe_code)]
//! ph2d-vec-edit — máquinas de estado de EDIÇÃO interativa da pipeline vetorial
//! nova (ADR-0108, Fase 1). Operam sobre `ph2d-vec-scene` em **world-space cru**;
//! o shell converte screen→world (via a câmera) e chama estes métodos. Puro, sem
//! vello/kurbo.
//!
//! `PenTool` unifica DESENHO e EDIÇÃO de ponto:
//! - **Desenhando** (há path ativo): pressão adiciona/​fecha; arrastar puxa os
//!   handles Bézier do vértice recém-posto (Fase 1.2).
//! - **Parado** (sem path ativo): pressão faz hit-test — se cair sobre uma âncora
//!   ou handle existente, **agarra** e arrasta (edição, Fase 1.3); senão começa um
//!   path novo. Botão direito finaliza o desenho ativo.

use ph2d_vec_scene::{Rgba8, VecPath, VecPathId, VecScene, VecVertex, VertexKind};

/// Resultado de uma pressão do Pen (para o shell logar/reagir se quiser).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PenClick {
    Started,
    Added,
    Closed,
    /// Agarrou uma âncora/handle existente para editar.
    Grabbed,
    Ignored,
}

/// Parte de um vértice que o hit-test pode agarrar.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Part {
    Anchor,
    In,
    Out,
}

#[derive(Copy, Clone, Debug)]
struct Grab {
    path: VecPathId,
    vert: usize,
    part: Part,
}

/// Ferramenta Pen + edição de ponto. Sem chrome (roda atrás de flag no shell na
/// Fase 1; a pill do topbar entra no cutover, Fase R).
#[derive(Default)]
pub struct PenTool {
    /// Path em construção (None = a próxima pressão edita ou começa um novo).
    active: Option<VecPathId>,
    /// Path "selecionado" (último tocado) — mostra e permite agarrar seus handles.
    selected: Option<VecPathId>,
    /// Arrastando o handle do vértice recém-posto (desenho, entre press e release).
    dragging: bool,
    /// Elemento agarrado para edição (entre press e release).
    grab: Option<Grab>,
}

impl PenTool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Há um traço em progresso (desenhando)?
    pub fn is_drawing(&self) -> bool {
        self.active.is_some()
    }

    /// Está manipulando algo agora (arrasto de desenho OU de edição)?
    pub fn is_dragging(&self) -> bool {
        self.dragging || self.grab.is_some()
    }

    /// Path selecionado (o shell mostra seus gizmos de handle).
    pub fn selected(&self) -> Option<VecPathId> {
        self.selected
    }

    /// Define a seleção (ex.: selecionar o resultado de uma booleana).
    pub fn select(&mut self, id: Option<VecPathId>) {
        self.selected = id;
    }

    /// Zera todo o estado (ex.: após apagar o path selecionado).
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Pressão primária em world-space `p`. `px_to_world` = world-units por pixel.
    pub fn on_press(&mut self, scene: &mut VecScene, p: [f64; 2], px_to_world: f64) -> PenClick {
        let close_dist = 12.0 * px_to_world;
        let hit_r = 10.0 * px_to_world;
        let stroke_w = 3.0 * px_to_world;

        // Desenhando → continua o pen (adiciona/fecha).
        if let Some(id) = self.active {
            let Some(path) = scene.path_mut(id) else {
                self.active = None;
                self.dragging = false;
                return PenClick::Ignored;
            };
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
            return PenClick::Added;
        }

        // Parado → hit-test para EDITAR um ponto existente.
        if let Some(g) = self.hit_test(scene, p, hit_r) {
            self.selected = Some(g.path);
            self.grab = Some(g);
            return PenClick::Grabbed;
        }

        // Nada agarrado → começa um path novo.
        let id = scene.push_path(VecPath {
            id: 0,
            verts: vec![VecVertex::corner(p)],
            closed: false,
            fill: None,
            stroke: Some((PEN_STROKE, stroke_w)),
        });
        self.active = Some(id);
        self.selected = Some(id);
        self.dragging = true;
        PenClick::Started
    }

    /// Arrasto: puxa os handles do vértice em desenho, OU move o elemento agarrado.
    /// Devolve `true` se consumiu.
    pub fn on_drag(&mut self, scene: &mut VecScene, p: [f64; 2]) -> bool {
        // Edição de ponto agarrado.
        if let Some(g) = self.grab {
            let Some(path) = scene.path_mut(g.path) else {
                return false;
            };
            let Some(v) = path.verts.get_mut(g.vert) else {
                return false;
            };
            match g.part {
                Part::Anchor => {
                    let d = [p[0] - v.anchor[0], p[1] - v.anchor[1]];
                    v.anchor = p;
                    v.in_handle = [v.in_handle[0] + d[0], v.in_handle[1] + d[1]];
                    v.out_handle = [v.out_handle[0] + d[0], v.out_handle[1] + d[1]];
                }
                Part::In => {
                    v.in_handle = p;
                    if v.kind == VertexKind::Smooth {
                        v.out_handle = mirror(v.anchor, p);
                    }
                }
                Part::Out => {
                    v.out_handle = p;
                    if v.kind == VertexKind::Smooth {
                        v.in_handle = mirror(v.anchor, p);
                    }
                }
            }
            return true;
        }
        // Arrasto de handle do vértice em desenho (ponto suave simétrico).
        if self.dragging
            && let Some(id) = self.active
            && let Some(path) = scene.path_mut(id)
            && let Some(v) = path.verts.last_mut()
        {
            v.out_handle = p;
            v.in_handle = mirror(v.anchor, p);
            v.kind = VertexKind::Smooth;
            return true;
        }
        false
    }

    /// Solta o botão: encerra arrasto/edição. `true` se havia manipulação (o clique
    /// foi consumido — o shell não deve deixar cair pra pan).
    pub fn on_release(&mut self) -> bool {
        let was = self.dragging || self.grab.is_some();
        self.dragging = false;
        self.grab = None;
        was
    }

    /// Finaliza o traço ativo deixando-o ABERTO (clique secundário / Esc).
    pub fn finish(&mut self) {
        self.active = None;
        self.dragging = false;
        self.grab = None;
    }

    /// Acha a âncora/handle mais próxima de `p` dentro do raio `r`. Handles só do
    /// path selecionado (é onde os gizmos aparecem); âncoras de todos os paths.
    fn hit_test(&self, scene: &VecScene, p: [f64; 2], r: f64) -> Option<Grab> {
        let r2 = r * r;
        if let Some(sel) = self.selected
            && let Some(path) = scene.paths().iter().find(|pp| pp.id == sel)
        {
            for (i, v) in path.verts.iter().enumerate() {
                if v.kind == VertexKind::Smooth {
                    if dist2(p, v.in_handle) <= r2 {
                        return Some(Grab { path: sel, vert: i, part: Part::In });
                    }
                    if dist2(p, v.out_handle) <= r2 {
                        return Some(Grab { path: sel, vert: i, part: Part::Out });
                    }
                }
            }
        }
        for path in scene.paths() {
            for (i, v) in path.verts.iter().enumerate() {
                if dist2(p, v.anchor) <= r2 {
                    return Some(Grab { path: path.id, vert: i, part: Part::Anchor });
                }
            }
        }
        None
    }
}

fn mirror(anchor: [f64; 2], h: [f64; 2]) -> [f64; 2] {
    [2.0 * anchor[0] - h[0], 2.0 * anchor[1] - h[1]]
}

fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx * dx + dy * dy
}

/// Cor do traço do Pen (claro, sobre o canvas escuro).
const PEN_STROKE: Rgba8 = Rgba8::new(240, 240, 245, 255);
/// Preenchimento leve aplicado ao fechar o path.
const PEN_FILL: Rgba8 = Rgba8::new(90, 150, 230, 120);

#[cfg(test)]
mod tests {
    use super::*;

    const PTW: f64 = 0.01; // world-units por pixel (câmera fictícia)

    fn draw_triangle(pen: &mut PenTool, scene: &mut VecScene) {
        pen.on_press(scene, [0.0, 0.0], PTW);
        pen.on_release();
        pen.on_press(scene, [4.0, 0.0], PTW);
        pen.on_release();
        pen.on_press(scene, [4.0, 4.0], PTW);
        pen.on_release();
        pen.on_press(scene, [0.02, 0.0], PTW); // fecha (perto do início)
    }

    #[test]
    fn press_builds_then_closes_a_path() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        draw_triangle(&mut pen, &mut scene);
        assert!(!pen.is_drawing());
        assert!(scene.paths()[0].closed);
        assert_eq!(scene.paths()[0].verts.len(), 3);
    }

    #[test]
    fn drag_makes_a_smooth_vertex_with_mirrored_handles() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW);
        assert!(pen.on_drag(&mut scene, [1.0, 0.5]));
        let v = scene.paths()[0].verts[0];
        assert_eq!(v.kind, VertexKind::Smooth);
        assert_eq!(v.out_handle, [1.0, 0.5]);
        assert_eq!(v.in_handle, [-1.0, -0.5]);
        pen.on_release();
    }

    #[test]
    fn grab_and_move_an_existing_anchor_translates_its_handles() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        draw_triangle(&mut pen, &mut scene); // fecha → active None, selected = path
        // pressão sobre a âncora [4,0] → agarra
        assert_eq!(pen.on_press(&mut scene, [4.0, 0.0], PTW), PenClick::Grabbed);
        assert!(pen.is_dragging());
        assert!(pen.on_drag(&mut scene, [5.0, 1.0]));
        pen.on_release();
        assert!(!pen.is_dragging());
        assert_eq!(scene.paths()[0].verts[1].anchor, [5.0, 1.0]);
    }

    #[test]
    fn grab_a_handle_reshapes_and_mirrors_when_smooth() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        // desenha 1 ponto suave e fecha via finish (fica selecionado)
        pen.on_press(&mut scene, [0.0, 0.0], PTW);
        pen.on_drag(&mut scene, [1.0, 0.0]); // out=[1,0], in=[-1,0], Smooth
        pen.on_release();
        pen.finish();
        // agarra o out-handle em [1,0] e move → in espelha
        assert_eq!(pen.on_press(&mut scene, [1.0, 0.0], PTW), PenClick::Grabbed);
        pen.on_drag(&mut scene, [0.0, 2.0]);
        pen.on_release();
        let v = scene.paths()[0].verts[0];
        assert_eq!(v.out_handle, [0.0, 2.0]);
        assert_eq!(v.in_handle, [0.0, -2.0]); // espelho pela âncora (0,0)
    }

    #[test]
    fn plain_click_stays_a_corner() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW);
        pen.on_release();
        let v = scene.paths()[0].verts[0];
        assert_eq!(v.kind, VertexKind::Corner);
        assert_eq!(v.out_handle, v.anchor);
    }
}
