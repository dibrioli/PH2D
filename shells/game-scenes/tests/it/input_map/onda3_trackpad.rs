//! Onda 3, item 7: o deslizar horizontal do trackpad nas outras abas.
//!
//! A onda 2 mandava o `dx` a toda aba como Shift+roda; Sprite, SFX, Planner (calendário e destino), as zonas
//! do Code e do Kanban que ignoram o Shift, e a própria lista do Input Map rolavam então na VERTICAL com os
//! dedos a andar de lado. A lei: o `dx` vai à `wheel_sideways` (por omissão nada) e o `dy` à roda de sempre,
//! cada eixo uma vez. Uma aba-espiã prova a porta de TODAS as famílias que não pedem o lado; o Input Map, a
//! única que o pede, prova-se pela porta real.

use std::cell::RefCell;
use std::rc::Rc;

use super::Mesa;
use super::lista::onze;
use ph2d_a11y::NodeId;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::doc_surface::{
    Compose, DocTab, DocumentSurface, KeyInput, KeyOutcome, PointerInput, SurfaceTab,
};
use ph2d_editor_core::screens::hero::board_view::{BoardScroll, scroll};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Uma família que NÃO pede o lado (o caso de Kanban, Code, Planner, Sprite, SFX e da aba da física): conta
/// cada roda que lhe chega, `(dy, shift)`.
struct Espia(Rodas);

