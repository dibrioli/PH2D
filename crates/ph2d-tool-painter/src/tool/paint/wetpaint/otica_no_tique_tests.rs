//! J13b P6 ([plano](../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md) §4 portões 2–3): com a
//! thread REAL e o compositor de pé, a tela que as imagens deixam é a composição de casa AO BYTE; o fio da tela não
//! paga a óptica com a água a correr; e uma imagem mais velha do que um dab composto em casa não deixa píxel velho.
//!
//! O interruptor liga-se por [`forca_no_teste`] (do fio do teste, lido quando o worker sobe), nunca pelo global:
//! os outros portões correm em paralelo no mesmo processo.

use super::agua_sobre_oleo_tests::preset;
use super::offthread::otica::{CaixaCel, forca_no_teste};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use std::time::{Duration, Instant};

pub(super) const LADO: usize = 192;
const QUADRO: Duration = Duration::from_millis(16);
const DT_MS: f32 = 1000.0 / 60.0;

pub(super) fn ponto(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.7,
        tilt: [0.0, 0.0],
        phase,
    }
}

pub(super) fn tique(t: &mut PainterTool) {
    t.on_tick(DT_MS);
    let _ = t.take_preview_arc();
}

pub(super) fn quadros(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        tique(t);
        std::thread::sleep(QUADRO);
    }
}

fn traco(t: &mut PainterTool, rel: &str, cor: [u8; 3], y: f32, smooth: f32, veu: bool) {
    t.preset_aplicar(&preset(rel)).expect("o preset");
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8(cor);
    t.set_wet_smooth_edges(smooth);
    t.paint.wetpaint.show_wet = veu;
    let lado = LADO as f32;
    let _ = t.on_canvas_pointer(ponto(lado * 0.15, y, PointerPhase::Down));
    for k in 1..=20 {
        for s in 0..3 {
            let f = (k - 1) as f32 + (s + 1) as f32 / 3.0;
            let x = lado * 0.15 + lado * 0.7 * f / 20.0;
            let _ = t.on_canvas_pointer(ponto(x, y + 0.1 * f, PointerPhase::Move));
        }
        tique(t);
        std::thread::sleep(QUADRO);
    }
    let _ = t.on_canvas_pointer(ponto(lado * 0.85, y, PointerPhase::Up));
}

/// A cena do J13 com a thread real: Dabber com Real Oil, depois Wet Drip a correr.
pub(super) fn cena(fora: bool, smooth: f32, veu: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
    t.set_paint_tool_mode("wetpaint");
    forca_no_teste(Some(fora));
    let lado = LADO as f32;
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    t.set_wet_oleo_real_ligado(true);
    traco(
        &mut t,
        "Oil/Dabber/Dabber.ph2d-brush.png",
        [40, 90, 220],
        lado * 0.4,
        smooth,
        veu,
    );
    quadros(&mut t, 6);
    let wd = "Watercolor/Fluid/Wet Drip.ph2d-brush.png";
    traco(&mut t, wd, [230, 200, 30], lado * 0.5, smooth, veu);
    assert_eq!(
        sess(&t).tem_compositor(),
        fora,
        "premissa: o compositor existe só com o interruptor"
    );
    t
}

pub(super) fn sess(t: &PainterTool) -> &WetSession {
    t.paint.wetpaint.session.as_ref().expect("sessao")
}

/// A água PÁRA (o traço do motor aberto, como no P5) com o motor a ir para o worker, e os tiques correm até a tela
/// mostrar o último passo e não haver nada no ar.
pub(super) fn assenta(t: &mut PainterTool) {
    congela(t);
    let t0 = Instant::now();
    loop {
        tique(t);
        let s = sess(t);
        if !s.otica.pendente() && s.sim_steps() == s.seen_steps && t0.elapsed() > QUADRO * 4 {
            break;
        }
        assert!(
            t0.elapsed() < Duration::from_secs(10),
            "a água não assentou"
        );
        std::thread::sleep(Duration::from_millis(4));
    }
}

