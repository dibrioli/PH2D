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

/// O tipo de simetria cujo chip é `id` (`None` se não for um).
///
/// ⚠️ A tabela é percorrida a partir de [`ph2d_symmetry::SymmetryKind::ALL`], e não escrita à mão:
/// um tipo novo entra na lista do vocabulário e o chip dele passa a responder de graça. Uma tabela
/// paralela aqui nasceria incompleta no dia do quinto.
pub(crate) fn symmetry_kind(id: ph2d_a11y::NodeId) -> Option<ph2d_symmetry::SymmetryKind> {
    ph2d_symmetry::SymmetryKind::ALL
        .iter()
        .copied()
        .find(|k| crate::params::symmetry_kind_id(*k) == id)
}

use crate::params::{
    DrawMode, StrokeCap, StrokeJoin, VectorDrawConfig, VectorStyleSnapshot, marker_from_value,
    slider_to_dash, slider_to_gap, slider_to_opacity, slider_to_px,
};
use ph2d_vec_scene::{Marker, StrokeAlign};

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
    /// **Fidelity do lápis** (px de tela): a tolerância do decimador. Autorada pelo slider da seção
    /// Pencil; a shell empurra-a para o `Pencil` a cada frame, como faz com o estilo.
    pencil_fidelity_px: f64,
    /// **Estabilização do lápis** (0 = ponteiro cru). A shell aplica-a na ENTRADA, com a MESMA
    /// `lazy_mouse_step` do Painter — o valor é autorado aqui, o filtro corre lá.
    pencil_stabilizer: f32,
    /// De onde a largura do traço de lápis vem (W1d). Estado AUTORADO da ferramenta, como o
    /// estabilizador: o documento guarda o `WidthStops` que ele produziu, não a fonte.
    pencil_width_source: ph2d_vec_edit::pencil_width::WidthSource,
    /// **O estilo da SIMETRIA de desenho** (plano 25 W6.3) — que espelho, quantas cópias, funde
    /// ou não. O LUGAR da linha não está aqui: ele pertence ao desenho e viaja no componente dele
    /// (`ph2d_ecs::VecSymmetry`). Um centro guardado na ferramenta seria um campo que nunca se lê.
    symmetry: ph2d_symmetry::SymmetryStyle,
    /// **A forma PEGAJOSA do marquee do modo Node** (`Box | Lasso`) — estado AUTORADO da
    /// ferramenta, como o estabilizador do lápis. O documento não a guarda: ela decide o GESTO,
    /// não o que o gesto produz.
    marquee: crate::params::MarqueeShape,
    /// Canvas gesture: Pen (draw + edit) vs a drag-to-size shape. The shell
    /// mirrors this each frame to route canvas input (`vector_bridge`).
    mode: DrawMode,
    /// **Blend:** cada passo nasce ACIMA do anterior (`true`, o default) ou abaixo. Mora na tool
    /// porque é uma config de ferramenta — o painel a pinta pelo snapshot e o shell a lê na hora
    /// do blend, sem uma 2ª cópia para dessincronizar.
    blend_stack_up: bool,
    /// A forma ATIVA do catálogo (o que o modo `Shape` desenha).
    shape: ShapeKind,
    /// ⭐ **O artista ACABOU de escolher uma forma no catálogo** — um EVENTO, drenado pela shell
    /// ([`Self::take_shape_armed`]). A tool não sabe o que está selecionado, então não pode dizer
    /// por quanto tempo isso vale; quem segura o nível é a shell, que apaga o latch assim que a
    /// selecção muda. *O clique é da tool, a duração é de quem vê os dois lados.*
    shape_armed: bool,
    /// Os parâmetros de CADA forma, na unidade de UI (px para raios), indexados pelo
    /// discriminante. Guardar por-forma é o que faz cada uma lembrar do "último usado":
    /// mexer no raio da estrela não mexe no do retângulo.
    shape_values: [ShapeValues; MAX_SHAPES],
    /// Stroke cap / join + dash & gap as multiples of the stroke width
    /// (`dash = 0` = solid; `gap` is the space between dashes).
    cap: StrokeCap,
    join: StrokeJoin,
    align: StrokeAlign,
    dash: f64,
    gap: f64,
    /// **Pontas do traço** (arrowheads): a do começo e a do fim. Propriedade do stroke,
    /// como cap/join/dash — valem para o próximo caminho desenhado E para o selecionado
    /// (o bridge as reaplica no `take_apply_to_selected`).
    marker_start: Marker,
    marker_end: Marker,
    /// **Tamanho** da ponta (múltiplo do que a largura dita) e **arredondamento** das
    /// quinas dela. Style, como as pontas — o bridge os leva ao `StrokeSpec` de TODOS os
    /// caminhos selecionados. **Não** há um flag "bidirecional": esse estado é derivado das
    /// duas pontas ([`VectorStyleSnapshot::both_ends`]).
    marker_scale: f64,
    marker_round: f64,
    /// Set when a colour changes → the shell recolours the selected path.
    /// Drained by [`Self::take_apply_to_selected`].
    apply_to_selected: bool,
    /// **A largura foi AUTORADA neste gesto** — armado no braço `VECTOR_WIDTH`, drenado por
    /// [`Self::take_width_authored`].
    ///
    /// ⚠️ Ele existe porque *"a largura acompanha o tool"* e *"uma escolha de COR não pode
    /// reengrossar a linha"* são as duas metades de UMA pergunta, e o bridge respondia à segunda
    /// perguntando ao store se o **slider estava em ARRASTO** — uma enumeração de UMA forma de
    /// autorar. Digitar na caixa ao lado escreve o valor do slider sem nunca o pôr em `Dragging`,
    /// então a largura chegava ao tool e **nunca à forma selecionada** (Enio 2026-08-01: *"modificar
    /// o valor da caixa de texto ao lado do slider Width não muda o stroke"*). Quem sabe que uma
    /// largura foi autorada é quem a recebeu — e é aqui.
    width_authored: bool,
    /// **Uma COR foi autorada neste gesto** — `(fill, stroke)`, armado nos dois setters e drenado
    /// por [`Self::take_colour_authored`].
    ///
    /// ⚠️ Irmão exacto do [`Self::width_authored`], e existe pela decisão de produto do plano de
    /// UI/UX W4a: **escolher uma cor SOLTA o token** daquela propriedade (o *detach* do Figma).
    /// Sem ele o artista escolheria uma cor, o token continuaria a cobri-la, e a swatch mostraria
    /// um valor que a arte não usa — o pior estado possível para um controlo.
    ///
    /// ⚠️ **Arma só quando o valor MUDA.** O read-back do picker corre em TODO frame em que ele
    /// está aberto, então armar incondicionalmente soltaria o token no instante em que o picker
    /// abrisse — antes de o artista tocar em coisa nenhuma.
    colour_authored: (bool, bool),
}

