use crate::{
    Affine, BezPath, Brush, Color, DrawOp, Rect, Shape as _, Stroke, VectorScene, VelloBlend,
};

fn kinds(ops: &[DrawOp]) -> Vec<&'static str> {
    ops.iter()
        .map(|o| match o {
            DrawOp::Fill { .. } => "fill",
            DrawOp::Stroke { .. } => "stroke",
            DrawOp::Layer { .. } => "layer",
            DrawOp::Pop => "pop",
            DrawOp::Glyphs(_) => "glyphs",
            DrawOp::BlurredRect { .. } => "blur",
        })
        .collect()
}

/// ⭐ Cada porta que grava grava, pela ordem em que se desenhou; `reset` esvazia e continua a gravar.
#[test]
fn every_recording_door_records_in_order() {
    let mut s = VectorScene::new();
    let red = Color::from_rgba8(255, 0, 0, 255);
    let r = Rect::new(0.0, 0.0, 10.0, 10.0);
    s.fill_rect(r, red); // antes de ligar: não grava
    s.record();
    s.fill_rect(r, red);
    s.push_object_layer(&r, VelloBlend::default(), 0.5);
    s.fill_path(
        &r.to_path(0.1),
        &Brush::Solid(red),
        Affine::translate((5.0, 0.0)),
    );
    s.stroke_path(&Stroke::new(2.0), Affine::IDENTITY, &Brush::Solid(red), &r);
    s.push_clip(&r);
    s.blurred_rounded_rect(Affine::IDENTITY, r, red, 1.0, 2.0);
    s.pop_layer();
    s.pop_layer();
    let ops = s.take_record();
    assert_eq!(
        kinds(&ops),
        [
            "fill", "layer", "fill", "stroke", "layer", "blur", "pop", "pop"
        ]
    );
    let DrawOp::Layer { alpha, .. } = &ops[1] else {
        unreachable!()
    };
    assert_eq!(*alpha, 0.5);
    let DrawOp::Fill { transform, .. } = &ops[2] else {
        unreachable!()
    };
    assert_eq!(*transform, Affine::translate((5.0, 0.0)));
    s.fill_rect(r, red);
    s.reset();
    assert!(s.take_record().is_empty(), "reset esvazia");
    s.fill_path(&BezPath::new(), &Brush::Solid(red), Affine::IDENTITY);
    assert_eq!(s.take_record().len(), 1, "e continua a gravar");
}

/// Sem `record`, nada se guarda.
#[test]
fn without_record_nothing_is_kept() {
    let mut s = VectorScene::new();
    s.fill_rect(
        Rect::new(0.0, 0.0, 1.0, 1.0),
        Color::from_rgba8(0, 0, 0, 255),
    );
    assert!(s.take_record().is_empty());
}
