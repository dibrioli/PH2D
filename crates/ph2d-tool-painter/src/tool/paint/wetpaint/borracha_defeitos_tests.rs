//! **Relief Eraser, relato do dono 06/11**: as sombras apagadas em rectângulos depois de sair e voltar, e a tinta de
//! outros modos a sair de uma vez. Sonda: `PH2D_SONDA_DIR=<pasta> cargo test --profile smoke -p ph2d-tool-painter
//! sonda_relief_defeitos -- --ignored --nocapture`.

use super::agua_sobre_oleo_tests::{PAPEL, ponteiro, preset, relato};
use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};

const RELIEF: &str = "Eraser/Relief/Relief Eraser.ph2d-brush.png";

/// O caminho da cena: o dos gates (relógio fixo, compõe no evento) ou o do app (worker, compositor, compõe por quadro).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Caminho {
    Fixo,
    App,
}

pub(super) fn quadros(t: &mut PainterTool, c: Caminho, n: u32) {
    match c {
        Caminho::Fixo => t.ticks_em_blocos(n),
        Caminho::App => {
            for _ in 0..n {
                std::thread::sleep(std::time::Duration::from_millis(16));
                t.on_tick(1000.0 / 60.0);
                let _ = t.take_preview_arc();
            }
        }
    }
}

/// O Dabber assentado em `y = 90`, sobre papel de cor, numa tela `w × h`.
pub(super) fn oleo(w: usize, h: usize, c: Caminho) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    match c {
        Caminho::Fixo => t.set_wet_relogio_fixo(true),
        Caminho::App => t.set_compor_por_quadro(true),
    }
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(ponteiro(40.0, 90.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(40.0 + 4.0 * k as f32, 90.0, PointerPhase::Move));
        quadros(&mut t, c, 1);
    }
    t.on_canvas_pointer(ponteiro(200.0, 90.0, PointerPhase::Up));
    quadros(&mut t, c, 60);
    t
}

pub(super) fn imagem(t: &mut PainterTool) -> Vec<u8> {
    t.invalidate_composite();
    let (px, _, _) = t.take_preview_arc().expect("a imagem");
    px.to_vec()
}

