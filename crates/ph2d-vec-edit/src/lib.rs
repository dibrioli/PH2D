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

pub mod shape;
pub use shape::{ShapeKind, ShapeTool};

use ph2d_vec_scene::{Rgba8, VecPath, VecPathId, VecScene, VecVertex, VertexKind};

/// Resultado de uma pressão do Pen (para o shell logar/reagir se quiser).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PenClick {
    Started,
    Added,
    Closed,
    /// Agarrou uma âncora/handle existente para editar.
    Grabbed,
    /// Inseriu um vértice novo num segmento (split de Bézier) e o agarrou.
    Inserted,
    Ignored,
}

/// Amostras por segmento no hit-test de "inserir vértice perto do traço".
const INSERT_SAMPLES: u32 = 24;

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

/// Estilo aplicado a paths **recém-criados** pelo Pen. O shell sincroniza a partir
/// da tool (`ph2d-tool-vector`): traço, largura em px, e o fill usado ao fechar.
/// Default = as cores de scaffold da Fase 1.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PenStyle {
    /// Cor do traço dos paths desenhados.
    pub stroke: Rgba8,
    /// Largura do traço em **pixels de tela** (o shell multiplica por `px_to_world`).
    pub stroke_w_px: f64,
    /// Preenchimento aplicado ao FECHAR um path.
    pub fill: Rgba8,
}

impl Default for PenStyle {
    fn default() -> Self {
        Self {
            stroke: PEN_STROKE,
            stroke_w_px: 3.0,
            fill: PEN_FILL,
        }
    }
}

/// Ferramenta Pen + edição de ponto. O estado de documento (`VecScene`) e a
/// history moram no shell (document ≠ tool); esta struct é a máquina de estado
/// de interação. Ativada pela tool real `ph2d-tool-vector` (cutover Fase R).
#[derive(Default)]
pub struct PenTool {
    /// Path em construção (None = a próxima pressão edita ou começa um novo).
    active: Option<VecPathId>,
    /// Path "selecionado" (último tocado) — mostra e permite agarrar seus handles.
    selected: Option<VecPathId>,
    /// Vértice "selecionado" no path selecionado (último tocado/agarrado) — o
    /// alvo dos botões de tipo (Corner/Smooth/Symmetric). `None` = nenhum vértice
    /// específico (ex.: path inteiro selecionado por uma booleana).
    selected_vert: Option<usize>,
    /// Arrastando o handle do vértice recém-posto (desenho, entre press e release).
    dragging: bool,
    /// Elemento agarrado para edição (entre press e release).
    grab: Option<Grab>,
    /// Estilo dos paths recém-criados (sincronizado da tool pelo shell).
    style: PenStyle,
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

    /// Índice do vértice selecionado no path selecionado (alvo dos botões de tipo).
    pub fn selected_vert(&self) -> Option<usize> {
        self.selected_vert
    }

    /// Define a seleção de PATH (ex.: selecionar o resultado de uma booleana).
    /// Limpa a seleção de vértice (não há vértice específico).
    pub fn select(&mut self, id: Option<VecPathId>) {
        self.selected = id;
        self.selected_vert = None;
    }

    /// Tipo do vértice selecionado (para o painel destacar Corner/Smooth/Symmetric).
    /// `None` se não há vértice selecionado ou o índice não existe mais.
    pub fn selected_vertex_kind(&self, scene: &VecScene) -> Option<VertexKind> {
        let i = self.selected_vert?;
        let sel = self.selected?;
        let path = scene.paths().iter().find(|p| p.id == sel)?;
        path.verts.get(i).map(|v| v.kind)
    }

    /// Retipa o vértice selecionado (botões Corner/Smooth/Symmetric). Devolve
    /// `true` se algo mudou (o shell empurra um passo de undo nesse caso).
    pub fn set_selected_vertex_kind(&mut self, scene: &mut VecScene, kind: VertexKind) -> bool {
        let (Some(id), Some(i)) = (self.selected, self.selected_vert) else {
            return false;
        };
        let Some(path) = scene.path_mut(id) else {
            return false;
        };
        ph2d_vec_scene::retype_vertex(path, i, kind)
    }

