use super::*;

const FRAME: Rect = Rect {
    x: 10.0,
    y: 20.0,
    w: 200.0,
    h: 100.0,
};

fn every_variant() -> [Mark; 5] {
    [
        Mark::Rect {
            at: [0.1, 0.7],
            size: [0.8, 0.2],
            fill: ColorToken::Text3,
            round: 0.05,
        },
        Mark::Disc {
            c: [0.3, 0.3],
            r: 0.1,
            fill: ColorToken::Accent,
        },
        Mark::Capsule {
            c: [0.6, 0.4],
            half_h: 0.1,
            r: 0.08,
            fill: ColorToken::Success,
        },
        Mark::Line {
            a: [0.2, 0.5],
            b: [0.8, 0.5],
            stroke: ColorToken::Text1,
            arrow: true,
        },
        Mark::Dashed {
            at: [0.4, 0.2],
            size: [0.3, 0.4],
            stroke: ColorToken::Warn,
        },
    ]
}

fn paths(marks: &[Mark]) -> u32 {
    let mut scene = VectorScene::new();
    paint_illustration(&mut scene, FRAME, marks, Theme::default());
    scene.inner().encoding().n_paths
}

/// ⭐ **Cada variante DESENHA alguma coisa** — uma variante esquecida no `match` seria um traço
/// autorado que não aparece. A seta são DOIS caminhos (a linha e a ponta).
#[test]
fn every_mark_variant_draws_something() {
    for m in every_variant() {
        assert!(paths(&[m]) >= 1, "{m:?} não desenhou nada");
    }
    let line_only = Mark::Line {
        a: [0.2, 0.5],
        b: [0.8, 0.5],
        stroke: ColorToken::Text1,
        arrow: false,
    };
    assert_eq!(
        paths(&[every_variant()[3]]),
        paths(&[line_only]) + 1,
        "a seta acrescenta a ponta"
    );
}

/// **O desenho é uma função dos dados**: as mesmas marcas dão a mesma codificação.
#[test]
fn the_illustration_is_deterministic() {
    let enc = |marks: &[Mark]| {
        let mut scene = VectorScene::new();
        paint_illustration(&mut scene, FRAME, marks, Theme::default());
        let e = scene.inner().encoding();
        (e.n_paths, e.path_tags.len(), e.path_data.clone())
    };
    assert_eq!(enc(&every_variant()), enc(&every_variant()));
    assert_eq!(paths(&[]), 0, "sem marcas, nada");
}
