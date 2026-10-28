//! O popover «Share» (o idioma do «Live» do Quadro): partilhar este Kanban, copiar o link, parar; e
//! entrar por um link colado.

use super::{Look, Role, ShareAct};
use crate::docs::KanbanDocs;
use crate::tab::Tab;
use crate::w6::sheet::{Item, Sheet};
use crate::w6::{Act, Boxed};
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_i18n::tr;

fn button(s: &mut Sheet, key: &str, primary: bool, a: ShareAct) {
    s.push(
        Item::Button {
            text: tr(key).to_owned(),
            primary,
        },
        Act::Share(a),
    );
}

fn note(s: &mut Sheet, text: impl Into<String>) {
    s.push(Item::Note(text.into()), Act::Inside);
}

/// ⭐ A folha, pelo papel de agora.
pub(crate) fn sheet(tab: &Tab, look: &Look) -> Sheet {
    let mut s = Sheet::new(crate::w6::sheet::W);
    s.title("kanban.share.title");
    match &look.role {
        Role::Off => {
            note(&mut s, tr("kanban.share.intro"));
            button(&mut s, "kanban.share.start", true, ShareAct::Start);
            s.caption("kanban.share.join_caption");
            let field = match &tab.w6.input {
                Some((Boxed::Link, f)) => Some(f.clone()),
                _ => None,
            };
            s.push(
                Item::Input {
                    field,
                    text: String::new(),
                    placeholder: tr("kanban.share.link_hint").to_owned(),
                },
                Act::Share(ShareAct::LinkBox),
            );
            button(&mut s, "kanban.share.join", false, ShareAct::Join);
        }
        Role::Hosting { link, lan, .. } => {
            s.caption("kanban.share.link_caption");
            match link {
                Some(l) => note(&mut s, l.clone()),
                None => note(&mut s, tr("kanban.share.preparing")),
            }
            button(&mut s, "kanban.share.copy", true, ShareAct::CopyLink);
            s.caption("kanban.share.lan_caption");
            note(&mut s, lan.clone());
            button(&mut s, "kanban.share.stop", false, ShareAct::Stop);
        }
        Role::Joining => {
            note(&mut s, tr("kanban.share.joining"));
            button(&mut s, "kanban.share.cancel", false, ShareAct::Stop);
        }
        Role::Guest { .. } => {
            note(&mut s, tr("kanban.share.guest"));
            button(&mut s, "kanban.share.leave", false, ShareAct::Stop);
        }
    }
    s
}

/// O texto da caixa do link.
fn link_text(docs: &mut KanbanDocs, id: u64) -> Option<String> {
    match docs.tab_mut(id)?.w6.input.as_ref() {
        Some((Boxed::Link, f)) => Some(f.text.clone()),
        _ => None,
    }
}

/// Um clique no popover.
pub(crate) fn click(docs: &mut KanbanDocs, id: u64, a: ShareAct) {
    match a {
        ShareAct::Start => docs.share_start(id),
        ShareAct::CopyLink => {
            let text = match &docs.share.look.role {
                Role::Hosting { link: Some(l), .. } => Some(l.clone()),
                Role::Hosting { lan, .. } => Some(lan.clone()),
                _ => None,
            };
            if let Some(t) = text {
                docs.share.copy = Some(t);
                docs.notices.push(tr("kanban.share.copied").to_owned());
            }
        }
        ShareAct::Stop => docs.share_stop(),
        ShareAct::LinkBox => {
            if let Some(t) = docs.tab_mut(id) {
                t.w6.focus(Boxed::Link, "");
            }
        }
        ShareAct::Join => {
            if let Some(text) = link_text(docs, id).filter(|t| !t.trim().is_empty()) {
                docs.share_join(text.trim());
            }
        }
    }
}

/// O teclado com o popover aberto: a caixa do link escreve; `Enter` entra; `Esc` fecha.
pub(crate) fn key(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> KeyOutcome {
    if !input.pressed {
        return crate::w6::taken();
    }
    if input.key == Key::Escape {
        crate::w6::close_top(docs, id);
        return crate::w6::taken();
    }
    if link_text(docs, id).is_none() {
        return KeyOutcome::default();
    }
    if input.key == Key::Enter {
        click(docs, id, ShareAct::Join);
        return crate::w6::taken();
    }
    crate::w6::outcome(crate::w6::type_into(docs, id, input))
}