    /// Apaga o vértice selecionado (Delete / botão), re-costurando os vizinhos.
    /// Se o path ficar com < 2 vértices, remove o path inteiro e limpa a seleção.
    /// A seleção segue no vizinho (delete encadeado). Devolve `true` se apagou.
    pub fn delete_selected_vertex(&mut self, scene: &mut VecScene) -> bool {
        let (Some(id), Some(i)) = (self.selected, self.selected_vert) else {
            return false;
        };
        let Some(path) = scene.path_mut(id) else {
            return false;
        };
        if i >= path.verts.len() {
            return false;
        }
        path.verts.remove(i);
        let remaining = path.verts.len();
        if remaining < 2 {
            scene.remove_path(id);
            self.selected = None;
            self.selected_vert = None;
            self.active = None;
        } else {
            self.selected_vert = Some(i.min(remaining - 1));
        }
        true
    }

    /// Zera todo o estado (ex.: após apagar o path selecionado), preservando o
    /// estilo corrente (é config da tool, não estado de interação).
    pub fn clear(&mut self) {
        let style = self.style;
        *self = Self::default();
        self.style = style;
    }

    /// Estilo dos paths recém-criados.
    pub fn style(&self) -> PenStyle {
        self.style
    }

    /// Ajusta o estilo aplicado aos próximos paths (o shell sincroniza da tool).
    pub fn set_style(&mut self, style: PenStyle) {
        self.style = style;
    }

