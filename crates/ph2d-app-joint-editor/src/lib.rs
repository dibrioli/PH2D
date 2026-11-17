//! **A aba Joint Editor** (plano `docs/JointEditor/01_plano.md` §W3, a versão B do dono): montar máquinas
//! físicas por GABARITO — escolher o tipo de junta e o gabarito, arrastar cada objeto da Hierarchy para o
//! papel dele no desenho, e afinar os números com alças e no painel de baixo. As juntas criadas aparecem no
//! cartão Joint do Inspector de cada peça, como as outras (o Inspector não perde nada).
//!
//! A shell chama [`install`] uma vez e [`frame`] uma vez por quadro, com o mundo na mão: é aqui que os
//! pedidos da aba viram mudanças no mundo (um passo de desfazer cada, pelo diff do projeto).

pub use ph2d_joint_editor_doc::{doc, model, stage};
pub mod paint;
mod paint_stage;
mod panel;
pub mod smoke;
mod surface;
mod widgets;

pub use ph2d_joint_editor_apply::{Outcome, apply};
pub use ph2d_joint_editor_doc::{Edit, Focus, Hit, JointEditorDoc};
pub use surface::{CMD_OPEN, JointEditorTab, TAB_ID};
pub use widgets::MOTOR_PREVIEW_S;

use ph2d_ecs::SimWorld;
use ph2d_editor_core::{HeroScreen, Toast, ToastQueue};

pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "joint-editor",
    routers: smoke::ROUTERS,
};

pub fn install(hero: &mut HeroScreen) {
    hero.documents
        .install_surface(Box::new(JointEditorTab::default()));
}

/// A aba é a que está à frente?
#[must_use]
pub fn is_front(hero: &mut HeroScreen) -> bool {
    hero.documents
        .active_surface_mut()
        .is_some_and(|(f, _)| f.as_any().is::<JointEditorTab>())
}

/// **O quadro da família**: abre a aba pedida (o botão do Inspector), aplica os pedidos dela no mundo e
/// relê as montagens. O «Make the parts» anuncia-se em `toasts`; `registry` é o da app (as cópias da fila
/// levam tudo o que a peça tem, como o Duplicate).
pub fn frame(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    toasts: &mut ToastQueue,
    registry: &ph2d_ecs::scene::ComponentRegistry,
) {
    let request = hero.store.take_joint_editor_request();
    let view = hero.grid.view.map(|v| v.camera_center);
    let mut select;
    let mut picked = Vec::new();
    'tab: {
        let Some(tab) = hero.documents.surfaces_mut().find_mut::<JointEditorTab>() else {
            return;
        };
        select = smoke::arm(sim, tab);
        if request.is_some() {
            tab.open();
        }
        if !tab.is_open() {
            break 'tab;
        }
        if let Some(c) = view {
            tab.doc.view_centre = c;
        }
        let mut made = None;
        for e in tab.doc.take_pending() {
            let o = apply(sim, registry, e);
            // A montagem nova abre-se SEMPRE (com ou sem frase): o gesto seguinte cai nela.
            if let Some(sid) = o.made {
                made = Some((sid, o.line.clone().unwrap_or_default()));
            } else if let Some(l) = o.line.clone() {
                tab.doc.note = Some(l);
            }
            select = o.select.or(select);
            if !o.picked.is_empty() {
                toasts.push(Toast::success(o.line.clone().unwrap_or_default()));
                picked = o.picked;
            }
        }
        tab.doc.set_snapshot(model::read(sim));
        if let Some((sid, line)) = made {
            tab.doc.made(sid, line);
        }
        if let Some(sid) = request {
            tab.doc.open_on(sid);
        }
    }
    if let Some((&first, rest)) = picked.split_first() {
        hero.gizmo.selection = Some(first);
        hero.gizmo.extra_selection = rest.to_vec();
    } else if let Some(bits) = select {
        hero.gizmo.selection = Some(bits);
        hero.gizmo.extra_selection.clear();
        if smoke::level() == Some(10) {
            hero.panel_visibility.insert("inspector", true);
            hero.panel_visibility.insert("physics", false);
            // A dobra «Joint» do cartão Body.
            hero.store
                .set_collapsed(ph2d_editor_core::ids::INSP_PHYS_FOLD_JOINT, false);
        }
    }
}

/// Há um gesto a meio na aba (uma alça presa, um número a ser escrito): o desfazer espera por ele.
#[must_use]
pub fn gesture_in_progress(hero: &HeroScreen) -> bool {
    hero.documents
        .surfaces()
        .find::<JointEditorTab>()
        .is_some_and(|t| t.doc.gesture_in_progress())
}
