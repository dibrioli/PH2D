//! ⭐ F44b — o corpo de ajustes de Fill/Stroke edita a linha ABERTA (base ou camada `i`); a linha
//! fixa (cor, Weight, Position) fica na base; uma base escondida é editada escondida.

use super::{ALIGN_CODE, join_code_for_id, open_slot, put_slot_fill, set_join_or_align};
use crate::vector_bridge::{StrokeStyle, restyle_slot_fields};
use ph2d_panel_vector::state::set_open_layer;
use ph2d_tool_vector::StyleFields as F;
use ph2d_vec_scene::{
    LineCap, LineJoin, Marker, Paint, PaintEntry, PaintKind, PaintSlot, Rgba8, StrokeAlign,
    StrokePaint, StrokeSpec, VecPath, VecPathId, VecScene, VecVertex,
};

const AZUL: Rgba8 = Rgba8 {
    r: 0,
    g: 0,
    b: 255,
    a: 255,
};
const VERDE: Rgba8 = Rgba8 {
    r: 0,
    g: 200,
    b: 0,
    a: 255,
};

/// Uma forma com traço de base azul e duas camadas: `0` = traço verde, `1` = preenchimento verde.
fn cena() -> (VecScene, VecPathId) {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(AZUL)),
        stroke: Some(StrokeSpec::new(AZUL, 1.0)),
        paints: vec![
            PaintEntry::stroke(StrokeSpec::new(VERDE, 2.0)),
            PaintEntry::fill(Paint::Solid(VERDE)),
        ],
        ..VecPath::default()
    });
    (scene, id)
}

fn ficha(color: Rgba8) -> StrokeStyle {
    StrokeStyle {
        color,
        cap: LineCap::Round,
        join: LineJoin::Bevel,
        align: StrokeAlign::Outer,
        dash: None,
        marker_start: Marker::default(),
        marker_end: Marker::default(),
        marker_scale: [1.0, 1.0],
        marker_round: 0.0,
    }
}

fn camada(scene: &VecScene, id: VecPathId, i: usize) -> &PaintKind {
    &scene.path(id).expect("forma").paints[i].kind
}

fn traco_da_camada(scene: &VecScene, id: VecPathId, i: usize) -> &StrokeSpec {
    match camada(scene, id, i) {
        PaintKind::Stroke(s) => s,
        PaintKind::Fill(_) => panic!("a camada {i} devia ser um traço"),
    }
}

#[test]
fn with_a_layer_open_the_body_fields_land_on_that_layer_and_the_row_fields_on_the_base() {
    let (mut scene, id) = cena();
    let base_antes = scene.path(id).and_then(|p| p.stroke.clone()).expect("base");
    let campos = F::CAP | F::JOIN | F::STROKE_RGB | F::ALIGN;
    let n = restyle_slot_fields(
        &mut scene,
        &[id],
        &ficha(Rgba8::new(255, 0, 0, 255)),
        1.0,
        campos,
        PaintSlot::Layer(0),
    );
    assert_eq!(n, 1);
    let l = traco_da_camada(&scene, id, 0);
    assert_eq!(
        (l.cap, l.join),
        (LineCap::Round, LineJoin::Bevel),
        "o corpo não chegou à camada"
    );
    assert_eq!(
        l.color(),
        VERDE,
        "a cor (linha fixa da base) vazou para a camada"
    );
    assert_eq!(
        l.align,
        StrokeAlign::Centre,
        "a Position da base vazou para a camada"
    );
    let b = scene
        .path(id)
        .and_then(|p| p.stroke.as_ref())
        .expect("base");
    assert_eq!(
        (b.cap, b.join),
        (base_antes.cap, base_antes.join),
        "o corpo pisou a base"
    );
    assert_eq!(
        b.color(),
        Rgba8::new(255, 0, 0, 255),
        "a cor da base não foi escrita"
    );
    assert_eq!(
        b.align,
        StrokeAlign::Outer,
        "a Position da base não foi escrita"
    );
}

