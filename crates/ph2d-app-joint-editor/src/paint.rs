//! **A pintura da aba** (maquete B, painel do dono 13/11): à esquerda as montagens; no meio o desenho do
//! gabarito; à direita o painel em cartões. As listas suspensas pintam-se por último, por cima.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect_dashed};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_joint_templates::Guide;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

use crate::doc::{Focus, Hit, JointEditorDoc, Menu, kind_key, kinds};
use crate::stage::{StageMap, shown_count};
use crate::widgets::{self as w, Ui};

/// Os rects da aba.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Layout {
    pub list: Rect,
    pub stage: Rect,
    pub panel: Rect,
}

impl Layout {
    /// As montagens à esquerda, o painel da largura do Inspector à direita, o palco com o resto; os três a
    /// toda a altura.
    #[must_use]
    pub fn of(area: Rect, pw: f32) -> Self {
        let g = m::CARD_GAP;
        let h = (area.h - g * 2.0).max(0.0);
        let list = Rect::new(area.x + g, area.y + g, w::LIST_W, h);
        let panel = Rect::new(area.x + area.w - g - pw, list.y, pw, h);
        let x = list.x + list.w + g;
        let stage = Rect::new(x, list.y, (panel.x - g - x).max(0.0), h);
        Self { list, stage, panel }
    }
}

/// Pinta a aba inteira em `area`.
pub fn paint(
    doc: &mut JointEditorDoc,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    doc.hits.clear();
    doc.area = area;
    let ink = CardInk::of_list(theme);
    fill_rounded_rect(scene, area, 0.0, ink.color(ColorToken::KanbanList));
    let mut ui = Ui {
        doc,
        scene,
        ts,
        ink,
        theme,
    };
    let l = Layout::of(area, ui.doc.panel_width());
    ui.doc.panel = l.panel;
    ui.doc.stage = l.stage;
    paint_split(&mut ui, l.panel);
    for r in [l.list, l.stage] {
        fill_rounded_rect(ui.scene, r, m::CARD_RADIUS, ui.c(ColorToken::KanbanCard));
    }
    paint_list(&mut ui, l.list);
    crate::paint_stage::paint(&mut ui, l.stage);
    let at = crate::panel::paint(&mut ui, l.panel);
    if let Some(menu) = ui.doc.menu {
        paint_menu(&mut ui, at[menu as usize], menu);
    }
}

/// A divisória à esquerda do painel (a do Inspector): o vão inteiro é a pega, a barrinha ao meio acende.
fn paint_split(ui: &mut Ui<'_>, panel: Rect) {
    let g = m::CARD_GAP;
    let pega = Rect::new(panel.x - g, panel.y, g, panel.h);
    let hot = ui.doc.hover == Some(Hit::Split) || ui.doc.grab.is_some();
    let (bw, bh) = (m::FIELD_BORDER * 2.0, m::FIELD_H);
    let bar = Rect::new(
        pega.x + (g - bw) * 0.5,
        pega.y + (pega.h - bh) * 0.5,
        bw,
        bh,
    );
    let tone = if hot {
        ColorToken::Accent
    } else {
        ColorToken::Text3
    };
    fill_rounded_rect(ui.scene, bar, bw * 0.5, ui.c(tone));
    ui.hit(pega, Hit::Split, tr("joint_editor.resize_panel"));
}

