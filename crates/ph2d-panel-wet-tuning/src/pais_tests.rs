use super::*;
use crate::rows::rows;

fn pincel() -> BrushSettings {
    let mut b = ph2d_tool_painter::PainterTool::default().brush_settings();
    b.wetpaint = true;
    b
}

fn liga(b: &mut BrushSettings, pai: Pai, on: bool) {
    match pai {
        Pai::Knob(chave) => {
            let i = KNOB_DEFS.iter().position(|d| d.key == chave).expect(chave);
            b.wet_knobs.knobs[i] = if on { KNOB_DEFS[i].max.min(1.0) } else { 0.0 };
        }
        Pai::OleoReal => b.wet_oleo_real = on,
    }
}

/// ⭐ B35.3: um por pai — com ele a zero a filha some do Tuning; ligado (e os outros pais dela ligados) ela volta.
#[test]
fn linha_morta_nao_aparece_com_o_pai_a_zero() {
    for (filha, pai) in PAIS {
        let row = rows()
            .iter()
            .find(|r| r.key == filha)
            .unwrap_or_else(|| panic!("{filha}: não é linha do Tuning"));
        let mut b = pincel();
        for (f, p) in PAIS {
            if f == filha {
                liga(&mut b, p, true);
            }
        }
        assert!(row.visivel(&b), "{filha}: todos os pais ligados => visível");
        liga(&mut b, pai, false);
        assert!(!row.visivel(&b), "{filha}: {pai:?} desligado => escondida");
    }
}

/// O pai é um knob do registo (uma chave errada nunca liga ⇒ a linha sumia para sempre).
#[test]
fn todo_pai_knob_existe_no_registo() {
    for (filha, pai) in PAIS {
        assert!(KNOB_DEFS.iter().any(|d| d.key == filha), "{filha}");
        if let Pai::Knob(k) = pai {
            assert!(KNOB_DEFS.iter().any(|d| d.key == k), "{k}");
        }
    }
}