    /// Pressão primária em world-space `p`. `px_to_world` = world-units por pixel.
    /// `alt` = quebrar a tangente ao agarrar um handle (vira cusp / Corner).
    pub fn on_press(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        px_to_world: f64,
        alt: bool,
    ) -> PenClick {
        let close_dist = 12.0 * px_to_world;
        let hit_r = 10.0 * px_to_world;
        let stroke_w = self.style.stroke_w_px * px_to_world;

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
                    path.fill = Some(self.style.fill);
                    self.active = None;
                    self.dragging = false;
                    return PenClick::Closed;
                }
            }
            path.verts.push(VecVertex::corner(p));
            self.selected_vert = Some(path.verts.len() - 1);
            self.dragging = true;
            return PenClick::Added;
        }

        // Parado → hit-test para EDITAR um ponto existente.
        if let Some(g) = self.hit_test(scene, p, hit_r) {
            self.selected = Some(g.path);
            self.selected_vert = Some(g.vert);
            // Alt + agarrar um HANDLE = quebrar a tangente (vira cusp Corner) antes
            // de arrastar → só esse handle se move (regra Corner). Convenção
            // Illustrator; o undo cobre break+drag num passo (begin no press).
            if alt
                && matches!(g.part, Part::In | Part::Out)
                && let Some(path) = scene.path_mut(g.path)
            {
                let _ = ph2d_vec_scene::retype_vertex(path, g.vert, VertexKind::Corner);
            }
            self.grab = Some(g);
            return PenClick::Grabbed;
        }

        // Perto de um SEGMENTO do path selecionado → insere um vértice (split de
        // Bézier, forma preservada) e o agarra pra arrastar de imediato. Só o
        // path selecionado (previsível — é onde os gizmos aparecem); mesmo raio
        // do grab de handle. Computa a proximidade com borrow imutável, depois muta.
        let insert = self.selected.and_then(|sel| {
            let path = scene.paths().iter().find(|pp| pp.id == sel)?;
            let (seg, t, d2) = ph2d_vec_scene::nearest_point_on_path(path, p, INSERT_SAMPLES)?;
            (d2.sqrt() <= hit_r).then_some((sel, seg, t))
        });
        if let Some((sel, seg, t)) = insert
            && let Some(path) = scene.path_mut(sel)
            && let Some(ni) = ph2d_vec_scene::split_segment(path, seg, t)
        {
            self.selected = Some(sel);
            self.selected_vert = Some(ni);
            self.grab = Some(Grab {
                path: sel,
                vert: ni,
                part: Part::Anchor,
            });
            return PenClick::Inserted;
        }

        // Nada agarrado → começa um path novo.
        let id = scene.push_path(VecPath {
            id: 0,
            verts: vec![VecVertex::corner(p)],
            closed: false,
            fill: None,
            stroke: Some((self.style.stroke, stroke_w)),
        });
        self.active = Some(id);
        self.selected = Some(id);
        self.selected_vert = Some(0);
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
                // O handle oposto segue a restrição do tipo: Symmetric espelha,
                // Smooth mantém colinear preservando o comprimento, Corner é livre.
                Part::In => {
                    v.in_handle = p;
                    match v.kind {
                        VertexKind::Symmetric => v.out_handle = mirror(v.anchor, p),
                        VertexKind::Smooth => {
                            v.out_handle = colinear_opposite(v.anchor, p, v.out_handle)
                        }
                        VertexKind::Corner => {}
                    }
                }
                Part::Out => {
                    v.out_handle = p;
                    match v.kind {
                        VertexKind::Symmetric => v.in_handle = mirror(v.anchor, p),
                        VertexKind::Smooth => {
                            v.in_handle = colinear_opposite(v.anchor, p, v.in_handle)
                        }
                        VertexKind::Corner => {}
                    }
                }
            }
            return true;
        }
        // Arrasto de handle do vértice em desenho: o Pen cria handles simétricos
        // (Symmetric) — o clássico "arrasta pra curvar", quebrável depois (Alt).
        if self.dragging
            && let Some(id) = self.active
            && let Some(path) = scene.path_mut(id)
            && let Some(v) = path.verts.last_mut()
        {
            v.out_handle = p;
            v.in_handle = mirror(v.anchor, p);
            v.kind = VertexKind::Symmetric;
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
            // Handles de QUALQUER tipo, desde que não-degenerados (offset da
            // âncora) — cusps (Corner) e Symmetric também têm handles agarráveis,
            // não só Smooth. Handle tem prioridade sobre âncora (checado antes).
            for (i, v) in path.verts.iter().enumerate() {
                if dist2(v.in_handle, v.anchor) > 1e-18 && dist2(p, v.in_handle) <= r2 {
                    return Some(Grab {
                        path: sel,
                        vert: i,
                        part: Part::In,
                    });
                }
                if dist2(v.out_handle, v.anchor) > 1e-18 && dist2(p, v.out_handle) <= r2 {
                    return Some(Grab {
                        path: sel,
                        vert: i,
                        part: Part::Out,
                    });
                }
            }
        }
        for path in scene.paths() {
            for (i, v) in path.verts.iter().enumerate() {
                if dist2(p, v.anchor) <= r2 {
                    return Some(Grab {
                        path: path.id,
                        vert: i,
                        part: Part::Anchor,
                    });
                }
            }
        }
        None
    }
}

fn mirror(anchor: [f64; 2], h: [f64; 2]) -> [f64; 2] {
    [2.0 * anchor[0] - h[0], 2.0 * anchor[1] - h[1]]
}

/// Handle oposto colinear (regra Smooth): aponta na direção contrária ao handle
/// movido, **preservando o próprio comprimento**. Se o handle movido coincidir
/// com a âncora (sem direção), devolve o oposto inalterado.
fn colinear_opposite(anchor: [f64; 2], moved: [f64; 2], opposite: [f64; 2]) -> [f64; 2] {
    let d = [moved[0] - anchor[0], moved[1] - anchor[1]];
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if l < 1e-9 {
        return opposite;
    }
    let t = [d[0] / l, d[1] / l];
    let o = [opposite[0] - anchor[0], opposite[1] - anchor[1]];
    let opp_len = (o[0] * o[0] + o[1] * o[1]).sqrt();
    [anchor[0] - t[0] * opp_len, anchor[1] - t[1] * opp_len]
}

fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx * dx + dy * dy
}

/// Cor do traço do Pen (claro, sobre o canvas escuro).
const PEN_STROKE: Rgba8 = Rgba8::new(240, 240, 245, 255);
/// Preenchimento leve aplicado ao fechar o path.
const PEN_FILL: Rgba8 = Rgba8::new(90, 150, 230, 120);

