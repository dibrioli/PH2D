//! **O palco**: o gabarito desenhado — uma forma tracejada por papel vazio, a peça do artista (cheia, com o
//! nome dela) em cada papel ocupado, e por cima o que cada junta faz: a linha e a barra do curso (amarelo e
//! laranja), a seta do motor (verde), o arco dos batentes, a cota do entre-eixos (azul), com alças.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, stroke_polyline, stroke_rounded_rect,
    stroke_rounded_rect_dashed,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_joint_templates::{Guide, JointKind, Link, Template};
use ph2d_tokens::ColorToken;
use ph2d_vector::Color;

use crate::doc::{Handle, Hit};
use crate::stage::{StageMap, centre_of, rule_ends, shown_count};
use crate::widgets::{self as w, Ui};

/// Como uma alça lê o cursor.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Geo {
    /// Ao longo de `dir` (no ecrã) a partir de `origin`, em metros (`scale` px/m).
    Linear {
        origin: (f32, f32),
        dir: (f32, f32),
        scale: f32,
    },
    /// O ângulo em volta de `centre`, em graus, a contar de `zero` (rad, no ecrã com y para cima).
    Angular { centre: (f32, f32), zero: f32 },
}

impl Geo {
    /// O valor (na unidade do cartão) que o cursor em `(x, y)` diz.
    #[must_use]
    pub fn value_at(self, x: f32, y: f32) -> f32 {
        match self {
            Self::Linear { origin, dir, scale } => {
                ((x - origin.0) * dir.0 + (y - origin.1) * dir.1) / scale.max(f32::EPSILON)
            }
            Self::Angular { centre, zero } => {
                let a = libm::atan2f(centre.1 - y, x - centre.0) - zero;
                let a = libm::atan2f(libm::sinf(a), libm::cosf(a));
                a.to_degrees()
            }
        }
    }
}

/// A direcção no ecrã de um ângulo do gabarito (y para cima).
fn screen_dir(rad: f32) -> (f32, f32) {
    let (s, c) = libm::sincosf(rad);
    (c, -s)
}

fn add(p: (f32, f32), d: (f32, f32), k: f32) -> (f32, f32) {
    (p.0 + d.0 * k, p.1 + d.1 * k)
}

pub fn paint(ui: &mut Ui<'_>, r: Rect) {
    ui.doc.geo.clear();
    let t: &'static Template = ui.doc.template();
    let placed: Vec<usize> = (0..t.roles.len())
        .map(|i| {
            ui.doc.assembly().map_or(0, |a| {
                a.parts.iter().filter(|p| usize::from(p.role) == i).count()
            })
        })
        .collect();
    let counts: Vec<u16> = t
        .roles
        .iter()
        .zip(&placed)
        .map(|(ro, &n)| shown_count(ro, n))
        .collect();
    let map = StageMap::fit(t, &counts, r, w::STAGE_PAD, w::MAX_SCALE);
    ui.doc.map = Some(map);
    ui.doc.counts.clone_from(&counts);
    // O chão do desenho.
    let gy = map.px([0.0, 0.0]).1;
    let ground = [(r.x + m::CARD_PAD_X, gy), (r.x + r.w - m::CARD_PAD_X, gy)];
    stroke_polyline(ui.scene, &ground, m::FIELD_BORDER, ui.c(ColorToken::Border));
    paint_shapes(ui, t, &map, &counts);
    paint_joints(ui, t, &map, &counts);
    paint_wheelbase(ui, t, &map);
    paint_foot(ui, r);
}

