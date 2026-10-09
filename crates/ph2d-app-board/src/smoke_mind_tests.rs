//! ⭐ A cena 8 ENSINA o que acontece (a lição da W5b: uma cena que promete o que não faz é pior que
//! nenhuma). Cada nota é conduzida pelo editor real com as medidas da interface
//! (`board_view::metrics`), a zoom 1 e a 0,8: o que a nota diz é o que o editor faz, e os nomes que
//! ela cita são os dos nós que existem.

use super::*;
use ph2d_board_edit::{Command, Mods, Pointer, Template, TextKey};
use ph2d_board_model::{MIND_COLORS, Rgba};

const VIEW: [f64; 4] = [-1e4, -1e4, 1e4, 1e4];
/// Px de ecrã por unidade do mundo: zoom 1 e 0,8.
const PXS: [f64; 2] = [1.0, 1.25];

struct Run {
    ed: Editor,
    doc: BoardDoc,
    h: History,
    ts: TextSystem,
    l: Lessons,
    px: f64,
}

fn run(px: f64) -> Run {
    let mut doc = BoardDoc::default();
    let l = build(&mut doc);
    Run {
        ed: Editor::new(default_style(), metrics()),
        doc,
        h: History::default(),
        ts: TextSystem::without_system_fonts(),
        l,
        px,
    }
}

impl Run {
    fn at(&self, p: [f64; 2]) -> Pointer {
        Pointer {
            world: p,
            px: self.px,
            mods: Mods::default(),
        }
    }

    fn cmd(&mut self, c: Command) -> bool {
        self.ed.command(&mut self.doc, &mut self.h, &mut self.ts, c)
    }

    fn select(&mut self, id: ElementId) {
        self.ed.commit_text(&mut self.doc, &mut self.h);
        self.ed.select(&self.doc, [id]);
    }

    fn focus(&self) -> ElementId {
        self.ed.mind_focus(&self.doc).expect("um nó em foco")
    }

    fn text(&self, id: ElementId) -> String {
        self.doc
            .get(id)
            .unwrap()
            .shape()
            .unwrap()
            .text
            .as_str()
            .to_owned()
    }

    /// Arrasta de `a` a `b` em passos de 4 px de ecrã.
    fn drag(&mut self, a: [f64; 2], b: [f64; 2]) {
        let p = self.at(a);
        self.ed
            .pointer_down(&mut self.doc, &mut self.h, &mut self.ts, p, VIEW);
        let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / (4.0 * self.px))
            .ceil()
            .max(2.0) as usize;
        for k in 1..=n {
            let t = k as f64 / n as f64;
            let p = self.at([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]);
            self.ed.pointer_move(&mut self.doc, &mut self.ts, p);
        }
        let p = self.at(b);
        self.ed.pointer_up(&mut self.doc, &mut self.h, p);
    }
}

