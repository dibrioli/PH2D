//! ⭐ **As alças no modo FORMA** (F10, pedido do dono 11/10): *«ao criar uma forma, os controles
//! devem aparecer imediatamente […] ainda no modo shape; no modo select não devem aparecer»* e *«ao
//! clicar nos controles, eles têm a preferência em relação a criar nova forma»*. Cada gate corre o
//! evento real (o gesto da ferramenta, as portas `on::*` que a shell chama) e lê o efeito.

use super::*;
use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_vector::VectorTool;
use ph2d_vec_edit::ShapeConstraint;

const PX: f64 = 0.01;

struct Mesa {
    vec: VecState,
    sim: SimWorld,
    scene: VecScene,
    tools: ToolRegistry,
    store: WidgetStore,
}

fn mesa() -> Mesa {
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(VectorTool::default()));
    let mut store = WidgetStore::default();
    <ph2d_panel_vector::VectorPanel as ph2d_editor_core::panel::Panel>::populate(&mut store);
    let mut vec = VecState::default();
    vec.draw_config.mode = DrawMode::Shape;
    Mesa {
        vec,
        sim: SimWorld::default(),
        scene: VecScene::new(),
        tools,
        store,
    }
}

impl Mesa {
    /// O clique numa forma do CATÁLOGO (arma o gesto: o latch acende no quadro seguinte).
    fn catalog(&mut self, kind: ShapeKind) {
        self.tools
            .tool_by_id_mut(&ToolId::new("vector"))
            .and_then(|x| x.as_any_mut().downcast_mut::<VectorTool>())
            .expect("a ferramenta de vetor")
            .set_shape(kind);
    }

    /// Um quadro: a entidade nasce (`sync`), a forma fica viva e o quadro dos campos corre.
    fn frame(&mut self) {
        ph2d_vec_entities::entities::sync(&mut self.sim, &mut self.scene, &mut self.vec.entities);
        crate::shape_live::make_committed_shape_live(
            &mut self.sim,
            &mut self.scene,
            &self.vec.entities,
            &mut self.vec.shape,
            false,
        );
        crate::shape_params::frame_fields(
            &mut self.vec,
            &self.sim,
            &mut self.tools,
            &mut self.store,
            PX,
        );
        // As posições dos objectos, como a shell as dá à caneta a cada quadro (o pick lê-as).
        let xf = ph2d_vec_entities::transform::build(&self.sim, &self.vec.entities);
        self.vec.pen.set_xforms(xf);
    }

    /// O gesto da ferramenta de forma (press, arrasto, soltar) e a seleção da forma nova — o que a
    /// shell faz no Up —, e o quadro seguinte. Devolve o id da forma nova.
    fn draw(&mut self, kind: ShapeKind, at: [f64; 2], half: f64) -> VecPathId {
        let c = ShapeConstraint::default();
        let (a, b) = ([at[0] - half, at[1] - half], [at[0] + half, at[1] + half]);
        let shape = &mut self.vec.shape;
        shape.on_press(&mut self.scene, kind, kind.defaults(), a, PX, c);
        shape.on_drag(&mut self.scene, b, c);
        assert!(shape.on_release(&mut self.scene), "o gesto não fez forma");
        crate::shape_live::after_the_shape_release(&mut self.vec, &self.scene, true);
        let id = self
            .vec
            .pen
            .selected()
            .expect("desenhar não seleccionou a forma nova");
        self.frame();
        id
    }

    fn view(&self) -> Option<ShapeHandlesView> {
        let sel = self.vec.pen.selected_paths();
        view(
            &self.sim,
            &self.vec.entities,
            sel,
            Gate::of(&self.vec),
            PX,
            None,
            None,
        )
    }

    fn press(&mut self, w: [f64; 2]) -> bool {
        on::press(&mut self.vec, &mut self.sim, &mut self.scene, w, PX, 0)
    }

    fn values(&self, id: VecPathId) -> ShapeValues {
        crate::shape_params::panel_shape_target(&self.sim, &self.vec.entities, &[id])
            .expect("forma viva")
            .3
    }
}

fn centroid(points: &[[f64; 2]]) -> [f64; 2] {
    let n = points.len() as f64;
    let s = points
        .iter()
        .fold([0.0, 0.0], |a, p| [a[0] + p[0], a[1] + p[1]]);
    [s[0] / n, s[1] / n]
}