pub(super) fn congela(t: &mut PainterTool) {
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    s.bring_home();
    s.engine.pointer_down(1.0, 1.0, None);
    assert!(!s.engine.sim_should_run(), "premissa: a água parada");
}

/// A tela de agora contra a composição de casa da tela inteira, sobre o mesmo motor.
pub(super) fn confere_com_casa(t: &mut PainterTool, onde: &str) {
    let vista = t.canvas_rgba.to_vec();
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    s.bring_home();
    s.engine.mark_dirty_full();
    t.wetpaint_composite();
    let casa = t.canvas_rgba.as_ref();
    let n = vista.iter().zip(casa).filter(|(a, b)| a != b).count();
    if let Some(i) = vista.iter().zip(casa).position(|(a, b)| a != b) {
        let px = i / 4;
        assert_eq!(
            n,
            0,
            "{onde}: a tela difere da composição de casa em ({}, {}) — tela {:?} casa {:?}",
            px % LADO,
            px / LADO,
            &vista[px * 4..px * 4 + 4],
            &casa[px * 4..px * 4 + 4],
        );
    }
    assert!(
        casa.as_chunks::<4>().0.iter().any(|p| *p != [255; 4]),
        "{onde}: a cena não pintou nada"
    );
}

#[test]
fn a_agua_pelo_compositor_e_a_tela_de_casa_ao_byte() {
    for (fora, smooth, veu) in [(false, 0.0, false), (true, 0.0, false), (true, 0.6, true)] {
        let mut t = cena(fora, smooth, veu);
        quadros(&mut t, 30);
        assenta(&mut t);
        let s = sess(&t);
        if fora {
            assert!(
                s.otica.aplicadas >= 5,
                "só {} imagens aplicadas — a cena não passou pelo compositor",
                s.otica.aplicadas
            );
            assert!(
                t.paint.wetpaint.oleo_na_tela,
                "o Real Oil não chegou à tela pela imagem"
            );
        } else {
            assert_eq!(
                s.otica.aplicadas, 0,
                "controlo: sem compositor, nenhuma imagem"
            );
        }
        confere_com_casa(&mut t, &format!("fora {fora} smooth {smooth} véu {veu}"));
    }
}

#[test]
fn o_fio_da_tela_nao_paga_a_otica() {
    for fora in [true, false] {
        let mut t = cena(fora, 0.0, false);
        // O pendente do último quadro do traço compõe-se em casa: fica fora da janela.
        quadros(&mut t, 3);
        let _ = super::composite_contagem::take();
        let antes = sess(&t).otica.aplicadas;
        let p0 = sess(&t).sim_steps();
        quadros(&mut t, 40);
        let (chamadas, _, _) = super::composite_contagem::take();
        let s = sess(&t);
        assert!(s.sim_steps() > p0 + 5, "premissa: a água andou");
        if fora {
            assert_eq!(
                chamadas, 0,
                "o fio da tela compôs a óptica {chamadas} vezes"
            );
            assert!(s.otica.aplicadas > antes + 5, "as imagens não chegaram");
        } else {
            assert!(
                chamadas > 5,
                "controlo: sem o compositor o fio da tela compõe ({chamadas})"
            );
        }
    }
}

