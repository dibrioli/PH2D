//! D11 — o fundo do Kanban: cada variante pinta coisa diferente, o padrão é UM preenchimento só dos
//! ladrilhos do rectângulo, de longe vira a média, e o cabeçalho escurece o que o fundo claro pede.

use super::*;
use ph2d_kanban_model::{FileId, Rgb};
use ph2d_vector::DrawOp;

fn record(bg: Background, scale: f64, image: Option<BgImage<'_>>) -> Vec<DrawOp> {
    let mut scene = VectorScene::new();
    scene.record();
    paint_kanban_bg(
        &mut scene,
        Theme::Forge,
        &KanbanBg {
            bg,
            rect: Rect::new(0.0, 0.0, 480.0, 320.0),
            to_scene: Affine::IDENTITY,
            screen_scale: scale,
            image,
        },
    );
    scene.take_record()
}

/// Uma impressão das operações (o tipo, o pincel e quantos elementos de caminho).
fn print(ops: &[DrawOp]) -> String {
    format!("{ops:?}")
}

fn pattern(kind: PatternKind) -> Background {
    Background::Pattern(Pattern::new(kind))
}

#[test]
fn every_background_kind_paints_something_different() {
    let px = Arc::new(vec![255_u8; 4 * 4 * 4]);
    let img = BgImage {
        rgba: &px,
        size: [4, 4],
        brightest: 1.0,
        mean: 1.0,
    };
    let mut all = vec![
        print(&record(Background::Color(Hue::Blue), 1.0, None)),
        print(&record(Background::Color(Hue::Green), 1.0, None)),
        print(&record(Background::Custom(Rgb([250, 240, 200])), 1.0, None)),
        print(&record(Background::Gradient(1), 1.0, None)),
        print(&record(Background::Gradient(3), 1.0, None)),
        print(&record(Background::Image(FileId::of(b"x")), 1.0, Some(img))),
    ];
    all.extend(
        PatternKind::ALL
            .into_iter()
            .map(|k| print(&record(pattern(k), 1.0, None))),
    );
    let n = all.len();
    all.sort();
    all.dedup();
    assert_eq!(all.len(), n, "cada fundo é uma pintura própria");
}

#[test]
fn a_pattern_is_one_ink_fill_over_the_tiles_of_the_rect() {
    for kind in PatternKind::ALL {
        let ops = record(pattern(kind), 1.0, None);
        let fills = ops
            .iter()
            .filter(|o| matches!(o, DrawOp::Fill { .. }))
            .count();
        assert_eq!(fills, 2, "{kind:?}: a base e UM preenchimento do traço");
        let p = pattern_path(kind, Rect::new(0.0, 0.0, 480.0, 320.0));
        let b = p.bounding_box();
        let t = tile();
        assert!(
            b.x0 >= -320.0 - t && b.x1 <= 480.0 + 2.0 * t + 320.0,
            "{kind:?}: só os ladrilhos que tocam o rectângulo ({b:?})"
        );
    }
    // O dobro da área, o dobro dos ladrilhos (nunca o mundo inteiro).
    let small = pattern_path(PatternKind::Dots, Rect::new(0.0, 0.0, 240.0, 240.0));
    let big = pattern_path(PatternKind::Dots, Rect::new(0.0, 0.0, 480.0, 240.0));
    let (a, b) = (small.elements().len(), big.elements().len());
    assert!(b > a * 3 / 2 && b < a * 5 / 2, "{a} → {b}");
}

#[test]
fn from_far_away_a_pattern_is_its_mean_colour() {
    let p = Pattern::new(PatternKind::Grid);
    let far = record(Background::Pattern(p), PATTERN_MIN_TILE_PX / tile() / 2.0, None);
    assert_eq!(far.len(), 1, "um preenchimento liso");
    let mean = kanban_board_bg(Theme::Forge, Background::Pattern(p));
    assert!(matches!(&far[0], DrawOp::Fill { brush: Brush::Solid(c), .. } if *c == mean));
    let base = kanban_tint(Theme::Forge, p.base, false);
    let ink = kanban_tint(Theme::Forge, p.ink, true);
    assert_ne!(mean, base);
    assert_ne!(mean, ink);
}

