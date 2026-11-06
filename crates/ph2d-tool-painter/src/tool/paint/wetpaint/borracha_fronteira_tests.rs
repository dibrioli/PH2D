//! **A fronteira tinta viva × tinta morta sob a Borracha** (relato do dono 06/11): a orla em degraus, com o papel a
//! aparecer. Sonda: `PH2D_SONDA_DIR=<pasta> cargo test --profile smoke -p ph2d-tool-painter sonda_fronteira -- --ignored
//! --nocapture`.

use super::agua_sobre_oleo_tests::{PAPEL, ponteiro, preset, relato};
use super::borracha_defeitos_tests::{Caminho, imagem, passada, quadros};
use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{PanelEvent, PointerPhase};

/// A lei de antes: célula com tinta corta tudo, sem tinta segue a gradual, e o enterro não guarda a orla (o controlo).
pub(super) mod com_degrau {
    use std::cell::Cell;
    thread_local! { static V: Cell<bool> = const { Cell::new(false) }; }
    pub(in crate::tool::paint) fn get() -> bool {
        V.with(Cell::get)
    }
    pub(super) fn set(v: bool) {
        V.with(|c| c.set(v));
    }
}

const RELIEF: &str = "Eraser/Relief/Relief Eraser.ph2d-brush.png";
const W: usize = 320;
const H: usize = 200;
const AZUL: [u8; 3] = [40, 90, 220];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Baixo {
    /// O Dabber sem sessão: um arquivo antigo.
    Morta,
    /// Digital por cima do óleo, e a volta ao Wet: o enterro.
    Enterrada,
    /// O mesmo, com o Digital de outra cor.
    EnterradaVermelha,
}

