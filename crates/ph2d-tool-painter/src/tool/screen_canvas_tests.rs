//! Os gates da [`super`] — a tela da vista 3D presa no Painter.
//!
//! ⚠️ Eles passam pela porta PÚBLICA do ponteiro (`on_canvas_pointer`), que é a
//! mesma que a escultura usa: um traço conduzido por dentro afirmaria sobre o
//! motor e nunca sobre a costura.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: u32 = 64;
const H: u32 = 48;

fn cp(p: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([10.0, 20.0], PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp([10.0 + 3.0 * k as f32, 20.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([40.0, 20.0], PointerPhase::Up));
}

fn pintor() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_brush_color_srgb8([255, 0, 0]);
    t.set_brush_strength(1.0);
    t.set_brush_size_px(6.0);
    t
}

/// ⭐⭐ **O motor do Painter pinta a tela da vista, e a drenagem dela entrega a
/// tinta com o rectângulo que mudou** — o elo de que a escultura depende.
#[test]
fn um_traco_na_tela_da_vista_sai_na_drenagem_com_o_rectangulo() {
    let mut t = pintor();
    assert!(t.bind_screen_canvas(W, H), "a tela nasceu");
    assert!(t.on_screen_canvas());
    traco(&mut t);
    let f = t.take_screen_canvas().expect("o traço sujou a tela");
    assert_eq!((f.w, f.h), (W, H));
    let pintados = f
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 0)
        .count();
    assert!(pintados > 20, "o traço deixou tinta: {pintados} píxeis");
    let r = f
        .rect
        .expect("uma tela de uma camada devolve o rectângulo sujo");
    assert!(
        r.2 > 0 && r.3 > 0 && r.2 < W,
        "o rectângulo é o do traço: {r:?}"
    );
    assert!(
        t.take_screen_canvas().is_none(),
        "drenar outra vez não entrega nada"
    );
}

/// ⭐⭐⭐ **Com a tela presa, a PONTE DA SPRITE não lhe toca** — as quatro portas
/// que ela usa a cada quadro. Sem isto a ponte drenava a tela antes da escultura
/// (o rectângulo sujo perdia-se), ligava a sprite escolhida por cima dela, e o
/// `Apply` assava a vista numa sprite.
#[test]
fn com_a_tela_presa_a_ponte_da_sprite_nao_lhe_toca() {
    let mut t = pintor();
    t.bind_screen_canvas(W, H);
    traco(&mut t);
    assert!(t.take_preview_arc().is_none(), "a drenagem da ponte");
    assert!(!t.take_preview_dirty(), "a pista GPU da ponte");
    assert!(
        !t.needs_document_bind(7),
        "a sprite escolhida não desloca a tela"
    );
    t.request_commit();
    assert!(
        !t.take_pending_commit(),
        "o Apply não assa a vista numa sprite"
    );
    // O CONTROLO: a escultura continua a drenar a mesma tinta.
    assert!(t.take_screen_canvas().is_some());
}

/// **Limpar devolve a tela transparente** — o fim de um traço, depois de ele ter
/// sido pousado; e **soltar devolve a ponte à sprite**.
#[test]
fn limpar_esvazia_e_soltar_devolve_a_ponte_a_sprite() {
    let mut t = pintor();
    t.bind_screen_canvas(W, H);
    traco(&mut t);
    let _ = t.take_screen_canvas();
    t.clear_screen_canvas();
    assert!(t.on_screen_canvas(), "limpar não solta");
    assert!(
        t.canvas_rgba.iter().all(|&b| b == 0),
        "a tela voltou transparente"
    );
    t.release_screen_canvas();
    assert!(!t.on_screen_canvas());
    assert!(
        t.needs_document_bind(7),
        "a ponte volta a poder ligar a sprite"
    );
}

/// **A vista muda de tamanho ⇒ a tela acompanha**; o mesmo tamanho não a refaz.
#[test]
fn a_tela_segue_o_tamanho_da_vista() {
    let mut t = pintor();
    assert!(t.bind_screen_canvas(W, H));
    assert!(!t.bind_screen_canvas(W, H), "o mesmo tamanho é idempotente");
    assert!(t.bind_screen_canvas(2 * W, H), "a vista cresceu");
    assert_eq!(t.canvas_size(), (2 * W, H));
}

/// ⭐⭐ **Prender a tela GUARDA a sprite de várias camadas que estava ligada**, e
/// soltar deixa-a voltar inteira — a tela entra como documento, nunca por um
/// `set_source` que achataria as camadas.
#[test]
fn prender_a_tela_nao_achata_a_sprite_que_estava_ligada() {
    let mut t = pintor();
    t.bind_document(1, vec![255u8; 32 * 32 * 4], 32, 32);
    t.layers.add_raster("Layer 2", 32, 32);
    let camadas = t.layers.root().len();
    assert!(camadas >= 2);
    t.bind_screen_canvas(W, H);
    assert!(t.is_trivial_stack(), "a tela é um documento de uma camada");
    t.release_screen_canvas();
    t.bind_document(1, vec![0u8; 4], 1, 1);
    assert_eq!(
        t.layers.root().len(),
        camadas,
        "a sprite voltou com as camadas dela"
    );
    assert_eq!(t.source_size, (32, 32));
}

