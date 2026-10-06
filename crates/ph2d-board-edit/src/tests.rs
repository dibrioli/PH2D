//! O editor pelo caminho de quem o usa: carregar, arrastar, largar e teclas — e o desfazer de cada
//! gesto como UM passo.

use super::*;
use ph2d_board_model::{ElementId, Rgba};

const PX: f64 = 1.0;
const VIEW: [f64; 4] = [-10_000.0, -10_000.0, 10_000.0, 10_000.0];

fn metrics() -> Metrics {
    Metrics {
        handle: 8.0,
        rotate_offset: 24.0,
        snap: 6.0,
        drag: 3.0,
        click_size: [160.0, 100.0],
        paste_offset: 10.0,
    }
}

struct World {
    ed: Editor,
    doc: BoardDoc,
    h: History,
    ts: TextSystem,
}

fn world() -> World {
    let ink = Rgba([20, 20, 20, 255]);
    World {
        ed: Editor::new(
            Style::new(Some(Rgba([255, 255, 255, 255])), Some(ink), ink),
            metrics(),
        ),
        doc: BoardDoc::default(),
        h: History::default(),
        ts: TextSystem::without_system_fonts(),
    }
}

fn at(x: f64, y: f64, mods: Mods) -> Pointer {
    Pointer {
        world: [x, y],
        px: PX,
        mods,
    }
}

const NONE: Mods = Mods {
    shift: false,
    ctrl: false,
    alt: false,
};
const SHIFT: Mods = Mods {
    shift: true,
    ctrl: false,
    alt: false,
};
const ALT: Mods = Mods {
    shift: false,
    ctrl: false,
    alt: true,
};
const CTRL: Mods = Mods {
    shift: false,
    ctrl: true,
    alt: false,
};

impl World {
    fn drag(&mut self, from: [f64; 2], to: [f64; 2], m: Mods) {
        self.ed.pointer_down(
            &mut self.doc,
            &mut self.h,
            &mut self.ts,
            at(from[0], from[1], m),
            VIEW,
        );
        // Dois passos: o primeiro passa o limiar, o segundo chega ao sítio.
        let mid = [(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0];
        self.ed
            .pointer_move(&mut self.doc, &mut self.ts, at(mid[0], mid[1], m));
        self.ed
            .pointer_move(&mut self.doc, &mut self.ts, at(to[0], to[1], m));
        self.ed
            .pointer_up(&mut self.doc, &mut self.h, at(to[0], to[1], m));
    }

    fn click(&mut self, p: [f64; 2], m: Mods) {
        self.drag(p, p, m);
    }

    fn rect(&mut self, bx: [f64; 4]) -> ElementId {
        self.ed.tool = Tool::Shape(ShapeType::Rectangle);
        self.drag([bx[0], bx[1]], [bx[0] + bx[2], bx[1] + bx[3]], CTRL);
        *self
            .ed
            .selection()
            .iter()
            .next()
            .expect("nasceu seleccionada")
    }

    fn el(&self, id: ElementId) -> &Element {
        self.doc.get(id).expect("existe")
    }

    fn cmd(&mut self, c: Command) -> bool {
        self.ed.command(&mut self.doc, &mut self.h, &mut self.ts, c)
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

#[test]
fn dragging_a_shape_tool_creates_it_selected_and_back_to_select_and_undo_removes_it() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 100.0, 50.0]);
    let el = w.el(id);
    assert_eq!([el.x, el.y, el.w, el.h], [0.0, 0.0, 100.0, 50.0]);
    assert_eq!(
        w.ed.tool,
        Tool::Select,
        "a ferramenta não voltou à selecção"
    );
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.live_len(), 0);
    assert!(
        w.ed.selection().is_empty(),
        "a selecção aponta para o que já não existe"
    );
    assert!(w.cmd(Command::Redo));
    assert_eq!(w.doc.live_len(), 1);
}

#[test]
fn a_click_with_a_shape_tool_creates_the_default_size_centred_on_it() {
    let mut w = world();
    w.ed.tool = Tool::Shape(ShapeType::Ellipse);
    w.click([500.0, 500.0], NONE);
    let el = w.doc.live_in_z_order()[0].clone();
    assert_eq!([el.x, el.y, el.w, el.h], [420.0, 450.0, 160.0, 100.0]);
    assert_eq!(el.shape().unwrap().kind, ShapeType::Ellipse);
}

