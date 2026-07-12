//! `ShapeTool` — desenho de primitiva por arrasto (ADR-0108). Irmão do
//! [`PenTool`](crate::PenTool): a shell roteia Down/Move/Up do canvas para cá quando o
//! modo de desenho da tool Vector é uma **forma** em vez da caneta.
//!
//! O gesto é o padrão: **press** ancora um canto e empurra um path vivo (degenerado) na
//! cena; **drag** o redimensiona contra a caixa; **release** o mantém (se passou de
//! alguns px) ou o remove (clique perdido → cancela). Como o path vive na cena o tempo
//! todo, o render já desenha o preview de graça — sem overlay.
//!
//! **A forma é DADO.** Este módulo não conhece retângulo, estrela ou seta: ele carrega
//! um [`ShapeKind`] + os [`ShapeValues`] e chama `ph2d_vec_scene::cook`. Uma forma nova
//! entra no catálogo e passa a ser desenhável aqui sem uma linha de código. O que mora
//! aqui é só a máquina de estados + o estilo (o documento e o undo ficam na shell).

use ph2d_vec_scene::{ShapeKind, ShapeValues, VecPath, VecPathId, VecScene};

use crate::PenStyle;
pub use crate::shape_constraint::ShapeConstraint;
use crate::shape_constraint::constrained_rect;

/// Um arrasto abaixo disto (em **pixels de tela**) nos dois eixos é um clique perdido: a
/// forma é descartada no release, para um clique errado não sujar a cena de paths de
/// tamanho zero.
const MIN_DRAG_PX: f64 = 3.0;

/// Desenho de forma por arrasto. Só máquina de estados — o documento (`VecScene`) e o
/// undo (`History`) vivem na shell, exatamente como no [`PenTool`](crate::PenTool).
#[derive(Default)]
pub struct ShapeTool {
    /// Estilo da forma sendo desenhada (a shell o sincroniza da tool a cada frame).
    style: PenStyle,
    /// O path vivo sendo dimensionado (entre press e release); `None` = ocioso.
    active: Option<VecPathId>,
    kind: ShapeKind,
    /// Os parâmetros da forma em **MUNDO**, na ordem do catálogo (a shell converte da
    /// unidade de UI antes de chamar — a geometria só fala mundo).
    values: ShapeValues,
    /// O canto pressionado, em mundo.
    start: [f64; 2],
    /// A última posição CRUA do cursor (= `start` até o 1º Move). O retângulo autorado
    /// sai dela pelas restrições ([`Self::bounds`]) — nunca a use direto.
    cur: [f64; 2],
    /// Shift / Alt correntes (o dispatch as espelha a cada Move e a cada mudança de
    /// modificador, então o preview reage na hora).
    constraint: ShapeConstraint,
    /// Unidades de mundo por pixel de tela, capturadas no press — dimensiona o traço e o
    /// limiar de clique-perdido.
    px_to_world: f64,
    /// A última forma commitada (a shell a seleciona, para já sair editável).
    committed: Option<VecPathId>,
}

