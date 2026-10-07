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
        bind: 8.0,
        dot: 40.0,
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
    assert!(w.ed.double_click(&mut w.doc, &mut w.h, &mut w.ts, at(60.0, 20.0, NONE)));
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

/// Com `Shift` no canto, manda o eixo que o dedo mais esticou — também quando é o VERTICAL (a
/// prova de mutação apanhou que só o horizontal estava medido).
#[test]
fn shift_on_a_corner_follows_the_axis_stretched_most() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 100.0, 50.0]);
    w.drag(
        [100.0, 50.0],
        [110.0, 150.0],
        Mods {
            ctrl: true,
            ..SHIFT
        },
    );
    let e = w.el(id);
    assert_eq!([e.w, e.h], [300.0, 150.0], "o vertical (×3) não mandou");
}

// ── setas (W2) ───────────────────────────────────────────────────────────────────────────────

use ph2d_board_model::{Anchor, End, Head, Route};

impl World {
    fn arrow_drag(&mut self, from: [f64; 2], to: [f64; 2], m: Mods) -> Option<ElementId> {
        self.ed.tool = Tool::Connector;
        self.drag(from, to, m);
        let id = *self.ed.selection().iter().next()?;
        self.el(id).connector().is_some().then_some(id)
    }

    fn ends(&self, id: ElementId) -> [End; 2] {
        self.el(id).connector().expect("é seta").ends()
    }
}

/// ⭐ Arrastar a ferramenta Seta do miolo de uma forma ao miolo de outra liga as duas ao CENTRO,
/// selecciona a seta, volta à selecção — e desfazer tira-a.
#[test]
fn dragging_the_arrow_tool_from_a_shape_to_another_binds_both_centers() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 160.0, 100.0]);
    let b = w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w
        .arrow_drag([80.0, 50.0], [480.0, 50.0], NONE)
        .expect("nasceu uma seta");
    let center = |target| End::Bound {
        target,
        anchor: Anchor::Center,
    };
    assert_eq!(w.ends(id), [center(a), center(b)]);
    assert_eq!(w.ed.tool, Tool::Select);
    assert_eq!(
        w.el(id).connector().unwrap().heads,
        [Head::None, Head::Arrow]
    );
    assert!(w.cmd(Command::Undo));
    assert!(w.doc.get(id).is_none());
    assert_eq!(w.doc.live_len(), 2);
}

/// Largada na faixa junto ao contorno, a ponta fica num PONTO FIXO (colado ao meio do lado).
#[test]
fn an_end_dropped_near_the_outline_is_a_fixed_point() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 160.0, 100.0]);
    let b = w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [403.0, 52.0], NONE).unwrap();
    assert_eq!(
        w.ends(id)[1],
        End::Bound {
            target: b,
            anchor: Anchor::Fixed([0.0, 0.5])
        }
    );
    let _ = a;
}

/// ⭐ `Ctrl` solta: com ele, a ponta fica no vazio mesmo sobre uma forma.
#[test]
fn ctrl_keeps_the_end_loose_over_a_shape() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    let b = w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([-200.0, 50.0], [480.0, 50.0], CTRL).unwrap();
    assert_eq!(
        w.ends(id),
        [End::Free([-200.0, 50.0]), End::Free([480.0, 50.0])]
    );
    let _ = b;
}

/// Durante o arrasto, a forma por baixo realça-se (o alvo do overlay).
#[test]
fn the_shape_under_the_dragged_end_is_highlighted() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    let b = w.rect([400.0, 0.0, 160.0, 100.0]);
    w.ed.tool = Tool::Connector;
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(80.0, 50.0, NONE), VIEW);
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(300.0, 50.0, NONE));
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(480.0, 50.0, NONE));
    let o = w.ed.overlay(&w.doc, &mut w.ts);
    assert_eq!(
        o.target,
        Some(Target {
            element: b,
            fixed: None
        })
    );
}

