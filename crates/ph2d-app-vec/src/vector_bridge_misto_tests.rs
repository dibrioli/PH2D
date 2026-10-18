//! ⭐ **Seleção múltipla: «misto» e escrita campo a campo · a espessura na régua do documento**
//! (plano 31, `docs/UI_New_and_Simple/vetor_nova_ui/31_plano_F20_misto_e_espessura_do_documento.md`).
//!
//! Fixture que contém o fenómeno: duas formas com TODOS os campos do estilo diferentes, as duas
//! selecionadas, o primário a vermelha. O despacho é o `dispatch` real, quadro a quadro.

use ph2d_editor_core::panel::Panel;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::{DocPx, PanelEvent, ToolId, ToolRegistry};
use ph2d_tool_vector::{StyleFields as F, VectorTool, ids};
use ph2d_vec_edit::{PenTool, Pencil, ShapeTool};
use ph2d_vec_scene::{
    DashPattern, LineCap, LineJoin, Marker, Paint, Rgba8, StrokeAlign, StrokeSpec, VecPath,
    VecPathId, VecScene,
};

const VERMELHO: [u8; 4] = [220, 40, 40, 255];
const AZUL: [u8; 4] = [40, 120, 240, 255];
const PRETO: [u8; 4] = [20, 20, 20, 255];
/// Mundo por px de ecrã a 100 % (a régua do documento por omissão).
const PX: f64 = 0.01;

fn rgba(c: [u8; 4]) -> Rgba8 {
    Rgba8::new(c[0], c[1], c[2], c[3])
}

fn forma(x: f64, fill: [u8; 4], stroke: StrokeSpec) -> VecPath {
    let mut p = ph2d_vec_scene::rounded_rect([x, 0.0], [x + 1.8, 0.7], 0.12);
    p.fill = Some(Paint::Solid(rgba(fill)));
    p.stroke = Some(stroke);
    p
}

/// A vermelha: traço preto de 2 px, Round/Round, tracejado, ponta no começo.
fn traco_vermelha() -> StrokeSpec {
    let mut s = StrokeSpec::new(rgba(PRETO), 2.0 * PX);
    s.cap = LineCap::Round;
    s.join = LineJoin::Round;
    s.dash = Some(DashPattern::single(2.0, 1.0));
    s.marker_start = Marker::Triangle;
    s.marker_scale_start = 1.5;
    s.marker_round = 0.2;
    s
}

/// A azul: traço branco de 6 px, Butt/Miter, outro tracejado, ponta no fim, alinhado por dentro.
fn traco_azul() -> StrokeSpec {
    let mut s = StrokeSpec::new(Rgba8::new(240, 240, 245, 200), 6.0 * PX);
    s.align = StrokeAlign::Inner;
    s.dash = Some(DashPattern::single(5.0, 2.0));
    s.marker_end = Marker::Triangle;
    s.marker_scale_end = 2.0;
    s
}

struct Quadro {
    hero: HeroScreen,
    tools: ToolRegistry,
    scene: VecScene,
    pen: PenTool,
    shape: ShapeTool,
    pencil: Pencil,
    sim: ph2d_ecs::SimWorld,
    map: ph2d_vec_entities::entities::VecEntityMap,
    vermelha: VecPathId,
    azul: VecPathId,
    px: f64,
}

impl Quadro {
    /// As duas formas, selecionadas com o primário na VERMELHA, e um quadro já corrido.
    fn novo() -> Self {
        Self::com(true)
    }

    fn com(as_duas: bool) -> Self {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(VectorTool::default()));
        assert!(tools.set_active(&ToolId::new("vector")));
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        <ph2d_panel_vector::VectorPanel as Panel>::populate(&mut hero.store);
        let mut scene = VecScene::new();
        let azul = scene.push_path(forma(3.0, AZUL, traco_azul()));
        let vermelha = scene.push_path(forma(0.0, VERMELHO, traco_vermelha()));
        let mut sim = ph2d_ecs::SimWorld::default();
        let mut map = ph2d_vec_entities::entities::VecEntityMap::default();
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
        let mut pen = PenTool::default();
        if as_duas {
            pen.select_many(&[azul, vermelha]);
        } else {
            pen.select(Some(vermelha));
        }
        let mut q = Self {
            hero,
            tools,
            scene,
            pen,
            shape: ShapeTool::default(),
            pencil: Pencil::default(),
            sim,
            map,
            vermelha,
            azul,
            px: PX,
        };
        assert_eq!(q.pen.selected(), Some(vermelha), "o primário é a vermelha");
        q.quadro();
        assert_eq!(
            q.tool().fill_rgba(),
            VERMELHO,
            "a semente do primário não correu"
        );
        q
    }

    fn quadro(&mut self) {
        super::dispatch(
            &mut self.hero,
            &mut self.tools,
            &mut self.scene,
            &mut self.pen,
            &mut self.shape,
            &mut self.pencil,
            self.px,
            None,
            &ph2d_vec_scene::VecXforms::default(),
            &self.sim,
            &self.map,
            false,
            crate::snap::VecSnapSettings::default(),
            [false; 2],
            [false; 2],
            &ph2d_vec_render::PatternTiles::default(),
        );
    }

    fn tool(&mut self) -> &mut VectorTool {
        self.tools
            .tool_by_id_mut(&ToolId::new("vector"))
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorTool>())
            .expect("a tool de vetor está registada")
    }

    fn evento(&mut self, e: PanelEvent) {
        self.tools
            .tool_by_id_mut(&ToolId::new("vector"))
            .expect("a tool de vetor está registada")
            .handle_panel_event(e);
    }

    fn path(&self, id: VecPathId) -> &VecPath {
        self.scene.path(id).expect("a forma existe")
    }

    fn traco(&self, id: VecPathId) -> &StrokeSpec {
        self.path(id).stroke.as_ref().expect("o traço existe")
    }
}

