use crate::{Paint, PaintEntry, PaintSlot, Rgba8, StrokeSpec, VecPath};

const A: Rgba8 = Rgba8::new(10, 20, 30, 255);
const B: Rgba8 = Rgba8::new(200, 100, 50, 255);

fn forma() -> VecPath {
    VecPath {
        fill: Some(Paint::solid(A)),
        stroke: Some(StrokeSpec::new(A, 2.0)),
        paints: vec![
            PaintEntry::fill(Paint::solid(B)),
            PaintEntry::stroke(StrokeSpec::new(B, 3.0)),
        ],
        ..VecPath::default()
    }
}

#[test]
fn o_olho_da_base_guarda_a_tinta_e_devolve_a_mesma() {
    let mut p = forma();
    assert!(p.set_fill_hidden(true));
    assert!(p.fill.is_none(), "escondida = o documento não a pinta");
    assert!(p.fill_is_hidden());
    assert_eq!(p.base_fill(), Some(&Paint::solid(A)), "o editor ainda a vê");
    assert!(!p.set_fill_hidden(true), "esconder de novo é no-op");
    assert!(p.set_fill_hidden(false));
    assert_eq!(p.fill, Some(Paint::solid(A)));
    assert!(p.set_stroke_hidden(true));
    assert!(p.stroke.is_none() && p.stroke_is_hidden());
    assert_eq!(p.base_stroke().map(|s| s.width), Some(2.0));
}

#[test]
fn uma_escrita_no_campo_a_vista_de_uma_base_escondida_volta_a_guarda() {
    let mut p = forma();
    p.set_fill_hidden(true);
    p.fill = Some(Paint::solid(B));
    assert!(p.settle_hidden());
    assert!(p.fill.is_none() && p.fill_hidden == Some(Paint::solid(B)));
    assert!(!p.settle_hidden());
}

#[test]
fn o_menos_da_base_tira_a_tinta_escondida_tambem() {
    let mut p = forma();
    p.set_fill_hidden(true);
    assert!(p.remove_base_fill());
    assert!(p.base_fill().is_none());
}

#[test]
fn o_slot_responde_com_a_tinta_da_linha_e_recusa_a_especie_errada() {
    let mut p = forma();
    assert_eq!(p.slot_fill(PaintSlot::Base), Some(&Paint::solid(A)));
    assert_eq!(p.slot_fill(PaintSlot::Layer(0)), Some(&Paint::solid(B)));
    assert!(p.slot_fill(PaintSlot::Layer(1)).is_none());
    assert!(p.slot_stroke(PaintSlot::Layer(0)).is_none());
    assert_eq!(
        p.slot_stroke(PaintSlot::Layer(1)).map(|s| s.width),
        Some(3.0)
    );
    assert!(p.slot_fill(PaintSlot::Layer(9)).is_none());
    if let Some(s) = p.slot_stroke_mut(PaintSlot::Layer(1)) {
        s.width = 7.0;
    }
    assert_eq!(
        p.slot_stroke(PaintSlot::Layer(1)).map(|s| s.width),
        Some(7.0)
    );
}

/// ⭐ **A tinta de base ESCONDIDA segue a forma** ao mover/escalar (auditoria do fecho UI/UX): o
/// degradê guardado anda com a forma e a largura do traço guardado escala com ela — mostrar depois
/// não devolve o desenho do sítio antigo.
#[test]
fn a_hidden_base_paint_follows_the_shape_when_it_moves_and_scales() {
    use crate::{GradientStop, StrokeSpec};
    let mut scene = crate::VecScene::default();
    let mut p = crate::rectangle([0.0, 0.0], [10.0, 10.0]);
    p.fill = Some(Paint::Linear {
        stops: vec![
            GradientStop::new(0.0, Rgba8::new(255, 0, 0, 255)),
            GradientStop::new(1.0, Rgba8::new(0, 0, 255, 255)),
        ],
        start: [0.0, 0.0],
        end: [10.0, 0.0],
    });
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 2.0));
    assert!(p.set_fill_hidden(true) && p.set_stroke_hidden(true));
    let id = scene.push_path(p);
    assert!(scene.translate_path(id, 100.0, 0.0));
    assert!(scene.scale_path(id, 3.0, 3.0, [100.0, 0.0]));
    let p = scene.path(id).expect("forma");
    let Some(Paint::Linear { start, .. }) = &p.fill_hidden else {
        panic!("o degradê continua guardado");
    };
    assert_eq!(*start, [100.0, 0.0], "o degradê guardado andou com a forma");
    assert_eq!(p.stroke_hidden.as_ref().map(|s| s.width), Some(6.0));
}