/// Teto de passos de undo (memória limitada; snapshots são baratos p/ cenas de
/// desenho, mas não infinitos).
const HISTORY_CAP: usize = 256;

/// Undo/redo por **snapshot** da `VecScene` inteira (ADR-0108 Fase 2). Barato: a
/// cena é `Clone`. Uso: `begin` no início de uma interação (Down), `commit_if_changed`
/// no fim (Up) → só vira passo se a cena mudou de fato; ou `push_undo(pre)` direto
/// numa operação atômica (booleana/delete/load).
#[derive(Default)]
pub struct History {
    undo: Vec<VecScene>,
    redo: Vec<VecScene>,
    pending: Option<VecScene>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot tentativo do estado ATUAL, antes de uma interação que pode mutar.
    pub fn begin(&mut self, scene: &VecScene) {
        self.pending = Some(scene.clone());
    }

    /// Fecha a interação: se a cena difere do snapshot de `begin`, vira um passo de
    /// undo (e limpa o redo). Se nada mudou, descarta o snapshot (não polui o histórico).
    pub fn commit_if_changed(&mut self, scene: &VecScene) {
        if let Some(pre) = self.pending.take()
            && &pre != scene
        {
            self.push_undo(pre);
        }
    }

    /// Descarta o snapshot pendente de `begin` SEM virar passo de undo. Usado quando
    /// uma interação é cancelada e o efeito colateral (ex.: um shape descartado que
    /// já consumiu um `next_id`) não deve poluir o histórico com um passo espúrio.
    pub fn cancel(&mut self) {
        self.pending = None;
    }

    /// Empurra um estado-pré direto pro undo (operação atômica que já sabe que mutou).
    pub fn push_undo(&mut self, pre: VecScene) {
        if self.undo.len() >= HISTORY_CAP {
            self.undo.remove(0);
        }
        self.undo.push(pre);
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Desfaz: devolve o estado anterior; empurra o `current` pro redo.
    pub fn undo(&mut self, current: &VecScene) -> Option<VecScene> {
        let prev = self.undo.pop()?;
        self.redo.push(current.clone());
        Some(prev)
    }

    /// Refaz: devolve o próximo estado; empurra o `current` de volta pro undo.
    pub fn redo(&mut self, current: &VecScene) -> Option<VecScene> {
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PTW: f64 = 0.01; // world-units por pixel (câmera fictícia)

    fn draw_triangle(pen: &mut PenTool, scene: &mut VecScene) {
        pen.on_press(scene, [0.0, 0.0], PTW, false);
        pen.on_release();
        pen.on_press(scene, [4.0, 0.0], PTW, false);
        pen.on_release();
        pen.on_press(scene, [4.0, 4.0], PTW, false);
        pen.on_release();
        pen.on_press(scene, [0.02, 0.0], PTW, false); // fecha (perto do início)
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
    fn drag_makes_a_symmetric_vertex_with_mirrored_handles() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
        assert!(pen.on_drag(&mut scene, [1.0, 0.5]));
        let v = scene.paths()[0].verts[0];
        // The Pen creates classic symmetric handles (mirrored) on drag-out.
        assert_eq!(v.kind, VertexKind::Symmetric);
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
        assert_eq!(pen.on_press(&mut scene, [4.0, 0.0], PTW, false), PenClick::Grabbed);
        assert!(pen.is_dragging());
        assert!(pen.on_drag(&mut scene, [5.0, 1.0]));
        pen.on_release();
        assert!(!pen.is_dragging());
        assert_eq!(scene.paths()[0].verts[1].anchor, [5.0, 1.0]);
    }

    #[test]
    fn grab_a_handle_reshapes_and_mirrors_when_symmetric() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        // desenha 1 ponto (Symmetric) e fecha via finish (fica selecionado)
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
        pen.on_drag(&mut scene, [1.0, 0.0]); // out=[1,0], in=[-1,0], Symmetric
        pen.on_release();
        pen.finish();
        // agarra o out-handle em [1,0] e move → in espelha
        assert_eq!(
            pen.on_press(&mut scene, [1.0, 0.0], PTW, false),
            PenClick::Grabbed
        );
        pen.on_drag(&mut scene, [0.0, 2.0]);
        pen.on_release();
        let v = scene.paths()[0].verts[0];
        assert_eq!(v.out_handle, [0.0, 2.0]);
        assert_eq!(v.in_handle, [0.0, -2.0]); // espelho pela âncora (0,0)
    }

    /// Draw a vertex, retype it Smooth, then dragging one handle keeps the other
    /// COLINEAR but preserves its length (unlike Symmetric which mirrors).
    #[test]
    fn smooth_handle_drag_keeps_opposite_colinear_and_length() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
        pen.on_drag(&mut scene, [2.0, 0.0]); // out=[2,0], in=[-2,0], Symmetric
        pen.on_release();
        pen.finish();
        assert!(pen.set_selected_vertex_kind(&mut scene, VertexKind::Smooth));
        // Grab the out handle ([2,0]) and swing it up-left.
        assert_eq!(
            pen.on_press(&mut scene, [2.0, 0.0], PTW, false),
            PenClick::Grabbed
        );
        pen.on_drag(&mut scene, [0.0, 3.0]); // out now points +Y, len 3
        pen.on_release();
        let v = scene.paths()[0].verts[0];
        assert_eq!(v.out_handle, [0.0, 3.0]);
        // Opposite stays colinear (opposite dir, −Y) but keeps ITS length (2).
        assert!((v.in_handle[0] - 0.0).abs() < 1e-9);
        assert!((v.in_handle[1] - (-2.0)).abs() < 1e-9, "kept length 2, flipped");
    }