#[test]
fn every_mind_lesson_does_what_it_says() {
    for px in PXS {
        let mut r = run(px);
        let l = &r.l;
        let (root, ideas, sketch, risks, next) = (l.root, l.ideas, l.sketch, l.risks, l.next);
        let notes = l.notes;
        let name = |r: &Run, id| r.text(id);
        // O que a cena mostra ao abrir: o número dos escondidos ao lado de «Risks».
        assert_eq!(ph2d_board_model::hidden_count(&r.doc, risks), 3);
        assert!(r.doc.get(risks).unwrap().mind().unwrap().collapsed);

        // 1. «Ideas» + Tab: nasce um filho, a escrever.
        assert!(
            r.text(notes[0]).contains(&name(&r, ideas)),
            "a nota 1 cita o nó"
        );
        r.select(ideas);
        assert!(r.cmd(Command::MindChild));
        let born = r.focus();
        assert_eq!(r.doc.mind_parent(born), Some(ideas));
        assert!(r.ed.is_editing_text());
        assert!(r.ed.text_input(&mut r.doc, &mut r.ts, "x"));

        // 2. A escrever, Enter: o irmão logo abaixo; Shift+Enter: uma linha nova.
        assert!(r.cmd(Command::MindSibling));
        let sib = r.focus();
        let kids = r.doc.mind_children(ideas);
        let i = kids.iter().position(|c| *c == born).unwrap();
        assert_eq!(kids.get(i + 1), Some(&sib), "logo depois");
        assert!(
            r.doc.get(sib).unwrap().y > r.doc.get(born).unwrap().y,
            "abaixo"
        );
        r.ed.text_key(&mut r.doc, &mut r.h, &mut r.ts, TextKey::Newline);
        assert!(r.text(sib).contains('\n'));

        // 3. As setas andam pela árvore; Delete leva o ramo (e selecciona o pai).
        r.select(sib);
        assert!(r.cmd(Command::Delete));
        assert!(r.doc.get(sib).is_none());
        assert_eq!(r.focus(), ideas);
        r.select(sketch);
        assert!(r.cmd(Command::MindNav([0.0, 1.0])));
        assert_ne!(r.focus(), sketch, "a seta andou");
        assert!(r.cmd(Command::MindNav([-1.0, 0.0])));
        assert_eq!(r.focus(), ideas);
        assert!(r.cmd(Command::MindNav([-1.0, 0.0])));
        assert_eq!(r.focus(), root);

        // 4. O número ao lado de «Risks» abre o ramo.
        assert!(r.text(notes[3]).contains(&name(&r, risks)));
        let n = ph2d_board_model::hidden_count(&r.doc, risks);
        let [x, y, w, h] =
            ph2d_board_model::badge_rect(&r.doc, risks, ph2d_board_edit::digits(n)).unwrap();
        // A ordem REAL das lições: «Plan» continua seleccionado da 3 (auditoria W6: o Ctrl+Up
        // fechava o mapa inteiro).
        r.drag([x + w / 2.0, y + h / 2.0], [x + w / 2.0, y + h / 2.0]);
        assert!(
            !r.doc.get(risks).unwrap().mind().unwrap().collapsed,
            "o ramo abriu"
        );
        assert!(r.cmd(Command::MindCollapse), "Ctrl+Up");
        assert!(
            r.doc.get(risks).unwrap().mind().unwrap().collapsed,
            "volta a fechar «Risks»"
        );
        assert!(
            !r.doc.get(root).unwrap().mind().unwrap().collapsed,
            "e não o mapa"
        );

        // 5. «Sketch» largado sobre «Next steps»: passa para lá, com a cor de lá.
        assert!(r.text(notes[4]).contains(&name(&r, sketch)));
        assert!(r.text(notes[4]).contains(&name(&r, next)));
        r.select(sketch);
        let (a, b) = (
            r.doc.get(sketch).unwrap().center(),
            r.doc.get(next).unwrap().center(),
        );
        r.drag(a, b);
        assert_eq!(r.doc.mind_parent(sketch), Some(next), "a {px} px");
        let col = r.doc.get(next).unwrap().mind().unwrap().color;
        assert_eq!(r.doc.get(sketch).unwrap().mind().unwrap().color, col);
        assert!(MIND_COLORS.iter().any(|c| Rgba(*c) == col));

        // 6. Um modelo da galeria: um quadro inteiro de uma vez (a retrospectiva: quatro frames).
        let before = r.doc.frames().count();
        r.ed.insert_template(
            &mut r.doc,
            &mut r.h,
            Template::Retrospective,
            [0.0, 2_000.0],
            &|k| ph2d_i18n::tr(k).to_owned(),
        );
        assert_eq!(r.doc.frames().count(), before + 4);
    }
}

/// As notas das lições não tapam o mapa (ficam inteiras abaixo dele) e o mapa abre arrumado.
#[test]
fn the_lessons_sit_below_the_map() {
    let r = run(1.0);
    let map: Vec<ElementId> = std::iter::once(r.l.root)
        .chain(r.doc.mind_descendants(r.l.root))
        .collect();
    let bottom = map
        .iter()
        .filter_map(|id| r.doc.get(*id))
        .map(|e| e.y + e.h)
        .fold(f64::NEG_INFINITY, f64::max);
    for n in r.l.notes {
        assert!(r.doc.get(n).unwrap().y > bottom);
    }
    let layout = ph2d_board_model::mind_layout(&r.doc, r.l.root);
    for (id, [x, y]) in layout {
        let e = r.doc.get(id).unwrap();
        assert!(
            (e.x - x).abs() < 1e-9 && (e.y - y).abs() < 1e-9,
            "o mapa abre arrumado"
        );
    }
}