/// ⭐ **Desenhar no modo Forma acende as alças da forma NOVA, já** — e armar outra no catálogo
/// apaga-as (os campos e as alças falam da forma armada: nenhuma alça); desenhar outra vez traz as
/// da forma acabada de desenhar, não as da anterior.
#[test]
fn drawing_in_shape_mode_shows_the_new_shapes_handles_at_once() {
    let mut m = mesa();
    let star = m.draw(ShapeKind::Star, [-3.0, 0.0], 1.0);
    let v = m
        .view()
        .expect("a estrela acabada de desenhar não tem alças");
    assert_eq!(v.points.len(), law::handles_of(ShapeKind::Star).len());
    assert_eq!(m.vec.pen.selected_paths(), &[star]);
    // Armar o Polígono no catálogo (com a estrela ainda seleccionada): nenhuma alça.
    m.catalog(ShapeKind::Polygon);
    m.frame();
    assert!(m.vec.shape_armed, "o clique no catálogo não armou");
    assert!(
        m.view().is_none(),
        "armado no catálogo, a estrela ficou com alças"
    );
    // Desenhar o polígono: o latch cai e as alças são DELE, onde ele está.
    let poly = m.draw(ShapeKind::Polygon, [3.0, 0.0], 1.0);
    assert!(!m.vec.shape_armed, "a forma nova não apagou o latch");
    let v = m
        .view()
        .expect("o polígono acabado de desenhar não tem alças");
    assert_eq!(v.points.len(), law::handles_of(ShapeKind::Polygon).len());
    let c = centroid(&v.points);
    assert!(
        (c[0] - 3.0).abs() < 1.0 && c[1].abs() < 1.0,
        "as alças não são as do polígono: centro {c:?}"
    );
    assert_ne!(star, poly);
}

impl Mesa {
    /// Um CLIQUE da ferramenta de forma (press e soltar no mesmo ponto) e o que a shell faz no Up.
    fn click(&mut self, at: [f64; 2]) {
        let shape = &mut self.vec.shape;
        shape.on_press(
            &mut self.scene,
            ShapeKind::Ellipse,
            ShapeKind::Ellipse.defaults(),
            at,
            PX,
            ShapeConstraint::default(),
        );
        let committed = shape.on_release(&mut self.scene);
        assert!(!committed, "um clique fez forma");
        crate::shape_live::after_the_shape_release(&mut self.vec, &self.scene, committed);
    }
}

/// ⭐ **No modo Forma, um CLIQUE numa forma que já existe SELECCIONA-a** (o dono, 11/10: «se a
/// ferramenta estiver em shape, deve ser possível selecionar shapes já colocadas no canvas»); no
/// vazio a selecção fica; e ARRASTAR por cima de uma forma continua a desenhar (o Affinity).
#[test]
fn a_click_in_shape_mode_selects_the_shape_under_it_and_a_drag_still_draws() {
    let mut m = mesa();
    let star = m.draw(ShapeKind::Star, [0.0, 0.0], 1.0);
    let disc = m.draw(ShapeKind::Ellipse, [4.0, 0.0], 1.0);
    assert_eq!(m.vec.pen.selected(), Some(disc));
    let paths = m.scene.paths().len();
    m.click([0.0, 0.0]);
    let fill = m
        .scene
        .paths()
        .iter()
        .find(|p| p.id == star)
        .map(|p| p.fill.is_some());
    let hit = m.vec.pen.path_at(&m.scene, [0.0, 0.0], 0.12);
    let bounds = m.vec.shape.bounds();
    assert_eq!(
        m.vec.pen.selected(),
        Some(star),
        "o clique na estrela não a seleccionou (fill {fill:?}, pick {hit:?}, estrela {star:?}, disco {disc:?}, caixa {bounds:?})"
    );
    assert_eq!(m.scene.paths().len(), paths, "o clique deixou uma forma");
    m.click([20.0, 20.0]);
    assert_eq!(
        m.vec.pen.selected(),
        Some(star),
        "o clique no vazio mudou a selecção"
    );
    let over = m.draw(ShapeKind::Ellipse, [0.0, 0.0], 0.5);
    assert!(
        over != star && m.scene.paths().len() == paths + 1,
        "arrastar por cima não desenhou"
    );
}

/// ⭐ **O press numa alça no modo Forma arrasta-a e não desenha** (a porta da shell consome-o: a
/// ferramenta de forma não chega a ver o press); no VAZIO a porta não consome, e o desenho segue.
#[test]
fn a_press_on_a_handle_in_shape_mode_drags_it_and_a_press_on_empty_canvas_draws() {
    let mut m = mesa();
    let star = m.draw(ShapeKind::Star, [0.0, 0.0], 1.0);
    let inner = law::handles_of(ShapeKind::Star)
        .iter()
        .position(|h| h.fields.first() == Some(&1))
        .expect("o vale da estrela");
    let p = m.view().expect("alças").points[inner];
    let (before, paths) = (m.values(star)[1], m.scene.paths().len());
    assert!(
        m.press(p),
        "o press na alça não foi consumido — a forma nova nasceria"
    );
    assert!(
        !m.vec.shape.is_active(),
        "a ferramenta de forma viu o press"
    );
    let out = [p[0] * 1.6, p[1] * 1.6];
    assert!(on::drag(
        &mut m.vec,
        &mut m.sim,
        &mut m.scene,
        out,
        Mods::default(),
        PX
    ));
    assert!(m.vec.shape_handles.release());
    assert!(m.values(star)[1] > before + 0.05, "o vale não andou");
    assert_eq!(m.scene.paths().len(), paths, "nasceu uma forma");
    // O vazio: a porta não consome — a shell segue para o desenho, e a forma nasce.
    assert!(
        !m.press([4.0, 4.0]),
        "o press no vazio foi engolido pelas alças"
    );
    m.draw(ShapeKind::Ellipse, [4.0, 4.0], 0.5);
    assert_eq!(m.scene.paths().len(), paths + 1);
}

