//! **A cor do papel em cada meio** (pedido do dono, 2026-10-05: *«A cor do papel na watercolor
//! precisa ser revisto pois funciona mal. Deveria funcionar para todos os modos e deveria ter um
//! botão para aplicar no papel como um todo.»*).

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// Um traço horizontal de `x = 20` a `108` em `y = 64` sobre uma tela `128²` (opaca branca, ou
/// transparente), com o papel `papel`; 30 quadros parados depois do pen-up.
fn traco(meio: PaintMedia, transparente: bool, papel: [u8; 3]) -> PainterTool {
    let mut t = tool(128, meio, 6.0);
    if transparente {
        t.set_source(vec![0u8; 128 * 128 * 4], 128, 128);
        t.set_paint_media(meio);
        t.set_brush_size_px(6.0);
    }
    t.set_wet_relogio_fixo(true);
    t.set_paper_color_rgb8(papel[0], papel[1], papel[2]);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
    for k in 1..=22 {
        #[allow(clippy::cast_precision_loss)]
        let x = 20.0 + 4.0 * k as f32;
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
    }
    t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
    for _ in 0..30 {
        t.paint_tick(1.0 / 60.0);
    }
    t
}

/// SONDA — o painel escreve onde · quem lê · o leitor DECIDE? Em cada meio, o mesmo traço com o
/// papel branco e o papel creme: os bytes da camada que mudam, sobre tela opaca e transparente, e
/// o texel do miolo e do papel limpo.
/// `cargo test -p ph2d-tool-painter --lib diag_a_cor_do_papel_em_cada_meio -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_a_cor_do_papel_em_cada_meio() {
    const CREME: [u8; 3] = [230, 200, 150];
    for meio in MEIOS {
        for transparente in [false, true] {
            let a = traco(meio, transparente, [255, 255, 255]);
            let b = traco(meio, transparente, CREME);
            let difere = a
                .canvas_rgba
                .chunks(4)
                .zip(b.canvas_rgba.chunks(4))
                .filter(|(p, q)| p != q)
                .count();
            let px = |t: &PainterTool, x: usize, y: usize| {
                let i = (y * 128 + x) * 4;
                [
                    t.canvas_rgba[i],
                    t.canvas_rgba[i + 1],
                    t.canvas_rgba[i + 2],
                    t.canvas_rgba[i + 3],
                ]
            };
            eprintln!(
                "{meio:?} tela {}: {difere} texels mudam com o papel creme · miolo branco {:?} creme {:?} · papel limpo branco {:?} creme {:?}",
                if transparente {
                    "transparente"
                } else {
                    "opaca"
                },
                px(&a, 64, 64),
                px(&b, 64, 64),
                px(&a, 64, 20),
                px(&b, 64, 20),
            );
        }
    }
}

/// SONDA — o dono (2026-10-05): *«digital realmente cobre o papel, mas não deveria»* · *«impasto
/// reconhece corretamente o papel»*. Com o Relief no máximo e o papel Cold, um traço largo em cada
/// meio: o desvio da luminância da IMAGEM MOSTRADA (composite + luz) no papel nu e dentro da tinta —
/// quanto do dente sobrevive sob a tinta. Relief 0 é o controlo (o dente some nos dois).
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_dente_sob_a_tinta -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_o_dente_sob_a_tinta() {
    let lum = |px: &[u8], i: usize| {
        0.299 * f64::from(px[i]) + 0.587 * f64::from(px[i + 1]) + 0.114 * f64::from(px[i + 2])
    };
    let bloco = |(x0, y0, x1, y1): (usize, usize, usize, usize)| {
        (y0..y1).flat_map(move |y| (x0..x1).map(move |x| (y * 128 + x) * 4))
    };
    // O dente: o desvio de `lum(relief 1) − lum(relief 0)` no bloco, e a média da luminância.
    let dente = |a: &[u8], b: &[u8], r| {
        let d: Vec<f64> = bloco(r).map(|i| lum(a, i) - lum(b, i)).collect();
        let m = d.iter().sum::<f64>() / d.len() as f64;
        let sd = (d.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / d.len() as f64).sqrt();
        let media = bloco(r).map(|i| lum(b, i)).sum::<f64>() / d.len() as f64;
        (sd, media)
    };
    for meio in MEIOS {
        let corre = |relief: f32| {
            let mut t = tool(128, meio, 30.0);
            t.set_wet_relogio_fixo(true);
            t.set_brush_paper_kind(26);
            t.paint.substrate_depth = relief;
            t.set_brush_color_srgb8([30, 60, 220]);
            t.on_canvas_pointer(cp([8.0, 64.0], PointerPhase::Down));
            for k in 1..=28 {
                #[allow(clippy::cast_precision_loss)]
                t.on_canvas_pointer(cp([8.0 + 4.0 * k as f32, 64.0], PointerPhase::Move));
                t.paint_tick(1.0 / 60.0);
            }
            t.on_canvas_pointer(cp([120.0, 64.0], PointerPhase::Up));
            for _ in 0..30 {
                t.paint_tick(1.0 / 60.0);
            }
            let (px, _, _) = t.take_preview_arc().expect("a imagem mostrada");
            (px.to_vec(), t.canvas_rgba.to_vec())
        };
        let ((v1, c1), (v0, c0)) = (corre(1.0), corre(0.0));
        let (nu, lnu) = dente(&v1, &v0, (40, 4, 88, 20));
        let (sob, lsob) = dente(&v1, &v0, (40, 56, 88, 72));
        let deposito = bloco((40, 56, 88, 72))
            .filter(|&i| c1[i..i + 4] != c0[i..i + 4])
            .count();
        eprintln!(
            "{meio:?}: dente no papel nu {nu:.2} (luz {lnu:.0}, {:.1} %) · sob a tinta {sob:.2} (luz {lsob:.0}, {:.1} %) · texels do depósito que o papel muda {deposito}/768",
            100.0 * nu / lnu,
            100.0 * sob / lsob,
        );
    }
}

