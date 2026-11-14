//! N10 F2c: o depósito pela FILA pinta a tela do depósito SÍNCRONO, ao bit, no mesmo processo (o oráculo é o
//! caminho síncrono), em 3 fundos (branco · blocos de cor · Textured com relevo) × Flat Bristle e Thick Impasto.

use super::super::super::agua_sobre_oleo_tests::{mostrada, preset};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

fn cp(pos: [f32; 2], pressure: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if blocos {
        for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            p[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
        }
    }
    v
}

fn pinta(fila: bool, qual: u8, pincel: &str, tamanho: f32) -> Vec<u8> {
    let (w, h) = (320u32, 160u32);
    super::fila_no_teste(fila);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.abre_a_sprite(fundo(w, h, qual == 1), w, h);
    }
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.preset_aplicar(&preset(pincel)).expect("o pincel");
    t.set_brush_size_px(tamanho);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(cp([60.0, 80.0], 0.6, PointerPhase::Down));
    for k in 1..=12 {
        let p = [60.0 + 16.0 * k as f32, 80.0 + 3.0 * k as f32];
        t.on_canvas_pointer(cp(p, 0.6 + 0.03 * k as f32, PointerPhase::Move));
        t.ticks_em_blocos(1);
    }
    t.on_canvas_pointer(cp([252.0, 116.0], 1.0, PointerPhase::Up));
    t.ticks_em_blocos(2);
    let px = mostrada(&mut t);
    super::fila_no_teste(false);
    px
}

#[test]
fn o_deposito_pela_fila_pinta_a_tela_do_deposito_sincrono_ao_bit() {
    for (pincel, tamanho) in [
        ("Oil/Flat/Flat Bristle.ph2d-brush.png", 70.0),
        ("Oil/Flat/Thick Impasto.ph2d-brush.png", 60.0),
    ] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "Textured")] {
            let sincrono = pinta(false, qual, pincel, tamanho);
            let fila = pinta(true, qual, pincel, tamanho);
            let fundo_so = pinta_nada(qual);
            assert!(
                sincrono != fundo_so,
                "{pincel} · {nome}: o traço não pintou"
            );
            assert!(sincrono == fila, "{pincel} · {nome}: a fila mudou a tela");
        }
    }
}

/// O fundo sem traço, para provar que o traço pintou (no papel Textured o branco não é 255).
fn pinta_nada(qual: u8) -> Vec<u8> {
    let (w, h) = (320u32, 160u32);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.abre_a_sprite(fundo(w, h, qual == 1), w, h);
    }
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    mostrada(&mut t)
}

/// O traço INTEIRO com a sessão em voo entre eventos (F2d): conta-gotas a meio, undo, redo, troca de camada e um
/// traço em cada camada. Devolve a tela mostrada, as amostras do conta-gotas e quantos lotes voaram.
fn traco_inteiro(fila: bool, qual: u8) -> (Vec<u8>, Vec<Option<[u8; 4]>>, usize) {
    traco_inteiro_com(fila, false, qual)
}

/// `encadeado` (F2f): a porta segura o operário durante os eventos, então cada lote fica ATRÁS do voo anterior;
/// abre-se antes do conta-gotas, do tique e do pen-up (os leitores que drenam).
fn traco_inteiro_com(
    fila: bool,
    encadeado: bool,
    qual: u8,
) -> (Vec<u8>, Vec<Option<[u8; 4]>>, usize) {
    let (w, h) = (320u32, 160u32);
    super::fila_no_teste(fila);
    super::super::sessao_do_wet::VOOS.set(0);
    let porta = |fechada: bool| {
        if encadeado {
            super::segura_o_operario(fechada);
        }
    };
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.abre_a_sprite(fundo(w, h, qual == 1), w, h);
    }
    let a = t.layers.active().expect("a camada");
    let b = t
        .add_raster_layer_with_pixels("B", vec![0u8; (w * h * 4) as usize])
        .expect("B");
    t.select_layer(a);
    t.set_compor_por_quadro(true); // o produto: só o lote que compõe no quadro pode voar
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.preset_aplicar(&preset("Oil/Flat/Flat Bristle.ph2d-brush.png"))
        .expect("o pincel");
    t.set_brush_size_px(60.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    let mut gotas = Vec::new();
    let mut traco = |t: &mut PainterTool, y0: f32, gotas: &mut Vec<Option<[u8; 4]>>| {
        porta(true);
        t.on_canvas_pointer(cp([60.0, y0], 0.6, PointerPhase::Down));
        for k in 1..=10 {
            let p = [60.0 + 18.0 * k as f32, y0 + 2.0 * k as f32];
            t.on_canvas_pointer(cp(p, 0.6 + 0.03 * k as f32, PointerPhase::Move));
            if k % 4 == 0 {
                porta(false);
                gotas.push(t.eyedropper_amostra([p[0] - 20.0, p[1]]));
                porta(true);
            }
            if k % 3 == 0 {
                porta(false);
                t.ticks_em_blocos(1);
                porta(true);
            }
        }
        porta(false);
        t.on_canvas_pointer(cp([250.0, y0 + 20.0], 1.0, PointerPhase::Up));
    };
    traco(&mut t, 50.0, &mut gotas);
    t.ticks_em_blocos(1);
    assert!(t.undo_last(), "o undo");
    gotas.push(t.eyedropper_amostra([120.0, 56.0]));
    assert!(t.redo_last(), "o redo");
    traco(&mut t, 90.0, &mut gotas);
    t.select_layer(b);
    traco(&mut t, 70.0, &mut gotas);
    t.select_layer(a);
    traco(&mut t, 110.0, &mut gotas);
    t.ticks_em_blocos(2);
    let px = mostrada(&mut t);
    let voos = super::super::sessao_do_wet::VOOS.get();
    super::fila_no_teste(false);
    (px, gotas, voos)
}

