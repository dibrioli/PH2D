//! ⭐ **O painel LÊ a forma depois de uma escrita ALHEIA** (F19, plano
//! `docs/UI_New_and_Simple/vetor_nova_ui/30_plano_F19_o_painel_le_a_forma.md`).
//!
//! A ponte real (`super::dispatch`) corre quadro a quadro; a escrita alheia entra pela porta que o
//! Show, o sinal e o Preview usam (`ui_state_edit::install`) ou pela troca da `VecScene` inteira
//! (o que o desfazer faz). Fixture: a forma da cena 25 e um Hover que muda TODA a família.

use ph2d_editor_core::panel::Panel;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::{PanelEvent, ToolId, ToolRegistry};
use ph2d_tool_vector::{VectorTool, ids};
use ph2d_vec_edit::{Pencil, PenTool, ShapeTool};
use ph2d_vec_scene::{
    DashPattern, LineCap, LineJoin, Marker, Paint, Rgba8, StrokeSpec, VecPathId, VecScene,
};

const AZUL: [u8; 4] = [40, 120, 240, 255];
const BRANCO: [u8; 4] = [240, 240, 245, 255];
const LARANJA: [u8; 4] = [240, 140, 40, 255];
const AMARELO: [u8; 4] = [250, 210, 60, 255];
/// Mundo por píxel de ecrã: 3 px de traço = `0,06` em mundo.
const PX: f64 = 0.02;

fn rgba(c: [u8; 4]) -> Rgba8 {
    Rgba8::new(c[0], c[1], c[2], c[3])
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
    id: VecPathId,
    px: f64,
}

impl Quadro {
    /// A forma da cena 25 (RoundRect `180×70`, fill azul, traço branco de 3 px), selecionada, e um
    /// quadro já corrido — o painel semeou dela.
    fn novo() -> Self {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(VectorTool::default()));
        assert!(tools.set_active(&ToolId::new("vector")));
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        <ph2d_panel_vector::VectorPanel as Panel>::populate(&mut hero.store);
        let mut scene = VecScene::new();
        let mut path = ph2d_vec_scene::rounded_rect([0.0, 0.0], [3.6, 1.4], 0.24);
        path.fill = Some(Paint::Solid(rgba(AZUL)));
        path.stroke = Some(StrokeSpec::new(rgba(BRANCO), 3.0 * PX));
        let id = scene.push_path(path);
        let mut sim = ph2d_ecs::SimWorld::default();
        let mut map = ph2d_vec_entities::entities::VecEntityMap::default();
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
        let mut pen = PenTool::default();
        pen.select(Some(id));
        let mut q = Self {
            hero,
            tools,
            scene,
            pen,
            shape: ShapeTool::default(),
            pencil: Pencil::default(),
            sim,
            map,
            id,
            px: PX,
        };
        q.quadro();
        assert_eq!(q.tool().fill_rgba(), AZUL, "a semente da seleção não correu");
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

    fn forma(&self) -> &ph2d_vec_scene::VecPath {
        self.scene.path(self.id).expect("a forma existe")
    }

    /// **O Show do Hover**: a pose capturada pela porta do *Rec*, com a família inteira mudada, e
    /// instalada pela porta que o Show, o sinal e o Preview usam.
    fn show_hover(&mut self) {
        let mut pose = crate::ui_state_edit::capture(&self.sim, &self.scene, &self.map, self.id);
        pose.fill = Some(Paint::Solid(rgba(LARANJA)));
        let mut s = StrokeSpec::new(rgba(AMARELO), 6.0 * PX);
        s.cap = LineCap::Round;
        s.join = LineJoin::Round;
        s.dash = Some(DashPattern::single(2.0, 1.0));
        s.marker_end = Marker::Triangle;
        pose.stroke = Some(s);
        crate::ui_state_edit::install(&mut self.sim, &mut self.scene, &self.map, &pose);
        assert_eq!(
            self.forma().fill,
            Some(Paint::Solid(rgba(LARANJA))),
            "a escrita alheia nem chegou à forma — o gate mediria o vazio"
        );
    }

