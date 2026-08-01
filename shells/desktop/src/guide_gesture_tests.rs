//! Gates do gesto das guias — todos sobre a política PURA, que é o que um harness sem janela
//! consegue dirigir.

use super::*;
use ph2d_editor::GridView;
use ph2d_editor::zones::Rect;
use ph2d_guides::GuideSet;

/// Canvas de 800×600 no canto, câmera na origem, 10 unidades de altura.
fn view() -> GridView {
    GridView {
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Rect::new(0.0, 0.0, 800.0, 600.0),
    }
}

/// Um press na régua de CIMA nasce uma guia HORIZONTAL, e a coordenada dela vem da posição
/// VERTICAL do cursor.
///
/// ⚠️ É o par cruzado que o `RulerAxis::spawns` existe para tornar explícito, aqui afirmado
/// pelo NÚMERO: mover o cursor na horizontal ao longo da régua de cima **não** muda onde a
/// guia pousa. A mutação que troca as duas coordenadas passa por qualquer gate que só olhe o
/// eixo do enum.
#[test]
fn a_press_on_the_top_ruler_spawns_a_horizontal_guide_at_the_cursors_height() {
    let v = view();
    let g = GuideSet::default();
    let GuidePress::Spawn(r, pos) = press_plan(&v, &g, true, (300.0, 10.0)) else {
        panic!("um press na faixa de cima tem de nascer uma guia");
    };
    assert_eq!(r, RulerAxis::Top);
    assert_eq!(r.spawns(), ph2d_guides::GuideAxis::Horizontal);

    let GuidePress::Spawn(_, pos2) = press_plan(&v, &g, true, (700.0, 10.0)) else {
        panic!("idem")
    };
    assert!(
        (pos - pos2).abs() < 1e-9,
        "andar 400 px na horizontal moveu a guia horizontal de {pos} para {pos2}"
    );
    // E o valor é o mundo sob a ALTURA do cursor, pela porta da régua da esquerda.
    let expected = ph2d_editor::ruler::world_at(&v, 10.0, RulerAxis::Left);
    assert!((pos - expected).abs() < 1e-9);
}

/// O espelho: a régua da esquerda nasce uma VERTICAL, e o valor vem do X do cursor.
#[test]
fn a_press_on_the_left_ruler_spawns_a_vertical_guide_at_the_cursors_x() {
    let v = view();
    let g = GuideSet::default();
    let GuidePress::Spawn(r, pos) = press_plan(&v, &g, true, (10.0, 300.0)) else {
        panic!("um press na faixa da esquerda tem de nascer uma guia");
    };
    assert_eq!(r, RulerAxis::Left);
    assert_eq!(r.spawns(), ph2d_guides::GuideAxis::Vertical);
    let expected = ph2d_editor::ruler::world_at(&v, 10.0, RulerAxis::Top);
    assert!((pos - expected).abs() < 1e-9);
}

/// Um press perto de uma guia posta a PEGA — e a régua devolvida é a que a governa, senão o
/// arrasto seguinte moveria a coordenada errada.
#[test]
fn a_press_near_a_placed_guide_grabs_it_with_the_ruler_that_governs_it() {
    let v = view();
    let mut g = GuideSet::default();
    // Uma vertical no mundo x=0, que na tela cai no meio (a câmera está na origem).
    g.push(ph2d_guides::Guide::vertical(0.0));
    let mid_x = 400.0;
    assert_eq!(
        press_plan(&v, &g, true, (mid_x, 300.0)),
        GuidePress::Grab(0, RulerAxis::Left),
        "uma guia VERTICAL é governada pela régua da ESQUERDA"
    );
    assert_eq!(
        press_plan(&v, &g, true, (mid_x + 200.0, 300.0)),
        GuidePress::Pass,
        "longe dela, o press segue o caminho de sempre"
    );
}

/// **O LOCK:** com as réguas fora nada é agarrável — e é o mesmo interruptor que se vê na
/// tela, então não há estado travado invisível.
#[test]
fn with_the_rulers_hidden_nothing_is_grabbable() {
    let v = view();
    let mut g = GuideSet::default();
    g.push(ph2d_guides::Guide::vertical(0.0));
    assert_eq!(press_plan(&v, &g, false, (400.0, 300.0)), GuidePress::Pass);
    assert_eq!(
        press_plan(&v, &g, false, (300.0, 10.0)),
        GuidePress::Pass,
        "nem a faixa da régua responde — ela nem está lá"
    );
}

/// Soltar sobre QUALQUER régua apaga; soltar no canvas guarda.
#[test]
fn releasing_over_a_ruler_deletes_and_releasing_on_the_canvas_keeps() {
    let v = view();
    assert!(release_deletes(&v, (300.0, 10.0)), "a faixa de cima apaga");
    assert!(
        release_deletes(&v, (10.0, 300.0)),
        "a da esquerda também, mesmo para uma guia horizontal"
    );
    assert!(!release_deletes(&v, (400.0, 300.0)), "o miolo guarda");
}

/// Um press FORA do canvas não é assunto das guias — ele pertence ao chrome.
#[test]
fn a_press_outside_the_canvas_is_not_ours() {
    let v = GridView {
        canvas: Rect::new(100.0, 50.0, 600.0, 400.0),
        ..view()
    };
    assert_eq!(
        press_plan(&v, &GuideSet::default(), true, (10.0, 10.0)),
        GuidePress::Pass
    );
}
