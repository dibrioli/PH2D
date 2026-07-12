//! [`VectorTool`] — the Style model for the Vector drawing tool.
//!
//! The tool is deliberately thin: it holds the current stroke colour, fill
//! colour, and stroke width. The real UI is the **docked** `ph2d-panel-vector`
//! (a `Panel<State>`, right-docked in the Inspector slot while the tool is
//! active) — tool `FloatingPanel`s are unpainted in this app, so the panel is a
//! separate crate that drives the tool through the generic `ToolPanelEvent`
//! channel + colour-picker read-back (mirror of the Padding tool+panel pair).
//!
//! The shell's `vector_bridge` reads this Style each frame (downcast via
//! [`Tool::as_any_mut`]) to restyle newly drawn paths and — on a picker pick /
//! Fill-None — recolour the selected path.
//!
//! ## Colour approach
//!
//! Docked panels CAN drive the shared OKLCH (Blender) colour picker (unlike
//! tool `FloatingPanel`s): the panel paints a `ColorSwatch` + calls
//! `register_picker_swatch`, the shell reads the picked colour back and feeds it
//! through [`VectorTool::set_stroke_rgba`] / [`VectorTool::set_fill_rgba`]. The
//! [`PALETTE`] below is retained as a curated preset list (seeds the defaults);
//! the picker is the live path.

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_vec_scene::{ALL_SHAPES, MAX_SHAPE_FIELDS, ShapeKind, ShapeValues};

/// Teto de formas no catálogo (o array de valores por-forma é indexado pelo
/// discriminante). Cresce quando o catálogo cresce; um gate prova que cabe.
const MAX_SHAPES: usize = 64;

/// Os valores default de cada forma, indexados pelo discriminante — o estado inicial do
/// "último usado" de cada uma. Os raios chegam do catálogo em MUNDO e a tool os guarda
/// em UI (px), então a semente do painel os converte na fronteira (a tool não conhece a
/// câmera): o default de raio nasce ZERO em px e o usuário digita o que quiser.
fn default_shape_values() -> [ShapeValues; MAX_SHAPES] {
    let mut out = [[0.0; MAX_SHAPE_FIELDS]; MAX_SHAPES];
    for &k in ALL_SHAPES {
        let mut v = k.defaults();
        // Campos em px: o default do catálogo é mundo — não faz sentido em px sem a
        // câmera. Nasce em 0 (canto vivo) e o usuário autora.
        for (i, f) in crate::shapes::desc(k).fields.iter().enumerate() {
            if f.unit == crate::shapes::FieldUnit::Px {
                v[i] = 0.0;
            }
        }
        out[k.as_u16() as usize] = v;
    }
    out
}

/// O índice do parâmetro cujo id de campo é `id` (`None` se não for um).
fn shape_field_index(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..MAX_SHAPE_FIELDS).find(|&i| ids::vector_shape_field_id(i) == id)
}

/// O índice da forma no catálogo cujo id de botão é `id` (`None` se não for um).
fn shape_index(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..crate::shapes::SHAPES.len()).find(|&i| ids::vector_shape_id(i) == id)
}

use crate::params::{
    DrawMode, StrokeCap, StrokeJoin, VectorDrawConfig, VectorStyleSnapshot, slider_to_dash,
    slider_to_gap, slider_to_opacity, slider_to_px,
};

/// Curated stroke / fill preset palette: `(key, label, sRGB8)`. Retained as the
/// seed source for the tool's defaults (and a stable named-colour reference);
/// the live colour path is the OKLCH picker driven by the docked panel.
pub const PALETTE: &[(&str, &str, [u8; 4])] = &[
    ("white", "White", [240, 240, 245, 255]),
    ("black", "Black", [20, 20, 24, 255]),
    ("gray", "Gray", [130, 130, 138, 255]),
    ("red", "Red", [220, 60, 60, 255]),
    ("orange", "Orange", [230, 140, 40, 255]),
    ("yellow", "Yellow", [235, 205, 50, 255]),
    ("green", "Green", [70, 190, 90, 255]),
    ("cyan", "Cyan", [60, 190, 205, 255]),
    ("blue", "Blue", [90, 150, 230, 255]),
    ("purple", "Purple", [160, 110, 220, 255]),
];

/// Default stroke width in screen pixels (matches the old `PenTool` default).
pub const DEFAULT_STROKE_WIDTH_PX: f64 = 3.0;

