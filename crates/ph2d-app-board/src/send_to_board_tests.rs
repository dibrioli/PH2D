//! W15 — «Send to board…»: o pedido abre o modal, o clique REAL num quadro do modal levanta a escolha,
//! e a entrega põe o conteúdo no MEIO da vista desse quadro, com a aba dele activa — num quadro aberto
//! ou num novo; o que o quadro recusa não muda nada.

use super::*;
use bumpalo::Bump;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, board_view, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

const ASKING_ROW: NodeId = NodeId(100_901);

/// O SVG na forma que o exportador vectorial escreve (o prólogo e a nota antes do `<svg>`).
const SVG: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!--\n  PH2D — exportacao vectorial\n-->\n\
<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"-12 -6 24 12\" width=\"240\" height=\"120\">\n  \
<path fill=\"#ff0000\" d=\"M-10,5 L10,5 L10,-5 L-10,-5 Z\"/>\n</svg>\n";

struct T {
    hero: HeroScreen,
    text: TextSystem,
}

impl T {
    /// O hero desenhado, com quadros `names` e a aba da CENA activa (quem manda está na Hierarquia).
    fn new(names: &[&str]) -> Self {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut hero = HeroScreen::new(NodeId(1));
        for n in names {
            let b = hero.documents.create((*n).to_owned());
            register_board(&mut hero.store, b);
        }
        hero.documents.activate(None);
        let mut t = Self {
            hero,
            text: TextSystem::without_system_fonts(),
        };
        t.paint();
        t
    }

    fn paint(&mut self) {
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.text);
    }

    fn board_ids(&self) -> Vec<BoardId> {
        self.hero
            .documents
            .boards()
            .boards()
            .iter()
            .map(|b| b.id)
            .collect()
    }

    /// As imagens do quadro `b`.
    fn images(&self, b: BoardId) -> Vec<ph2d_board_model::Element> {
        let board = self.hero.documents.boards().get(b).expect("o quadro");
        board
            .doc
            .live_in_z_order()
            .into_iter()
            .filter(|e| e.image().is_some())
            .cloned()
            .collect()
    }

    /// Down + Up sobre o centro do alvo `id` que o pintor registou, pelo despacho do hero.
    fn click(&mut self, id: NodeId) {
        let vp = self.hero.last_viewport;
        let mut hits = Vec::new();
        let mut y = vp.y;
        while y < vp.y + vp.h {
            let mut x = vp.x;
            while x < vp.x + vp.w {
                if self.hero.hit_index.hit(x, y) == Some(id) {
                    hits.push((x, y));
                }
                x += 2.0;
            }
            y += 2.0;
        }
        assert!(!hits.is_empty(), "{id:?} não está pintado");
        let n = hits.len() as f32;
        let x = hits.iter().map(|p| p.0).sum::<f32>() / n;
        let y = hits.iter().map(|p| p.1).sum::<f32>() / n;
        let arena = Bump::new();
        let mut events: Vec<WidgetEvent> = Vec::new();
        for kind in [PointerKind::Down, PointerKind::Up] {
            let e = PointerEvent {
                x,
                y,
                pressure: 1.0,
                kind,
                source: PointerSource::Mouse,
                button: PointerButton::Primary,
                timestamp_ns: 0,
            };
            events.extend_from_slice(self.hero.handle_pointer(e, &arena));
        }
        for e in events {
            self.hero.apply_event(e);
        }
    }

    /// A escolha que o modal levantou (o que a shell drena).
    fn chosen(&mut self) -> Option<(NodeId, Option<u64>)> {
        self.hero.bus.drain().find_map(|a| match a {
            EditorAction::Hierarchy(HierRequest::SendToBoardChosen { row, board }) => {
                Some((row, board))
            }
            _ => None,
        })
    }
}

/// Um objecto só, sem pose.
fn one(name: &str, content: Option<Content>) -> Vec<Sent> {
    vec![Sent::new(name, content)]
}

fn red(w: u32, h: u32) -> Content {
    Content::Pixels(Pixels {
        size: [w, h],
        rgba: [255, 0, 0, 255].repeat((w * h) as usize),
    })
}

