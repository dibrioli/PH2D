//! As barras do quadro para ORGANIZAR (W5, módulo filho de [`super`]): o botão do Frame na barra
//! curta; na barra de contexto agrupar, desagrupar, travar e a ordem — e, com duas ou mais
//! unidades, alinhar e distribuir —; e o aviso de uma recusa, pintado no próprio quadro (a frase que
//! o artista lê; nunca só o terminal).
//!
//! ⚠️ Um controlo que não faria nada à selecção de agora NÃO se pinta (agrupar uma coisa só,
//! desagrupar sem grupo, alinhar uma unidade): um botão sem efeito é um controlo morto.

use super::look::stroke;
use super::{Item, Selected};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon as icon, paint_text_centered, rect_for_label, resolve};
use crate::zones::Rect;
use ph2d_board_edit::{Align, Axis, Command, Editor, Order, Tool};
use ph2d_board_model::{BoardDoc, Dash, History};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{BezPath, Color, VectorScene};

/// Um controlo de organizar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrgItem {
    Group,
    Ungroup,
    Lock,
    Order(Order),
    Align(Align),
    Distribute(Axis),
    /// A cor de nascença de um frame (o branco do Miro) — para um frame pintado poder voltar a ela.
    Paper,
}

/// O botão do Frame na barra curta (tecla `F`, a do Miro).
pub(super) const FRAME_TOOL: Item = Item::Tool(Tool::Frame);

const ORDERS: [Order; 4] = [Order::Front, Order::Forward, Order::Backward, Order::Back];
const ALIGNS: [Align; 6] = [
    Align::Left,
    Align::CenterH,
    Align::Right,
    Align::Top,
    Align::CenterV,
    Align::Bottom,
];

/// O grupo de organizar (agrupar, travar, ordem) — num frame só travar (os frames ficam sempre
/// atrás: a ordem não é deles).
fn order_group(sel: Selected) -> Vec<Item> {
    let mut v = vec![
        Item::Organize(OrgItem::Group),
        Item::Organize(OrgItem::Ungroup),
        Item::Organize(OrgItem::Lock),
    ];
    if sel != Selected::Frames {
        v.extend(ORDERS.map(|o| Item::Organize(OrgItem::Order(o))));
    }
    v
}

fn align_group() -> Vec<Item> {
    ALIGNS
        .map(|a| Item::Organize(OrgItem::Align(a)))
        .into_iter()
        .chain([Axis::Horizontal, Axis::Vertical].map(|x| Item::Organize(OrgItem::Distribute(x))))
        .collect()
}

/// As cores de um frame: o branco de nascença, nenhuma, e as pastéis do marcador.
pub(super) fn frame_fill_group() -> Vec<Item> {
    std::iter::once(Item::Organize(OrgItem::Paper))
        .chain(std::iter::once(Item::Fill(None)))
        .chain(super::PASTELS.map(|i| Item::Fill(Some(i))))
        .collect()
}

/// Os grupos de organizar que a tabela regista (todos, para qualquer selecção).
pub(super) fn groups(sel: Selected) -> Vec<Vec<Item>> {
    match sel {
        Selected::TextNote | Selected::TextShape => Vec::new(),
        _ => vec![order_group(sel), align_group()],
    }
}

/// O que a selecção de AGORA permite — para não pintar um controlo sem efeito.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Now {
    /// Quantas coisas se moveriam separadas (grupos inteiros, soltos).
    units: usize,
    any_group: bool,
    all_locked: bool,
}

impl Now {
    pub(super) fn of(ed: &Editor, doc: &BoardDoc) -> Self {
        let els: Vec<_> = ed.selection().iter().filter_map(|id| doc.get(*id)).collect();
        let mut keys = std::collections::BTreeSet::new();
        for el in els.iter().filter(|el| el.connector().is_none() && !el.locked()) {
            keys.insert(el.outer_group().map_or((0, el.id.0), |g| (1, g.0)));
        }
        Self {
            units: keys.len(),
            any_group: els.iter().any(|el| el.outer_group().is_some()),
            all_locked: !els.is_empty() && els.iter().all(|el| el.locked()),
        }
    }

    /// `it` faria alguma coisa agora? (`None` = não é de organizar.)
    pub(super) fn shows(self, it: Item) -> Option<bool> {
        let Item::Organize(o) = it else {
            return None;
        };
        Some(match o {
            OrgItem::Group => self.units > 1,
            OrgItem::Ungroup => self.any_group,
            OrgItem::Lock | OrgItem::Order(_) | OrgItem::Paper => true,
            OrgItem::Align(_) => self.units > 1,
            OrgItem::Distribute(_) => self.units > 2,
        })
    }

