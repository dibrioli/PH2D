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
mod shape_constraint;
pub use shape::{ShapeConstraint, ShapeTool};

/// Pen STYLE + edit HISTORY (sibling module, HR-18 file-LOC cap).
mod pen_support;
pub use pen_support::{History, PenStyle};

/// Selection + the document ops that act on it (sibling module, LOC cap).
mod selection;
pub use selection::SelectedKind;

pub mod snap;
pub use snap::{SnapConfig, SnapResult, SnapTargets, bbox_key_points, collect_targets};

/// A alça do **raio de quina** (Live Corners) — módulo irmão (LOC cap).
pub mod corner_handle;

/// **O press das ferramentas Fillet / Chamfer** (`on_press_corner`) — módulo irmão (LOC cap).
/// Bloco `impl PenTool` inerente; move e release reusam o caminho do pen.
mod corner_tool;
/// **O que um press do modo Node EDITARIA aqui** (`node_edit_hit_at` + a busca do insert) —
/// módulo irmão (LOC cap). Bloco `impl PenTool` inerente.
mod node_hit;
/// **O arrasto** de um ponto agarrado (âncora / handle bézier / alça de raio) — módulo
/// irmão (LOC cap). Bloco `impl PenTool` inerente; a API pública não muda.
mod pen_drag;
use ph2d_vec_scene::{Paint, VecPath, VecPathId, VecScene, VecVertex, VertexKind};

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

/// Parte de um vértice que o hit-test pode agarrar.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Part {
    Anchor,
    In,
    Out,
    /// A alça do **raio de quina** (Live Corners): corre pela bissetriz, e arrastá-la
    /// para dentro arredonda o canto. Só existe onde há quina de verdade —
    /// `corner_at` é o mesmo predicado que o cozimento usa.
    Radius,
}

#[derive(Copy, Clone, Debug)]
struct Grab {
    path: VecPathId,
    vert: usize,
    part: Part,
    /// **O arrasto relativo da alça de raio, num número só:** a diferença entre o recuo
    /// VERDADEIRO e a projeção do cursor na bissetriz, medida no instante do grab. Durante
    /// o arrasto, `recuo = projeção_agora + este offset`.
    ///
    /// É relativo porque a alça de uma quina AFIADA não pode ser desenhada em cima da
    /// âncora (sumiria dentro dela): ela ESTACIONA alguns pixels adiante, sobre a
    /// bissetriz. Com arrasto absoluto, o primeiro pixel de movimento faria o raio saltar
    /// para o valor daquele estacionamento. Com o offset, a quina afiada tem
    /// `offset = 0 − park` e o raio cresce de zero, como o dedo espera. Só o
    /// `Part::Radius` usa.
    radius_offset: f64,
    /// Para um `Part::Radius`: o ESTILO que o arrasto FORÇA — `Some(true)` = chanfro,
    /// `Some(false)` = arredondado. É como as ferramentas Fillet/Chamfer decidem a quina
    /// enquanto o dedo só dita a MAGNITUDE. `None` = **preserva** o estilo atual do vértice
    /// (o arrasto só muda o recuo, mantendo o sinal via `set_corner_size`).
    ///
    /// Precisa ser forçado, e não semeado no vértice antes do arrasto, por causa do
    /// **zero com sinal**: uma quina recém-afiada tem `corner_radius = ±0`, e `-0.0 < 0.0`
    /// é `false` — logo `is_chamfer()` mentiria e o `set_corner_size` do arrasto pintaria o
    /// arredondado. O estilo viaja no grab e é aplicado a cada frame, quando já há magnitude.
    chamfer: Option<bool>,
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
    /// O que a ÁRVORE do editor esconde/trava neste frame (ADR-0110). Publicado
    /// pela shell; um path escondido ou travado não é agarrável.
    view: ph2d_vec_scene::VecViewState,
    /// Onde cada path ESTÁ (ADR-0111): o afim local→mundo vindo do `Transform` da
    /// entidade e da cadeia de pais. Publicado pela shell a cada frame.
    ///
    /// A regra desta ferramenta: **o que o usuário vê, aponta e encaixa é mundo; o
    /// que o documento guarda é local.** A conversão mora só na fronteira, nos
    /// helpers abaixo — nenhum cálculo de geometria muda.
    xforms: ph2d_vec_scene::VecXforms,
}