#[test]
fn with_the_base_open_the_body_fields_land_on_the_base_and_no_layer_moves() {
    let (mut scene, id) = cena();
    let camadas_antes = scene.path(id).expect("forma").paints.clone();
    restyle_slot_fields(
        &mut scene,
        &[id],
        &ficha(AZUL),
        1.0,
        F::CAP | F::JOIN,
        PaintSlot::Base,
    );
    let b = scene
        .path(id)
        .and_then(|p| p.stroke.as_ref())
        .expect("base");
    assert_eq!((b.cap, b.join), (LineCap::Round, LineJoin::Bevel));
    assert_eq!(
        scene.path(id).expect("forma").paints,
        camadas_antes,
        "uma camada mudou"
    );
}

#[test]
fn a_fill_layer_open_skips_the_stroke_body_and_never_creates_a_stroke() {
    let (mut scene, id) = cena();
    let antes = scene.clone();
    restyle_slot_fields(
        &mut scene,
        &[id],
        &ficha(AZUL),
        1.0,
        F::CAP,
        PaintSlot::Layer(1),
    );
    restyle_slot_fields(
        &mut scene,
        &[id],
        &ficha(AZUL),
        1.0,
        F::CAP,
        PaintSlot::Layer(7),
    );
    assert!(
        scene == antes,
        "uma linha de espécie errada / inexistente foi escrita"
    );
}

#[test]
fn a_hidden_base_stroke_is_edited_and_stays_hidden() {
    let (mut scene, id) = cena();
    assert!(scene.path_mut(id).expect("forma").set_stroke_hidden(true));
    restyle_slot_fields(
        &mut scene,
        &[id],
        &ficha(AZUL),
        1.0,
        F::CAP,
        PaintSlot::Base,
    );
    let p = scene.path(id).expect("forma");
    assert!(p.stroke.is_none(), "editar a base escondida mostrou-a");
    assert_eq!(
        p.stroke_hidden.as_ref().map(|s| s.cap),
        Some(LineCap::Round)
    );
}

#[test]
fn the_layer_position_chips_ride_the_join_channel_and_set_the_open_layer_align() {
    use ph2d_panel_vector::ids as i;
    assert_eq!(
        join_code_for_id(i::VECTOR_PAINT_ALIGN_INNER),
        Some(ALIGN_CODE)
    );
    assert_eq!(
        join_code_for_id(i::VECTOR_PAINT_ALIGN_CENTRE),
        Some(ALIGN_CODE + 1)
    );
    assert_eq!(
        join_code_for_id(i::VECTOR_PAINT_ALIGN_OUTER),
        Some(ALIGN_CODE + 2)
    );
    assert_eq!(join_code_for_id(i::VECTOR_PAINT_JOIN_BEVEL), Some(2));
    let (mut scene, id) = cena();
    let code = join_code_for_id(i::VECTOR_PAINT_ALIGN_OUTER).expect("mapeia");
    assert!(set_join_or_align(&mut scene, &[id], 0, code));
    assert_eq!(traco_da_camada(&scene, id, 0).align, StrokeAlign::Outer);
    assert_eq!(
        scene
            .path(id)
            .and_then(|p| p.stroke.as_ref())
            .map(|s| s.align),
        Some(StrokeAlign::Centre),
        "a Position da camada mexeu na base"
    );
    assert!(
        !set_join_or_align(&mut scene, &[id], 1, code),
        "num preenchimento é no-op"
    );
    assert!(
        set_join_or_align(&mut scene, &[id], 0, 2),
        "o código da quina ainda é a quina"
    );
    assert_eq!(scene.path(id).expect("forma").paints[0].dilate_join, 2);
}