fn paint_shapes(ui: &mut Ui<'_>, t: &Template, map: &StageMap, counts: &[u16]) {
    let target = ui
        .doc
        .dropping
        .as_ref()
        .and_then(|d| d.role.map(|ro| (ro, d.ok)));
    // As caixas primeiro: as rodas por cima do corpo.
    let mut order: Vec<usize> = (0..t.roles.len()).collect();
    order.sort_by_key(|&i| matches!(t.roles[i].guide, Guide::Disc { .. }));
    let tone = ui
        .doc
        .assembly()
        .and_then(|a| ui.doc.snap.assemblies.iter().position(|x| x.sid == a.sid))
        .unwrap_or(ui.doc.snap.assemblies.len());
    for i in order {
        let role = &t.roles[i];
        let ri = u16::try_from(i).unwrap_or(u16::MAX);
        let parts: Vec<(String, u64)> = ui
            .doc
            .assembly()
            .map(|a| a.parts_of(ri).map(|p| (p.name.clone(), p.bits)).collect())
            .unwrap_or_default();
        for n in 0..counts[i] {
            let s = map.shape_rect(role, n);
            let rad = match role.guide {
                Guide::Disc { .. } => s.w * 0.5,
                Guide::Box { .. } => m::FIELD_RADIUS,
            };
            let part = parts.get(usize::from(n));
            let lit = target.filter(|(ro, _)| *ro == ri);
            match (part, lit) {
                (_, Some((_, ok))) => {
                    let (fill, line) = if ok {
                        (ColorToken::AccentSoft, ColorToken::Accent)
                    } else {
                        (ColorToken::DangerSoft, ColorToken::Danger)
                    };
                    fill_rounded_rect(ui.scene, s, rad, ui.c(fill));
                    stroke_rounded_rect_dashed(
                        ui.scene,
                        s,
                        rad,
                        w::LINE,
                        m::LOOSE_DASH,
                        ui.c(line),
                    );
                }
                (Some((_, bits)), None) if *bits == 0 => {
                    // O papel é o cenário: um traço cheio, sem peça.
                    stroke_rounded_rect(ui.scene, s, rad, w::LINE, ui.c(ColorToken::Text2));
                }
                (Some(_), None) => match role.guide {
                    // A peça cheia tem a cor da montagem na lista (maquete B3: o «Car 1» vermelho).
                    Guide::Box { .. } => {
                        let c = ui.c(w::palette(tone));
                        fill_rounded_rect(ui.scene, s, rad, c.multiply_alpha(w::PART_FILL_ALPHA));
                        stroke_rounded_rect(ui.scene, s, rad, w::LINE, c);
                    }
                    // A roda é um pneu: escuro, com aro e raios.
                    Guide::Disc { .. } => tyre(ui, s),
                },
                (None, None) => {
                    stroke_rounded_rect_dashed(
                        ui.scene,
                        s,
                        rad,
                        w::LINE,
                        m::LOOSE_DASH,
                        ui.c(ColorToken::Text2),
                    );
                }
            }
            let label = match part {
                Some((name, 0)) if name.is_empty() => tr("joint_editor.scene").to_owned(),
                Some((name, _)) => name.clone(),
                None if role.many.is_some() => format!("{} {}", role.name, n + 1),
                None => role.name.to_owned(),
            };
            let below = Rect::new(
                s.x - s.w,
                s.y + s.h + m::SEG_PAD,
                s.w * 3.0,
                m::LABEL_PX + m::LABEL_BOTTOM,
            );
            // Vazio, o nome mora DENTRO da forma, no terço de baixo (o eixo passa pelo meio).
            let inside = Rect::new(s.x, s.y + s.h * 0.62, s.w, m::LABEL_PX + m::LABEL_BOTTOM);
            let (lr, px, c) = if part.is_some() {
                (below, w::SMALL_PX, ui.c(ColorToken::Text1))
            } else {
                (inside, w::SMALL_PX, ui.c(ColorToken::Text2))
            };
            // Numa fila longa (a ponte) só a primeira e a última dizem o nome: o resto não cabe.
            if role.many.is_none() || n == 0 || n + 1 == counts[i] {
                tag(ui, &label, lr, px, c);
            }
            let a11y = tr_with(
                "joint_editor.shape_a11y",
                &[("role", &role.name), ("name", &label)],
            );
            ui.hit(s, Hit::Shape(ri, n), &a11y);
            if part.is_none() && role.may_be_world && n == 0 && ui.doc.dropping.is_none() {
                let bw = s.w.max(m::FIELD_VALUE_MIN_W * 2.0);
                let b = Rect::new(
                    s.x + (s.w - bw) * 0.5,
                    s.y + s.h + m::SEG_PAD,
                    bw,
                    m::FIELD_H,
                );
                ui.button(b, tr("joint_editor.use_scene"), Hit::World(ri), false);
            }
        }
    }
}

