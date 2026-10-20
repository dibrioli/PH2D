//! ⭐ **A ponta de N IMAGENS** (L19, `docs/Painter/Pinceis/03_MOTORES.md` §4) pela porta do ponteiro, no Digital: a
//! imagem única é o carimbo de hoje ao bit, o sorteio respeita o PESO (a lei medida no Krita `Stamp Herbals`: 3
//! imagens, A em 10/15 — sorteio com peso, não ciclo), e o preset com N imagens volta ao bit.

use ph2d_brush_presets::{Miniatura, Origem, Parametros, Preset};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{LinhaDaPonta, PaintMedia, PainterTool, nome_do_meio};

const W: usize = 640;
const H: usize = 160;
const LADO: u32 = 24;

/// Uma imagem `LADO × LADO` opaca da cor `c` (um quadrado, um disco ou um losango: as três formas da ponta).
fn imagem(c: [u8; 3], forma: u8) -> Vec<u8> {
    let m = LADO as f32 / 2.0;
    (0..LADO * LADO)
        .flat_map(|i| {
            let (x, y) = ((i % LADO) as f32 + 0.5 - m, (i / LADO) as f32 + 0.5 - m);
            let dentro = match forma {
                0 => true,
                1 => x * x + y * y < m * m,
                _ => x.abs() + y.abs() < m,
            };
            if dentro { [c[0], c[1], c[2], 255] } else { [0, 0, 0, 0] }
        })
        .collect()
}

const CORES: [[u8; 3]; 3] = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];

/// O pincel Digital com a Shape das `n` imagens (cada uma com a SUA cor, a silhueta pelo alpha), raio 10. `forma`
/// varia a forma por imagem (quadrado · disco · losango); sem ela, três quadrados (a régua conta área igual).
fn pincel(n: usize, f: impl Fn(&mut PainterTool)) -> PainterTool {
    pincel_com(n, false, f)
}

