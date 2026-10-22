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
            if dentro {
                [c[0], c[1], c[2], 255]
            } else {
                [0, 0, 0, 0]
            }
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
        t.add_brush_shape_hose_image_rgba(
            &imagem(CORES[k], if forma { k as u8 } else { 0 }),
            LADO,
            LADO,
        );
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
    assert!(
        hoje == mexida,
        "o modo e o peso com uma imagem mudaram a tinta"
    );
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
        assert!(
            (medido - p).abs() < 4.0 * sd + 0.02,
            "imagem {k}: {medido:.3} × {p:.3} ± {sd:.3}"
        );
    }
    let s = por_cor(&pinta(pincel(3, |t| {
        t.set_ponta(LinhaDaPonta::HoseMode, 1.0)
    })));
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
        let (parametros, texturas) = fonte.preset_capturar(&Parametros::novo(
            nome_do_meio(PaintMedia::Digital),
            "Ponta",
            "N",
        ));
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
        assert!(
            t.preset_aplicar(&lido).unwrap().is_empty(),
            "texturas por ler"
        );
        assert_eq!(t.brush_shape_hose_count(), 3);
        t.set_brush_size_px(10.0);
        let depois = pinta(t);
        assert!(
            antes == depois,
            "cor {cor}: o preset com 3 imagens pinta outra tela"
        );
    }
}

/// Um TOQUE com a pressão `p` em `pos`: o pen-down e um passo de 1 px (o Wet deposita no movimento).
fn toque(t: &mut PainterTool, pos: [f32; 2], p: f32) {
    let cp = |dx: f32, phase| CanvasPointer {
        pos: [pos[0] + dx, pos[1]],
        pressure: p,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(0.0, PointerPhase::Down));
    t.on_tick(8.0);
    t.on_canvas_pointer(cp(1.0, PointerPhase::Move));
    t.on_tick(8.0);
    t.on_canvas_pointer(cp(1.0, PointerPhase::Up));
    t.on_tick(8.0);
}

/// O pincel de raio 30 do meio `m` com a Shape das imagens `formas` (brancas, a silhueta pelo alpha), a ponta pela
/// PRESSÃO; os toques `(x, p)` em y 80; a tela depois de 30 quadros.
fn toques_no_meio(m: PaintMedia, formas: &[u8], toques: &[(f32, f32)]) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(m);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_shape_image_rgba(&imagem([255, 255, 255], formas[0]), LADO, LADO, None);
    t.toggle_brush_shape_alpha_from_image();
    for &f in &formas[1..] {
        t.add_brush_shape_hose_image_rgba(&imagem([255, 255, 255], f), LADO, LADO);
    }
    t.set_ponta(LinhaDaPonta::HoseMode, 2.0); // pela pressão: < 0,5 a 1.ª, ≥ 0,5 a 2.ª
    for &(x, p) in toques {
        toque(&mut t, [x, 80.0], p);
    }
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    t.run_full().0
}

/// O recorte `±24` px à volta de `(x, 80)`.
fn recorte(px: &[u8], x: usize) -> Vec<u8> {
    (56..105)
        .flat_map(|y| px[(y * W + x - 24) * 4..(y * W + x + 25) * 4].to_vec())
        .collect()
}

/// ⭐ **Cada meio carimba a imagem ESCOLHIDA**: a ponta de duas imagens (quadrado, losango) pela pressão — o toque
/// leve tem a silhueta do toque da Shape só com o quadrado, e o forte a da Shape só com o losango (< 5 % da
/// diferença entre os dois, o controlo) — no Digital, no Impasto e no Wet. A Watercolor «Automatic» não lê a Shape: as duas imagens pintam
/// o mesmo carimbo (o controlo inverte). Antes do adiado do Wet passar pela ponta, o Wet carimbava sempre a 1.ª.
#[test]
fn a_ponta_de_n_imagens_age_nos_meios() {
    for m in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
        PaintMedia::Watercolor,
    ] {
        let ponta = toques_no_meio(m, &[0, 2], &[(160.0, 0.3), (480.0, 0.9)]);
        let quadrado = toques_no_meio(m, &[0], &[(160.0, 0.3)]);
        let losango = toques_no_meio(m, &[2], &[(480.0, 0.9)]);
        let quadrado_forte = recorte(&toques_no_meio(m, &[0], &[(480.0, 0.9)]), 480);
        let (pa, pb) = (recorte(&ponta, 160), recorte(&ponta, 480));
        let (qa, lb) = (recorte(&quadrado, 160), recorte(&losango, 480));
        // A silhueta: os píxeis mais escuros que o papel em ≥ 30 % da tinta mais escura do recorte (a aguada do Wet é
        // leve). A rota por dab (a ponta de N imagens leva a imagem na pegada) arredonda a orla diferente da cache
        // da imagem única — por isso a régua é a SILHUETA e não o byte.
        let silhueta = |c: &[u8]| -> Vec<bool> {
            let minimo = c
                .chunks_exact(4)
                .map(|p| i32::from(p[0]))
                .min()
                .unwrap_or(255);
            let limiar = 255 - (255 - minimo) * 3 / 10;
            c.chunks_exact(4)
                .map(|p| i32::from(p[0]) < limiar)
                .collect()
        };
        let difere = |a: &[u8], b: &[u8]| {
            silhueta(a)
                .iter()
                .zip(silhueta(b))
                .filter(|(x, y)| **x != *y)
                .count()
        };
        println!(
            "[mangueira] {m:?}: silhueta leve×quadrado {} · forte×losango {} · quadrado×losango {} px",
            difere(&pa, &qa),
            difere(&pb, &lb),
            difere(&quadrado_forte, &lb)
        );
        let controlo = difere(&quadrado_forte, &lb);
        if m == PaintMedia::Watercolor {
            assert_eq!(
                controlo, 0,
                "a Watercolor «Automatic» passou a ler a Shape?"
            );
            continue;
        }
        // O Wet espalha a tinta do toque: o controlo dele é menor (36 px medidos) e continua a separar.
        assert!(
            controlo > 20,
            "{m:?}: o quadrado e o losango pintam o mesmo (o controlo)"
        );
        assert!(
            difere(&pa, &qa) * 20 < controlo,
            "{m:?}: o toque leve não é o quadrado"
        );
        assert!(
            difere(&pb, &lb) * 20 < controlo,
            "{m:?}: o toque forte não é o losango"
        );
    }
}

