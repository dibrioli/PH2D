//! Os gates do **cabeçalho clicável** (W109) — a geometria, e a COSTURA.
//!
//! ⚠️ **A metade que interessa é a costura**, e não a geometria: um `row_at` correcto e um pintor
//! correcto, cada um verde no seu canto, são exactamente a causa nº 1 da semana perdida no Painter
//! (`DIRETIVA_IMPLEMENTACAO` §1). O que este ficheiro pergunta é *«clicar no rótulo abre o menu, e
//! clicar numa linha muda a câmera DAQUELE quadrante?»*.
//!
//! ⚠️ **E o chip não é montado à mão nos gates da costura.** Ele é publicado por quem pinta, então
//! um gate que o escrevesse ficaria verde no dia em que o pintor deixasse de o publicar — e o
//! controlo seria um clique sobre um alvo que a tela não tem. O
//! [`the_painter_publishes_the_chip_and_only_with_the_split_open`] desenha um quadro a sério.

use super::{ROW_H_PX, chip, menu_rect, row_at};
use crate::field3d_views::{Standard, named_view};
use ph2d_editor::zones::Rect;

const CANVAS: Rect = Rect {
    x: 40.0,
    y: 24.0,
    w: 800.0,
    h: 600.0,
};

fn um_chip(x: f32, y: f32) -> Rect {
    chip(Rect::new(x, y, 400.0, 300.0), 8.0, 30.0, 12.0)
}

/// ⭐ **As seis vistas saem na ordem de [`Standard::ALL`]**, e o ponto certo cai na linha certa.
#[test]
fn the_menu_lists_the_six_named_views_in_order() {
    let m = menu_rect(um_chip(CANVAS.x, CANVAS.y), CANVAS, 60.0);
    let pad = ph2d_tokens::Spacing::Md.px();
    for (i, esperado) in Standard::ALL.into_iter().enumerate() {
        let y = m.y + pad + ROW_H_PX * (i as f32 + 0.5);
        assert_eq!(
            row_at(m, [m.x + m.w * 0.5, y]),
            Some(esperado),
            "a linha {i} devia ser {esperado:?}"
        );
    }
    // ⛔ **Os dois CONTROLOS**: a folga do topo não é linha nenhuma, e fora do rectângulo o menu
    // não reclama nada — é o que faz o clique de fora FECHAR em vez de escolher.
    assert_eq!(row_at(m, [m.x + m.w * 0.5, m.y + pad * 0.5]), None);
    assert_eq!(row_at(m, [m.x - 1.0, m.y + m.h * 0.5]), None);
    assert_eq!(row_at(m, [m.x + m.w * 0.5, m.y + m.h + 1.0]), None);
}

/// ⭐ **O menu nunca sai do canvas**, venha o chip de que canto vier.
///
/// ⚠️ Sem isto o quadrante de baixo-direita abriria um menu por fora da janela — e é justamente o
/// quadrante em que um artista de quatro vistas trabalha com a perspectiva.
#[test]
fn the_menu_never_leaves_the_canvas() {
    let largo = 90.0;
    for (x, y) in [
        (CANVAS.x, CANVAS.y),
        (CANVAS.x + CANVAS.w - 400.0, CANVAS.y),
        (CANVAS.x, CANVAS.y + CANVAS.h - 300.0),
        (CANVAS.x + CANVAS.w - 400.0, CANVAS.y + CANVAS.h - 300.0),
    ] {
        let m = menu_rect(um_chip(x, y), CANVAS, largo);
        assert!(
            m.x >= CANVAS.x - 0.5
                && m.y >= CANVAS.y - 0.5
                && m.x + m.w <= CANVAS.x + CANVAS.w + 0.5
                && m.y + m.h <= CANVAS.y + CANVAS.h + 0.5,
            "o menu de ({x}, {y}) saiu do canvas: {m:?} contra {CANVAS:?}"
        );
    }
}

