//! B15.2: numa tela sem papel, a luz só dobra as colunas onde há relevo — ao bit com o passe por píxel (o oráculo
//! verbatim de antes das faixas) e com o passe por faixas sem poda.

use super::super::Region;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Tela branca SEM papel, um traço de Impasto de `diametro` de `de` a `ate` (fica aberto se `aberto`).
fn tela_nova_com_traco(
    w: u32,
    h: u32,
    diametro: f32,
    de: [f32; 2],
    ate: [f32; 2],
    aberto: bool,
) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.toggle_brush_impasto();
    t.set_brush_size_px(diametro);
    t.paint.brush.color = [0.2, 0.4, 0.8];
    let at = |p: [f32; 2], phase| CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(de, PointerPhase::Down));
    let mid = [(de[0] + ate[0]) * 0.5, (de[1] + ate[1]) * 0.5];
    t.on_canvas_pointer(at(mid, PointerPhase::Move));
    t.on_canvas_pointer(at(ate, PointerPhase::Move));
    if !aberto {
        t.on_canvas_pointer(at(ate, PointerPhase::Up));
    }
    t
}

fn recorte(t: &PainterTool, r: Region) -> Vec<u8> {
    let w = t.source_size.0 as usize;
    (r.y..r.y + r.h)
        .flat_map(|y| {
            let i = (y as usize * w + r.x as usize) * 4;
            t.canvas_rgba[i..i + r.w as usize * 4].to_vec()
        })
        .collect()
}

/// ⭐ **O 1.º quadro de tela nova só dobra o relevo que existe** (B15.2: 376,88 ms no 1.º quadro r60 4K). Sem papel, o
/// passe da tela inteira dobra as faixas e colunas do traço (+1 px) e mais nada, e o desenho é o do passe por píxel ao
/// bit: traço fechado no meio, traço ABERTO (o relevo vivo), traço que toca a borda esquerda e a de baixo, uma
/// sub-região fora da origem. *Mutação provada INERTE (B15.2): a dilatação de 1 coluna a 0 dá o mesmo bit — a coluna
/// vizinha do relevo não tem tinta (`body = 0` sem papel) e o `shade` já a devolve intacta; fica como margem.*
#[test]
fn a_luz_da_tela_nova_so_dobra_o_relevo_ao_bit() {
    let (w, h) = (300u32, 200u32);
    let casos = [
        ("meio", [120.0, 100.0], [180.0, 110.0], false),
        ("aberto", [120.0, 100.0], [180.0, 110.0], true),
        ("borda", [2.0, 190.0], [40.0, 198.0], false),
    ];
    let mut alguma_poda = false;
    for (nome, de, ate, aberto) in casos {
        let t = tela_nova_com_traco(w, h, 24.0, de, ate, aberto);
        assert!(t.substrate().is_none(), "{nome}: a tela nova não tem papel");
        for r in [
            Region { x: 0, y: 0, w, h },
            Region {
                x: 13,
                y: 71,
                w: 211,
                h: 120,
            },
        ] {
            let base = recorte(&t, r);
            let mut oraculo = base.clone();
            t.apply_impasto_light_por_pixel(&mut oraculo, r);
            assert!(oraculo != base, "{nome} {r:?}: a fixture não acendeu nada");
            let mut inteira = base.clone();
            let todos = t.acende_contando(&mut inteira, r, super::FAIXA_LINHAS, false);
            let mut podada = base.clone();
            let dobrados = t.acende_contando(&mut podada, r, super::FAIXA_LINHAS, true);
            assert!(
                inteira == oraculo,
                "{nome} {r:?}: as faixas sem poda diferem do passe por píxel"
            );
            assert!(podada == oraculo, "{nome} {r:?}: a poda mudou o desenho");
            assert_eq!(
                todos,
                (r.w * r.h) as usize,
                "{nome} {r:?}: sem poda dobra-se tudo"
            );
            // A região inteira: o traço é < ⅓ da tela. A sub-região: ele cobre ~⅓ dela, basta que pode.
            let tecto = if r.w == w { todos / 3 } else { todos / 2 };
            assert!(
                dobrados < tecto,
                "{nome} {r:?}: a tela nova dobrou {dobrados} de {todos} texels — o plano sem relevo não foi podado"
            );
            alguma_poda |= dobrados < todos;
        }
    }
    assert!(alguma_poda);
}

/// Sonda: o passe de luz da tela 4K nova inteira (o 1.º quadro) por raio, sem poda × com poda, intercalados, o MÍNIMO
/// de 5, mais os texels dobrados (o relógio está sujo pela carga; o contador não).
/// `cargo test --profile smoke -p ph2d-tool-painter --lib sonda_luz_da_tela_nova_4k -- --ignored --nocapture`
#[test]
#[ignore = "sonda de custo"]
fn sonda_luz_da_tela_nova_4k() {
    let (w, h) = (3840u32, 2160u32);
    let full = Region { x: 0, y: 0, w, h };
    for raio in [15.0f32, 60.0, 200.0] {
        let t = tela_nova_com_traco(w, h, raio * 2.0, [1200.0, 1000.0], [2600.0, 1150.0], false);
        let (mut antes, mut depois, mut n0, mut n1) = (f64::MAX, f64::MAX, 0, 0);
        for _ in 0..5 {
            for podar in [false, true] {
                let mut px = t.canvas_rgba.to_vec();
                let t0 = std::time::Instant::now();
                let n = t.acende_contando(&mut px, full, super::FAIXA_LINHAS, podar);
                let ms = t0.elapsed().as_secs_f64() * 1e3;
                if podar {
                    (depois, n1) = (depois.min(ms), n);
                } else {
                    (antes, n0) = (antes.min(ms), n);
                }
            }
        }
        eprintln!(
            "[luz tela nova 4K r{raio}] sem poda {antes:.2} ms ({n0} texels) · com poda {depois:.2} ms ({n1} texels)"
        );
    }
}