impl ShapeTool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Estilo aplicado às formas desenhadas a seguir (a shell sincroniza da tool a cada
    /// frame, espelho do `PenTool::set_style`).
    pub fn set_style(&mut self, style: PenStyle) {
        self.style = style;
    }

    /// Há um arrasto de forma em curso?
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// A última forma commitada (para a shell selecionar no release).
    #[must_use]
    pub fn selected(&self) -> Option<VecPathId> {
        self.committed
    }

    /// O tipo da última forma desenhada (a shell a registra como forma VIVA).
    #[must_use]
    pub fn kind(&self) -> ShapeKind {
        self.kind
    }

    /// Os parâmetros (MUNDO) da última forma desenhada — a shell os grava no `VecShape`.
    #[must_use]
    pub fn values(&self) -> ShapeValues {
        self.values
    }

    /// O retângulo AUTORADO do último gesto, **já com Shift/Alt aplicados** — a shell
    /// tira daqui o `w`/`h` (com sinal) e o centro da forma viva. Tem de ser o mesmo
    /// retângulo que o [`Self::build`] desenhou, senão a forma VIVA nasceria diferente
    /// do que o usuário viu no preview.
    #[must_use]
    pub fn bounds(&self) -> ([f64; 2], [f64; 2]) {
        constrained_rect(self.start, self.cur, self.kind, self.constraint)
    }

    /// Unidades de mundo por pixel de tela capturadas no press (dimensiona o traço e o
    /// limiar de clique-perdido). Os PARÂMETROS já chegam em mundo — não passam por aqui.
    #[must_use]
    pub fn px_to_world(&self) -> f64 {
        self.px_to_world
    }

    /// O path que o arrasto está construindo. Ele é reescrito em coordenadas de MUNDO a
    /// cada Move, então a shell não pode assentar a origem dele no meio do gesto — a
    /// geometria e o `Transform` ficariam somando (ADR-0112).
    #[must_use]
    pub fn active_path(&self) -> Option<VecPathId> {
        self.active
    }

    /// Começa uma forma no ponto `p` (mundo): empurra um path degenerado (que já
    /// renderiza) e registra kind + valores + `px_to_world`. Devolve o id novo.
    pub fn on_press(
        &mut self,
        scene: &mut VecScene,
        kind: ShapeKind,
        values: ShapeValues,
        p: [f64; 2],
        px_to_world: f64,
        constraint: ShapeConstraint,
    ) -> VecPathId {
        if let Some(id) = self.active.take() {
            scene.remove_path(id);
        }
        self.kind = kind;
        self.values = values;
        self.start = p;
        self.cur = p;
        self.px_to_world = px_to_world;
        self.constraint = constraint;
        let id = scene.push_path(self.build());
        self.active = Some(id);
        id
    }

    /// Redimensiona a forma ativa contra a caixa `start..p` (com Shift/Alt). `true` se
    /// consumiu o move (há um arrasto vivo).
    pub fn on_drag(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        constraint: ShapeConstraint,
    ) -> bool {
        self.cur = p;
        self.constraint = constraint;
        self.rebuild(scene)
    }

    /// Shift/Alt mudaram SEM mexer o mouse: reconstrói a forma no lugar, para o preview
    /// reagir na hora (sem isto o usuário aperta Shift e nada acontece até mover).
    /// `true` se havia um gesto vivo.
    pub fn set_constraint(&mut self, scene: &mut VecScene, constraint: ShapeConstraint) -> bool {
        if self.constraint == constraint {
            return self.active.is_some();
        }
        self.constraint = constraint;
        self.rebuild(scene)
    }

    /// Reescreve a geometria do path em gesto a partir do estado corrente.
    fn rebuild(&mut self, scene: &mut VecScene) -> bool {
        let Some(id) = self.active else {
            return false;
        };
        let rebuilt = self.build();
        let Some(path) = scene.path_mut(id) else {
            self.active = None;
            return false;
        };
        path.verts = rebuilt.verts;
        path.closed = rebuilt.closed;
        path.fill = rebuilt.fill;
        path.stroke = rebuilt.stroke;
        true
    }

    /// Encerra o arrasto: mantém a forma se ela vence [`MIN_DRAG_PX`] em algum eixo (e a
    /// registra como a seleção commitada), senão a remove (cancela). `true` sse commitou.
    pub fn on_release(&mut self, scene: &mut VecScene) -> bool {
        let Some(id) = self.active.take() else {
            return false;
        };
        let min = MIN_DRAG_PX * self.px_to_world;
        let keep = scene
            .paths()
            .iter()
            .find(|pp| pp.id == id)
            .is_some_and(|pp| bbox_span(pp) >= min);
        if keep {
            self.committed = Some(id);
            true
        } else {
            scene.remove_path(id);
            false
        }
    }

    /// Descarta a forma em curso (troca de tool / clique secundário / Esc).
    pub fn cancel(&mut self, scene: &mut VecScene) {
        if let Some(id) = self.active.take() {
            scene.remove_path(id);
        }
    }

    /// A forma desenhada sobre o retângulo **efetivo** do gesto (Shift/Alt já aplicados),
    /// estilizada. A geometria vem do `cook` do CATÁLOGO — a mesma porta que o re-cook da
    /// forma viva usa, então o preview do arrasto e o objeto que nasce dele nunca divergem.
    fn build(&self) -> VecPath {
        let (a, b) = self.bounds();
        let mut path = ph2d_vec_scene::cook(self.kind, a, b, &self.values);
        let stroke_w = self.style.stroke_w_px * self.px_to_world;
        // Forma ABERTA (linha / arco / espiral) nunca tem fill — não há interior.
        path.fill = if !self.kind.is_closed() || self.style.fill.a == 0 {
            None
        } else {
            Some(ph2d_vec_scene::Paint::solid(self.style.fill))
        };
        path.stroke = Some(self.style.stroke_spec(stroke_w));
        path
    }
}

