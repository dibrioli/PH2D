//! B19.7 (resto, ADR-0202): fora do compositor, as passadas em faixas de linhas — o overlay da seleção,
//! o levante / a cor / o corpo do Transform e a recomposição por camada do carimbo colorido — correm no
//! pool do rayon (TIDs novos em `/proc/self/task` = 0) e dão o byte do laço serial a 1..32 faixas.

use crate::compositor::FIOS_FORCADOS;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Dab, Falloff, StrokeMethod};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

const S: u32 = 640;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tela(b: BrushSpec) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t
}

/// O overlay de uma seleção retangular (hachura + formigas) na tela inteira.
fn selecao() -> PainterTool {
    let mut t = tela(BrushSpec::default());
    t.set_rect_selection(100, 120, 300, 260);
    t.paint.selection_active = true;
    t
}
fn selecao_age(t: &mut PainterTool) -> Vec<u8> {
    t.selection_overlay_rgba(3).expect("overlay").0
}

/// Um dab de raio 250 de uma ponta de duas camadas com cor por camada (a recomposição por camada).
fn carimbo() -> PainterTool {
    let mut t = tela(BrushSpec {
        radius_px: 250.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stroke_method: StrokeMethod::Space,
        ..Default::default()
    });
    let mut metade = vec![0u8; 64];
    for row in 0..8 {
        metade[row * 8 + 4..row * 8 + 8].fill(255);
    }
    t.set_brush_shape_layers(vec![(vec![200u8; 64], 8, 8), (metade, 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [0.9, 0.1, 0.1]);
    t.set_brush_shape_layer_color(1, [0.1, 0.8, 0.2]);
    t
}
fn carimbo_age(t: &mut PainterTool) -> Vec<u8> {
    let dab = |cx: f32| Dab {
        center: [cx, 320.0],
        radius_px: 250.0,
        coverage: 1.0,
        color: [0.0; 3],
        color_shift: [0.0; 3],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 250.0,
        step: [0.0, 0.0],
        ante: None,
        tilt: None,
    };
    t.stamp_dabs(&[dab(300.0)]);
    t.stamp_dabs(&[dab(340.0)]);
    t.canvas_rgba.to_vec()
}

/// Impasto real, Transform da tela inteira (levante), arrasto (cor + corpo).
fn transform() -> PainterTool {
    let mut t = tela(BrushSpec {
        radius_px: 30.0,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        impasto: true,
        ..Default::default()
    });
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    t.on_canvas_pointer(cp([80.0, 300.0], PointerPhase::Down));
    let mut x = 90.0;
    while x <= 560.0 {
        t.on_canvas_pointer(cp(
            [x, 300.0 + (x * 0.02).sin() * 120.0],
            PointerPhase::Move,
        ));
        x += 10.0;
    }
    t.on_canvas_pointer(cp([x, 300.0], PointerPhase::Up));
    t.set_paint_tool_mode("liquify");
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);
    t.set_rect_selection(0, 0, S, S);
    t
}
fn transform_age(t: &mut PainterTool) -> Vec<u8> {
    t.set_deform_transform_on(true);
    t.on_canvas_pointer(cp([320.0, 320.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([350.0, 330.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([350.0, 330.0], PointerPhase::Up));
    t.canvas_rgba.to_vec()
}

fn tids() -> BTreeSet<u64> {
    std::fs::read_dir("/proc/self/task")
        .map(|d| {
            d.filter_map(|e| e.ok()?.file_name().to_str()?.parse().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Fios do SO novos (fora do pool já aquecido) enquanto `f` corre.
fn fios_novos(f: impl FnOnce()) -> usize {
    rayon::broadcast(|_| ());
    let para = AtomicBool::new(false);
    std::thread::scope(|sc| {
        let vigia = sc.spawn(|| {
            let mut v = BTreeSet::new();
            while !para.load(Relaxed) {
                v.extend(tids());
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
            v
        });
        std::thread::sleep(std::time::Duration::from_millis(5));
        let antes = tids();
        f();
        std::thread::sleep(std::time::Duration::from_millis(2));
        para.store(true, Relaxed);
        vigia.join().unwrap().difference(&antes).count()
    })
}

type Cena = (
    &'static str,
    fn() -> PainterTool,
    fn(&mut PainterTool) -> Vec<u8>,
);
const CENAS: [Cena; 3] = [
    ("selecao", selecao, selecao_age),
    ("carimbo", carimbo, carimbo_age),
    ("transform", transform, transform_age),
];

#[test]
fn as_faixas_fora_do_compositor_nao_abrem_fios_do_so() {
    if std::thread::available_parallelism().map_or(1, |n| n.get()) < 2 {
        return;
    }
    // Controlo: as passadas desta família forçadas a uma faixa; o que sobra é de fora (o kernel do
    // carimbo no `ph2d-painter-brush`). Mínimo de 2 corridas contra o máximo de 2 do controlo.
    let fios = |forca: Option<usize>,
                prepara: fn() -> PainterTool,
                age: fn(&mut PainterTool) -> Vec<u8>| {
        FIOS_FORCADOS.with(|c| c.set(forca));
        let mut t = prepara();
        let n = fios_novos(|| drop(age(&mut t)));
        FIOS_FORCADOS.with(|c| c.set(None));
        n
    };
    let contas: Vec<(&str, usize, usize)> = CENAS
        .iter()
        .map(|&(nome, prepara, age)| {
            let prod = fios(None, prepara, age).min(fios(None, prepara, age));
            let controlo = fios(Some(1), prepara, age).max(fios(Some(1), prepara, age));
            (nome, prod, controlo)
        })
        .collect();
    assert!(
        contas.iter().all(|&(_, p, c)| p <= c),
        "fios do SO abertos fora do pool (cena, produção, controlo serial): {contas:?}"
    );
}

#[test]
fn as_faixas_fora_do_compositor_dao_o_serial_ao_bit() {
    for (nome, prepara, age) in CENAS {
        let com = |n| {
            FIOS_FORCADOS.with(|c| c.set(Some(n)));
            let v = age(&mut prepara());
            FIOS_FORCADOS.with(|c| c.set(None));
            v
        };
        let serial = com(1);
        assert!(
            serial.iter().any(|&b| b != 255),
            "{nome}: a cena não pintou"
        );
        for n in [2, 3, 7, 16, 32] {
            assert!(com(n) == serial, "{nome}: {n} faixas ≠ o serial");
        }
    }
}
