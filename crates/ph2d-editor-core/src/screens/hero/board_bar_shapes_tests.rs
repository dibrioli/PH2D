//! O botão «Forma» da barra de contexto pelo caminho real do ecrã: o clique pelo despacho sobre o que
//! o pintor registou, o rato no quadro pela porta da shell.

use super::*;

impl T {
    /// Uma forma pela ferramenta `k` da barra curta, entre as fracções `a` e `b` da área.
    fn shape(&mut self, k: ShapeType, a: (f32, f32), b: (f32, f32)) {
        self.click_bar(Item::Tool(Tool::Shape(k)));
        let (a, b) = (self.at(a.0, a.1), self.at(b.0, b.1));
        self.drag(a, b);
    }

    fn kinds(&self) -> Vec<ShapeType> {
        self.elements()
            .iter()
            .filter_map(|e| e.shape().map(|s| s.kind))
            .collect()
    }
}

fn on_screen(t: &T, it: Item) -> bool {
    t.find(node(it)).is_some()
}

#[test]
fn the_shape_button_turns_every_selected_shape_in_one_undo_step() {
    let mut t = T::new();
    t.shape(ShapeType::Rectangle, (0.2, 0.3), (0.3, 0.4));
    t.shape(ShapeType::Ellipse, (0.5, 0.3), (0.6, 0.4));
    t.key(BoardKey::Char('a'), CTRL, None);
    assert!(
        !on_screen(&t, Item::Pick(ShapeType::Hexagon)),
        "a grelha nasce aberta"
    );
    t.click_bar(Item::Reshape);
    t.click_bar(Item::Pick(ShapeType::Hexagon));
    assert_eq!(t.kinds(), [ShapeType::Hexagon; 2]);
    assert!(
        !on_screen(&t, Item::Pick(ShapeType::Hexagon)),
        "a grelha não fechou ao escolher"
    );
    let tool = t.hero.documents.live.editor.as_ref().unwrap().tool;
    assert_eq!(tool, Tool::Select, "trocar o tipo escolheu uma ferramenta");
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    let before = [ShapeType::Rectangle, ShapeType::Ellipse];
    assert_eq!(t.kinds(), before, "não foi UM passo do desfazer");
}

/// ⛔ Uma selecção com uma nota não mostra o «Forma» (o cofre do modo privado sela por nota).
#[test]
fn a_selection_with_a_note_has_no_shape_button() {
    let mut t = T::new();
    t.shape(ShapeType::Rectangle, (0.2, 0.3), (0.3, 0.4));
    assert!(
        on_screen(&t, Item::Reshape),
        "uma forma sozinha não mostra o «Forma»"
    );
    t.click_bar(Item::Tool(Tool::Shape(ShapeType::Sticky)));
    let p = t.at(0.6, 0.5);
    t.drag(p, p);
    t.key(BoardKey::Char('a'), CTRL, None);
    assert!(
        on_screen(&t, Item::Fill(None)),
        "a barra da forma e da nota sumiu"
    );
    assert!(
        !on_screen(&t, Item::Reshape),
        "o «Forma» aparece com uma nota na selecção"
    );
}

/// A grelha nunca fica aberta por baixo de outra coisa: trocar a selecção fecha-a, e o «mais
/// formas» da barra curta fecha-a e volta a escolher a FERRAMENTA (o mesmo `Pick`).
#[test]
fn the_shape_grid_closes_on_another_selection_and_on_more_shapes() {
    let mut t = T::new();
    t.shape(ShapeType::Rectangle, (0.2, 0.3), (0.3, 0.4));
    t.shape(ShapeType::Ellipse, (0.5, 0.3), (0.6, 0.4));
    t.click_bar(Item::Reshape);
    assert!(
        on_screen(&t, Item::Pick(ShapeType::Hexagon)),
        "o «Forma» não abriu a grelha"
    );
    let first = t.at(0.25, 0.35);
    t.drag(first, first);
    assert!(
        on_screen(&t, Item::Reshape),
        "o clique não seleccionou a 1.ª forma"
    );
    assert!(
        !on_screen(&t, Item::Pick(ShapeType::Hexagon)),
        "a grelha ficou noutra selecção"
    );
    // (Nunca o mesmo botão duas vezes seguidas: o arnês carimba tudo no instante 0 — duplo clique.)
    t.click_bar(Item::MoreShapes);
    t.click_bar(Item::Reshape);
    let live = &t.hero.documents.live;
    assert!(
        live.reshape.is_some() && !live.shapes_open,
        "o «Forma» não fechou o «mais formas»"
    );
    t.click_bar(Item::MoreShapes);
    let live = &t.hero.documents.live;
    assert!(
        live.reshape.is_none() && live.shapes_open,
        "o «mais formas» não fechou o «Forma»"
    );
    t.click_bar(Item::Pick(ShapeType::Cylinder));
    let before = [ShapeType::Rectangle, ShapeType::Ellipse];
    assert_eq!(t.kinds(), before, "o «mais formas» trocou o tipo");
    let tool = t.hero.documents.live.editor.as_ref().unwrap().tool;
    assert_eq!(tool, Tool::Shape(ShapeType::Cylinder));
}

/// Arrastar a forma esconde a barra (o quadro repinta a cada passo do rato): a grelha fecha e não
/// volta a abrir ao largar.
#[test]
fn dragging_the_shape_closes_the_shape_grid() {
    let mut t = T::new();
    t.shape(ShapeType::Rectangle, (0.2, 0.3), (0.3, 0.4));
    t.click_bar(Item::Reshape);
    let (a, b) = (t.at(0.25, 0.35), t.at(0.4, 0.5));
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    let p = PointerButton::Primary;
    assert!(board_view::pointer(
        h,
        inp(ts, c),
        PointerKind::Down,
        p,
        a.0,
        a.1,
        true
    ));
    board_view::pointer_move(h, inp(ts, c), b.0, b.1);
    t.paint();
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    board_view::pointer(h, inp(ts, c), PointerKind::Up, p, b.0, b.1, true);
    t.paint();
    assert!(
        on_screen(&t, Item::Reshape),
        "a forma largada não ficou seleccionada"
    );
    assert!(
        !on_screen(&t, Item::Pick(ShapeType::Hexagon)),
        "a grelha voltou depois do arrasto"
    );
}