/// Um tool `128²` branco opaco no `meio`, com a cor do papel `cor` escolhida pelo SELETOR (o evento
/// que o painel encaminha) e um traço largo horizontal; `aplica` = o botão antes do traço.
fn com_papel(meio: PaintMedia, cor: [u8; 3], aplica: bool) -> PainterTool {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = tool(128, meio, 10.0);
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", cor[0], cor[1], cor[2]),
    ));
    if aplica {
        t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_PAPER_APPLY));
    }
    t.set_brush_color_srgb8([30, 60, 220]);
    t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
    for k in 1..=22 {
        #[allow(clippy::cast_precision_loss)]
        t.on_canvas_pointer(cp([20.0 + 4.0 * k as f32, 64.0], PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
    }
    t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
    for _ in 0..30 {
        t.paint_tick(1.0 / 60.0);
    }
    t
}

/// O texel `(x, y)` da IMAGEM MOSTRADA (o composite com a luz).
fn mostrado(t: &mut PainterTool, x: usize, y: usize) -> [u8; 4] {
    t.invalidate_composite(); // a pré-visualização só se entrega quando há algo novo
    let (px, _, _) = t.take_preview_arc().expect("a imagem mostrada");
    let i = (y * 128 + x) * 4;
    [px[i], px[i + 1], px[i + 2], px[i + 3]]
}

const CREME: [u8; 3] = [230, 200, 150];

/// ⭐ **O PAPEL APLICADO É O CHÃO NOS QUATRO MEIOS** (pedido do dono, 2026-10-05) — com o botão, o
/// papel limpo mostra a cor do seletor em todo meio; a tinta opaca (Digital, Impasto) cobre-a — o
/// miolo do traço é o de sem papel —, e a aguada deixa-a ver (o miolo muda). CONTROLO: sem o botão a
/// cor do seletor não muda um texel (a linha dela fica esmaecida até o papel existir).
#[test]
fn o_papel_aplicado_e_o_chao_nos_quatro_meios() {
    for meio in MEIOS {
        let mut sem = com_papel(meio, CREME, false);
        let mut com = com_papel(meio, CREME, true);
        assert_eq!(
            com.papel(),
            Some(CREME),
            "{meio:?}: o botão não aplicou o papel"
        );
        assert_eq!(
            sem.papel(),
            None,
            "{meio:?}: o seletor sozinho aplicou o papel"
        );
        let limpo = mostrado(&mut com, 64, 20);
        assert_eq!(
            limpo,
            [CREME[0], CREME[1], CREME[2], 255],
            "{meio:?}: o papel limpo não é o papel"
        );
        assert_eq!(
            mostrado(&mut sem, 64, 20),
            [255, 255, 255, 255],
            "{meio:?}: controlo: sem o botão é branco"
        );
        let (m_sem, m_com) = (mostrado(&mut sem, 64, 64), mostrado(&mut com, 64, 64));
        match meio {
            PaintMedia::Digital | PaintMedia::Impasto => {
                assert_eq!(m_com, m_sem, "{meio:?}: a tinta opaca não cobriu o papel")
            }
            PaintMedia::Watercolor => assert_ne!(
                m_com, m_sem,
                "{meio:?}: a aguada não deixa ver o papel por baixo"
            ),
            PaintMedia::WetPaint => {}
        }
    }
}

/// **A COR MUDA AO VIVO E DESFAZ-SE NUM PASSO** — com o papel aplicado, três mudanças seguidas no
/// seletor (um arrasto) repintam o papel; um desfazer volta à cor do botão, o seguinte tira o papel
/// e devolve o branco da camada.
#[test]
fn a_cor_do_papel_muda_ao_vivo_e_desfaz_num_passo() {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = com_papel(PaintMedia::Digital, CREME, true);
    for c in ["200,220,240", "180,210,250", "120,200,255"] {
        t.handle_panel_event(PanelEvent::SelectOption(
            crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
            c.into(),
        ));
    }
    assert_eq!(
        mostrado(&mut t, 64, 20),
        [120, 200, 255, 255],
        "o papel não seguiu a cor ao vivo"
    );
    // O traço é o passo mais recente antes do arrasto? Não: o traço veio DEPOIS do botão; o arrasto
    // é o último passo.
    assert!(t.undo_last(), "há o que desfazer");
    assert_eq!(
        mostrado(&mut t, 64, 20),
        [CREME[0], CREME[1], CREME[2], 255],
        "um desfazer não voltou à cor do botão"
    );
    assert!(t.undo_last(), "o traço");
    assert!(t.undo_last(), "o botão");
    assert_eq!(t.papel(), None, "o desfazer do botão não tirou o papel");
    assert_eq!(
        mostrado(&mut t, 64, 20),
        [255, 255, 255, 255],
        "o branco da camada não voltou"
    );
}

/// **O PAPEL VIAJA COM O DOCUMENTO** — trocar de sprite e voltar devolve o papel ao seu documento, e
/// o outro nasce sem ele.
#[test]
fn o_papel_viaja_com_o_documento() {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let branco = || vec![255u8; 64 * 64 * 4];
    let mut t = PainterTool::default();
    t.bind_document(1, branco(), 64, 64);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        "230,200,150".into(),
    ));
    t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_PAPER_APPLY));
    assert_eq!(t.papel(), Some(CREME));
    t.bind_document(2, branco(), 64, 64);
    assert_eq!(t.papel(), None, "o outro documento herdou o papel");
    t.bind_document(1, branco(), 64, 64);
    assert_eq!(
        t.papel(),
        Some(CREME),
        "o papel não voltou com o seu documento"
    );
}