#[test]
fn a_imagem_velha_nao_deixa_pixel_velho_debaixo_do_dab() {
    let mut t = cena(true, 0.0, false);
    quadros(&mut t, 10);
    // Um pedido no ar, servido pelo worker durante a espera: a imagem fica no canal, mais velha que o dab.
    let t0 = Instant::now();
    loop {
        tique(&mut t);
        if sess(&t).otica.pendente() {
            break;
        }
        assert!(t0.elapsed() < Duration::from_secs(5), "nenhum pedido saiu");
        std::thread::sleep(Duration::from_millis(4));
    }
    std::thread::sleep(Duration::from_millis(150));
    // O dab sobre a água, composto em casa no tique seguinte; a água pára antes dele.
    let lado = LADO as f32;
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    t.set_brush_color_srgb8([200, 40, 40]);
    let _ = t.on_canvas_pointer(ponto(lado * 0.4, lado * 0.5, PointerPhase::Down));
    for k in 0..6 {
        let x = lado * (0.4 + 0.03 * k as f32);
        let _ = t.on_canvas_pointer(ponto(x, lado * 0.5, PointerPhase::Move));
    }
    let _ = t.on_canvas_pointer(ponto(lado * 0.58, lado * 0.5, PointerPhase::Up));
    congela(&mut t);
    let d0 = sess(&t).otica.tocadas;
    tique(&mut t);
    assert!(
        sess(&t).otica.tocadas > d0,
        "premissa: a imagem velha chegou depois do dab e tocou a composição de casa"
    );
    assenta(&mut t);
    confere_com_casa(&mut t, "depois do dab");
}

/// A mesma lei, sem corrida: a água parada, o motor muda SEM compor (o Fast dry do motor, a tela inteira suja), a
/// imagem disso fica no canal, um dab pequeno compõe-se em casa e a imagem chega depois dele. Descartá-la é obrigatório
/// (é mais velha que o dab) e a caixa dela TEM de ir para a dívida (fora do dab, a tela ainda mostra a água molhada).
#[test]
fn a_divida_leva_a_caixa_da_imagem_descartada() {
    let mut t = cena(true, 0.0, false);
    quadros(&mut t, 10);
    assenta(&mut t);
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    s.bring_home();
    s.engine.fast_dry_now();
    // O motor não andou: só a dívida (uma célula) faz o tique pedir.
    s.otica.divida = Some((1, 1, 1, 1));
    tique(&mut t);
    assert!(sess(&t).otica.pendente(), "premissa: o pedido saiu");
    std::thread::sleep(Duration::from_millis(150));
    let lado = LADO as f32;
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    t.set_brush_color_srgb8([200, 40, 40]);
    let _ = t.on_canvas_pointer(ponto(lado * 0.5, lado * 0.45, PointerPhase::Down));
    let _ = t.on_canvas_pointer(ponto(lado * 0.52, lado * 0.45, PointerPhase::Move));
    let _ = t.on_canvas_pointer(ponto(lado * 0.52, lado * 0.45, PointerPhase::Up));
    congela(&mut t);
    let d0 = sess(&t).otica.tocadas;
    tique(&mut t);
    assert!(
        sess(&t).otica.tocadas > d0,
        "premissa: a imagem do Fast dry chegou depois do dab"
    );
    assenta(&mut t);
    confere_com_casa(&mut t, "Fast dry e dab");
}

/// Um traço curto de Smooth Edges no máximo, sem tique, à direita da caixa pedida ou (`abaixo`) em baixo dela.
fn risca_da_margem(t: &mut PainterTool, abaixo: bool, tamanho: f32, oleo: bool, suave: f32) {
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    risca_com_o_preset(t, abaixo, tamanho, oleo, suave);
}

fn risca_com_o_preset(t: &mut PainterTool, abaixo: bool, tamanho: f32, oleo: bool, suave: f32) {
    t.set_wet_oleo_real_ligado(oleo);
    t.set_brush_size_px(tamanho);
    t.set_brush_color_srgb8([200, 40, 40]);
    t.set_wet_smooth_edges(suave);
    let (a, b) = if abaixo {
        ([30.0, 130.0], [170.0, 130.0])
    } else {
        ([110.0, 90.0], [110.0, 170.0])
    };
    let p = |k: f32, f| ponto(a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, f);
    let _ = t.on_canvas_pointer(p(0.0, PointerPhase::Down));
    for k in 1..=12 {
        let _ = t.on_canvas_pointer(p(k as f32 / 12.0, PointerPhase::Move));
    }
    let _ = t.on_canvas_pointer(p(1.0, PointerPhase::Up));
}

