//! Planear (W6) pelo caminho REAL do ecrã: o botão do Mapa mental e a galeria dos Modelos pela ordem
//! da shell (`shell_click_bar`), o rato pela porta da shell, as teclas pela do teclado — e a fronteira
//! do `Tab` entre um NÓ (o filho) e uma NOTA (a seguinte), que é a mesma tecla.

use super::*;
use crate::screens::hero::board_bar::mind::{MIND_TOOL, MindItem, TEMPLATES};
use ph2d_board_edit::Template;

const NONE: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};
const SHIFT: Modifiers = Modifiers {
    shift: true,
    ctrl: false,
    alt: false,
    meta: false,
};

fn mind(m: MindItem) -> Item {
    Item::Mind(m)
}

impl T {
    fn doc(&self) -> &ph2d_board_model::BoardDoc {
        &self.hero.documents.active_board().unwrap().doc
    }

    fn editor(&self) -> &ph2d_board_edit::Editor {
        self.hero.documents.live.editor.as_ref().unwrap()
    }

    fn write(&mut self, s: &str) {
        for ch in s.chars() {
            let k = BoardKey::Char(ch.to_ascii_lowercase());
            self.key(k, NONE, Some(&ch.to_string()));
        }
    }

    fn focus(&self) -> ph2d_board_model::ElementId {
        self.editor().mind_focus(self.doc()).expect("um nó em foco")
    }

    fn text_of(&self, id: ph2d_board_model::ElementId) -> String {
        self.doc()
            .get(id)
            .unwrap()
            .shape()
            .unwrap()
            .text
            .as_str()
            .to_owned()
    }

    fn select(&mut self, id: ph2d_board_model::ElementId) {
        let (board, live) = self.hero.documents.active_parts().unwrap();
        live.editor.as_mut().unwrap().select(&board.doc, [id]);
        self.paint();
    }

    /// Começa um mapa com o botão da barra e um clique no meio do quadro.
    fn start_map(&mut self, root: &str) -> ph2d_board_model::ElementId {
        self.shell_click_bar(MIND_TOOL);
        assert_eq!(self.editor().tool, Tool::MindMap);
        let p = self.at(0.5, 0.5);
        self.drag(p, p);
        self.write(root);
        self.focus()
    }
}

/// ⭐ O dono vê: começar um mapa pelo botão e expandi-lo pelo TECLADO — `Tab` filho, `Enter` irmão,
/// `Shift+Enter` uma linha nova, `Esc`, as setas a andar (sem empurrar), `Delete` leva o ramo,
/// `Ctrl+Z` devolve-o.
#[test]
fn a_map_is_started_from_the_bar_and_grown_from_the_keyboard() {
    let mut t = T::new();
    let root = t.start_map("Plan");
    assert_eq!(t.text_of(root), "Plan");
    assert!(t.key(BoardKey::Tab, NONE, None));
    t.write("Ideas");
    let a = t.focus();
    assert!(t.key(BoardKey::Enter, NONE, None));
    t.write("Risks");
    let b = t.focus();
    assert_ne!(a, b);
    assert_eq!(t.doc().mind_children(root), vec![a, b]);
    assert!(
        t.key(BoardKey::Enter, SHIFT, None),
        "Shift+Enter: uma linha nova"
    );
    t.write("two");
    assert_eq!(t.text_of(b), "Risks\ntwo");
    assert_eq!(t.doc().mind_children(root).len(), 2);
    assert!(t.key(BoardKey::Escape, NONE, None));
    assert!(!t.editor().is_editing_text());
    let before = t.doc().get(b).unwrap().clone();
    assert!(t.key(BoardKey::Up, NONE, None));
    assert_eq!(t.focus(), a, "a seta anda para o irmão de cima");
    assert_eq!(t.doc().get(b).unwrap().y, before.y, "e não empurra ninguém");
    assert!(t.key(BoardKey::Left, NONE, None));
    assert_eq!(t.focus(), root);
    assert!(t.key(BoardKey::Right, NONE, None));
    let target = t.focus();
    assert_ne!(target, root, "da raiz, a seta para o lado dá um filho");
    assert!(
        t.key(BoardKey::Tab, NONE, None),
        "Tab num nó seleccionado também"
    );
    t.write("Next");
    let a1 = t.focus();
    assert_eq!(t.doc().mind_parent(a1), Some(target));
    assert!(t.key(BoardKey::Escape, NONE, None));
    let ctrl = Modifiers { ctrl: true, ..NONE };
    let parent = t.doc().mind_parent(a1).unwrap();
    t.select(parent);
    assert!(t.key(BoardKey::Up, ctrl, None), "Ctrl+Up recolhe");
    assert!(t.doc().get(a1).unwrap().is_hidden());
    assert!(t.key(BoardKey::Down, ctrl, None), "Ctrl+Down expande");
    assert!(!t.doc().get(a1).unwrap().is_hidden());
    assert!(t.key(BoardKey::Delete, NONE, None));
    assert!(t.doc().get(a1).is_none(), "Delete leva o ramo");
    assert_eq!(t.focus(), root, "e selecciona o pai");
    assert!(t.key(BoardKey::Char('z'), ctrl, None));
    assert!(t.doc().get(a1).is_some());
}