/// ⭐ **NO DIGITAL A TINTA ENTRA NO DENTE** (dono: *«digital realmente cobre o papel, mas não
/// deveria»*) — com o Relief ligado e o papel Cold, o Tooth faz a CAMADA variar com o papel sob o
/// traço (menos tinta nos vales); Tooth 0 é o traço de antes ao byte. No Impasto (*«reconhece
/// corretamente o papel»*) o Tooth não mexe na camada.
#[test]
fn no_digital_a_tinta_entra_no_dente() {
    let traco = |meio: PaintMedia, relief: f32, tooth: f32| {
        let mut t = tool(128, meio, 30.0);
        t.set_brush_paper_kind(26);
        t.paint.substrate_depth = relief;
        t.paint.brush.paper_depth = tooth;
        t.set_brush_color_srgb8([30, 60, 220]);
        t.on_canvas_pointer(cp([8.0, 64.0], PointerPhase::Down));
        for k in 1..=28 {
            #[allow(clippy::cast_precision_loss)]
            t.on_canvas_pointer(cp([8.0 + 4.0 * k as f32, 64.0], PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
        t.on_canvas_pointer(cp([120.0, 64.0], PointerPhase::Up));
        t.canvas_rgba.to_vec()
    };
    let miolo = |c: &[u8]| -> Vec<u8> {
        (56..72)
            .flat_map(|y| (40..88).map(move |x| (y * 128 + x) * 4))
            .map(|i| c[i])
            .collect()
    };
    let (sem_relevo, zero, dois) = (
        traco(PaintMedia::Digital, 0.0, 2.0),
        traco(PaintMedia::Digital, 1.0, 0.0),
        traco(PaintMedia::Digital, 1.0, 2.0),
    );
    assert_eq!(zero, sem_relevo, "Tooth 0 mudou o traço do Digital");
    let m = miolo(&dois);
    let (lo, hi) = (
        *m.iter().min().expect("miolo"),
        *m.iter().max().expect("miolo"),
    );
    assert!(
        hi - lo > 40,
        "a tinta não entrou no dente: o vermelho do miolo varia só {lo}..{hi}"
    );
    let base = miolo(&sem_relevo);
    let (blo, bhi) = (
        *base.iter().min().expect("miolo"),
        *base.iter().max().expect("miolo"),
    );
    assert!(
        bhi - blo <= 2,
        "controlo: sem o dente o miolo é chapado ({blo}..{bhi})"
    );
    assert_eq!(
        traco(PaintMedia::Impasto, 1.0, 0.0),
        traco(PaintMedia::Impasto, 1.0, 2.0),
        "o Tooth mexeu na camada do Impasto"
    );
    // Com um Grain escolhido, ELE é a textura do traço: o Tooth não mexe.
    let com_grain = |tooth: f32| {
        let mut t = tool(128, PaintMedia::Digital, 30.0);
        t.set_brush_paper_kind(26);
        t.set_brush_texture_kind(1);
        t.paint.substrate_depth = 1.0;
        t.paint.brush.paper_depth = tooth;
        t.on_canvas_pointer(cp([8.0, 64.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([120.0, 64.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([120.0, 64.0], PointerPhase::Up));
        t.canvas_rgba.to_vec()
    };
    assert_eq!(
        com_grain(0.0),
        com_grain(2.0),
        "com um Grain escolhido o Tooth mexeu no traço"
    );
}

/// **O APPLY ASSA O PAPEL NO SPRITE** — o «bake» do Apply é a imagem mostrada: com papel, o papel
/// limpo sai da cor do papel e opaco.
#[test]
fn o_apply_assa_o_papel() {
    use ph2d_editor_core::tool::RasterEditTool;
    let mut t = com_papel(PaintMedia::Digital, CREME, true);
    let (px, _, _) = t.run_full();
    let i = (20 * 128 + 64) * 4;
    assert_eq!(
        px[i..i + 4],
        [CREME[0], CREME[1], CREME[2], 255],
        "o Apply perdeu o papel"
    );
}

/// **A PISTA RÁPIDA TAMBÉM COMPÕE SOBRE O PAPEL** — depois de a imagem inteira ser entregue, um traço
/// novo recompõe só a região suja: o papel em volta dele, dentro dessa região, continua o papel.
#[test]
fn a_regiao_suja_compoe_sobre_o_papel() {
    let mut t = com_papel(PaintMedia::Digital, CREME, true);
    let _ = mostrado(&mut t, 0, 0); // entrega a imagem inteira
    t.on_canvas_pointer(cp([40.0, 100.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 100.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([60.0, 100.0], PointerPhase::Up));
    let (px, _, _) = t.take_preview_arc().expect("a região suja");
    // Dentro da caixa do traço (raio 10: x 30..71, y 90..111) e fora da tinta (a 12,7 px do 1.º ponto).
    let i = (91 * 128 + 31) * 4;
    assert_eq!(
        px[i..i + 4],
        [CREME[0], CREME[1], CREME[2], 255],
        "a região recomposta perdeu o papel"
    );
}

/// **NA AGUADA O PAPEL É O CHÃO ÓPTICO** — o mesmo traço sobre o papel aplicado e sobre uma camada de
/// baixo pintada com a cor do papel dá a mesma imagem, ao byte: a óptica vê o papel como vê a camada.
#[test]
fn na_aguada_o_papel_e_o_chao_optico() {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let traco = |t: &mut PainterTool| {
        t.set_brush_color_srgb8([30, 60, 220]);
        t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
        for k in 1..=22 {
            #[allow(clippy::cast_precision_loss)]
            t.on_canvas_pointer(cp([20.0 + 4.0 * k as f32, 64.0], PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
        t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
    };
    let mut a = tool(128, PaintMedia::Watercolor, 10.0);
    a.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        "230,200,150".into(),
    ));
    a.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_PAPER_APPLY));
    traco(&mut a);
    let mut b = tool(128, PaintMedia::Watercolor, 10.0);
    let creme: Vec<u8> = (0..128 * 128)
        .flat_map(|_| [CREME[0], CREME[1], CREME[2], 255])
        .collect();
    b.add_raster_layer_with_pixels("papel", creme)
        .expect("a camada de baixo");
    b.add_raster_layer("tinta").expect("a camada da tinta");
    traco(&mut b);
    a.invalidate_composite();
    b.invalidate_composite();
    let (pa, _, _) = a.take_preview_arc().expect("a");
    let (pb, _, _) = b.take_preview_arc().expect("b");
    let difere = pa
        .chunks(4)
        .zip(pb.chunks(4))
        .filter(|(x, y)| x != y)
        .count();
    assert_eq!(
        difere, 0,
        "a aguada sobre o papel difere da aguada sobre a camada da mesma cor"
    );
    assert!(
        pa.chunks(4).any(|p| p[2] > p[0] + 20),
        "controlo: a aguada pintou"
    );
}

/// **O «USE AS» VÊ O PAPEL** — a luminância do documento que o Grain/Paper «Use as» lê é a da imagem
/// mostrada: o papel limpo creme lê `203`, não o `255` do branco.
#[test]
fn o_use_as_ve_o_papel() {
    let t = com_papel(PaintMedia::Digital, CREME, true);
    let (lum, w, _) = t.composite_to_lum().expect("a luminância do documento");
    assert_eq!(lum[20 * w as usize + 64], 203, "o «Use as» não vê o papel");
}
