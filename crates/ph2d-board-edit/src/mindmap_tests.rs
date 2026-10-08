//! O mapa mental pelo caminho de quem o usa (W6, `docs/MiroClone/04_planear_mapa_e_modelos.md` §2):
//! a ferramenta, `Tab`, `Enter`, as setas, apagar, recolher, o «+» e largar — com o moldador de
//! texto REAL (o passo de 54 do Miro depende da altura da linha).
//!
//! Números transcritos: Miro (imagem oficial, 04 §1.3) — passo 54 entre nós de uma linha; o
//! mind-elixir e o simple-mind-map (oráculo `ferramentas/mindmap_oracle/`, `teclas_*`) — `Tab` = o
//! ÚLTIMO filho, seleccionado e em edição; `Enter` = o irmão LOGO DEPOIS; `Delete` apaga o ramo (o ME
//! passa a selecção ao pai). ⛔ Divergências escolhidas: setas ↑/↓ atravessam para o primo à mesma
//! profundidade (o ME pára no último irmão) e para fora dão o filho do MEIO (o ME dá o 1.º); `Tab`
//! na raiz enche o lado mais leve (o ME alterna, o SMM conta).

use std::collections::BTreeSet;

use ph2d_board_model::{MIND_COLORS, Rgba, Side};

use super::*;
use crate::{Command, Metrics, Mods, Pointer, Style, TextKey, Tool};

const VIEW: [f64; 4] = [-10_000.0, -10_000.0, 10_000.0, 10_000.0];
const NONE: Mods = Mods {
    shift: false,
    ctrl: false,
    alt: false,
};
const CTRL: Mods = Mods {
    shift: false,
    ctrl: true,
    alt: false,
};

struct W {
    ed: Editor,
    doc: BoardDoc,
    h: History,
    ts: TextSystem,
}

fn at(p: [f64; 2], mods: Mods) -> Pointer {
    Pointer {
        world: p,
        px: 1.0,
        mods,
    }
}

impl W {
    /// Um mapa novo com a raiz «Plan» em (0, 0).
    fn new() -> (Self, ElementId) {
        let ink = Rgba([20, 20, 20, 255]);
        let metrics = Metrics {
            handle: 8.0,
            rotate_offset: 24.0,
            snap: 8.0,
            drag: 3.0,
            click_size: [160.0, 100.0],
            paste_offset: 10.0,
            bind: 8.0,
            dot: 40.0,
            eraser: 10.0,
            frame_title: 20.0,
        };
        let mut w = Self {
            ed: Editor::new(Style::new(None, Some(ink), ink), metrics),
            doc: BoardDoc::default(),
            h: History::default(),
            ts: TextSystem::without_system_fonts(),
        };
        w.ed.tool = Tool::MindMap;
        w.click([0.0, 0.0], NONE);
        w.write("Plan");
        let root = w.focus();
        (w, root)
    }

    fn focus(&self) -> ElementId {
        self.ed.mind_focus(&self.doc).expect("um nó em foco")
    }

    fn write(&mut self, s: &str) {
        assert!(
            self.ed.text_input(&mut self.doc, &mut self.ts, s),
            "a escrever"
        );
    }

    fn cmd(&mut self, c: Command) -> bool {
        self.ed.command(&mut self.doc, &mut self.h, &mut self.ts, c)
    }

    /// `Tab` e escrever: o filho novo do nó em foco.
    fn child(&mut self, text: &str) -> ElementId {
        assert!(self.cmd(Command::MindChild));
        self.write(text);
        self.focus()
    }

    fn sibling(&mut self, text: &str) -> ElementId {
        assert!(self.cmd(Command::MindSibling));
        self.write(text);
        self.focus()
    }

    fn select(&mut self, id: ElementId) {
        self.ed.commit_text(&mut self.doc, &mut self.h);
        self.ed.select(&self.doc, [id]);
    }

    fn el(&self, id: ElementId) -> &Element {
        self.doc.get(id).expect("vivo")
    }

    fn cy(&self, id: ElementId) -> f64 {
        self.el(id).center()[1]
    }