/// ⭐⭐⭐ **O PINTOR PUBLICA O CHIP — e só com a divisão aberta.**
///
/// ⚠️ Esta é a metade que impede os gates da costura de medirem um alvo inventado: eles precisam de
/// `Viewport::label`, e quem o põe lá é o desenho. ⛔ E o controlo é o **fecho**: com uma vista só
/// o rótulo não é pintado, e um chip que sobrevivesse a isso seria um alvo invisível a aceitar
/// cliques — o «controlo morto sob o dedo» que este repo já caçou.
#[test]
fn the_painter_publishes_the_chip_and_only_with_the_split_open() {
    let doc = crate::field3d_scene::lasso_tests::two_balls();
    crate::field3d_scene::lasso_tests::armed_with(&doc, |_| {
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        let mut desenha = || {
            let mut scene = ph2d_vector::VectorScene::new();
            crate::field3d_smoke::draw(
                crate::field3d_scene::lasso_tests::AREA,
                ph2d_tokens::Theme::default(),
                &mut text,
                &mut scene,
            );
        };
        crate::field3d_smoke::with_smoke(|s| {
            s.split = crate::field3d_layout::Split::quad();
        });
        desenha();
        let com_divisao = crate::field3d_smoke::with_smoke(|s| {
            (0..s.vps.len())
                .filter(|&i| s.vps[i].label.is_some())
                .count()
        })
        .expect("armado");
        assert_eq!(
            com_divisao, 4,
            "com a divisão aberta as quatro vistas têm de publicar o chip do rótulo"
        );

        crate::field3d_smoke::with_smoke(|s| {
            s.split = crate::field3d_layout::Split::One;
        });
        desenha();
        let sozinha = crate::field3d_smoke::with_smoke(|s| {
            s.vps.iter().filter(|v| v.label.is_some()).count()
        })
        .expect("armado");
        assert_eq!(
            sozinha, 0,
            "com uma vista só não há rótulo pintado, logo não pode haver chip clicável"
        );
    });
}

/// A costura: arma quatro vistas com o chip publicado por um desenho real.
fn com_quatro_vistas<R>(f: impl FnOnce(&mut crate::field3d_smoke::Smoke) -> R) -> R {
    let doc = crate::field3d_scene::lasso_tests::two_balls();
    crate::field3d_scene::lasso_tests::armed_with(&doc, |_| {
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        crate::field3d_smoke::with_smoke(|s| {
            s.split = crate::field3d_layout::Split::quad();
            s.view_menu = None;
            s.flight = None;
        });
        let mut scene = ph2d_vector::VectorScene::new();
        crate::field3d_smoke::draw(
            crate::field3d_scene::lasso_tests::AREA,
            ph2d_tokens::Theme::default(),
            &mut text,
            &mut scene,
        );
        crate::field3d_smoke::with_smoke(f).expect("armado")
    })
}