/// O que mudou numa forma: os campos do traço e os do preenchimento.
fn mudou(antes: &VecPath, depois: &VecPath) -> F {
    let mut f = super::fields::stroke_diff(
        antes.stroke.as_ref().expect("traço"),
        depois.stroke.as_ref().expect("traço"),
    );
    let cor = |p: &VecPath| match &p.fill {
        Some(Paint::Solid(c)) => [c.r, c.g, c.b, c.a],
        _ => [0; 4],
    };
    let (a, d) = (cor(antes), cor(depois));
    if a[..3] != d[..3] {
        f |= F::FILL_RGB;
    }
    if a[3] != d[3] {
        f |= F::FILL_A;
    }
    f
}

/// ⛔ **O defeito medido no plano 31:** mexer só na Width de uma vermelha e de uma azul pintava as
/// duas com a cor (e o traço inteiro) do primário.
#[test]
fn mexer_na_width_de_duas_formas_so_muda_a_width() {
    let mut q = Quadro::novo();
    let azul_antes = q.path(q.azul).clone();
    let vermelha_antes = q.path(q.vermelha).clone();
    q.evento(PanelEvent::SetValue(
        ids::VECTOR_WIDTH,
        f64::from(ph2d_tool_vector::params::px_to_slider(9.0)),
    ));
    q.quadro();
    let nove = DocPx::DEFAULT.to_world(9.0);
    for (id, antes, nome) in [
        (q.azul, &azul_antes, "azul"),
        (q.vermelha, &vermelha_antes, "vermelha"),
    ] {
        // A trilha do slider é `f32`: 9 volta como 9,0000004.
        assert!(
            (q.traco(id).width - nove).abs() < 1e-8,
            "a {nome} não recebeu a Width"
        );
        assert_eq!(
            mudou(antes, q.path(id)),
            F::WIDTH,
            "a Width da {nome} levou outros campos com ela"
        );
    }
    assert_eq!(
        q.path(q.azul).fill,
        Some(Paint::Solid(rgba(AZUL))),
        "a azul ficou vermelha"
    );
    assert_eq!(q.traco(q.azul).color(), Rgba8::new(240, 240, 245, 200));
}

/// **Cada controlo escreve SÓ o seu campo** — um evento por controlo, sobre a fixture inteira; na
/// forma que NÃO é o primário, muda exatamente o bit que o controlo nomeia.
#[test]
fn cada_controlo_escreve_so_o_seu_campo() {
    type Gesto = fn(&mut Quadro);
    let casos: &[(&str, Gesto, F)] = &[
        (
            "Fill",
            |q| q.tool().set_fill_rgba([10, 200, 90, 255]),
            F::FILL_RGB,
        ),
        (
            "Fill Opacity",
            |q| q.evento(PanelEvent::SetValue(ids::VECTOR_FILL_OPACITY, 0.5)),
            F::FILL_A,
        ),
        (
            "Stroke",
            |q| q.tool().set_stroke_rgba([10, 200, 90, 255]),
            F::STROKE_RGB,
        ),
        (
            "Stroke Opacity",
            |q| q.evento(PanelEvent::SetValue(ids::VECTOR_STROKE_OPACITY, 0.5)),
            F::STROKE_A,
        ),
        (
            "Cap",
            |q| q.evento(PanelEvent::Click(ids::VECTOR_CAP_SQUARE)),
            F::CAP,
        ),
        (
            "Join",
            |q| q.evento(PanelEvent::Click(ids::VECTOR_JOIN_BEVEL)),
            F::JOIN,
        ),
        (
            "Align",
            |q| q.evento(PanelEvent::Click(ids::VECTOR_ALIGN_OUTER)),
            F::ALIGN,
        ),
        (
            "Dash 1",
            |q| q.evento(PanelEvent::SetValue(ids::VECTOR_DASH_NUM, 4.0)),
            F::dash(0, false),
        ),
        (
            "Gap 1",
            |q| q.evento(PanelEvent::SetValue(ids::VECTOR_GAP_NUM, 3.0)),
            F::dash(0, true),
        ),
        (
            "Head Size (fim)",
            |q| q.evento(PanelEvent::SetValue(ids::VECTOR_MARKER_SCALE_END, 3.0)),
            F::MARKER_SCALE_END,
        ),
        (
            "Head Round",
            |q| q.evento(PanelEvent::SetValue(ids::VECTOR_MARKER_ROUND, 0.6)),
            F::MARKER_ROUND,
        ),
    ];
    for (nome, gesto, campo) in casos {
        let mut q = Quadro::novo();
        let antes = q.path(q.azul).clone();
        gesto(&mut q);
        q.quadro();
        assert_eq!(
            mudou(&antes, q.path(q.azul)),
            *campo,
            "«{nome}» na seleção de duas: a forma que não é o primário mudou outros campos"
        );
    }
}

