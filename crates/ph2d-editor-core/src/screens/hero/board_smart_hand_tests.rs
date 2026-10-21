//! ⭐ O QuickShape do quadro pelo caminho REAL do ecrã com traços de MÃO (smoke do dono 21/10): o
//! `Shift` regulariza a forma segurada SEM mexer o rato (o quadro de pintura lê-o, como o lápis do
//! vetor lê `modifiers.shift_key()` a cada quadro), e num quadro em RASCUNHO a forma da caneta
//! pinta-se como a da ferramenta dessa forma.

use super::*;
use ph2d_board_edit::Tool;
use ph2d_board_model::{Pen, ShapeType};

#[path = "../../../../ph2d-board-edit/src/smart_test_hand.rs"]
#[allow(dead_code)]
mod hand;

/// Desenhar: um evento a cada 4 ms; parado: o evento seguinte 600 ms depois, no mesmo sítio.
const DT: u128 = 4_000_000;
const STILL: u128 = 600_000_000;

fn input(ts: &mut TextSystem, now_ns: u128, shift: bool) -> Input<'_> {
    Input {
        text: ts,
        mods: Modifiers {
            shift,
            ..Modifiers::default()
        },
        now_ns,
    }
}

/// `pts` posto na área do quadro (a caixa dele a começar a 10 % da área), em px de ecrã.
fn place(t: &T, pts: &[[f64; 2]]) -> Vec<(f32, f32)> {
    let lo = pts
        .iter()
        .fold([f64::MAX; 2], |m, p| [m[0].min(p[0]), m[1].min(p[1])]);
    let o = t.at(0.1, 0.1);
    pts.iter()
        .map(|p| (o.0 + (p[0] - lo[0]) as f32, o.1 + (p[1] - lo[1]) as f32))
        .collect()
}

impl T {
    /// A caneta desenha `pts` e fica PARADA no fim (o `Shift` dos eventos = `shift`); um quadro pintado.
    fn pen_hold(&mut self, pts: &[[f64; 2]], shift: bool) -> (f32, f32) {
        self.board_editor().tool = Tool::Pen(Pen::Pen);
        let q = place(self, pts);
        let end = q[q.len() - 1];
        let (h, ts) = (&mut self.hero, &mut self.ts);
        let mut now = self.clock;
        let (down, p) = (PointerKind::Down, PointerButton::Primary);
        assert!(board_view::pointer(
            h,
            input(ts, now, shift),
            down,
            p,
            q[0].0,
            q[0].1,
            true
        ));
        for s in &q[1..] {
            now += DT;
            board_view::pointer_move(h, input(ts, now, shift), s.0, s.1);
        }
        now += STILL;
        board_view::pointer_move(h, input(ts, now, shift), end.0, end.1);
        self.clock = now;
        self.paint();
        end
    }

    fn release(&mut self, end: (f32, f32), shift: bool) {
        self.clock += DT;
        let (h, ts) = (&mut self.hero, &mut self.ts);
        let up = PointerKind::Up;
        board_view::pointer(
            h,
            input(ts, self.clock, shift),
            up,
            PointerButton::Primary,
            end.0,
            end.1,
            true,
        );
        self.paint();
    }

    /// A tecla `Shift` sozinha (a shell só actualiza os modificadores) e um quadro pintado.
    fn shift(&mut self, held: bool) {
        self.hero.store.set_shift_held(held);
        self.paint();
    }

    fn board_editor(&mut self) -> &mut ph2d_board_edit::Editor {
        self.hero
            .documents
            .live
            .editor
            .as_mut()
            .expect("o editor do quadro")
    }

    fn px(&self) -> f64 {
        1.0 / self
            .hero
            .documents
            .active_board()
            .expect("um quadro")
            .camera
            .zoom
    }

    fn only(&self) -> Element {
        let els = self.elements();
        assert_eq!(els.len(), 1, "{els:?}");
        els[0].clone()
    }
}

/// A forma viva, segurada: (tipo, largura, altura).
fn live(t: &T) -> (Option<ShapeType>, f64, f64) {
    let el = t.only();
    (el.shape().map(|s| s.kind), el.w, el.h)
}

/// ⭐ O `Shift` SEM mexer: a caixa (a elipse) segurada passa a quadrado (círculo) e, largado, volta à
/// forma livre — o que o lápis do vetor faz (`Hold::tick` refaz a forma quando o `regular` muda).
#[test]
fn shift_alone_regularises_the_held_shape_and_letting_go_gives_it_back() {
    for (name, pts, kind) in [
        ("caixa", hand::hand_rect(), ShapeType::Rectangle),
        ("elipse", hand::hand_ellipse(), ShapeType::Ellipse),
    ] {
        let mut t = T::new();
        let end = t.pen_hold(&pts, false);
        let px = t.px();
        let free = live(&t);
        assert_eq!(free.0, Some(kind), "{name}");
        assert!(
            (free.1 - free.2).abs() > 50.0 * px,
            "{name}: livre {free:?}"
        );
        t.shift(true);
        let reg = live(&t);
        assert_eq!(reg.0, Some(kind), "{name}");
        assert!(
            (reg.1 - reg.2).abs() <= px,
            "{name}: o Shift sem mexer regulariza: {reg:?}"
        );
        t.shift(false);
        let back = live(&t);
        assert!(
            (back.1 - free.1).abs() <= px && (back.2 - free.2).abs() <= px,
            "{name}: largar o Shift volta à forma livre: {free:?} → {back:?}"
        );
        t.shift(true);
        t.release(end, true);
        let fin = live(&t);
        assert!(
            (fin.1 - fin.2).abs() <= px,
            "{name}: soltar guarda o regular: {fin:?}"
        );
    }
}

