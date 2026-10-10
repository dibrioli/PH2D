use super::*;

fn com(curve: i32, dir: i32, shape: Shape) -> Atual<'static> {
    Atual {
        curve,
        dir,
        shape,
        custom: None,
    }
}

/// ⭐ **IDA E VOLTA**: arrastar a alça para onde ela JÁ está devolve a forma que a pôs lá — senão
/// o primeiro toque numa alça muda a curva sem o dedo se mexer.
#[test]
fn dragging_a_handle_onto_itself_gives_back_its_shape() {
    for dir in 0..3 {
        for c in [0.6_f32, 1.70158, 3.2] {
            let a = com(
                ph2d_ease::BACK,
                dir,
                Shape {
                    overshoot: c,
                    ..Shape::DEFAULT
                },
            );
            let (t, y) = alcas(&a)[0];
            let w = arrasta(&a, 0, t, y);
            assert!((w[0].1 - c).abs() < 0.02, "Back dir {dir} c {c}: {w:?}");
        }
        for p in [0.2_f32, 0.3, 0.5] {
            let a = com(
                ph2d_ease::ELASTIC,
                dir,
                Shape {
                    period: p,
                    ..Shape::DEFAULT
                },
            );
            let (t, y) = alcas(&a)[0];
            let w = arrasta(&a, 0, t, y);
            assert!((w[0].1 - p).abs() < 0.03, "Elastic dir {dir} p {p}: {w:?}");
        }
    }
    for z in [0.15_f32, 0.3, 0.6] {
        let a = com(
            ph2d_ease::SPRING,
            0,
            Shape {
                damping: z,
                ..Shape::DEFAULT
            },
        );
        let (t, y) = alcas(&a)[0];
        let w = arrasta(&a, 0, t, y);
        assert!((w[0].1 - z).abs() < 0.02, "Spring z {z}: {w:?}");
    }
    for n in [2.0_f32, 5.0, 9.0] {
        let a = com(
            ph2d_ease::STEPS,
            0,
            Shape {
                steps: n,
                ..Shape::DEFAULT
            },
        );
        let (t, y) = alcas(&a)[0];
        assert_eq!(arrasta(&a, 0, t, y), vec![(nomes::STEPS, n)]);
    }
    let a = com(ph2d_ease::BEZIER, 0, Shape::DEFAULT);
    let h = alcas(&a);
    assert_eq!(
        arrasta(&a, 1, h[1].0, h[1].1),
        vec![("bez_x2", 0.25), ("bez_y2", 1.0)]
    );
}

/// Puxar o pico do `Back` para mais longe da faixa pede MAIS ultrapassagem; o da mola, MENOS
/// amortecimento; e uma mola puxada para dentro da faixa fica crítica (sem pico).
#[test]
fn pulling_the_peak_further_out_asks_for_more() {
    let a = com(ph2d_ease::BACK, 0, Shape::DEFAULT);
    let (t, y) = alcas(&a)[0];
    assert!(arrasta(&a, 0, t, y - 0.2)[0].1 > Shape::DEFAULT.overshoot);
    let m = com(ph2d_ease::SPRING, 0, Shape::DEFAULT);
    let (t, y) = alcas(&m)[0];
    assert!(arrasta(&m, 0, t, y + 0.2)[0].1 < Shape::DEFAULT.damping);
    assert_eq!(arrasta(&m, 0, t, 0.7), vec![(nomes::DAMPING, 1.0)]);
}

/// As famílias SEM forma não têm alça (uma alça que não move nada é um controlo morto), e as
/// que a têm têm-na.
#[test]
fn only_the_families_with_a_shape_have_handles() {
    for f in 0..=ph2d_ease::LAST {
        let n = alcas(&com(f, 0, Shape::DEFAULT)).len();
        let esperado = match f {
            ph2d_ease::BEZIER => 2,
            ph2d_ease::BACK | ph2d_ease::SPRING | ph2d_ease::ELASTIC | ph2d_ease::STEPS => 1,
            _ => 0,
        };
        assert_eq!(n, esperado, "familia {f}");
    }
}

/// Cada miniatura escreve a sua família / direção.
#[test]
fn a_tile_writes_its_family_or_direction() {
    let key = EditorKey {
        own: "t/ease",
        swatch: "t/sw",
    };
    assert_eq!(
        click(key, key.sub("fam/13")),
        Some(vec![(nomes::CURVE, 13.0)])
    );
    assert_eq!(click(key, key.sub("dir/2")), Some(vec![(nomes::DIR, 2.0)]));
    assert_eq!(click(key, key.sub("outra")), None);
}

/// Cada família tem nome que o pintor traduz (nenhuma miniatura com a chave crua).
#[test]
fn every_family_tile_has_a_translated_name() {
    for k in NOMES_FAMILIA.iter().chain(NOMES_DIRECAO.iter()) {
        assert_ne!(tr(k), *k, "chave sem texto: {k}");
    }
}