fn paint_list(ui: &mut Ui<'_>, r: Rect) {
    let i = Rect::new(
        r.x + m::CARD_PAD_X,
        r.y + m::CARD_PAD_TOP,
        r.w - m::CARD_PAD_X * 2.0,
        r.h - m::CARD_PAD_TOP - m::CARD_PAD_BOTTOM,
    );
    ui.label(
        tr("joint_editor.assemblies"),
        Rect::new(i.x, i.y, i.w, m::LABEL_PX + m::LABEL_BOTTOM),
    );
    let mut y = i.y + m::LABEL_PX + m::LABEL_BOTTOM * 2.0;
    let buttons_y = i.y + i.h - m::FIELD_H;
    let rows: Vec<(u64, String, usize)> = ui
        .doc
        .snap
        .assemblies
        .iter()
        .map(|a| (a.sid, a.name.clone(), a.joints.len()))
        .collect();
    let focus = ui.doc.focus;
    let draft = focus == Focus::Draft;
    let mut lines: Vec<(Option<u64>, String, String, usize)> = rows
        .into_iter()
        .enumerate()
        .map(|(k, (sid, name, n))| (Some(sid), name, n.to_string(), k))
        .collect();
    if draft {
        let k = lines.len();
        lines.push((
            None,
            tr("joint_editor.new_assembly").to_owned(),
            "0".to_owned(),
            k,
        ));
    }
    for (sid, name, count, k) in lines {
        if y + m::FIELD_H > buttons_y {
            break;
        }
        let row = Rect::new(i.x, y, i.w, m::FIELD_H);
        let sel = match sid {
            Some(s) => focus == Focus::Assembly(s),
            None => draft,
        };
        let h = sid.map_or(Hit::New, Hit::Row);
        if sel {
            fill_rounded_rect(ui.scene, row, m::FIELD_RADIUS, ui.c(ColorToken::Selection));
        } else if ui.doc.hover.as_ref() == Some(&h) {
            fill_rounded_rect(
                ui.scene,
                row,
                m::FIELD_RADIUS,
                ui.c(ColorToken::CardFieldHover),
            );
        }
        let dot = m::CHECK_BOX * 0.5;
        ph2d_editor_core::paint::fill_circle(
            ui.scene,
            row.x + m::FIELD_PAD_R + dot * 0.5,
            row.y + row.h * 0.5,
            dot * 0.5,
            ui.c(w::palette(k)),
        );
        let tx = row.x + m::FIELD_PAD_R + dot + m::CHECK_GAP;
        let cw = ui.ts.prefix_width(&count, m::VALUE_PX) + m::FIELD_PAD_R;
        ui.put(
            &name,
            Rect::new(tx, row.y, (row.x + row.w - cw - tx).max(0.0), row.h),
            m::VALUE_PX,
            ColorToken::Text1,
        );
        ui.put(
            &count,
            Rect::new(row.x + row.w - cw, row.y, cw, row.h),
            m::VALUE_PX,
            ColorToken::Text2,
        );
        ui.hit(row, h, &name);
        y += m::FIELD_H + m::FIELD_ROW_GAP;
    }
    let half = (i.w - m::FIELD_GAP) * 0.5;
    ui.button(
        Rect::new(i.x, buttons_y, half, m::FIELD_H),
        tr("joint_editor.new"),
        Hit::New,
        true,
    );
    let rm = Rect::new(i.x + half + m::FIELD_GAP, buttons_y, half, m::FIELD_H);
    let on = ui.doc.assembly().is_some();
    ui.button(rm, tr("joint_editor.remove"), Hit::Remove, on);
}

/// A miniatura de um gabarito: as formas-guia tracejadas, sem rótulos (desenho, não campo).
pub fn thumbnail(ui: &mut Ui<'_>, t: &ph2d_joint_templates::Template, r: Rect) {
    let counts: Vec<u16> = t.roles.iter().map(|ro| shown_count(ro, 0)).collect();
    let map = StageMap::fit(t, &counts, r, m::SEG_PAD * 2.0, f32::MAX);
    let c = ui.c(ColorToken::Text2);
    for (k, role) in t.roles.iter().enumerate() {
        for n in 0..counts[k] {
            let s = map.shape_rect(role, n);
            let rad = match role.guide {
                Guide::Disc { .. } => s.w * 0.5,
                Guide::Box { .. } => m::SEG_RADIUS,
            };
            stroke_rounded_rect_dashed(ui.scene, s, rad, m::FIELD_BORDER, m::LOOSE_DASH, c);
        }
    }
}

/// Uma opção da lista: o alvo, o nome, a linha de baixo e a miniatura do gabarito.
type MenuRow = (
    Hit,
    String,
    String,
    Option<&'static ph2d_joint_templates::Template>,
);