/// Default polygon side count (a pentagon reads clearly as "polygon").
pub const DEFAULT_POLYGON_SIDES: u32 = 5;

/// Default star point count / inner ratio / rounded-rect corner radius (px).
pub const DEFAULT_STAR_POINTS: u32 = 5;
pub const DEFAULT_STAR_INNER: f64 = 0.5;
pub const DEFAULT_CORNER_RADIUS_PX: f64 = 12.0;
/// Polígono e estrela nascem de quinas VIVAS (o canto redondo é opt-in).
pub const DEFAULT_POLYGON_RADIUS_PX: f64 = 0.0;
pub const DEFAULT_STAR_OUTER_RADIUS_PX: f64 = 0.0;
pub const DEFAULT_STAR_INNER_RADIUS_PX: f64 = 0.0;
/// Default spiral turn count.
pub const DEFAULT_SPIRAL_TURNS: u32 = 3;
/// Default span de um arco novo (semicírculo).
pub const DEFAULT_ARC_DEGREES: f64 = 180.0;

/// Look up a palette colour by key (defaults only — the live path is the picker).
fn color_of(key: &str) -> Option<[u8; 4]> {
    PALETTE
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, _, c)| *c)
}

/// Vector drawing tool — Style + draw-mode model only.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorTool {
    stroke: [u8; 4],
    /// Fill applied on close; alpha 0 ⇒ no fill.
    fill: [u8; 4],
    /// Stroke width in screen pixels, held in `WIDTH_MIN_PX..=WIDTH_MAX_PX`.
    stroke_width_px: f64,
    /// Canvas gesture: Pen (draw + edit) vs a drag-to-size shape. The shell
    /// mirrors this each frame to route canvas input (`vector_bridge`).
    mode: DrawMode,
    /// A forma ATIVA do catálogo (o que o modo `Shape` desenha).
    shape: ShapeKind,
    /// Os parâmetros de CADA forma, na unidade de UI (px para raios), indexados pelo
    /// discriminante. Guardar por-forma é o que faz cada uma lembrar do "último usado":
    /// mexer no raio da estrela não mexe no do retângulo.
    shape_values: [ShapeValues; MAX_SHAPES],
    /// Stroke cap / join + dash & gap as multiples of the stroke width
    /// (`dash = 0` = solid; `gap` is the space between dashes).
    cap: StrokeCap,
    join: StrokeJoin,
    dash: f64,
    gap: f64,
    /// Set when a colour changes → the shell recolours the selected path.
    /// Drained by [`Self::take_apply_to_selected`].
    apply_to_selected: bool,
}

impl Default for VectorTool {
    fn default() -> Self {
        Self {
            stroke: color_of("white").unwrap_or([240, 240, 245, 255]),
            fill: color_of("blue").unwrap_or([90, 150, 230, 255]),
            stroke_width_px: DEFAULT_STROKE_WIDTH_PX,
            mode: DrawMode::Select,
            shape: ShapeKind::default(),
            shape_values: default_shape_values(),
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            dash: 0.0,
            gap: crate::params::GAP_DEFAULT,
            apply_to_selected: false,
        }
    }
}

