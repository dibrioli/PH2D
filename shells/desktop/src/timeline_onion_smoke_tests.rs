//! O que a cena do onion AUTORA é testável sem janela: as tracks do Mover. (O passe de
//! fantasmas em si é gateado em `render_loop::timeline_onion`; a pose, em `ph2d-timeline`.)

use super::author_mover;
use ph2d_anim::Interp;
use ph2d_timeline::{PropKind, TimelineDoc};

fn keys(doc: &TimelineDoc, bits: u64, prop: PropKind) -> usize {
    doc.binding_for(bits, prop)
        .and_then(|b| doc.active_clip().track(b.target))
        .map_or(0, |t| t.keys().len())
}

#[test]
fn the_mover_is_keyed_on_translation_and_rotation() {
    let mut doc = TimelineDoc::new();
    let bits = 7_u64;
    author_mover(&mut doc, bits, -6.0, 6.0, 1.2);
    assert_eq!(keys(&doc, bits, PropKind::TranslationX), 2, "X keyado");
    assert_eq!(keys(&doc, bits, PropKind::Rotation), 2, "rotacao keyada");
    // O objeto de fato se MOVE (senão o onion não teria passado/futuro a mostrar): o
    // primeiro key de X é um ease, não um Hold.
    let interp = doc
        .binding_for(bits, PropKind::TranslationX)
        .and_then(|b| doc.active_clip().track(b.target))
        .and_then(|t| t.keys().first().map(|k| k.interp))
        .unwrap();
    assert!(
        matches!(interp, Interp::Bezier { .. }),
        "o ease é o que faz o espaçamento dos fantasmas contar o ritmo"
    );
}

#[test]
fn the_keys_scene_authors_a_pose_per_instant() {
    use super::author_poses;
    let mut doc = TimelineDoc::new();
    let bits = 3_u64;
    let poses = [
        (0.0, -6.0, -3.0, 0.0),
        (1.0, -3.0, 1.0, 0.5),
        (2.0, 0.0, -2.0, -0.4),
    ];
    author_poses(&mut doc, bits, &poses);
    // Cada instante vira uma coluna de keys (X, Y, Rotação) — logo 3 tempos de key.
    assert_eq!(
        ph2d_timeline::entity_key_times(&doc, bits),
        vec![0.0, 1.0, 2.0]
    );
    assert_eq!(
        keys(&doc, bits, PropKind::TranslationY),
        3,
        "Y keyado por pose"
    );
}