    fn slider(&self, id: ph2d_editor_core::NodeId) -> f32 {
        self.hero.store.slider(id).map(|(_, v)| v).expect("slider registado")
    }

    /// O seletor de cor flutuante, registado como o `pre_populate` o regista, e aberto sobre `swatch`.
    fn abre_seletor(&mut self, swatch: ph2d_editor_core::NodeId) {
        self.hero.store.register(
            ph2d_editor_core::ids::INSP_BLENDER_PICKER,
            ph2d_editor_core::InteractiveState::BlenderPicker {
                value: ph2d_tokens::ColorValue::from_rgba8(0, 0, 0, 255),
                channel_mode: ph2d_editor_core::widget::ChannelMode::Rgb,
                interpolation: ph2d_editor_core::widget::InterpolationMode::Perceptual,
                active_palette: 0,
                hsv_h: 0.0,
                hsv_s: 0.0,
                harmony: ph2d_editor_core::widget::Harmony::None,
            },
        );
        self.hero.store.open_picker_on(swatch);
    }

    fn escolhe(&mut self, c: [u8; 4]) {
        self.hero.store.set_blender_value(
            ph2d_editor_core::ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], c[3]),
        );
    }

    fn seletor(&self) -> [u8; 4] {
        self.hero
            .store
            .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
            .map(|(v, ..)| v.rgba)
            .expect("o seletor está registado")
    }
}

/// **Depois de um Show, o painel mostra a forma** — os sete campos da família, na tool (que o painel
/// pinta), nas amostras e nas barras do store.
#[test]
fn o_painel_reve_a_forma_depois_de_uma_escrita_alheia() {
    let mut q = Quadro::novo();
    q.show_hover();
    q.quadro();
    let fill_swatch = q.hero.store.widget_color(ids::VECTOR_FILL_SWATCH);
    let stroke_swatch = q.hero.store.widget_color(ids::VECTOR_STROKE_SWATCH);
    let width_track = q.slider(ids::VECTOR_WIDTH);
    let t = q.tool();
    assert_eq!(t.fill_rgba(), LARANJA, "o Fill do painel ficou com a cor de antes do Show");
    assert_eq!(t.stroke_rgba(), AMARELO, "o Stroke do painel ficou com a cor de antes");
    assert!(
        (t.stroke_width_px() - 6.0).abs() < 1e-9,
        "a Width ficou em {} px",
        t.stroke_width_px()
    );
    assert_eq!(super::line_cap(t.cap()), LineCap::Round, "o Cap ficou o de antes");
    assert_eq!(super::line_join(t.join()), LineJoin::Round, "o Join ficou o de antes");
    assert_eq!(t.dash_pattern(), Some(DashPattern::single(2.0, 1.0)), "o tracejado ficou o de antes");
    assert_eq!(t.marker_end(), Marker::Triangle, "a ponta ficou a de antes");
    assert_eq!(fill_swatch, Some(LARANJA), "a amostra Fill (painel e barra) mente");
    assert_eq!(stroke_swatch, Some(AMARELO), "a amostra Stroke (painel e barra) mente");
    assert!(
        (f64::from(width_track) - f64::from(ph2d_tool_vector::params::px_to_slider(6.0))).abs()
            < 1e-6,
        "a barra de Width do store ficou no valor de antes ({width_track})"
    );
}