/// A tela só com água a 0, sem óleo (o tecto das grades e os planos em zero) e parada, com o compositor de pé.
fn agua_sem_suave() -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
    t.set_paint_tool_mode("wetpaint");
    forca_no_teste(Some(true));
    let wd = "Watercolor/Fluid/Wet Drip.ph2d-brush.png";
    traco(&mut t, wd, [230, 200, 30], 20.0, 0.0, false);
    quadros(&mut t, 3);
    assenta(&mut t);
    t
}

/// O traço pintado e composto em casa: a célula 0-based onde a mancha começa, no eixo da margem.
fn pinta_e_mede(t: &mut PainterTool, abaixo: bool, tamanho: f32) -> usize {
    let antes = sess(t).planos.pigment.clone();
    risca_da_margem(t, abaixo, tamanho, false, 1.0);
    congela(t);
    tique(t);
    let s = sess(t);
    let (gw, _) = s.grid;
    let mut inicio = usize::MAX;
    for (i, (a, b)) in s.planos.pigment.chunks(4).zip(antes.chunks(4)).enumerate() {
        if a != b && f32::from(a[3]) / 255.0 >= crate::tool::paint::watercolor_render::SS0 {
            inicio = inicio.min(if abaixo { i / gw } else { i % gw });
        }
    }
    assert_ne!(inicio, usize::MAX, "premissa: o traço pintou");
    inicio
}

/// Uma caixa de 4 células no eixo da margem, acabada em `fim` (1-based), na banda do meio do traço.
fn caixa_ate(fim: usize, abaixo: bool, rr: usize) -> CaixaCel {
    let (b0, b1) = if abaixo { (90, 105) } else { (120, 135) };
    let (b0, b1) = (b0 / rr + 1, b1 / rr + 1);
    if abaixo {
        (b0, fim - 3, b1, fim)
    } else {
        (fim - 3, b0, fim, b1)
    }
}

/// **O traço composto dab a dab é a recomposição inteira ao byte**: dentro da tinta o pixel lê a tinta desfocada a `raio`
/// px de cada lado, por isso a região cresce o dobro do raio (com o simples, medido: 1 byte em (105, 86), sem compositor).
#[test]
fn o_traco_suave_dab_a_dab_e_a_recomposicao_inteira_ao_byte() {
    for abaixo in [false, true] {
        let mut t = agua_sem_suave();
        let _ = pinta_e_mede(&mut t, abaixo, 16.0);
        confere_com_casa(&mut t, &format!("dab a dab (abaixo {abaixo})"));
    }
}

/// **A imagem velha não esvazia o que a casa já pintou.** Um pedido de quando a tela não tinha óleo nem Smooth Edges fica
/// no canal, o 1.º traço com os dois compõe-se em casa e a imagem chega depois dele, sem o tocar. Ela dizia «planos
/// vazios» e apagava a sombra e os degraus do traço; a imagem seguinte só traz a caixa dela, e o laço de píxel lia a
/// sombra 0 na margem (Bug #64, medido: 727 píxeis, 1 018 células de sombra). Vai para a dívida, como a que toca.
#[test]
fn a_imagem_velha_nao_esvazia_o_que_a_casa_ja_pintou() {
    for (abaixo, oleo, suave) in [
        (false, true, 1.0),
        (true, true, 1.0),
        (false, false, 1.0),
        (false, true, 0.0),
    ] {
        let mut t = agua_sem_suave();
        let velha = caixa_ate(5, abaixo, usize::from(sess(&t).ratio));
        let s = t.paint.wetpaint.session.as_mut().expect("sessao");
        s.bring_home();
        s.otica.divida = Some(velha);
        tique(&mut t);
        assert!(sess(&t).otica.pendente(), "premissa: o pedido saiu");
        std::thread::sleep(Duration::from_millis(150));
        risca_da_margem(&mut t, abaixo, 16.0, oleo, suave);
        congela(&mut t);
        let (a0, d0) = (sess(&t).otica.aplicadas, sess(&t).otica.tocadas);
        tique(&mut t);
        let s = sess(&t);
        // Respondida, a dívida sai logo noutro pedido, que limpa a união; sem resposta, a união ficava.
        assert!(s.otica.uniao_local.is_none(), "premissa: a imagem chegou");
        assert_eq!(s.otica.tocadas, d0, "premissa: a imagem não toca o traço");
        assert_eq!(s.otica.aplicadas, a0, "a imagem velha foi aplicada");
        assert_eq!(
            s.planos.suave.is_empty(),
            suave == 0.0,
            "a imagem velha mexeu nos degraus"
        );
        assert_eq!(
            s.planos.sombra.is_empty(),
            !oleo,
            "a imagem velha mexeu na sombra"
        );
        assenta(&mut t);
        confere_com_casa(
            &mut t,
            &format!("a imagem velha (abaixo {abaixo}, óleo {oleo}, suave {suave})"),
        );
    }
}