/// ⭐⭐ **O caminho inteiro do lado do quadro, pelo clique real:** o pedido abre o modal; o clique na
/// 2.ª linha escolhe o 2.º quadro; a entrega abre a aba dele e a imagem nasce no MEIO da vista, no
/// tamanho natural, seleccionada e num passo de desfazer. O 1.º quadro fica como estava.
#[test]
fn the_modal_pick_puts_the_picture_in_the_middle_of_that_boards_view() {
    let mut t = T::new(&["Moodboard", "Ideas"]);
    let b = t.board_ids();
    assert!(ask(&mut t.hero, ASKING_ROW, true).is_none());
    t.paint();
    t.click(ids::CTX_SEND_BOARD_ROWS[1]);
    let (row, board) = t.chosen().expect("o modal levantou a escolha");
    assert_eq!((row, board), (ASKING_ROW, Some(b[1].0)));
    let (got, left) =
        deliver(&mut t.hero, board, one("Hero", Some(red(120, 80)))).expect("entregue");
    assert_eq!(got, b[1]);
    assert!(left.is_none());
    assert_eq!(
        t.hero.documents.active(),
        Some(b[1]),
        "a aba do quadro não abriu"
    );
    t.paint();
    let imgs = t.images(b[1]);
    assert_eq!(imgs.len(), 1);
    assert_eq!([imgs[0].w, imgs[0].h], [120.0, 80.0]);
    let board = t.hero.documents.active_board().unwrap();
    let r = board_view::area(t.hero.last_layout.as_ref().unwrap());
    let area = [r.x, r.y, r.w, r.h].map(f64::from);
    let mid = board.camera.to_screen(area, imgs[0].center());
    let (cx, cy) = (area[0] + area[2] / 2.0, area[1] + area[3] / 2.0);
    assert!(
        (mid[0] - cx).abs() < 1e-6 && (mid[1] - cy).abs() < 1e-6,
        "{mid:?} ≠ meio"
    );
    assert!(t.images(b[0]).is_empty(), "o outro quadro recebeu");
    let file = board
        .file(imgs[0].image().unwrap().file)
        .expect("o ficheiro");
    assert_eq!((file.mime(), file.size), (Some("image/png"), [120, 80]));
}

/// ⭐ «New board» pelo clique real: nasce um quadro com o nome por omissão das abas, activo, com a
/// aba registada (clicável), e a imagem dentro.
#[test]
fn new_board_is_made_opened_and_receives_the_content() {
    let mut t = T::new(&["Moodboard"]);
    ask(&mut t.hero, ASKING_ROW, true);
    t.paint();
    t.click(ids::CTX_SEND_BOARD_NEW);
    let (_, board) = t.chosen().expect("a escolha");
    assert_eq!(board, None);
    let (got, _) = deliver(&mut t.hero, board, one("Hero", Some(red(10, 10)))).expect("entregue");
    let b = t.hero.documents.boards().get(got).expect("o quadro novo");
    assert_eq!(b.name, "Board 2");
    assert_eq!(t.hero.documents.active(), Some(got));
    assert_eq!(t.images(got).len(), 1);
    let tab = ph2d_editor_core::screens::hero::document_tabs::tab_node_id(got);
    assert!(
        t.hero.store.button_state(tab).is_some(),
        "a aba nova não é clicável"
    );
}

/// ⭐ O vector entra como SVG (nítido a qualquer zoom), com o tamanho natural do `<svg>` — a forma
/// que o exportador escreve, prólogo e nota incluídos.
#[test]
fn a_vector_goes_in_as_svg_at_its_natural_size() {
    let mut t = T::new(&["Moodboard"]);
    let b = t.board_ids()[0];
    deliver(
        &mut t.hero,
        Some(b.0),
        one("Shape", Some(Content::Svg(SVG.to_owned()))),
    )
    .expect("o SVG");
    let img = t.images(b).remove(0);
    let file = t
        .hero
        .documents
        .boards()
        .get(b)
        .unwrap()
        .file(img.image().unwrap().file)
        .unwrap();
    assert_eq!(
        (file.mime(), file.size),
        (Some("image/svg+xml"), [240, 120])
    );
    assert_eq!(
        &*file.bytes.0,
        SVG.as_bytes(),
        "os bytes como saíram do exportador"
    );
}

/// O que não é para um quadro diz porquê e não abre o modal; o que o quadro recusa, ou um quadro que
/// fechou entretanto, não cria quadro nem troca a aba.
#[test]
fn a_refusal_changes_nothing_and_says_why() {
    let mut t = T::new(&["Moodboard"]);
    assert!(ask(&mut t.hero, ASKING_ROW, false).is_some());
    assert!(
        t.hero.store.context_menu().is_none(),
        "abriu o modal para um esqueleto"
    );
    let bad = Content::Svg("<svg xmlns=\"http://www.w3.org/2000/svg\"><g".to_owned());
    assert!(deliver(&mut t.hero, None, one("Shape", Some(bad))).is_err());
    assert!(
        deliver(&mut t.hero, None, one("Flip", None)).is_err(),
        "nada a mostrar"
    );
    assert!(
        deliver(&mut t.hero, Some(999), one("Hero", Some(red(2, 2)))).is_err(),
        "quadro fechado"
    );
    assert_eq!(t.hero.documents.boards().boards().len(), 1);
    assert_eq!(t.hero.documents.active(), None);
}