impl PenTool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Há um traço em progresso (desenhando)?
    pub fn is_drawing(&self) -> bool {
        self.active.is_some()
    }

    /// O path que a caneta está construindo — ainda cresce a cada clique, então a
    /// shell não deve assentar a origem dele.
    pub fn active_path(&self) -> Option<VecPathId> {
        self.active
    }

    /// Está manipulando algo agora (arrasto de desenho OU de edição)?
    pub fn is_dragging(&self) -> bool {
        self.dragging || self.grab.is_some()
    }

    /// As ÂNCORAS que o arrasto atual move: `(path, índices planos)`. `None` quando
    /// não há arrasto de âncora (handle, desenho, ou nada). O snap usa isto para
    /// excluir da lista de alvos as âncoras que estão vindo junto com o cursor.
    #[must_use]
    pub fn dragging_anchors(&self) -> Option<(VecPathId, Vec<usize>)> {
        let g = self.grab?;
        if g.part != Part::Anchor {
            return None;
        }
        let verts = if self.selected_verts.contains(&g.vert) {
            self.selected_verts.clone()
        } else {
            vec![g.vert]
        };
        Some((g.path, verts))
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

    /// Publica o que a árvore esconde/trava. Chamado uma vez por frame pela shell,
    /// antes de qualquer hit-test.
    pub fn set_view(&mut self, view: ph2d_vec_scene::VecViewState) {
        self.view = view;
    }

    /// Publica o afim de cada path (ADR-0111). Chamado com `set_view`, uma vez por
    /// frame, antes de qualquer hit-test.
    pub fn set_xforms(&mut self, xforms: ph2d_vec_scene::VecXforms) {
        self.xforms = xforms;
    }

    /// O afim local→mundo de `id` (identidade se o path não tem entidade ainda).
    fn xf(&self, id: VecPathId) -> ph2d_vec_scene::Xform {
        ph2d_vec_scene::xform_of(&self.xforms, id)
    }

    /// Um PONTO local do path `id`, no mundo.
    fn to_world(&self, id: VecPathId, p: [f64; 2]) -> [f64; 2] {
        self.xf(id).apply(p)
    }

    /// Um PONTO do mundo, no espaço local de `id`. Um afim degenerado (escala 0)
    /// não tem volta: devolve `p` — a forma é invisível e nada será agarrado nela.
    fn to_local(&self, id: VecPathId, p: [f64; 2]) -> [f64; 2] {
        self.xf(id).inverse().map_or(p, |inv| inv.apply(p))
    }

    /// Um DELTA do mundo, no espaço local de `id` (só a parte linear).
    fn delta_to_local(&self, id: VecPathId, d: [f64; 2]) -> [f64; 2] {
        self.xf(id).inverse().map_or(d, |inv| inv.apply_vec(d))
    }

    /// Pressão primária em world-space `p`. `px_to_world` = world-units por pixel.
    /// `alt` = quebrar a tangente ao agarrar um handle (vira cusp / Corner).
    ///
    /// `snap` roda **só onde um ponto é POSICIONADO** (vértice novo do pen, 1º ponto
    /// de um path novo) — nunca antes do hit-test, senão encaixar o cursor mudaria o
    /// que ele agarra; nem no insert-sobre-segmento, cujo ponto tem de ficar na curva.
    /// Sem snap, passe `&mut |p| p`.
    pub fn on_press(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        px_to_world: f64,
        alt: bool,
        snap: &mut dyn FnMut([f64; 2]) -> [f64; 2],
    ) -> PenClick {
        let close_dist = 12.0 * px_to_world;
        let hit_r = 10.0 * px_to_world;
        let stroke_w = self.style.stroke_w_px * px_to_world;

        // Desenhando → continua o pen (adiciona/fecha).
        if let Some(id) = self.active {
            // O ponto de fechamento é comparado no MUNDO (o cursor está lá).
            let first_world = scene
                .paths()
                .iter()
                .find(|pp| pp.id == id)
                .and_then(|pp| pp.verts.first())
                .map(|v| self.to_world(id, v.anchor));
            let snapped = snap(p);
            let anchor_local = self.to_local(id, snapped);
            // Fechar: o cursor voltou ao PRIMEIRO ponto do path ativo (≥3 vértices).
            let close = scene
                .paths()
                .iter()
                .find(|pp| pp.id == id)
                .is_some_and(|pp| pp.verts.len() >= 3)
                && first_world.is_some_and(|fw| {
                    let (dx, dy) = (p[0] - fw[0], p[1] - fw[1]);
                    (dx * dx + dy * dy).sqrt() <= close_dist
                });
            if close {
                let Some(path) = scene.path_mut(id) else {
                    return PenClick::Ignored;
                };
                path.closed = true;
                path.fill = Some(Paint::solid(self.style.fill));
                self.active = None;
                self.dragging = false;
                return PenClick::Closed;
            }
            // Clicar sobre o endpoint de OUTRO path aberto FUNDE os dois num só
            // objeto (Enio 2026-07-09) — a via de construir uma forma fechada com
            // várias linhas/arcos/traços.
            if let Some(click) = self.join_open_path(scene, id, snapped, close_dist) {
                return click;
            }
            let Some(path) = scene.path_mut(id) else {
                self.active = None;
                self.dragging = false;
                return PenClick::Ignored;
            };
            path.verts.push(VecVertex::corner(anchor_local));
            self.selected_verts = vec![path.verts.len() - 1];
            self.dragging = true;
            return PenClick::Added;
        }

        // Parado → CONTINUAR um path aberto pelo seu endpoint tem prioridade: clicar
        // na ponta de uma linha/arco a reativa para seguir desenhando (e, fechando no
        // outro extremo, vira uma forma fechada). Convenção da Pen do Illustrator.
        if self.reopen_endpoint(scene, p, hit_r) {
            return PenClick::Grabbed;
        }
        // hit-test para EDITAR um ponto existente. **Nenhum modo agarra alça de raio aqui:** ela
        // saiu do Node e virou o par de ferramentas Fillet / Chamfer (`corner_tool`), então o
        // arredondar/chanfrar quina tem press próprio. (Este comentário dizia *"ela é do modo
        // NODE"* e contradizia o código shipado 35 linhas abaixo — a auditoria do plano 25 o pegou.)
        if let Some(g) = self.hit_test(scene, p, hit_r) {
            self.grab_vertex(scene, g, alt);
            return PenClick::Grabbed;
        }
        if let Some(click) = self.insert_on_selected_segment(scene, p, hit_r) {
            return click;
        }

        // Nada agarrado → começa um path novo. É a ÚNICA coisa que a caneta faz e a
        // edição de nós não (ADR-0112): `on_press_node` para antes desta linha.
        let id = scene.push_path(VecPath {
            verts: vec![VecVertex::corner(snap(p))],
            stroke: Some(self.style.stroke_spec(stroke_w)),
            ..VecPath::default()
        });
        self.active = Some(id);
        self.selected = Some(id);
        self.selected_paths = vec![id];
        self.selected_verts = vec![0];
        self.dragging = true;
        PenClick::Started
    }

    /// Pressão primária no modo **Node** (seta branca): edita âncoras e handles, e
    /// **nunca cria um path**.
    ///
    /// A ordem é a do Illustrator: agarra um handle/âncora; senão insere um vértice
    /// no segmento do path selecionado; senão seleciona a forma sob o cursor (o
    /// clique no preenchimento acende as âncoras dela); senão desseleciona.
    ///
    /// Não há gizmo neste modo — as alças dele comeriam o clique do nó — então o
    /// clique no interior de uma forma nunca a transforma, só a seleciona.
    ///
    /// Arredondar/chanfrar quina NÃO é mais deste modo — virou o par de ferramentas
    /// Fillet/Chamfer (`on_press_corner`), então o hit-test aqui não agarra alça de raio.
    pub fn on_press_node(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        px_to_world: f64,
        alt: bool,
    ) -> PenClick {
        let hit_r = crate::node_hit::NODE_HIT_PX * px_to_world;
        if let Some(g) = self.hit_test(scene, p, hit_r) {
            self.grab_vertex(scene, g, alt);
            return PenClick::Grabbed;
        }
        if let Some(click) = self.insert_on_selected_segment(scene, p, hit_r) {
            return click;
        }
        match self.path_at(scene, p, hit_r) {
            Some(id) => {
                self.select(Some(id));
                PenClick::Grabbed
            }
            None => {
                self.select(None);
                PenClick::Ignored
            }
        }
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
    /// Se `p` (mundo) cai num ENDPOINT (1º ou último vértice) de um path ABERTO,
    /// reabre-o para continuar desenhando dele. O cabeçote de desenho é sempre o
    /// ÚLTIMO vértice, então clicar no 1º **reverte** o path (e troca in/out handles,
    /// que também invertem de papel). É o que permite continuar uma linha de onde
    /// parou e fechá-la numa forma (Enio 2026-07-09). `false` se nenhum endpoint.
    fn reopen_endpoint(&mut self, scene: &mut VecScene, p: [f64; 2], hit_r: f64) -> bool {
        let r2 = hit_r * hit_r;
        let found = scene.paths().iter().find_map(|path| {
            if path.closed || path.verts.len() < 2 || !self.view.is_pickable(path.id) {
                return None;
            }
            let xf = self.xf(path.id);
            // Endpoints são só do contorno primário (`verts`), em world.
            if dist2(p, xf.apply(path.verts.last()?.anchor)) <= r2 {
                return Some((path.id, false));
            }
            (dist2(p, xf.apply(path.verts.first()?.anchor)) <= r2).then_some((path.id, true))
        });
        let Some((id, is_first)) = found else {
            return false;
        };
        if is_first && let Some(path) = scene.path_mut(id) {
            path.verts.reverse();
            for v in &mut path.verts {
                std::mem::swap(&mut v.in_handle, &mut v.out_handle);
            }
        }
        let last_idx = scene
            .paths()
            .iter()
            .find(|pp| pp.id == id)
            .map_or(0, |pp| pp.verts.len().saturating_sub(1));
        self.active = Some(id);
        self.selected = Some(id);
        self.selected_paths = vec![id];
        self.selected_verts = vec![last_idx];
        true
    }

    /// Funde OUTRO path aberto `B` no path ativo `active` quando `p` (mundo) cai num
    /// endpoint de B: anexa os vértices de B a A e remove B — os dois viram um só
    /// objeto (Enio 2026-07-09). Se o endpoint tocado é o ÚLTIMO de B, B é revertido
    /// antes (o costura segue contínua). `None` se nenhum endpoint de outro path.
    ///
    /// A geometria de B está no espaço LOCAL de B; sobe ao mundo pelo afim de B e
    /// desce ao espaço local de A — a mesma regra de fronteira do resto da tool.
    fn join_open_path(
        &mut self,
        scene: &mut VecScene,
        active: VecPathId,
        p_world: [f64; 2],
        hit_r: f64,
    ) -> Option<PenClick> {
        let r2 = hit_r * hit_r;
        let (bid, b_first) = scene.paths().iter().find_map(|path| {
            if path.id == active || path.closed || path.verts.is_empty() {
                return None;
            }
            if !self.view.is_pickable(path.id) {
                return None;
            }
            let xf = self.xf(path.id);
            if dist2(p_world, xf.apply(path.verts.first()?.anchor)) <= r2 {
                Some((path.id, true))
            } else if dist2(p_world, xf.apply(path.verts.last()?.anchor)) <= r2 {
                Some((path.id, false))
            } else {
                None
            }
        })?;
        // Vértices de B em MUNDO, na ordem de junção.
        let xf_b = self.xf(bid);
        let mut b_world: Vec<VecVertex> = scene
            .paths()
            .iter()
            .find(|pp| pp.id == bid)?
            .verts
            .iter()
            .map(|v| VecVertex {
                anchor: xf_b.apply(v.anchor),
                in_handle: xf_b.apply(v.in_handle),
                out_handle: xf_b.apply(v.out_handle),
                kind: v.kind,
                // O raio é um COMPRIMENTO no espaço local, e aqui B sobe para o mundo:
                // escala com a geometria (e desce de novo para o local de A, abaixo).
                corner_radius: v.corner_radius * xf_b.mean_scale(),
            })
            .collect();
        if !b_first {
            // Tocou o ÚLTIMO de B → percorre B de trás pra frente; in/out trocam de
            // papel ao inverter o sentido.
            b_world.reverse();
            for v in &mut b_world {
                std::mem::swap(&mut v.in_handle, &mut v.out_handle);
            }
        }
        let inv_a = self.xf(active).inverse()?;
        let a = scene.path_mut(active)?;
        for v in b_world {
            a.verts.push(VecVertex {
                anchor: inv_a.apply(v.anchor),
                in_handle: inv_a.apply(v.in_handle),
                out_handle: inv_a.apply(v.out_handle),
                kind: v.kind,
                corner_radius: v.corner_radius * inv_a.mean_scale(),
            });
        }
        let last = a.verts.len().saturating_sub(1);
        scene.remove_path(bid);
        self.selected_verts = vec![last];
        self.dragging = false;
        Some(PenClick::Added)
    }

    /// Agarra `g` para arrastar: reduz a seleção de objeto ao path tocado e a de
    /// vértice ao vértice tocado (salvo quando ele já estava num grupo selecionado,
    /// caso em que o arrasto move o grupo).
    ///
    /// A expansão da seleção para o GRUPO é da shell (a árvore é a Hierarquia,
    /// ADR-0110): ela chama `set_object_selection` logo depois, se houver grupo.
    fn grab_vertex(&mut self, scene: &mut VecScene, g: Grab, alt: bool) {
        self.selected = Some(g.path);
        self.selected_paths = vec![g.path];
        if !(g.part == Part::Anchor && self.selected_verts.contains(&g.vert)) {
            self.selected_verts = vec![g.vert];
        }
        // Alt + agarrar um HANDLE = quebrar a tangente (vira cusp Corner) antes de
        // arrastar → só esse handle se move (regra Corner). Convenção Illustrator;
        // o undo cobre break+drag num passo (begin no press).
        if alt
            && matches!(g.part, Part::In | Part::Out)
            && let Some(path) = scene.path_mut(g.path)
        {
            let _ = ph2d_vec_scene::retype_vertex(path, g.vert, VertexKind::Corner);
        }
        self.grab = Some(g);
    }

    /// O raio `r` é WORLD-units (px × zoom), então a comparação acontece no mundo:
    /// cada ponto local sobe pelo afim do path dele. Assim uma forma escalada a 10×
    /// não fica com um raio de captura 10× maior — o alvo é do tamanho que se vê.
    ///
    /// Agarra handle Bézier (do path selecionado) e depois ÂNCORA (de qualquer path
    /// agarrável). A antiga alça de raio na bissetriz saiu com o modo Node de arredondar —
    /// o gesto de quina agora é das ferramentas Fillet/Chamfer (`on_press_corner`).
    fn hit_test(&self, scene: &VecScene, p: [f64; 2], r: f64) -> Option<Grab> {
        let r2 = r * r;
        if let Some(sel) = self.selected
            && let Some(path) = scene.paths().iter().find(|pp| pp.id == sel)
        {
            let xf = self.xf(sel);
            // Handles agarráveis na posição de EXIBIÇÃO (`ghost_handle`): a alça real
            // quando deslocada; ou o TOCO sintético de um ponto Smooth/Symmetric cuja
            // alça é zero (a "alça lateral" antes invisível) — assim ela é agarrável
            // sem a curva ter mudado (o `on_drag` é que escreve a alça real). A quina
            // de um Corner não sintetiza toco. Handle tem prioridade sobre âncora.
            for i in 0..path.total_verts() {
                for (out, part) in [(false, Part::In), (true, Part::Out)] {
                    if let Some(h) = ph2d_vec_scene::ghost_handle(path, i, out)
                        && dist2(p, xf.apply(h)) <= r2
                    {
                        return Some(Grab {
                            path: sel,
                            vert: i,
                            part,
                            radius_offset: 0.0,
                            chamfer: None,
                        });
                    }
                }
            }
        }
        for path in scene.paths() {
            // Escondido ou travado (por si ou por um ancestral) não é agarrável.
            if !self.view.is_pickable(path.id) {
                continue;
            }
            let xf = self.xf(path.id);
            for (i, v) in path.verts_all().enumerate() {
                if dist2(p, xf.apply(v.anchor)) <= r2 {
                    return Some(Grab {
                        path: path.id,
                        vert: i,
                        part: Part::Anchor,
                        radius_offset: 0.0,
                        chamfer: None,
                    });
                }
            }
        }
        None
    }
}

fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests;

/// Gates das ferramentas Fillet / Chamfer (`on_press_corner` + arrasto): arredonda,
/// chanfra, e transforma um ponto suave em quina primeiro.
#[cfg(test)]
#[path = "corner_tool_tests.rs"]
mod corner_tool_tests;