#[test]
fn shift_squares_and_alt_centres_a_new_shape() {
    let mut w = world();
    w.ed.tool = Tool::Shape(ShapeType::Rectangle);
    w.drag(
        [0.0, 0.0],
        [100.0, 40.0],
        Mods {
            ctrl: true,
            ..SHIFT
        },
    );
    let a = w.doc.live_in_z_order()[0].clone();
    assert_eq!([a.w, a.h], [100.0, 100.0]);
    w.ed.tool = Tool::Shape(ShapeType::Rectangle);
    w.drag(
        [1000.0, 1000.0],
        [1050.0, 1020.0],
        Mods { ctrl: true, ..ALT },
    );
    let b = w.doc.live_in_z_order()[1].clone();
    assert_eq!([b.x, b.y, b.w, b.h], [950.0, 980.0, 100.0, 40.0]);
}

#[test]
fn moving_is_one_undo_step_and_a_click_does_not_move() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 100.0, 50.0]);
    w.drag([50.0, 25.0], [150.0, 225.0], CTRL);
    assert_eq!([w.el(id).x, w.el(id).y], [100.0, 200.0]);
    w.click([150.0, 225.0], CTRL);
    assert_eq!([w.el(id).x, w.el(id).y], [100.0, 200.0], "um clique moveu");
    assert!(w.cmd(Command::Undo));
    assert_eq!(
        [w.el(id).x, w.el(id).y],
        [0.0, 0.0],
        "o arrasto desfez-se aos bocados"
    );
}

#[test]
fn shift_locks_the_move_to_the_main_axis() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 100.0, 50.0]);
    // O Shift entra DEPOIS de carregar: carregar com ele num seleccionado tira-o da selecção.
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(50.0, 25.0, CTRL), VIEW);
    let lock = Mods {
        ctrl: true,
        ..SHIFT
    };
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(150.0, 40.0, lock));
    w.ed.pointer_up(&mut w.doc, &mut w.h, at(150.0, 40.0, lock));
    assert_eq!([w.el(id).x, w.el(id).y], [100.0, 0.0]);
}

#[test]
fn alt_drag_leaves_the_original_and_moves_a_copy_and_undo_drops_the_copy() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 100.0, 50.0]);
    w.drag([50.0, 25.0], [250.0, 25.0], Mods { ctrl: true, ..ALT });
    assert_eq!(w.doc.live_len(), 2);
    assert_eq!(w.el(id).x, 0.0, "o original saiu do sítio");
    let copy = *w.ed.selection().iter().next().unwrap();
    assert_ne!(copy, id);
    assert_eq!(w.el(copy).x, 200.0);
    assert!(w.el(copy).z > w.el(id).z, "a cópia nasceu atrás");
    w.cmd(Command::Undo);
    assert_eq!(w.doc.live_len(), 1);
}

#[test]
fn shift_click_toggles_and_the_marquee_takes_only_what_is_fully_inside() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 50.0, 50.0]);
    let b = w.rect([100.0, 0.0, 50.0, 50.0]);
    let c = w.rect([300.0, 0.0, 50.0, 50.0]);
    w.click([25.0, 25.0], NONE);
    w.click([125.0, 25.0], SHIFT);
    assert_eq!(w.ed.selection(), &BTreeSet::from([a, b]));
    w.click([25.0, 25.0], SHIFT);
    assert_eq!(w.ed.selection(), &BTreeSet::from([b]));
    // De (-10,-10) a (200,60): a e b inteiros; c de fora.
    w.drag([-10.0, -10.0], [200.0, 60.0], NONE);
    assert_eq!(w.ed.selection(), &BTreeSet::from([a, b]));
    w.drag([-10.0, -10.0], [320.0, 60.0], NONE);
    assert!(!w.ed.selection().contains(&c), "meio c dentro não conta");
    w.click([200.0, 200.0], NONE);
    assert!(w.ed.selection().is_empty(), "clicar no vazio não limpou");
}

#[test]
fn clicking_one_of_a_multi_selection_without_dragging_keeps_only_it() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 50.0, 50.0]);
    let b = w.rect([100.0, 0.0, 50.0, 50.0]);
    w.cmd(Command::SelectAll);
    w.click([125.0, 25.0], NONE);
    assert_eq!(w.ed.selection(), &BTreeSet::from([b]));
    let _ = a;
}

#[test]
fn the_corner_handle_resizes_keeping_the_opposite_corner_and_shift_keeps_the_ratio() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 100.0, 50.0]);
    w.drag([100.0, 50.0], [200.0, 70.0], CTRL);
    assert_eq!(
        [w.el(id).x, w.el(id).y, w.el(id).w, w.el(id).h],
        [0.0, 0.0, 200.0, 70.0]
    );
    w.drag(
        [200.0, 70.0],
        [400.0, 80.0],
        Mods {
            ctrl: true,
            ..SHIFT
        },
    );
    let e = w.el(id);
    assert!(
        close(e.w / e.h, 200.0 / 70.0),
        "Shift perdeu a proporção: {}×{}",
        e.w,
        e.h
    );
    assert_eq!([e.x, e.y], [0.0, 0.0]);
    w.cmd(Command::Undo);
    assert_eq!(w.el(id).w, 200.0, "um passo por arrasto");
}