/// **O painel diz «misto» onde as formas diferem, e só aí.**
#[test]
fn o_painel_diz_misto_onde_as_formas_diferem() {
    let mut q = Quadro::novo();
    let esperado = F::FILL_RGB
        | F::STROKE_RGB
        | F::STROKE_A
        | F::WIDTH
        | F::CAP
        | F::JOIN
        | F::ALIGN
        | F::MARKER_START
        | F::MARKER_END
        | F::MARKER_SCALE_START
        | F::MARKER_SCALE_END
        | F::MARKER_ROUND
        | F::dash(0, false)
        | F::dash(0, true);
    assert_eq!(
        super::mixed_of(&q.pen, &q.scene),
        esperado,
        "o «misto» não é a lista dos campos que diferem"
    );
    let (_, buffer, ..) = q.hero.store.number_chip_view(ids::VECTOR_WIDTH_NUM);
    assert_eq!(buffer, Some(""), "o número da Width não ficou em branco");
    // Depois da Width, as duas concordam nela — e o número volta.
    q.evento(PanelEvent::SetValue(
        ids::VECTOR_WIDTH,
        f64::from(ph2d_tool_vector::params::px_to_slider(9.0)),
    ));
    q.quadro();
    q.quadro();
    assert!(!super::mixed_of(&q.pen, &q.scene).any(F::WIDTH));
    let n = q
        .hero
        .store
        .number_value(ids::VECTOR_WIDTH_NUM)
        .expect("caixa da Width");
    assert!((n - 9.0).abs() < 1e-5, "o número voltou com {n}");
    let (_, buffer, ..) = q.hero.store.number_chip_view(ids::VECTOR_WIDTH_NUM);
    assert_ne!(buffer, Some(""), "o número não voltou à caixa");

    let uma = Quadro::com(false);
    assert_eq!(
        super::mixed_of(&uma.pen, &uma.scene),
        F::NONE,
        "uma forma não é «misto»"
    );
}

/// ⛔ **O número da Width era px de TELA**: um traço de 3 lia 2,952 a 98 %. Na régua do documento
/// ele lê 3 a qualquer zoom.
#[test]
fn a_width_e_a_mesma_a_qualquer_zoom() {
    for px in [PX * 2.0, PX / 0.98, PX / 4.0] {
        let mut q = Quadro::com(false);
        q.px = px;
        // Uma escrita alheia força a re-semente neste zoom.
        let mut s = q.traco(q.vermelha).clone();
        s.width = DocPx::DEFAULT.to_world(3.0);
        q.scene.path_mut(q.vermelha).expect("forma").stroke = Some(s);
        q.quadro();
        assert!(
            (q.tool().stroke_width_px() - 3.0).abs() < 1e-9,
            "a {px}: a Width leu {}",
            q.tool().stroke_width_px()
        );
        assert_eq!(q.hero.store.number_value(ids::VECTOR_WIDTH_NUM), Some(3.0));
    }
}

/// **Desenhar com «3» dá a mesma linha a qualquer zoom** — o estilo das formas novas leva a
/// largura já em mundo, pela régua do documento.
#[test]
fn desenhar_com_3_da_a_mesma_linha_a_qualquer_zoom() {
    let mut larguras = Vec::new();
    for px in [PX / 2.0, PX * 4.0] {
        let mut q = Quadro::com(false);
        q.pen.select(None);
        q.px = px;
        q.evento(PanelEvent::SetValue(
            ids::VECTOR_WIDTH,
            f64::from(ph2d_tool_vector::params::px_to_slider(3.0)),
        ));
        q.quadro();
        // A caneta, a forma e o lápis recebem a MESMA ficha (`set_style` nos três).
        larguras.push(q.pen.style().stroke_w);
    }
    let tres = DocPx::DEFAULT.to_world(3.0);
    for w in larguras {
        assert!(
            (w - tres).abs() < 1e-9,
            "a forma nova nasceu com {w} de mundo"
        );
    }
}

/// O controlo: o mesmo gesto com UMA forma escreve o mesmo de sempre (a cor inteira, a Width).
#[test]
fn com_uma_forma_o_gesto_escreve_como_antes() {
    let mut q = Quadro::com(false);
    q.tool().set_fill_rgba([10, 200, 90, 128]);
    q.quadro();
    assert_eq!(
        q.path(q.vermelha).fill,
        Some(Paint::Solid(Rgba8::new(10, 200, 90, 128))),
        "o seletor (que traz RGB e alfa) deixou de pintar a forma selecionada"
    );
    assert_eq!(q.path(q.azul).fill, Some(Paint::Solid(rgba(AZUL))));
}