    fn drag(&mut self, from: [f64; 2], to: [f64; 2], m: Mods) {
        self.ed
            .pointer_down(&mut self.doc, &mut self.h, &mut self.ts, at(from, m), VIEW);
        let mid = [(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0];
        self.ed
            .pointer_move(&mut self.doc, &mut self.ts, at(mid, m));
        self.ed.pointer_move(&mut self.doc, &mut self.ts, at(to, m));
        self.ed.pointer_up(&mut self.doc, &mut self.h, at(to, m));
    }

    fn click(&mut self, p: [f64; 2], m: Mods) {
        self.drag(p, p, m);
    }

    fn sel(&self) -> BTreeSet<ElementId> {
        self.ed.selection().clone()
    }
}

/// ⭐ M2 com o moldador real: três irmãos de uma linha a 54 (o Miro) — 53,94 com a nossa linha.
#[test]
fn single_line_siblings_step_fifty_four() {
    let (mut w, _) = W::new();
    let a = w.child("Alpha");
    let b = w.sibling("Beta");
    let c = w.sibling("Gamma");
    for (x, y) in [(a, b), (b, c)] {
        let step = w.cy(y) - w.cy(x);
        assert!((step - 54.0).abs() < 0.1, "passo {step}, o Miro 54");
    }
}

/// M6: `Tab` cria o ÚLTIMO filho, seleccionado e a escrever.
#[test]
fn tab_adds_the_last_child_and_edits_it() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    w.select(a);
    let a2 = w.child("A2");
    assert_eq!(w.doc.mind_children(a), vec![a1, a2]);
    assert_eq!(w.sel(), BTreeSet::from([a2]));
    assert!(w.ed.is_editing_text());
    assert_eq!(w.doc.mind_parent(a), Some(root));
}

/// M7: `Enter` a escrever cria o irmão LOGO DEPOIS (no meio, se há outro a seguir); na raiz, um filho.
#[test]
fn enter_adds_the_next_sibling_right_after() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let c = w.sibling("C");
    w.select(a);
    assert!(w.cmd(Command::EditText));
    let b = w.sibling("B");
    assert_eq!(w.doc.mind_children(root), vec![a, b, c]);
    assert!(
        w.cy(a) < w.cy(b) && w.cy(b) < w.cy(c),
        "o do meio fica no meio"
    );
    w.select(root);
    assert!(w.cmd(Command::EditText));
    let d = w.sibling("D");
    assert_eq!(
        w.doc.mind_parent(d),
        Some(root),
        "na raiz, Enter dá um filho"
    );
}

/// M15: o nó cresce de LADO com o texto; a linha nova (`Shift+Enter` → `Newline`) cresce para baixo.
#[test]
fn a_node_grows_in_width_with_its_text() {
    let (mut w, _) = W::new();
    let a = w.child("A");
    let (w1, h1) = (w.el(a).w, w.el(a).h);
    w.write(" much longer text");
    assert!(w.el(a).w > w1 + 50.0);
    assert!(
        (w.el(a).h - h1).abs() < 1e-9,
        "uma linha continua uma linha"
    );
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::Newline);
    w.write("two");
    assert!(w.el(a).h > h1 + 10.0);
    let gap =
        w.el(a).x - (w.el(w.doc.mind_parent(a).unwrap()).x + w.el(w.doc.mind_parent(a).unwrap()).w);
    assert!(
        (gap - ph2d_board_model::MIND_LEVEL_GAP).abs() < 1e-6,
        "o layout segue o tamanho"
    );
}

/// M5: `Tab` na raiz enche o lado mais leve; empate à direita.
#[test]
fn tab_on_the_root_fills_the_lighter_side() {
    let (mut w, root) = W::new();
    let side = |w: &W, id| w.el(id).mind().unwrap().side;
    let a = w.child("A");
    assert_eq!(side(&w, a), Side::After, "empate: à direita");
    w.child("A1");
    w.select(a);
    w.child("A2");
    w.select(root);
    let b = w.child("B");
    assert_eq!(side(&w, b), Side::Before);
    w.select(root);
    let c = w.child("C");
    assert_eq!(
        side(&w, c),
        Side::Before,
        "A com dois filhos pesa mais que B"
    );
    w.select(root);
    let d = w.child("D");
    assert_eq!(side(&w, d), Side::After);
}