/// Uma roda cheia: o pneu escuro, o aro e quatro raios.
fn tyre(ui: &mut Ui<'_>, s: Rect) {
    let (cx, cy, r) = (s.x + s.w * 0.5, s.y + s.h * 0.5, s.w * 0.5);
    fill_circle(ui.scene, cx, cy, r, ui.c(ColorToken::KanbanList));
    let rim = ui.c(ColorToken::Text2);
    stroke_rounded_rect(ui.scene, s, r, w::LINE * 1.5, rim);
    for k in 0..4 {
        #[allow(clippy::cast_precision_loss)]
        let d = screen_dir(std::f32::consts::FRAC_PI_4 * (2 * k + 1) as f32);
        stroke_polyline(
            ui.scene,
            &[add((cx, cy), d, -r * 0.7), add((cx, cy), d, r * 0.7)],
            m::FIELD_BORDER,
            rim.multiply_alpha(w::PART_FILL_ALPHA * 2.0),
        );
    }
}

/// Um rótulo com fundo do palco por trás: legível por cima de linhas e formas.
fn tag(ui: &mut Ui<'_>, s: &str, r: Rect, px: f32, c: Color) {
    let tw = ui.ts.prefix_width(s, px).min(r.w) + m::SEG_PAD * 4.0;
    let bg = Rect::new(r.x + (r.w - tw) * 0.5, r.y, tw, r.h);
    fill_rounded_rect(ui.scene, bg, m::SEG_RADIUS, ui.c(ColorToken::KanbanCard));
    w::text_centred(ui.ts, ui.scene, s, r, px, c);
}

/// Os pares de formas que cada regra liga, no desenho.
fn pairs(t: &Template, rule: u16, counts: &[u16]) -> Vec<[u16; 2]> {
    let Some(r) = t.rule(rule) else {
        return Vec::new();
    };
    let nb = counts.get(usize::from(r.b)).copied().unwrap_or(1).max(1);
    match r.link {
        Link::Pair | Link::ToFirst => vec![[0, 0]],
        Link::ToLast => vec![[0, nb - 1]],
        Link::Chain => (0..nb.saturating_sub(1)).map(|k| [k, k + 1]).collect(),
    }
}

fn handle(ui: &mut Ui<'_>, at: (f32, f32), h: Handle, geo: Geo, label: &str) {
    let hot = ui.doc.hover == Some(Hit::Handle(h)) || ui.doc.drag.is_some_and(|d| d.handle == h);
    let rad = if hot { w::HANDLE_R * 1.3 } else { w::HANDLE_R };
    fill_circle(ui.scene, at.0, at.1, rad, ui.c(ColorToken::KanbanCard));
    let ring = ui.c(ColorToken::Accent);
    let box_ = Rect::new(at.0 - rad, at.1 - rad, rad * 2.0, rad * 2.0);
    stroke_rounded_rect(ui.scene, box_, rad, w::LINE, ring);
    let hit = Rect::new(
        at.0 - w::HANDLE_R * 2.0,
        at.1 - w::HANDLE_R * 2.0,
        w::HANDLE_R * 4.0,
        w::HANDLE_R * 4.0,
    );
    ui.hit(hit, Hit::Handle(h), label);
    ui.doc.geo.push((h, geo));
}

