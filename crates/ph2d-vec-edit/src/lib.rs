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
pub use shape::{ShapeKind, ShapeParams, ShapeTool};

use ph2d_vec_scene::{
    LineCap, LineJoin, Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecScene, VecVertex,
    VertexKind,
};

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
    /// Ponta / junção do traço.
    pub cap: LineCap,
    pub join: LineJoin,
    /// Dash/vão como **múltiplos da largura**: `Some((dash, gap))` ⇒ traço e vão
    /// de `dash·width`/`gap·width`; `None` = contínuo. O render multiplica pela
    /// largura do path.
    pub dash: Option<(f64, f64)>,
}

impl Default for PenStyle {
    fn default() -> Self {
        Self {
            stroke: PEN_STROKE,
            stroke_w_px: 3.0,
            fill: PEN_FILL,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            dash: None,
        }
    }
}

impl PenStyle {
    /// `StrokeSpec` do traço para `width` world-units (aplica cap/join/dash).
    #[must_use]
    pub fn stroke_spec(&self, width: f64) -> StrokeSpec {
        StrokeSpec {
            color: self.stroke,
            width,
            cap: self.cap,
            join: self.join,
            dash: self.dash,
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
    /// Path "selecionado" (último tocado / PRIMÁRIO) — mostra e permite agarrar
    /// seus handles + é o alvo do painel de estilo. É o último de [`Self::selected_paths`].
    selected: Option<VecPathId>,
    /// Seleção de OBJETO multi-path (Align/Distribute + move em grupo). Um clique
    /// simples num path a reduz a `[esse path]`; Shift+clique num path o alterna
    /// (`toggle_path`). O primário ([`Self::selected`]) é sempre o último desta lista.
    selected_paths: Vec<VecPathId>,
    /// Vértices selecionados no path selecionado — o alvo dos botões de tipo
    /// (Corner/Smooth/Symmetric), do delete e do move em lote. Vazio = nenhum
    /// vértice específico (ex.: path inteiro selecionado por uma booleana). O
    /// ÚLTIMO é o "primário" (destaque do painel). Populado por clique único ou
    /// por box-select (Shift+arrastar).
    selected_verts: Vec<usize>,
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

    /// A seleção de objeto multi-path (para overlay + Align/Distribute + move em
    /// grupo). Sempre inclui o primário; vazia quando nada está selecionado.
    pub fn selected_paths(&self) -> &[VecPathId] {
        &self.selected_paths
    }

    /// Shift+clique: alterna `id` na seleção de objeto. Ao adicionar, vira o primário;
    /// ao remover, o primário passa a ser o último remanescente (ou `None`). Limpa a
    /// seleção de vértice (a edição de ponto é do primário de clique simples).
    pub fn toggle_path(&mut self, id: VecPathId) {
        self.selected_verts.clear();
        if let Some(pos) = self.selected_paths.iter().position(|&p| p == id) {
            self.selected_paths.remove(pos);
            self.selected = self.selected_paths.last().copied();
        } else {
            self.selected_paths.push(id);
            self.selected = Some(id);
        }
    }

    /// Hit-test de OBJETO: o path cujo âncora/handle OU contorno está a `hit_r` de
    /// `p` (o mais próximo). Para o Shift+clique de multi-seleção. `None` = vazio.
    pub fn path_at(&self, scene: &VecScene, p: [f64; 2], hit_r: f64) -> Option<VecPathId> {
        if let Some(g) = self.hit_test(scene, p, hit_r) {
            return Some(g.path);
        }
        let mut best: Option<(VecPathId, f64)> = None;
        for path in scene.paths() {
            if let Some((_, _, d2)) = ph2d_vec_scene::nearest_point_on_path(path, p, INSERT_SAMPLES)
                && d2.sqrt() <= hit_r
                && best.is_none_or(|(_, b)| d2 < b)
            {
                best = Some((path.id, d2));
            }
        }
        best.map(|(id, _)| id)
    }

    /// Vértice "primário" (último tocado) — o do destaque do painel; `None` se
    /// nada selecionado.
    pub fn selected_vert(&self) -> Option<usize> {
        self.selected_verts.last().copied()
    }

    /// Todos os vértices selecionados (para o overlay destacá-los).
    pub fn selected_verts(&self) -> &[usize] {
        &self.selected_verts
    }

    /// Define a seleção de PATH (ex.: selecionar o resultado de uma booleana).
    /// Reduz a seleção de objeto a `[id]` (ou vazia) e limpa a seleção de vértice.
    pub fn select(&mut self, id: Option<VecPathId>) {
        self.selected = id;
        self.selected_paths = id.map(|i| vec![i]).unwrap_or_default();
        self.selected_verts.clear();
    }

    /// Nudge por teclado: desloca a seleção por `(dx, dy)` world-units. Se há
    /// vértices selecionados, translada só eles (âncora + handles); senão, o path
    /// inteiro. Devolve `true` se moveu algo (nada selecionado ⇒ `false`).
    pub fn nudge(&mut self, scene: &mut VecScene, dx: f64, dy: f64) -> bool {
        // Multi-path OBJECT selection (no specific vertices) → move every selected
        // path wholesale (Align/Distribute companion).
        if self.selected_verts.is_empty() && self.selected_paths.len() > 1 {
            let mut moved = false;
            for &id in &self.selected_paths {
                moved |= scene.translate_path(id, dx, dy);
            }
            return moved;
        }
        let Some(sel) = self.selected else {
            return false;
        };
        let Some(path) = scene.path_mut(sel) else {
            return false;
        };
        let shift = |v: &mut VecVertex| {
            v.anchor = [v.anchor[0] + dx, v.anchor[1] + dy];
            v.in_handle = [v.in_handle[0] + dx, v.in_handle[1] + dy];
            v.out_handle = [v.out_handle[0] + dx, v.out_handle[1] + dy];
        };
        if self.selected_verts.is_empty() {
            path.verts.iter_mut().for_each(shift);
        } else {
            for &i in &self.selected_verts {
                if let Some(v) = path.verts.get_mut(i) {
                    shift(v);
                }
            }
        }
        true
    }

    /// Box-select: seleciona as âncoras do path (selecionado; senão o que tiver
    /// mais âncoras na caixa) dentro do retângulo world `[min,max]`. Substitui a
    /// seleção. Só muda estado de seleção — não muta a cena, não gera undo.
    pub fn box_select(&mut self, scene: &VecScene, min: [f64; 2], max: [f64; 2]) {
        let (x0, x1) = (min[0].min(max[0]), min[0].max(max[0]));
        let (y0, y1) = (min[1].min(max[1]), min[1].max(max[1]));
        let inside = |a: [f64; 2]| a[0] >= x0 && a[0] <= x1 && a[1] >= y0 && a[1] <= y1;
        let target = self.selected.or_else(|| {
            scene
                .paths()
                .iter()
                .map(|p| (p.id, p.verts.iter().filter(|v| inside(v.anchor)).count()))
                .filter(|&(_, c)| c > 0)
                .max_by_key(|&(_, c)| c)
                .map(|(id, _)| id)
        });
        let Some(id) = target else {
            self.selected_verts.clear();
            return;
        };
        self.selected = Some(id);
        self.selected_paths = vec![id];
        self.selected_verts = scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .map(|p| {
                p.verts
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| inside(v.anchor))
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
    }

    /// Tipo do vértice primário (para o painel destacar Corner/Smooth/Symmetric).
    /// `None` se não há vértice selecionado ou o índice não existe mais.
    pub fn selected_vertex_kind(&self, scene: &VecScene) -> Option<VertexKind> {
        let i = self.selected_vert()?;
        let sel = self.selected?;
        let path = scene.paths().iter().find(|p| p.id == sel)?;
        path.verts.get(i).map(|v| v.kind)
    }

    /// Retipa TODOS os vértices selecionados (botões Corner/Smooth/Symmetric).
    /// Devolve `true` se algo mudou (o shell empurra um passo de undo nesse caso).
    pub fn set_selected_vertex_kind(&mut self, scene: &mut VecScene, kind: VertexKind) -> bool {
        let Some(id) = self.selected else {
            return false;
        };
        let Some(path) = scene.path_mut(id) else {
            return false;
        };
        let mut changed = false;
        for &i in &self.selected_verts {
            changed |= ph2d_vec_scene::retype_vertex(path, i, kind);
        }
        changed
    }

    /// Apaga TODOS os vértices selecionados (Delete / botão), re-costurando os
    /// vizinhos. Se o path ficar com < 2 vértices, remove o path inteiro e limpa
    /// a seleção. A seleção segue no vizinho do 1º apagado (delete encadeado de um
    /// só). Devolve `true` se apagou algo.
    pub fn delete_selected_vertex(&mut self, scene: &mut VecScene) -> bool {
        let Some(id) = self.selected else {
            return false;
        };
        if self.selected_verts.is_empty() {
            return false;
        }
        let Some(path) = scene.path_mut(id) else {
            return false;
        };
        // Remove do maior índice pro menor pra não invalidar os índices.
        let mut idxs = self.selected_verts.clone();
        idxs.sort_unstable();
        idxs.dedup();
        let lowest = idxs.first().copied().unwrap_or(0);
        for &i in idxs.iter().rev() {
            if i < path.verts.len() {
                path.verts.remove(i);
            }
        }
        let remaining = path.verts.len();
        self.selected_verts.clear();
        if remaining < 2 {
            scene.remove_path(id);
            self.selected = None;
            self.active = None;
        } else if idxs.len() == 1 {
            // Delete de um só: seleção segue no vizinho (delete encadeado).
            self.selected_verts = vec![lowest.min(remaining - 1)];
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
                    path.fill = Some(Paint::solid(self.style.fill));
                    self.active = None;
                    self.dragging = false;
                    return PenClick::Closed;
                }
            }
            path.verts.push(VecVertex::corner(p));
            self.selected_verts = vec![path.verts.len() - 1];
            self.dragging = true;
            return PenClick::Added;
        }

        // Parado → hit-test para EDITAR um ponto existente.
        if let Some(g) = self.hit_test(scene, p, hit_r) {
            self.selected = Some(g.path);
            // Clique simples reduz a seleção de OBJETO ao path tocado (Shift+clique
            // multi-seleção é tratado no shell ANTES do on_press).
            self.selected_paths = vec![g.path];
            // Agarrar uma âncora que JÁ está na multi-seleção de vértice mantém o
            // grupo (o arrasto move todos); qualquer outro grab vira seleção única.
            if g.part == Part::Anchor && self.selected_verts.contains(&g.vert) {
                // mantém a seleção de grupo
            } else {
                self.selected_verts = vec![g.vert];
            }
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
            self.selected_paths = vec![sel];
            self.selected_verts = vec![ni];
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
            stroke: Some(self.style.stroke_spec(stroke_w)),
        });
        self.active = Some(id);
        self.selected = Some(id);
        self.selected_paths = vec![id];
        self.selected_verts = vec![0];
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
            // Arrasto de ÂNCORA: se a agarrada está na multi-seleção, TODAS as
            // âncoras selecionadas transladam pelo mesmo delta (âncora + handles);
            // senão só a agarrada. Handles ficam de fora (são per-vértice).
            if g.part == Part::Anchor {
                let Some(grabbed) = path.verts.get(g.vert) else {
                    return false;
                };
                let d = [p[0] - grabbed.anchor[0], p[1] - grabbed.anchor[1]];
                let group = self.selected_verts.contains(&g.vert);
                let n = path.verts.len();
                for i in 0..n {
                    if i != g.vert && !(group && self.selected_verts.contains(&i)) {
                        continue;
                    }
                    let v = &mut path.verts[i];
                    v.anchor = [v.anchor[0] + d[0], v.anchor[1] + d[1]];
                    v.in_handle = [v.in_handle[0] + d[0], v.in_handle[1] + d[1]];
                    v.out_handle = [v.out_handle[0] + d[0], v.out_handle[1] + d[1]];
                }
                return true;
            }
            let Some(v) = path.verts.get_mut(g.vert) else {
                return false;
            };
            match g.part {
                Part::Anchor => unreachable!("handled above"),
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
mod tests;