fn traco(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(ponteiro(40.0, y, PointerPhase::Down));
    for k in 1..=60 {
        t.on_canvas_pointer(ponteiro(40.0 + 4.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(280.0, y, PointerPhase::Up));
    t.ticks_em_blocos(3);
}

fn dabber(t: &mut PainterTool) {
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8(AZUL);
}

/// A cena do dono: em cima o traço morto ou enterrado (`y = 75`), em baixo o Dabber vivo (`y = 95`), a fronteira no meio.
fn cena(baixo: Baixo, ratio: f64) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_wet_grid_ratio(ratio);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    dabber(&mut t);
    match baixo {
        Baixo::Morta => {
            traco(&mut t, 75.0);
            t.ticks_em_blocos(60);
            t.wetpaint_end_session();
            t.wet_esquece_as_adormecidas();
            dabber(&mut t);
            traco(&mut t, 95.0);
            t.ticks_em_blocos(60);
        }
        Baixo::Enterrada | Baixo::EnterradaVermelha => {
            traco(&mut t, 95.0);
            t.ticks_em_blocos(60);
            t.set_paint_media(PaintMedia::Digital);
            t.set_brush_size_px(30.0);
            t.set_brush_color_srgb8(if baixo == Baixo::Enterrada {
                AZUL
            } else {
                [220, 30, 30]
            });
            traco(&mut t, 75.0);
            t.set_paint_media(PaintMedia::WetPaint);
            t.ticks_em_blocos(3);
        }
    }
    t.preset_aplicar(&preset(RELIEF)).expect("o meio");
    t.set_brush_size_px(60.0);
    t.ticks_em_blocos(3);
    t
}

/// A cobertura da tinta viva, por célula alargada a píxel (0 = sem pigmento).
fn cobertura(t: &PainterTool) -> Vec<u8> {
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let q = usize::from(s.ratio);
    let (gw, gh) = s.grid;
    (0..W * H)
        .map(|i| {
            let c = ((i / W) / q).min(gh - 1) * gw + ((i % W) / q).min(gw - 1);
            s.planos.pigment[c * 4 + 3]
        })
        .collect()
}

fn salto(p: &[u8], i: usize, j: usize) -> i32 {
    (0..3)
        .map(|k| (i32::from(p[i * 4 + k]) - i32::from(p[j * 4 + k])).abs())
        .max()
        .unwrap_or(0)
}

/// `(p90, p99, máximo)` dos saltos de cor entre vizinhos da imagem mostrada, sobre o papel.
#[derive(Debug, Default, Clone, Copy)]
struct Aspereza {
    p90: i32,
    p99: i32,
    max: i32,
}

fn resumo(mut v: Vec<i32>) -> Aspereza {
    v.sort_unstable();
    if v.is_empty() {
        return Aspereza::default();
    }
    let n = v.len() - 1;
    Aspereza {
        p90: v[n * 90 / 100],
        p99: v[n * 99 / 100],
        max: v[n],
    }
}

/// A caixa apagada: as passadas passam-na toda, com o papel ainda de fora em baixo.
const XS: std::ops::Range<usize> = 110..210;
const YS: std::ops::Range<usize> = 60..125;

/// A aspereza nos pares de vizinhos da FRONTEIRA (um píxel com tinta viva, outro sem) e longe dela, de cada lado. `viva`
/// é a cobertura de antes da Borracha.
fn aspereza(p: &[u8], viva: &[u8]) -> (Aspereza, Aspereza, Aspereza) {
    let (mut f, mut m, mut v) = (Vec::new(), Vec::new(), Vec::new());
    let longe = |x: usize, y: usize, vivo: bool| {
        (y - 4..y + 5).all(|yy| (x - 4..x + 5).all(|xx| (viva[yy * W + xx] > 0) == vivo))
    };
    for y in YS {
        for x in XS {
            let i = y * W + x;
            let vi = viva[i] > 0;
            for j in [i + 1, i + W] {
                let dif = salto(p, i, j);
                if vi != (viva[j] > 0) {
                    f.push(dif);
                } else if longe(x, y, vi) {
                    if vi { &mut v } else { &mut m }.push(dif);
                }
            }
        }
    }
    (resumo(f), resumo(m), resumo(v))
}

/// Os píxeis onde o papel aparece na fronteira: a tinta viva acaba e ainda há tinta dos dois lados, por cima do fim do
/// traço vivo.
fn furos(p: &[u8], ate_y: usize) -> usize {
    YS.filter(|y| *y < ate_y)
        .flat_map(|y| XS.map(move |x| (y * W + x) * 4))
        .filter(|&o| (0..3).all(|k| p[o + k].abs_diff(PAPEL[k]) < 40))
        .count()
}

/// Até onde a tinta de baixo cobre o traço vivo na cena: abaixo dela o papel à volta é legítimo.
fn ate_y(baixo: Baixo) -> usize {
    if baixo == Baixo::Morta { 96 } else { 104 }
}

/// As passadas do dono: para cá e para lá sobre os dois traços, a fronteira no meio.
fn apaga_a_fronteira(t: &mut PainterTool, n: usize) {
    for k in 0..n {
        let y = 70.0 + (k % 3) as f32 * 15.0;
        let (a, b) = if k % 2 == 0 {
            (100.0, 220.0)
        } else {
            (220.0, 100.0)
        };
        passada(t, Caminho::Fixo, (a, y), (b, y));
    }
}

fn grava_ppm(nome: &str, p: &[u8]) {
    let Ok(dir) = std::env::var("PH2D_SONDA_DIR") else {
        return;
    };
    let mut v = format!("P6\n{W} {H}\n255\n").into_bytes();
    for px in p.chunks(4) {
        v.extend_from_slice(&px[..3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), v).unwrap();
}

/// A medida de uma cena depois de `n` passadas, na lei com ou sem degrau.
fn mede(baixo: Baixo, ratio: f64, degrau: bool, n: usize) -> (Aspereza, usize, Vec<u8>) {
    com_degrau::set(degrau);
    let mut t = cena(baixo, ratio);
    let viva = cobertura(&t);
    apaga_a_fronteira(&mut t, n);
    let r = imagem(&mut t);
    com_degrau::set(false);
    (aspereza(&r, &viva).0, furos(&r, ate_y(baixo)), r)
}

/// ⭐ A cena do dono, tinta morta ou enterrada e o Dabber vivo: a passada da Borracha pela fronteira não destapa o papel.
#[test]
fn a_fronteira_viva_morta_nao_destapa_o_papel() {
    for (baixo, ratio) in [
        (Baixo::Morta, 1.0),
        (Baixo::Morta, 4.0),
        (Baixo::Enterrada, 1.0),
        (Baixo::Enterrada, 4.0),
    ] {
        let (_, antes, _) = mede(baixo, ratio, true, 1);
        let (_, depois, _) = mede(baixo, ratio, false, 1);
        relato(format!(
            "{baixo:?} q={ratio}: papel na fronteira, {antes} px com degrau, {depois} sem"
        ));
        assert!(antes >= 20, "{baixo:?} q={ratio}: CONTROLO {antes}");
        assert!(
            depois * 4 <= antes,
            "{baixo:?} q={ratio}: {depois} px de papel, contra {antes} com o degrau"
        );
    }
}

/// ⭐ O salto de cor na fronteira, com a tinta morta: a metade do da lei com degrau e perto do de dentro da tinta viva.
#[test]
fn a_fronteira_viva_morta_e_macia() {
    for ratio in [1.0, 4.0] {
        let (com, _, _) = mede(Baixo::Morta, ratio, true, 1);
        let (sem, _, _) = mede(Baixo::Morta, ratio, false, 1);
        relato(format!("q={ratio}: salto na fronteira {com:?} → {sem:?}"));
        assert!(com.p90 >= 55, "q={ratio}: CONTROLO {com:?}");
        assert!(sem.p90 * 2 <= com.p90, "q={ratio}: {sem:?} contra {com:?}");
        assert!(
            sem.p99 < com.p99 && sem.max < com.max,
            "q={ratio}: {sem:?} contra {com:?}"
        );
    }
}

/// Tinta morta longe da viva, com a sessão viva: nenhum píxel difere da lei de antes. O caminho sem sessão escreve a mesma
/// lei com corte 0; a prova dele contra o commit de antes está no handoff.
#[test]
fn a_morta_longe_da_viva_nao_muda() {
    let r = |degrau: bool| {
        com_degrau::set(degrau);
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        dabber(&mut t);
        traco(&mut t, 75.0);
        t.ticks_em_blocos(60);
        t.wetpaint_end_session();
        t.wet_esquece_as_adormecidas();
        dabber(&mut t);
        traco(&mut t, 170.0);
        t.ticks_em_blocos(60);
        t.preset_aplicar(&preset(RELIEF)).expect("o meio");
        t.set_brush_size_px(30.0);
        t.ticks_em_blocos(3);
        let antes = imagem(&mut t);
        for _ in 0..2 {
            passada(&mut t, Caminho::Fixo, (100.0, 75.0), (220.0, 75.0));
        }
        com_degrau::set(false);
        let r = imagem(&mut t);
        assert_ne!(r, antes, "CONTROLO: a passada apaga");
        r
    };
    let (a, b) = (r(true), r(false));
    let n = a.chunks(4).zip(b.chunks(4)).filter(|(p, q)| p != q).count();
    assert_eq!(n, 0, "{n} px");
}

/// No caminho do app (worker, compositor, compõe por quadro) a passada também não destapa o papel.
#[test]
fn a_fronteira_no_caminho_do_app() {
    let mede = |degrau: bool| {
        com_degrau::set(degrau);
        let c = Caminho::App;
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_compor_por_quadro(true);
        t.handle_panel_event(PanelEvent::SelectOption(
            crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
            format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
        ));
        for (y, acaba) in [(75.0, true), (95.0, false)] {
            dabber(&mut t);
            t.on_canvas_pointer(ponteiro(40.0, y, PointerPhase::Down));
            for k in 1..=60 {
                t.on_canvas_pointer(ponteiro(40.0 + 4.0 * k as f32, y, PointerPhase::Move));
                quadros(&mut t, c, 1);
            }
            t.on_canvas_pointer(ponteiro(280.0, y, PointerPhase::Up));
            quadros(&mut t, c, 40);
            if acaba {
                t.wetpaint_end_session();
                t.wet_esquece_as_adormecidas();
            }
        }
        t.preset_aplicar(&preset(RELIEF)).expect("o meio");
        t.set_brush_size_px(60.0);
        quadros(&mut t, c, 3);
        passada(&mut t, c, (100.0, 70.0), (220.0, 70.0));
        com_degrau::set(false);
        furos(&imagem(&mut t), ate_y(Baixo::Morta))
    };
    let (antes, depois) = (mede(true), mede(false));
    relato(format!(
        "caminho do app: papel {antes} px com degrau, {depois} sem"
    ));
    assert!(antes >= 20, "CONTROLO {antes}");
    assert!(depois * 4 <= antes, "{depois} px, contra {antes}");
}

/// O undo devolve a tela e a `base` da fronteira ao bit.
#[test]
fn desfazer_na_fronteira_devolve_tudo() {
    for baixo in [Baixo::Morta, Baixo::Enterrada] {
        let mut t = cena(baixo, 1.0);
        let antes = imagem(&mut t);
        let base = t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessão")
            .base
            .to_vec();
        apaga_a_fronteira(&mut t, 1);
        let meio = imagem(&mut t);
        assert_ne!(meio, antes, "CONTROLO: a passada apaga");
        assert!(t.undo_last(), "o undo aplica-se");
        assert_eq!(imagem(&mut t), antes, "{baixo:?}: a tela");
        if let Some(s) = t.paint.wetpaint.session.as_ref() {
            assert_eq!(*s.base, base, "{baixo:?}: a base");
        }
    }
}

#[test]
#[ignore = "sonda: corre por nome com --ignored"]
fn sonda_fronteira() {
    for baixo in [Baixo::Morta, Baixo::Enterrada, Baixo::EnterradaVermelha] {
        for ratio in [1.0, 2.0, 4.0] {
            for degrau in [true, false] {
                com_degrau::set(degrau);
                grava_ppm(
                    &format!("{baixo:?}_{ratio}_{degrau}_0"),
                    &imagem(&mut cena(baixo, ratio)),
                );
                com_degrau::set(false);
                for n in [1, 3, 6] {
                    let (f, fu, r) = mede(baixo, ratio, degrau, n);
                    grava_ppm(&format!("{baixo:?}_{ratio}_{degrau}_{n}"), &r);
                    relato(format!(
                        "{baixo:?} q={ratio} degrau={degrau} n={n}: {f:?} papel {fu}"
                    ));
                }
            }
        }
    }
}
