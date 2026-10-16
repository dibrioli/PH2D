use super::*;
use crate::Rgba8;

fn cheio() -> StrokeSpec {
    let mut s = StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.75);
    s.cap = LineCap::Square;
    s.join = LineJoin::Bevel;
    let mut p = DashPattern::single(3.0, 1.0);
    p.pairs[1] = [0.0, 2.0];
    p.pairs[2] = [5.0, 0.5];
    s.dash = Some(p);
    s.marker_start = Marker::Triangle;
    s.marker_end = Marker::Diamond;
    s.marker_scale_start = 0.5;
    s.marker_scale_end = 2.25;
    s.marker_round = 0.3;
    s.align = StrokeAlign::Outer;
    s
}

/// O espelho `StrokeSpecV23` e o `Serialize` derivado andam na MESMA ordem — com dois campos do
/// mesmo tipo trocados, o round-trip troca-lhes os valores, e todos os valores aqui são distintos.
#[test]
fn the_live_layout_round_trips() {
    let s = cheio();
    let bytes = postcard::to_allocvec(&s).unwrap();
    let back: StrokeSpec = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back, s);
}

/// O âmbito v22 lê a forma antiga, e fecha: a leitura seguinte volta à viva.
#[test]
fn the_v22_scope_reads_the_old_layout_and_closes() {
    #[derive(serde::Serialize)]
    struct Antigo {
        paint: StrokePaint,
        width: f64,
        cap: LineCap,
        join: LineJoin,
        dash: Option<(f64, f64)>,
        marker_start: Marker,
        marker_end: Marker,
        marker_scale: f64,
        marker_round: f64,
        align: StrokeAlign,
    }
    let antigo = postcard::to_allocvec(&Antigo {
        paint: StrokePaint::Solid(Rgba8::new(9, 8, 7, 255)),
        width: 0.5,
        cap: LineCap::Round,
        join: LineJoin::Round,
        dash: Some((2.0, 1.5)),
        marker_start: Marker::Circle,
        marker_end: Marker::None,
        marker_scale: 1.7,
        marker_round: 0.25,
        align: StrokeAlign::Inner,
    })
    .unwrap();
    let s: StrokeSpec = with_stroke_layout_v22(|| postcard::from_bytes(&antigo)).unwrap();
    assert_eq!(s.dash, Some(DashPattern::single(2.0, 1.5)));
    assert_eq!((s.marker_scale_start, s.marker_scale_end), (1.7, 1.7));
    assert_eq!((s.marker_start, s.marker_round, s.align), (Marker::Circle, 0.25, StrokeAlign::Inner));
    assert!(!reading_v22(), "o âmbito fechou");
    let vivo = postcard::to_allocvec(&cheio()).unwrap();
    assert_eq!(postcard::from_bytes::<StrokeSpec>(&vivo).unwrap(), cheio());
}

/// ⭐ **O PONTO do «traço-ponto» tem comprimento que uma coordenada de mundo representa** — a FOTO da
/// cena 24 (F16c) mostrou os pontos a sumir com o piso `f64::EPSILON`: somado a um `x` de mundo, o
/// segmento colapsava e a kurbo descartava-o.
#[test]
fn a_zero_dash_after_the_first_is_a_dot_that_survives_world_coordinates() {
    let mut s = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.05);
    s.dash = Some(DashPattern {
        pairs: [[4.0, 1.5], [0.0, 1.5], [0.0, 1.5]],
    });
    let l = s.dash_lengths().expect("tracejado");
    let ponto = l[2];
    assert!(ponto >= crate::MIN_DASH_FRACTION * s.width, "o ponto mede {ponto}");
    for x in [1.0_f64, 1.3e3, 1.0e5] {
        assert!(x + ponto > x, "em x = {x} o ponto colapsa");
    }
    // ⛔ CONTROLO: o piso antigo colapsava já em x = 1.
    assert_eq!(1.0 + f64::EPSILON / 2.0, 1.0);
}