impl VectorTool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Current stroke colour (sRGB8).
    #[must_use]
    pub fn stroke_rgba(&self) -> [u8; 4] {
        self.stroke
    }

    /// Current fill colour (sRGB8); alpha 0 ⇒ no fill on close.
    #[must_use]
    pub fn fill_rgba(&self) -> [u8; 4] {
        self.fill
    }

    /// Current stroke width in screen pixels.
    #[must_use]
    pub fn stroke_width_px(&self) -> f64 {
        self.stroke_width_px
    }

    /// Current canvas draw-mode (the shell mirrors this to route input).
    #[must_use]
    pub fn mode(&self) -> DrawMode {
        self.mode
    }

    /// Set the canvas draw-mode. The panel's mode row goes through
    /// `handle_panel_event`; this is the equivalent entry point for a keyboard
    /// shortcut (e.g. `T` → [`DrawMode::Text`]) driven from the shell.
    pub fn set_mode(&mut self, mode: DrawMode) {
        self.mode = mode;
    }

    /// A forma ativa do catálogo.
    #[must_use]
    pub fn shape(&self) -> ShapeKind {
        self.shape
    }

    /// Escolhe a forma ativa E entra no modo de desenho de forma — é o que o clique num
    /// botão do catálogo faz (escolher a forma sem armar o gesto seria um clique morto).
    pub fn set_shape(&mut self, shape: ShapeKind) {
        self.shape = shape;
        self.mode = DrawMode::Shape;
    }

    /// Os parâmetros da forma `k` na unidade de UI (px para raios).
    #[must_use]
    pub fn shape_values(&self, k: ShapeKind) -> ShapeValues {
        self.shape_values[k.as_u16() as usize]
    }

    /// Escreve o parâmetro `i` da forma ATIVA (o valor vem da caixa do painel, já na
    /// unidade de UI) — clampado à faixa que o catálogo declara.
    pub fn set_shape_field(&mut self, i: usize, v: f64) {
        if i >= MAX_SHAPE_FIELDS {
            return;
        }
        let k = self.shape;
        let slot = &mut self.shape_values[k.as_u16() as usize];
        slot[i] = v;
        crate::shapes::clamp(k, slot);
    }

    /// ADOTA os parâmetros de uma forma (a shell chama ao selecionar uma forma viva):
    /// eles viram os correntes daquela forma, então o painel para de mentir e a próxima
    /// desenhada os herda (modelo Figma "último usado").
    pub fn adopt_shape_values(&mut self, k: ShapeKind, v: ShapeValues) {
        self.shape_values[k.as_u16() as usize] = v;
        crate::shapes::clamp(k, &mut self.shape_values[k.as_u16() as usize]);
    }

    /// Stroke cap / join / dash (multiple of width) — the shell maps cap/join to
    /// the geometry enums; the render multiplies dash by the path's width.
    #[must_use]
    pub fn cap(&self) -> StrokeCap {
        self.cap
    }
    #[must_use]
    pub fn join(&self) -> StrokeJoin {
        self.join
    }
    #[must_use]
    pub fn dash(&self) -> f64 {
        self.dash
    }
    #[must_use]
    pub fn gap(&self) -> f64 {
        self.gap
    }

    /// Set the cap / join + flag the selected path for restyle.
    fn set_cap(&mut self, cap: StrokeCap) {
        self.cap = cap;
        self.apply_to_selected = true;
    }
    fn set_join(&mut self, join: StrokeJoin) {
        self.join = join;
        self.apply_to_selected = true;
    }

    /// Mode + shape parameters the shell mirrors to drive the `ShapeTool`.
    #[must_use]
    pub fn draw_config(&self) -> VectorDrawConfig {
        VectorDrawConfig {
            mode: self.mode,
            shape: self.shape,
            values: self.shape_values(self.shape),
        }
    }

    /// Set the stroke colour (picker read-back) + flag the selected path for
    /// recolour. `a = 0` is accepted (a fully-transparent stroke is unusual but
    /// not rejected here — the panel drives opaque picks).
    pub fn set_stroke_rgba(&mut self, rgba: [u8; 4]) {
        self.stroke = rgba;
        self.apply_to_selected = true;
    }

    /// Set the fill colour (picker read-back) + flag the selected path for
    /// recolour. `a = 0` ⇒ "None" (no fill).
    pub fn set_fill_rgba(&mut self, rgba: [u8; 4]) {
        self.fill = rgba;
        self.apply_to_selected = true;
    }

    /// Project the current Style into the snapshot the docked panel paints.
    #[must_use]
    pub fn ui_snapshot(&self) -> VectorStyleSnapshot {
        VectorStyleSnapshot {
            stroke: self.stroke,
            fill: self.fill,
            stroke_width_px: self.stroke_width_px,
            mode: self.mode,
            shape: self.shape,
            values: self.shape_values(self.shape),
            cap: self.cap,
            join: self.join,
            dash: self.dash,
            gap: self.gap,
        }
    }

    /// Drain the "recolour the selected path" request (set on any colour change).
    pub fn take_apply_to_selected(&mut self) -> bool {
        std::mem::take(&mut self.apply_to_selected)
    }
}

impl Tool for VectorTool {
    fn id(&self) -> ToolId {
        ToolId::new("vector")
    }

    fn label(&self) -> &str {
        "Vector"
    }

    fn icon_slug(&self) -> &str {
        "vector"
    }