/// ⭐ Mover uma caixa: a seta presa a ela segue-a (a rota refaz-se, o documento da seta não muda).
#[test]
fn moving_a_box_moves_the_arrow_end_with_it() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    let b = w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 50.0], NONE).unwrap();
    let before = w.ed.routes(&w.doc).get(id).unwrap().ends()[1];
    let saved = w.el(id).clone();
    w.ed.select(&w.doc, [b]);
    w.drag([480.0, 50.0], [480.0, 350.0], CTRL);
    let after = w.ed.routes(&w.doc).get(id).unwrap().ends()[1];
    assert_ne!(before, after);
    assert!(after[1] >= 300.0, "a ponta foi com a caixa: {after:?}");
    assert_eq!(
        w.el(id).connector(),
        saved.connector(),
        "a seta não precisou de mudar"
    );
}

/// Arrastar a PONTA de uma seta seleccionada religa-a a outra forma (UM passo).
#[test]
fn dragging_an_end_handle_rebinds_it() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([400.0, 0.0, 160.0, 100.0]);
    let c = w.rect([400.0, 300.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 50.0], NONE).unwrap();
    let tip = w.ed.routes(&w.doc).get(id).unwrap().ends()[1];
    w.drag(tip, [480.0, 350.0], NONE);
    assert_eq!(
        w.ends(id)[1],
        End::Bound {
            target: c,
            anchor: Anchor::Center
        }
    );
    assert!(w.cmd(Command::Undo));
    assert_ne!(
        w.ends(id)[1],
        End::Bound {
            target: c,
            anchor: Anchor::Center
        }
    );
}

/// ⭐ Os pontos azuis: clicar no da direita cria a forma seguinte já ligada, seleccionada.
#[test]
fn clicking_a_blue_dot_creates_the_next_shape_already_linked() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 160.0, 100.0]);
    let o = w.ed.overlay(&w.doc, &mut w.ts);
    assert_eq!(o.dots.len(), 4, "a forma seleccionada mostra os quatro");
    let (at_side, dir) = o.dots[1];
    let p = [at_side[0] + dir[0] * 40.0, at_side[1] + dir[1] * 40.0];
    w.click(p, NONE);
    assert_eq!(w.doc.live_len(), 3, "a forma nova e a seta");
    let new = *w.ed.selection().iter().next().unwrap();
    let n = w.el(new);
    assert_eq!([n.x, n.y, n.w, n.h], [160.0 + NEXT_GAP, 0.0, 160.0, 100.0]);
    let wire = w.doc.live().find(|el| el.connector().is_some()).unwrap();
    let ends = wire.connector().unwrap().ends();
    assert!(
        matches!(ends, [End::Bound { target: s, .. }, End::Bound { target: t, .. }] if s == a && t == new)
    );
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.live_len(), 1, "UM passo tira as duas");
}

/// `Ctrl+seta` faz o mesmo pelo teclado, e salta por cima de uma forma que já ocupa o sítio.
#[test]
fn ctrl_arrow_grows_the_flow_and_skips_an_occupied_place() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([0.0, 100.0 + NEXT_GAP, 160.0, 100.0]);
    w.ed.select(&w.doc, [a]);
    assert!(w.cmd(Command::Grow([0.0, 1.0])));
    let new = *w.ed.selection().iter().next().unwrap();
    let n = w.el(new);
    assert_eq!(
        n.y,
        2.0 * (100.0 + NEXT_GAP),
        "o sítio estava ocupado: salta um passo"
    );
}

/// ⭐ Apagar uma forma solta a ponta da seta ONDE ESTAVA — e desfazer prende-a outra vez.
#[test]
fn deleting_a_shape_releases_the_arrow_end_in_place() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    let b = w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 50.0], NONE).unwrap();
    let tip = w.ed.routes(&w.doc).get(id).unwrap().ends()[1];
    w.ed.select(&w.doc, [b]);
    w.cmd(Command::Delete);
    assert_eq!(w.ends(id)[1], End::Free(tip));
    w.cmd(Command::Undo);
    assert!(matches!(w.ends(id)[1], End::Bound { target, .. } if target == b));
}

/// Duplicar duas formas e a seta entre elas: a cópia da seta liga as CÓPIAS.
#[test]
fn duplicating_shapes_with_their_arrow_links_the_copies() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 160.0, 100.0]);
    let b = w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 50.0], NONE).unwrap();
    w.ed.select(&w.doc, [a, b, id]);
    w.cmd(Command::Duplicate);
    let copies: Vec<ElementId> = w.ed.selection().iter().copied().collect();
    let wire = copies
        .iter()
        .find(|c| w.el(**c).connector().is_some())
        .unwrap();
    for t in w.el(*wire).connector().unwrap().targets() {
        assert!(copies.contains(&t), "a cópia da seta prende-se às cópias");
    }
}

