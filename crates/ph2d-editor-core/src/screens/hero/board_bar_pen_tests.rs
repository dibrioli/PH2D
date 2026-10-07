//! A CANETA e o RASCUNHO (W4) pelo caminho real do ecrã: o botão e o painel pelo despacho, o rato
//! pela porta da shell, as teclas pela do teclado.

use super::*;
use ph2d_board_model::Pen;

const NONE: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};

impl T {
    fn inks(&self) -> Vec<Element> {
        self.elements()
            .into_iter()
            .filter(|e| e.ink().is_some())
            .collect()
    }

    fn tool(&self) -> Tool {
        self.hero.documents.live.editor.as_ref().unwrap().tool
    }

    fn sketch(&self) -> bool {
        self.hero.documents.active_board().unwrap().sketch
    }

    fn press(&mut self, c: char) {
        let s = c.to_string();
        assert!(
            self.key(BoardKey::Char(c), NONE, Some(&s)),
            "a tecla {c} não foi do quadro"
        );
    }
}

/// ⭐ O botão da caneta abre o painel; uma predefinição escolhe-se; arrastar no quadro desenha um
/// traço com ela — e a caneta fica na mão (o idioma do Miro).
#[test]
fn the_pen_button_opens_its_panel_and_a_drag_draws_with_the_chosen_preset() {
    let mut t = T::new();
    assert!(
        t.find(node(Item::Pen(PenItem::Preset(1)))).is_none(),
        "o painel nasce fechado"
    );
    t.click_bar(Item::Tool(Tool::Pen(Pen::Pen)));
    assert_eq!(t.tool(), Tool::Pen(Pen::Pen));
    t.click_bar(Item::Pen(PenItem::Preset(1)));
    let want = t
        .hero
        .documents
        .live
        .editor
        .as_ref()
        .unwrap()
        .pen
        .current(Pen::Pen);
    t.drag(t.at(0.3, 0.5), t.at(0.6, 0.55));
    let inks = t.inks();
    assert_eq!(inks.len(), 1, "o arrasto não desenhou");
    assert_eq!(inks[0].style().stroke, Some(want.color));
    assert_eq!(inks[0].style().stroke_width, want.width);
    assert_eq!(t.tool(), Tool::Pen(Pen::Pen), "a caneta fica na mão");
    assert!(
        t.find(node(Item::Pen(PenItem::Laser))).is_some(),
        "e o painel aberto"
    );
}

/// O marcador, e a cor e a espessura do painel mudam a predefinição ACTIVA dele.
#[test]
fn the_colour_and_width_of_the_panel_edit_the_active_preset() {
    let mut t = T::new();
    t.click_bar(Item::Tool(Tool::Pen(Pen::Pen)));
    t.click_bar(Item::Pen(PenItem::Kind(Pen::Highlighter)));
    t.click_bar(Item::Pen(PenItem::Color(Some(6))));
    t.click_bar(Item::Pen(PenItem::Width(4)));
    t.drag(t.at(0.3, 0.4), t.at(0.5, 0.4));
    let ink = t.inks().pop().expect("um traço");
    assert_eq!(ink.ink().unwrap().pen, Pen::Highlighter);
    assert_eq!(ink.style().stroke, Some(Rgba(HIGHLIGHTER_RGBA[6])));
    assert_eq!(ink.style().stroke_width, ph2d_board_edit::PEN_WIDTHS[4]);
}

/// `P` caneta (a última usada), `E` borracha, `K` laser.
#[test]
fn p_e_and_k_pick_the_pen_the_eraser_and_the_laser() {
    let mut t = T::new();
    t.press('e');
    assert_eq!(t.tool(), Tool::Eraser { precise: false });
    t.press('k');
    assert_eq!(t.tool(), Tool::Laser);
    t.press('p');
    assert_eq!(t.tool(), Tool::Pen(Pen::Pen));
    t.click_bar(Item::Pen(PenItem::Kind(Pen::Highlighter)));
    t.press('v');
    t.press('p');
    assert_eq!(
        t.tool(),
        Tool::Pen(Pen::Highlighter),
        "o P volta a pegar na última caneta"
    );
}

/// ⭐ O botão «Rascunho ↔ Final» sem selecção troca o quadro INTEIRO, e o que nasce depois nasce
/// à mão; outra vez, tudo volta a final.
#[test]
fn the_sketch_button_flips_the_whole_board_and_what_is_born_after_it() {
    let mut t = T::new();
    t.press('r');
    t.drag(t.at(0.2, 0.2), t.at(0.35, 0.35));
    t.press('v');
    t.key(BoardKey::Escape, NONE, None);
    assert!(!t.sketch());
    t.click_bar(Item::Sketch);
    assert!(t.sketch(), "o quadro passou a rascunho");
    assert!(t.elements().iter().all(|e| e.style().sketch));
    t.press('r');
    t.drag(t.at(0.5, 0.5), t.at(0.65, 0.65));
    assert!(
        t.elements().iter().all(|e| e.style().sketch),
        "a forma nova nasceu à mão"
    );
    t.press('v');
    t.key(BoardKey::Escape, NONE, None);
    // Outro botão antes: dois cliques seguidos no mesmo são um duplo-clique no relógio do teste.
    t.click_bar(Item::Tool(Tool::Select));
    t.click_bar(Item::Sketch);
    assert!(!t.sketch());
    assert!(t.elements().iter().all(|e| !e.style().sketch));
}

