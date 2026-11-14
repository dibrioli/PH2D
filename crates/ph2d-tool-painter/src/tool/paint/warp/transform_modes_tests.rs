//! Gates do Transform por SUB-MODO (Uniform · Distort · Warp em malha): o corpo anda com a cor em todos,
//! entrar em Warp depois de mover o box não muda nada, e a malha de quatro cantos É o afim, borda incluída.
//! Missão 4K, bloco B06b (B06.2 · B06.3 · B06.A5).

use super::transform_geom::TransformFrame;
use super::transform_mesh::Mesh;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

pub(super) const W: u32 = 160;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um traço de Impasto REAL (cor + relevo) sobre branco, já no Transform.
pub(super) fn relief_in_transform() -> (PainterTool, crate::tool::RtLayerId) {
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

pub(super) fn drag(t: &mut PainterTool, from: [f32; 2], to: [f32; 2]) {
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
/// Três casos: rodado ~14° com escala 1,13 × 0,91; rodado ~30° ENCOLHIDO a 0,41 × 0,47 (aí meio texel
/// de ecrã passa de um texel de origem, e a margem já não esconde um teste feito fora do ponto amostrado);
/// e LONGE DA ORIGEM (canvas 2048, traço em x ≈ 1500): as homografias em coordenadas locais da célula
/// continuam a dar o afim lá (a costura dobrada tem o seu gate, [`a_malha_dobrada_nao_tem_costura_sem_dono`]).
///
/// **Mutações que têm de sangrar:** voltar o teste de cobertura a `[x+0,5; y+0,5]` (o encolhido) ou tirar
/// a margem de um texel de origem nas arestas de fora (todos).
#[test]
fn a_malha_de_quatro_cantos_e_o_afim_na_borda() {
    // (rotação cos, sin · escala x, y · deslocamento do centro)
    const PEQ: (u32, [u32; 4]) = (96, [24, 30, 70, 62]);
    let casos = [
        (
            "rodado",
            PEQ,
            [0.970_296f32, 0.241_922],
            [1.13f32, 0.91],
            [3.37f32, -2.61],
        ),
        (
            "encolhido",
            PEQ,
            [0.866_025, 0.5],
            [0.41, 0.47],
            [-1.73, 2.29],
        ),
        (
            "longe da origem",
            (2048, [1380, 1410, 1620, 1490]),
            [0.999_848, 0.017_452],
            [1.07, 0.96],
            [37.31, -21.57],
        ),
    ];
    let mut falhas = Vec::new();
    for (nome, (n, rect), rot, esc, desl) in casos {
        let (diff, worst) = malha_contra_afim(n, rect, rot, esc, desl);
        if diff != 0 {
            falhas.push(format!("[{nome}] {diff} texels (pior {worst})"));
        }
    }
    assert!(
        falhas.is_empty(),
        "a malha de 4 cantos difere do afim — a borda da malha não é a do afim: {falhas:?}"
    );
}

/// Pinta o mesmo box pelo afim e pela malha de 4 cantos; devolve `(texels com diferença > 1, pior)`.
fn malha_contra_afim(
    n: u32,
    [rx0, ry0, rx1, ry1]: [u32; 4],
    rot: [f32; 2],
    esc: [f32; 2],
    desl: [f32; 2],
) -> (usize, u8) {
    let mut src = vec![0u8; (n * n * 4) as usize];
    for y in ry0..ry1 {
        for x in rx0..rx1 {
            let i = ((y * n + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[200, 60, (x * 3) as u8, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, n, n);
    t.set_paint_tool_mode("liquify");
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);
    t.set_deform_transform_mode(1);
    let x = t.paint.deform.xform.expect("o levante");
    let cur = TransformFrame {
        center: [
            x.pristine.center[0] + desl[0],
            x.pristine.center[1] + desl[1],
        ],
        u: rot,
        hx: x.pristine.hx * esc[0],
        hy: x.pristine.hy * esc[1],
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
    println!(
        "[malha × afim {esc:?}] texels > 1: {diff} · pior {worst} · borda parcial do afim {parcial}"
    );
    assert!(
        parcial > 20,
        "fixture: o afim não tem borda antialiasada ({parcial})"
    );
    (diff, worst)
}

/// **B06.A5 — a costura sem dono.** Malha 3×3 DOBRADA (o ponto do meio de cima puxado), patch opaco: dentro
/// do traço nenhum texel pode ficar com a base (alfa 0) entre dois vizinhos opacos. Longe da origem
/// (canvas 2048, x ≈ 1500) uma homografia montada em f32 com coordenadas do CANVAS erra mais de um texel na
/// aresta partilhada, e o texel da costura fica fora das duas células (3 texels nesta fixture).
///
/// **Mutação que tem de sangrar:** `MeshCell::new` com a origem local a `[0, 0]` (coordenadas do canvas).
#[test]
fn a_malha_dobrada_nao_tem_costura_sem_dono() {
    let mut falhas = Vec::new();
    for (nome, n, [rx0, ry0, rx1, ry1]) in [
        ("perto", 256u32, [40u32, 90, 220, 140]),
        ("longe", 2048, [1300, 1400, 1700, 1460]),
    ] {
        let mut src = vec![0u8; (n * n * 4) as usize];
        for y in ry0..ry1 {
            for x in rx0..rx1 {
                let i = ((y * n + x) * 4) as usize;
                src[i..i + 4].copy_from_slice(&[200, 60, 50, 255]);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, n, n);
        t.set_paint_tool_mode("liquify");
        t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);
        t.set_deform_transform_mode(1);
        let x = t.paint.deform.xform.expect("o levante");
        let h = x.pristine.handles();
        let m = Mesh::seed([h[0], h[1], h[2], h[3]], 3, 3);
        let mut current: Vec<[f32; 2]> = m
            .current
            .iter()
            .map(|p| [p[0] + 17.37, p[1] + 9.61])
            .collect();
        // A linha de cima NA IMAGEM é a última (o `TL` do frame tem `+hy`, para baixo).
        current[13][1] -= 23.3;
        current[14][1] -= 11.9;
        t.paint.deform.transform_mode = 3;
        t.paint.deform.xform_mesh = Some(Mesh {
            cols: 3,
            rows: 3,
            pristine: m.pristine,
            current,
        });
        t.recomposite_transform();
        let a = |x: u32, y: u32| t.canvas_rgba[((y * n + x) * 4 + 3) as usize];
        let mut buracos = 0usize;
        for y in 1..n - 1 {
            for x in 1..n - 1 {
                if a(x, y) == 0
                    && ((a(x - 1, y) == 255 && a(x + 1, y) == 255)
                        || (a(x, y - 1) == 255 && a(x, y + 1) == 255))
                {
                    buracos += 1;
                }
            }
        }
        println!("[costura {nome}] texels sem dono entre opacos: {buracos}");
        if buracos != 0 {
            falhas.push(format!("[{nome}] {buracos}"));
        }
    }
    assert!(
        falhas.is_empty(),
        "a malha dobrada deixa texels sem dono na costura entre células: {falhas:?}"
    );
}

/// B17.6 (F1o): o Transform de uma SELEÇÃO (afim e Warp) forka a tela pelo `dirty` (a caixa de antes ∪ a de
/// agora), não pela tela: todo texel mudado está no journal, e o journal é menos de metade da tela.
#[test]
fn o_transform_de_uma_selecao_captura_so_o_dirty() {
    const N: u32 = 512;
    for (nome, warp) in [("afim", false), ("warp", true)] {
        let mut src = vec![255u8; (N * N * 4) as usize];
        for (i, p) in src.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            if (i % N as usize / 8).is_multiple_of(2) {
                p[..3].copy_from_slice(&[20, 40, 200]);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, N, N);
        t.set_rect_selection(100, 120, 60, 40);
        t.set_paint_tool_mode("liquify");
        let before: Vec<u8> = t.canvas_rgba.to_vec();
        t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);
        t.set_deform_transform_mode(1);
        let x = t.paint.deform.xform.expect("o levante");
        let mut cur = x.pristine;
        cur.center = [cur.center[0] + 7.0, cur.center[1] + 5.0];
        t.paint.deform.xform = Some(super::transform::Xform { current: cur, ..x });
        if warp {
            let corners = |f: &TransformFrame| {
                let h = f.handles();
                [h[0], h[1], h[2], h[3]]
            };
            t.paint.deform.transform_mode = 3;
            t.paint.deform.xform_mesh = Some(Mesh {
                cols: 3,
                rows: 3,
                pristine: Mesh::seed(corners(&x.pristine), 3, 3).pristine,
                current: Mesh::seed(corners(&cur), 3, 3).current,
            });
        }
        t.recomposite_transform();
        let (mut changed, mut undescribed) = (0usize, 0usize);
        for (i, (&a, &b)) in before.iter().zip(t.canvas_rgba.iter()).enumerate() {
            if a != b {
                changed += 1;
                if t.undo.write_state.canvas_before(i) != Some(a) {
                    undescribed += 1;
                }
            }
        }
        assert!(changed > 0, "{nome}: controlo — o transform não mexeu");
        assert_eq!(
            undescribed, 0,
            "{nome}: {undescribed}/{changed} texels fora do journal"
        );
        let retido = t.undo.write_state.canvas_journal().heap_bytes();
        assert!(
            retido * 2 < before.len(),
            "{nome}: o journal reteve {retido} B de uma tela de {} B (a tela inteira)",
            before.len()
        );
        println!("[transform {nome}] mudou {changed} B, journal {retido} B");
    }
}