/// Uma passada da Borracha de `a` a `b`, com pressão plena.
pub(super) fn passada(t: &mut PainterTool, c: Caminho, a: (f32, f32), b: (f32, f32)) {
    let p = |f: f32, phase| ponteiro(a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f, phase);
    t.on_canvas_pointer(p(0.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(p(k as f32 / 30.0, PointerPhase::Move));
        quadros(t, c, 1);
    }
    t.on_canvas_pointer(p(1.0, PointerPhase::Up));
    quadros(t, c, 30);
}

/// `(px, Δ máx, x0, y0, x1, y1)` dos píxeis que mudaram fora do `raio` em torno do segmento.
fn fora_do_rasto(
    (a, b): (&[u8], &[u8]),
    w: usize,
    seg: ((f32, f32), (f32, f32)),
    raio: f32,
) -> (usize, u8, usize, usize, usize, usize) {
    let ((ax, ay), (bx, by)) = seg;
    let mut s = (0, 0u8, usize::MAX, usize::MAX, 0, 0);
    for (i, (p, q)) in a.chunks(4).zip(b.chunks(4)).enumerate() {
        if p == q {
            continue;
        }
        let (x, y) = (i % w, i / w);
        let (dx, dy) = (bx - ax, by - ay);
        let f =
            (((x as f32 - ax) * dx + (y as f32 - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        let (ex, ey) = (x as f32 - ax - f * dx, y as f32 - ay - f * dy);
        if (ex * ex + ey * ey).sqrt() <= raio {
            continue;
        }
        s.0 += 1;
        s.1 =
            s.1.max((0..4).map(|k| p[k].abs_diff(q[k])).max().unwrap_or(0));
        s.2 = s.2.min(x);
        s.3 = s.3.min(y);
        s.4 = s.4.max(x);
        s.5 = s.5.max(y);
    }
    s
}

fn grava_em(nome: &str, px: &[u8], w: usize, h: usize) {
    let Ok(dir) = std::env::var("PH2D_SONDA_DIR") else {
        return;
    };
    let mut v = format!("P6\n{w} {h}\n255\n").into_bytes();
    for p in px.chunks(4) {
        // Sobre branco: a falta de alfa aparece.
        for k in 0..3 {
            v.push(
                ((u32::from(p[k]) * u32::from(p[3]) + 255 * (255 - u32::from(p[3]))) / 255) as u8,
            );
        }
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), v).unwrap();
}

#[test]
#[ignore = "sonda: corre por nome com --ignored"]
fn sonda_relief_cenas() {
    use super::adormecida_tests::traco_de_outro_meio;
    let sobre = ((120.0, 60.0), (141.0, 120.0));
    let ao_lado = ((60.0, 125.0), (200.0, 135.0));
    for c in [Caminho::Fixo, Caminho::App] {
        for fundo in [255u8, 0] {
            for (cena, meio, traco) in [
                ("nada", PaintMedia::Digital, None),
                (
                    "dig_longe",
                    PaintMedia::Digital,
                    Some(((20.0, 150.0), (220.0, 150.0))),
                ),
                (
                    "dig_cola",
                    PaintMedia::Digital,
                    Some(((20.0, 112.0), (220.0, 112.0))),
                ),
                (
                    "agu_longe",
                    PaintMedia::Watercolor,
                    Some(((20.0, 150.0), (220.0, 150.0))),
                ),
                (
                    "imp_longe",
                    PaintMedia::Impasto,
                    Some(((20.0, 150.0), (220.0, 150.0))),
                ),
                ("imediato", PaintMedia::Digital, None),
            ] {
                for (zona, seg) in [("sobre", sobre), ("lado", ao_lado)] {
                    let (w, h) = (240usize, 160usize);
                    let mut t = PainterTool::default();
                    t.abre_a_sprite(vec![fundo; w * h * 4], w as u32, h as u32);
                    t.set_paint_tool_mode("wetpaint");
                    match c {
                        Caminho::Fixo => t.set_wet_relogio_fixo(true),
                        Caminho::App => t.set_compor_por_quadro(true),
                    }
                    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
                        .expect("o meio");
                    t.set_brush_size_px(30.0);
                    t.set_brush_color_srgb8([40, 90, 220]);
                    t.on_canvas_pointer(ponteiro(40.0, 90.0, PointerPhase::Down));
                    for k in 1..=40 {
                        t.on_canvas_pointer(ponteiro(
                            40.0 + 4.0 * k as f32,
                            90.0,
                            PointerPhase::Move,
                        ));
                        quadros(&mut t, c, 1);
                    }
                    t.on_canvas_pointer(ponteiro(200.0, 90.0, PointerPhase::Up));
                    if cena != "imediato" {
                        quadros(&mut t, c, 40);
                    }
                    let saindo = imagem(&mut t);
                    t.set_paint_media(meio);
                    let saiu = imagem(&mut t);
                    if let Some((a, b)) = traco {
                        traco_de_outro_meio(&mut t, a, b);
                        quadros(&mut t, c, 3);
                    }
                    let com = imagem(&mut t);
                    t.preset_aplicar(&preset(RELIEF)).expect("o meio");
                    quadros(&mut t, c, 3);
                    let antes = imagem(&mut t);
                    t.set_brush_size_px(20.0);
                    passada(&mut t, c, seg.0, seg.1);
                    let r = imagem(&mut t);
                    let f = fora_do_rasto((&antes, &r), w, seg, 32.0);
                    let dif = |a: &[u8], b: &[u8]| {
                        a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
                    };
                    relato(format!(
                        "{c:?} fundo={fundo:3} {cena:9} {zona:5} sair {} voltar {} | fora: {} px Δ{} ({},{})-({},{})",
                        dif(&saindo, &saiu),
                        dif(&com, &antes),
                        f.0,
                        f.1,
                        f.2,
                        f.3,
                        f.4,
                        f.5
                    ));
                    grava_em(&format!("c_{c:?}_{fundo}_{cena}_{zona}"), &r, w, h);
                    grava_em(
                        &format!("c_{c:?}_{fundo}_{cena}_{zona}_antes"),
                        &antes,
                        w,
                        h,
                    );
                }
            }
        }
    }
}

/// A lei de antes: a `base` corta seco (a régua do antes × depois, no mesmo processo).
pub(super) mod corte_seco {
    use std::cell::Cell;
    thread_local! { static LIGADO: Cell<bool> = const { Cell::new(false) }; }
    pub(in crate::tool::paint) fn get() -> bool {
        LIGADO.with(Cell::get)
    }
    pub(super) fn set(v: bool) {
        LIGADO.with(|c| c.set(v));
    }
}

const W: usize = 240;
const H: usize = 160;

fn dif(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

/// Os píxeis fora do rasto com Δ ≥ 10: uma sombra apagada, ou tinta de volta.
fn fortes_fora(a: &[u8], b: &[u8], seg: ((f32, f32), (f32, f32)), raio: f32) -> usize {
    let ((ax, ay), (bx, by)) = seg;
    a.chunks(4)
        .zip(b.chunks(4))
        .enumerate()
        .filter(|(i, (p, q))| {
            let (x, y) = ((i % W) as f32, (i / W) as f32);
            let (dx, dy) = (bx - ax, by - ay);
            let f = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
            let (ex, ey) = (x - ax - f * dx, y - ay - f * dy);
            (ex * ex + ey * ey).sqrt() > raio && (0..4).any(|k| p[k].abs_diff(q[k]) >= 10)
        })
        .count()
}

/// O traço de outro meio numa cena de óleo, a volta à Relief Eraser e a imagem de então.
fn com_outro_meio(
    c: Caminho,
    meio: PaintMedia,
    a: (f32, f32),
    b: (f32, f32),
) -> (PainterTool, Vec<u8>) {
    let mut t = oleo(W, H, c);
    t.set_paint_media(meio);
    super::adormecida_tests::traco_de_outro_meio(&mut t, a, b);
    quadros(&mut t, c, 3);
    t.preset_aplicar(&preset(RELIEF)).expect("o meio");
    t.set_brush_size_px(20.0);
    quadros(&mut t, c, 3);
    let antes = imagem(&mut t);
    (t, antes)
}

const SOBRE: ((f32, f32), (f32, f32)) = ((120.0, 60.0), (141.0, 120.0));

/// ⭐ Defeito A do dono: com aguada na camada, a caixa recomposta perdia a sombra macia do óleo num rectângulo.
#[test]
fn a_sombra_do_relevo_fica_depois_da_aguada() {
    for c in [Caminho::Fixo, Caminho::App] {
        let (mut t, antes) =
            com_outro_meio(c, PaintMedia::Watercolor, (20.0, 150.0), (220.0, 150.0));
        passada(&mut t, c, SOBRE.0, SOBRE.1);
        let n = fortes_fora(&antes, &imagem(&mut t), SOBRE, 32.0);
        assert_eq!(n, 0, "{c:?}: {n} px perderam a sombra fora do rasto");
    }
}

/// ⭐ Defeito A: colado ao óleo, o traço de outro modo enterra células; a caixa não põe a tinta de lá de volta.
#[test]
fn a_tinta_enterrada_nao_volta_na_margem_da_caixa() {
    let azul = |p: &[u8]| p[2] > 120 && p[0] < 110;
    for c in [Caminho::Fixo, Caminho::App] {
        let (mut t, antes) = com_outro_meio(c, PaintMedia::Digital, (20.0, 112.0), (220.0, 112.0));
        passada(&mut t, c, SOBRE.0, SOBRE.1);
        let r = imagem(&mut t);
        // Sobre o vermelho: y ≥ 104, longe do rasto.
        let voltou = (104..118usize)
            .flat_map(|y| (20..220usize).map(move |x| (x, y)))
            .filter(|&(x, _)| !(100..165).contains(&x))
            .filter(|&(x, y)| {
                let i = (y * W + x) * 4;
                azul(&r[i..i + 4]) && !azul(&antes[i..i + 4])
            })
            .count();
        assert_eq!(voltou, 0, "{c:?}: {voltou} px de óleo enterrado voltaram");
    }
}

/// Adormecer com uma imagem no ar: o compositor morre com o worker, e a resposta nunca vem.
#[test]
fn adormecer_nao_deixa_uma_imagem_no_ar() {
    let mut t = oleo(W, H, Caminho::App);
    t.paint
        .wetpaint
        .session
        .as_mut()
        .expect("a sessão")
        .otica
        .toca_tudo((W, H));
    t.set_paint_media(PaintMedia::Digital);
    let a = t.paint.wetpaint.adormecidas.first().expect("adormecida");
    let s = a.sess.lock().expect("trinco");
    assert!(!s.otica.pendente());
    assert!(s.otica.uniao_local.is_none());
}

/// O alfa e o «tem tinta visível» de um rasto de `(x, y)`.
fn ponto(p: &[u8], x: usize, y: usize) -> [u8; 4] {
    let i = (y * W + x) * 4;
    [p[i], p[i + 1], p[i + 2], p[i + 3]]
}

/// ⭐ Defeito B, a cena do dono: óleo e, ao lado, um traço de cada outro modo. Uma passada não chega ao papel no traço;
/// várias chegam.
#[test]
fn o_traco_de_outro_modo_sai_aos_poucos() {
    let seg = ((90.0, 125.0), (150.0, 125.0));
    for meio in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
    ] {
        let mut t = oleo(W, H, Caminho::Fixo);
        t.set_paint_media(meio);
        super::adormecida_tests::traco_de_outro_meio(&mut t, (30.0, 125.0), (210.0, 125.0));
        t.preset_aplicar(&preset(RELIEF)).expect("o meio");
        t.set_brush_size_px(20.0);
        t.ticks_em_blocos(3);
        let alfa = |t: &PainterTool| t.canvas_rgba[(125 * W + 120) * 4 + 3];
        let alvo = alfa(&t);
        assert!(alvo > 40, "{meio:?}: CONTROLO {alvo}");
        passada(&mut t, Caminho::Fixo, seg.0, seg.1);
        let uma = alfa(&t);
        assert!(
            f32::from(uma) > f32::from(alvo) * 0.4,
            "{meio:?}: uma passada chegou ao papel: alfa {uma} de {alvo}"
        );
        let mut passadas = 1;
        while alfa(&t) > 8 && passadas < 20 {
            passada(&mut t, Caminho::Fixo, seg.0, seg.1);
            passadas += 1;
        }
        relato(format!(
            "{meio:?}: alfa {alvo} → {uma} numa passada; ao papel em {passadas}"
        ));
        assert!(passadas < 20, "{meio:?}: nunca chega ao papel");
    }
}

/// Arquivo antigo: a tinta velha na `base`, sessão nascida com um traço longe, e apagar a velha também é aos poucos.
#[test]
fn a_tinta_velha_sai_aos_poucos_com_sessao_viva() {
    let mut px = vec![0u8; W * H * 4];
    for y in 110..150 {
        for x in 30..210 {
            px[(y * W + x) * 4..(y * W + x) * 4 + 4].copy_from_slice(&[200, 40, 40, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.abre_a_sprite(px, W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(12.0);
    t.on_canvas_pointer(ponteiro(40.0, 30.0, PointerPhase::Down));
    t.on_canvas_pointer(ponteiro(80.0, 30.0, PointerPhase::Move));
    t.on_canvas_pointer(ponteiro(80.0, 30.0, PointerPhase::Up));
    t.ticks_em_blocos(10);
    t.preset_aplicar(&preset(RELIEF)).expect("o meio");
    t.set_brush_size_px(20.0);
    assert!(t.paint.wetpaint.session.is_some(), "a sessão viva");
    passada(&mut t, Caminho::Fixo, (90.0, 130.0), (150.0, 130.0));
    let a = ponto(&imagem(&mut t), 120, 130)[3];
    assert!(a > 100, "uma passada cortou seco: alfa {a}");
    for _ in 0..12 {
        passada(&mut t, Caminho::Fixo, (90.0, 130.0), (150.0, 130.0));
    }
    let a = ponto(&imagem(&mut t), 120, 130)[3];
    assert!(a < 8, "várias passadas não chegaram ao papel: alfa {a}");
}

/// O undo devolve a tinta da `base` que a passada gradual tirou, ao bit.
#[test]
fn desfazer_a_passada_devolve_a_tinta_da_base() {
    let (mut t, antes) = com_outro_meio(
        Caminho::Fixo,
        PaintMedia::Digital,
        (30.0, 125.0),
        (210.0, 125.0),
    );
    let base_antes = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .base
        .to_vec();
    passada(&mut t, Caminho::Fixo, (90.0, 125.0), (150.0, 125.0));
    assert!(
        dif(&antes, &imagem(&mut t)) > 100,
        "CONTROLO: a passada apaga"
    );
    assert!(t.undo_last(), "o undo aplica-se");
    assert_eq!(dif(&antes, &imagem(&mut t)), 0, "a tela");
    if let Some(s) = t.paint.wetpaint.session.as_ref() {
        assert_eq!(dif(&base_antes, &s.base), 0, "a base");
    }
}

/// Óleo vivo e traço enterrado na mesma passada: o óleo sai igual, e a passada não deixa um rectângulo à volta.
#[test]
fn a_passada_sobre_oleo_e_traco_nao_deixa_rectangulo() {
    let seg = ((120.0, 70.0), (135.0, 140.0));
    let (mut t, antes) = com_outro_meio(
        Caminho::Fixo,
        PaintMedia::Digital,
        (30.0, 125.0),
        (210.0, 125.0),
    );
    passada(&mut t, Caminho::Fixo, seg.0, seg.1);
    let r = imagem(&mut t);
    grava_em("sobrepoe_antes", &antes, W, H);
    grava_em("sobrepoe", &r, W, H);
    let n = fortes_fora(&antes, &r, seg, 40.0);
    assert_eq!(n, 0, "{n} px fora do rasto");
}

/// ⭐ A Borracha sobre o óleo vivo aprovada: nas células com tinta nada muda ao bit contra a lei de antes.
#[test]
fn a_borracha_no_oleo_vivo_nao_muda() {
    let cena = |seco: bool| {
        corte_seco::set(seco);
        let mut t = super::agua_sobre_oleo_tests::com_oleo();
        t.set_paint_tool_mode("eraser");
        let g = {
            let s = t.paint.wetpaint.session.as_mut().expect("a sessão");
            s.bring_home();
            let g = s.engine.active_grid();
            (0..W * H)
                .map(|i| {
                    let k = (i / W + 1) * g.s + i % W + 1;
                    g.sett[k] > 0.0 || g.susp[k] > 0.0
                })
                .collect::<Vec<_>>()
        };
        let r = super::borracha_de_fabrica_tests::apaga(&mut t);
        corte_seco::set(false);
        (r, g)
    };
    let (seco, tinta) = cena(true);
    let (gradual, _) = cena(false);
    let total = dif(&seco, &gradual);
    let nas_celulas = seco
        .chunks(4)
        .zip(gradual.chunks(4))
        .zip(&tinta)
        .filter(|((a, b), t)| **t && a != b)
        .count();
    let forte = seco
        .chunks(4)
        .zip(gradual.chunks(4))
        .filter(|(a, b)| (0..4).any(|k| a[k].abs_diff(b[k]) >= 10))
        .count();
    relato(format!(
        "óleo vivo: lei de antes × gradual: {total} px diferentes, {nas_celulas} em células com tinta, {forte} com Δ ≥ 10"
    ));
    grava_em("viva_seco", &seco, W, H);
    grava_em("viva_gradual", &gradual, W, H);
    assert_eq!(forte, 0);
}

#[test]
#[ignore = "sonda: corre por nome com --ignored"]
fn sonda_passadas_ate_o_papel() {
    let vermelho = |p: &[u8]| p[0] > 150 && p[1] < 90 && p[3] > 40;
    let azul = |p: &[u8]| p[2] > 120 && p[0] < 110 && p[3] > 40;
    let rasto = |p: &[u8]| {
        (70..110usize)
            .map(|y| {
                let x = 120 + (y - 60) * 7 / 20;
                let i = (y * W + x) * 4;
                usize::from(vermelho(&p[i..i + 4]) || azul(&p[i..i + 4]))
            })
            .sum::<usize>()
    };
    for (nome, pinta) in [("só óleo", false), ("traço por cima", true)] {
        let mut t = super::agua_sobre_oleo_tests::com_oleo();
        if pinta {
            t.set_paint_media(PaintMedia::Digital);
            super::adormecida_tests::traco_de_outro_meio(&mut t, (130.0, 60.0), (130.0, 120.0));
            t.set_paint_media(PaintMedia::WetPaint);
            t.ticks_em_blocos(10);
        }
        t.set_paint_tool_mode("eraser");
        let mut linha = format!("{nome}, pressão variável (resto no rasto de 40 px):");
        for n in 1..=12 {
            let r = super::borracha_de_fabrica_tests::apaga(&mut t);
            linha += &format!(" {n}:{}", rasto(&r));
        }
        relato(linha);
    }
}

/// O defeito A com a tinta comprimida à força em RAM, pelo mesmo código do fio: a mesma sombra.
#[test]
fn a_sombra_fica_com_a_tinta_comprimida() {
    let (w, h) = (2400usize, 1400usize);
    let mut t = oleo(w, h, Caminho::App);
    t.set_paint_media(PaintMedia::Watercolor);
    super::adormecida_tests::traco_de_outro_meio(&mut t, (20.0, 150.0), (220.0, 150.0));
    t.paint.wetpaint.adormecidas[0].encolhe_ja();
    t.preset_aplicar(&preset(RELIEF)).expect("o meio");
    t.set_brush_size_px(20.0);
    quadros(&mut t, Caminho::App, 3);
    let antes = imagem(&mut t);
    passada(&mut t, Caminho::App, SOBRE.0, SOBRE.1);
    let r = imagem(&mut t);
    let linha = |p: &[u8]| p[..200 * w * 4].to_vec();
    let fortes = linha(&antes)
        .chunks(4)
        .zip(linha(&r).chunks(4))
        .enumerate()
        .filter(|(i, (p, q))| {
            let (x, y) = (i % w, i / w);
            ((x as i32 - 130).abs() > 60 || (y as i32 - 90).abs() > 60)
                && (0..4).any(|k| p[k].abs_diff(q[k]) >= 10)
        })
        .count();
    assert_eq!(fortes, 0, "{fortes} px perderam a sombra");
}