impl DocumentSurface for Espia {
    fn tabs(&self) -> Vec<DocTab> {
        vec![DocTab {
            id: 1,
            title: String::new(),
            dirty: false,
        }]
    }
    fn paint(&mut self, _: u64, _: Rect, _: &mut VectorScene, _: &mut TextSystem, _: &Theme) {}
    fn pointer(
        &mut self,
        _: u64,
        _: PointerInput<'_>,
        _: PointerKind,
        _: PointerButton,
        _: f32,
        _: f32,
    ) -> bool {
        false
    }
    fn pointer_move(&mut self, _: u64, _: PointerInput<'_>, _: f32, _: f32) -> bool {
        false
    }
    fn wheel(&mut self, _: u64, _: f32, _: f32, dy: f32, shift: bool) -> bool {
        self.0.borrow_mut().push((dy, shift));
        true
    }
    fn key(&mut self, _: u64, _: &mut TextSystem, _: KeyInput<'_>) -> KeyOutcome {
        KeyOutcome::default()
    }
    fn compose(&mut self, _: u64, _: Compose<'_>) -> bool {
        false
    }
    fn close(&mut self, _: u64) {}
    fn undo(&mut self, _: u64, _: bool) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// O que a espia ouviu: cada roda (dy, shift).
type Rodas = Rc<RefCell<Vec<(f32, bool)>>>;

fn com_espia() -> (HeroScreen, Rodas) {
    let mut hero = HeroScreen::new(NodeId(1));
    let rodas = Rc::new(RefCell::new(Vec::new()));
    hero.documents
        .install_surface(Box::new(Espia(rodas.clone())));
    hero.documents
        .activate_surface(SurfaceTab { family: 0, id: 1 });
    (hero, rodas)
}

/// O que a família ouve de um gesto (e esvazia a conta).
fn ouve(hero: &mut HeroScreen, rodas: &Rodas, s: BoardScroll) -> Vec<(f32, bool)> {
    scroll(hero, 500.0, 300.0, s); // LITERAL-PX-OK: um ponto qualquer da área
    std::mem::take(&mut *rodas.borrow_mut())
}

/// ⭐ A família que não pede o lado ouve EXACTAMENTE o que ouvia antes da onda 2: o `dy` do gesto com o Shift
/// do teclado, uma vez — nunca o `dx` disfarçado de Shift+roda.
#[test]
fn o_deslizar_de_lado_nao_chega_a_familia_que_nao_o_pede() {
    let (mut hero, rodas) = com_espia();
    assert_eq!(
        ouve(&mut hero, &rodas, BoardScroll::Pan([40.0, 0.0])),
        vec![(0.0, false)],
        "de lado puro"
    );
    assert_eq!(
        ouve(&mut hero, &rodas, BoardScroll::Pan([-40.0, 0.0])),
        vec![(0.0, false)],
        "de lado, ao contrário"
    );
    assert_eq!(
        ouve(&mut hero, &rodas, BoardScroll::Pan([30.0, -20.0])),
        vec![(-20.0, false)],
        "diagonal: só o dy, uma vez"
    );
    assert_eq!(
        ouve(&mut hero, &rodas, BoardScroll::Pan([0.0, -20.0])),
        vec![(-20.0, false)],
        "vertical puro"
    );
    assert_eq!(
        ouve(&mut hero, &rodas, BoardScroll::Wheel(-16.0)),
        vec![(-16.0, false)],
        "a roda do rato"
    );
    assert_eq!(
        ouve(&mut hero, &rodas, BoardScroll::Pinch(1.5)),
        vec![(0.0, false)],
        "a pinça"
    );
    hero.store.set_shift_held(true);
    assert_eq!(
        ouve(&mut hero, &rodas, BoardScroll::Pan([40.0, 0.0])),
        vec![(0.0, true)],
        "com o Shift premido"
    );
    assert_eq!(
        ouve(&mut hero, &rodas, BoardScroll::Wheel(-16.0)),
        vec![(-16.0, true)],
        "Shift+roda de sempre"
    );
}

/// Sem aba de documento activa (a Cena, o Pixel, o quadro) o caminho da roda não muda: nada é tomado aqui.
#[test]
fn sem_aba_de_documento_o_deslizar_de_lado_segue_para_a_cena() {
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(!scroll(
        &mut hero,
        500.0,
        300.0,
        BoardScroll::Pan([40.0, 0.0])
    )); // LITERAL-PX-OK: ponto de teste
}

fn area(h: f32) -> Rect {
    Rect::new(297.0, 28.0, 360.0, h) // LITERAL-PX-OK: estreita (o teclado rola de lado)
}

fn pinta(view: View, a: Rect) -> Mesa {
    let mut m = Mesa::aberta(view);
    m.hero.input_map = onze();
    m.frame();
    if view == View::Keyboard {
        m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    }
    m.paint_in(a);
    m
}

fn pan(m: &mut Mesa, a: Rect, d: [f32; 2]) {
    scroll(
        &mut m.hero,
        a.x + a.w * 0.5,
        a.y + a.h * 0.5,
        BoardScroll::Pan(d),
    );
    m.paint_in(a);
}

/// `(hscroll do teclado, rolagem vertical da lista das acções do Keyboard)`.
fn rolagens(m: &Mesa) -> (f32, f32) {
    let kb = &m.tab().doc.kb;
    (kb.hscroll, kb.scroll)
}

/// ⭐ Keyboard e Try it (que têm Shift+roda = o teclado de lado): o deslizar de lado é o Shift+roda da mesma
/// grandeza; o vertical puro não mexe no teclado; o diagonal aplica cada eixo uma vez.
#[test]
fn no_teclado_o_deslizar_de_lado_e_o_shift_roda() {
    let a = area(604.0); // LITERAL-PX-OK: a altura da aba real a 1280×720
    for view in [View::Keyboard, View::TryIt] {
        let mut gesto = pinta(view, a);
        let mut roda = pinta(view, a);
        assert!(
            gesto.tab().doc.kb.hbar.is_some(),
            "{view:?}: a 360 px o teclado rola de lado"
        );

        pan(&mut gesto, a, [-40.0, 0.0]);
        roda.tab_mut().doc.wheel_shift(-40.0, true);
        roda.paint_in(a);
        assert!(
            rolagens(&gesto).0 > 0.0,
            "{view:?}: o deslizar de lado rolou o teclado"
        );
        assert_eq!(
            rolagens(&gesto),
            rolagens(&roda),
            "{view:?}: de lado = Shift+roda"
        );

        let h = rolagens(&gesto).0;
        pan(&mut gesto, a, [0.0, -40.0]);
        roda.tab_mut().doc.wheel_shift(-40.0, false);
        roda.paint_in(a);
        assert_eq!(
            rolagens(&gesto).0,
            h,
            "{view:?}: o vertical puro não rola o teclado"
        );
        assert_eq!(
            rolagens(&gesto),
            rolagens(&roda),
            "{view:?}: vertical = a roda de sempre"
        );

        // Diagonal: o lado uma vez (não somado ao vertical), o vertical uma vez.
        pan(&mut gesto, a, [-30.0, 20.0]);
        roda.tab_mut().doc.wheel_shift(-30.0, true);
        roda.tab_mut().doc.wheel_shift(20.0, false);
        roda.paint_in(a);
        assert_eq!(
            rolagens(&gesto),
            rolagens(&roda),
            "{view:?}: diagonal = cada eixo uma vez"
        );
    }
}

/// A lista das acções (List) não tem Shift+roda: o deslizar de lado não a rola (a onda 2 rolava-a na vertical).
#[test]
fn na_lista_o_deslizar_de_lado_nao_muda_nada() {
    let a = area(220.0); // LITERAL-PX-OK: baixa, para a lista transbordar
    let mut m = pinta(View::List, a);
    assert!(m.tab().doc.list.scroll_max > 40.0, "a lista transborda");
    let antes = m.tab().doc.list.scroll;
    pan(&mut m, a, [40.0, 0.0]);
    assert_eq!(m.tab().doc.list.scroll, antes, "de lado não rola a lista");
    pan(&mut m, a, [-40.0, 0.0]);
    assert_eq!(m.tab().doc.list.scroll, antes, "nem ao contrário");
    pan(&mut m, a, [0.0, 40.0]);
    assert!(
        m.tab().doc.list.scroll > antes,
        "o vertical continua a rolar"
    );
}