/// ⭐ A fronteira (W3 × W6): a escrever numa NOTA, o `Tab` continua a ser a nota seguinte e o `Enter`
/// uma linha nova — o «filho» e o «irmão» são só dos nós.
#[test]
fn a_note_keeps_its_tab_and_enter() {
    let mut t = T::new();
    assert!(t.key(BoardKey::Char('n'), NONE, Some("n")));
    let p = t.at(0.3, 0.3);
    t.drag(p, p);
    assert!(t.key(BoardKey::Enter, NONE, None));
    t.write("hi");
    assert!(t.key(BoardKey::Enter, NONE, None));
    assert!(t.key(BoardKey::Tab, NONE, None));
    let notes: Vec<_> = t
        .elements()
        .into_iter()
        .filter(|e| e.shape().is_some_and(|s| s.kind.is_note()))
        .collect();
    assert_eq!(notes.len(), 2, "Tab numa nota: a seguinte");
    assert!(notes.iter().all(|n| n.mind().is_none()));
    assert!(
        notes
            .iter()
            .any(|n| n.shape().unwrap().text.as_str() == "hi\n")
    );
}

/// ⭐ A galeria dos modelos abre-se pelo botão, insere no centro da vista (seleccionado) e fecha.
#[test]
fn the_templates_gallery_inserts_a_template_at_the_centre_of_the_view() {
    let mut t = T::new();
    t.shell_click_bar(TEMPLATES);
    for i in 0..Template::ALL.len() {
        assert!(
            t.find(node(mind(MindItem::Template(i)))).is_some(),
            "o modelo {i} não está na galeria"
        );
    }
    let k = Template::ALL
        .iter()
        .position(|x| *x == Template::Kanban)
        .unwrap();
    t.shell_click_bar(mind(MindItem::Template(k)));
    assert_eq!(t.doc().frames().count(), 3, "o kanban tem três colunas");
    assert!(!t.editor().selection().is_empty(), "entra seleccionado");
    assert!(
        t.find(node(mind(MindItem::Template(0)))).is_none(),
        "a galeria fecha"
    );
    let cam = t.hero.documents.active_board().unwrap().camera;
    let f: Vec<_> = t.doc().frames().map(|f| f.aabb()).collect();
    let x0 = f.iter().map(|b| b[0]).fold(f64::INFINITY, f64::min);
    let x1 = f.iter().map(|b| b[2]).fold(f64::NEG_INFINITY, f64::max);
    assert!(
        x0 < cam.center_x && x1 > cam.center_x,
        "o modelo nasce à volta do centro da vista"
    );
}

/// Os controlos do nó só se pintam quando fazem alguma coisa; recolher e a direcção pelo botão.
#[test]
fn the_mind_controls_show_only_when_they_act() {
    let mut t = T::new();
    let root = t.start_map("Plan");
    assert!(t.key(BoardKey::Tab, NONE, None));
    t.write("Leaf");
    assert!(t.key(BoardKey::Escape, NONE, None));
    t.paint();
    assert!(
        t.find(node(mind(MindItem::Collapse))).is_none(),
        "uma folha não recolhe"
    );
    assert!(t.find(node(mind(MindItem::Auto))).is_none());
    assert!(t.find(node(mind(MindItem::Vertical))).is_some());
    t.select(root);
    t.shell_click_bar(mind(MindItem::Collapse));
    assert!(t.doc().get(root).unwrap().mind().unwrap().collapsed);
    t.shell_click_bar(mind(MindItem::Vertical));
    assert!(t.doc().get(root).unwrap().mind().unwrap().vertical);
}
