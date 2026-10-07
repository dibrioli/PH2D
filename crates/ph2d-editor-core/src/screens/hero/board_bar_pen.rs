//! As barras do quadro para a CANETA (W4, módulo filho de [`super`]): o botão da caneta e o painel
//! dela — caneta e marcador, as duas borrachas, o laser, as três predefinições da caneta escolhida
//! e a cor e a espessura da predefinição activa (o Miro edita-as com duplo-clique; aqui estão à
//! vista) —, a barra de estilo de uma selecção de traços, e o botão «Rascunho ↔ Final».

use super::{Item, btn_px, doc_color, gap, toolbar_rects};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon as icon};
use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;
use crate::zones::Rect;
use ph2d_board_edit::{Editor, PEN_WIDTHS, PenBox, PenPreset, Tool};
use ph2d_board_model::{BoardDoc, History, Pen, Rgba};
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::{BezPath, Color, VectorScene};

/// Um controlo da caneta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PenItem {
    /// A caneta ou o marcador.
    Kind(Pen),
    /// A predefinição `i` da caneta escolhida.
    Preset(usize),
    /// A cor da predefinição activa: `None` = a tinta do tema; `Some(i)` = a `i` do marcador.
    Color(Option<usize>),
    /// A espessura da predefinição activa (`PEN_WIDTHS`).
    Width(usize),
    /// A espessura dos traços SELECCIONADOS (`PEN_WIDTHS`).
    InkWidth(usize),
    /// A borracha; `true` = a de precisão.
    Eraser(bool),
    Laser,
}

/// O botão da caneta na barra curta.
pub(super) const PEN_TOOL: Item = Item::Tool(Tool::Pen(Pen::Pen));
/// Colunas do painel da caneta.
const FLYOUT_COLS: usize = 5;
/// O glifo do marcador: a altura da barra translúcida, em fracção do miolo.
const ICON_MARKER_H: f32 = 0.35; // LITERAL-PX-OK: desenho do glifo
const ICON_MARKER_ALPHA: f32 = 0.5; // LITERAL-PX-OK: a transparência do glifo do marcador
/// O glifo do laser: o ponto e o rasto.
const ICON_LASER_DOT: f32 = 0.18; // LITERAL-PX-OK: raio do ponto, fracção do miolo
/// O ponto da borracha de precisão (fracção do miolo).
const ICON_PRECISE_DOT: f32 = 0.12; // LITERAL-PX-OK: raio do ponto
/// A espessura desenhada da mais fina e da mais grossa, em fracção do miolo.
const ICON_WIDTH_RANGE: [f32; 2] = [0.08, 0.6]; // LITERAL-PX-OK: glifo das espessuras
/// A linha ondulada do botão do rascunho: os pontos de duas cúbicas, em fracções do miolo `(x, y)`.
const ICON_WAVE: [[f64; 2]; 7] = [
    [0.0, 0.5],
    [1.0 / 6.0, 0.25], // LITERAL-PX-OK: desenho do glifo (onda a ¼ da altura do meio)
    [1.0 / 3.0, 0.75], // LITERAL-PX-OK: desenho do glifo
    [0.5, 0.5],
    [2.0 / 3.0, 0.25], // LITERAL-PX-OK: desenho do glifo
    [5.0 / 6.0, 0.75], // LITERAL-PX-OK: desenho do glifo
    [1.0, 0.5],
];
/// A tolerância de achatar os círculos dos glifos (px).
const PATH_TOLERANCE: f64 = 0.1; // LITERAL-PX-OK: precisão do achatamento

fn kinds() -> [PenItem; 5] {
    [
        PenItem::Kind(Pen::Pen),
        PenItem::Kind(Pen::Highlighter),
        PenItem::Eraser(false),
        PenItem::Eraser(true),
        PenItem::Laser,
    ]
}

fn colors() -> impl Iterator<Item = PenItem> {
    std::iter::once(PenItem::Color(None))
        .chain((0..HIGHLIGHTER_RGBA.len()).map(|i| PenItem::Color(Some(i))))
}