fn arrow_head(ui: &mut Ui<'_>, tip: (f32, f32), dir: (f32, f32), c: Color) {
    let k = w::HANDLE_R * 1.4;
    let n = (-dir.1, dir.0);
    let back = add(tip, dir, -k);
    let pts = [tip, add(back, n, k * 0.6), add(back, n, -k * 0.6)];
    ph2d_editor_core::paint_shapes::fill_polygon(ui.scene, &pts, c);
}

/// O valor em curso de uma alça presa, ou o guardado.
fn live(ui: &Ui<'_>, h: Handle, stored: f32) -> f32 {
    ui.doc
        .drag
        .filter(|d| d.handle == h)
        .map_or(stored, |d| d.value)
}

fn paint_joints(ui: &mut Ui<'_>, t: &Template, map: &StageMap, counts: &[u16]) {
    let targets: Vec<u16> = ui.doc.targets().iter().map(|j| j.rule).collect();
    let (warn, ok, line) = (
        ui.c(ColorToken::Warn),
        ui.c(ColorToken::Success),
        ui.c(ColorToken::Text2),
    );
    for (ri, rule) in t.joints.iter().enumerate() {
        let ri = u16::try_from(ri).unwrap_or(u16::MAX);
        let row = ui
            .doc
            .assembly()
            .and_then(|a| a.joints_of(ri).next().cloned());
        let info = row.as_ref().and_then(|j| j.info.clone());
        let fab = rule.joint(0.0);
        // A montagem por fazer mostra os números de fábrica na unidade do cartão (graus na dobradiça).
        let deg = |v: f32, metres: bool| if metres { v } else { v.to_degrees() };
        let (lm, mm) = (rule.kind.limits_in_metres(), rule.kind.motor_in_metres());
        let fab_limits = (
            fab.limits_enabled,
            deg(fab.limit_min, lm),
            deg(fab.limit_max, lm),
        );
        let limits = info.as_ref().map_or(fab_limits, |i| {
            (i.limits_enabled, i.limit_min_ui, i.limit_max_ui)
        });
        let motor = info.as_ref().map_or(fab.motor_enabled, |i| i.motor_enabled);
        let speed = info
            .as_ref()
            .map_or(deg(fab.motor_speed, mm), |i| i.motor_speed_ui);
        let editable = row.is_some() && targets.contains(&ri);
        for n in pairs(t, ri, counts) {
            let Some((ea, eb)) = rule_ends(t, ri, n) else {
                continue;
            };
            let (pa, pb) = (map.px(ea), map.px(eb));
            let first = n == pairs(t, ri, counts)[0];
            match rule.kind {
                JointKind::Wheel | JointKind::Slider => {
                    let dir = screen_dir(rule.axis.unwrap_or(0.0));
                    let reach = t
                        .role(rule.b)
                        .map_or(0.3, |r| r.guide.half()[1].max(r.guide.half()[0]))
                        * map.scale
                        * 1.15;
                    let axis = [add(pa, dir, -reach), add(pa, dir, reach)];
                    dashed_line(ui, axis, warn);
                    if limits.0 && row.is_some() {
                        let (lo, hi) = (
                            live(ui, Handle::LimitMin, limits.1),
                            live(ui, Handle::LimitMax, limits.2),
                        );
                        let bar = [add(pa, dir, lo * map.scale), add(pa, dir, hi * map.scale)];
                        stroke_polyline(ui.scene, &bar, w::BAR, warn);
                        if editable && first {
                            let geo = Geo::Linear {
                                origin: pa,
                                dir,
                                scale: map.scale,
                            };
                            handle(
                                ui,
                                bar[0],
                                Handle::LimitMin,
                                geo,
                                tr("joint_editor.handle.min"),
                            );
                            handle(
                                ui,
                                bar[1],
                                Handle::LimitMax,
                                geo,
                                tr("joint_editor.handle.max"),
                            );
                        }
                    }
                    let wheel_r = t.role(rule.b).map_or(0.3, |r| r.guide.half()[0]) * map.scale;
                    let motor = motor && row.is_some();
                    if motor && rule.kind == JointKind::Wheel {
                        motor_arc(ui, pa, wheel_r * 0.6, speed, ok);
                    } else if motor {
                        let tip = add(pa, dir, reach * speed.signum());
                        stroke_polyline(ui.scene, &[pa, tip], w::LINE, ok);
                        arrow_head(
                            ui,
                            tip,
                            (dir.0 * speed.signum(), dir.1 * speed.signum()),
                            ok,
                        );
                    }
                    fill_circle(ui.scene, pa.0, pa.1, w::HANDLE_R * 0.5, warn);
                }
                JointKind::Pin | JointKind::Weld => {
                    fill_circle(ui.scene, pa.0, pa.1, w::HANDLE_R * 0.7, line);
                    let cb = t.role(rule.b).map_or(ea, |r| centre_of(r, n[1]));
                    let zero = libm::atan2f(cb[1] - ea[1], cb[0] - ea[0]);
                    let rad = t
                        .role(rule.b)
                        .map_or(0.3, |r| r.guide.half()[0].max(r.guide.half()[1]))
                        * map.scale
                        * 0.8;
                    if rule.kind == JointKind::Pin && limits.0 && row.is_some() {
                        let (lo, hi) = (
                            live(ui, Handle::LimitMin, limits.1),
                            live(ui, Handle::LimitMax, limits.2),
                        );
                        let arc: Vec<(f32, f32)> = (0..=24)
                            .map(|k| {
                                let a = zero + (lo + (hi - lo) * k as f32 / 24.0).to_radians();
                                add(pa, screen_dir(a), rad)
                            })
                            .collect();
                        stroke_polyline(ui.scene, &arc, w::BAR, warn);
                        if editable && first {
                            let geo = Geo::Angular { centre: pa, zero };
                            let ends = (arc[0], arc[arc.len() - 1]);
                            handle(
                                ui,
                                ends.0,
                                Handle::LimitMin,
                                geo,
                                tr("joint_editor.handle.min"),
                            );
                            handle(
                                ui,
                                ends.1,
                                Handle::LimitMax,
                                geo,
                                tr("joint_editor.handle.max"),
                            );
                        }
                    }
                    if motor && row.is_some() {
                        motor_arc(ui, pa, rad * 0.6, speed, ok);
                    }
                }
                _ => {
                    // As de duas pontas: a corda tracejada, a mola e a haste cheias.
                    if rule.kind == JointKind::Rope {
                        dashed_line(ui, [pa, pb], line);
                    } else {
                        stroke_polyline(ui.scene, &[pa, pb], w::LINE, line);
                    }
                    fill_circle(ui.scene, pa.0, pa.1, w::HANDLE_R * 0.5, line);
                    fill_circle(ui.scene, pb.0, pb.1, w::HANDLE_R * 0.5, line);
                }
            }
        }
    }
}

