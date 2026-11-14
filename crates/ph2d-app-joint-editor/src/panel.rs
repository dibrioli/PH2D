//! **O painel da direita** (dono 13/11, W3d): uma coluna de cartões no idioma do Inspector — o título que
//! recolhe (› quando fechado), o rótulo pequeno ACIMA de cada caixa, dois campos por fila (a letra ou o
//! ícone dentro da caixa, a unidade no rótulo), o segmentado com o rótulo por cima e a caixa de marcar na
//! moldura. A coluna rola com a roda e com a barra arrastada; a largura vem da divisória.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, paint_icon, paint_text_title_elided_tracked, rect_to_vello,
};
use ph2d_editor_core::widget::card_field::{Glyph, paint_label};
use ph2d_editor_core::widget::{ScrollbarState, paint_scrollbar, scrollbar_track_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_joint_templates::JointKind;
use ph2d_tokens::{ColorToken, StrokeToken};

use crate::doc::{Field, Flag, Hit, Menu, kind_key, scope_options};
use crate::widgets::{FIELD_MIN_W, Ui, seg_h, seg_rows};

/// Um controlo de um cartão.
enum Item {
    /// O número, a pega (a letra é uma chave i18n), o rótulo e a unidade (chaves; `""` = sem unidade).
    Num(Field, Glyph<'static>, &'static str, &'static str),
    /// A caixa «On» e o nome dela.
    Check(Flag, &'static str, bool),
    /// Uma frase (parte-se em linhas na largura do cartão).
    Note(&'static str),
    /// Uma lista suspensa com o rótulo por cima.
    Menu(Menu),
    /// O resumo da montagem (à esquerda de «Inspector»).
    Line(String),
    Inspect,
    /// Uma escolha segmentada a toda a largura, com o rótulo por cima.
    Scope(&'static str, Vec<(String, Hit, bool)>),
}

/// Um cartão: o título (chave i18n, também a chave do recolhido) e os controlos.
struct Group(&'static str, Vec<Item>);

/// Os cartões do tipo `k`, lidos da 1.ª junta do alcance. Uma tabela: um controlo por linha.
#[rustfmt::skip]
fn groups_of(k: JointKind, info: &ph2d_editor_core::screens::hero::InspectorJointInfo, wheelbase: bool, drive: Vec<(String, Hit, bool)>) -> Vec<Group> {
    use Item::{Check, Num};
    use Glyph::{Icon, Text as Key};
    let metres = k.limits_in_metres();
    let lim = if metres { "joint_editor.unit.m" } else { "joint_editor.unit.deg" };
    let spd = if k.motor_in_metres() { "joint_editor.unit.m_per_s" } else { "joint_editor.unit.deg_per_s" };
    let metre = "joint_editor.unit.m";
    let speed = || Num(Field::MotorSpeed, Key("joint_editor.glyph.speed"), "joint_editor.label.speed", spd);
    let force = || Num(Field::MotorForce, Key("joint_editor.glyph.force"), "joint_editor.label.force", "joint_editor.unit.n");
    let stiff = |l| Num(Field::Stiffness, Key("joint_editor.glyph.stiffness"), l, "");
    let damp = || Num(Field::Damping, Key("joint_editor.glyph.damping"), "joint_editor.label.damping", "");
    let mut out = Vec::new();
    match k {
        JointKind::Wheel => {
            out.push(Group("joint_editor.group.suspension", vec![
                stiff("joint_editor.label.stiffness"), damp(),
                Num(Field::LimitMin, Icon(IconId::ArrowDown), "joint_editor.label.min_travel", lim),
                Num(Field::LimitMax, Icon(IconId::ArrowUp), "joint_editor.label.max_travel", lim),
            ]));
            let mut wheels = vec![Item::Scope("joint_editor.label.drive", drive)];
            if info.motor_enabled {
                wheels.extend([speed(), force()]);
            } else {
                wheels.push(Item::Note("joint_editor.no_motor"));
            }
            if wheelbase {
                wheels.push(Num(Field::Wheelbase, Key("joint_editor.glyph.wheelbase"), "joint_editor.wheelbase", metre));
            }
            out.push(Group("joint_editor.group.wheels", wheels));
        }
        JointKind::Pin | JointKind::Slider => {
            let (lo, hi) = if metres {
                ("joint_editor.label.min_distance", "joint_editor.label.max_distance")
            } else {
                ("joint_editor.label.min_angle", "joint_editor.label.max_angle")
            };
            let mut limits = vec![Check(Flag::Limits, "component.field.joint.4", info.limits_enabled)];
            if info.limits_enabled {
                limits.extend([
                    Num(Field::LimitMin, Icon(IconId::ArrowDown), lo, lim),
                    Num(Field::LimitMax, Icon(IconId::ArrowUp), hi, lim),
                ]);
            }
            out.push(Group("joint_editor.group.limits", limits));
            let mut motor = vec![Check(Flag::Motor, "component.field.joint.7", info.motor_enabled)];
            if info.motor_enabled {
                motor.extend([speed(), force()]);
            }
            out.push(Group("joint_editor.group.motor", motor));
        }
        JointKind::Spring => out.push(Group("joint_editor.group.spring", vec![
            stiff("joint_editor.label.stiffness"), damp(),
            Num(Field::RestLength, Key("joint_editor.glyph.length"), "joint_editor.label.rest_length", metre),
        ])),
        JointKind::Rope | JointKind::Rod | JointKind::Pulley => out.push(Group("joint_editor.group.length", vec![
            Num(Field::MaxLength, Key("joint_editor.glyph.length"), "joint_editor.label.max_length", metre),
        ])),
        JointKind::Weld | JointKind::Custom => {}
    }
    if k.can_break() {
        let mut breaks = vec![Check(Flag::Breakable, "component.field.joint.19", info.break_enabled)];
        if info.break_enabled {
            breaks.extend([
                Num(Field::BreakForce, Key("joint_editor.glyph.force"), "joint_editor.label.break_force", "joint_editor.unit.n"),
                Num(Field::BreakTorque, Key("joint_editor.glyph.torque"), "joint_editor.label.break_torque", "joint_editor.unit.nm"),
            ]);
        }
        out.push(Group("joint_editor.group.breaks", breaks));
    }
    out
}

/// Os cartões da coluna, de cima para baixo.
fn cards(ui: &Ui<'_>) -> Vec<Group> {
    let mut head = vec![Item::Menu(Menu::Kind), Item::Menu(Menu::Template)];
    let Some(a) = ui.doc.assembly().cloned() else {
        let t = ui.doc.template().name;
        head.push(Item::Line(tr_with(
            "joint_editor.draft_title",
            &[("template", &t)],
        )));
        head.push(Item::Note("joint_editor.empty_panel"));
        return vec![Group("joint_editor.group.assembly", head)];
    };
    let (n, of, joints) = (a.filled_roles(), a.template.roles.len(), a.joints.len());
    let parts = [
        ("name", &a.name as &dyn std::fmt::Display),
        ("n", &n),
        ("of", &of),
        ("joints", &joints),
    ];
    head.push(Item::Line(tr_with("joint_editor.parts_of", &parts)));
    let info = ui.doc.targets().first().and_then(|j| j.info.clone());
    head.push(match info {
        Some(_) => Item::Inspect,
        None => Item::Note("joint_editor.no_joint_yet"),
    });
    let opts = scope_options(a.template);
    if opts.len() > 1 && !a.joints.is_empty() {
        let cur = ui.doc.scope_index(&a, &opts);
        let segs = opts
            .iter()
            .enumerate()
            .map(|(k, o)| (o.label.clone(), Hit::Scope(k), k == cur))
            .collect();
        head.push(Item::Scope("joint_editor.group.applies_to", segs));
    }
    let mut out = vec![Group("joint_editor.group.assembly", head)];
    if let Some(info) = info {
        let kind = ph2d_joint_assembly::joint_info::kind_of(info.kind_tag);
        out.extend(groups_of(
            kind,
            &info,
            ui.doc.wheelbase().is_some(),
            drive_segs(ui),
        ));
    }
    out
}

/// Pinta a coluna em `r`, a partir do rolamento da aba, e devolve as caixas das listas suspensas (por
/// [`Menu`]): a lista abre-se por baixo da dela.
pub fn paint(ui: &mut Ui<'_>, r: Rect) -> [Rect; 2] {
    let mut at = [Rect::default(); 2];
    let n = ui.doc.hits.len();
    let groups = cards(ui);
    let total = |ui: &mut Ui<'_>, w: f32| -> f32 {
        let hs: Vec<f32> = groups.iter().map(|g| card_h(ui, g, w)).collect();
        hs.iter().sum::<f32>() + m::CARD_GAP * (hs.len().max(1) - 1) as f32
    };
    // A barra do nosso painel só quando não cabe; os cartões encolhem para lhe dar a faixa.
    let mut content = total(ui, r.w);
    let track = scrollbar_track_rect(r);
    let cw = if content > r.h {
        let cw = track.x - m::LIST_BAR_GAP - r.x;
        content = total(ui, cw);
        cw
    } else {
        r.w
    };
    ui.doc.scroll_max = (content - r.h).max(0.0);
    ui.doc.scroll = ui.doc.scroll.clamp(0.0, ui.doc.scroll_max);
    ui.doc.scroll_per_px = content / r.h.max(1.0);
    ui.scene.push_clip(&rect_to_vello(r));
    let mut y = r.y - ui.doc.scroll;
    for g in &groups {
        y += card(ui, Rect::new(r.x, y, cw, 0.0), g, &mut at) + m::CARD_GAP;
    }
    ui.scene.pop_layer();
    let state = match (ui.doc.grab, ui.doc.hover.as_ref()) {
        (Some(crate::doc::Grab::Bar { .. }), _) => ScrollbarState::Dragging,
        (_, Some(Hit::Bar)) => ScrollbarState::Hovered,
        _ => ScrollbarState::Normal,
    };
    paint_scrollbar(
        r,
        ui.doc.scroll,
        content,
        r.h,
        (state, 0.0),
        ui.scene,
        ui.theme,
    );
    // O que a coluna corta não se clica: um alvo vale só se couber inteiro na parte à vista.
    let tail = ui.doc.hits.split_off(n);
    let seen = |h: &Rect| h.y >= r.y && h.y + h.h <= r.y + r.h;
    ui.doc
        .hits
        .extend(tail.into_iter().filter(|(h, _, _)| seen(h)));
    if ui.doc.scroll_max > 0.0 {
        ui.hit(track, Hit::Bar, "");
    }
    at
}

/// As linhas de uma frase na largura `wide`.
fn wrap(ui: &mut Ui<'_>, s: &str, wide: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in s.split(' ') {
        match lines.last_mut() {
            Some(l) if ui.ts.prefix_width(&format!("{l} {word}"), m::VALUE_PX) <= wide => {
                l.push(' ');
                l.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

/// A altura de uma linha de frase.
const NOTE_LINE: f32 = m::VALUE_PX + m::LABEL_BOTTOM * 2.0;
/// O rótulo acima de uma caixa: a letra e o vão até à caixa.
const LABEL_H: f32 = m::LABEL_PX + m::LABEL_BOTTOM;

/// ⭐ Cabem dois campos legíveis lado a lado em `iw` (dono 13/11)? Cada metade tem de levar
/// [`FIELD_MIN_W`]; senão cada par passa a uma coluna.
fn two_up(iw: f32) -> bool {
    (iw - m::FIELD_GAP) * 0.5 >= FIELD_MIN_W
}

/// As filas de um cartão: números, listas e o resumo com o botão aos pares quando `two`; o resto sozinho.
fn rows(items: &[Item], two: bool) -> Vec<Vec<&Item>> {
    let mut out: Vec<Vec<&Item>> = Vec::new();
    for it in items {
        let pairs = |a: &Item| {
            two && matches!(
                (a, it),
                (Item::Num(..), Item::Num(..))
                    | (Item::Menu(_), Item::Menu(_))
                    | (Item::Line(_), Item::Inspect)
            )
        };
        match out.last_mut() {
            Some(last) if last.len() == 1 && pairs(last[0]) => last.push(it),
            _ => out.push(vec![it]),
        }
    }
    out
}

/// A altura de uma fila; numa coluna o resumo parte-se em linhas como uma frase.
fn row_h(ui: &mut Ui<'_>, row: &[&Item], wide: f32) -> f32 {
    match row[0] {
        Item::Menu(_) | Item::Num(..) => LABEL_H + m::FIELD_H,
        Item::Scope(_, segs) => LABEL_H + seg_h(seg_rows(ui.ts, segs, wide).1.len()),
        #[allow(clippy::cast_precision_loss)]
        Item::Note(k) => wrap(ui, tr(k), wide).len() as f32 * NOTE_LINE,
        #[allow(clippy::cast_precision_loss)]
        Item::Line(s) if row.len() == 1 && !two_up(wide) => {
            (wrap(ui, s, wide).len() as f32 * NOTE_LINE).max(m::FIELD_H)
        }
        _ => m::FIELD_H,
    }
}

/// A cabeça de um cartão: a folga de cima, o título e o vão até ao corpo.
const TOP: f32 = m::CARD_PAD_TOP + m::CARD_HEAD_H + m::CARD_HEAD_BOTTOM;

/// A altura de um cartão na largura `w` (recolhido = só a cabeça).
fn card_h(ui: &mut Ui<'_>, g: &Group, w: f32) -> f32 {
    if ui.doc.is_folded(g.0) {
        return m::CARD_PAD_TOP * 2.0 + m::CARD_HEAD_H;
    }
    let iw = (w - m::CARD_PAD_X * 2.0).max(0.0);
    let mut body = 0.0;
    for (k, row) in rows(&g.1, two_up(iw)).iter().enumerate() {
        body += row_h(ui, row, iw) + if k > 0 { m::FIELD_ROW_GAP } else { 0.0 };
    }
    TOP + body + m::CARD_PAD_BOTTOM
}

/// As larguras de uma fila em `iw`: a toda a largura numa coluna; senão metade e metade; o resumo cede ao botão compacto o que ele pede.
fn widths(ui: &mut Ui<'_>, row: &[&Item], iw: f32) -> Vec<f32> {
    let half = (iw - m::FIELD_GAP) * 0.5;
    if !two_up(iw) {
        return vec![iw];
    }
    match row {
        [Item::Line(_), Item::Inspect] => {
            let bw = ui
                .ts
                .prefix_width(tr("joint_editor.open_short"), m::VALUE_PX)
                + m::FIELD_PAD_R * 2.0;
            vec![(iw - bw - m::FIELD_GAP).max(0.0), bw]
        }
        [_, _] => vec![half, half],
        [Item::Num(..)] => vec![half],
        _ => vec![iw],
    }
}

/// Pinta um cartão no topo de `r` (a largura da coluna) e devolve a altura dele.
fn card(ui: &mut Ui<'_>, r: Rect, g: &Group, at: &mut [Rect; 2]) -> f32 {
    let iw = (r.w - m::CARD_PAD_X * 2.0).max(0.0);
    let h = card_h(ui, g, r.w);
    let bg = ui.c(ColorToken::KanbanCard);
    fill_rounded_rect(ui.scene, Rect::new(r.x, r.y, r.w, h), m::CARD_RADIUS, bg);
    let folded = ui.doc.is_folded(g.0);
    let head = Rect::new(r.x, r.y + m::CARD_PAD_TOP, r.w, m::CARD_HEAD_H);
    let mut tx = r.x + m::CARD_PAD_X;
    // O título do Inspector: aberto, só o nome; recolhido, o › à frente.
    if folded {
        let c = m::CARD_CHEVRON;
        let chev = Rect::new(tx, head.y + (head.h - c) * 0.5, c, c);
        let ink = ui.c(ColorToken::Text2);
        paint_icon(
            ui.scene,
            IconId::ChevronRight,
            chev,
            ink,
            StrokeToken::Default.px(),
        );
        tx += c + m::CARD_CHEVRON_GAP * 2.0;
    }
    let ink = ui.c(ColorToken::Text1);
    let ty = head.y + folga_do_texto(m::CARD_HEAD_H, m::CARD_TITLE_PX);
    let title = tr(g.0);
    let tw = (r.x + r.w - m::CARD_PAD_X - tx).max(0.0);
    paint_text_title_elided_tracked(
        ui.ts,
        ui.scene,
        title,
        [tx, ty],
        m::CARD_TITLE_PX,
        tw,
        ink,
        0.0,
    );
    ui.hit(head, Hit::Fold(g.0), title);
    if folded {
        return h;
    }
    let left = r.x + m::CARD_PAD_X;
    let mut y = r.y + TOP;
    for row in rows(&g.1, two_up(iw)) {
        let rh = row_h(ui, &row, iw);
        let ws = widths(ui, &row, iw);
        let mut x = left;
        for (it, w) in row.into_iter().zip(ws) {
            item(ui, Rect::new(x, y, w, rh), it, at);
            x += w + m::FIELD_GAP;
        }
        y += rh + m::FIELD_ROW_GAP;
    }
    h
}

fn drive_segs(ui: &Ui<'_>) -> Vec<(String, Hit, bool)> {
    let cur = ui.doc.drive();
    ui.doc
        .drive_options()
        .into_iter()
        .map(|(d, l)| (l, Hit::Drive(d), Some(d) == cur))
        .collect()
}

/// O rótulo com a unidade, como o do Inspector: «Min travel (m)».
fn full_name(name: &str, unit: &str) -> String {
    if unit.is_empty() {
        return tr(name).to_owned();
    }
    tr_with(
        "joint_editor.with_unit",
        &[("label", &tr(name)), ("unit", &tr(unit))],
    )
}

/// O rótulo pequeno acima da caixa (elidido numa linha) e a caixa por baixo.
fn labelled(ui: &mut Ui<'_>, b: Rect, label: &str) -> Rect {
    paint_label(ui.scene, ui.ts, ui.ink, b.x, b.y, b.w, label);
    Rect::new(b.x, b.y + LABEL_H, b.w, m::FIELD_H)
}

fn item(ui: &mut Ui<'_>, b: Rect, it: &Item, at: &mut [Rect; 2]) {
    match it {
        Item::Num(f, glyph, name, unit) => {
            let gl = match *glyph {
                Glyph::Text(k) => Glyph::Text(tr(k)),
                icon @ Glyph::Icon(_) => icon,
            };
            let label = full_name(name, unit);
            let bx = labelled(ui, b, &label);
            ui.number(bx, gl, &label, *f);
        }
        Item::Check(f, name, on) => {
            ui.check(b, tr("joint_editor.on"), tr(name), Hit::Flag(*f), *on);
        }
        Item::Scope(label, segs) => {
            let bx = labelled(ui, b, tr(label));
            ui.segmented(Rect::new(bx.x, bx.y, bx.w, b.h - LABEL_H), segs);
        }
        Item::Note(k) => note(ui, tr(k), b),
        Item::Line(s) if wrap(ui, s, b.w).len() > 1 && b.h > m::FIELD_H => note(ui, s, b),
        Item::Line(s) => ui.put(s, b, m::VALUE_PX, ColorToken::Text2),
        Item::Inspect => {
            let a11y = tr("joint_editor.open_inspector");
            ui.button(b, tr("joint_editor.open_short"), Hit::Inspect, true);
            if let Some(last) = ui.doc.hits.last_mut() {
                last.2 = a11y.to_owned();
            }
        }
        Item::Menu(menu) => {
            let t = ui.doc.template();
            let (label, value, tip) = match menu {
                Menu::Kind => (
                    "joint_editor.joint",
                    tr(kind_key(t.kind)),
                    "joint_editor.joint_tip",
                ),
                Menu::Template => ("joint_editor.template", t.name, "joint_editor.template_tip"),
            };
            let (label, h) = (tr(label), Hit::OpenMenu(*menu));
            let bx = labelled(ui, b, label);
            ui.dropdown(bx, value, h, &format!("{label}: {value}. {}", tr(tip)));
            at[*menu as usize] = bx;
        }
    }
}

/// Uma frase partida em linhas na largura de `b`.
fn note(ui: &mut Ui<'_>, s: &str, b: Rect) {
    for (n, l) in wrap(ui, s, b.w).iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let lr = Rect::new(b.x, b.y + NOTE_LINE * n as f32, b.w, NOTE_LINE);
        ui.put(l, lr, m::VALUE_PX, ColorToken::Text2);
    }
}
