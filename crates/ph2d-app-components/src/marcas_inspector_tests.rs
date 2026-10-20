//! Gates do [`super`] (`marcas_inspector.rs`) — o losango dos campos do Inspector.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};

const E: u64 = 11;

fn em(t: f64) -> Instante {
    Instante {
        t_cut: t,
        solo: true,
    }
}

fn na_curva(_: u64, _: PropKind) -> Option<f32> {
    Some(2.0)
}

fn chave(d: &mut TimelineDoc, p: PropKind, t: f64) {
    d.insert_key(
        E,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(2.0),
        Interp::Linear,
    );
}

fn add(e: u64, p: PropKind) -> Option<TimelineIntent> {
    Some(TimelineIntent::AddKey {
        entity: e,
        prop: p,
        t: RationalTime::from_seconds(0.5),
        value: AnimValue::Float(1.0),
        interp: Interp::Linear,
    })
}

fn gravados(i: &[TimelineIntent]) -> Vec<PropKind> {
    i.iter()
        .filter_map(|x| match x {
            TimelineIntent::AddKey { prop, .. } => Some(*prop),
            _ => None,
        })
        .collect()
}

/// O X e o Y do Transform partilham o losango da posição; a rotação é a dela.
#[test]
fn o_x_e_o_y_mostram_a_posicao_inteira() {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    chave(&mut d, PropKind::TranslationY, 0.0);
    chave(&mut d, PropKind::TranslationY, 1.0);
    let m = marcas(&d, &[E], em(0.0), na_curva);
    assert_eq!(m[&insp::INSP_TRANSFORM_POS_X], KeyMark::Chave);
    assert_eq!(m[&insp::INSP_TRANSFORM_ROT], KeyMark::Livre);
    let m = marcas(&d, &[E], em(0.5), na_curva);
    assert_eq!(m[&insp::INSP_TRANSFORM_POS_Y], KeyMark::Animado);
    assert!(marcas(&d, &[], em(0.0), na_curva).is_empty());
}

/// Um objecto de eixos separados grava X e Y; um novo (o padrão do After Effects, a porta da K)
/// grava a `Position` — nunca os dois modos. (Mutação: gravar o grupo inteiro ⇒ os dois modos.)
#[test]
fn o_clique_grava_a_posicao_do_modo_do_objecto() {
    let d = TimelineDoc::new();
    let i = clique(
        &d,
        &[insp::INSP_TRANSFORM_POS_X],
        &[E],
        em(0.5),
        na_curva,
        add,
    );
    assert_eq!(gravados(&i), vec![PropKind::Position]);
    let mut separados = TimelineDoc::new();
    separados.fps_display = 24.0;
    chave(&mut separados, PropKind::TranslationX, 0.0);
    let i = clique(
        &separados,
        &[insp::INSP_TRANSFORM_POS_Y],
        &[E],
        em(0.5),
        na_curva,
        add,
    );
    assert_eq!(
        gravados(&i),
        vec![PropKind::TranslationX, PropKind::TranslationY]
    );
    let i = clique(
        &d,
        &[insp::INSP_TRANSFORM_SCALE_Y],
        &[E],
        em(0.5),
        na_curva,
        add,
    );
    assert_eq!(gravados(&i), vec![PropKind::ScaleX, PropKind::ScaleY]);
    assert!(
        clique(
            &d,
            &[insp::INSP_TRANSFORM_SKEW_X],
            &[E],
            em(0.5),
            na_curva,
            add
        )
        .is_empty()
    );
}

/// Numa chave, o clique apaga — aqui a da rotação, num passo de undo.
#[test]
fn o_clique_numa_chave_apaga() {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    chave(&mut d, PropKind::Rotation, 0.0);
    let i = clique(
        &d,
        &[insp::INSP_TRANSFORM_ROT],
        &[E],
        em(0.0),
        na_curva,
        add,
    );
    assert_eq!(i.first(), Some(&TimelineIntent::BeginEdit));
    assert!(i.contains(&TimelineIntent::DeleteSelection));
    assert!(gravados(&i).is_empty());
}