#[test]
fn o_traco_inteiro_em_voo_pinta_a_tela_do_sincrono_ao_bit() {
    for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "Textured")] {
        let (sincrono, gotas_s, voos_s) = traco_inteiro(false, qual);
        let (fila, gotas_f, voos_f) = traco_inteiro(true, qual);
        assert_eq!(voos_s, 0, "{nome}: o síncrono não voa");
        assert!(
            voos_f > 0,
            "{nome}: nenhum lote ficou em voo (o gate não mediria nada)"
        );
        assert!(sincrono != pinta_nada(qual), "{nome}: o traço não pintou");
        assert_eq!(
            gotas_s, gotas_f,
            "{nome}: o conta-gotas leu outra tela em voo"
        );
        assert!(sincrono == fila, "{nome}: a sessão em voo mudou a tela");
    }
}

/// F2h: o operário compõe a ÓPTICA do lote que voa logo depois do depósito e o quadro só corre o laço de píxel — o
/// traço inteiro (conta-gotas a meio, undo, redo, troca A→B→A) pinta a tela do síncrono ao bit nos três fundos, e a
/// prova de que o caminho correu é `COLADAS > 0` (a mutação «o quadro não cola as ópticas do operário» reprova a tela).
#[test]
fn a_otica_composta_no_operario_pinta_a_tela_do_sincrono_ao_bit() {
    for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "Textured")] {
        let (sincrono, gotas_s, _) = traco_inteiro(false, qual);
        super::super::sessao_do_wet::COLADAS.set(0);
        let (fila, gotas_f, _) = traco_inteiro(true, qual);
        let coladas = super::super::sessao_do_wet::COLADAS.get();
        assert!(
            coladas > 0,
            "{nome}: nenhuma óptica do operário chegou à tela (o gate não mediria nada)"
        );
        assert_eq!(
            gotas_s, gotas_f,
            "{nome}: o conta-gotas leu outra tela com a óptica no operário"
        );
        assert!(
            sincrono == fila,
            "{nome}: a óptica no operário mudou a tela"
        );
    }
}

/// A porta do operário é uma só no processo: os testes que a fecham correm um de cada vez.
static PORTA_DE_UM: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// F2f: o lote que chega com o anterior ainda no operário fica ATRÁS dele (o evento não espera) e o traço inteiro —
/// conta-gotas a meio, undo, redo, troca de camada — pinta a tela do síncrono ao bit, nos três fundos.
#[test]
fn o_lote_encadeado_atras_do_voo_pinta_a_tela_do_sincrono_ao_bit() {
    let _so_um = PORTA_DE_UM
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "Textured")] {
        let (sincrono, gotas_s, _) = traco_inteiro(false, qual);
        super::super::sessao_do_wet::ATRAS.set(0);
        let (fila, gotas_f, voos_f) = traco_inteiro_com(true, true, qual);
        let atras = super::super::sessao_do_wet::ATRAS.get();
        assert!(
            voos_f > 0 && atras > 0,
            "{nome}: nenhum lote ficou atrás de um voo ({voos_f} voos, {atras} atrás)"
        );
        assert_eq!(
            gotas_s, gotas_f,
            "{nome}: o conta-gotas leu outra tela com lotes atrás do voo"
        );
        assert!(sincrono == fila, "{nome}: o encadeado mudou a tela");
    }
}