/// Um traço seleccionado mostra a barra de estilo DOS TRAÇOS; a espessura dela muda o traço e não o
/// estilo das próximas formas.
#[test]
fn a_selected_stroke_gets_the_ink_style_bar_and_its_width() {
    let mut t = T::new();
    t.press('p');
    let (a, b) = (t.at(0.3, 0.5), t.at(0.6, 0.5));
    t.drag(a, b);
    t.press('v');
    let mid = ((a.0 + b.0) / 2.0, a.1);
    t.drag(mid, mid);
    assert!(
        t.find(node(Item::Pen(PenItem::InkWidth(3)))).is_some(),
        "a barra dos traços não abriu"
    );
    let before = t
        .hero
        .documents
        .live
        .editor
        .as_ref()
        .unwrap()
        .style
        .stroke_width;
    t.click_bar(Item::Pen(PenItem::InkWidth(3)));
    assert_eq!(
        t.inks()[0].style().stroke_width,
        ph2d_board_edit::PEN_WIDTHS[3]
    );
    let after = t
        .hero
        .documents
        .live
        .editor
        .as_ref()
        .unwrap()
        .style
        .stroke_width;
    assert_eq!(
        after, before,
        "as próximas formas não herdam a espessura da caneta"
    );
}

/// Um quadro que JÁ está em rascunho (aberto de um ficheiro, ou outra aba) faz nascer as formas à
/// mão — o editor é partilhado entre quadros, e o modo vem do quadro a cada clique.
#[test]
fn a_board_already_in_sketch_makes_new_shapes_sketched() {
    let mut t = T::new();
    t.hero.documents.active_board_mut().unwrap().sketch = true;
    t.press('r');
    t.drag(t.at(0.2, 0.2), t.at(0.35, 0.35));
    let els = t.elements();
    assert_eq!(els.len(), 1);
    assert!(
        els[0].style().sketch,
        "a forma nasceu final num quadro em rascunho"
    );
}

impl T {
    /// ⭐ Um clique num botão das barras PELA ORDEM DA SHELL (`input_dispatch::on_mouse_input`): o
    /// carregar e o largar passam PRIMEIRO pelo quadro (`board_view::pointer`), e só o que ele não
    /// tomar chega à barra. (O `click_bar` fala directamente com a barra e salta esse passo — por isso
    /// nunca viu o defeito de 07/10.)
    fn shell_click_bar(&mut self, item: Item) {
        let id = node(item);
        let (x, y) = self
            .find(id)
            .unwrap_or_else(|| panic!("{item:?} não está pintado"));
        let on_canvas =
            self.hero.chrome_panel_at(x, y).is_none() && self.hero.chrome_hit(x, y).is_none();
        assert!(!on_canvas, "um botão da barra não é quadro");
        let arena = Bump::new();
        for kind in [PointerKind::Down, PointerKind::Up] {
            let (h, ts, c) = (&mut self.hero, &mut self.ts, &mut self.clock);
            let took =
                board_view::pointer(h, inp(ts, c), kind, PointerButton::Primary, x, y, on_canvas);
            if !took {
                let ev = self.hero.handle_pointer(ptr(kind, x, y), &arena).to_vec();
                for e in ev {
                    self.hero.apply_event(e);
                }
            }
        }
        self.paint();
    }

