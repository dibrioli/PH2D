//! Gates do Transform por SUB-MODO (Uniform · Distort · Warp em malha): o corpo anda com a cor em todos,
//! entrar em Warp depois de mover o box não muda nada, e a malha de quatro cantos É o afim, borda incluída.
//! Missão 4K, bloco B06b (B06.2 · B06.3 · B06.A5).

use super::transform_geom::TransformFrame;
use super::transform_mesh::Mesh;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

const W: u32 = 160;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um traço de Impasto REAL (cor + relevo) sobre branco, já no Transform.
fn relief_in_transform() -> (PainterTool, crate::tool::RtLayerId) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * W * 4) as usize], W, W);
    let b = BrushSpec {
        radius_px: 14.0,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        impasto: true,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    let layer = t.layers.active().expect("a layer");
    t.on_canvas_pointer(cp([50.0, 80.0], PointerPhase::Down));
    let mut x = 54.0;
    while x <= 110.0 {
        t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Move));
        x += 4.0;
    }
    t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Up));
    t.set_paint_tool_mode("liquify");
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);
    (t, layer)
}

fn drag(t: &mut PainterTool, from: [f32; 2], to: [f32; 2]) {
    t.on_canvas_pointer(cp(from, PointerPhase::Down));
    t.on_canvas_pointer(cp(to, PointerPhase::Move));
    t.on_canvas_pointer(cp(to, PointerPhase::Up));
}

/// Centroide pesado `(x, y)`; `None` quando não há nada a pesar.
fn centroid(vals: impl Iterator<Item = f32>) -> Option<[f64; 2]> {
    let (mut sx, mut sy, mut ws) = (0.0f64, 0.0f64, 0.0f64);
    for (i, v) in vals.enumerate() {
        let v = f64::from(v);
        sx += v * (i as u32 % W) as f64;
        sy += v * (i as u32 / W) as f64;
        ws += v;
    }
    (ws > 0.0).then(|| [sx / ws, sy / ws])
}

/// O peso da tinta é o PIGMENTO (déficit de verde sobre branco opaco), não o alfa (a moldura).
fn ink(t: &PainterTool) -> [f64; 2] {
    centroid(t.canvas_rgba.as_chunks::<4>().0.iter().map(|p| {
        let g = 255.0 - f32::from(p[1]);
        if g > 8.0 { g } else { 0.0 }
    }))
    .expect("fixture: a tinta existe")
}

/// O peso do corpo é a altura onde há TINTA (cobertura): a luz pesa a altura pela cobertura.
fn body(t: &PainterTool, l: crate::tool::RtLayerId) -> [f64; 2] {
    let h = t.heights.get(&l).expect("planos");
    let c = t.covers.get(&l).expect("planos");
    centroid(
        h.iter()
            .zip(c.iter())
            .map(|(h, c)| h.max(0.0) * f32::from(*c) / 255.0),
    )
    .expect("fixture: o relevo existe")
}

/// **O corpo viaja com a cor em TODO sub-modo do Transform** — o report do Enio (2026-08-08) curado no
/// afim; o Warp em malha (B06.2, T1-37) movia a tinta e deixava o corpo onde estava.
///
/// Gesto por modo: Uniform arrasta o centro; Distort, um canto; Warp, um ponto de controlo de dentro.
/// O oráculo: o DESLOCAMENTO do centroide do corpo = o da tinta, ≤ 0,5 px em x e em y.
///
/// **Mutação que tem de sangrar:** tirar a chamada ao relevo do `composite_mesh` (ou do `composite_transform`).
#[test]
fn the_transform_carries_the_body_where_it_carries_the_colour() {
    // (modo, de, para): o ponto (1,1) da malha 3×3 sobre a tela inteira está em (53,3; 53,3).
    let w3 = W as f32 / 3.0;
    let cases: [(&str, u8, [f32; 2], [f32; 2]); 3] = [
        ("Uniform", 0, [80.0, 80.0], [100.0, 86.0]),
        ("Distort", 2, [0.0, 0.0], [30.0, 24.0]),
        ("Warp", 3, [w3, w3], [w3 + 18.0, w3 + 22.0]),
    ];
    for (name, mode, from, to) in cases {
        let (mut t, layer) = relief_in_transform();
        t.set_deform_transform_mode(mode);
        let (i0, b0) = (ink(&t), body(&t, layer));
        drag(&mut t, from, to);
        let (i1, b1) = (ink(&t), body(&t, layer));
        let di = [i1[0] - i0[0], i1[1] - i0[1]];
        let db = [b1[0] - b0[0], b1[1] - b0[1]];
        println!("[{name}] tinta {di:.3?} · corpo {db:.3?}");
        assert!(
            di[0].abs() + di[1].abs() > 2.0,
            "fixture [{name}]: o gesto não moveu a TINTA ({di:.2?})"
        );
        assert!(
            (db[0] - di[0]).abs() <= 0.5 && (db[1] - di[1]).abs() <= 0.5,
            "[{name}] o Transform moveu a tinta {di:.2?} px e o CORPO {db:.2?} px — a luz sombreia \
             uma crista que não está mais lá"
        );
    }
}