/// Sem grão de papel há sombra sem grão: é este o caso em que só a sombra diz que a imagem velha esvazia.
#[test]
fn a_imagem_velha_nao_esvazia_a_sombra_sem_grao() {
    let mut t = agua_sem_suave();
    let velha = caixa_ate(5, false, usize::from(sess(&t).ratio));
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    s.bring_home();
    s.otica.divida = Some(velha);
    tique(&mut t);
    assert!(sess(&t).otica.pendente(), "premissa: o pedido saiu");
    std::thread::sleep(Duration::from_millis(150));
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    super::grao_no_filme::sem_grao::set(true);
    risca_com_o_preset(&mut t, false, 16.0, true, 0.0);
    congela(&mut t);
    let a0 = sess(&t).otica.aplicadas;
    assert!(!sess(&t).planos.sombra.is_empty(), "premissa: há sombra");
    assert!(sess(&t).planos.grao.is_empty(), "premissa: não há grão");
    tique(&mut t);
    let s = sess(&t);
    assert!(s.otica.uniao_local.is_none(), "premissa: a imagem chegou");
    assert_eq!(s.otica.aplicadas, a0, "a imagem velha foi aplicada");
    assert!(
        !s.planos.sombra.is_empty(),
        "a imagem velha apagou a sombra"
    );
    assenta(&mut t);
    confere_com_casa(&mut t, "a imagem velha, sem papel");
    super::grao_no_filme::sem_grao::set(false);
}

