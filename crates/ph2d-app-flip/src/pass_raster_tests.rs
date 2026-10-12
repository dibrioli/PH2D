//! O quadro actual de um objecto Flip fora do ecrã (W15): a caixa (CPU) e a imagem (placa, `#[ignore]`).

use super::*;
use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};

/// Um objecto com um traço vermelho de `(0,0)` a `(2,0)`, raio `0,1`, no quadro 0; e um 2.º traço
/// noutra chave (quadro 6) que NÃO pode entrar na imagem do quadro 0.
fn doc_line() -> (FlipDoc, FlipObjectId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Line");
    let obj = doc.object_mut(oid).unwrap();
    obj.fps = 12.0;
    let layer = obj.add_layer("L");
    let line = |to: f32| {
        let mut s = FlipStroke::new();
        for x in [0.0, to] {
            s.push_point(Point {
                pos: Vec2::new(x, 0.0),
                width: 0.1,
                opacity: 1.0,
                color: Rgba::new(1.0, 0.0, 0.0, 1.0),
            });
        }
        s
    };
    let d = obj
        .insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    obj.drawing_mut(d).unwrap().strokes.push(line(2.0));
    let d6 = obj
        .insert_frame(layer, 6, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    obj.drawing_mut(d6).unwrap().strokes.push(line(9.0));
    (doc, oid)
}

fn at(f: i32) -> Playhead {
    let mut p = Playhead::new(1.0 / 12.0);
    p.pause();
    p.seek(f64::from(f) / 12.0);
    p
}

/// ⭐ A caixa é a do QUADRO ACTUAL (a chave do quadro 6 não entra), alargada pelo raio, e escala
/// com o `model` do objecto (posição e espessura). (Mutação: ignorar o raio ⇒ `y0 = 0` — RED.)
#[test]
fn the_box_is_the_current_frames_strokes_with_their_radius_posed() {
    let (doc, oid) = doc_line();
    let boxed = |model: Xform| {
        let models = [(oid, model)];
        let (layers, _) = collect_layers(
            &doc,
            &at(0),
            None,
            FlipTarget::default(),
            &models,
            None,
            None,
        );
        world_box(&layers).expect("um traço")
    };
    let b = boxed(Xform::IDENTITY);
    let close = |a: [f64; 4], e: [f64; 4]| a.iter().zip(e).all(|(x, y)| (x - y).abs() < 1e-6);
    assert!(close(b, [-0.1, -0.1, 2.1, 0.1]), "{b:?}");
    let b2 = boxed(Xform([2.0, 0.0, 0.0, 2.0, 5.0, 1.0]));
    assert!(close(b2, [4.8, 0.8, 9.2, 1.2]), "{b2:?}");
}

fn gpu() -> Option<GpuContext> {
    GpuContext::new(wgpu::Instance::default(), None).ok()
}

/// ⭐⭐ A imagem pela placa: `100 px` por unidade dão `2,2 × 0,2` unidades + 2 px de orla =
/// `222 × 22`; o meio do traço é vermelho opaco (alfa reto: o vermelho não escurece), o canto é
/// transparente (a ponta é redonda), e sem traço neste quadro não há imagem.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn the_current_frame_comes_out_at_the_editors_pixel_ruler() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — nada a medir");
        return;
    };
    let (doc, oid) = doc_line();
    let mut render = FlipRenderer::new(&gpu.device, ph2d_render::GameRt::FORMAT);
    let mut compose = FlipCompose::new(&gpu.device, ph2d_render::GameRt::FORMAT);
    let img = current_frame(
        &doc,
        oid,
        Xform::IDENTITY,
        &at(0),
        &mut render,
        &mut compose,
        100.0,
        &gpu,
    )
    .expect("o quadro 0 tem traço");
    assert_eq!(img.size, [222, 22]);
    let px = |x: u32, y: u32| {
        let i = ((y * img.size[0] + x) * 4) as usize;
        [
            img.rgba[i],
            img.rgba[i + 1],
            img.rgba[i + 2],
            img.rgba[i + 3],
        ]
    };
    let mid = px(111, 11);
    assert!(
        mid[3] == 255 && mid[0] > 240 && mid[1] < 16 && mid[2] < 16,
        "{mid:?}"
    );
    assert_eq!(
        px(0, 0)[3],
        0,
        "o canto da caixa fica fora da ponta redonda"
    );
    let wide = current_frame(
        &doc,
        oid,
        Xform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]),
        &at(0),
        &mut render,
        &mut compose,
        100.0,
        &gpu,
    )
    .expect("escalado");
    assert_eq!(
        wide.size,
        [442, 42],
        "o objecto escalado 2× sai com o dobro"
    );
    let empty = FlipDoc::new();
    let none = current_frame(
        &empty,
        oid,
        Xform::IDENTITY,
        &at(0),
        &mut render,
        &mut compose,
        100.0,
        &gpu,
    );
    assert_eq!(none, Err(RasterError::Empty));
}
