//! ⭐ **D13 — «Use as Kanban background…»** (dono 30/10: *«os projetos terão milhares de imagens,
//! não podem ser listadas no Kanban; melhor uma opção no menu do botão direito na hierarchy abrindo
//! um modal que escolhe o kanban onde aplicar»*).
//!
//! O caminho: a linha do menu (`CTX_MENU_HIER_KANBAN_BACKGROUND`, só para Image · Flip) →
//! [`ask_background`] abre o modal do «Send to board» com a lista dos Kanbans → a shell lê os píxeis
//! do objecto pela MESMA porta do envio ao quadro → [`scene_background`] guarda o ficheiro no
//! conjunto e grava o fundo como UM gesto (desfaz-se na aba do Kanban, que se abre).

use crate::docs::KanbanDocs;
use ph2d_editor_core::Toast;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_model::{Background, KanbanId, KanbanOp, Plan};

/// O pedido da linha: abre o modal se o objecto tem píxeis (`goes`); senão, o aviso que o diz.
pub fn ask_background(
    hero: &mut HeroScreen,
    row: ph2d_editor_core::NodeId,
    goes: bool,
) -> Option<Toast> {
    if !goes {
        return Some(Toast::warning(tr("kanban.bg.pick_wrong_kind")));
    }
    ph2d_editor_core::screens::hero::send_to_board::open_kanban_background(&mut hero.store, row);
    None
}

/// ⭐ A escolha do modal: o ficheiro `file` (PNG) do objecto `name` passa a ser o fundo do Kanban
/// `kanban` (`None` = um Kanban novo). `Err` = a chave i18n do porquê (com `{name}`). Devolve o aviso
/// a mostrar, se há.
pub fn scene_background(
    hero: &mut HeroScreen,
    kanban: Option<u64>,
    name: &str,
    file: Result<Option<Vec<u8>>, &'static str>,
) -> Option<Toast> {
    let bytes = match file {
        Err(why) => return Some(Toast::warning(tr_with(why, &[("name", &name)]))),
        Ok(None) => {
            return Some(Toast::warning(tr_with(
                "board.send.empty",
                &[("name", &name)],
            )));
        }
        Ok(Some(b)) => b,
    };
    let Some(size) = crate::back::attach::image_size(&bytes) else {
        return Some(Toast::warning(tr_with(
            "kanban.bg.not_image",
            &[("t", &name)],
        )));
    };
    let docs = hero.documents.surfaces_mut().find_mut::<KanbanDocs>()?;
    let kid = match kanban.map(KanbanId) {
        Some(k) if docs.set.borrow().get(k).is_some() => k,
        Some(_) => return Some(Toast::warning(tr("kanban.bg.pick_gone"))),
        None => docs.create(),
    };
    let f = docs.set.borrow_mut().put_file(bytes, Some(size));
    let header = docs.set.borrow().get(kid).map(|k| k.header.clone());
    if let Some(mut h) = header.filter(|h| h.background != Background::Image(f)) {
        h.background = Background::Image(f);
        let mut plan = Plan::default();
        plan.extend(kid, [KanbanOp::Header(h)]);
        crate::hooks::write(docs, Some(plan));
    }
    docs.open(kid);
    None
}