impl Default for VectorTool {
    fn default() -> Self {
        Self {
            stroke: color_of("white").unwrap_or([240, 240, 245, 255]),
            fill: color_of("blue").unwrap_or([90, 150, 230, 255]),
            stroke_width_px: DEFAULT_STROKE_WIDTH_PX,
            pencil_fidelity_px: crate::params::PENCIL_FIDELITY_DEFAULT_PX,
            pencil_stabilizer: crate::params::PENCIL_STABILIZER_DEFAULT,
            pencil_width_source: ph2d_vec_edit::pencil_width::WidthSource::default(),
            symmetry: ph2d_symmetry::SymmetryStyle::default(),
            marquee: crate::params::MarqueeShape::default(),
            mode: DrawMode::Select,
            blend_stack_up: true,
            shape: ShapeKind::default(),
            shape_armed: false,
            shape_values: default_shape_values(),
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            align: StrokeAlign::Centre,
            dash: 0.0,
            gap: crate::params::GAP_DEFAULT,
            marker_start: Marker::None,
            marker_end: Marker::None,
            marker_scale: crate::params::DEFAULT_MARKER_SCALE,
            marker_round: crate::params::DEFAULT_MARKER_ROUND,
            apply_to_selected: false,
            width_authored: false,
            colour_authored: (false, false),
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

    /// A Fidelity autorada do lápis (px de tela) — a shell empurra-a para o `Pencil`.
    #[must_use]
    pub fn pencil_fidelity_px(&self) -> f64 {
        self.pencil_fidelity_px
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

    /// **Blend:** cada passo acima do anterior?
    #[must_use]
    pub fn blend_stack_up(&self) -> bool {
        self.blend_stack_up
    }

    pub fn set_blend_stack_up(&mut self, up: bool) {
        self.blend_stack_up = up;
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
        // ⚠️ **Marcado mesmo quando a forma NÃO muda.** O sinal é *"o artista carregou no
        // catálogo"*, e um diff de valor perderia o clique que re-arma a forma que já estava
        // acesa — que é precisamente o gesto de *"volta a mostrar-me o que vou desenhar"*.
        self.shape_armed = true;
    }

    /// Drena o *"acabei de escolher uma forma no catálogo"*. `true` uma vez por clique.
    ///
    /// ⚠️ **Quem o consome tem de o guardar até a SELECÇÃO mudar** — é o que separa
    /// *"armei o Polígono, mostra-me o Polígono"* de *"desenhei uma estrela, deixa-me ajustar as
    /// pontas dela"*. Os dois são o modo `Shape`; só a ordem dos gestos os distingue.
    pub fn take_shape_armed(&mut self) -> bool {
        std::mem::take(&mut self.shape_armed)
    }

    /// Os parâmetros da forma `k` na unidade de UI (px para raios).
    #[must_use]
    pub fn shape_values(&self, k: ShapeKind) -> ShapeValues {
        self.shape_values[k.as_u16() as usize]
    }

    /// Escreve o parâmetro `i` da forma que o modo DESENHA (o valor vem da caixa do painel, já
    /// na unidade de UI) — clampado à faixa que o catálogo declara.
    ///
    /// ⚠️ **O mesmo kind efectivo que semeou o campo** ([`Self::draw_config`]): a leitura e a
    /// escrita do slot têm de casar, senão o valor autorado cai num slot que o gesto não lê e a
    /// próxima forma desenhada não o herda — o controlo funciona uma vez, sobre a forma
    /// selecionada, e depois parece esquecer. Fora do modo Moldura isto é o que já era (o modo
    /// Shape desenha o catálogo, e os outros não desenham forma nenhuma).
    pub fn set_shape_field(&mut self, i: usize, v: f64) {
        if i >= MAX_SHAPE_FIELDS {
            return;
        }
        let k = self.mode.shape_kind(self.shape).unwrap_or(self.shape);
        let slot = &mut self.shape_values[k.as_u16() as usize];
        slot[i] = v;
        crate::shapes::clamp(k, slot);
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
    pub fn stroke_align(&self) -> StrokeAlign {
        self.align
    }
    #[must_use]
    pub fn dash(&self) -> f64 {
        self.dash
    }
    #[must_use]
    pub fn gap(&self) -> f64 {
        self.gap
    }

    /// As pontas do traço (começo / fim).
    #[must_use]
    pub fn marker_start(&self) -> Marker {
        self.marker_start
    }
    #[must_use]
    pub fn marker_end(&self) -> Marker {
        self.marker_end
    }

    /// Tamanho da ponta (múltiplo) e arredondamento das quinas dela — o bridge os leva ao
    /// `StrokeSpec` dos caminhos selecionados, ao lado das pontas.
    #[must_use]
    pub fn marker_scale(&self) -> f64 {
        self.marker_scale
    }
    #[must_use]
    pub fn marker_round(&self) -> f64 {
        self.marker_round
    }

    /// **Dupla via — o estado é DERIVADO**: há ponta nos dois extremos.
    #[must_use]
    pub fn both_ends(&self) -> bool {
        self.marker_start != Marker::None && self.marker_end != Marker::None
    }

    /// O clique em **Both Ends**. Não há flag a inverter — o que se inverte são as PONTAS:
    ///
    /// - **ligado → desligado:** limpa a ponta do COMEÇO (a linha volta a ser via única, com
    ///   a seta no fim, que é o que "uma via" significa num diagrama).
    /// - **desligado → ligado:** copia a ponta que existe para o outro extremo (o fim manda,
    ///   por ser o lado default de uma seta); se **nenhuma** existe, as duas nascem
    ///   [`Triangle`](Marker::Triangle) — senão o botão "acenderia" sem desenhar nada.
    ///
    /// Um `bool` guardado seria uma SEGUNDA verdade sobre "é bidirecional?", e divergiria no
    /// instante em que o usuário trocasse uma ponta pelo chip de Start/End.
    fn toggle_both_ends(&mut self) {
        if self.both_ends() {
            self.marker_start = Marker::None;
        } else {
            let head = match (self.marker_start, self.marker_end) {
                (_, end) if end != Marker::None => end,
                (start, _) if start != Marker::None => start,
                _ => crate::params::DEFAULT_BOTH_ENDS_MARKER,
            };
            self.marker_start = head;
            self.marker_end = head;
        }
        self.apply_to_selected = true;
    }

    /// Escreve o tamanho / o arredondamento da ponta (clampados à faixa que o painel
    /// registra na caixa) + marca a seleção para reestilizar — são Style, como as pontas.
    fn set_marker_scale(&mut self, v: f64) {
        self.marker_scale = crate::shapes::clamp_to(&crate::params::MARKER_SCALE, v);
        self.apply_to_selected = true;
    }
    fn set_marker_round(&mut self, v: f64) {
        self.marker_round = crate::shapes::clamp_to(&crate::params::MARKER_ROUND, v);
        self.apply_to_selected = true;
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
    /// Escolhe de que lado da linha a faixa cai + marca a seleção para reestilizar.
    ///
    /// Mesmo caminho de cap/join — alinhamento é Style, então vale para a forma que está na
    /// tela e não só para a próxima desenhada.
    fn set_stroke_align(&mut self, align: StrokeAlign) {
        self.align = align;
        self.apply_to_selected = true;
    }

    /// Escolhe a ponta do começo / do fim + marca a seleção para reestilizar (mesmo
    /// caminho de cap/join: a ponta é Style, então vale para o caminho que está na tela,
    /// não só para o próximo desenhado).
    fn set_marker_start(&mut self, m: Marker) {
        self.marker_start = m;
        self.apply_to_selected = true;
    }
    fn set_marker_end(&mut self, m: Marker) {
        self.marker_end = m;
        self.apply_to_selected = true;
    }

    /// Mode + shape parameters the shell mirrors to drive the `ShapeTool`.
    ///
    /// ⚠️ **Os valores são os do kind que o MODO desenha, não os da forma ativa do catálogo.**
    /// O gesto cozinha com `DrawMode::shape_kind` e lia os parâmetros de `self.shape` — duas
    /// perguntas diferentes que só concordavam no modo Shape. Ficou mudo enquanto a moldura era
    /// `Rectangle` (que ignora todo parâmetro); com ela em `RoundRect`, desenhar uma moldura com
    /// a estrela ativa passaria o **número de pontas** ao campo de raio.
    #[must_use]
    pub fn draw_config(&self) -> VectorDrawConfig {
        VectorDrawConfig {
            mode: self.mode,
            shape: self.shape,
            values: self.shape_values(self.mode.shape_kind(self.shape).unwrap_or(self.shape)),
            pencil_stabilizer: self.pencil_stabilizer,
            pencil_width_source: self.pencil_width_source,
            symmetry: self.symmetry,
            marquee: self.marquee,
        }
    }

    /// Set the stroke colour (picker read-back) + flag the selected path for
    /// recolour. `a = 0` is accepted (a fully-transparent stroke is unusual but
    /// not rejected here — the panel drives opaque picks).
    pub fn set_stroke_rgba(&mut self, rgba: [u8; 4]) {
        self.colour_authored.1 |= self.stroke != rgba;
        self.stroke = rgba;
        self.apply_to_selected = true;
    }

    /// Set the fill colour (picker read-back) + flag the selected path for
    /// recolour. `a = 0` ⇒ "None" (no fill).
    pub fn set_fill_rgba(&mut self, rgba: [u8; 4]) {
        self.colour_authored.0 |= self.fill != rgba;
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
            blend_stack_up: self.blend_stack_up,
            pencil_width_source: self.pencil_width_source,
            symmetry: self.symmetry,
            marquee: self.marquee,
            shape: self.shape,
            values: self.shape_values(self.shape),
            cap: self.cap,
            join: self.join,
            align: self.align,
            dash: self.dash,
            gap: self.gap,
            marker_start: self.marker_start,
            marker_end: self.marker_end,
            marker_scale: self.marker_scale,
            marker_round: self.marker_round,
        }
    }

    /// Drain the "recolour the selected path" request (set on any colour change).
    pub fn take_apply_to_selected(&mut self) -> bool {
        std::mem::take(&mut self.apply_to_selected)
    }

    /// Drena *"a largura foi autorada"* — verdadeiro pelo arrasto do slider **e** pela caixa
    /// numérica ao lado, porque as duas chegam pelo MESMO `SetValue(VECTOR_WIDTH)`.
    ///
    /// É o que o bridge pergunta antes de reescrever a largura da seleção; uma escolha de cor
    /// deixa-o falso, e é isso que impede um pick de reengrossar a linha.
    pub fn take_width_authored(&mut self) -> bool {
        std::mem::take(&mut self.width_authored)
    }

    /// `(fill, stroke)` — uma cor foi autorada desde a última drenagem? Ver
    /// [`Self::colour_authored`].
    pub fn take_colour_authored(&mut self) -> (bool, bool) {
        std::mem::take(&mut self.colour_authored)
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
        // O router inteiro mora no módulo filho — ver `tool_panel_event.rs`.
        self.apply_panel_event(event);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[path = "tool_tests.rs"]
mod tests;

/// A família que LÊ o documento — ver o cabeçalho do módulo.
#[path = "tool_adopt.rs"]
mod adopt;

/// O router de eventos do PAINEL — ver o cabeçalho do módulo.
#[path = "tool_panel_event.rs"]
mod panel_event;