/// ⛔ **Tocar UM controlo depois do Show não pode reescrever o estilo velho por cima da forma.**
/// Hoje a Width armava o restyle, e o detector comparava a forma com a tool velha: o laranja virava azul.
#[test]
fn tocar_um_controlo_depois_do_show_nao_apaga_o_hover() {
    let mut q = Quadro::novo();
    q.show_hover();
    q.quadro();
    let v = f64::from(ph2d_tool_vector::params::px_to_slider(9.0));
    q.tools
        .tool_by_id_mut(&ToolId::new("vector"))
        .expect("a tool de vetor está registada")
        .handle_panel_event(PanelEvent::SetValue(ids::VECTOR_WIDTH, v));
    q.quadro();
    let f = q.forma();
    assert_eq!(f.fill, Some(Paint::Solid(rgba(LARANJA))), "a Width apagou o laranja do Hover");
    let s = f.stroke.as_ref().expect("o traço existe");
    assert_eq!(s.color(), rgba(AMARELO), "a Width apagou o amarelo do Hover");
    assert_eq!(s.dash, Some(DashPattern::single(2.0, 1.0)), "a Width apagou o tracejado do Hover");
    assert!((s.width - 9.0 * PX).abs() < 1e-6, "a Width não chegou à forma ({})", s.width);
}

/// ⛔ **O seletor aberto não desfaz uma escrita alheia** — e passa a mostrar a cor nova.
#[test]
fn o_seletor_aberto_nao_desfaz_uma_escrita_alheia() {
    let mut q = Quadro::novo();
    q.abre_seletor(ids::VECTOR_FILL_SWATCH);
    q.quadro();
    assert_eq!(q.seletor(), AZUL, "o seletor abriu semeado da amostra");
    q.show_hover();
    for _ in 0..3 {
        q.quadro();
    }
    assert_eq!(
        q.forma().fill,
        Some(Paint::Solid(rgba(LARANJA))),
        "o seletor aberto reescreveu a cor de antes por cima do Show"
    );
    assert_eq!(q.seletor(), LARANJA, "o seletor aberto continua a mostrar a cor de antes");
}

/// **O desfazer vê-se no painel** — a `VecScene` inteira trocada por uma anterior, o MESMO id
/// selecionado (é o que `undo::restore` faz).
#[test]
fn o_desfazer_se_ve_no_painel() {
    let mut q = Quadro::novo();
    let antes = q.scene.clone();
    q.show_hover();
    q.quadro();
    assert_eq!(q.tool().fill_rgba(), LARANJA);
    q.scene = antes;
    q.quadro();
    let t = q.tool();
    assert_eq!(t.fill_rgba(), AZUL, "depois do desfazer o Fill do painel ficou laranja");
    assert_eq!(t.stroke_rgba(), BRANCO, "depois do desfazer o Stroke ficou amarelo");
    assert!((t.stroke_width_px() - 3.0).abs() < 1e-9, "a Width não voltou");
    assert_eq!(t.dash_pattern(), None, "o tracejado não voltou");
}

/// Controlo: **o painel não briga com a própria escrita** — escolhas sucessivas no seletor aberto
/// chegam à forma, uma por quadro, e o seletor fica com a última. Verde antes e depois da cura; uma
/// re-semeadura a cada quadro (sem a régua do que o painel viu) puxava a cor anterior de volta.
#[test]
fn o_painel_nao_briga_com_a_propria_escrita() {
    let mut q = Quadro::novo();
    q.abre_seletor(ids::VECTOR_FILL_SWATCH);
    q.quadro();
    for c in [LARANJA, AMARELO, [10, 200, 90, 255], AZUL] {
        q.escolhe(c);
        q.quadro();
        assert_eq!(q.forma().fill, Some(Paint::Solid(rgba(c))), "a escolha {c:?} não chegou");
        assert_eq!(q.seletor(), c, "o seletor foi puxado para outra cor");
        assert_eq!(q.tool().fill_rgba(), c);
    }
}

/// Controlo: **o zoom não re-semeia** — a largura do painel é de ecrã e a régua é do documento.
#[test]
fn o_zoom_nao_re_semeia() {
    let mut q = Quadro::novo();
    q.px = PX * 2.0;
    q.quadro();
    assert!(
        (q.tool().stroke_width_px() - 3.0).abs() < 1e-9,
        "o zoom re-semeou a Width ({} px) sem escrita nenhuma na forma",
        q.tool().stroke_width_px()
    );
}