/// M14: cada filho novo da raiz toma a cor seguinte; os de baixo herdam a do pai.
#[test]
fn a_new_branch_takes_the_next_colour_and_children_inherit() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    w.select(root);
    let b = w.child("B");
    let col = |w: &W, id| w.el(id).mind().unwrap().color;
    assert_eq!(col(&w, a), Rgba(MIND_COLORS[0]));
    assert_eq!(col(&w, b), Rgba(MIND_COLORS[1]));
    assert_eq!(col(&w, a1), col(&w, a));
}

/// ⭐ M8: as setas andam pela árvore.
#[test]
fn arrows_walk_the_tree() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    let a2 = w.sibling("A2");
    let a3 = w.sibling("A3");
    w.select(root);
    let c = w.child("C");
    w.select(root);
    assert!(
        w.ed.mind_child(&mut w.doc, &mut w.h, &mut w.ts, Some(Side::After)),
        "o «+» da direita"
    );
    w.write("B");
    let b = w.focus();
    let b1 = w.child("B1");
    assert_eq!(w.el(c).mind().unwrap().side, Side::Before);
    w.select(root);
    let go = |w: &mut W, d: [f64; 2]| {
        assert!(w.cmd(Command::MindNav(d)));
        *w.sel().iter().next().unwrap()
    };
    assert_eq!(
        go(&mut w, [1.0, 0.0]),
        a,
        "da raiz para a direita: o filho do meio desse lado"
    );
    assert_eq!(go(&mut w, [1.0, 0.0]), a2, "para fora: o do meio");
    assert_eq!(go(&mut w, [0.0, 1.0]), a3);
    assert_eq!(
        go(&mut w, [0.0, 1.0]),
        b1,
        "atravessa para o primo à mesma profundidade"
    );
    assert_eq!(go(&mut w, [0.0, 1.0]), b1, "no fim fica");
    assert_eq!(go(&mut w, [-1.0, 0.0]), b, "para dentro: o pai");
    assert_eq!(go(&mut w, [-1.0, 0.0]), root);
    assert_eq!(go(&mut w, [-1.0, 0.0]), c, "o lado esquerdo");
    assert_eq!(
        go(&mut w, [1.0, 0.0]),
        root,
        "do lado esquerdo, para a direita é para dentro"
    );
    let _ = a1;
}

/// M9: `Delete` apaga o ramo inteiro e selecciona o pai; desfazer devolve tudo, no sítio.
#[test]
fn delete_takes_the_whole_branch_and_selects_the_parent() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    let a2 = w.sibling("A2");
    w.select(root);
    let b = w.child("B");
    let by = w.cy(b);
    w.select(a);
    assert!(w.cmd(Command::Delete));
    for id in [a, a1, a2] {
        assert!(w.doc.get(id).is_none());
    }
    assert_eq!(w.sel(), BTreeSet::from([root]));
    assert!(
        (w.cy(b) - w.cy(root)).abs() < 1e-9,
        "B sozinho do seu lado... "
    );
    assert!(w.cmd(Command::Undo));
    assert!(w.doc.get(a2).is_some());
    let _ = by;
}

/// ⭐ M10: recolher esconde o ramo (e tira-o da selecção) sem mexer em ninguém; o selo expande.
#[test]
fn collapsing_hides_and_the_badge_expands() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    w.sibling("A2");
    w.select(root);
    let b = w.child("B");
    let by = w.cy(b);
    w.select(a);
    assert!(w.cmd(Command::MindCollapse));
    assert!(w.doc.live().all(|e| e.id != a1), "o ramo some do desenho");
    assert!((w.cy(b) - by).abs() < 1e-9, "ninguém se mexe");
    let n = ph2d_board_model::hidden_count(&w.doc, a);
    assert_eq!(n, 2);
    let [x, y, bw, bh] = ph2d_board_model::badge_rect(&w.doc, a, digits(n)).unwrap();
    w.ed.select(&w.doc, []);
    w.click([x + bw / 2.0, y + bh / 2.0], NONE);
    assert!(w.doc.live().any(|e| e.id == a1), "o selo expande");
}