    /// Alt + grabbing a handle breaks the tangent: the vertex becomes a Corner
    /// (cusp) and only the grabbed handle moves.
    #[test]
    fn alt_grab_breaks_the_tangent_into_a_corner() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
        pen.on_drag(&mut scene, [2.0, 0.0]); // Symmetric: out=[2,0], in=[-2,0]
        pen.on_release();
        pen.finish();
        // Alt-grab the out handle → break to Corner.
        assert_eq!(
            pen.on_press(&mut scene, [2.0, 0.0], PTW, true),
            PenClick::Grabbed
        );
        pen.on_drag(&mut scene, [2.0, 2.0]);
        pen.on_release();
        let v = scene.paths()[0].verts[0];
        assert_eq!(v.kind, VertexKind::Corner);
        assert_eq!(v.out_handle, [2.0, 2.0]);
        assert_eq!(v.in_handle, [-2.0, 0.0], "in handle untouched (independent)");
    }

    /// Clicking near a segment of the selected path inserts a vertex (Bézier
    /// split) and grabs it — the path gains one Smooth vertex.
    #[test]
    fn click_on_a_segment_inserts_and_grabs_a_vertex() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        // Two-vertex open path: a straight segment from (0,0) to (4,0).
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
        pen.on_release();
        pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
        pen.on_release();
        pen.finish();
        assert_eq!(scene.paths()[0].verts.len(), 2);
        // Click on the middle of the segment (well within hit_r = 10·PTW).
        assert_eq!(
            pen.on_press(&mut scene, [2.0, 0.0], PTW, false),
            PenClick::Inserted
        );
        assert!(pen.is_dragging(), "new vertex is grabbed for immediate drag");
        assert_eq!(scene.paths()[0].verts.len(), 3);
        assert_eq!(scene.paths()[0].verts[1].kind, VertexKind::Smooth);
        assert_eq!(pen.selected_vert(), Some(1));
        pen.on_release();
    }

    #[test]
    fn far_click_does_not_insert_but_starts_a_new_path() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
        pen.on_release();
        pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
        pen.on_release();
        pen.finish();
        // Far from the segment → a new path starts (not an insert).
        assert_eq!(
            pen.on_press(&mut scene, [2.0, 5.0], PTW, false),
            PenClick::Started
        );
        assert_eq!(scene.paths().len(), 2);
    }

    #[test]
    fn delete_selected_vertex_removes_one_node_and_keeps_the_path() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        draw_triangle(&mut pen, &mut scene); // closed, 3 verts
        pen.on_press(&mut scene, [4.0, 0.0], PTW, false); // select vertex 1
        pen.on_release();
        assert_eq!(pen.selected_vert(), Some(1));
        assert!(pen.delete_selected_vertex(&mut scene));
        assert_eq!(scene.paths()[0].verts.len(), 2, "one node gone, path kept");
    }

    #[test]
    fn deleting_below_two_vertices_removes_the_whole_path() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        // Two-vertex open path.
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
        pen.on_release();
        pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
        pen.on_release();
        pen.finish();
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false); // select vertex 0
        pen.on_release();
        assert!(pen.delete_selected_vertex(&mut scene)); // 2 → 1 → remove path
        assert!(scene.is_empty());
        assert_eq!(pen.selected(), None);
        assert_eq!(pen.selected_vert(), None);
    }

    /// The Vertex-type buttons target the selected vertex; `selected_vertex_kind`
    /// reports it for the panel to highlight.
    #[test]
    fn selected_vertex_kind_tracks_the_last_touched_vertex() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        draw_triangle(&mut pen, &mut scene); // closes; last vertex selected
        // Grab a specific anchor to select that vertex.
        pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
        pen.on_release();
        assert_eq!(pen.selected_vert(), Some(1));
        assert_eq!(
            pen.selected_vertex_kind(&scene),
            Some(VertexKind::Corner) // straight corners from clicks
        );
        assert!(pen.set_selected_vertex_kind(&mut scene, VertexKind::Symmetric));
        assert_eq!(pen.selected_vertex_kind(&scene), Some(VertexKind::Symmetric));
        // Selecting a whole path (boolean result) clears the vertex selection.
        pen.select(Some(scene.paths()[0].id));
        assert_eq!(pen.selected_vert(), None);
        assert_eq!(pen.selected_vertex_kind(&scene), None);
    }

    #[test]
    fn plain_click_stays_a_corner() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
        pen.on_release();
        let v = scene.paths()[0].verts[0];
        assert_eq!(v.kind, VertexKind::Corner);
        assert_eq!(v.out_handle, v.anchor);
    }

    #[test]
    fn history_undo_redo_cycle() {
        let mut h = History::new();
        let mut scene = VecScene::new();
        h.begin(&scene);
        scene = VecScene::demo(); // muta (vazio → 2 paths)
        h.commit_if_changed(&scene);
        assert!(h.can_undo());
        let changed = scene.clone();

        scene = h.undo(&scene).unwrap();
        assert!(scene.is_empty());
        assert!(h.can_redo());

        scene = h.redo(&scene).unwrap();
        assert_eq!(scene, changed);
    }

    #[test]
    fn commit_without_change_is_noop() {
        let mut h = History::new();
        let scene = VecScene::new();
        h.begin(&scene);
        h.commit_if_changed(&scene); // nada mudou entre begin e commit
        assert!(!h.can_undo());
    }

    #[test]
    fn set_style_colors_new_paths_and_survives_clear() {
        let mut scene = VecScene::new();
        let mut pen = PenTool::new();
        let style = PenStyle {
            stroke: Rgba8::new(200, 40, 40, 255),
            stroke_w_px: 5.0,
            fill: Rgba8::new(40, 200, 40, 128),
        };
        pen.set_style(style);
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
        let (stroke, w) = scene.paths()[0].stroke.expect("stroke");
        assert_eq!(stroke, style.stroke);
        assert_eq!(w, style.stroke_w_px * PTW);
        // Fechar aplica o fill do estilo.
        pen.on_release();
        pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
        pen.on_release();
        pen.on_press(&mut scene, [4.0, 4.0], PTW, false);
        pen.on_release();
        pen.on_press(&mut scene, [0.02, 0.0], PTW, false); // fecha
        assert_eq!(scene.paths()[0].fill, Some(style.fill));
        // O estilo é config da tool → sobrevive a `clear`.
        pen.clear();
        assert_eq!(pen.style(), style);
    }
}