/// F2e: o TIQUE não espera o lote em voo — o operário fica preso pela porta e o tique tem de voltar com a sessão
/// ainda fora (o guarda e a sim drenariam: a mutação «sem o `assentou` no tique» espera os 2 s da porta e reprova).
#[test]
fn o_tique_nao_espera_o_lote_em_voo() {
    let _so_um = PORTA_DE_UM
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (w, h) = (320u32, 160u32);
    super::fila_no_teste(true);
    let mut t = PainterTool::default();
    t.abre_a_sprite(fundo(w, h, false), w, h);
    t.set_compor_por_quadro(true);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(&preset("Oil/Flat/Flat Bristle.ph2d-brush.png"))
        .expect("o pincel");
    t.set_brush_size_px(60.0);
    t.on_canvas_pointer(cp([60.0, 50.0], 0.6, PointerPhase::Down));
    for k in 1..=5 {
        let p = [60.0 + 18.0 * k as f32, 50.0 + 2.0 * k as f32];
        t.on_canvas_pointer(cp(p, 0.6 + 0.03 * k as f32, PointerPhase::Move));
    }
    t.ticks_em_blocos(1);
    super::segura_o_operario(true);
    t.on_canvas_pointer(cp([168.0, 62.0], 0.8, PointerPhase::Move));
    let voou = t.paint.wetpaint.session.em_voo();
    let antes = std::time::Instant::now();
    t.wetpaint_tick(1.0 / 60.0);
    let esperou = antes.elapsed();
    let ainda = t.paint.wetpaint.session.em_voo();
    super::segura_o_operario(false);
    t.on_canvas_pointer(cp([200.0, 60.0], 1.0, PointerPhase::Up));
    t.ticks_em_blocos(2);
    super::fila_no_teste(false);
    assert!(voou, "o lote não voou (o gate não mediria nada)");
    assert!(
        ainda && esperou < std::time::Duration::from_secs(1),
        "o tique esperou o lote em voo ({esperou:?})"
    );
}

/// F2g: a drenagem do quadro (`take_preview_dirty` → `reconcile_substrate` → o grão do Óleo Real) não espera o lote
/// em voo — o `[wet-espera]` mediu 32–85 ms por quadro nesse dreno (Flat Bristle 240, 4K).
#[test]
fn a_drenagem_do_quadro_nao_espera_o_lote_em_voo() {
    let _so_um = PORTA_DE_UM
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (w, h) = (320u32, 160u32);
    super::fila_no_teste(true);
    let mut t = PainterTool::default();
    t.abre_a_sprite(fundo(w, h, false), w, h);
    t.set_compor_por_quadro(true);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(&preset("Oil/Flat/Flat Bristle.ph2d-brush.png"))
        .expect("o pincel");
    t.set_brush_size_px(60.0);
    t.paint.wetpaint.oleo_na_tela = true;
    t.on_canvas_pointer(cp([60.0, 50.0], 0.6, PointerPhase::Down));
    for k in 1..=5 {
        let p = [60.0 + 18.0 * k as f32, 50.0 + 2.0 * k as f32];
        t.on_canvas_pointer(cp(p, 0.6 + 0.03 * k as f32, PointerPhase::Move));
    }
    t.ticks_em_blocos(1);
    super::segura_o_operario(true);
    t.on_canvas_pointer(cp([168.0, 62.0], 0.8, PointerPhase::Move));
    let voou = t.paint.wetpaint.session.em_voo();
    let antes = std::time::Instant::now();
    let _ = t.take_preview_dirty();
    let esperou = antes.elapsed();
    let ainda = t.paint.wetpaint.session.em_voo();
    super::segura_o_operario(false);
    t.on_canvas_pointer(cp([200.0, 60.0], 1.0, PointerPhase::Up));
    t.ticks_em_blocos(2);
    super::fila_no_teste(false);
    assert!(voou, "o lote não voou (o gate não mediria nada)");
    assert!(
        ainda && esperou < std::time::Duration::from_secs(1),
        "a drenagem do quadro esperou o lote em voo ({esperou:?})"
    );
}