/// O `Shift` premido ANTES de desenhar (nos eventos e no teclado) dá a forma regular ao segurar.
#[test]
fn shift_held_from_the_start_gives_the_regular_shape() {
    for (name, pts) in [
        ("caixa", hand::hand_rect()),
        ("elipse", hand::hand_ellipse()),
    ] {
        let mut t = T::new();
        t.hero.store.set_shift_held(true);
        t.pen_hold(&pts, true);
        let reg = live(&t);
        assert!((reg.1 - reg.2).abs() <= t.px(), "{name}: {reg:?}");
    }
}

fn sides(corners: &[[f64; 2]], per: usize) -> Vec<[f64; 2]> {
    let mut v = vec![corners[0]];
    for w in corners.windows(2) {
        v.extend((1..=per).map(|k| {
            let u = k as f64 / per as f64;
            [
                w[0][0] + (w[1][0] - w[0][0]) * u,
                w[0][1] + (w[1][1] - w[0][1]) * u,
            ]
        }));
    }
    v
}

/// As perguntas que o pintor faz para decidir o traço à mão (`ph2d-board-render`: a forma
/// `hand = (sketch ∧ ¬longe) ∨ ¬sólido`, a ligação `sketch ∧ traço`, a tinta nenhuma) e a semente.
fn hand_questions(el: &Element) -> (bool, ph2d_board_model::Look, Option<u32>, &'static str) {
    let st = el.style();
    let what = if el.ink().is_some() {
        "tinta"
    } else if el.connector().is_some() {
        "ligação"
    } else {
        "forma"
    };
    (
        st.sketch,
        st.look,
        el.org.as_ref().and_then(|o| o.seed),
        what,
    )
}

/// ⭐ Num quadro em RASCUNHO, cada forma que a caneta dá (segurar) pinta-se como a da ferramenta
/// dessa forma; o que fica tinta limpa pinta-se como a tinta crua da caneta.
#[test]
fn pen_shapes_in_a_sketch_board_are_drawn_like_the_tools_shapes() {
    let rhombus = sides(
        &[
            [0.0, -150.0],
            [100.0, 0.0],
            [0.0, 150.0],
            [-100.0, 0.0],
            [0.0, -150.0],
        ],
        40,
    );
    let line: Vec<[f64; 2]> = (0..=60).map(|i| [10.0 * f64::from(i), 0.0]).collect();
    let mut arrow = line.clone();
    arrow.extend(sides(&[[600.0, 0.0], [540.0, -40.0]], 10));
    arrow.extend(sides(&[[540.0, -40.0], [600.0, 0.0], [540.0, 40.0]], 10));
    let zigzag = sides(
        &[[0.0, 0.0], [200.0, 150.0], [400.0, 0.0], [600.0, 150.0]],
        40,
    );
    #[derive(Clone, Copy)]
    enum Other {
        Tool(Tool),
        RawInk,
    }
    let cases: [(&str, Vec<[f64; 2]>, bool, Other); 9] = [
        (
            "rectângulo",
            hand::hand_rect(),
            false,
            Other::Tool(Tool::Shape(ShapeType::Rectangle)),
        ),
        (
            "quadrado",
            hand::hand_rect(),
            true,
            Other::Tool(Tool::Shape(ShapeType::Rectangle)),
        ),
        (
            "elipse",
            hand::hand_ellipse(),
            false,
            Other::Tool(Tool::Shape(ShapeType::Ellipse)),
        ),
        (
            "círculo",
            hand::hand_ellipse(),
            true,
            Other::Tool(Tool::Shape(ShapeType::Ellipse)),
        ),
        (
            "losango",
            rhombus,
            false,
            Other::Tool(Tool::Shape(ShapeType::Diamond)),
        ),
        ("linha", line, false, Other::Tool(Tool::Connector)),
        ("seta", arrow, false, Other::Tool(Tool::Connector)),
        (
            "triângulo (tinta limpa)",
            hand::hand_triangle(),
            false,
            Other::RawInk,
        ),
        ("polilinha (tinta limpa)", zigzag, false, Other::RawInk),
    ];
    for (name, pts, shift, other) in cases {
        let mut t = T::new();
        t.hero
            .documents
            .active_board_mut()
            .expect("um quadro")
            .sketch = true;
        t.hero.store.set_shift_held(shift);
        let end = t.pen_hold(&pts, shift);
        t.release(end, shift);
        let pen = t.only();
        let mut u = T::new();
        u.hero
            .documents
            .active_board_mut()
            .expect("um quadro")
            .sketch = true;
        match other {
            Other::Tool(tool) => {
                u.board_editor().tool = tool;
                let (a, b) = (u.at(0.2, 0.2), u.at(0.5, 0.45));
                u.drag(a, b);
            }
            Other::RawInk => {
                u.board_editor().tool = Tool::Pen(Pen::Pen);
                let q = place(&u, &pts);
                u.drag(q[0], q[q.len() / 2]);
            }
        }
        let tool = u.only();
        assert_eq!(
            hand_questions(&pen),
            hand_questions(&tool),
            "{name}: a caneta × a ferramenta (rascunho, look, semente, espécie)"
        );
        // A cor e a espessura são as da CANETA (o que o artista escolheu nela).
        let preset = t.board_editor().pen.current(Pen::Pen);
        let st = pen.style();
        assert_eq!(
            (st.stroke, st.stroke_width),
            (Some(preset.color), preset.width),
            "{name}: o traço da caneta"
        );
    }
}