// ── ETAPA 2: A TELA SEMEADA ────────────────────────────────────────────────

/// ⭐⭐ **Só a pintura simples pinta sobre o transparente; o resto lê a peça** —
/// a pergunta que decide se a tela da vista 3D começa com o retrato.
#[test]
fn so_a_pintura_simples_dispensa_o_retrato_da_peca() {
    let mut t = pintor();
    assert!(
        !t.screen_canvas_reads_the_piece(),
        "o pincel de fábrica é o «over»"
    );
    t.set_paint_tool_mode("eraser");
    assert!(
        !t.screen_canvas_reads_the_piece(),
        "a borracha não arranca cor da peça"
    );
    for modo in ["smear", "blur", "clone", "fill", "inpaint"] {
        t.set_paint_tool_mode(modo);
        assert!(
            t.screen_canvas_reads_the_piece(),
            "`{modo}` lê a cor debaixo do pincel"
        );
    }
    t.set_paint_tool_mode("brush");
    assert!(
        !t.screen_canvas_reads_the_piece(),
        "o CONTROLO: de volta ao pincel"
    );
    t.set_brush_blend(ph2d_painter_brush::BrushBlend::Multiply as u8);
    assert!(t.screen_canvas_reads_the_piece(), "o Multiply lê a peça");
    t.set_brush_blend(ph2d_painter_brush::BrushBlend::Mix as u8);
    t.set_paint_media(crate::PaintMedia::Watercolor);
    assert!(t.screen_canvas_reads_the_piece(), "a aquarela lê a peça");
}

/// **Semear põe o retrato na tela**, e só com a tela presa e o tamanho certo.
#[test]
fn semear_a_tela_poe_o_retrato_nela() {
    let mut t = pintor();
    let retrato: Vec<u8> = (0..W * H).flat_map(|_| [10u8, 20, 30, 255]).collect();
    assert!(
        !t.seed_screen_canvas(retrato.clone()),
        "sem tela presa não semeia"
    );
    t.bind_screen_canvas(W, H);
    assert!(
        !t.seed_screen_canvas(vec![0; 16]),
        "o tamanho errado não semeia"
    );
    assert!(t.seed_screen_canvas(retrato.clone()));
    assert_eq!(t.canvas_rgba.as_slice(), retrato.as_slice());
}