    /// O cadeado aceso = tudo travado; o papel aceso = o frame está no branco de nascença.
    pub(super) fn is_on(self, it: Item, fill: Option<ph2d_board_model::Rgba>) -> Option<bool> {
        match it {
            Item::Organize(OrgItem::Lock) => Some(self.all_locked),
            Item::Organize(OrgItem::Paper) => {
                Some(fill == Some(ph2d_board_model::Rgba(ph2d_board_model::FRAME_FILL)))
            }
            Item::Organize(_) => Some(false),
            _ => None,
        }
    }
}

pub(super) fn tooltip_key(it: Item) -> Option<&'static str> {
    Some(match it {
        FRAME_TOOL => "board.tool.frame",
        Item::Organize(o) => match o {
            OrgItem::Group => "board.org.group",
            OrgItem::Ungroup => "board.org.ungroup",
            OrgItem::Lock => "board.org.lock",
            OrgItem::Order(Order::Front) => "board.org.front",
            OrgItem::Order(Order::Forward) => "board.org.forward",
            OrgItem::Order(Order::Backward) => "board.org.backward",
            OrgItem::Order(Order::Back) => "board.org.back",
            OrgItem::Align(Align::Left) => "board.org.align_left",
            OrgItem::Align(Align::CenterH) => "board.org.align_center_h",
            OrgItem::Align(Align::Right) => "board.org.align_right",
            OrgItem::Align(Align::Top) => "board.org.align_top",
            OrgItem::Align(Align::CenterV) => "board.org.align_center_v",
            OrgItem::Align(Align::Bottom) => "board.org.align_bottom",
            OrgItem::Distribute(Axis::Horizontal) => "board.org.distribute_h",
            OrgItem::Distribute(Axis::Vertical) => "board.org.distribute_v",
            OrgItem::Paper => "board.org.paper",
        },
        _ => return None,
    })
}

/// O comando de cada controlo.
fn command(o: OrgItem) -> Command {
    match o {
        OrgItem::Group => Command::Group,
        OrgItem::Ungroup => Command::Ungroup,
        OrgItem::Lock => Command::Lock,
        OrgItem::Order(x) => Command::Order(x),
        OrgItem::Align(a) => Command::Align(a),
        OrgItem::Distribute(x) => Command::Distribute(x),
        OrgItem::Paper => unreachable!("o papel é um estilo, não um comando"),
    }
}

/// Um clique num controlo de organizar. `true` = era um.
pub(super) fn apply(ed: &mut Editor, doc: &mut BoardDoc, history: &mut History, it: Item) -> bool {
    match it {
        FRAME_TOOL => {
            ed.tool = Tool::Frame;
            true
        }
        Item::Organize(OrgItem::Paper) => {
            let paper = ph2d_board_model::Rgba(ph2d_board_model::FRAME_FILL);
            ed.set_style(doc, history, |s| {
                s.fill = Some(paper);
                s.text_color = paper.readable_ink();
            });
            true
        }
        Item::Organize(o) => {
            ed.organize(doc, history, command(o));
            true
        }
        _ => false,
    }
}

/// Uma fracção `f` do miolo, em px.
fn at(r: Rect, fx: f32, fy: f32) -> (f64, f64) {
    (f64::from(r.x + r.w * fx), f64::from(r.y + r.h * fy))
}

fn rect_path(r: Rect, x0: f32, y0: f32, x1: f32, y1: f32) -> BezPath {
    let mut p = BezPath::new();
    p.move_to(at(r, x0, y0));
    p.line_to(at(r, x1, y0));
    p.line_to(at(r, x1, y1));
    p.line_to(at(r, x0, y1));
    p.close_path();
    p
}

fn line(r: Rect, a: (f32, f32), b: (f32, f32)) -> BezPath {
    let mut p = BezPath::new();
    p.move_to(at(r, a.0, a.1));
    p.line_to(at(r, b.0, b.1));
    p
}