/// O painel da caneta, na ordem da grelha (cada grupo numa linha nova).
fn rows() -> Vec<Vec<PenItem>> {
    vec![
        kinds().to_vec(),
        (0..3).map(PenItem::Preset).collect(),
        colors().collect(),
        (0..PEN_WIDTHS.len()).map(PenItem::Width).collect(),
    ]
}

pub(super) fn flyout_items() -> Vec<Item> {
    rows()
        .into_iter()
        .flatten()
        .chain((0..PEN_WIDTHS.len()).map(PenItem::InkWidth))
        .map(Item::Pen)
        .collect()
}

/// A ferramenta activa é da caneta? Então o painel dela está aberto.
#[must_use]
pub fn flyout_open(tool: Tool) -> bool {
    matches!(tool, Tool::Pen(_) | Tool::Eraser { .. } | Tool::Laser)
}

/// O painel à direita do botão da caneta.
#[must_use]
pub fn flyout_rects(area: Rect) -> Vec<(Item, Rect)> {
    let pen = toolbar_rects(area)
        .into_iter()
        .find(|(it, _)| *it == PEN_TOOL)
        .map(|(_, r)| r)
        .expect("a barra tem a caneta");
    let (s, g) = (btn_px(), gap());
    let x0 = pen.x + pen.w + Spacing::Sm.px();
    let mut out = Vec::new();
    let mut row = 0.0;
    for group in rows() {
        for chunk in group.chunks(FLYOUT_COLS) {
            for (c, it) in chunk.iter().enumerate() {
                let r = Rect::new(x0 + c as f32 * (s + g), pen.y + row * (s + g), s, s);
                out.push((Item::Pen(*it), r));
            }
            row += 1.0;
        }
    }
    out
}

/// A caneta de que o painel fala: a da mão, ou a última usada (com a borracha ou o laser na mão).
fn current(ed: &Editor) -> Pen {
    match ed.tool {
        Tool::Pen(k) => k,
        _ => ed.pen.last,
    }
}

fn color(c: Option<usize>, theme: Theme) -> Rgba {
    c.map_or_else(|| super::ink(theme), |i| Rgba(HIGHLIGHTER_RGBA[i]))
}

/// As predefinições de nascença: a caneta na tinta do tema, a vermelha e a azul fortes; o marcador
/// amarelo, verde e azul, largos. (O Miro não publica as dele.)
#[must_use]
pub fn pen_box(theme: Theme) -> PenBox {
    let p = |c: Option<usize>, w: usize| PenPreset {
        color: color(c, theme),
        width: PEN_WIDTHS[w],
    };
    PenBox {
        presets: [
            [p(None, 0), p(Some(5), 0), p(Some(6), 1)],
            [p(Some(8), 3), p(Some(7), 3), p(Some(6), 4)],
        ],
        ..PenBox::default()
    }
}

/// Os grupos da barra de estilo de uma selecção de TRAÇOS: a cor, a espessura, a opacidade.
pub(super) fn ink_groups() -> Vec<Vec<Item>> {
    vec![
        std::iter::once(Item::Stroke(Some(None)))
            .chain((0..HIGHLIGHTER_RGBA.len()).map(|i| Item::Stroke(Some(Some(i)))))
            .collect(),
        (0..PEN_WIDTHS.len())
            .map(|i| Item::Pen(PenItem::InkWidth(i)))
            .collect(),
        (0..super::OPACITIES.len()).map(Item::Opacity).collect(),
    ]
}

/// O controlo está «ligado»? `None` = não é da caneta.
pub(super) fn is_current(
    it: Item,
    ed: &Editor,
    sel_width: Option<f64>,
    theme: Theme,
) -> Option<bool> {
    let k = current(ed);
    let preset = ed.pen.current(k);
    Some(match it {
        PEN_TOOL => flyout_open(ed.tool),
        Item::Pen(PenItem::Kind(p)) => ed.tool == Tool::Pen(p),
        Item::Pen(PenItem::Preset(i)) => ed.pen.active(k) == i,
        Item::Pen(PenItem::Color(c)) => preset.color == color(c, theme),
        Item::Pen(PenItem::Width(i)) => preset.width == PEN_WIDTHS[i],
        Item::Pen(PenItem::InkWidth(i)) => sel_width == Some(PEN_WIDTHS[i]),
        Item::Pen(PenItem::Eraser(p)) => ed.tool == Tool::Eraser { precise: p },
        Item::Pen(PenItem::Laser) => ed.tool == Tool::Laser,
        _ => return None,
    })
}

