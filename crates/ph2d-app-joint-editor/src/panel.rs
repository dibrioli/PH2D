//! **O painel de baixo** (dono 13/11): a faixa (montagem, peças, alcance «Both wheels · Rear · Front», «Open
//! in Inspector») e os números em cartões por assunto, lado a lado, que só quebram quando não cabem. Cada
//! número é o campo do cartão do app (a letra ou o ícone na caixa, valor, unidade); o nome é o rótulo a11y.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::stroke_rounded_rect;
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_joint_templates::JointKind;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

use crate::doc::{Field, Flag, Hit, scope_options};
use crate::widgets::{self as w, Ui};

/// Um controlo de um cartão.
enum Item {
    /// O número, a pega (a letra é uma chave i18n), o nome e a unidade (chaves; `""` = sem unidade).
    Num(Field, Glyph<'static>, &'static str, &'static str),
    /// A caixa «On» e o nome dela.
    Check(Flag, &'static str, bool),
    Drive,
    Note(&'static str),
}

/// Um cartão: o título e os controlos, lado a lado.
struct Group(&'static str, Vec<Item>);

fn limit_unit(k: JointKind) -> &'static str {
    if k.limits_in_metres() {
        "joint_editor.unit.m"
    } else {
        "joint_editor.unit.deg"
    }
}

fn speed_unit(k: JointKind) -> &'static str {
    if k.motor_in_metres() {
        "joint_editor.unit.m_per_s"
    } else {
        "joint_editor.unit.deg_per_s"
    }
}

/// Os cartões do tipo `k`, lidos da 1.ª junta do alcance. Uma tabela: um controlo por linha.
#[rustfmt::skip]
fn groups_of(k: JointKind, info: &ph2d_editor_core::screens::hero::InspectorJointInfo, wheelbase: bool) -> Vec<Group> {
    use Item::{Check, Num};
    use Glyph::{Icon, Text as Key};
    let (lim, spd, metre) = (limit_unit(k), speed_unit(k), "joint_editor.unit.m");
    let speed = |name| Num(Field::MotorSpeed, Key("joint_editor.glyph.speed"), name, spd);
    let force = || Num(Field::MotorForce, Key("joint_editor.glyph.force"), "component.field.joint.9", "");
    let mut out = Vec::new();
    match k {
        JointKind::Wheel => {
            out.push(Group("joint_editor.group.suspension", vec![
                Num(Field::Stiffness, Key("joint_editor.glyph.stiffness"), "panel.inspector.joint.suspension_stiffness", ""),
                Num(Field::Damping, Key("joint_editor.glyph.damping"), "panel.inspector.joint.suspension_damping", ""),
            ]));
            out.push(Group("panel.inspector.joint.travel", vec![
                Num(Field::LimitMin, Icon(IconId::ArrowDown), "panel.inspector.joint.travel_down", lim),
                Num(Field::LimitMax, Icon(IconId::ArrowUp), "panel.inspector.joint.travel_up", lim),
            ]));
            let mut drive = vec![Item::Drive];
            if info.motor_enabled {
                drive.extend([speed("panel.inspector.joint.spin_speed"), force()]);
            } else {
                drive.push(Item::Note("joint_editor.no_motor"));
            }
            out.push(Group("panel.inspector.joint.drive", drive));
            if wheelbase {
                out.push(Group("joint_editor.group.body", vec![
                    Num(Field::Wheelbase, Key("joint_editor.glyph.wheelbase"), "joint_editor.wheelbase", metre),
                ]));
            }
        }
        JointKind::Pin | JointKind::Slider => {
            let mut limits = vec![Check(Flag::Limits, "component.field.joint.4", info.limits_enabled)];
            if info.limits_enabled {
                limits.extend([
                    Num(Field::LimitMin, Key("joint_editor.glyph.min"), "component.field.joint.5", lim),
                    Num(Field::LimitMax, Key("joint_editor.glyph.max"), "component.field.joint.6", lim),
                ]);
            }
            out.push(Group("component.field.joint.4", limits));
            let mut motor = vec![Check(Flag::Motor, "component.field.joint.7", info.motor_enabled)];
            if info.motor_enabled {
                motor.extend([speed("component.field.joint.8"), force()]);
            }
            out.push(Group("component.field.joint.7", motor));
        }
        JointKind::Spring => out.push(Group("joint_editor.group.spring", vec![
            Num(Field::Stiffness, Key("joint_editor.glyph.stiffness"), "component.field.joint.11", ""),
            Num(Field::Damping, Key("joint_editor.glyph.damping"), "component.field.joint.12", ""),
            Num(Field::RestLength, Key("joint_editor.glyph.length"), "component.field.joint.10", metre),
        ])),
        JointKind::Rope | JointKind::Rod | JointKind::Pulley => out.push(Group("joint_editor.group.length", vec![
            Num(Field::MaxLength, Key("joint_editor.glyph.length"), "component.field.joint.13", metre),
        ])),
        JointKind::Weld | JointKind::Custom => {}
    }
    if k.can_break() {
        let mut breaks = vec![Check(Flag::Breakable, "component.field.joint.19", info.break_enabled)];
        if info.break_enabled {
            breaks.extend([
                Num(Field::BreakForce, Key("joint_editor.glyph.force"), "component.field.joint.20", "joint_editor.unit.n"),
                Num(Field::BreakTorque, Key("joint_editor.glyph.torque"), "component.field.joint.21", "joint_editor.unit.nm"),
            ]);
        }
        out.push(Group("joint_editor.group.breaks", breaks));
    }
    out
}

/// A folga de dentro de um cartão do painel.
const PAD: f32 = m::FIELD_GAP;
/// A altura de um cartão: a folga, o título, a fila dos controlos e a folga.
const GROUP_H: f32 = PAD * 2.0 + m::LABEL_PX + m::LABEL_BOTTOM + m::FIELD_H;

/// A altura que o painel pede na largura `wide`: a pintura num rascunho (os textos decidem as quebras).
pub fn wanted(ui: &mut Ui<'_>, wide: f32) -> f32 {
    let n = ui.doc.hits.len();
    let mut scratch = VectorScene::new();
    let mut dry = Ui {
        doc: &mut *ui.doc,
        scene: &mut scratch,
        ts: &mut *ui.ts,
        ink: ui.ink,
    };
    let h = paint(&mut dry, Rect::new(0.0, 0.0, wide, f32::INFINITY));
    ui.doc.hits.truncate(n);
    h.max(w::PANEL_MIN_H)
}

/// Pinta o painel em `r` e devolve a altura que ocupou.
pub fn paint(ui: &mut Ui<'_>, r: Rect) -> f32 {
    let i = Rect::new(
        r.x + m::CARD_PAD_X,
        r.y + m::CARD_PAD_TOP,
        (r.w - m::CARD_PAD_X * 2.0).max(0.0),
        (r.h - m::CARD_PAD_TOP - m::CARD_PAD_BOTTOM).max(0.0),
    );
    let line = |y: f32| Rect::new(i.x, y, i.w, m::FIELD_H);
    let Some(a) = ui.doc.assembly().cloned() else {
        let head = tr_with(
            "joint_editor.draft_title",
            &[("template", &ui.doc.template().name)],
        );
        ui.put(&head, line(i.y), m::CARD_TITLE_PX, ColorToken::Text1);
        let s = tr("joint_editor.empty_panel");
        ui.put(s, line(i.y + m::FIELD_H), m::VALUE_PX, ColorToken::Text2);
        return w::PANEL_MIN_H;
    };
    // A faixa: o nome, quantas peças, o alcance e o botão do cartão inteiro, à direita.
    let tw = ui.ts.prefix_width(&a.name, m::CARD_TITLE_PX);
    let nr = Rect::new(i.x, i.y, tw + 1.0, m::FIELD_H);
    ui.put(&a.name, nr, m::CARD_TITLE_PX, ColorToken::Text1);
    let mut right = i.x + i.w;
    if !ui.doc.targets().is_empty() {
        let label = tr("joint_editor.open_inspector");
        let bw = ui.button_w(label);
        right -= bw;
        let b = Rect::new(right, i.y, bw, m::FIELD_H);
        ui.button(b, label, Hit::Inspect, false);
        right -= m::CARD_GAP;
    }
    let parts = tr_with(
        "joint_editor.parts_of",
        &[
            ("n", &a.filled_roles()),
            ("of", &a.template.roles.len()),
            ("joints", &a.joints.len()),
        ],
    );
    let px = i.x + tw + m::CARD_GAP;
    let pw = ui
        .ts
        .prefix_width(&parts, m::VALUE_PX)
        .min((right - px).max(0.0));
    let pr = Rect::new(px, i.y, pw + 1.0, m::FIELD_H);
    ui.put(&parts, pr, m::VALUE_PX, ColorToken::Text2);
    let mut y = i.y + m::FIELD_H + m::FIELD_ROW_GAP;
    let opts = scope_options(a.template);
    if opts.len() > 1 && !a.joints.is_empty() {
        let cur = ui.doc.scope_index(&a, &opts);
        let segs: Vec<(String, Hit, bool)> = opts
            .iter()
            .enumerate()
            .map(|(k, o)| (o.label.clone(), Hit::Scope(k), k == cur))
            .collect();
        let sw = ui.segmented_w(&segs);
        let sx = px + pw + m::CARD_GAP * 2.0;
        if sx + sw <= right {
            ui.segmented(Rect::new(sx, i.y, sw, m::FIELD_H), &segs);
        } else {
            ui.segmented(Rect::new(i.x, y, sw, m::FIELD_H), &segs);
            y += m::FIELD_H + m::FIELD_ROW_GAP;
        }
    }
    let Some(info) = ui.doc.targets().first().and_then(|j| j.info.clone()) else {
        let s = tr("joint_editor.no_joint_yet");
        ui.put(s, line(y), m::VALUE_PX, ColorToken::Text2);
        return y + m::FIELD_H + m::CARD_PAD_BOTTOM - r.y;
    };
    let kind = ph2d_app_physics::joint::kind_of(info.kind_tag);
    let mut x = i.x;
    let mut bottom = y;
    for g in groups_of(kind, &info, ui.doc.wheelbase().is_some()) {
        let gw = group_w(ui, &g);
        if x > i.x && x + gw > i.x + i.w {
            x = i.x;
            y += GROUP_H + m::CARD_GAP;
        }
        if y + GROUP_H <= i.y + i.h {
            paint_group(ui, Rect::new(x, y, gw, GROUP_H), &g);
        }
        bottom = y + GROUP_H;
        x += gw + m::CARD_GAP;
    }
    bottom + m::CARD_PAD_BOTTOM - r.y
}

/// A largura de um controlo dentro de um cartão.
fn item_w(ui: &mut Ui<'_>, it: &Item) -> f32 {
    match it {
        Item::Num(..) => w::NUM_W,
        Item::Check(..) => ui.check_w(tr("joint_editor.on")),
        Item::Drive => {
            let segs = drive_segs(ui);
            ui.segmented_w(&segs)
        }
        Item::Note(k) => ui.ts.prefix_width(tr(k), m::VALUE_PX),
    }
}

fn group_w(ui: &mut Ui<'_>, g: &Group) -> f32 {
    let mut sum = 0.0;
    for (n, it) in g.1.iter().enumerate() {
        sum += item_w(ui, it) + if n > 0 { m::FIELD_GAP } else { 0.0 };
    }
    let title = ui.ts.prefix_width(tr(g.0), m::LABEL_PX);
    sum.max(title) + PAD * 2.0
}

fn drive_segs(ui: &Ui<'_>) -> Vec<(String, Hit, bool)> {
    let cur = ui.doc.drive();
    ui.doc
        .drive_options()
        .into_iter()
        .map(|(d, l)| (l, Hit::Drive(d), Some(d) == cur))
        .collect()
}

/// O nome acessível com a unidade: «Travel down (m)».
fn full_name(name: &str, unit: &str) -> String {
    if unit.is_empty() {
        return tr(name).to_owned();
    }
    tr_with(
        "joint_editor.with_unit",
        &[("label", &tr(name)), ("unit", &tr(unit))],
    )
}

fn paint_group(ui: &mut Ui<'_>, r: Rect, g: &Group) {
    let border = ui.c(ColorToken::Border);
    stroke_rounded_rect(ui.scene, r, m::CARD_RADIUS, m::FIELD_BORDER, border);
    let t = Rect::new(r.x + PAD, r.y + PAD, r.w - PAD * 2.0, m::LABEL_PX);
    ui.label(tr(g.0), t);
    let mut x = r.x + PAD;
    let y = r.y + PAD + m::LABEL_PX + m::LABEL_BOTTOM;
    for it in &g.1 {
        let iw = item_w(ui, it);
        let b = Rect::new(x, y, iw, m::FIELD_H);
        match *it {
            Item::Num(f, glyph, name, unit) => {
                let gl = match glyph {
                    Glyph::Text(k) => Glyph::Text(tr(k)),
                    icon @ Glyph::Icon(_) => icon,
                };
                let u = if unit.is_empty() { "" } else { tr(unit) };
                ui.number(b, gl, &full_name(name, unit), u, f);
            }
            Item::Check(f, name, on) => {
                ui.check(b, tr("joint_editor.on"), tr(name), Hit::Flag(f), on)
            }
            Item::Drive => {
                let segs = drive_segs(ui);
                ui.segmented(b, &segs);
            }
            Item::Note(k) => ui.put(tr(k), b, m::VALUE_PX, ColorToken::Text2),
        }
        x += iw + m::FIELD_GAP;
    }
}
