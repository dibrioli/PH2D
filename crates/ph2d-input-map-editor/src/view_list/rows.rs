//! As linhas das acções (maquete A): o nome, os chips (o nome inteiro da tecla; o do comando com a etiqueta «Pad»
//! e a seta da metade da manche), o «+» que escuta no lugar, o lixo, os dois números só com manche, e o pé.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, stroke_rounded_rect};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::widget::{
    SCROLLBAR_W, scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{ActionId, Binding, GamepadAxis, InputAction};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use super::{foot, footer, text};
use crate::doc::InputMapDoc;
use crate::ids::{self, ListHit};
use crate::interact::{Control, ControlRole};

const ROW_H: f32 = m::FIELD_H + m::SEG_INSET;
const CHIP_H: f32 = m::FIELD_H - m::SEG_INSET;
/// O nome da acção: o corpo de título da lista, maior que o texto dos chips (maquete A1).
const NAME_PX: f32 = m::LIST_TITLE_PX;
/// A linha dos números: o rótulo por cima e o campo.
const ZONE_H: f32 = m::VALUE_PX + m::LABEL_BOTTOM + m::FIELD_H + m::FIELD_ROW_GAP;
/// A folga de cada lado do texto de um chip: cabe o «×» do hover sem o texto sair do centro.
const CHIP_SIDE: f32 = m::KEY_MARK_BOX + m::SEG_INSET + m::SEG_PAD * 2.0;

fn row_h(a: &InputAction) -> f32 {
    if a.is_analog() { ROW_H + ZONE_H } else { ROW_H }
}

pub(super) fn paint_rows(
    doc: &mut InputMapDoc,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let shown = super::shown(doc);
    let actions: Vec<InputAction> = shown
        .iter()
        .filter_map(|id| doc.snap.map.get(*id).cloned())
        .collect();
    // A coluna dos nomes mede-se sobre TODAS as acções: escrever na busca nunca mexe os chips.
    let name_w = (doc.snap.map.actions().iter())
        .map(|a| text::title_width(ts, &a.name, NAME_PX))
        .fold(0.0_f32, f32::max)
        + m::LIST_PAD_X
        + m::CARD_PAD_X * 2.0;
    let analog = actions.iter().any(InputAction::is_analog);
    let laid = foot::lay(ts, foot(doc, analog), area.w);
    let list_h = (area.h - laid.h() - m::FIELD_ROW_GAP).max(0.0);
    let total: f32 = actions.iter().map(|a| row_h(a) + m::FIELD_ROW_GAP).sum();
    doc.list.scroll_max = (total - list_h).max(0.0);
    doc.list.scroll = doc.list.scroll.clamp(0.0, doc.list.scroll_max);
    // A barra do cartão quando a lista rola; as linhas encolhem para ela não tapar o lixo.
    let view = Rect::new(area.x, area.y, area.w, list_h);
    let bar = scrollbar_is_needed(total, list_h);
    let row_w = if bar {
        (area.w - SCROLLBAR_W - m::SEG_INSET).max(0.0)
    } else {
        area.w
    };
    doc.list.bar = None;
    if bar {
        // A barra discreta do cartão: só o polegar, fino, sem trilho; agarra-se pela faixa inteira; o acento
        // só enquanto se arrasta.
        let track = scrollbar_track_rect(view);
        let thumb = scrollbar_thumb_rect(track, doc.list.scroll, total, list_h);
        let color = if doc.list.thumb_drag.is_some() {
            ColorToken::Accent
        } else if doc.thumb_fade(ids::LIST_THUMB) > 0.5 {
            ColorToken::Text2
        } else {
            ColorToken::BorderEmph
        };
        let pill = Rect::new(
            thumb.x + thumb.w - m::BAR_TRACK,
            thumb.y,
            m::BAR_TRACK,
            thumb.h,
        );
        fill_rounded_rect(scene, pill, m::BAR_TRACK * 0.5, ink.color(color));
        let bar = tr("input_map_list.list_bar");
        doc.control(Control::new(
            ids::LIST_THUMB,
            thumb,
            ControlRole::ScrollThumb,
            bar,
        ));
        doc.list.bar = Some(super::state::Bar {
            thumb,
            track_h: track.h,
            content_h: total,
            visible_h: list_h,
        });
    }
    // A lista rola por LINHAS: a 1.ª pintada é a 1.ª que começa no deslocamento, encostada ao topo; o pé encosta
    // na última que coube inteira — nenhuma faixa vazia por cima nem por baixo.
    let mut top = 0.0_f32;
    let mut y = area.y;
    for a in &actions {
        let h = row_h(a);
        let starts = top;
        top += h + m::FIELD_ROW_GAP;
        if starts + f32::EPSILON < doc.list.scroll {
            continue;
        }
        if y + h > area.y + list_h + f32::EPSILON {
            break;
        }
        paint_row(
            doc,
            a,
            Rect::new(area.x, y, row_w, h),
            name_w,
            scene,
            ts,
            ink,
        );
        y += h + m::FIELD_ROW_GAP;
    }
    foot::paint(
        doc,
        &laid,
        Rect::new(area.x, y, area.w, laid.h()),
        scene,
        ts,
        ink,
    );
}

