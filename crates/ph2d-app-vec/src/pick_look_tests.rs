//! O realce é SÓ o contorno de acento (dono 02/11, auditoria contra a pilha F45): limpar `fill` e
//! `stroke` deixava as camadas da forma a desenhar por baixo dele.

use super::*;
use ph2d_vec_scene::{Opacity, Paint, PaintEntry, Rgba8, VecPath, VecVertex};

fn vestida() -> (VecScene, VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(9, 9, 9, 255))),
        paints: vec![
            PaintEntry::fill(Paint::Solid(Rgba8::new(1, 2, 3, 255))),
            PaintEntry::stroke(StrokeSpec::new(Rgba8::new(4, 5, 6, 255), 0.5)),
        ],
        opacity: Opacity::new(0.2),
        ..VecPath::default()
    });
    (scene, id)
}

fn so_o_realce(p: &VecPath) {
    assert_eq!(p.fill, None);
    assert_eq!(p.stroke, Some(pick_stroke()));
    assert!(
        p.paints.is_empty(),
        "as camadas da forma desenham sob o realce"
    );
    assert_eq!(p.opacity, Opacity::default(), "o realce sai desbotado");
}

#[test]
fn the_hover_outline_is_only_the_accent() {
    let (scene, id) = vestida();
    so_o_realce(&hover_outline(&scene, &VecXforms::default(), id).expect("forma"));
}