#[test]
fn resizing_a_rotated_shape_keeps_its_opposite_corner_still_in_the_world() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 100.0, 50.0]);
    let mut el = w.el(id).clone();
    el.angle = std::f64::consts::FRAC_PI_6;
    w.h.apply(&mut w.doc, vec![BoardOp::Put(el)]);
    let before = w.el(id).corners()[0]; // NW — o oposto da SE
    let se = Frame::of(w.el(id)).handle(Handle::Resize(Dir::SE), PX, &metrics());
    w.drag(se, [se[0] + 30.0, se[1] + 40.0], CTRL);
    let after = w.el(id).corners()[0];
    assert!(
        close(before[0], after[0]) && close(before[1], after[1]),
        "{before:?} → {after:?}"
    );
    assert!(w.el(id).w > 100.0);
}

#[test]
fn alt_resizes_about_the_centre() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 100.0, 50.0]);
    let c = w.el(id).center();
    w.drag([100.0, 25.0], [120.0, 25.0], Mods { ctrl: true, ..ALT });
    assert_eq!(w.el(id).w, 140.0);
    assert_eq!(w.el(id).center(), c);
}

#[test]
fn rotating_follows_the_pointer_and_shift_steps_by_fifteen_degrees() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 100.0, 100.0]);
    let knob = Frame::of(w.el(id)).handle(Handle::Rotate, PX, &metrics());
    // Do topo para a direita do centro (50,50): +90°.
    w.drag(knob, [200.0, 50.0], NONE);
    assert!(
        close(w.el(id).angle, std::f64::consts::FRAC_PI_2),
        "{}",
        w.el(id).angle
    );
    w.cmd(Command::Undo);
    let knob = Frame::of(w.el(id)).handle(Handle::Rotate, PX, &metrics());
    w.drag(knob, [100.0, -10.0], SHIFT);
    let a = w.el(id).angle.to_degrees();
    assert!(
        close(a % 15.0, 0.0) || close(a % 15.0, 15.0),
        "não deu passo de 15°: {a}"
    );
}

#[test]
fn a_box_moved_near_another_snaps_to_its_edge_with_a_guide_and_ctrl_turns_it_off() {
    let mut w = world();
    let _a = w.rect([0.0, 0.0, 100.0, 100.0]);
    let b = w.rect([300.0, 0.0, 50.0, 50.0]);
    // Leva a borda esquerda de b para 103 (3 da borda direita de a, 100): cola em 100.
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(325.0, 25.0, NONE), VIEW);
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(200.0, 225.0, NONE));
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(128.0, 225.0, NONE));
    assert_eq!(w.el(b).x, 100.0);
    assert!(
        !w.ed.overlay(&w.doc, &mut w.ts).guides.is_empty(),
        "colou sem guia"
    );
    w.ed.pointer_up(&mut w.doc, &mut w.h, at(128.0, 225.0, NONE));
    w.drag([125.0, 225.0], [128.0, 425.0], CTRL);
    assert_eq!(w.el(b).x, 103.0, "o Ctrl não desligou as guias");
}

#[test]
fn resizing_several_scales_their_positions_with_the_frame() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 50.0, 50.0]);
    let b = w.rect([150.0, 0.0, 50.0, 50.0]);
    w.cmd(Command::SelectAll);
    // Moldura 200×50; a pega E até 400: tudo ×2 na horizontal.
    w.drag([200.0, 25.0], [400.0, 25.0], CTRL);
    assert_eq!([w.el(a).x, w.el(a).w], [0.0, 100.0]);
    assert_eq!([w.el(b).x, w.el(b).w], [300.0, 100.0]);
}

#[test]
fn double_click_writes_inside_the_text_wraps_the_shape_grows_and_undo_is_one_step() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 120.0, 40.0]);
    assert!(w.ed.double_click(&mut w.doc, &mut w.ts, at(60.0, 20.0, NONE)));
    assert!(w.ed.is_editing_text());
    w.ed.text_input(
        &mut w.doc,
        &mut w.ts,
        "uma ideia comprida que precisa de várias linhas",
    );
    let e = w.el(id);
    assert_eq!(
        e.shape().unwrap().text,
        "uma ideia comprida que precisa de várias linhas"
    );
    assert!(e.h > 40.0, "a forma não cresceu: {}", e.h);
    assert!(
        w.ed.overlay(&w.doc, &mut w.ts)
            .text
            .is_some_and(|t| t.caret.is_some())
    );
    assert!(
        w.ed.overlay(&w.doc, &mut w.ts).frame.is_none(),
        "pegas por cima do texto"
    );
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::Commit);
    assert!(!w.ed.is_editing_text());
    w.cmd(Command::Undo);
    let e = w.el(id);
    assert!(
        e.shape().unwrap().text.is_empty() && e.h == 40.0,
        "o texto desfez-se aos bocados"
    );
}