/// A frase do pé, UMA, pela prioridade: a escuta > a busca por tecla > a nota das zonas (com uma linha de manche à
/// vista) > o resumo (ou, no lugar dele, o aviso de uma tecla repetida por engano).
fn foot(doc: &InputMapDoc, analog: bool) -> footer::Phrase {
    if let Some(id) = doc.snap.listening {
        return footer::listening(&doc.snap, id);
    }
    if let Some(k) = doc.list.search_key {
        return footer::search(&doc.snap, k, &super::shown(doc));
    }
    if analog {
        return footer::zones();
    }
    footer::summary(&doc.snap)
}

/// O texto de um chip: a etiqueta do aparelho (só o comando) e o nome. A metade da manche diz só a manche
/// («Left Stick»): a direcção vai na seta desenhada ao lado.
fn chip_text(b: Binding) -> (Option<&'static str>, String) {
    match b {
        Binding::Key(_) => (None, footer::binding_label(b)),
        Binding::PadButton(_) => (Some(tr("input_map_list.pad_tag")), footer::binding_label(b)),
        Binding::PadAxis { axis, .. } => {
            let name = match axis {
                GamepadAxis::LeftStickX | GamepadAxis::LeftStickY => {
                    tr("input_map_list.left_stick").to_owned()
                }
                GamepadAxis::RightStickX | GamepadAxis::RightStickY => {
                    tr("input_map_list.right_stick").to_owned()
                }
                _ => footer::binding_label(b),
            };
            (Some(tr("input_map_list.pad_tag")), name)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_row(
    doc: &mut InputMapDoc,
    a: &InputAction,
    r: Rect,
    name_w: f32,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let head = Rect::new(r.x, r.y, r.w, ROW_H);
    fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(ColorToken::KanbanCard));
    if doc.snap.listening == Some(a.id) {
        // A linha à escuta: o acento vivo no traço do cartão (maquete A2), mais forte que o anel do foco.
        stroke_rounded_rect(
            scene,
            r,
            m::FIELD_RADIUS,
            StrokeToken::Default.px(),
            ink.color(ColorToken::Accent),
        );
        doc.census.outlined.push(r);
    }
    doc.control(Control::new(
        ListHit::Row(a.id).id(),
        r,
        ControlRole::ListItem,
        a.name.clone(),
    ));
    let hover = doc.list.hover_row == Some(a.id);
    let name_x = head.x + m::LIST_PAD_X;
    let name_r = Rect::new(
        name_x,
        head.y,
        name_w - m::LIST_PAD_X - m::CARD_PAD_X,
        head.h,
    );
    text::title(
        &mut doc.census,
        ts,
        scene,
        &a.name,
        name_r,
        NAME_PX,
        ink.color(ColorToken::Text1),
    );
    let trash_w = m::ICON_CELL;
    let right = head.x + head.w - trash_w - m::SEG_INSET;
    let mut x = head.x + name_w;
    let cy = head.y + (head.h - CHIP_H) * 0.5;
    for (n, b) in a.bindings.iter().enumerate() {
        let chip_id = ListHit::Chip(a.id, n).id();
        let chip = paint_chip(
            doc,
            chip_id,
            *b,
            Rect::new(x, cy, (right - x).max(0.0), CHIP_H),
            scene,
            ts,
            ink,
        );
        if hover {
            let g = m::KEY_MARK_BOX;
            let xr = Rect::new(
                chip.x + chip.w - g - m::SEG_INSET,
                chip.y + (chip.h - g) * 0.5,
                g,
                g,
            );
            let name = footer::binding_label(*b);
            let label = tr("input_map_list.remove_named").replace("{key}", &name);
            let x_id = ListHit::RemoveBinding(a.id, n).id();
            if doc.is_pressed(x_id) || doc.is_hovered(x_id) {
                let bg = if doc.is_pressed(x_id) {
                    ColorToken::AccentSoft
                } else {
                    ColorToken::CardFieldHover
                };
                fill_rounded_rect(scene, xr, m::SEG_RADIUS, ink.color(bg));
            }
            paint_icon(
                scene,
                IconId::Close,
                xr,
                ink.color(ColorToken::Text2),
                StrokeToken::Default.px(),
            );
            doc.control(Control::new(x_id, xr, ControlRole::Button, label));
        }
        x += chip.w + m::SEG_PAD * 2.0;
    }
    let plus = Rect::new(x, cy, (right - x).max(0.0), CHIP_H);
    paint_plus(
        doc,
        a.id,
        plus,
        hover || a.bindings.is_empty(),
        scene,
        ts,
        ink,
    );
    if hover {
        let g = m::KEY_MARK_BOX;
        let tr_box = Rect::new(right, head.y + (head.h - g) * 0.5, trash_w, g);
        let gl = Rect::new(tr_box.x + (trash_w - g) * 0.5, tr_box.y, g, g);
        let t_id = ListHit::Trash(a.id).id();
        if doc.is_pressed(t_id) || doc.is_hovered(t_id) {
            let bg = if doc.is_pressed(t_id) {
                ColorToken::AccentSoft
            } else {
                ColorToken::CardFieldHover
            };
            fill_rounded_rect(scene, tr_box, m::SEG_RADIUS, ink.color(bg));
        }
        paint_icon(
            scene,
            IconId::Trash,
            gl,
            ink.color(ColorToken::Text2),
            StrokeToken::Default.px(),
        );
        let label = tr("input_map_list.remove_action_tip");
        doc.control(Control::new(t_id, tr_box, ControlRole::Button, label));
    }
    if a.is_analog() {
        paint_zones(
            doc,
            a,
            Rect::new(r.x, r.y + ROW_H, r.w, ZONE_H),
            scene,
            ts,
            ink,
        );
    }
}

/// Um chip dentro de `room` (x, y e altura; a largura é o que sobra): o texto centrado, com a etiqueta «Pad» antes
/// e a seta da manche depois. A tecla que a busca procura enche-se do amarelo do destaque, sem mudar de tamanho.
fn paint_chip(
    doc: &mut InputMapDoc,
    id: ph2d_a11y::NodeId,
    b: Binding,
    room: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) -> Rect {
    let g = m::KEY_MARK_BOX;
    let (tag, label) = chip_text(b);
    let tag_w = tag.map_or(0.0, |t| text::body_width(ts, t, m::LABEL_PX));
    let tag_gap = if tag.is_some() { m::SEG_ICON_GAP } else { 0.0 };
    let tw = text::body_width(ts, &label, m::VALUE_PX);
    let arrow = crate::view_keyboard::half_icon(b);
    let aw = if arrow.is_some() {
        m::SEG_ICON_GAP + g
    } else {
        0.0
    };
    let content = tag_w + tag_gap + tw + aw;
    let chip = Rect::new(
        room.x,
        room.y,
        (content + CHIP_SIDE * 2.0).min(room.w),
        room.h,
    );
    let lit = doc.list.search_key.is_some_and(|k| b == Binding::Key(k));
    let bg = if lit {
        ColorToken::InputMapHighlight
    } else if doc.is_pressed(id) {
        ColorToken::AccentSoft
    } else if doc.is_hovered(id) {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    let (fg, dim) = if lit {
        (super::on_highlight(ink), super::on_highlight(ink))
    } else {
        (
            ink.color(ColorToken::Text1),
            ink.color(ColorToken::CardLabel),
        )
    };
    fill_rounded_rect(scene, chip, m::SEG_RADIUS, ink.color(bg));
    doc.control(Control::new(id, chip, ControlRole::Chip, label.clone()));
    if lit {
        doc.census.filled.push((chip, bg));
    }
    let mut cx = chip.x + ((chip.w - content) * 0.5).max(0.0);
    let end = chip.x + chip.w;
    if let Some(t) = tag {
        let tr_r = Rect::new(cx, chip.y, tag_w.min((end - cx).max(0.0)), chip.h);
        cx += text::body(&mut doc.census, ts, scene, t, tr_r, m::LABEL_PX, dim) + tag_gap;
    }
    let lr = Rect::new(cx, chip.y, tw.min((end - cx - aw).max(0.0)), chip.h);
    cx += text::body(&mut doc.census, ts, scene, &label, lr, m::VALUE_PX, fg);
    if let Some(icon) = arrow {
        let ar = Rect::new(cx + m::SEG_ICON_GAP, chip.y + (chip.h - g) * 0.5, g, g);
        paint_icon(scene, icon, ar, fg, StrokeToken::Default.px());
        doc.census.icons.push((ar, icon));
    }
    chip
}

/// O «+» fantasma (só sob o ponteiro, ou numa linha ainda sem ligação), ou — à escuta — a caixa tracejada no lugar
/// onde a tecla vai cair.
#[allow(clippy::too_many_arguments)]
fn paint_plus(
    doc: &mut InputMapDoc,
    id: ActionId,
    room: Rect,
    visible: bool,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let g = m::KEY_MARK_BOX;
    if doc.snap.listening == Some(id) {
        let key = if doc.snap.gamepad_connected {
            "input_map_list.listen_pad"
        } else {
            "input_map_list.listen_key"
        };
        let msg = tr(key);
        let w = (ts.prefix_width(msg, m::VALUE_PX) + m::SEG_INSET * 2.0).min(room.w);
        let r = Rect::new(room.x, room.y, w, room.h);
        // A caixa à escuta tingida de azul por baixo do tracejado (maquete A2); o tingido pulsa (`doc.pulse`,
        // estático sem movimento).
        let lid = ListHit::Listen(id).id();
        if doc.pulse(lid).is_none_or(|t| t > 0.5) {
            fill_rounded_rect(scene, r, m::SEG_RADIUS, ink.color(ColorToken::AccentSoft));
        }
        super::listening_ring(&mut doc.census, scene, r, ink);
        let inner = Rect::new(r.x + m::SEG_INSET, r.y, r.w - m::SEG_INSET * 2.0, r.h);
        text::body(
            &mut doc.census,
            ts,
            scene,
            msg,
            inner,
            m::VALUE_PX,
            ink.color(ColorToken::Text1),
        );
        let label = tr("input_map_list.add_key_tip");
        doc.control(Control::new(lid, r, ControlRole::Button, label));
        return;
    }
    // Numa linha SEM teclas: a caixa tracejada «No key» (o desenho da escuta, neutro); clicar nela escuta.
    let empty = doc.snap.map.get(id).is_some_and(|a| a.bindings.is_empty());
    if empty {
        return paint_no_key(doc, id, room, scene, ts, ink);
    }
    let r = Rect::new(room.x, room.y, (g + m::SEG_INSET * 2.0).min(room.w), room.h);
    // O alvo fica registado mesmo invisível: só se chega a ele com o ponteiro na linha, e aí ele desenha-se.
    let lid = ListHit::Listen(id).id();
    doc.control(Control::new(
        lid,
        r,
        ControlRole::Button,
        tr("input_map_list.add_key_tip"),
    ));
    if !visible {
        return;
    }
    if doc.list.hover_row == Some(id) {
        let bg = if doc.is_pressed(lid) {
            ColorToken::AccentSoft
        } else {
            ColorToken::CardFieldHover
        };
        fill_rounded_rect(scene, r, m::SEG_RADIUS, ink.color(bg));
    }
    let gl = Rect::new(r.x + (r.w - g) * 0.5, r.y + (r.h - g) * 0.5, g, g);
    paint_icon(
        scene,
        IconId::Plus,
        gl,
        ink.color(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
    doc.census.plus.push(id);
}

/// A caixa tracejada «No key» da linha sem teclas: o tracejado da escuta em aresta neutra, o texto em `Text2`; sob
/// o ponteiro ou premida acende como o «+». É o alvo que escuta.
fn paint_no_key(
    doc: &mut InputMapDoc,
    id: ActionId,
    room: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let msg = tr("input_map_list.no_key");
    let w = (ts.prefix_width(msg, m::VALUE_PX) + m::SEG_INSET * 2.0).min(room.w);
    let r = Rect::new(room.x, room.y, w, room.h);
    let lid = ListHit::Listen(id).id();
    let bg = if doc.is_pressed(lid) {
        Some(ColorToken::AccentSoft)
    } else {
        (doc.list.hover_row == Some(id)).then_some(ColorToken::CardFieldHover)
    };
    if let Some(bg) = bg {
        fill_rounded_rect(scene, r, m::SEG_RADIUS, ink.color(bg));
    }
    ph2d_editor_core::paint::stroke_rounded_rect_dashed(
        scene,
        r,
        m::SEG_RADIUS,
        m::FIELD_BORDER,
        m::LOOSE_DASH,
        ink.color(ColorToken::BorderStrong),
    );
    let inner = Rect::new(r.x + m::SEG_INSET, r.y, r.w - m::SEG_INSET * 2.0, r.h);
    text::body(
        &mut doc.census,
        ts,
        scene,
        msg,
        inner,
        m::VALUE_PX,
        ink.color(ColorToken::Text2),
    );
    let label = tr("input_map_list.add_key_tip");
    doc.control(Control::new(lid, r, ControlRole::Button, label));
    doc.census.plus.push(id);
}

/// «Dead zone (0 to 1)» e «Press at (0 to 1)»: dois campos lado a lado, arrastáveis, com a marca «D»/«P».
/// Sem o ◆ da maquete: o losango é a marca do Auto Key, e aqui não há o que animar — seria um controlo morto.
fn paint_zones(
    doc: &mut InputMapDoc,
    a: &InputAction,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let pad = m::LIST_PAD_X;
    let half = (r.w - pad * 2.0 - m::FIELD_GAP) * 0.5;
    for (i, (hit, label, value)) in [
        (ListHit::Dead(a.id), "input_map_list.dead", a.dead_zone),
        (ListHit::Press(a.id), "input_map_list.press", a.press_point),
    ]
    .into_iter()
    .enumerate()
    {
        let x = r.x + pad + (half + m::FIELD_GAP) * i as f32;
        let zone = if i == 0 {
            super::state::Zone::Dead
        } else {
            super::state::Zone::Press
        };
        let handle_hit = if i == 0 {
            ListHit::DeadHandle(a.id)
        } else {
            ListHit::PressHandle(a.id)
        };
        let lr = Rect::new(x, r.y, half, m::VALUE_PX);
        // O aviso do limite toma o lugar do rótulo, colado ao campo, até à próxima edição válida.
        let warn = (doc.list.warn)
            .filter(|(w, z, _)| *w == a.id && *z == zone)
            .map(|(_, _, c)| c);
        let warned = warn.map(|c| c.sentence(zone.max()));
        let (above, tint) = warned
            .as_deref()
            .map_or((tr(label), ColorToken::CardLabel), |s| {
                (s, ColorToken::Warn)
            });
        text::body(
            &mut doc.census,
            ts,
            scene,
            above,
            lr,
            m::VALUE_PX,
            ink.color(tint),
        );
        let f = Rect::new(x, lr.y + lr.h + m::LABEL_BOTTOM, half, m::FIELD_H);
        let live = doc
            .list
            .scrub
            .is_some_and(|s| s.action == a.id && s.zone == zone);
        let typing = doc
            .list
            .typing
            .as_ref()
            .filter(|t| t.action == a.id && t.zone == zone)
            .map(|t| t.buffer.clone());
        let hover = doc.is_hovered(hit.id()) || doc.is_hovered(handle_hit.id());
        crate::field::paint_frame(scene, ink, f, hover || live, typing.is_some() || live);
        let mark = tr(if i == 0 {
            "input_map_try.dead_mark"
        } else {
            "input_map_try.press_mark"
        });
        let mw = ts.prefix_width(mark, m::VALUE_PX);
        let handle = Rect::new(f.x, f.y, mw + m::SEG_INSET * 2.0, f.h);
        let mr = Rect::new(f.x + m::SEG_INSET, f.y, mw, f.h);
        let mark_tint = if live || doc.is_hovered(handle_hit.id()) {
            ColorToken::Accent
        } else {
            ColorToken::CardLabel
        };
        text::body(
            &mut doc.census,
            ts,
            scene,
            mark,
            mr,
            m::VALUE_PX,
            ink.color(mark_tint),
        );
        let vx = handle.x + handle.w;
        let vr = Rect::new(vx, f.y, (f.x + f.w - m::SEG_INSET - vx).max(0.0), f.h);
        let v = typing
            .clone()
            .unwrap_or_else(|| crate::field::format(value));
        let fid = hit.id();
        if typing.is_some() {
            let c = doc.list.caret;
            let on = doc.caret_on(fid);
            crate::field::paint_edit(scene, ts, ink, vr, vx, &v, c.caret, c.anchor, on);
        }
        text::body(
            &mut doc.census,
            ts,
            scene,
            &v,
            vr,
            m::VALUE_PX,
            ink.color(ColorToken::Text1),
        );
        let mut body = Control::new(fid, f, ControlRole::NumberField, tr(label));
        body.look = ph2d_editor_core::doc_surface::PointerLook::Text;
        doc.control(body);
        let mut grip = Control::new(
            handle_hit.id(),
            handle,
            ControlRole::DragHandle,
            tr("input_map_list.drag_tip"),
        );
        grip.focusable = false;
        doc.control(grip);
        doc.census.numbers.push(a.id);
    }
}