/// Uma seta sozinha copiada solta as pontas onde estão (não fica presa às formas do original).
#[test]
fn copying_an_arrow_alone_frees_its_ends() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 50.0], NONE).unwrap();
    w.cmd(Command::Duplicate);
    let copy = *w.ed.selection().iter().next().unwrap();
    assert_ne!(copy, id);
    assert!(w.ends(copy).iter().all(|e| matches!(e, End::Free(_))));
}

/// Clicar na LINHA de uma seta selecciona-a; a rota e as pontas aparecem no overlay.
#[test]
fn clicking_the_line_selects_the_arrow() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 50.0], NONE).unwrap();
    w.cmd(Command::Escape);
    w.click([280.0, 51.0], NONE);
    assert_eq!(
        w.ed.selection().iter().copied().collect::<Vec<_>>(),
        vec![id]
    );
    let o = w.ed.overlay(&w.doc, &mut w.ts);
    assert_eq!(o.wires.len(), 1);
    assert!(
        o.frame.is_none(),
        "uma seta não tem moldura de redimensionar"
    );
}

/// A rota, as pontas e o rótulo pela barra/teclado.
#[test]
fn route_heads_and_label_change_the_selected_arrow() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([400.0, 300.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 350.0], NONE).unwrap();
    w.ed.set_route(&mut w.doc, &mut w.h, Route::Curved);
    w.ed.set_head(&mut w.doc, &mut w.h, 0, Head::Circle);
    let c = w.el(id).connector().unwrap();
    assert_eq!((c.route, c.heads[0]), (Route::Curved, Head::Circle));
    assert!(w.cmd(Command::EditText), "Enter escreve no rótulo");
    w.ed.text_input(&mut w.doc, &mut w.ts, "sim");
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::Commit);
    assert_eq!(w.el(id).connector().unwrap().label, "sim");
    assert!(w.cmd(Command::Undo));
    assert_eq!(
        w.el(id).connector().unwrap().label,
        "",
        "a escrita é UM passo"
    );
}

/// O rótulo de nascença das setas: a largura de quebra é a de uma caixa de nascença.
#[test]
fn the_label_wraps_at_the_click_width() {
    assert_eq!(ph2d_board_route::LABEL_WRAP, CLICK_SIZE[0]);
}

/// Os pontos azuis ficam LONGE da pega de rodar (senão a de cima rouba-lhe o clique).
#[test]
fn the_top_dot_clears_the_rotate_knob() {
    let m = metrics();
    assert!(m.dot - m.rotate_offset >= 2.0 * m.handle);
}

/// ⭐ Os pontos de ajuste do Miro: arrastar a bolinha do MEIO de uma seta seleccionada cria um ponto
/// ali e leva-o (UM passo); arrastar o ponto move-o; duplo-clique apaga-o; um toque no meio sem
/// arrastar não cria nada.
#[test]
fn dragging_the_middle_handle_bends_the_arrow_through_a_new_point() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 50.0], NONE).unwrap();
    let mid = w.ed.overlay(&w.doc, &mut w.ts).wires[0].mids[0];
    w.click(mid, NONE);
    assert!(
        w.el(id).connector().unwrap().waypoints.is_empty(),
        "um toque não cria ponto"
    );
    w.drag(mid, [280.0, 200.0], NONE);
    assert_eq!(
        w.el(id).connector().unwrap().waypoints,
        vec![[280.0, 200.0]]
    );
    let line = w.ed.routes(&w.doc).get(id).unwrap().polyline();
    assert!(
        line.contains(&[280.0, 200.0]),
        "a seta passa pelo ponto: {line:?}"
    );
    w.drag([280.0, 200.0], [300.0, 260.0], NONE);
    assert_eq!(
        w.el(id).connector().unwrap().waypoints,
        vec![[300.0, 260.0]]
    );
    assert!(w.cmd(Command::Undo));
    assert_eq!(
        w.el(id).connector().unwrap().waypoints,
        vec![[280.0, 200.0]]
    );
    assert!(w.ed.double_click(&mut w.doc, &mut w.h, &mut w.ts, at(280.0, 200.0, NONE)));
    assert!(
        w.el(id).connector().unwrap().waypoints.is_empty(),
        "duplo-clique apaga o ponto"
    );
    assert!(w.cmd(Command::Undo));
    assert_eq!(
        w.el(id).connector().unwrap().waypoints,
        vec![[280.0, 200.0]]
    );
}