#[test]
fn a_light_background_darkens_the_header_until_white_reads() {
    let th = Theme::Forge;
    let dark = kanban_bg_tone(th, Background::Color(Hue::Blue), None);
    assert_eq!(
        kanban_header_veil(th, dark),
        token(T::KanbanHeaderBg, th),
        "o azul do Trello leva o véu do token"
    );
    assert!(!kanban_floor_is_light(th, dark));
    let white = kanban_bg_tone(th, Background::Custom(Rgb([255, 255, 255])), None);
    let veil = kanban_header_veil(th, white);
    assert!(veil.components[3] > token(T::KanbanHeaderBg, th).components[3]);
    let a = veil.components[3];
    let under = (1.0 - a) * white.brightest + a * luminance(veil.with_alpha(1.0));
    let ink = luminance(token(T::KanbanHeaderFg, th));
    assert!(contrast(ink, under) >= MIN_CONTRAST - 1e-3, "{}", contrast(ink, under));
    assert!(kanban_floor_is_light(th, white));
    // A imagem: a parte mais clara dela decide.
    let px = Arc::new(Vec::new());
    let bright = BgImage {
        rgba: &px,
        size: [1, 1],
        brightest: 1.0,
        mean: 1.0,
    };
    let img = Background::Image(FileId::of(b"y"));
    assert!(
        kanban_bg_tone(th, img, Some(&bright)).brightest > kanban_bg_tone(th, img, None).brightest
    );
}

#[test]
fn every_gradient_index_has_stops_and_an_unknown_one_falls_back() {
    for i in 0..GRADIENTS.len() {
        #[allow(clippy::cast_possible_truncation)]
        let s = kanban_gradient_stops(Theme::Forge, i as u16);
        assert!(s.len() >= 2);
    }
    assert_eq!(
        kanban_gradient_stops(Theme::Forge, 999),
        kanban_gradient_stops(Theme::Forge, 0)
    );
}

/// ⚠️ MEDIÇÃO (D11): o custo na CPU de montar um padrão a ecrã inteiro (2560×1440, 1×), o mínimo de
/// 30 corridas por tipo. `cargo test -p ph2d-board-render --profile smoke kanban_bg_measure -- --ignored --nocapture`.
#[test]
#[ignore = "medição"]
fn kanban_bg_measure_full_screen_pattern_cost() {
    let r = Rect::new(0.0, 0.0, 2560.0, 1440.0);
    for kind in PatternKind::ALL {
        let mut best = f64::MAX;
        let mut els = 0;
        for _ in 0..30 {
            let t0 = std::time::Instant::now();
            let mut scene = VectorScene::new();
            paint_kanban_bg(
                &mut scene,
                Theme::Forge,
                &KanbanBg {
                    bg: pattern(kind),
                    rect: r,
                    to_scene: Affine::IDENTITY,
                    screen_scale: 1.0,
                    image: None,
                },
            );
            best = best.min(t0.elapsed().as_secs_f64() * 1e3);
            els = pattern_path(kind, r).elements().len();
        }
        println!("{kind:?}: {best:.3} ms, {els} elementos de caminho");
    }
}

/// Report do agente D11: os botões da edição rápida sobre um fundo livre muito claro — o véu sobre o
/// chão liso deixa o texto branco a 4,5:1, nos temas claro e escuro.
#[test]
fn the_quick_edit_veil_reads_white_text_over_a_white_floor() {
    for theme in [Theme::Light, Theme::Dark] {
        let white = ph2d_vector::Color::new([1.0, 1.0, 1.0, 1.0]);
        let veil = kanban_header_veil_over(theme, white);
        let a = veil.components[3];
        let mut composite = white;
        for i in 0..3 {
            composite.components[i] = veil.components[i] * a + 1.0 - a;
        }
        let ink = luminance(token(T::KanbanHeaderFg, theme));
        let c = contrast(ink, luminance(composite));
        assert!(c >= MIN_CONTRAST - 0.05, "{theme:?}: {c}");
    }
}