/// Os glifos: o frame (o «#» do Miro), a ordem (dois quadrados, o que se move cheio) e os de
/// alinhar (a guia e as duas barras). `true` = pintou.
pub(super) fn paint_icon(
    scene: &mut VectorScene,
    theme: Theme,
    it: Item,
    inner: Rect,
    fg: Color,
    w: f64,
) -> bool {
    let solid = |scene: &mut VectorScene, p: &BezPath| scene.fill_path(p, &ph2d_vector::Brush::Solid(fg), ph2d_vector::Affine::IDENTITY);
    match it {
        FRAME_TOOL => {
            // LITERAL-PX-OK: o «#» — duas verticais e duas horizontais a ¼ e ¾, a passar a borda.
            for f in [0.25, 0.75] {
                stroke(scene, &line(inner, (f, 0.0), (f, 1.0)), fg, w, Dash::Solid);
                stroke(scene, &line(inner, (0.0, f), (1.0, f)), fg, w, Dash::Solid);
            }
        }
        Item::Organize(OrgItem::Group) => icon(scene, IconId::Group, inner, fg, w as f32),
        Item::Organize(OrgItem::Ungroup) => icon(scene, IconId::Ungroup, inner, fg, w as f32),
        Item::Organize(OrgItem::Lock) => icon(scene, IconId::Lock, inner, fg, w as f32),
        Item::Organize(OrgItem::Paper) => {
            let paper = ph2d_board_model::Rgba(ph2d_board_model::FRAME_FILL);
            let r = crate::paint::frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, inner, r, super::look::shown(paper, theme));
            stroke(scene, &rect_path(inner, 0.0, 0.0, 1.0, 1.0), fg, w, Dash::Solid);
        }
        Item::Organize(OrgItem::Order(o)) => {
            // LITERAL-PX-OK: os dois quadrados do glifo, em fracções do miolo.
            let back = rect_path(inner, 0.0, 0.0, 0.62, 0.62);
            let front = rect_path(inner, 0.38, 0.38, 1.0, 1.0);
            let moved_on_top = matches!(o, Order::Front | Order::Forward);
            let (under, over) = if moved_on_top { (back, front) } else { (front, back) };
            stroke(scene, &under, fg, w, Dash::Solid);
            solid(scene, &over);
            if matches!(o, Order::Forward | Order::Backward) {
                // Um passo: o tracejado distingue-o de «à frente/atrás de tudo».
                stroke(scene, &rect_path(inner, 0.0, 0.0, 1.0, 1.0), fg, w, Dash::Dotted);
            }
        }
        Item::Organize(OrgItem::Align(a)) => {
            // LITERAL-PX-OK: a guia e duas barras (larga e curta), em fracções do miolo.
            let (guide, bars): ((f32, f32, f32, f32), [(f32, f32, f32, f32); 2]) = match a {
                Align::Left => ((0.0, 0.0, 0.0, 1.0), [(0.0, 0.15, 0.9, 0.4), (0.0, 0.6, 0.55, 0.85)]),
                Align::Right => ((1.0, 0.0, 1.0, 1.0), [(0.1, 0.15, 1.0, 0.4), (0.45, 0.6, 1.0, 0.85)]),
                Align::CenterH => ((0.5, 0.0, 0.5, 1.0), [(0.05, 0.15, 0.95, 0.4), (0.25, 0.6, 0.75, 0.85)]),
                Align::Top => ((0.0, 0.0, 1.0, 0.0), [(0.15, 0.0, 0.4, 0.9), (0.6, 0.0, 0.85, 0.55)]),
                Align::Bottom => ((0.0, 1.0, 1.0, 1.0), [(0.15, 0.1, 0.4, 1.0), (0.6, 0.45, 0.85, 1.0)]),
                Align::CenterV => ((0.0, 0.5, 1.0, 0.5), [(0.15, 0.05, 0.4, 0.95), (0.6, 0.25, 0.85, 0.75)]),
            };
            stroke(scene, &line(inner, (guide.0, guide.1), (guide.2, guide.3)), fg, w, Dash::Solid);
            for b in bars {
                solid(scene, &rect_path(inner, b.0, b.1, b.2, b.3));
            }
        }
        Item::Organize(OrgItem::Distribute(x)) => {
            // LITERAL-PX-OK: três barras com vãos iguais entre as duas guias das pontas.
            let spans = [(0.1, 0.25), (0.42, 0.58), (0.75, 0.9)];
            for (a, b) in spans {
                let r = match x {
                    Axis::Horizontal => rect_path(inner, a, 0.2, b, 0.8),
                    Axis::Vertical => rect_path(inner, 0.2, a, 0.8, b),
                };
                solid(scene, &r);
            }
            let ends = match x {
                Axis::Horizontal => [((0.0, 0.0), (0.0, 1.0)), ((1.0, 0.0), (1.0, 1.0))],
                Axis::Vertical => [((0.0, 0.0), (1.0, 0.0)), ((0.0, 1.0), (1.0, 1.0))],
            };
            for (a, b) in ends {
                stroke(scene, &line(inner, a, b), fg, w, Dash::Solid);
            }
        }
        _ => return false,
    }
    true
}

/// ⭐ **O aviso de uma recusa**, no topo do quadro, enquanto é fresco (a duração de um toast).
pub(super) fn paint_notice(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    area: Rect,
    key: &'static str,
) {
    let text = ph2d_i18n::tr(key);
    let font = TypeToken::Sm.px();
    let w = rect_for_label(text_system.prefix_width(text, font)).min(area.w);
    let h = Spacing::Xl2.px();
    let r = Rect::new(
        area.x + (area.w - w) / 2.0,
        area.y + Spacing::Md.px(),
        w,
        h,
    );
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, theme));
    paint_text_centered(text_system, scene, text, r, font, resolve(ColorToken::Text1, theme));
}