fn dashed_line(ui: &mut Ui<'_>, [a, b]: [(f32, f32); 2], c: Color) {
    let len = libm::hypotf(b.0 - a.0, b.1 - a.1);
    if len <= f32::EPSILON {
        return;
    }
    #[allow(clippy::cast_possible_truncation)]
    let step = (m::LOOSE_DASH.0 + m::LOOSE_DASH.1) as f32;
    let on = m::LOOSE_DASH.0 as f32;
    let d = ((b.0 - a.0) / len, (b.1 - a.1) / len);
    let mut s = 0.0;
    while s < len {
        let e = (s + on).min(len);
        stroke_polyline(ui.scene, &[add(a, d, s), add(a, d, e)], w::LINE, c);
        s += step;
    }
}

/// A seta curva do motor: o sentido é o da velocidade (negativo = para a frente, `+x`).
fn motor_arc(ui: &mut Ui<'_>, c: (f32, f32), rad: f32, speed: f32, col: Color) {
    let dir = if speed < 0.0 { -1.0 } else { 1.0 };
    let (a0, span) = (
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI * 1.2 * dir,
    );
    let pts: Vec<(f32, f32)> = (0..=20)
        .map(|k| add(c, screen_dir(a0 + span * k as f32 / 20.0), rad))
        .collect();
    stroke_polyline(ui.scene, &pts, w::LINE, col);
    let end = a0 + span;
    let tangent = screen_dir(end + std::f32::consts::FRAC_PI_2 * dir);
    arrow_head(ui, pts[pts.len() - 1], tangent, col);
}