/// ⭐ **No Select não há alças de forma** — nem desenhadas nem agarráveis (a inversão da F9). Controlo:
/// as mesmas alças no modo Forma.
#[test]
fn in_select_mode_no_shape_handle_is_drawn_or_grabbed() {
    let mut m = mesa();
    let star = m.draw(ShapeKind::Star, [0.0, 0.0], 1.0);
    let p = m.view().expect("controlo: no modo Forma há alças").points[0];
    m.vec.draw_config.mode = DrawMode::Select;
    assert!(m.view().is_none(), "o Select desenhou alças de forma");
    let before = m.values(star);
    assert!(!m.press(p), "o Select agarrou uma alça de forma");
    assert!(m.vec.shape_handles.grab.is_none());
    assert_eq!(m.values(star), before);
}

/// ⭐ **Onde uma alça da FORMA e uma da CAIXA coincidem, ganha a da forma** (a regra da F9, «dentro
/// do raio dela a alça ganha», mantida no modo Forma: a shell chama a porta das alças ANTES do gizmo e
/// com `over_canvas_or_gizmo`, que aceita o hit da caixa). Varre o catálogo à procura das alças que
/// caem sobre uma alça da caixa (cantos e meios das arestas do rectângulo autorado) e prime NO PONTO
/// DA CAIXA: a alça da forma tem de ser agarrada. Controlo: pelo menos uma coincidência existe.
#[test]
fn where_a_shape_handle_meets_a_box_handle_the_shape_handle_wins() {
    let mut hits = 0;
    for &kind in ph2d_vec_scene::ALL_SHAPES {
        if law::handles_of(kind).is_empty() {
            continue;
        }
        let mut m = mesa();
        let (at, half) = ([0.0, 0.0], 1.0);
        m.draw(kind, at, half);
        let Some(v) = m.view() else { continue };
        let r = HIT_PX * PX;
        let caixa = [-1.0, 0.0, 1.0]
            .iter()
            .flat_map(|&x| [-1.0, 0.0, 1.0].map(|y| [at[0] + x * half, at[1] + y * half]))
            .filter(|p| *p != at);
        for b in caixa {
            if !v
                .points
                .iter()
                .any(|p| (p[0] - b[0]).hypot(p[1] - b[1]) <= r)
            {
                continue;
            }
            hits += 1;
            assert!(
                m.press(b),
                "{kind:?}: a alça da forma sobre a alça da caixa em {b:?} não ganhou o press"
            );
            assert!(m.vec.shape_handles.release());
        }
    }
    assert!(
        hits > 0,
        "controlo: nenhuma alça de forma caiu sobre uma da caixa"
    );
}

/// ⭐ **Com as alças à vista, os NÓS da forma somem** (a foto da cena 16: as âncoras laranja e as
/// alças bézier azuis tapavam as verdes). Só no modo Forma com a forma viva como alvo: no Select,
/// armado no catálogo, ou sem forma viva, a pergunta responde `false` e o overlay de nós é o de
/// sempre.
#[test]
fn the_nodes_hide_only_while_the_shape_handles_show() {
    let mut m = mesa();
    m.draw(ShapeKind::Star, [0.0, 0.0], 1.0);
    assert!(
        hides_the_nodes(&m.vec, &m.sim),
        "modo Forma, a estrela viva: sem nós"
    );
    m.vec.draw_config.mode = DrawMode::Select;
    assert!(
        !hides_the_nodes(&m.vec, &m.sim),
        "no Select a pergunta nem se põe"
    );
    m.vec.draw_config.mode = DrawMode::Shape;
    m.catalog(ShapeKind::Polygon);
    m.frame();
    assert!(
        !hides_the_nodes(&m.vec, &m.sim),
        "armado no catálogo: sem alças, nós de sempre"
    );
    m.vec.pen.select_many(&[]);
    m.frame();
    assert!(!hides_the_nodes(&m.vec, &m.sim), "nada seleccionado");
}