/// Pinta o painel da caneta (aberto).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_flyout(
    scene: &mut VectorScene,
    ts: &mut ph2d_text::TextSystem,
    theme: Theme,
    hit: &mut crate::interaction::HitIndex,
    store: &crate::interaction::WidgetStore,
    area: Rect,
    ed: &Editor,
) {
    let grid = flyout_rects(area);
    super::look::paint_panel(scene, &grid, theme);
    for (it, r) in grid {
        let on = is_current(it, ed, None, theme).unwrap_or(false);
        super::paint_item(scene, ts, theme, hit, store, it, r, on);
        if let Item::Pen(PenItem::Preset(i)) = it {
            // A predefinição: um ponto da cor dela, do tamanho da espessura dela.
            let p = ed.pen.presets[usize::from(current(ed) == Pen::Highlighter)][i];
            let inset = Spacing::Xs.px();
            let inner = Rect::new(
                r.x + inset,
                r.y + inset,
                r.w - 2.0 * inset,
                r.h - 2.0 * inset,
            );
            let w = PEN_WIDTHS.iter().position(|w| *w == p.width).unwrap_or(0);
            let rad = width_px(w, inner) as f32 / 2.0 + inner.h * ICON_PRECISE_DOT;
            let (cx, cy) = (inner.x + inner.w / 2.0, inner.y + inner.h / 2.0);
            circle(scene, cx, cy, rad, super::shown(p.color, theme));
        }
    }
}

fn stroke_line(scene: &mut VectorScene, p: &BezPath, c: Color, width: f64) {
    super::look::stroke(scene, p, c, width, ph2d_board_model::Dash::Solid);
}

fn circle(scene: &mut VectorScene, cx: f32, cy: f32, r: f32, c: Color) {
    use ph2d_vector::Shape as _;
    let path = ph2d_vector::Circle::new((f64::from(cx), f64::from(cy)), f64::from(r))
        .to_path(PATH_TOLERANCE);
    scene.fill_path(
        &path,
        &ph2d_vector::Brush::Solid(c),
        ph2d_vector::Affine::IDENTITY,
    );
}

fn width_px(i: usize, inner: Rect) -> f64 {
    let t = i as f32 / (PEN_WIDTHS.len() - 1) as f32;
    let [lo, hi] = ICON_WIDTH_RANGE;
    f64::from(inner.h * (lo + (hi - lo) * t))
}

/// Pinta o ícone de um controlo da caneta (ou do rascunho). `false` = não é daqui.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_icon(
    scene: &mut VectorScene,
    theme: Theme,
    it: Item,
    inner: Rect,
    fg: Color,
    line: f64,
) -> bool {
    let (cx, cy) = (inner.x + inner.w / 2.0, inner.y + inner.h / 2.0);
    let mid = |scene: &mut VectorScene, width: f64, c: Color| {
        let mut p = BezPath::new();
        p.move_to((f64::from(inner.x), f64::from(cy)));
        p.line_to((f64::from(inner.x + inner.w), f64::from(cy)));
        stroke_line(scene, &p, c, width);
    };
    let px = ph2d_tokens::StrokeToken::Default.px();
    match it {
        PEN_TOOL | Item::Pen(PenItem::Kind(Pen::Pen)) => {
            icon(scene, IconId::VectorPencil, inner, fg, px)
        }
        Item::Pen(PenItem::Kind(Pen::Highlighter)) => {
            mid(
                scene,
                f64::from(inner.h * ICON_MARKER_H),
                fg.with_alpha(ICON_MARKER_ALPHA),
            );
        }
        Item::Pen(PenItem::Eraser(precise)) => {
            icon(scene, IconId::Erase, inner, fg, px);
            if precise {
                let r = inner.w * ICON_PRECISE_DOT;
                circle(scene, inner.x + inner.w - r, inner.y + inner.h - r, r, fg);
            }
        }
        Item::Pen(PenItem::Laser) => {
            let red = crate::paint::resolve(ColorToken::Danger, theme);
            let r = inner.w * ICON_LASER_DOT;
            let mut tail = BezPath::new();
            tail.move_to((f64::from(inner.x), f64::from(inner.y + inner.h)));
            tail.quad_to(
                (f64::from(inner.x + inner.w * ICON_LASER_DOT), f64::from(cy)),
                (f64::from(cx + r), f64::from(cy - r)),
            );
            stroke_line(scene, &tail, red.with_alpha(ICON_MARKER_ALPHA), line);
            circle(scene, cx + r, cy - r, r, red);
        }
        // O ponto da predefinição pinta-o o painel (só ele conhece as predefinições).
        Item::Pen(PenItem::Preset(_)) => {}
        Item::Pen(PenItem::Color(c)) => {
            let dot = crate::paint::frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, inner, dot, super::shown(color(c, theme), theme));
        }
        Item::Pen(PenItem::Width(i) | PenItem::InkWidth(i)) => mid(scene, width_px(i, inner), fg),
        Item::Sketch => {
            // Uma linha tremida: o «à mão».
            let at = |[fx, fy]: [f64; 2]| {
                (
                    f64::from(inner.x) + fx * f64::from(inner.w),
                    f64::from(inner.y) + fy * f64::from(inner.h),
                )
            };
            let w = ICON_WAVE;
            let mut p = BezPath::new();
            p.move_to(at(w[0]));
            p.curve_to(at(w[1]), at(w[2]), at(w[3]));
            p.curve_to(at(w[4]), at(w[5]), at(w[6]));
            stroke_line(scene, &p, fg, line);
        }
        _ => return false,
    }
    true
}

