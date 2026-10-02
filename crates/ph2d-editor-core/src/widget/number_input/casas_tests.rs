//! Gates do [`super::numero_que_cabe`].

use super::*;
use crate::text_elide::em_todo_estilo;
use crate::widget::{NumberInput, TextInputState, paint_number_chip, paint_number_input};
use crate::zones::Rect;
use ph2d_tokens::{Theme, TypeToken};
use ph2d_vector::VectorScene;

fn largura_de(ts: &mut TextSystem, s: &str) -> f32 {
    ts.prefix_width_weighted(s, TypeToken::Sm.px(), FontWeight::MEDIUM)
}

/// A lei, por partes: arredonda (nunca trunca), uma casa de cada vez, guarda a unidade, e um zero
/// não leva sinal.
#[test]
fn perde_casas_arredondando_e_guarda_a_unidade() {
    let mut ts = TextSystem::without_system_fonts();
    let f = TypeToken::Sm.px();
    let w = largura_de(&mut ts, "12345.7");
    assert_eq!(numero_que_cabe(&mut ts, "12345.67", f, w), "12345.7");
    let w = largura_de(&mut ts, "12346");
    assert_eq!(numero_que_cabe(&mut ts, "12345.67", f, w), "12346");
    let w = largura_de(&mut ts, "1.2 px");
    assert_eq!(numero_que_cabe(&mut ts, "1.234 px", f, w), "1.2 px");
    let w = largura_de(&mut ts, "0");
    assert_eq!(numero_que_cabe(&mut ts, "-0.4", f, w), "0");
    // Cabe ⇒ intocado; sem decimal ⇒ intocado.
    assert_eq!(numero_que_cabe(&mut ts, "12345.67", f, 1.0e6), "12345.67");
    assert_eq!(numero_que_cabe(&mut ts, "123456789", f, 1.0), "123456789");
}

/// ⭐⭐ **No piso, o número sai INTEIRO e arredondado — nunca recortado nem `…`** — na caixa e no
/// chip do slider, em toda fonte, peso, tamanho de texto e nitidez. Régua: o que foi PINTADO.
#[test]
fn no_piso_o_numero_sai_inteiro_e_arredondado() {
    let v = 12345.67_f64;
    let acusados = em_todo_estilo(|ts| {
        let mut fora = Vec::new();
        let mut scene = VectorScene::new();
        let (_, medidos) = crate::text_elide::elisao::medindo(|| {
            let caixa = Rect::new(0.0, 0.0, crate::widget::number_input_min_w_px(), 22.0);
            paint_number_input(
                &NumberInput::new(ph2d_a11y::NodeId(1), "x", v),
                caixa,
                &mut scene,
                ts,
                Theme::default(),
            );
            let chip = Rect::new(0.0, 0.0, crate::widget::default_chip_w(), 22.0);
            paint_number_chip(
                chip,
                TextInputState::Normal,
                v,
                None,
                None,
                0,
                None,
                &mut scene,
                ts,
                Theme::default(),
            );
        });
        let numeros: Vec<_> = medidos
            .iter()
            .filter(|m| m.texto.starts_with("1234"))
            .collect();
        // Controlo de vacuidade: a caixa e o chip pintaram (o chip mede-o duas vezes: centrar e pintar).
        if numeros.len() < 2 {
            fora.push(format!(
                "esperava os 2 pintores, vieram {} números",
                numeros.len()
            ));
        }
        for m in numeros {
            let lido = m.pintado.parse::<f64>();
            if !m.coube() || !lido.as_ref().is_ok_and(|x| (x - v).abs() <= 0.5) {
                fora.push(format!(
                    "«{}» -> «{}» ({:.1} px)",
                    m.texto, m.pintado, m.largura
                ));
            }
        }
        fora
    });
    assert!(
        acusados.is_empty(),
        "números cortados no piso ({}):\n  {}",
        acusados.len(),
        acusados
            .iter()
            .take(30)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