/// **Entrar em Warp depois de mover o box não muda a tela, e o 1.º arrasto parte DALI** (B06.3, T1-38).
///
/// A malha era semeada com `pristine == current == box movido`: o mapa do 1.º arrasto ia buscar o patch
/// ONDE O BOX ESTÁ AGORA (o vazio, ou a tinta de antes do movimento), e a tinta saltava para a origem.
/// Um toque de 0,5 px num canto longe do traço tem de deixar a tinta onde o Uniform a pôs.
///
/// **Mutação que tem de sangrar:** semear o `pristine` da malha com o box ACTUAL (`box4(&cur)`).
#[test]
fn entering_warp_after_moving_the_box_keeps_the_paint_where_it_was() {
    let (mut t, _) = relief_in_transform();
    t.set_deform_transform_mode(0);
    drag(&mut t, [80.0, 80.0], [100.0, 86.0]);
    let moved = ink(&t);
    t.set_deform_transform_mode(3);
    assert_eq!(ink(&t), moved, "entrar em Warp não pode escrever na tela");
    // O canto TL do box movido está em (20, 6): toque de 0,5 px.
    drag(&mut t, [20.0, 6.0], [20.5, 6.0]);
    let after = ink(&t);
    let d = [after[0] - moved[0], after[1] - moved[1]];
    assert!(
        d[0].abs() <= 0.5 && d[1].abs() <= 0.5,
        "o 1.º arrasto do Warp depois de mover o box saltou a tinta {d:.2?} px — a malha amostrou o \
         patch no sítio errado"
    );
}

/// **A malha de quatro cantos é o afim, BORDA incluída** (B06.A5, AU09-8).
///
/// A malha testava a cobertura no centro do texel (`x+0,5`) e amostrava no canto (`x`), sem deixar o
/// alfa decidir a borda: serrilha e meio texel contra o afim com a mesma geometria. Fixture fora dos
/// eixos (rodada, escala e posição não inteiras), patch opaco sobre transparente: a malha seguida pelo
/// afim do mesmo box tem de dar os MESMOS bytes (±1 do arredondamento das homografias por célula).
///
/// **Mutação que tem de sangrar:** voltar o teste de cobertura a `[x+0,5; y+0,5]` ou tirar a margem de
/// um texel de origem nas arestas de fora.
#[test]
fn a_malha_de_quatro_cantos_e_o_afim_na_borda() {
    const N: u32 = 96;
    let mut src = vec![0u8; (N * N * 4) as usize];
    for y in 30..62u32 {
        for x in 24..70u32 {
            let i = ((y * N + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[200, 60, (x * 3) as u8, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, N, N);
    t.set_paint_tool_mode("liquify");
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);
    t.set_deform_transform_mode(1);
    let x = t.paint.deform.xform.expect("o levante");
    // Rodada de ~14°, escala 1,13 × 0,91, centro deslocado de (3,37; −2,61).
    let (c, s) = (0.970_296f32, 0.241_922f32);
    let cur = TransformFrame {
        center: [x.pristine.center[0] + 3.37, x.pristine.center[1] - 2.61],
        u: [c, s],
        hx: x.pristine.hx * 1.13,
        hy: x.pristine.hy * 0.91,
    };
    t.paint.deform.xform = Some(super::transform::Xform { current: cur, ..x });
    t.recomposite_transform();
    let afim = t.canvas_rgba.to_vec();

    let corners = |f: &TransformFrame| {
        let h = f.handles();
        [h[0], h[1], h[2], h[3]]
    };
    let pristine = Mesh::seed(corners(&x.pristine), 3, 3).pristine;
    let current = Mesh::seed(corners(&cur), 3, 3).current;
    t.paint.deform.transform_mode = 3;
    t.paint.deform.xform_mesh = Some(Mesh {
        cols: 3,
        rows: 3,
        pristine,
        current,
    });
    t.recomposite_transform();
    let malha = t.canvas_rgba.to_vec();

    let (mut diff, mut worst, mut parcial) = (0usize, 0u8, 0usize);
    for (a, m) in afim.as_chunks::<4>().0.iter().zip(malha.as_chunks::<4>().0) {
        if a[3] > 0 && a[3] < 255 {
            parcial += 1;
        }
        let d = (0..4).map(|k| a[k].abs_diff(m[k])).max().unwrap_or(0);
        if d > 1 {
            diff += 1;
        }
        worst = worst.max(d);
    }
    println!("[malha × afim] texels > 1: {diff} · pior {worst} · borda parcial do afim {parcial}");
    assert!(
        parcial > 50,
        "fixture: o afim não tem borda antialiasada ({parcial})"
    );
    assert_eq!(
        diff, 0,
        "a malha de 4 cantos difere do afim em {diff} texels (pior {worst}) — a borda da malha não é a do afim"
    );
}
