//! Os modelos (T1–T6, `docs/MiroClone/04_planear_mapa_e_modelos.md` §2): cada um entra no CENTRO da
//! vista, seleccionado, num passo de desfazer, com o que o modelo do Miro traz.

use ph2d_board_model::{End, Rgba};

use super::*;
use crate::{Command, Metrics};
use ph2d_text::TextSystem;

struct W {
    ed: Editor,
    doc: BoardDoc,
    h: History,
    ts: TextSystem,
}

fn w() -> W {
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
    W {
        ed: Editor::new(Style::new(None, Some(ink), ink), metrics),
        doc: BoardDoc::default(),
        h: History::default(),
        ts: TextSystem::without_system_fonts(),
    }
}

fn tr(k: &'static str) -> String {
    format!("«{k}»")
}

const CENTER: [f64; 2] = [1_000.0, -500.0];

impl W {
    fn insert(&mut self, t: Template) -> Vec<ElementId> {
        let ids = self
            .ed
            .insert_template(&mut self.doc, &mut self.h, t, CENTER, &tr);
        // O moldador acerta os nós do mapa no próximo desenho.
        let _ = self.ed.overlay(&mut self.doc, &mut self.ts);
        ids
    }

    fn bbox(&self) -> [f64; 4] {
        self.doc
            .live()
            .filter(|e| e.connector().is_none())
            .map(Element::aabb)
            .fold(
                [
                    f64::INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                ],
                |a, b| {
                    [
                        a[0].min(b[0]),
                        a[1].min(b[1]),
                        a[2].max(b[2]),
                        a[3].max(b[3]),
                    ]
                },
            )
    }

    fn count(&self, f: impl Fn(&Element) -> bool) -> usize {
        self.doc.live().filter(|e| f(e)).count()
    }
}

/// Cada modelo: centrado, seleccionado, UM passo de desfazer, e o que vive num frame está DENTRO dele.
#[test]
fn every_template_lands_centred_selected_and_in_one_undo_step() {
    for t in Template::ALL {
        let mut w = w();
        let picked = w.insert(t);
        assert!(!picked.is_empty(), "{t:?}: nada seleccionado");
        assert_eq!(w.ed.selection().len(), picked.len());
        let b = w.bbox();
        let c = [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0];
        // O mapa arruma-se antes de centrar com as larguras ESTIMADAS do texto; o 1.º desenho acerta-as
        // ao moldado e os ramos andam o que a estimativa errou (meia largura de um nó, no máximo).
        let tol = if t == Template::MindMap { 40.0 } else { 1e-6 };
        assert!(
            (c[0] - CENTER[0]).abs() < tol && (c[1] - CENTER[1]).abs() < tol,
            "{t:?}: centro {c:?}"
        );
        for el in w.doc.live() {
            if let Some(f) = el.frame {
                let fr = w.doc.get(f).expect("o frame existe");
                assert!(
                    fr.box_contains_all_of(el),
                    "{t:?}: {:?} sai do frame",
                    el.id
                );
            }
        }
        assert!(w.cmd_undo(), "{t:?}");
        assert_eq!(w.doc.live_len(), 0, "{t:?}: UM passo de desfazer");
    }
}

impl W {
    fn cmd_undo(&mut self) -> bool {
        self.ed
            .command(&mut self.doc, &mut self.h, &mut self.ts, Command::Undo)
    }
}

/// T1: quatro frames (Start, Stop, Continue, Action items), uma pergunta e quatro notas em cada.
#[test]
fn the_retrospective_has_four_columns_with_a_question_and_notes() {
    let mut w = w();
    w.insert(Template::Retrospective);
    let names: Vec<String> = w
        .doc
        .frames_in_panel_order()
        .iter()
        .map(|f| f.as_frame().unwrap().name.clone())
        .collect();
    for k in ["start", "stop", "continue", "actions"] {
        assert!(names.iter().any(|n| n.contains(k)), "{k} em {names:?}");
    }
    assert_eq!(w.count(|e| e.shape().is_some_and(|s| s.kind.is_note())), 16);
}

/// T3: três colunas; o Backlog com dois cartões (as duas raias), o «In progress» com um.
#[test]
fn the_kanban_has_three_columns_and_two_lanes() {
    let mut w = w();
    w.insert(Template::Kanban);
    assert_eq!(w.doc.frames().count(), 3);
    assert_eq!(
        w.count(|e| e.shape().is_some_and(|s| s.kind == ShapeType::StickyWide)),
        3
    );
}

/// T4: um mapa mental — a raiz, quatro tópicos (dois de cada lado) e dois subtópicos, arrumados.
#[test]
fn the_mind_map_template_is_a_laid_out_map() {
    let mut w = w();
    let picked = w.insert(Template::MindMap);
    let root = picked[0];
    assert_eq!(w.doc.mind_root(root), Some(root));
    let kids = w.doc.mind_children(root);
    assert_eq!(kids.len(), 4);
    assert_eq!(w.doc.mind_descendants(root).len(), 6);
    let r = w.doc.get(root).unwrap().clone();
    let right = kids
        .iter()
        .filter(|k| w.doc.get(**k).unwrap().x > r.x + r.w)
        .count();
    assert_eq!(right, 2);
}

/// T5: seis formas e seis setas presas nas duas pontas, duas com rótulo (Yes/No).
#[test]
fn the_flowchart_is_bound_arrows_between_shapes() {
    let mut w = w();
    w.insert(Template::Flowchart);
    let arrows: Vec<&Element> = w.doc.live().filter(|e| e.connector().is_some()).collect();
    assert_eq!(arrows.len(), 6);
    for a in &arrows {
        let c = a.connector().unwrap();
        assert!(matches!(c.start, End::Bound { .. }) && matches!(c.end, End::Bound { .. }));
    }
    assert_eq!(
        arrows
            .iter()
            .filter(|a| !a.connector().unwrap().label.is_empty())
            .count(),
        2
    );
}

/// T6: a persona, quatro estágios × cinco linhas de notas.
#[test]
fn the_journey_map_is_stages_by_rows() {
    let mut w = w();
    w.insert(Template::Journey);
    assert_eq!(
        w.count(|e| e.shape().is_some_and(|s| s.kind == ShapeType::Sticky)),
        1 + 4 * 5
    );
}