#[test]
fn deleting_the_text_shrinks_back_but_never_below_the_starting_height() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 120.0, 40.0]);
    w.ed.begin_text(&mut w.doc, &mut w.ts, id, None);
    w.ed.text_input(&mut w.doc, &mut w.ts, "linha\nlinha\nlinha\nlinha");
    assert!(w.el(id).h > 40.0);
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::SelectAll);
    w.ed.text_key(
        &mut w.doc,
        &mut w.h,
        &mut w.ts,
        TextKey::Backspace { word: false },
    );
    assert_eq!(w.el(id).h, 40.0);
}

#[test]
fn clicking_outside_while_writing_ends_the_edit() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 200.0, 60.0]);
    w.ed.begin_text(&mut w.doc, &mut w.ts, id, None);
    w.ed.text_input(&mut w.doc, &mut w.ts, "x");
    w.click([900.0, 900.0], NONE);
    assert!(!w.ed.is_editing_text());
    assert!(w.cmd(Command::Undo), "a edição não ficou como passo");
    assert!(w.el(id).shape().unwrap().text.is_empty());
}

#[test]
fn delete_duplicate_copy_paste_and_nudge_are_each_one_step() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 50.0, 50.0]);
    assert!(w.cmd(Command::Duplicate));
    let dup = *w.ed.selection().iter().next().unwrap();
    assert_eq!([w.el(dup).x, w.el(dup).y], [10.0, 10.0]);
    assert!(w.cmd(Command::Nudge([1.0, 0.0])));
    assert_eq!(w.el(dup).x, 11.0);
    assert!(w.cmd(Command::Copy));
    assert!(w.cmd(Command::Paste));
    assert!(w.cmd(Command::Paste));
    assert_eq!(w.doc.live_len(), 4);
    assert!(w.cmd(Command::SelectAll));
    assert!(w.cmd(Command::Delete));
    assert_eq!(w.doc.live_len(), 0);
    for expect in [4, 3, 2, 2, 1] {
        w.cmd(Command::Undo);
        assert_eq!(w.doc.live_len(), expect);
    }
    let _ = id;
}

#[test]
fn escape_cancels_a_drag_mid_way_then_clears_the_selection_then_the_tool() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 50.0, 50.0]);
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(25.0, 25.0, CTRL), VIEW);
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(300.0, 300.0, CTRL));
    assert!(w.cmd(Command::Escape));
    assert_eq!(w.el(id).x, 0.0, "o Esc não devolveu o arrasto");
    // O desfazer seguinte é o da CRIAÇÃO — o arrasto cancelado não deixou passo.
    w.cmd(Command::Undo);
    assert_eq!(w.doc.live_len(), 0, "um arrasto cancelado virou passo");
    w.cmd(Command::Redo);
    w.ed.select(&w.doc, [id]);
    assert!(w.cmd(Command::Escape));
    assert!(w.ed.selection().is_empty());
    w.ed.tool = Tool::Shape(ShapeType::Diamond);
    assert!(w.cmd(Command::Escape));
    assert_eq!(w.ed.tool, Tool::Select);
    assert!(!w.cmd(Command::Escape));
}

#[test]
fn style_changes_apply_to_the_selection_as_one_step_and_to_the_next_shape() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 50.0, 50.0]);
    let b = w.rect([100.0, 0.0, 50.0, 50.0]);
    w.cmd(Command::SelectAll);
    w.ed.set_style(&mut w.doc, &mut w.h, |s| {
        s.dash = ph2d_board_model::Dash::Dotted
    });
    for id in [a, b] {
        assert_eq!(
            w.el(id).shape().unwrap().style.dash,
            ph2d_board_model::Dash::Dotted
        );
    }
    let c = w.rect([300.0, 0.0, 50.0, 50.0]);
    assert_eq!(
        w.el(c).shape().unwrap().style.dash,
        ph2d_board_model::Dash::Dotted
    );
    w.cmd(Command::Undo); // a criação de c
    w.cmd(Command::Undo); // o estilo
    assert_eq!(
        w.el(a).shape().unwrap().style.dash,
        ph2d_board_model::Dash::Solid
    );
}

#[test]
fn the_hand_tool_hands_the_drag_back_to_the_view() {
    let mut w = world();
    w.ed.tool = Tool::Hand;
    let d =
        w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(0.0, 0.0, NONE), VIEW);
    assert_eq!(d, Down::Pan);
}
