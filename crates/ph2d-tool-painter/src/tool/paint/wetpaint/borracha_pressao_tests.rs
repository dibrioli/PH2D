//! **A Borracha com pouca pressão também chega ao papel, em mais passadas** (dono 06/11).

use super::adormecida_tests::traco_de_outro_meio;
use super::agua_sobre_oleo_tests::{PAPEL, W, com_oleo, grava, mostrada, ponteiro, relato};
use super::borracha_de_fabrica_tests::apaga;
use super::sonda_borracha_morta::morta;
use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// O piso da lei, à mão: as candidatas no mesmo processo.
pub(super) mod piso {
    use std::cell::Cell;
    thread_local! { static V: Cell<Option<f32>> = const { Cell::new(None) }; }
    pub(in crate::tool::paint) fn get() -> Option<f32> {
        V.with(Cell::get)
    }
    pub(super) fn set(v: Option<f32>) {
        V.with(|c| c.set(v));
    }
}

/// Uma passada sobre o rasto de sempre, com a pressão `p` do princípio ao fim.
fn passada(t: &mut PainterTool, p: f32) {
    let ponto = |k: usize, phase| {
        let mut q = ponteiro(120.0 + 0.7 * k as f32, 60.0 + 2.0 * k as f32, phase);
        q.pressure = p;
        q
    };
    t.on_canvas_pointer(ponto(0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponto(k, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(30, PointerPhase::Up));
    t.ticks_em_blocos(2);
}

/// A tinta que resta no meio do rasto: a distância média à cor do papel.
fn resto(p: &[u8]) -> f32 {
    let mut soma = 0.0;
    for k in 10..=20usize {
        let (x, y) = (120 + k * 7 / 10, 60 + 2 * k);
        let i = (y * W + x) * 4;
        soma += (0..3)
            .map(|c| f32::from(p[i + c].abs_diff(PAPEL[c])))
            .fold(0.0, f32::max);
    }
    soma / 11.0
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Cena {
    Morta,
    TracoPorCima,
    SoOleo,
}

fn cena(c: Cena) -> PainterTool {
    match c {
        Cena::Morta => {
            let mut t = morta();
            t.set_paint_tool_mode("eraser");
            t
        }
        Cena::SoOleo => {
            let mut t = com_oleo();
            t.set_paint_tool_mode("eraser");
            t
        }
        Cena::TracoPorCima => {
            let mut t = com_oleo();
            t.set_paint_media(PaintMedia::Digital);
            traco_de_outro_meio(&mut t, (130.0, 60.0), (130.0, 120.0));
            t.set_paint_media(PaintMedia::WetPaint);
            t.ticks_em_blocos(10);
            t.set_paint_tool_mode("eraser");
            t
        }
    }
}

/// ⭐ Decisão do dono: com a mão leve a tinta morta também chega ao papel, aos poucos, e não queima a parede.
#[test]
fn a_mao_leve_chega_ao_papel_em_mais_passadas() {
    for c in [Cena::Morta, Cena::TracoPorCima] {
        let mut t = cena(c);
        passada(&mut t, 0.35);
        let uma = resto(&mostrada(&mut t));
        assert!(uma > 70.0, "{c:?}: uma passada leve deixa a tinta ({uma})");
        for _ in 1..30 {
            passada(&mut t, 0.35);
        }
        let m = mostrada(&mut t);
        let r = resto(&m);
        assert!(
            r < 12.0,
            "{c:?}: 30 passadas leves não chegaram ao papel ({r})"
        );
    }
}

/// A pressão plena é a lei aprovada: o piso não muda um píxel.
#[test]
fn a_pressao_plena_nao_muda() {
    for c in [Cena::Morta, Cena::TracoPorCima] {
        let imagem = |candidata| {
            piso::set(Some(candidata));
            let mut t = cena(c);
            passada(&mut t, 1.0);
            let m = mostrada(&mut t);
            piso::set(None);
            m
        };
        let (sem, com) = (imagem(0.0), imagem(0.3));
        let d = sem
            .chunks(4)
            .zip(com.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(d, 0, "{c:?}: a pressão plena mudou {d} px");
    }
}

/// Quantas passadas até o meio do rasto chegar perto do papel, por pressão e candidata.
#[test]
#[ignore = "sonda: corre por nome com --ignored --nocapture"]
fn sonda_passadas_ate_o_papel() {
    let limite = 60;
    for c in [Cena::Morta, Cena::TracoPorCima, Cena::SoOleo] {
        for candidata in [0.0f32, 0.15, 0.3] {
            piso::set(Some(candidata));
            let mut linha = format!("{c:?} piso {candidata}:");
            for p in [0.35f32, 0.6, 1.0] {
                let mut t = cena(c);
                let mut n = None;
                let mut uma = 0.0;
                for k in 1..=limite {
                    passada(&mut t, p);
                    let r = resto(&mostrada(&mut t));
                    if k == 1 {
                        uma = r;
                    }
                    if r < 8.0 {
                        n = Some(k);
                        break;
                    }
                }
                linha += &format!(
                    "  p{p}: {} ({uma:.0})",
                    n.map_or(format!(">{limite}"), |k| k.to_string())
                );
            }
            relato(linha);
        }
    }
    piso::set(None);
}

/// As fotos: cada cena e candidata, com a mão leve (0,35) e com a rampa de pressão de sempre.
#[test]
#[ignore = "sonda: corre por nome com --ignored --nocapture"]
fn sonda_foto_pouca_pressao() {
    for c in [Cena::Morta, Cena::TracoPorCima, Cena::SoOleo] {
        for (nome, candidata) in [("p0", 0.0f32), ("p15", 0.15), ("p30", 0.3)] {
            if matches!(c, Cena::SoOleo) && candidata > 0.0 {
                continue; // a lei da tinta viva não passa pelo piso
            }
            piso::set(Some(candidata));
            let mut t = cena(c);
            grava(&format!("{c:?}_{nome}_00"), &mostrada(&mut t));
            let mut azuis = Vec::new();
            for k in 1..=30 {
                passada(&mut t, 0.35);
                if [1, 3, 6, 10, 20, 30].contains(&k) {
                    let m = mostrada(&mut t);
                    grava(&format!("{c:?}_{nome}_{k:02}"), &m);
                    azuis.push((k, resto(&m) as u32));
                }
            }
            relato(format!(
                "{c:?} {nome}, mão leve (passada, distância ao papel): {azuis:?}"
            ));
            let mut r = cena(c);
            grava(&format!("{c:?}_{nome}_rampa"), &apaga(&mut r));
        }
    }
    piso::set(None);
}
