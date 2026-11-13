//! **O painel da direita** (dono 13/11): uma coluna de cartões como os nossos painéis, um assunto cada —
//! Assembly (Joint, Template, as peças, «Open in Inspector»), Applies to (o alcance) e os números do tipo.
//! Cada número é o campo do cartão do app (a letra ou o ícone na caixa, valor, unidade), dois por fila; o
//! nome é o rótulo a11y. A coluna rola com a roda quando não cabe.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, paint_text_title_elided_tracked, rect_to_vello,
};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::{ScrollbarState, paint_scrollbar, scrollbar_track_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_joint_templates::JointKind;
use ph2d_tokens::ColorToken;

use crate::doc::{Field, Flag, Hit, Menu, kind_key, scope_options};
use crate::widgets::Ui;

/// Um controlo de um cartão.
enum Item {
    /// O número, a pega (a letra é uma chave i18n), o nome e a unidade (chaves; `""` = sem unidade).
    Num(Field, Glyph<'static>, &'static str, &'static str),
    /// A caixa «On» e o nome dela.
    Check(Flag, &'static str, bool),
    /// Uma frase (parte-se em linhas na largura do cartão).
    Note(&'static str),
    /// Uma lista suspensa com o rótulo por cima.
    Menu(Menu),
    /// Uma linha de leitura apagada.
    Line(String),
    Inspect,
    /// Uma escolha segmentada a toda a largura (o alcance, o Drive).
    Scope(Vec<(String, Hit, bool)>),
}

/// Um cartão: o título e os controlos (os números dois a dois, o resto a toda a largura).
struct Group(&'static str, Vec<Item>);

/// Os cartões do tipo `k`, lidos da 1.ª junta do alcance. Uma tabela: um controlo por linha.
#[rustfmt::skip]
fn groups_of(k: JointKind, info: &ph2d_editor_core::screens::hero::InspectorJointInfo, wheelbase: bool, drive: Vec<(String, Hit, bool)>) -> Vec<Group> {
    use Item::{Check, Num};
    use Glyph::{Icon, Text as Key};
    let lim = if k.limits_in_metres() { "joint_editor.unit.m" } else { "joint_editor.unit.deg" };
    let spd = if k.motor_in_metres() { "joint_editor.unit.m_per_s" } else { "joint_editor.unit.deg_per_s" };
    let metre = "joint_editor.unit.m";
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
            let mut drive = vec![Item::Scope(drive)];
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
    let mut out = vec![Group("joint_editor.group.assembly", head)];
    let opts = scope_options(a.template);
    if opts.len() > 1 && !a.joints.is_empty() {
        let cur = ui.doc.scope_index(&a, &opts);
        let segs = opts
            .iter()
            .enumerate()
            .map(|(k, o)| (o.label.clone(), Hit::Scope(k), k == cur))
            .collect();
        out.push(Group(
            "joint_editor.group.applies_to",
            vec![Item::Scope(segs)],
        ));
    }
    if let Some(info) = info {
        let kind = ph2d_app_physics::joint::kind_of(info.kind_tag);
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
    ui.scene.push_clip(&rect_to_vello(r));
    let mut y = r.y - ui.doc.scroll;
    for g in &groups {
        y += card(ui, Rect::new(r.x, y, cw, 0.0), g, &mut at) + m::CARD_GAP;
    }
    ui.scene.pop_layer();
    let state = (ScrollbarState::Normal, 0.0);
    paint_scrollbar(r, ui.doc.scroll, content, r.h, state, ui.scene, ui.theme);
    // O que a coluna corta não se clica: um alvo vale só se couber inteiro na parte à vista.
    let tail = ui.doc.hits.split_off(n);
    let seen = |h: &Rect| h.y >= r.y && h.y + h.h <= r.y + r.h;
    ui.doc
        .hits
        .extend(tail.into_iter().filter(|(h, _, _)| seen(h)));
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

/// As filas de um cartão: os números aos pares, o resto sozinho.
fn rows(items: &[Item]) -> Vec<Vec<&Item>> {
    let mut out: Vec<Vec<&Item>> = Vec::new();
    for it in items {
        match (it, out.last_mut()) {
            (Item::Num(..), Some(last)) if last.len() == 1 && matches!(last[0], Item::Num(..)) => {
                last.push(it);
            }
            _ => out.push(vec![it]),
        }
    }
    out
}

fn row_h(ui: &mut Ui<'_>, it: &Item, wide: f32) -> f32 {
    match it {
        Item::Menu(_) => m::LABEL_PX + m::LABEL_BOTTOM + m::FIELD_H,
        #[allow(clippy::cast_precision_loss)]
        Item::Note(k) => wrap(ui, tr(k), wide).len() as f32 * NOTE_LINE,
        _ => m::FIELD_H,
    }
}

/// A cabeça de um cartão: a folga de cima, o título e o vão até ao corpo.
const TOP: f32 = m::CARD_PAD_TOP + m::CARD_HEAD_H + m::CARD_HEAD_BOTTOM;

/// A altura de um cartão na largura `w`.
fn card_h(ui: &mut Ui<'_>, g: &Group, w: f32) -> f32 {
    let iw = (w - m::CARD_PAD_X * 2.0).max(0.0);
    let mut body = 0.0;
    for (k, row) in rows(&g.1).iter().enumerate() {
        body += row_h(ui, row[0], iw) + if k > 0 { m::FIELD_ROW_GAP } else { 0.0 };
    }
    TOP + body + m::CARD_PAD_BOTTOM
}

/// Pinta um cartão no topo de `r` (a largura da coluna) e devolve a altura dele.
fn card(ui: &mut Ui<'_>, r: Rect, g: &Group, at: &mut [Rect; 2]) -> f32 {
    let iw = (r.w - m::CARD_PAD_X * 2.0).max(0.0);
    let h = card_h(ui, g, r.w);
    let bg = ui.c(ColorToken::KanbanCard);
    fill_rounded_rect(ui.scene, Rect::new(r.x, r.y, r.w, h), m::CARD_RADIUS, bg);
    let ink = ui.c(ColorToken::Text1);
    let ty = r.y + m::CARD_PAD_TOP + folga_do_texto(m::CARD_HEAD_H, m::CARD_TITLE_PX);
    let tx = r.x + m::CARD_PAD_X;
    paint_text_title_elided_tracked(
        ui.ts,
        ui.scene,
        tr(g.0),
        [tx, ty],
        m::CARD_TITLE_PX,
        iw,
        ink,
        0.0,
    );
    let mut y = r.y + TOP;
    let half = (iw - m::FIELD_GAP) * 0.5;
    for row in rows(&g.1) {
        let rh = row_h(ui, row[0], iw);
        let wide = if matches!(row[0], Item::Num(..)) {
            half
        } else {
            iw
        };
        for (k, it) in row.into_iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let x = tx + (half + m::FIELD_GAP) * k as f32;
            item(ui, Rect::new(x, y, wide, rh), it, at);
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

fn item(ui: &mut Ui<'_>, b: Rect, it: &Item, at: &mut [Rect; 2]) {
    match it {
        Item::Num(f, glyph, name, unit) => {
            let gl = match *glyph {
                Glyph::Text(k) => Glyph::Text(tr(k)),
                icon @ Glyph::Icon(_) => icon,
            };
            let u = if unit.is_empty() { "" } else { tr(unit) };
            ui.number(b, gl, &full_name(name, unit), u, *f);
        }
        Item::Check(f, name, on) => {
            ui.check(b, tr("joint_editor.on"), tr(name), Hit::Flag(*f), *on);
        }
        Item::Scope(segs) => ui.segmented(b, segs),
        Item::Note(k) => {
            for (n, l) in wrap(ui, tr(k), b.w).iter().enumerate() {
                #[allow(clippy::cast_precision_loss)]
                let lr = Rect::new(b.x, b.y + NOTE_LINE * n as f32, b.w, NOTE_LINE);
                ui.put(l, lr, m::VALUE_PX, ColorToken::Text2);
            }
        }
        Item::Line(s) => ui.put(s, b, m::VALUE_PX, ColorToken::Text2),
        Item::Inspect => ui.button(b, tr("joint_editor.open_inspector"), Hit::Inspect, true),
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
            ui.label(
                label,
                Rect::new(b.x, b.y, b.w, m::LABEL_PX + m::LABEL_BOTTOM),
            );
            let bx = Rect::new(b.x, b.y + m::LABEL_PX + m::LABEL_BOTTOM, b.w, m::FIELD_H);
            ui.dropdown(bx, value, h, &format!("{label}: {value}. {}", tr(tip)));
            at[*menu as usize] = bx;
        }
    }
}