/// **A caixa dos degraus é maior que a do pigmento** (Bug #65 D1): a imagem pedida a `folga` células do traço não toca a
/// caixa dele com o pigmento, mas cola os degraus velhos, `alcance` além, por cima dos que a casa acabou de pousar.
/// Medido antes da cura: 2 389 bytes a Smooth Edges 1,0 e folga 6. O plano já existe: a imagem não vem «vazia».
#[test]
fn a_imagem_velha_nao_cola_degraus_velhos_no_traco_ao_lado() {
    for suave in [1.0, 0.12] {
        for folga in [6, 8, 10, 14] {
            for abaixo in [false, true] {
                let agua = |t: &mut PainterTool| {
                    t.abre_a_sprite(vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
                    t.set_paint_tool_mode("wetpaint");
                    forca_no_teste(Some(true));
                    let wd = "Watercolor/Fluid/Wet Drip.ph2d-brush.png";
                    traco(t, wd, [230, 200, 30], 20.0, suave, false);
                    quadros(t, 3);
                    assenta(t);
                };
                let mut prova = PainterTool::default();
                agua(&mut prova);
                let inicio = pinta_e_mede(&mut prova, abaixo, 16.0);
                let mut t = PainterTool::default();
                agua(&mut t);
                let rr = usize::from(sess(&t).ratio);
                let s = t.paint.wetpaint.session.as_mut().expect("sessao");
                s.bring_home();
                s.otica.divida = Some(caixa_ate(inicio + 1 - folga, abaixo, rr));
                tique(&mut t);
                assert!(sess(&t).otica.pendente(), "premissa: o pedido saiu");
                std::thread::sleep(Duration::from_millis(150));
                risca_da_margem(&mut t, abaixo, 16.0, false, suave);
                congela(&mut t);
                let n0 = sess(&t).otica.aplicadas + sess(&t).otica.tocadas;
                tique(&mut t);
                assert!(
                    sess(&t).otica.aplicadas + sess(&t).otica.tocadas > n0,
                    "premissa: a imagem chegou"
                );
                assenta(&mut t);
                confere_com_casa(
                    &mut t,
                    &format!("suave {suave}, folga {folga}, abaixo {abaixo}"),
                );
            }
        }
    }
}

/// **A tinta suave NA MARGEM da caixa:** com o plano do Smooth Edges a nascer (esvaziado à mão: a porta que o fazia era
/// a imagem velha) e a tinta suave fora do sujo, a imagem tem de trazer o degrau de tudo o que o laço de píxel lê,
/// `alcance(RAIO_MAX)` além da região. Sem isso a tinta na margem lê o degrau 0 e os píxeis da caixa perdem a orla.
#[test]
fn a_tinta_suave_na_margem_da_caixa_e_a_tela_de_casa_ao_byte() {
    for abaixo in [false, true] {
        let mut t = agua_sem_suave();
        let rr = usize::from(sess(&t).ratio);
        let inicio = pinta_e_mede(&mut t, abaixo, 16.0);
        // O resto do sujo do traço sai com uma imagem no canto oposto.
        let s = t.paint.wetpaint.session.as_mut().expect("sessao");
        s.otica.divida = Some(caixa_ate(5, abaixo, rr));
        assenta(&mut t);
        // A região acaba `r` depois da caixa: a duas células da mancha, ao alcance da orla dela.
        let r = super::smooth_edges::alcance_da_mudanca(super::smooth_edges::RAIO_MAX);
        let caixa = caixa_ate(inicio - r / rr - 2, abaixo, rr);
        let s = t.paint.wetpaint.session.as_mut().expect("sessao");
        let inteiro = s.planos.suave.iter().filter(|d| **d > 0).count();
        s.planos.suave = Vec::new();
        s.otica.divida = Some(caixa);
        assenta(&mut t);
        let n = sess(&t).planos.suave.iter().filter(|d| **d > 0).count();
        assert!(
            n < inteiro,
            "premissa: a imagem nova traz só a caixa dela ({n} de {inteiro} células)"
        );
        confere_com_casa(&mut t, &format!("a tinta na margem (abaixo {abaixo})"));
    }
}

/// A tela inteira composta sobre o motor de agora, com o pincel na tela (`na_tela`) ou fora dela.
fn composta(t: &mut PainterTool, na_tela: bool) -> Vec<u8> {
    let (gesto, aberto) = (t.paint.wetpaint.live_gesture, sess(t).stroke_open);
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    s.bring_home();
    s.engine.mark_dirty_full();
    s.stroke_open = na_tela && aberto;
    t.paint.wetpaint.live_gesture = na_tela && gesto;
    assert_eq!(t.wet_pincel_na_tela(), na_tela, "premissa: o pincel");
    t.wetpaint_composite();
    t.paint.wetpaint.live_gesture = gesto;
    t.paint
        .wetpaint
        .session
        .as_mut()
        .expect("sessao")
        .stroke_open = aberto;
    t.canvas_rgba.to_vec()
}

fn difere(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

/// Dono 12/11: a luz do traço é a ÚNICA luz do Óleo Real. O mesmo motor composto com o pincel na tela e fora dela dá
/// o mesmo byte; a tela logo depois do pen-up é a de muitos tiques de repouso e a da recomposição inteira — nada se
/// recompõe com outra luz ao soltar (fora × Live Water).
#[test]
fn a_luz_do_traco_e_a_unica_luz_do_oleo() {
    for (fora, viva) in [(true, false), (true, true), (false, false)] {
        let onde = format!("fora {fora} agua viva {viva}");
        let mut t = cena(fora, 0.0, false);
        assenta(&mut t);
        let lado = LADO as f32;
        t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
            .expect("o preset");
        t.set_wet_oleo_real_ligado(true);
        t.set_wet_agua_viva(viva);
        // A luz rasante: a que mais mexe na sombra (a que separava a antiga luz Cheia do repouso da do traço).
        t.paint.wetpaint.oleo_real_valores.altitude = 12f64.to_radians();
        t.set_brush_size_px(16.0);
        t.set_brush_color_srgb8([200, 40, 40]);
        let _ = t.on_canvas_pointer(ponto(lado * 0.2, lado * 0.45, PointerPhase::Down));
        for k in 1..=12 {
            let x = lado * (0.2 + 0.05 * k as f32);
            let _ = t.on_canvas_pointer(ponto(x, lado * 0.45, PointerPhase::Move));
            tique(&mut t);
            std::thread::sleep(QUADRO);
        }
        quadros(&mut t, 10);
        assert!(t.wet_pincel_na_tela(), "{onde}: premissa: o pincel na tela");
        let (gesto, repouso) = (composta(&mut t, true), composta(&mut t, false));
        assert!(
            gesto.as_chunks::<4>().0.iter().any(|p| *p != [255; 4]),
            "{onde}: premissa: há tinta na tela"
        );
        assert_eq!(
            difere(&gesto, &repouso),
            0,
            "{onde}: o mesmo motor com o pincel na tela e fora dela"
        );
        let _ = t.on_canvas_pointer(ponto(lado * 0.8, lado * 0.45, PointerPhase::Up));
        t.set_wet_agua_viva(false);
        quadros(&mut t, 2);
        assenta(&mut t);
        let solto = t.canvas_rgba.to_vec();
        quadros(&mut t, 30);
        assenta(&mut t);
        assert_eq!(
            difere(&solto, &t.canvas_rgba),
            0,
            "{onde}: a tela logo depois do pen-up × 30 quadros de repouso"
        );
        confere_com_casa(&mut t, &format!("{onde}: o repouso depois do gesto"));
    }
}

/// **B11.3: os RGBA do compositor com Smooth Edges são a tela de casa ao byte.** A água suave corre pelo compositor e um
/// dab suave composto EM CASA fica ao lado, a `folga` px: os planos do compositor não o têm, e o laço dele lê a
/// vizinhança da caixa. A tela copia os RGBA só onde os seus planos na janela de leitura são os que ele leu.
#[test]
fn os_rgba_do_compositor_com_smooth_edges_sao_a_casa_ao_byte() {
    let mut usadas = 0;
    for folga in [10.0, 16.0, 22.0, 30.0] {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
        t.set_paint_tool_mode("wetpaint");
        forca_no_teste(Some(true));
        let wd = "Watercolor/Fluid/Wet Drip.ph2d-brush.png";
        traco(&mut t, wd, [230, 200, 30], 60.0, 1.0, false);
        quadros(&mut t, 4);
        // O dab suave, composto em casa, logo abaixo da água que ainda corre.
        t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
            .expect("o preset");
        t.set_wet_oleo_real_ligado(false);
        t.set_brush_size_px(10.0);
        t.set_brush_color_srgb8([200, 40, 40]);
        t.set_wet_smooth_edges(1.0);
        let y = 60.0 + folga;
        let _ = t.on_canvas_pointer(ponto(60.0, y, PointerPhase::Down));
        for k in 1..=8 {
            let _ = t.on_canvas_pointer(ponto(60.0 + 10.0 * k as f32, y, PointerPhase::Move));
        }
        let _ = t.on_canvas_pointer(ponto(140.0, y, PointerPhase::Up));
        quadros(&mut t, 20);
        assenta(&mut t);
        usadas += sess(&t).otica.rgba_usadas;
        confere_com_casa(&mut t, &format!("Smooth Edges 1, dab de casa a {folga} px"));
    }
    assert!(usadas > 0, "premissa: a tela copiou RGBA do compositor com Smooth Edges");
}