    /// Carregar no quadro e arrastar SEM largar — o largar perdeu-se (foi para outro, ou caiu fora).
    fn press_and_drag_without_release(&mut self, a: (f32, f32), b: (f32, f32)) {
        let (h, ts, c) = (&mut self.hero, &mut self.ts, &mut self.clock);
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
        board_view::pointer_move(h, inp(ts, c), (a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        board_view::pointer_move(h, inp(ts, c), b.0, b.1);
        self.paint();
    }
}

/// O caminho da shell, sem nada perdido: caneta → desenhar → outra ferramenta.
#[test]
fn after_drawing_with_the_pen_another_tool_is_one_click_away() {
    let mut t = T::new();
    t.shell_click_bar(Item::Tool(Tool::Pen(Pen::Pen)));
    t.drag(t.at(0.3, 0.5), t.at(0.6, 0.55));
    t.shell_click_bar(Item::Tool(Tool::Select));
    assert_eq!(t.tool(), Tool::Select);
}

/// ⭐ Report do dono (07/10): *«se uma ferramenta como a caneta estiver seleccionada, não consigo
/// clicar directamente noutra ferramenta até apertar Esc»*. Um gesto cujo largar se perdeu ficava
/// PENDURADO, e o largar do clique seguinte ia para ele — o botão nunca recebia o clique. Agora o
/// carregar num botão fecha o gesto (o traço fica, num passo de desfazer) e o clique é do botão; e
/// o rato a passear depois já não prolonga o traço.
#[test]
fn a_hanging_gesture_never_swallows_a_click_on_another_tool() {
    for tool in [
        Item::Tool(Tool::Pen(Pen::Pen)),
        Item::Pen(PenItem::Eraser(false)),
        Item::Pen(PenItem::Laser),
        Item::Tool(Tool::Shape(ShapeType::Rectangle)),
        Item::Tool(Tool::Shape(ShapeType::Sticky)),
    ] {
        let mut t = T::new();
        t.shell_click_bar(Item::Tool(Tool::Pen(Pen::Pen)));
        if tool != Item::Tool(Tool::Pen(Pen::Pen)) {
            t.shell_click_bar(tool);
        }
        t.press_and_drag_without_release(t.at(0.3, 0.5), t.at(0.6, 0.55));
        t.shell_click_bar(Item::Tool(Tool::Select));
        assert_eq!(
            t.tool(),
            Tool::Select,
            "{tool:?}: o clique no Select foi engolido"
        );
        let ed = t.hero.documents.live.editor.as_ref().unwrap();
        assert!(!ed.is_busy(), "{tool:?}: o gesto continua pendurado");
    }
    // A caneta: o traço pendurado fica (um passo), e mexer o rato depois não o prolonga.
    let mut t = T::new();
    t.shell_click_bar(Item::Tool(Tool::Pen(Pen::Pen)));
    t.press_and_drag_without_release(t.at(0.3, 0.5), t.at(0.6, 0.55));
    t.shell_click_bar(Item::Tool(Tool::Select));
    let points = |t: &T| t.inks()[0].ink().unwrap().points.len();
    let n = points(&t);
    assert_eq!(t.inks().len(), 1, "o traço ficou");
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    board_view::pointer_move(h, inp(ts, c), 900.0, 700.0);
    assert_eq!(points(&t), n, "o rato solto prolongou o traço");
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert!(t.inks().is_empty(), "UM passo desfaz o traço");
}

/// ⭐ **Nenhum botão das barras do quadro mora numa zona que a SHELL trata antes delas** — a faixa
/// das réguas (o gesto das guias), a costura e o botão de reabrir de uma doca: são geométricos e
/// correm ANTES do clique de chrome (`input_dispatch::on_mouse_input`). Report do dono (07/10):
/// *«o Select é inseleccionável; toda a barra tem problemas»* — com as réguas ligadas, a faixa
/// invisível de 20 px cobria a borda da barra, e uma guia da cena roubava o clique em todo o
/// comprimento dela. A cura: um quadro activo não tem réguas (`rulers_live`).
#[test]
fn no_shell_gesture_zone_sits_on_a_board_bar_button() {
    let mut t = T::new();
    t.hero.view.rulers_visible = true;
    t.paint();
    assert!(
        !t.hero.rulers_live(),
        "um quadro activo não tem réguas — nem a faixa nem as guias"
    );
    let mut rects = Vec::new();
    for open in [
        None,
        Some(Item::Tool(Tool::Pen(Pen::Pen))),
        Some(Item::Tool(Tool::Shape(ShapeType::Sticky))),
        Some(Item::MoreShapes),
    ] {
        if let Some(it) = open {
            t.click_bar(it);
        }
        let area = crate::screens::hero::board_view::area(t.hero.last_layout.as_ref().unwrap());
        rects.extend(toolbar_rects(area));
        rects.extend(
            super::flyout_rects(area)
                .into_iter()
                .filter(|_| open == Some(Item::Tool(Tool::Shape(ShapeType::Sticky)))),
        );
        rects.extend(
            super::pen::flyout_rects(area)
                .into_iter()
                .filter(|_| open == Some(Item::Tool(Tool::Pen(Pen::Pen)))),
        );
        rects.extend(
            shapes_rects(area)
                .into_iter()
                .filter(|_| open == Some(Item::MoreShapes)),
        );
    }
    assert!(
        rects.len() > 40,
        "a régua cobre as barras e os painéis: {}",
        rects.len()
    );
    let layout = t.hero.last_layout.clone().unwrap();
    for (it, r) in rects {
        for p in [
            (r.x + 1.0, r.y + 1.0),
            (r.x + r.w / 2.0, r.y + r.h / 2.0),
            (r.x + r.w - 1.0, r.y + r.h - 1.0),
        ] {
            assert!(
                layout.dock_seam_at(p).is_none(),
                "{it:?} em {p:?}: a costura da doca rouba-o"
            );
            assert!(
                layout.dock_reopen_at(p).is_none(),
                "{it:?} em {p:?}: o reabrir da doca rouba-o"
            );
            let band = t.hero.rulers_live() && crate::ruler::hit(t.hero.last_canvas, p).is_some();
            assert!(!band, "{it:?} em {p:?}: a faixa da régua rouba-o");
        }
    }
    // E a cena continua com as réguas do artista.
    t.hero.documents.activate(None);
    assert!(t.hero.rulers_live());
}