/// ⭐ W16 — a SELECÇÃO inteira (a linha clicada dentro dela) vai numa leva: cada um com a pose dele,
/// os que não têm conteúdo nomeados no aviso; o resto entra.
#[test]
fn the_selection_goes_whole_each_with_its_pose_and_the_left_out_are_named() {
    let mut t = T::new(&["Moodboard"]);
    let b = t.board_ids()[0];
    assert_eq!(
        targets(7, &[3, 7, 9]),
        [3, 7, 9],
        "dentro: a selecção, pela ordem dela"
    );
    assert_eq!(targets(5, &[3, 7, 9]), [5], "fora: só a linha clicada");
    let turned = Pose {
        scale: [2.0, 0.5],
        angle: 0.3,
        flip: [true, false],
    };
    let items = vec![
        Sent {
            name: "Hero".into(),
            content: Ok(red(40, 20)),
            pose: turned,
        },
        Sent::new("Flip", None),
        Sent::new("Shape", Some(Content::Svg(SVG.to_owned()))),
        Sent::missing("Bones", "board.send.wrong_kind"),
    ];
    let (got, left) = deliver(&mut t.hero, Some(b.0), items).expect("entregue");
    assert_eq!(got, b);
    let left = left.expect("o aviso dos que ficaram de fora");
    assert!(
        left.message.contains("Flip") && left.message.contains("Bones"),
        "{}",
        left.message
    );
    assert!(!left.message.contains("Hero"));
    let imgs = t.images(b);
    assert_eq!(imgs.len(), 2);
    let hero = imgs
        .iter()
        .find(|e| e.image().is_some_and(|i| i.flip == [true, false]))
        .expect("a sprite com o espelho dela");
    assert!((hero.angle - 0.3).abs() < 1e-12);
    assert!(
        (hero.w / hero.h - (40.0 * 2.0) / (20.0 * 0.5)).abs() < 1e-9,
        "a escala de cada eixo"
    );
}

/// ⭐ W16 — a pose de uma sprite: o mundo (y para cima, ângulo anti-horário) para o quadro (y para
/// baixo, ângulo horário), à régua `ppm`; o espelho horizontal sem rodar 180°; a inclinação (que o
/// quadro não exprime) redesenha os píxeis; sem área não vai.
#[test]
fn a_sprite_goes_with_its_scene_pose() {
    let px = || red(4, 2);
    let pose = |m: [f64; 4]| posed_sprite(px(), m, [8.0, 4.0], 10.0).map(|(_, p)| p);
    // 4×2 píxeis num tamanho de 8×4 m a 10 px/m: cada píxel vale 20 px de quadro.
    let id = pose([1.0, 0.0, 0.0, 1.0]).unwrap();
    assert_eq!(
        (id.scale, id.angle, id.flip),
        ([20.0, 20.0], 0.0, [false; 2])
    );
    let (s, c) = 30f64.to_radians().sin_cos();
    let turned = pose([c, s, -s, c]).unwrap();
    assert!(
        (turned.angle + 30f64.to_radians()).abs() < 1e-12,
        "anti-horário no mundo = −θ"
    );
    assert!(turned.scale.iter().all(|k| (k - 20.0).abs() < 1e-9));
    let mirrored = pose([-1.0, 0.0, 0.0, 1.0]).unwrap();
    assert_eq!((mirrored.angle, mirrored.flip), (0.0, [true, false]));
    let upside = pose([1.0, 0.0, 0.0, -1.0]).unwrap();
    assert_eq!((upside.angle, upside.flip), (0.0, [false, true]));
    assert!(pose([0.0, 0.0, 0.0, 1.0]).is_none(), "escala 0");
    // Inclinada: os píxeis vêm redesenhados na caixa que ocupam, sem ângulo.
    let (content, p) = posed_sprite(px(), [1.0, 0.0, 0.5, 1.0], [8.0, 4.0], 10.0).unwrap();
    let Content::Pixels(baked) = content else {
        unreachable!()
    };
    assert_eq!((p.angle, p.flip), (0.0, [false; 2]));
    // A caixa: 80 de largo + o desvio de 0,5 × 40 de alto = 100 × 40.
    let world = baked.size.map(|v| f64::from(v) * p.scale[0]);
    assert!(
        (world[0] - 100.0).abs() <= 1.0 && (world[1] - 40.0).abs() <= 1.0,
        "{world:?}"
    );
    let opaque = baked.rgba.chunks(4).filter(|q| q[3] == 255).count();
    assert!(
        opaque > 0 && opaque < baked.rgba.len() / 4,
        "um paralelogramo dentro da caixa"
    );
}
