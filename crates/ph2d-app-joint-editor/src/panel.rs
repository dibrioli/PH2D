//! **O painel de baixo**: a montagem aberta («Car 1 · 3 of 3 parts»), a que juntas os números se aplicam
//! («Both wheels · Rear wheel · Front wheel», o mesmo «Same as» do cartão do Inspector) e os números de
//! cada tipo, com a unidade no rótulo. O resto dos campos fica no cartão Joint: «Open in Inspector».

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_joint_templates::JointKind;
use ph2d_tokens::ColorToken;

use crate::doc::{Field, Flag, Hit, scope_options};
use crate::widgets::{self as w, Ui};

/// O rótulo com a unidade: «Travel down (m)».
fn unit(label: &str, unit_key: &str) -> String {
    tr_with("joint_editor.with_unit", &[("label", &label), ("unit", &tr(unit_key))])
}

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

/// Uma linha do painel: dois números, uma caixa, ou a escolha do Drive.
enum Row {
    Two((Field, String), (Field, String)),
    One(Field, String),
    Flag(Flag, String, bool),
    Drive,
    Note(String),
}

/// As linhas do tipo `k`, lidas da 1.ª junta do alcance.
fn rows_of(k: JointKind, info: &ph2d_editor_core::screens::hero::InspectorJointInfo, wheelbase: bool) -> Vec<Row> {
    let f = |key: &str| tr(key).to_owned();
    let lim = limit_unit(k);
    let spd = speed_unit(k);
    let mut out = Vec::new();
    match k {
        JointKind::Wheel => {
            out.push(Row::Two(
                (Field::Stiffness, f("panel.inspector.joint.suspension_stiffness")),
                (Field::Damping, f("panel.inspector.joint.suspension_damping")),
            ));
            out.push(Row::Two(
                (Field::LimitMin, unit(tr("panel.inspector.joint.travel_down"), lim)),
                (Field::LimitMax, unit(tr("panel.inspector.joint.travel_up"), lim)),
            ));
            out.push(Row::Drive);
            if info.motor_enabled {
                out.push(Row::Two(
                    (Field::MotorSpeed, unit(tr("panel.inspector.joint.spin_speed"), spd)),
                    (Field::MotorForce, f("component.field.joint.9")),
                ));
            } else {
                out.push(Row::Note(f("joint_editor.no_motor")));
            }
            if wheelbase {
                out.push(Row::One(Field::Wheelbase, unit(tr("joint_editor.wheelbase"), "joint_editor.unit.m")));
            }
        }
        JointKind::Pin | JointKind::Slider => {
            out.push(Row::Flag(Flag::Limits, f("component.field.joint.4"), info.limits_enabled));
            if info.limits_enabled {
                out.push(Row::Two(
                    (Field::LimitMin, unit(tr("component.field.joint.5"), lim)),
                    (Field::LimitMax, unit(tr("component.field.joint.6"), lim)),
                ));
            }
            out.push(Row::Flag(Flag::Motor, f("component.field.joint.7"), info.motor_enabled));
            if info.motor_enabled {
                out.push(Row::Two(
                    (Field::MotorSpeed, unit(tr("component.field.joint.8"), spd)),
                    (Field::MotorForce, f("component.field.joint.9")),
                ));
            }
        }
        JointKind::Spring => {
            out.push(Row::Two(
                (Field::Stiffness, f("component.field.joint.11")),
                (Field::Damping, f("component.field.joint.12")),
            ));
            out.push(Row::One(Field::RestLength, unit(tr("component.field.joint.10"), "joint_editor.unit.m")));
        }
        JointKind::Rope | JointKind::Rod | JointKind::Pulley => {
            out.push(Row::One(Field::MaxLength, unit(tr("component.field.joint.13"), "joint_editor.unit.m")));
        }
        JointKind::Weld | JointKind::Custom => {}
    }
    if k.can_break() {
        out.push(Row::Flag(Flag::Breakable, f("component.field.joint.19"), info.break_enabled));
        if info.break_enabled {
            out.push(Row::Two(
                (Field::BreakForce, unit(tr("component.field.joint.20"), "joint_editor.unit.n")),
                (Field::BreakTorque, unit(tr("component.field.joint.21"), "joint_editor.unit.nm")),
            ));
        }
    }
    out
}

/// A altura que o painel pede: a faixa, o alcance e as linhas do tipo da junta.
#[must_use]
pub fn wanted(doc: &crate::doc::JointEditorDoc) -> f32 {
    let Some(a) = doc.assembly() else {
        return w::PANEL_MIN_H;
    };
    let line = m::FIELD_H + m::FIELD_ROW_GAP;
    let field = m::LABEL_PX + m::LABEL_BOTTOM + m::FIELD_H + m::FIELD_ROW_GAP;
    let mut h = m::CARD_PAD_TOP + m::CARD_PAD_BOTTOM + line;
    if scope_options(a.template).len() > 1 && !a.joints.is_empty() {
        h += line;
    }
    let Some(info) = doc.targets().first().and_then(|j| j.info.clone()) else {
        return (h + line).max(w::PANEL_MIN_H);
    };
    let kind = ph2d_app_physics::joint::kind_of(info.kind_tag);
    for row in rows_of(kind, &info, doc.wheelbase().is_some()) {
        h += match row {
            Row::Two(..) | Row::One(..) => field,
            _ => line,
        };
    }
    h.max(w::PANEL_MIN_H)
}