/// A maior extensão da bbox de âncoras do path (para o limiar de clique-perdido).
fn bbox_span(p: &VecPath) -> f64 {
    let (mut minx, mut miny) = (f64::MAX, f64::MAX);
    let (mut maxx, mut maxy) = (f64::MIN, f64::MIN);
    for v in &p.verts {
        minx = minx.min(v.anchor[0]);
        maxx = maxx.max(v.anchor[0]);
        miny = miny.min(v.anchor[1]);
        maxy = maxy.max(v.anchor[1]);
    }
    (maxx - minx).max(maxy - miny)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::{ALL_SHAPES, VertexKind};

    const PTW: f64 = 0.01; // unidades de mundo por pixel (câmera falsa)

    const SHIFT: ShapeConstraint = ShapeConstraint {
        uniform: true,
        from_center: false,
    };

    fn draw(kind: ShapeKind, values: ShapeValues, to: [f64; 2]) -> VecScene {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            kind,
            values,
            [0.0, 0.0],
            PTW,
            ShapeConstraint::default(),
        );
        shape.on_drag(&mut scene, to, ShapeConstraint::default());
        shape.on_release(&mut scene);
        scene
    }

    /// **Gate de cobertura:** TODA forma do catálogo é desenhável pelo gesto — press,
    /// drag, release — e sai estilizada (fechada ganha fill; aberta, nunca). Uma forma
    /// nova que esqueça o `cook` (ou o `is_closed`) fica vermelha aqui, não na tela.
    #[test]
    fn every_catalog_shape_draws_through_the_gesture() {
        for &k in ALL_SHAPES {
            let scene = draw(k, k.defaults(), [4.0, 3.0]);
            assert_eq!(scene.paths().len(), 1, "{k:?}: nao commitou");
            let p = &scene.paths()[0];
            assert!(!p.verts.is_empty(), "{k:?}: geometria vazia");
            assert!(p.stroke.is_some(), "{k:?}: sem traco");
            assert_eq!(p.closed, k.is_closed(), "{k:?}: fechamento errado");
            assert_eq!(
                p.fill.is_some(),
                k.is_closed(),
                "{k:?}: so forma fechada tem interior"
            );
        }
    }

    #[test]
    fn rectangle_press_drag_release_commits_a_selected_closed_path() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Rectangle,
            ShapeKind::Rectangle.defaults(),
            [0.0, 0.0],
            PTW,
            ShapeConstraint::default(),
        );
        assert!(shape.is_active());
        shape.on_drag(&mut scene, [4.0, 3.0], ShapeConstraint::default());
        assert!(shape.on_release(&mut scene), "um arrasto real commita");
        assert!(!shape.is_active());
        let p = &scene.paths()[0];
        assert!(p.closed && p.verts.len() == 4);
        assert_eq!(shape.selected(), Some(p.id));
    }

    #[test]
    fn tiny_drag_is_discarded_on_release() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Rectangle,
            ShapeKind::Rectangle.defaults(),
            [0.0, 0.0],
            PTW,
            ShapeConstraint::default(),
        );
        shape.on_drag(&mut scene, [0.01, 0.01], ShapeConstraint::default());
        assert!(!shape.on_release(&mut scene), "clique perdido cancela");
        assert!(scene.is_empty(), "forma cancelada nao deixa path");
    }

    #[test]
    fn ellipse_uses_smooth_verts_polygon_uses_corners() {
        let scene = draw(
            ShapeKind::Ellipse,
            ShapeKind::Ellipse.defaults(),
            [4.0, 2.0],
        );
        assert!(
            scene.paths()[0]
                .verts
                .iter()
                .all(|v| v.kind == VertexKind::Smooth)
        );

        let mut v = ShapeKind::Polygon.defaults();
        v[0] = 7.0; // lados
        let scene2 = draw(ShapeKind::Polygon, v, [4.0, 4.0]);
        let p = &scene2.paths()[0];
        assert_eq!(p.verts.len(), 7);
        assert!(p.verts.iter().all(|v| v.kind == VertexKind::Corner));
    }

    /// Os parâmetros chegam em MUNDO: o canto redondo do polígono divide cada quina em
    /// duas âncoras, e a estrela arredonda pontas e vales por raios independentes.
    #[test]
    fn corner_radius_rounds_the_polygon_and_the_star() {
        let mut v = ShapeKind::Polygon.defaults();
        v[0] = 5.0;
        v[1] = 0.3; // raio de canto (mundo)
        assert_eq!(
            draw(ShapeKind::Polygon, v, [4.0, 4.0]).paths()[0]
                .verts
                .len(),
            10
        );

        let mut s = ShapeKind::Star.defaults();
        s[0] = 5.0; // pontas
        s[2] = 0.2; // so as PONTAS arredondam
        s[3] = 0.0;
        assert_eq!(
            draw(ShapeKind::Star, s, [4.0, 4.0]).paths()[0].verts.len(),
            15,
            "5 pontas x2 + 5 vales crus"
        );
    }

    #[test]
    fn fill_none_style_yields_stroke_only_shape() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.set_style(PenStyle {
            fill: ph2d_vec_scene::Rgba8::new(0, 0, 0, 0), // None
            ..PenStyle::default()
        });
        shape.on_press(
            &mut scene,
            ShapeKind::Rectangle,
            ShapeKind::Rectangle.defaults(),
            [0.0, 0.0],
            PTW,
            ShapeConstraint::default(),
        );
        shape.on_drag(&mut scene, [4.0, 3.0], ShapeConstraint::default());
        shape.on_release(&mut scene);
        let p = &scene.paths()[0];
        assert!(p.fill.is_none() && p.stroke.is_some());
    }

    /// A restrição vale para o PREVIEW e para o retângulo autorado ao mesmo tempo: o
    /// `bounds()` (de onde a shell tira o `w`/`h` da forma viva) devolve o retângulo JÁ
    /// constrangido. Se divergissem, a forma nasceria diferente do que se viu.
    #[test]
    fn the_authored_rect_is_the_constrained_one_the_preview_drew() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Rectangle,
            ShapeKind::Rectangle.defaults(),
            [0.0, 0.0],
            PTW,
            ShapeConstraint::default(),
        );
        shape.on_drag(&mut scene, [10.0, 3.0], SHIFT);
        assert_eq!(shape.bounds(), ([0.0, 0.0], [10.0, 10.0]));
        let max_y = scene.paths()[0]
            .verts
            .iter()
            .map(|v| v.anchor[1])
            .fold(f64::MIN, f64::max);
        assert!((max_y - 10.0).abs() < 1e-9, "o preview desenhou o quadrado");
    }

    /// Apertar Shift SEM mexer o mouse já reconstrói a forma (senão o usuário aperta e
    /// nada acontece até o próximo Move).
    #[test]
    fn toggling_a_modifier_alone_rebuilds_the_live_shape() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Rectangle,
            ShapeKind::Rectangle.defaults(),
            [0.0, 0.0],
            PTW,
            ShapeConstraint::default(),
        );
        shape.on_drag(&mut scene, [10.0, 3.0], ShapeConstraint::default());
        assert!(shape.set_constraint(&mut scene, SHIFT), "gesto vivo");
        let max_y = scene.paths()[0]
            .verts
            .iter()
            .map(|v| v.anchor[1])
            .fold(f64::MIN, f64::max);
        assert!(
            (max_y - 10.0).abs() < 1e-9,
            "virou quadrado sem mover o mouse"
        );
    }

    #[test]
    fn cancel_removes_in_progress_shape() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Ellipse,
            ShapeKind::Ellipse.defaults(),
            [1.0, 1.0],
            PTW,
            ShapeConstraint::default(),
        );
        assert_eq!(scene.paths().len(), 1);
        shape.cancel(&mut scene);
        assert!(scene.is_empty() && !shape.is_active());
    }
}