    fn build_panel(&self) -> FloatingPanel {
        // The real UI is the docked `ph2d-panel-vector` crate; tool
        // `FloatingPanel`s are unpainted (input-dispatch only) in this app. A
        // minimal empty panel shell is returned so `Tool::build_panel` has a
        // value — it carries no controls (mirror of `PaddingTool`).
        let mut panel = FloatingPanel::new(self.id(), "Vector");
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        // Docked-panel control ids are the shared `ph2d_editor_core::ids::VECTOR_*`
        // chrome NodeIds (the panel forwards `SetValue` / `Click` over
        // `ToolPanelEvent`; the swatch colours arrive via the setters above,
        // driven by the picker read-back in `vector_bridge`).
        match event {
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_WIDTH => {
                self.stroke_width_px = slider_to_px(v as f32);
                // Also restyle the selected path (mirror of a colour change), so
                // the width slider affects the path you're looking at — not just
                // the next one drawn.
                self.apply_to_selected = true;
            }
            // **Campo de forma** — um braço só para TODAS as formas: o id carrega o
            // ÍNDICE do parâmetro no catálogo, e a forma ativa diz o que ele significa.
            // Antes era um braço por parâmetro por forma; com 25 formas seria um pântano.
            PanelEvent::SetValue(id, v) if shape_field_index(id).is_some() => {
                if let Some(i) = shape_field_index(id) {
                    self.set_shape_field(i, v);
                }
            }
            // Opacity sliders own the fill/stroke alpha (the single source). The
            // picker only sets RGB. `0 %` alpha ⇒ invisible (no fill).
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_FILL_OPACITY => {
                self.fill[3] = slider_to_opacity(v as f32);
                self.apply_to_selected = true;
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_STROKE_OPACITY => {
                self.stroke[3] = slider_to_opacity(v as f32);
                self.apply_to_selected = true;
            }
            // Draw-mode segmented row: switches the canvas gesture. No recolour
            // (mode is not a Style change) — the shell reads `mode()` to route.
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_SELECT => self.mode = DrawMode::Select,
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_NODE => self.mode = DrawMode::Node,
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_PEN => self.mode = DrawMode::Pen,
            // **Forma** — o 5º pill. Re-arma o gesto na forma ATIVA do catálogo (não a
            // troca): é o caminho de volta ao desenho depois de um desvio pelo Select,
            // e é o pill que ACENDE enquanto se desenha (antes o modo Shape não tinha
            // botão nenhum e a fileira inteira ficava apagada).
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_SHAPE => self.mode = DrawMode::Shape,
            // **Botão de forma** — idem: o id carrega o índice no catálogo. Escolher a
            // forma JÁ arma o gesto de desenho (`set_shape` põe o modo em Shape).
            PanelEvent::Click(id) if shape_index(id).is_some() => {
                if let Some(k) = shape_index(id).and_then(|i| crate::shapes::SHAPES.get(i)) {
                    self.set_shape(k.kind);
                }
            }
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_TEXT => self.mode = DrawMode::Text,
            // Stroke cap / join segmented rows + Dash slider. These are Style →
            // restyle the selected path (mirror of colour/width).
            PanelEvent::Click(id) if id == ids::VECTOR_CAP_BUTT => self.set_cap(StrokeCap::Butt),
            PanelEvent::Click(id) if id == ids::VECTOR_CAP_ROUND => self.set_cap(StrokeCap::Round),
            PanelEvent::Click(id) if id == ids::VECTOR_CAP_SQUARE => {
                self.set_cap(StrokeCap::Square)
            }
            PanelEvent::Click(id) if id == ids::VECTOR_JOIN_MITER => {
                self.set_join(StrokeJoin::Miter)
            }
            PanelEvent::Click(id) if id == ids::VECTOR_JOIN_ROUND => {
                self.set_join(StrokeJoin::Round)
            }
            PanelEvent::Click(id) if id == ids::VECTOR_JOIN_BEVEL => {
                self.set_join(StrokeJoin::Bevel)
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_DASH => {
                self.dash = slider_to_dash(v as f32);
                self.apply_to_selected = true;
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_GAP => {
                self.gap = slider_to_gap(v as f32);
                self.apply_to_selected = true;
            }
            _ => {}
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{WIDTH_MAX_PX, WIDTH_MIN_PX};
    use ph2d_a11y::NodeId;

    #[test]
    fn fresh_tool_defaults() {
        let t = VectorTool::new();
        assert_eq!(t.stroke_rgba(), [240, 240, 245, 255]); // white
        assert_eq!(t.fill_rgba(), [90, 150, 230, 255]); // blue
        assert_eq!(t.stroke_width_px(), DEFAULT_STROKE_WIDTH_PX);
    }

    #[test]
    fn width_slider_maps_normalized_to_px() {
        let mut t = VectorTool::new();
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_WIDTH, 0.0));
        assert_eq!(t.stroke_width_px(), WIDTH_MIN_PX);
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_WIDTH, 1.0));
        assert_eq!(t.stroke_width_px(), WIDTH_MAX_PX);
    }

    #[test]
    fn set_stroke_sets_colour_and_flags_apply() {
        let mut t = VectorTool::new();
        assert!(!t.take_apply_to_selected());
        t.set_stroke_rgba([220, 60, 60, 255]);
        assert_eq!(t.stroke_rgba(), [220, 60, 60, 255]);
        assert!(t.take_apply_to_selected());
        assert!(!t.take_apply_to_selected(), "drained");
    }

    #[test]
    fn set_fill_sets_colour_and_flags_apply() {
        let mut t = VectorTool::new();
        t.set_fill_rgba([70, 190, 90, 255]);
        assert_eq!(t.fill_rgba(), [70, 190, 90, 255]);
        assert!(t.take_apply_to_selected());
    }

    #[test]
    fn opacity_sliders_set_fill_and_stroke_alpha_and_flag_apply() {
        let mut t = VectorTool::new();
        // Fill Opacity → 0 % = invisible (replaces the old "None" button).
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_FILL_OPACITY, 0.0));
        assert_eq!(t.fill_rgba()[3], 0);
        assert!(t.take_apply_to_selected());
        // Fill Opacity → 100 %.
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_FILL_OPACITY, 1.0));
        assert_eq!(t.fill_rgba()[3], 255);
        // Stroke Opacity → 50 % ≈ 128.
        Tool::handle_panel_event(
            &mut t,
            PanelEvent::SetValue(ids::VECTOR_STROKE_OPACITY, 0.5),
        );
        assert_eq!(t.stroke_rgba()[3], 128);
        assert!(t.take_apply_to_selected());
    }

    #[test]
    fn foreign_node_id_ignored() {
        let mut t = VectorTool::new();
        let before = t.clone();
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(NodeId(999), 0.5));
        Tool::handle_panel_event(&mut t, PanelEvent::Click(NodeId(999)));
        assert_eq!(t, before);
    }

    #[test]
    fn mode_buttons_switch_the_draw_mode() {
        let mut t = VectorTool::new();
        assert_eq!(t.mode(), DrawMode::Select); // default
        for (id, want) in [
            (ids::VECTOR_MODE_PEN, DrawMode::Pen),
            (ids::VECTOR_MODE_NODE, DrawMode::Node),
            (ids::VECTOR_MODE_TEXT, DrawMode::Text),
            // O 5º pill: sem ele, desenhar uma forma deixava a fileira toda apagada.
            (ids::VECTOR_MODE_SHAPE, DrawMode::Shape),
            (ids::VECTOR_MODE_SELECT, DrawMode::Select),
        ] {
            Tool::handle_panel_event(&mut t, PanelEvent::Click(id));
            assert_eq!(t.mode(), want);
        }
        // Trocar de modo NAO e edicao de Style -> nunca marca recolor.
        assert!(!t.take_apply_to_selected());
    }

    /// **Gate do seam do catálogo:** o botão de CADA forma escolhe aquela forma E arma o
    /// gesto (modo Shape). Uma forma nova entra no catálogo e este teste já a cobre —
    /// nenhum botão pode nascer morto.
    #[test]
    fn every_catalog_button_selects_its_shape_and_arms_the_gesture() {
        let mut t = VectorTool::new();
        for (i, d) in crate::shapes::SHAPES.iter().enumerate() {
            Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::vector_shape_id(i)));
            assert_eq!(t.shape(), d.kind, "botao {i} nao escolheu {:?}", d.kind);
            assert_eq!(t.mode(), DrawMode::Shape, "escolher a forma arma o desenho");
            assert_eq!(t.draw_config().shape, d.kind, "o cfg espelha a forma");
        }
    }

    /// **Gate do campo genérico:** o campo `i` escreve o parâmetro `i` da forma ATIVA,
    /// clampado à faixa do catálogo — e os valores são POR-FORMA (mexer no raio da
    /// estrela não mexe no do retângulo: é o "último usado" de cada uma).
    #[test]
    fn a_shape_field_writes_the_active_shapes_parameter_and_is_per_shape() {
        let mut t = VectorTool::new();
        t.set_shape(ShapeKind::Star);
        // Campo 0 da estrela = Points (contagem: clampa e arredonda).
        Tool::handle_panel_event(
            &mut t,
            PanelEvent::SetValue(ids::vector_shape_field_id(0), 7.4),
        );
        assert!(
            (t.draw_config().values[0] - 7.0).abs() < 1e-9,
            "7.4 pontas -> 7"
        );
        Tool::handle_panel_event(
            &mut t,
            PanelEvent::SetValue(ids::vector_shape_field_id(0), 9_999.0),
        );
        assert!(
            (t.draw_config().values[0] - 60.0).abs() < 1e-9,
            "clampa no teto"
        );
        // Campo 2 da estrela = raio da ponta (px).
        Tool::handle_panel_event(
            &mut t,
            PanelEvent::SetValue(ids::vector_shape_field_id(2), 30.0),
        );
        assert!((t.shape_values(ShapeKind::Star)[2] - 30.0).abs() < 1e-9);
        // O poligono NAO foi tocado (os valores sao por-forma).
        assert!(
            (t.shape_values(ShapeKind::Polygon)[1] - 0.0).abs() < 1e-9,
            "o raio do poligono ficou onde estava"
        );
    }

    #[test]
    fn stroke_cap_join_dash_arms() {
        use crate::params::{DASH_MAX, GAP_MAX, StrokeCap, StrokeJoin};
        let mut t = VectorTool::new();
        Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_CAP_ROUND));
        assert_eq!(t.cap(), StrokeCap::Round);
        assert!(
            t.take_apply_to_selected(),
            "cap change restyles the selection"
        );
        Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_JOIN_BEVEL));
        assert_eq!(t.join(), StrokeJoin::Bevel);
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_DASH, 1.0));
        assert!((t.dash() - DASH_MAX).abs() < 1e-6);
        assert!(t.take_apply_to_selected());
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_GAP, 1.0));
        assert!((t.gap() - GAP_MAX).abs() < 1e-6);
        assert!(
            t.take_apply_to_selected(),
            "gap change restyles the selection"
        );
        // Snapshot carries them.
        let s = t.ui_snapshot();
        assert_eq!(s.cap, StrokeCap::Round);
        assert_eq!(s.join, StrokeJoin::Bevel);
        assert!((s.dash - DASH_MAX).abs() < 1e-6);
        assert!((s.gap - GAP_MAX).abs() < 1e-6);
    }

    #[test]
    fn ui_snapshot_round_trips_style() {
        let mut t = VectorTool::new();
        t.set_stroke_rgba([1, 2, 3, 255]);
        t.set_fill_rgba([4, 5, 6, 255]);
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_WIDTH, 0.5));
        t.set_shape(ShapeKind::Polygon);
        Tool::handle_panel_event(
            &mut t,
            PanelEvent::SetValue(ids::vector_shape_field_id(0), 7.0),
        );
        let s = t.ui_snapshot();
        assert_eq!(s.stroke, [1, 2, 3, 255]);
        assert_eq!(s.fill, [4, 5, 6, 255]);
        assert_eq!(s.stroke_width_px, t.stroke_width_px());
        assert_eq!(s.mode, DrawMode::Shape, "escolher a forma arma o desenho");
        assert_eq!(s.shape, ShapeKind::Polygon);
        assert!(
            (s.values[0] - 7.0).abs() < 1e-9,
            "o snapshot leva os valores"
        );
    }

    #[test]
    fn empty_panel_has_no_controls() {
        let t = VectorTool::new();
        let panel = t.build_panel();
        assert!(panel.controls.is_empty());
    }

    #[test]
    fn id_label_icon_stable() {
        let t = VectorTool::new();
        assert_eq!(t.id(), ToolId::new("vector"));
        assert_eq!(t.label(), "Vector");
        assert_eq!(t.icon_slug(), "vector");
    }
}