fn paint_menu(ui: &mut Ui<'_>, b: Rect, menu: Menu) {
    let rows: Vec<MenuRow> = match menu {
        Menu::Kind => kinds()
            .into_iter()
            .map(|k| {
                let n = ph2d_joint_templates::of_kind(k).count();
                (
                    Hit::Kind(k),
                    tr(kind_key(k)).to_owned(),
                    tr_with("joint_editor.templates_n", &[("n", &n)]),
                    None,
                )
            })
            .collect(),
        Menu::Template => ph2d_joint_templates::of_kind(ui.doc.template().kind)
            .map(|t| {
                let roles: Vec<&str> = t.roles.iter().map(|r| r.name).collect();
                (
                    Hit::Template(t.id.0),
                    t.name.to_owned(),
                    roles.join(" · "),
                    Some(t),
                )
            })
            .collect(),
    };
    let row_h = match menu {
        Menu::Kind => m::FIELD_H + m::LABEL_PX,
        Menu::Template => w::MENU_ROW_H,
    };
    #[allow(clippy::cast_precision_loss)]
    let h = row_h * rows.len() as f32 + m::SEG_PAD * 2.0;
    // Por baixo da caixa; se não cabe até ao fundo da aba, sobe o que falta.
    let area = ui.doc.area;
    let y0 = (b.y + b.h + m::SEG_PAD)
        .min(area.y + area.h - h)
        .max(area.y);
    // A lista abre na largura do cartão (Joint e Template partilham a fila; a miniatura e o nome cabem).
    let p = ui.doc.panel;
    let r = Rect::new(p.x + m::CARD_PAD_X, y0, p.w - m::CARD_PAD_X * 2.0, h);
    let shade = ui.c(ColorToken::KanbanListShadow);
    ph2d_editor_core::paint_flight::shadow(ui.scene, r, m::CARD_RADIUS, shade, m::LIST_SHADOW_FAR);
    fill_rounded_rect(
        ui.scene,
        r,
        m::CARD_RADIUS,
        ui.c(ColorToken::KanbanCardRaised),
    );
    let cur = ui.doc.template();
    // O véu: um clique fora fecha a lista sem fazer mais nada.
    ui.hit(ui.doc.area, Hit::OpenMenu(menu), "");
    for (k, (hit, name, sub, t)) in rows.into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let row = Rect::new(
            r.x + m::SEG_PAD,
            r.y + m::SEG_PAD + row_h * k as f32,
            r.w - m::SEG_PAD * 2.0,
            row_h,
        );
        let on = match hit {
            Hit::Kind(kk) => kk == cur.kind,
            Hit::Template(id) => id == cur.id.0,
            _ => false,
        };
        if on {
            fill_rounded_rect(ui.scene, row, m::FIELD_RADIUS, ui.c(ColorToken::Selection));
        } else if ui.doc.hover.as_ref() == Some(&hit) {
            fill_rounded_rect(
                ui.scene,
                row,
                m::FIELD_RADIUS,
                ui.c(ColorToken::CardFieldHover),
            );
        }
        let mut tx = row.x + m::FIELD_PAD_R;
        if let Some(t) = t {
            let th = Rect::new(
                tx,
                row.y + (row.h - w::THUMB.1) * 0.5,
                w::THUMB.0,
                w::THUMB.1,
            );
            thumbnail(ui, t, th);
            tx += w::THUMB.0 + m::FIELD_GAP;
        }
        let tw = (row.x + row.w - m::FIELD_PAD_R - tx).max(0.0);
        let half_h = row.h * 0.5;
        ui.put(
            &name,
            Rect::new(tx, row.y + m::SEG_PAD, tw, half_h),
            m::VALUE_PX,
            ColorToken::Text1,
        );
        ui.put(
            &sub,
            Rect::new(tx, row.y + half_h - m::SEG_PAD, tw, half_h),
            w::SMALL_PX,
            ColorToken::Text2,
        );
        let a11y = format!("{name}: {sub}");
        ui.hit(row, hit, &a11y);
    }
}