fn pincel_com(n: usize, forma: bool, f: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(10.0);
    t.set_brush_spacing(1.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_shape_image_rgba(&imagem(CORES[0], 0), LADO, LADO, None);
    t.toggle_brush_shape_alpha_from_image();
    t.toggle_brush_shape_per_layer_color();
    for k in 1..n {
        t.add_brush_shape_hose_image_rgba(&imagem(CORES[k], if forma { k as u8 } else { 0 }), LADO, LADO);
    }
    f(&mut t);
    t
}

/// Quatro traços rectos (y 30, 65, 100, 135) de x 20 a 620; a tela.
fn pinta(mut t: PainterTool) -> Vec<u8> {
    for y in [30.0, 65.0, 100.0, 135.0] {
        let cp = |x: f32, phase| CanvasPointer {
            pos: [x, y],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
        let mut x = 20.0;
        while x < 620.0 {
            x += 4.0;
            t.on_canvas_pointer(cp(x, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp(x, PointerPhase::Up));
    }
    t.run_full().0
}

/// Os píxeis de cada cor primária (o canal dominante, os outros baixos).
fn por_cor(px: &[u8]) -> [usize; 3] {
    let mut c = [0; 3];
    for p in px.chunks_exact(4) {
        for (k, cor) in CORES.iter().enumerate() {
            if (0..3).all(|j| if cor[j] == 255 { p[j] > 200 } else { p[j] < 80 }) {
                c[k] += 1;
            }
        }
    }
    c
}

/// ⭐ **A imagem única é o carimbo de hoje** (ao bit): acrescentar imagens e tirá-las, ou mexer no modo e nos pesos
/// com uma imagem só, não muda um byte da tinta.
#[test]
fn uma_imagem_e_o_carimbo_de_hoje() {
    let hoje = pinta(pincel(1, |_| {}));
    let limpa = pinta(pincel(3, |t| t.clear_brush_shape_hose()));
    let mexida = pinta(pincel(1, |t| {
        t.set_ponta(LinhaDaPonta::HoseMode, 1.0);
        t.set_ponta(LinhaDaPonta::Peso(0), 7.0);
    }));
    assert!(por_cor(&hoje)[0] > 1000, "a imagem não pintou");
    assert!(hoje == limpa, "acrescentar e tirar imagens mudou a tinta");
    assert!(hoje == mexida, "o modo e o peso com uma imagem mudaram a tinta");
}

/// ⭐ **O sorteio respeita o PESO**: pesos 4 · 2 · 1 nas três imagens — a parte da tinta de cada uma (a mesma
/// área por carimbo) é o peso ÷ 7, dentro de 4 desvios binomiais sobre os carimbos do traço; em sequência, terços.
#[test]
fn o_sorteio_respeita_o_peso() {
    let c = por_cor(&pinta(pincel(3, |t| {
        t.set_ponta(LinhaDaPonta::Peso(0), 4.0);
        t.set_ponta(LinhaDaPonta::Peso(1), 2.0);
        t.set_ponta(LinhaDaPonta::Peso(2), 1.0);
    })));
    let carimbos: Vec<f64> = (0..3).map(|k| c[k] as f64).collect();
    let n: f64 = carimbos.iter().sum();
    let estampas = n / (20.0 * 20.0); // o carimbo de raio 10: 20 × 20 px o quadrado
    println!("[mangueira] píxeis {c:?} · carimbos ≈ {estampas:.0}");
    for (k, peso) in [4.0, 2.0, 1.0].into_iter().enumerate() {
        let p = peso / 7.0;
        let medido = carimbos[k] / n;
        let sd = (p * (1.0 - p) / estampas).sqrt();
        assert!((medido - p).abs() < 4.0 * sd + 0.02, "imagem {k}: {medido:.3} × {p:.3} ± {sd:.3}");
    }
    let s = por_cor(&pinta(pincel(3, |t| t.set_ponta(LinhaDaPonta::HoseMode, 1.0))));
    let tot: f64 = s.iter().map(|&v| v as f64).sum();
    assert!(
        s.iter().all(|&v| (v as f64 / tot - 1.0 / 3.0).abs() < 0.04),
        "a sequência não é em terços: {s:?}"
    );
}

/// ⭐ **Gravar → carregar com N imagens volta ao bit**: o preset leva as três imagens (com a cor), o modo e os pesos;
/// aplicado a um pincel novo pinta a MESMA tela. E a ponta TINGIDA (a cor do pincel) também.
#[test]
fn gravar_e_carregar_com_n_imagens_volta_ao_bit() {
    for cor in [true, false] {
        let arma = |t: &mut PainterTool| {
            t.set_ponta(LinhaDaPonta::Peso(0), 3.0);
            t.set_ponta(LinhaDaPonta::Peso(2), 5.0);
            if !cor {
                t.toggle_brush_shape_per_layer_color();
            }
        };
        // A cor do pincel é da OBRA (o preset não a leva): branca com a cor das imagens, preta tingida.
        let tinta = if cor { [255, 255, 255] } else { [0, 0, 0] };
        let arma = |t: &mut PainterTool| {
            arma(t);
            t.set_brush_color_srgb8(tinta);
        };
        let antes = pinta(pincel_com(3, true, arma));
        let fonte = pincel_com(3, true, arma);
        let (parametros, texturas) =
            fonte.preset_capturar(&Parametros::novo(nome_do_meio(PaintMedia::Digital), "Ponta", "N"));
        let p = Preset {
            parametros,
            origem: Origem::Utilizador,
            miniatura: Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![0; 4],
            },
            texturas,
        };
        let lido = ph2d_brush_presets::ler(&ph2d_brush_presets::escrever(&p).unwrap()).unwrap();
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_brush_color_srgb8(tinta);
        assert!(t.preset_aplicar(&lido).unwrap().is_empty(), "texturas por ler");
        assert_eq!(t.brush_shape_hose_count(), 3);
        t.set_brush_size_px(10.0);
        let depois = pinta(t);
        assert!(antes == depois, "cor {cor}: o preset com 3 imagens pinta outra tela");
    }
}

/// A ponta de N imagens AGE em todo meio que carimba a Shape: três formas (quadrado · disco · losango) sorteadas
/// pintam outra tela que a imagem única, com a mesma tinta (tingida). Mede quais meios a leem.
#[test]
fn a_ponta_de_n_imagens_age_nos_meios() {
    for m in [PaintMedia::Digital, PaintMedia::Impasto, PaintMedia::WetPaint, PaintMedia::Watercolor] {
        let pinta_no = |n: usize| {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
            t.set_paint_media(m);
            t.set_brush_size_px(10.0);
            t.set_brush_spacing(1.0);
            t.set_brush_color_srgb8([0, 0, 0]);
            t.set_brush_shape_image_rgba(&imagem([255, 255, 255], 0), LADO, LADO, None);
            t.toggle_brush_shape_alpha_from_image();
            for k in 1..n {
                t.add_brush_shape_hose_image_rgba(&imagem([255, 255, 255], k as u8), LADO, LADO);
            }
            pinta(t)
        };
        let (um, tres) = (pinta_no(1), pinta_no(3));
        let n = um.iter().zip(&tres).filter(|(a, b)| a != b).count();
        println!("[mangueira] {m:?}: {n} bytes mudam com 3 imagens");
        if m != PaintMedia::Watercolor {
            assert!(n > 2000, "{m:?}: a ponta de 3 imagens mudou só {n} bytes");
        }
    }
}