/// F2e: um traço pintado DENTRO de um job do rayon (a miniatura viva, os alarmes de pincel) com a fila ligada não
/// voa nem espera o operário no job (Bug #66: `drena_os_dabs` entraria em pânico) e pinta a tela do síncrono.
#[test]
fn dentro_de_um_job_do_rayon_o_lote_pousa_sincrono() {
    let pinta = |fila: bool| {
        super::fila_no_teste(fila);
        let (w, h) = (200u32, 100u32);
        let mut t = PainterTool::default();
        t.abre_a_sprite(fundo(w, h, false), w, h);
        t.set_compor_por_quadro(true);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        t.preset_aplicar(&preset("Oil/Flat/Flat Bristle.ph2d-brush.png"))
            .expect("o pincel");
        t.set_brush_size_px(40.0);
        t.on_canvas_pointer(cp([30.0, 50.0], 0.6, PointerPhase::Down));
        for k in 1..=8 {
            t.on_canvas_pointer(cp([30.0 + 18.0 * k as f32, 50.0], 0.7, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([180.0, 50.0], 1.0, PointerPhase::Up));
        t.ticks_em_blocos(2);
        let voos = super::super::sessao_do_wet::VOOS.get();
        super::fila_no_teste(false);
        (mostrada(&mut t), voos)
    };
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("o pool");
    let (sincrono, _) = pinta(false);
    let (no_job, voos) = pool.install(|| {
        super::super::sessao_do_wet::VOOS.set(0);
        pinta(true)
    });
    assert_eq!(voos, 0, "um lote voou dentro do job do rayon");
    assert!(
        sincrono == no_job,
        "a tela pintada no job difere da síncrona"
    );
}

/// F2j: o traço solta com o lote ainda no operário (a porta fechada até ao Up) e a PORTA seguinte vem logo — `0`
/// Ctrl+Z (conta-gotas, Ctrl+Shift+Z) · `1` troca de camada · `2` traço novo · `3` o tique · `4` camada nova + Ctrl+Z
/// (edição estrutural, F2k) · `5` o conta-gotas armado e lido sem tique (F2k). `fila = false` é o oráculo
/// síncrono (nenhum voo, nenhum adiado).
fn pen_up_e_a_porta(fila: bool, qual: u8, porta: u8) -> (Vec<u8>, Vec<Option<[u8; 4]>>, usize) {
    let (w, h) = (320u32, 160u32);
    super::fila_no_teste(fila);
    super::super::ciclo::adia_no_teste(true);
    super::super::ciclo::FECHOS_ADIADOS.set(0);
    let segura = |fechada: bool| {
        if fila {
            super::segura_o_operario(fechada);
        }
    };
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.abre_a_sprite(fundo(w, h, qual == 1), w, h);
    }
    let a = t.layers.active().expect("a camada");
    let b = t
        .add_raster_layer_with_pixels("B", vec![0u8; (w * h * 4) as usize])
        .expect("B");
    t.select_layer(a);
    t.set_compor_por_quadro(true);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.preset_aplicar(&preset("Oil/Flat/Flat Bristle.ph2d-brush.png"))
        .expect("o pincel");
    t.set_brush_size_px(60.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    let traco = |t: &mut PainterTool, y0: f32| {
        t.on_canvas_pointer(cp([60.0, y0], 0.6, PointerPhase::Down)); // conclui o fecho de antes: porta aberta
        segura(true);
        for k in 1..=10 {
            let p = [60.0 + 18.0 * k as f32, y0 + 2.0 * k as f32];
            t.on_canvas_pointer(cp(p, 0.6 + 0.03 * k as f32, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([250.0, y0 + 20.0], 1.0, PointerPhase::Up));
        segura(false);
    };
    let mut gotas = Vec::new();
    traco(&mut t, 50.0);
    match porta {
        0 => {
            assert!(t.undo_last(), "o undo");
            gotas.push(t.eyedropper_amostra([120.0, 56.0]));
            assert!(t.redo_last(), "o redo");
        }
        1 => t.select_layer(b),
        2 => traco(&mut t, 90.0),
        3 => t.ticks_em_blocos(1),
        4 => {
            assert!(t.add_raster_layer("C").is_some(), "a camada nova");
            assert!(t.undo_last(), "o undo da camada nova"); // tem de desfazer a camada, não o traço
        }
        _ => {
            t.set_paint_tool_mode("eyedropper"); // armar o conta-gotas conclui o fecho; a amostra `&self` sem tique
            gotas.push(t.eyedropper_amostra([150.0, 60.0]));
            t.set_paint_tool_mode("wetpaint");
        }
    }
    // ⚠️ o `eyedropper_amostra` é `&self` (não é porta): lê depois de um tique, que conclui o fecho de um traço novo.
    t.ticks_em_blocos(1);
    gotas.push(t.eyedropper_amostra([150.0, 60.0]));
    traco(&mut t, 110.0);
    t.ticks_em_blocos(2);
    assert!(t.undo_last(), "o undo do último");
    gotas.push(t.eyedropper_amostra([150.0, 120.0]));
    assert!(t.undo_last(), "o undo do anterior");
    let px = mostrada(&mut t);
    let adiados = super::super::ciclo::FECHOS_ADIADOS.get();
    super::super::ciclo::adia_no_teste(false);
    super::fila_no_teste(false);
    (px, gotas, adiados)
}

/// F2j: o pen-up ADIADO (o `close_stroke` espera a sessão assentar) fecha o passo de undo do síncrono ao bit — com
/// Ctrl+Z, troca de camada, traço novo ou tique logo a seguir ao soltar, nos três fundos; a prova de que o caminho
/// correu é `FECHOS_ADIADOS > 0` (a mutação «o undo não conclui o fecho» desfaz o traço de antes e reprova).
#[test]
fn o_pen_up_adiado_fecha_o_passo_de_undo_do_sincrono_ao_bit() {
    let _so_um = PORTA_DE_UM
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "Textured")] {
        for porta in 0..6u8 {
            let (sincrono, gotas_s, adiados_s) = pen_up_e_a_porta(false, qual, porta);
            let (fila, gotas_f, adiados_f) = pen_up_e_a_porta(true, qual, porta);
            assert_eq!(adiados_s, 0, "{nome}/{porta}: o síncrono não adia");
            assert!(
                adiados_f > 0,
                "{nome}/{porta}: nenhum pen-up adiado (o gate não mediria nada)"
            );
            assert_eq!(
                gotas_s, gotas_f,
                "{nome}/{porta}: o conta-gotas leu outra tela"
            );
            assert!(
                sincrono == fila,
                "{nome}/{porta}: o pen-up adiado mudou a tela"
            );
        }
    }
}

/// N10 F11: o lote em voo NÃO suja a prévia — a tela ainda não mudou. O sujo chega com a composição que o trouxer,
/// com a caixa: a pista da placa dobra e acende só ela. Antes, o `preview_dirty` do voo chegava à drenagem sem caixa
/// («não confinado») e cada quadro do traço dobrava e acendia a tela inteira (Filbert 4K: traço 15 → 30 ms).
#[test]
fn o_lote_em_voo_nao_suja_a_previa_e_a_volta_dele_e_confinada() {
    let _so_um = PORTA_DE_UM
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (w, h) = (320u32, 160u32);
    super::fila_no_teste(true);
    let mut t = PainterTool::default();
    t.agenda_papel_texturizado(7);
    t.bind_document(7, fundo(w, h, false), w, h);
    t.set_compor_por_quadro(true);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(&preset("Oil/Flat/Flat Bristle.ph2d-brush.png"))
        .expect("o pincel");
    t.set_brush_size_px(30.0);
    t.on_canvas_pointer(cp([60.0, 50.0], 0.6, PointerPhase::Down));
    for k in 1..=4 {
        t.on_canvas_pointer(cp([60.0 + 12.0 * k as f32, 50.0], 0.7, PointerPhase::Move));
    }
    t.ticks_em_blocos(1);
    // Drena tudo o que havia (e o «stale» da pista da CPU): daqui em diante só o voo.
    let prazo = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while t.paint.wetpaint.session.em_voo() && std::time::Instant::now() < prazo {
        let _ = t.take_preview_dirty();
    }
    let _ = t.take_preview_dirty();
    super::segura_o_operario(true);
    t.on_canvas_pointer(cp([120.0, 52.0], 0.8, PointerPhase::Move));
    let voou = t.paint.wetpaint.session.em_voo();
    let sujo_em_voo = t.take_preview_dirty();
    super::segura_o_operario(false);
    let mut chegou = false;
    while !chegou && std::time::Instant::now() < prazo {
        chegou = t.take_preview_dirty();
    }
    let regiao = t.preview_gpu_region();
    t.on_canvas_pointer(cp([130.0, 52.0], 1.0, PointerPhase::Up));
    t.ticks_em_blocos(2);
    super::fila_no_teste(false);
    assert!(voou, "o lote não voou (o gate não mediria nada)");
    assert!(!sujo_em_voo, "o lote em voo sujou a prévia sem caixa");
    assert!(chegou, "a volta do lote não sujou a prévia");
    let (_, _, rw, rh) = regiao.expect("a volta do lote saiu NÃO confinada (a tela inteira)");
    assert!(
        rw < w || rh < h,
        "a volta do lote saiu com a tela inteira: {regiao:?}"
    );
}