/// O balão de um controlo da caneta. `None` = não é daqui.
pub(super) fn tooltip_key(it: Item) -> Option<&'static str> {
    Some(match it {
        PEN_TOOL => "board.tool.pen",
        Item::Sketch => "board.sketch",
        Item::Pen(PenItem::Kind(Pen::Pen)) => "board.pen.pen",
        Item::Pen(PenItem::Kind(Pen::Highlighter)) => "board.pen.highlighter",
        Item::Pen(PenItem::Eraser(false)) => "board.pen.eraser",
        Item::Pen(PenItem::Eraser(true)) => "board.pen.eraser_precise",
        Item::Pen(PenItem::Laser) => "board.pen.laser",
        Item::Pen(PenItem::Preset(_)) => "board.pen.preset",
        Item::Pen(PenItem::Color(_)) => "board.pen.color",
        Item::Pen(PenItem::Width(_) | PenItem::InkWidth(_)) => "board.pen.width",
        _ => return None,
    })
}

/// Um clique num controlo da caneta (ou do rascunho). `false` = não é daqui. `sketch` = o modo do
/// quadro (o botão sem selecção troca-o).
pub(super) fn apply(
    ed: &mut Editor,
    doc: &mut BoardDoc,
    history: &mut History,
    sketch: &mut bool,
    theme: Theme,
    it: Item,
) -> bool {
    let k = current(ed);
    match it {
        PEN_TOOL => ed.tool = Tool::Pen(ed.pen.last),
        Item::Sketch => {
            if let Some(on) = ed.toggle_sketch(doc, history, *sketch) {
                *sketch = on;
            }
        }
        Item::Pen(p) => match p {
            PenItem::Kind(pen) => {
                ed.tool = Tool::Pen(pen);
                ed.pen.last = pen;
            }
            PenItem::Preset(i) => {
                ed.pen.choose(k, i);
                ed.tool = Tool::Pen(k);
            }
            PenItem::Color(c) => {
                let a = ed.pen.active(k);
                ed.pen.preset_mut(k, a).color = color(c, theme);
                ed.tool = Tool::Pen(k);
            }
            PenItem::Width(i) => {
                let a = ed.pen.active(k);
                ed.pen.preset_mut(k, a).width = PEN_WIDTHS[i];
                ed.tool = Tool::Pen(k);
            }
            PenItem::InkWidth(i) => ed.set_ink_width(doc, history, PEN_WIDTHS[i]),
            PenItem::Eraser(precise) => ed.tool = Tool::Eraser { precise },
            PenItem::Laser => ed.tool = Tool::Laser,
        },
        _ => return false,
    }
    true
}