#[test]
fn the_fill_kind_writer_follows_the_open_row_and_never_creates_a_layer_fill() {
    let (mut scene, id) = cena();
    set_open_layer(Some(1));
    assert_eq!(open_slot(), PaintSlot::Layer(1));
    let rampa = Paint::Solid(Rgba8::new(9, 9, 9, 255));
    assert!(put_slot_fill(
        scene.path_mut(id).expect("forma"),
        rampa.clone()
    ));
    assert_eq!(camada(&scene, id, 1), &PaintKind::Fill(rampa.clone()));
    assert_eq!(
        scene.path(id).and_then(|p| p.fill.clone()),
        Some(Paint::Solid(AZUL))
    );
    set_open_layer(Some(0));
    assert!(
        !put_slot_fill(scene.path_mut(id).expect("forma"), rampa.clone()),
        "virou traço?"
    );
    set_open_layer(None);
    // A base ESCONDIDA é editada escondida.
    assert!(scene.path_mut(id).expect("forma").set_fill_hidden(true));
    assert!(put_slot_fill(
        scene.path_mut(id).expect("forma"),
        rampa.clone()
    ));
    let p = scene.path(id).expect("forma");
    assert!(p.fill.is_none(), "editar a base escondida mostrou-a");
    assert_eq!(p.fill_hidden, Some(rampa));
}

#[test]
fn the_stroke_kind_and_the_pattern_reader_follow_the_open_layer() {
    let (mut scene, id) = cena();
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    set_open_layer(Some(0));
    assert!(crate::stroke_paint::set_kind(
        &mut scene,
        &pen,
        ph2d_panel_vector::StrokePaintKind::Brush,
        None
    ));
    assert!(
        traco_da_camada(&scene, id, 0).brush().is_some(),
        "o pincel não nasceu na camada"
    );
    assert!(matches!(
        scene
            .path(id)
            .and_then(|p| p.stroke.as_ref())
            .map(|s| &s.paint),
        Some(StrokePaint::Solid(_))
    ));
    assert_eq!(
        crate::stroke_paint::selected_stroke_paint_kind(&scene, &pen),
        Some(ph2d_panel_vector::StrokePaintKind::Brush),
        "o chip lê a base, não a camada aberta"
    );
    assert!(crate::stroke_paint::apply(
        &mut scene,
        &pen,
        crate::stroke_paint::BrushCmd::Scale(3.0)
    ));
    assert_eq!(
        traco_da_camada(&scene, id, 0).brush().map(|b| b.scale),
        Some(3.0)
    );
    set_open_layer(None);
    assert_eq!(
        crate::stroke_paint::selected_stroke_paint_kind(&scene, &pen),
        Some(ph2d_panel_vector::StrokePaintKind::Solid)
    );
}

/// ⭐ **O gizmo do degradê no canvas segue a linha ABERTA** pela porta da app (`grad_hit`/`grad_drag`
/// lêem o `open_slot()`): com a camada `0` aberta, a alça que se apanha e arrasta é a DELA e a base
/// fica. *Mutação (prova de fecho, sobreviveu antes deste teste): `PaintSlot::Base` fixo ⇒ a alça da
/// camada não responde e a base é que mexe.*
#[test]
fn the_canvas_gradient_gizmo_follows_the_open_layer_through_the_app_door() {
    use ph2d_vec_scene::GradientStop;
    let ramp = |a: [f64; 2], b: [f64; 2]| Paint::Linear {
        stops: vec![
            GradientStop::new(0.0, AZUL),
            GradientStop::new(1.0, VERDE),
        ],
        start: a,
        end: b,
    };
    let mut p = VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(ramp([0.0, 0.0], [10.0, 0.0])),
        paints: vec![PaintEntry::fill(ramp([0.0, 8.0], [10.0, 8.0]))],
        ..VecPath::default()
    };
    set_open_layer(Some(0));
    let h = super::grad_hit(&p, [0.0, 8.0], 0.5).expect("a alça de início da CAMADA");
    assert!(super::grad_drag(&mut p, h, 2.0, 9.0));
    set_open_layer(None);
    let PaintKind::Fill(Paint::Linear { start, .. }) = &p.paints[0].kind else {
        panic!("a camada continua um degradê");
    };
    assert_eq!(*start, [2.0, 9.0], "a alça mexeu na camada aberta");
    let Some(Paint::Linear { start, .. }) = &p.fill else {
        panic!("a base continua um degradê");
    };
    assert_eq!(*start, [0.0, 0.0], "e a base ficou");
}