/// A cota do entre-eixos (Wheel com duas rodas): a distância medida na CENA, com a alça na roda da frente.
fn paint_wheelbase(ui: &mut Ui<'_>, t: &Template, map: &StageMap) {
    let wheels: Vec<&ph2d_joint_templates::JointRule> = t
        .joints
        .iter()
        .filter(|r| r.kind == JointKind::Wheel)
        .collect();
    let [rear, front] = wheels[..] else {
        return;
    };
    let (Some(rr), Some(fr), Some(body)) = (t.role(rear.b), t.role(front.b), t.role(rear.a)) else {
        return;
    };
    let top = body.at[1] + body.guide.half()[1];
    let y = map.px([0.0, top]).1 - m::FIELD_H;
    let x0 = map.px(rr.at).0;
    let measured = ui.doc.wheelbase();
    let template_len = fr.at[0] - rr.at[0];
    let len = live(ui, Handle::Wheelbase, measured.unwrap_or(template_len));
    let x1 = x0 + len * map.scale;
    let c = ui.c(ColorToken::Info);
    stroke_polyline(ui.scene, &[(x0, y), (x1, y)], m::FIELD_BORDER, c);
    for x in [x0, x1] {
        stroke_polyline(
            ui.scene,
            &[(x, y - w::HANDLE_R), (x, y + w::HANDLE_R)],
            m::FIELD_BORDER,
            c,
        );
    }
    let label = tr_with("joint_editor.wheelbase_m", &[("m", &w::format(len))]);
    let lr = Rect::new(
        x0,
        y - m::LABEL_PX - m::LABEL_BOTTOM * 2.0,
        (x1 - x0).max(1.0),
        m::LABEL_PX + m::LABEL_BOTTOM,
    );
    w::text_centred(ui.ts, ui.scene, &label, lr, w::SMALL_PX, c);
    if measured.is_some() {
        let geo = Geo::Linear {
            origin: (x0, y),
            dir: (1.0, 0.0),
            scale: map.scale,
        };
        handle(
            ui,
            (x1, y),
            Handle::Wheelbase,
            geo,
            tr("joint_editor.handle.wheelbase"),
        );
    }
}

/// O pé do palco: o que a queda faria, ou o que a última fez, ou como começar.
fn paint_foot(ui: &mut Ui<'_>, r: Rect) {
    let (s, tok) = if let Some(d) = &ui.doc.dropping {
        (
            d.line.clone(),
            if d.ok {
                ColorToken::Accent
            } else {
                ColorToken::Danger
            },
        )
    } else if let Some(n) = &ui.doc.note {
        (n.clone(), ColorToken::Text2)
    } else if ui.doc.assembly().is_some() {
        (
            tr("joint_editor.hint_assembled").to_owned(),
            ColorToken::Text2,
        )
    } else {
        (tr("joint_editor.hint").to_owned(), ColorToken::Text2)
    };
    let fr = Rect::new(
        r.x + m::CARD_PAD_X,
        r.y + r.h - m::FIELD_H,
        r.w - m::CARD_PAD_X * 2.0,
        m::FIELD_H,
    );
    ui.put(&s, fr, m::VALUE_PX, tok);
}