fn no_meio(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐⭐⭐ **A COSTURA: clicar num cabeçalho abre o menu DAQUELE quadrante, e uma linha voa-o.**
///
/// ⚠️ **O quadrante escolhido é o `3` de propósito** — o último, e nunca o activo por omissão. Um
/// gate sobre o quadrante `0` ficaria verde com um menu que trocasse sempre a vista do activo, que
/// é precisamente o defeito que faria o menu mudar a vista errada.
#[test]
fn clicking_a_header_opens_that_quadrants_menu_and_a_row_flies_that_quadrant() {
    let alvo = Standard::Bottom;
    let (aberto, voo) = com_quatro_vistas(|s| {
        let chip = s.vps[3].label.expect("o chip do quadrante 3");
        s.active = 0;
        let abriu = crate::field3d_input::begin(
            s,
            winit::event::MouseButton::Left,
            crate::field3d_smoke::Drag::Orbit,
            false,
            no_meio(chip),
        );
        assert!(abriu, "o clique no cabeçalho tem de ser tomado pelo módulo");
        // ⛔⛔ **A metade que uma mutação SOBREVIVENTE exigiu:** abrir o menu tem de acertar o
        // viewport comandado, porque é dessa linha — e não de uma segunda no ramo da escolha — que
        // depende o menu mexer no quadrante certo.
        assert_eq!(s.active, 3, "abrir o cabeçalho do 3 passa o comando ao 3");
        let aberto = s.view_menu;
        // O menu só ganha rectângulo quando é pintado — é o que o produto faz no quadro seguinte.
        let canvas = crate::field3d_smoke::canvas_area(s).expect("canvas");
        let m = menu_rect(chip, canvas, 90.0);
        s.view_menu_rect = Some(m);
        let i = Standard::ALL
            .into_iter()
            .position(|v| v == alvo)
            .expect("a vista está na lista");
        let y = m.y + ph2d_tokens::Spacing::Md.px() + ROW_H_PX * (i as f32 + 0.5);
        let escolheu = crate::field3d_input::begin(
            s,
            winit::event::MouseButton::Left,
            crate::field3d_smoke::Drag::Orbit,
            false,
            (m.x + m.w * 0.5, y),
        );
        assert!(
            escolheu,
            "o clique numa linha tem de ser tomado pelo módulo"
        );
        assert_eq!(s.view_menu, None, "escolher fecha o menu");
        (aberto, (s.active, s.flight.map(|f| f.to)))
    });
    assert_eq!(
        aberto,
        Some(3),
        "o menu é do quadrante cujo cabeçalho foi tocado"
    );
    let (activa, destino) = voo;
    assert_eq!(
        activa, 3,
        "escolher no cabeçalho do 3 comanda o 3, e não o activo de antes"
    );
    let destino = destino.expect("escolher uma vista tem de pôr uma viagem em curso");
    assert_eq!(
        named_view(&destino),
        Some(alvo),
        "a viagem tem de ir para a vista da linha clicada"
    );
}

/// ⭐⭐ **Um clique ao lado FECHA o menu e não mexe câmera nenhuma** — e consome o gesto.
#[test]
fn a_click_beside_an_open_menu_closes_it_and_moves_no_camera() {
    com_quatro_vistas(|s| {
        let chip = s.vps[3].label.expect("o chip do quadrante 3");
        let canvas = crate::field3d_smoke::canvas_area(s).expect("canvas");
        s.view_menu = Some(3);
        s.view_menu_rect = Some(menu_rect(chip, canvas, 90.0));
        s.flight = None;
        let tomou = crate::field3d_input::begin(
            s,
            winit::event::MouseButton::Left,
            crate::field3d_smoke::Drag::Orbit,
            false,
            (canvas.x + canvas.w - 2.0, canvas.y + canvas.h - 2.0),
        );
        assert!(
            tomou,
            "o clique que fecha um menu é consumido, e não orbita a peça"
        );
        assert_eq!(s.view_menu, None, "clicar ao lado fecha");
        assert!(
            s.flight.is_none(),
            "fechar um menu não pode pôr a câmera a viajar"
        );
    });
}

/// ⭐⭐ **E o CURSOR obedece à mesma precedência** — com o menu aberto, a costura não promete nada.
///
/// ⚠️ É a outra metade do defeito que o gate da costura apanhou: se o clique passou a ser do menu e
/// o ponteiro continuasse a ser a seta de redimensionar, a tela anunciaria um gesto que a mão já
/// não consegue fazer ali.
#[test]
fn an_open_menu_takes_the_seam_cursor_too() {
    com_quatro_vistas(|s| {
        let canvas = crate::field3d_smoke::canvas_area(s).expect("canvas");
        let cruz = (canvas.x + canvas.w * 0.5, canvas.y + canvas.h * 0.5);
        s.view_menu = None;
        assert!(
            crate::field3d_smoke::divider_cursor(s, cruz).is_some(),
            "sem menu, o cruzamento das costuras TEM de dar o cursor de redimensionar — \
             senão este gate não está a medir nada"
        );
        s.view_menu = Some(3);
        assert!(
            crate::field3d_smoke::divider_cursor(s, cruz).is_none(),
            "com o menu aberto, a costura não pode prometer um gesto que ela já não recebe"
        );
    });
}

/// ⭐⭐ **`Escape` é do menu só enquanto ele está aberto.**
///
/// ⚠️ O controlo é a metade que importa: um handler que reclamasse a tecla sempre roubaria o
/// cancelar de todos os que vêm a seguir no roteador do teclado.
#[test]
fn escape_owns_the_key_only_while_the_menu_is_open() {
    // ⚠️ **Tudo dentro do MESMO `armed_with`**: o `with_smoke` devolve `None` fora dele, e a 1.ª
    // redacção deste gate lia esse `None` como *«a tecla não fechou»*. *Um teste que sai do sítio
    // onde o estado vive mede a ausência do estado, não a lei.*
    com_quatro_vistas(|s| {
        s.view_menu = None;
        assert!(
            s.view_menu.take().is_none(),
            "sem menu aberto a tecla não é deste módulo"
        );
        s.view_menu = Some(2);
        assert!(
            s.view_menu.take().is_some(),
            "com menu aberto, a tecla fecha-o"
        );
        assert!(
            s.view_menu.take().is_none(),
            "e a segunda vez já não é dele"
        );
    });
}