/// ⭐ **Gravar «Add image» e tocar = a mesma ponta, sem pedido pendente**: a macro grava o clique, o pedido
/// consumido e a IMAGEM acrescentada (RGBA, como a da Shape); o tocador acrescenta-a sem abrir o seletor.
#[test]
fn gravar_add_image_e_tocar_e_a_mesma_ponta() {
    use super::macro_comum::{Gravador, toca};
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = pincel(1, |_| {});
    let mut g = Gravador::arma(&mut t);
    t.handle_panel_event(PanelEvent::Click(
        ph2d_tool_painter::ids::PAINTER_SHAPE_HOSE_ADD,
    ));
    g.quadro(&mut t);
    assert!(
        t.take_brush_shape_hose_request(),
        "o «Add image» não pediu o seletor"
    );
    t.add_brush_shape_hose_image_rgba(&imagem(CORES[1], 1), LADO, LADO);
    // O menu «Image order» pelo canal do painel (o que a macro grava): em sequência.
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_tool_painter::ids::PAINTER_PONTA_FIELDS[5],
        "1".into(),
    ));
    g.quadro(&mut t);
    let doc = g.para(&mut t);
    let mut toc = toca(&doc).expect("a macro toca");
    let tocado = toc.tool_mut();
    assert!(
        !tocado.take_brush_shape_hose_request(),
        "ficou um pedido de seletor pendente ao tocar"
    );
    assert_eq!(tocado.brush_shape_hose_count(), 2);
    assert_eq!(tocado.brush_shape_hose_count(), t.brush_shape_hose_count());
    let mut a = std::mem::take(tocado);
    a.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    let mut b = t;
    b.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    assert!(pinta(a) == pinta(b), "a ponta tocada pinta outra tela");
}

/// ⭐ **No Wet o lote ADIADO carimba a imagem escolhida**: com o worker em voo (o ritmo escrito do `macro_comum`)
/// parte dos lotes fica na fila e pousa no tique, fora da porta do lote; a ponta de duas imagens (todo dab escolhe
/// o losango, pela pressão) pinta a silhueta da Shape só com o losango. Antes, os adiados pousavam o quadrado.
#[test]
fn no_wet_o_lote_adiado_carimba_a_imagem_escolhida() {
    let traco = |formas: &[u8]| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_wet_ritmo_de_teste(Some(super::macro_comum::ritmo()));
        t.set_wet_agua_viva(true); // a água corre com o traço aberto: o motor anda em voo e há lotes adiados
        t.set_brush_size_px(30.0);
        t.set_brush_color_srgb8([0, 0, 0]);
        t.set_brush_shape_image_rgba(&imagem([255, 255, 255], formas[0]), LADO, LADO, None);
        t.toggle_brush_shape_alpha_from_image();
        for &f in &formas[1..] {
            t.add_brush_shape_hose_image_rgba(&imagem([255, 255, 255], f), LADO, LADO);
        }
        t.set_ponta(LinhaDaPonta::HoseMode, 2.0);
        t.set_brush_spacing(1.0);
        let cp = |x: f32, phase| CanvasPointer {
            pos: [x, 80.0],
            pressure: 0.9,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(cp(40.0, PointerPhase::Down));
        t.on_tick(1000.0 / 60.0);
        let mut x = 40.0;
        for _ in 0..30 {
            for _ in 0..3 {
                x += 6.0;
                t.on_canvas_pointer(cp(x, PointerPhase::Move));
            }
            let _ = t.take_preview_arc();
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(cp(x, PointerPhase::Up));
        for _ in 0..40 {
            t.on_tick(1000.0 / 60.0);
        }
        t.run_full().0
    };
    let silhueta = |c: &[u8]| -> Vec<bool> {
        let minimo = c
            .chunks_exact(4)
            .map(|p| i32::from(p[0]))
            .min()
            .unwrap_or(255);
        let limiar = 255 - (255 - minimo) * 3 / 10;
        c.chunks_exact(4)
            .map(|p| i32::from(p[0]) < limiar)
            .collect()
    };
    let difere = |a: &[u8], b: &[u8]| {
        silhueta(a)
            .iter()
            .zip(silhueta(b))
            .filter(|(x, y)| **x != *y)
            .count()
    };
    let (ponta, losango, quadrado) = (traco(&[0, 2]), traco(&[2]), traco(&[0]));
    let (d, controlo) = (difere(&ponta, &losango), difere(&quadrado, &losango));
    println!("[mangueira] Wet adiado: ponta×losango {d} · quadrado×losango {controlo} px");
    assert!(controlo > 50, "o controlo não separa: {controlo}");
    assert!(
        d * 20 < controlo,
        "os lotes adiados não carimbaram o losango: {d} × {controlo}"
    );
}
