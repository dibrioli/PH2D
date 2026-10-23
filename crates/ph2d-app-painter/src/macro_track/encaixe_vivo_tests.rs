//! O encaixe vivo segue o gizmo do objecto Macro: mover +N unidades desloca N·ppm texels, rodar 90°
//! roda a parte linear — pelo palco em cena, quadro a quadro.

use super::super::{Lido, PalcosDoPainter};
use super::do_mundo;
use ph2d_ecs::{Entity, Macro, StableId, Transform, World};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_macro::{MacroRecorder, RefRect};
use ph2d_render::Sprite;
use ph2d_tool_painter::macro_track::{encaixe::Encaixe, inicial};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use std::collections::BTreeMap;

const ALVO: u64 = 7;

/// Um traço gravado numa tela 64×64, posto por cima de uma de 128×128 (a etiqueta 6).
fn macro_por_cima() -> Vec<u8> {
    let n = 64u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    let mut rec = MacroRecorder::new(RefRect { w: 64.0, h: 64.0 }, 0);
    let trilha = t.macro_abre(&mut rec, ALVO).expect("arma");
    for (i, phase) in [PointerPhase::Down, PointerPhase::Move, PointerPhase::Up]
        .into_iter()
        .enumerate()
    {
        t.on_canvas_pointer(CanvasPointer {
            pos: [10.0 + 20.0 * i as f32, 30.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        drop(t.take_preview_arc());
        t.on_tick(16.0);
        for c in t.macro_passos() {
            rec.push(trilha, i as u64 * 16_666, c)
                .expect("tempo crescente");
        }
    }
    for c in t.macro_para() {
        rec.push(trilha, 3 * 16_666, c).expect("tempo crescente");
    }
    let mut doc = rec.finish();
    let mut grande = PainterTool::default();
    grande.bind_document(1, vec![255u8; 128 * 128 * 4], 128, 128);
    let base = grande
        .collect_documents(&BTreeMap::from([(1, 0)]))
        .pop()
        .expect("o documento do alvo");
    let tr = &mut doc.tracks[0];
    tr.initial = inicial::sobre_documento(&tr.initial, &base).expect("por cima");
    tr.target = ALVO;
    ph2d_macro::codec::write(&doc)
}

/// O mundo: o objecto Macro na origem e a sprite-alvo 128×128 a 100 px por unidade.
fn mundo() -> (World, Entity, Entity) {
    let mut w = World::new();
    let alvo = w
        .spawn((
            Transform::IDENTITY,
            Sprite::atlas(0, [1.28, 1.28], [1.0; 4]),
            StableId(ALVO),
        ))
        .id();
    let objeto = w
        .spawn((
            Transform::IDENTITY,
            Macro {
                doc: macro_por_cima(),
                modo: ph2d_ecs::ModoDaMacro::Fiel,
            },
        ))
        .id();
    (w, objeto, alvo)
}

fn perto(e: Encaixe, esperado: [f64; 6]) {
    for (a, b) in e.afim.iter().zip(esperado) {
        assert!((a - b).abs() < 1e-3, "{:?} ≠ {esperado:?}", e.afim);
    }
}

fn do_palco(p: &PalcosDoPainter, objeto: Entity) -> Option<Encaixe> {
    match p.lidos.get(&objeto.to_bits()) {
        Some(Lido {
            palco: Ok(Some(x)), ..
        }) => x.em_cena.encaixe(),
        _ => None,
    }
}

#[test]
fn o_encaixe_vivo_segue_o_gizmo() {
    let (mut w, objeto, alvo) = mundo();
    // Na pose do alvo: a gravação 64×64 centrada na 128×128 cai 32 texels para dentro, sem esticar.
    perto(
        do_mundo(&w, objeto, alvo, (128, 128), (64, 64)).expect("encaixa"),
        [1.0, 0.0, 32.0, 0.0, 1.0, 32.0],
    );
    let mut palcos = PalcosDoPainter::default();
    palcos.segue(&mut w);
    perto(
        do_palco(&palcos, objeto).expect("o palco pôs o encaixe"),
        [1.0, 0.0, 32.0, 0.0, 1.0, 32.0],
    );

    // Mover o gizmo +0,5 unidade em x: +50 texels, o resto igual.
    w.get_mut::<Transform>(objeto).unwrap().translation.x = 0.5;
    palcos.segue(&mut w);
    perto(
        do_palco(&palcos, objeto).expect("segue"),
        [1.0, 0.0, 82.0, 0.0, 1.0, 32.0],
    );

    // Rodar 90°: o x gravado (direita) sobe no mundo ⇒ desce no texel; o y gravado (baixo) vai à direita.
    let t = &mut *w.get_mut::<Transform>(objeto).unwrap();
    t.translation.x = 0.0;
    t.rotation = std::f32::consts::FRAC_PI_2;
    palcos.segue(&mut w);
    perto(
        do_palco(&palcos, objeto).expect("segue"),
        [0.0, 1.0, 32.0, -1.0, 0.0, 96.0],
    );
}
