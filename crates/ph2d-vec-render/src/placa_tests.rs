//! Gates da [`super::forma_para_a_placa`] — a porta que decide que forma o passe instanciado
//! desenha e que forma fica no Vello (doc 121 §2.1 do Motion).

use super::forma_para_a_placa;
use ph2d_vec_scene::{BrushStroke, Paint, Rgba8, StrokePaint, StrokeSpec, VecPathId};

/// Um primitivo sem traço: só o preenchimento (com a cor da CÓPIA), e nenhuma linha.
#[test]
fn um_primitivo_sem_traco_da_so_o_preenchimento() {
    let p = ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.4);
    let f = forma_para_a_placa(&p).expect("um primitivo sem tinta própria vai à placa");
    assert!(f.fill.is_some(), "a estrela tem preenchimento");
    assert!(f.tracos.is_empty(), "sem traço, nenhuma linha");
    assert!(f.preenchimentos_do_traco.is_empty());
}

/// Com traço sólido: a linha sai com a LARGURA do traço e a cor dele, os mesmos bytes que o
/// Vello recebe (`Color::from_rgba8`).
#[test]
fn o_traco_solido_leva_a_largura_e_a_cor() {
    let mut p = ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.3);
    p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 0, 51, 128), 0.07));
    let f = forma_para_a_placa(&p).expect("um traço sólido vai à placa");
    assert!(f.fill.is_some());
    assert!(!f.tracos.is_empty(), "o traço tem de chegar como linha");
    for (bp, st) in &f.tracos {
        assert!(!bp.elements().is_empty());
        assert!((st.width - 0.07).abs() < 1e-12, "largura {}", st.width);
    }
    let esperado = [1.0, 0.0, 51.0 / 255.0, 128.0 / 255.0];
    for (a, b) in f.cor_do_traco.iter().zip(esperado) {
        assert!((a - b).abs() < 1e-6, "cor {:?}", f.cor_do_traco);
    }
}

/// ⛔ Um desenho com TINTA PRÓPRIA fica no Vello: o passe pinta com a cor da cópia.
#[test]
fn a_tinta_propria_fica_no_vello() {
    let mut p = ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5);
    p.fill = Some(Paint::Solid(Rgba8::new(10, 20, 30, 255)));
    assert!(forma_para_a_placa(&p).is_none());
}

/// ⛔ Um traço de PINCEL fica no Vello: a tinta dele é uma arte, não uma cor.
#[test]
fn o_traco_de_pincel_fica_no_vello() {
    let mut p = ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5);
    let mut s = StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.1);
    s.paint = StrokePaint::Brush(Box::new(BrushStroke {
        art: Some(VecPathId::from(42u64)),
        fallback: Rgba8::new(11, 22, 33, 200),
        spacing: 1.0,
        offset: 0.0,
        flip: false,
        rotation_deg: 0.0,
        scale: 1.0,
    }));
    p.stroke = Some(s);
    assert!(forma_para_a_placa(&p).is_none());
}