/// M5: o «+» de um lado da raiz cria o filho desse lado.
#[test]
fn the_plus_on_a_side_of_the_root_adds_there() {
    let (mut w, root) = W::new();
    w.select(root);
    let (p, d, s) =
        w.ed.mind_plus(&w.doc)
            .into_iter()
            .find(|(_, _, s)| *s == Side::Before)
            .expect("a raiz tem os dois «+»");
    assert_eq!(s, Side::Before);
    w.click([p[0] + d[0] * 40.0, p[1] + d[1] * 40.0], NONE);
    let n = w.focus();
    assert_eq!(w.doc.mind_parent(n), Some(root));
    assert_eq!(w.el(n).mind().unwrap().side, Side::Before);
    assert!(w.el(n).x < w.el(root).x);
}

/// ⭐ M12: largar um nó sobre outro faz dele o último filho (com a cor do ramo de lá).
#[test]
fn dropping_a_node_on_another_reparents_it() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    w.select(root);
    w.select(root);
    let b = w.child("Bee");
    let b1 = w.child("B1");
    w.select(b1);
    let from = w.el(b1).center();
    let to = w.el(a).center();
    w.drag(from, to, NONE);
    assert_eq!(w.doc.mind_parent(b1), Some(a));
    assert_eq!(w.doc.mind_children(a), vec![a1, b1]);
    assert_eq!(
        w.el(b1).mind().unwrap().color,
        w.el(a).mind().unwrap().color
    );
    assert!(w.doc.mind_children(b).is_empty());
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.mind_parent(b1), Some(b), "UM passo de desfazer");
}

/// M12: largar no vazio reordena pela posição; as guias calam-se num nó arrumado.
#[test]
fn dropping_beside_siblings_reorders_without_guides() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    let a2 = w.sibling("A2");
    let a3 = w.sibling("A3");
    w.select(a3);
    let from = w.el(a3).center();
    let to = [from[0], w.el(a1).y - 40.0];
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(from, NONE), VIEW);
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at([from[0], from[1] - 20.0], NONE));
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(to, NONE));
    assert!(
        w.ed.overlay(&mut w.doc, &mut w.ts).guides.is_empty(),
        "as guias calam-se"
    );
    w.ed.pointer_up(&mut w.doc, &mut w.h, at(to, NONE));
    assert_eq!(w.doc.mind_children(a), vec![a3, a1, a2]);
    assert!(w.cy(a3) < w.cy(a1));
    let _ = root;
}

/// M12: um filho da raiz largado do outro lado dela muda de lado; com `Ctrl` nunca muda de pai.
#[test]
fn dropping_across_the_root_switches_side_and_ctrl_never_reparents() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    w.select(root);
    let b = w.child("B");
    let from = w.el(a).center();
    let to = [w.el(root).x - 150.0, from[1]];
    w.select(a);
    w.drag(from, to, NONE);
    assert_eq!(w.el(a).mind().unwrap().side, Side::Before);
    assert!(w.el(a).x < w.el(root).x);
    let from = w.el(a).center();
    let onto = w.el(b).center();
    w.select(a);
    w.drag(from, onto, CTRL);
    assert_eq!(w.doc.mind_parent(a), Some(root), "Ctrl só reordena");
}

/// Copiar um ramo e colar: a cópia é irmã do original e leva os filhos (debaixo dela).
#[test]
fn a_pasted_branch_is_a_sibling_with_its_own_children() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    w.select(a);
    assert!(w.cmd(Command::Copy));
    assert!(w.cmd(Command::Paste));
    let kids = w.doc.mind_children(root);
    assert_eq!(kids.len(), 2);
    let copy = kids[1];
    assert_ne!(copy, a);
    let ck = w.doc.mind_children(copy);
    assert_eq!(ck.len(), 1);
    assert_ne!(ck[0], a1);
    assert!(
        (w.el(copy).x - w.el(a).x).abs() < 1e-9,
        "o layout põe-na na coluna dos irmãos"
    );
}

/// A fronteira do `Tab` (W3 × W6): numa NOTA, «o filho» não existe — o `Tab` dela é a nota seguinte.
#[test]
fn a_note_has_no_mind_child() {
    let (mut w, _) = W::new();
    w.ed.commit_text(&mut w.doc, &mut w.h);
    w.ed.tool = Tool::Shape(ShapeType::Sticky);
    w.click([2_000.0, 2_000.0], NONE);
    assert!(!w.cmd(Command::MindChild));
    assert!(!w.cmd(Command::MindNav([1.0, 0.0])));
}