/// A seta de nascença é a CURVA (ordem do dono, 06/10 — a do Miro).
#[test]
fn a_new_arrow_is_curved() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([400.0, 200.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 250.0], NONE).unwrap();
    assert_eq!(w.el(id).connector().unwrap().route, Route::Curved);
}

/// O meio do 2.º trecho de uma seta que JÁ tem um ponto cria o ponto novo ENTRE o 1.º e o fim (a
/// ordem da rota), não no fim da lista.
#[test]
fn bending_the_second_leg_inserts_the_point_in_route_order() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([600.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [680.0, 50.0], NONE).unwrap();
    let mut el = w.el(id).clone();
    el.connector_mut().unwrap().waypoints = vec![[200.0, 300.0]];
    w.h.apply(&mut w.doc, vec![BoardOp::Put(el)]);
    let mid = w.ed.overlay(&w.doc, &mut w.ts).wires[0].mids[1];
    w.drag(mid, [500.0, 300.0], NONE);
    assert_eq!(
        w.el(id).connector().unwrap().waypoints,
        vec![[200.0, 300.0], [500.0, 300.0]]
    );
    let mid0 = w.ed.overlay(&w.doc, &mut w.ts).wires[0].mids[0];
    w.drag(mid0, [100.0, 400.0], NONE);
    assert_eq!(
        w.el(id).connector().unwrap().waypoints,
        vec![[100.0, 400.0], [200.0, 300.0], [500.0, 300.0]],
        "o meio do 1.º trecho entra ANTES do 1.º ponto"
    );
}

/// Arrastar uma seta SOLTA leva os seus pontos de ajuste com ela.
#[test]
fn moving_a_loose_arrow_carries_its_points() {
    let mut w = world();
    let id = w.arrow_drag([0.0, 0.0], [300.0, 0.0], NONE).unwrap();
    let mut el = w.el(id).clone();
    el.connector_mut().unwrap().waypoints = vec![[150.0, 100.0]];
    w.h.apply(&mut w.doc, vec![BoardOp::Put(el)]);
    w.cmd(Command::Nudge([10.0, 5.0]));
    let c = w.el(id).connector().unwrap().clone();
    assert_eq!(c.waypoints, vec![[160.0, 105.0]]);
    assert_eq!(c.ends(), [End::Free([10.0, 5.0]), End::Free([310.0, 5.0])]);
}

/// ⭐ Arrastar um ponto de ajuste para cima do VIZINHO (outro ponto, ou uma ponta) funde-o: o ponto
/// some, UM passo de desfazer devolve-o (pedido do dono, 06/10 — o idioma do Miro).
#[test]
fn dropping_a_point_on_its_neighbour_deletes_it() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([600.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [680.0, 50.0], NONE).unwrap();
    let mut el = w.el(id).clone();
    el.connector_mut().unwrap().waypoints = vec![[300.0, 250.0], [450.0, 250.0]];
    w.h.apply(&mut w.doc, vec![BoardOp::Put(el)]);
    // O 1.º ponto largado em cima do 2.º: funde-se.
    w.drag([300.0, 250.0], [452.0, 251.0], NONE);
    assert_eq!(
        w.el(id).connector().unwrap().waypoints,
        vec![[450.0, 250.0]]
    );
    // O que sobra largado em cima da PONTA de fim: funde-se também.
    let tip = w.ed.routes(&w.doc).get(id).unwrap().ends()[1];
    w.drag([450.0, 250.0], [tip[0] + 2.0, tip[1]], NONE);
    assert!(w.el(id).connector().unwrap().waypoints.is_empty());
    assert!(w.cmd(Command::Undo));
    assert_eq!(
        w.el(id).connector().unwrap().waypoints,
        vec![[450.0, 250.0]],
        "UM passo por fusão"
    );
    // Largado LONGE do vizinho, fica.
    w.drag([450.0, 250.0], [500.0, 300.0], NONE);
    assert_eq!(
        w.el(id).connector().unwrap().waypoints,
        vec![[500.0, 300.0]]
    );
}