/// ⭐⭐⭐ **Um borrão do Painter sobre a tela semeada ARRASTA a cor do retrato** —
/// o motor dele, pela porta pública do ponteiro, a ler o que a peça tinha.
/// Metade vermelha, metade azul; um borrão da esquerda para a direita leva
/// vermelho para o lado azul.
#[test]
fn um_borrao_sobre_a_tela_semeada_arrasta_a_cor() {
    let mut t = pintor();
    t.set_paint_tool_mode("smear");
    t.bind_screen_canvas(W, H);
    let retrato: Vec<u8> = (0..W * H)
        .flat_map(|i| {
            if i % W < W / 2 {
                [255u8, 0, 0, 255]
            } else {
                [0u8, 0, 255, 255]
            }
        })
        .collect();
    assert!(t.seed_screen_canvas(retrato.clone()));
    let _ = t.take_screen_canvas();
    t.on_canvas_pointer(cp([20.0, 24.0], PointerPhase::Down));
    for k in 1..=12 {
        t.on_canvas_pointer(cp([20.0 + 2.0 * k as f32, 24.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([44.0, 24.0], PointerPhase::Up));
    let f = t.take_screen_canvas().expect("o borrão mudou a tela");
    let o = ((24 * W + 40) * 4) as usize;
    assert!(
        f.rgba[o] > 0,
        "o píxel (40,24) do lado azul recebeu vermelho: {:?}",
        &f.rgba[o..o + 4]
    );
    let longe = ((5 * W + 60) * 4) as usize;
    assert_eq!(
        &f.rgba[longe..longe + 4],
        &[0, 0, 255, 255],
        "o CONTROLO: longe do traço"
    );
}

/// 🔎 **SONDA (não é gate)** — cada modo da etapa 2 sobre a tela semeada, pelo
/// mesmo traço, e quantos píxeis a drenagem traz mudados. Um `0` é um modo que
/// sobre a vista 3D não faz nada.
#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_cada_modo_sobre_a_tela_semeada() {
    let retrato: Vec<u8> = (0..W * H)
        .flat_map(|i| {
            if i % W < W / 2 {
                [255u8, 0, 0, 255]
            } else {
                [0u8, 0, 255, 255]
            }
        })
        .collect();
    let casos: [(&str, Option<crate::PaintMedia>); 8] = [
        ("smear", None),
        ("blur", None),
        ("liquify", None),
        ("fill", None),
        ("clone", None),
        ("inpaint", None),
        ("brush", Some(crate::PaintMedia::Watercolor)),
        ("brush", Some(crate::PaintMedia::WetPaint)),
    ];
    for (modo, meio) in casos {
        let mut t = pintor();
        t.set_brush_color_srgb8([0, 255, 0]);
        t.set_paint_tool_mode(modo);
        if let Some(m) = meio {
            t.set_paint_media(m);
        }
        t.bind_screen_canvas(W, H);
        assert!(t.seed_screen_canvas(retrato.clone()));
        let _ = t.take_screen_canvas();
        t.on_canvas_pointer(cp([20.0, 24.0], PointerPhase::Down));
        for k in 1..=12 {
            t.on_canvas_pointer(cp([20.0 + 2.0 * k as f32, 24.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([44.0, 24.0], PointerPhase::Up));
        for _ in 0..30 {
            t.on_tick(1.0 / 40.0);
        }
        let mudados = t.take_screen_canvas().map_or(0, |f| {
            f.rgba
                .chunks(4)
                .zip(retrato.chunks(4))
                .filter(|(a, b)| a != b)
                .count()
        });
        println!("  {modo:8} {meio:?}: {mudados} píxeis mudados");
    }
}

/// 🔎 **SONDA** — os dois modos que um traço só não mostra: o Clone (1.º gesto
/// escolhe a fonte, o 2.º pinta) e o Inpaint (cura um DEFEITO; sobre um
/// retrato sem defeito a reconstrução devolve os mesmos píxeis).
#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_clone_e_inpaint_sobre_a_tela_semeada() {
    let mut retrato: Vec<u8> = (0..W * H)
        .flat_map(|i| {
            if i % W < W / 2 {
                [255u8, 0, 0, 255]
            } else {
                [0u8, 0, 255, 255]
            }
        })
        .collect();
    let traco = |t: &mut PainterTool, x0: f32, y: f32| {
        t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
        for k in 1..=6 {
            t.on_canvas_pointer(cp([x0 + 2.0 * k as f32, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x0 + 12.0, y], PointerPhase::Up));
    };
    let mudados = |t: &mut PainterTool, r: &[u8]| {
        t.take_screen_canvas().map_or(0, |f| {
            f.rgba
                .chunks(4)
                .zip(r.chunks(4))
                .filter(|(a, b)| a != b)
                .count()
        })
    };
    // Clone: fonte no azul, pintar no vermelho.
    let mut t = pintor();
    t.set_paint_tool_mode("clone");
    t.bind_screen_canvas(W, H);
    t.seed_screen_canvas(retrato.clone());
    let _ = t.take_screen_canvas();
    t.arm_clone_sample();
    t.on_canvas_pointer(cp([50.0, 24.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([50.0, 24.0], PointerPhase::Up));
    traco(&mut t, 6.0, 24.0);
    println!(
        "  clone (fonte + traço): {} píxeis mudados",
        mudados(&mut t, &retrato)
    );
    // Inpaint: um defeito branco no vermelho.
    for y in 20..28 {
        for x in 8..16 {
            let o = ((y * W + x) * 4) as usize;
            retrato[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let mut t = pintor();
    t.set_paint_tool_mode("inpaint");
    t.set_brush_size_px(10.0);
    t.bind_screen_canvas(W, H);
    t.seed_screen_canvas(retrato.clone());
    let _ = t.take_screen_canvas();
    traco(&mut t, 6.0, 24.0);
    println!(
        "  inpaint (defeito): {} píxeis mudados",
        mudados(&mut t, &retrato)
    );
}

/// 🔎 **SONDA** — o que um meio ainda muda DEPOIS do pen-up (os ticks do
/// coração do Painter): a tela no instante do `Up` contra a mesma tela 30
/// ticks depois.
#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_o_que_muda_depois_do_pen_up() {
    let retrato: Vec<u8> = (0..W * H).flat_map(|_| [200u8, 200, 200, 255]).collect();
    for meio in [
        crate::PaintMedia::Digital,
        crate::PaintMedia::Watercolor,
        crate::PaintMedia::WetPaint,
    ] {
        let mut t = pintor();
        t.set_paint_media(meio);
        t.bind_screen_canvas(W, H);
        t.seed_screen_canvas(retrato.clone());
        let _ = t.take_screen_canvas();
        t.on_canvas_pointer(cp([20.0, 24.0], PointerPhase::Down));
        for k in 1..=12 {
            t.on_canvas_pointer(cp([20.0 + 2.0 * k as f32, 24.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([44.0, 24.0], PointerPhase::Up));
        let no_up = t
            .take_screen_canvas()
            .map(|f| f.rgba.to_vec())
            .unwrap_or_default();
        for _ in 0..30 {
            t.on_tick(1.0 / 40.0);
        }
        let depois = t
            .take_screen_canvas()
            .map_or_else(|| no_up.clone(), |f| f.rgba.to_vec());
        let mudou = no_up
            .chunks(4)
            .zip(depois.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        println!("  {meio:?}: {mudou} píxeis mudam depois do pen-up");
    }
}
