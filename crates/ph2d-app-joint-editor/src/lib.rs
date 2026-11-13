//! **A aba Joint Editor** (plano `docs/JointEditor/01_plano.md` §W3, a versão B do dono): montar máquinas
//! físicas por GABARITO — escolher o tipo de junta e o gabarito, arrastar cada objeto da Hierarchy para o
//! papel dele no desenho, e afinar os números com alças e no painel de baixo. As juntas criadas aparecem no
//! cartão Joint do Inspector de cada peça, como as outras (o Inspector não perde nada).
//!
//! A shell chama [`install`] uma vez e [`frame`] uma vez por quadro, com o mundo na mão: é aqui que os
//! pedidos da aba viram mudanças no mundo (um passo de desfazer cada, pelo diff do projeto).

mod apply;
pub mod doc;
mod input;
pub mod model;
pub mod paint;
mod paint_stage;
mod panel;
pub mod smoke;
pub mod stage;
mod surface;
mod widgets;

pub use apply::{Outcome, apply};
pub use doc::{Edit, Focus, Hit, JointEditorDoc};
pub use surface::{CMD_OPEN, JointEditorTab, TAB_ID};

use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;

pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "joint-editor",
    routers: smoke::ROUTERS,
};

pub fn install(hero: &mut HeroScreen) {
    hero.documents.install_surface(Box::new(JointEditorTab::default()));
}

/// A aba é a que está à frente?
#[must_use]
pub fn is_front(hero: &mut HeroScreen) -> bool {
    hero.documents
        .active_surface_mut()
        .is_some_and(|(f, _)| f.as_any().is::<JointEditorTab>())
}

/// **O quadro da família**: abre a aba pedida (o botão do Inspector), aplica os pedidos dela no mundo e
/// relê as montagens.
pub fn frame(sim: &mut SimWorld, hero: &mut HeroScreen) {
    let request = hero.store.take_joint_editor_request();
    let mut select = None;
    {
        let Some(tab) = hero.documents.surfaces_mut().find_mut::<JointEditorTab>() else {
            return;
        };
        smoke::arm(sim, tab);
        if request.is_some() {
            tab.open();
        }
        if !tab.is_open() {
            return;
        }
        let mut made = None;
        for e in tab.doc.take_pending() {
            let o = apply(sim, e);
            if o.made.is_some() {
                made = o.made.zip(o.line.clone());
            } else if let Some(l) = o.line {
                tab.doc.note = Some(l);
            }
            select = o.select.or(select);
        }
        tab.doc.set_snapshot(model::read(sim));
        if let Some((sid, line)) = made {
            tab.doc.made(sid, line);
        }
        if let Some(sid) = request {
            tab.doc.open_on(sid);
        }
    }
    if let Some(bits) = select {
        hero.gizmo.selection = Some(bits);
        hero.gizmo.extra_selection.clear();
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
