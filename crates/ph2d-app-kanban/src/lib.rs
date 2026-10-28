//! **A família do KANBAN** (TrelloClone W2, [ADR-0195]) — um Kanban do projecto aberto numa ABA e
//! usado como no Trello: listas e cartões, criar e escrever no lugar, arrastar com mola, inclinação
//! e rolagem nas bordas, cores e recolher listas (recolher é da VISTA), arquivo, atalhos, e o
//! desfazer de cada Kanban.
//!
//! A aba é uma VISTA ([`ph2d_kanban_model::KanbanView`]) sobre o documento que o projecto guarda —
//! o mesmo `Rc` do `Documents` ([`ph2d_editor_core::documents::SharedKanbans`]): não há cópia nem
//! sincronização (o elemento do canvas do Quadro, na W4, é outra vista do mesmo registo).
//!
//! [ADR-0195]: ../../../docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md

mod actions;
mod anim;
mod archive;
mod back;
mod canvas;
mod docs;
mod edit;
mod frame;
mod front;
mod home;
mod keys;
mod layout;
mod list_title;
mod menu;
mod metrics;
mod on_board;
mod overlay;
mod paint;
mod paint_card;
mod paint_list;
mod pointer;
pub mod project;
pub mod smoke;
mod surface;
mod tab;
mod view;

pub use back::outside::{Outside, Served, System, World};
pub use docs::KanbanDocs;
pub use surface::{CMD_HOME, CMD_NEW};

/// O arranque da família: a superfície das abas Kanban no `Documents` e as cenas de smoke armadas.
pub fn install(hero: &mut ph2d_editor_core::screens::hero::HeroScreen) {
    let mut docs = KanbanDocs::default();
    docs.canvas = canvas::install();
    hero.documents.install_surface(Box::new(docs));
    smoke::stage_armed_smokes(hero);
}

/// O quadro da família (a shell chama-o uma vez por quadro de imagem): o conjunto, o relógio, a aba
/// activa — e quem é «eu» e as equipas dos quadros do projecto (os membros do verso).
pub fn frame(hero: &mut ph2d_editor_core::screens::hero::HeroScreen) {
    frame::frame(hero);
    let author = hero.documents.author().trim().to_owned();
    let team: Vec<String> = hero
        .documents
        .boards()
        .boards()
        .iter()
        .flat_map(|b| b.team.people(&author, &[]))
        .collect();
    if let Some(docs) = hero.documents.surfaces_mut().find_mut::<KanbanDocs>() {
        docs.follow_people(&author, team);
    }
}

/// ⭐ A fila do Kanban para o MUNDO (a shell chama-a uma vez por imagem, ao lado do `io::service` do
/// Quadro): o diálogo «Choose a file», os links e os anexos a abrir, o link copiado e os avisos.
pub fn service(hero: &mut ph2d_editor_core::screens::hero::HeroScreen) {
    service_with(hero, &mut System);
}

/// [`service`] por outro mundo (um teste responde no lugar do diálogo e do navegador).
pub fn service_with(hero: &mut ph2d_editor_core::screens::hero::HeroScreen, world: &mut dyn World) {
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<KanbanDocs>() else {
        return;
    };
    let out = docs.serve(world);
    if let Some(text) = out.copy {
        hero.store.set_clipboard_copy(text);
    }
    for n in out.notices {
        hero.documents.tell(n);
    }
}

/// A declaração da família ao registo (`ph2d-app-registry-init`).
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "kanban",
    routers: smoke::ROUTERS,
};

/// O id do nó `Document` da aba `id`.
pub(crate) const A11Y_SALT: u64 = 0x4B41_4E42_0000_0000;