pub fn paint(ui: &mut Ui<'_>, r: Rect) {
    let i = Rect::new(
        r.x + m::CARD_PAD_X,
        r.y + m::CARD_PAD_TOP,
        (r.w - m::CARD_PAD_X * 2.0).max(0.0),
        (r.h - m::CARD_PAD_TOP - m::CARD_PAD_BOTTOM).max(0.0),
    );
    let Some(a) = ui.doc.assembly().cloned() else {
        let c = ui.c(ColorToken::Text1);
        let t = ui.doc.template();
        let head = tr_with("joint_editor.draft_title", &[("template", &t.name)]);
        w::text(ui.ts, ui.scene, &head, Rect::new(i.x, i.y, i.w, m::FIELD_H), m::CARD_TITLE_PX, c);
        let c2 = ui.c(ColorToken::Text2);
        w::text(ui.ts, ui.scene, tr("joint_editor.empty_panel"), Rect::new(i.x, i.y + m::FIELD_H, i.w, m::FIELD_H), m::VALUE_PX, c2);
        return;
    };
    // A faixa: o nome, quantas peças, e o botão do cartão inteiro.
    let head = Rect::new(i.x, i.y, i.w, m::FIELD_H);
    let bw = m::FIELD_VALUE_MIN_W * 2.4;
    let c = ui.c(ColorToken::Text1);
    w::text(ui.ts, ui.scene, &a.name, Rect::new(head.x, head.y, head.w * 0.4, head.h), m::CARD_TITLE_PX, c);
    let parts = tr_with(
        "joint_editor.parts_of",
        &[("n", &a.filled_roles()), ("of", &a.template.roles.len())],
    );
    let c2 = ui.c(ColorToken::Text2);
    let nx = head.x + ui.ts.prefix_width(&a.name, m::CARD_TITLE_PX) + m::CARD_GAP;
    w::text(ui.ts, ui.scene, &parts, Rect::new(nx, head.y, (head.x + head.w - bw - nx).max(0.0), head.h), m::VALUE_PX, c2);
    let targets = ui.doc.targets().len();
    if targets > 0 {
        ui.button(Rect::new(head.x + head.w - bw, head.y, bw, head.h), tr("joint_editor.open_inspector"), Hit::Inspect, false);
    }
    let mut y = head.y + head.h + m::FIELD_ROW_GAP;
    let opts = scope_options(a.template);
    if opts.len() > 1 && !a.joints.is_empty() {
        let cur = ui.doc.scope_index(&a, &opts);
        let segs: Vec<(String, Hit, bool)> = opts
            .iter()
            .enumerate()
            .map(|(k, o)| (o.label.clone(), Hit::Scope(k), k == cur))
            .collect();
        ui.segmented(Rect::new(i.x, y, i.w, m::FIELD_H), &segs);
        y += m::FIELD_H + m::FIELD_ROW_GAP;
    }
    let Some(info) = ui.doc.targets().first().and_then(|j| j.info.clone()) else {
        let c2 = ui.c(ColorToken::Text2);
        w::text(ui.ts, ui.scene, tr("joint_editor.no_joint_yet"), Rect::new(i.x, y, i.w, m::FIELD_H), m::VALUE_PX, c2);
        return;
    };
    let kind = ph2d_app_physics::joint::kind_of(info.kind_tag);
    let rows = rows_of(kind, &info, ui.doc.wheelbase().is_some());
    let col = (i.w - m::FIELD_GAP) * 0.5;
    let field_h = m::LABEL_PX + m::LABEL_BOTTOM + m::FIELD_H;
    for row in rows {
        let h = match row {
            Row::Two(..) | Row::One(..) => field_h,
            _ => m::FIELD_H,
        };
        if y + h > i.y + i.h {
            break;
        }
        match row {
            Row::Two((fa, la), (fb, lb)) => {
                let (va, vb) = (ui.doc.value(fa), ui.doc.value(fb));
                ui.number(Rect::new(i.x, y, col, h), &la, fa, va);
                ui.number(Rect::new(i.x + col + m::FIELD_GAP, y, col, h), &lb, fb, vb);
            }
            Row::One(f, l) => {
                let v = ui.doc.value(f);
                ui.number(Rect::new(i.x, y, col, h), &l, f, v);
            }
            Row::Flag(f, l, on) => ui.check(Rect::new(i.x, y, i.w, h), &l, Hit::Flag(f), on),
            Row::Drive => {
                let label = tr("panel.inspector.joint.drive");
                let lw = m::FIELD_VALUE_MIN_W * 1.5;
                ui.label(label, Rect::new(i.x, y, lw, h));
                let cur = ui.doc.drive();
                let segs: Vec<(String, Hit, bool)> = ui
                    .doc
                    .drive_options()
                    .into_iter()
                    .map(|(d, l)| (l, Hit::Drive(d), Some(d) == cur))
                    .collect();
                ui.segmented(Rect::new(i.x + lw, y, i.w - lw, h), &segs);
            }
            Row::Note(s) => {
                let c2 = ui.c(ColorToken::Text2);
                w::text(ui.ts, ui.scene, &s, Rect::new(i.x, y, i.w, h), m::VALUE_PX, c2);
            }
        }
        y += h + m::FIELD_ROW_GAP;
    }
}
