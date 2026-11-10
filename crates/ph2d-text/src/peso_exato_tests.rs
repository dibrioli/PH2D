use super::{ativo, com_peso_exato};
use crate::system::effective_weight;
use crate::{FontWeight, PositionedLayoutItem, TextSystem, run_coord_bits, set_active_text_style};
use ph2d_tokens::{TextRendering, UiTextStyle, UiWeight, crisp_weight_boost_for};

const PX: f32 = 11.0;
const CHP: TextRendering = TextRendering::CrispHeavyPlus;

/// O eixo variável (o `wght` incluído) de cada corrida de `texto` pintado pela porta de pintar.
fn eixo(ts: &mut TextSystem, rendering: TextRendering, w: FontWeight) -> Vec<Vec<i16>> {
    let l = ts.layout_for_rendering("Hexagon", PX, f32::INFINITY, w, rendering);
    let mut out = Vec::new();
    for line in l.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(g) = item {
                out.push(run_coord_bits(g.run()));
            }
        }
    }
    out
}

#[test]
fn dentro_do_ambito_o_preset_nao_reforca_o_peso() {
    let boost = crisp_weight_boost_for(CHP.params(), PX);
    assert!(boost > 0, "controlo: o CrispHeavyPlus reforça a {PX} px");
    let n = FontWeight::NORMAL;
    let fora = effective_weight(n, PX, CHP).value();
    assert_eq!(
        fora,
        n.value() + f32::from(boost),
        "fora do âmbito o reforço soma-se"
    );
    let dentro = com_peso_exato(|| effective_weight(n, PX, CHP).value());
    assert_eq!(dentro, n.value(), "dentro do âmbito o peso é o pedido");
    assert_eq!(
        effective_weight(n, PX, CHP).value(),
        fora,
        "à saída volta o reforço"
    );
}

#[test]
fn o_ambito_repoe_o_estado_aninhado_e_em_panico() {
    assert!(!ativo());
    com_peso_exato(|| {
        com_peso_exato(|| assert!(ativo()));
        assert!(ativo(), "o interior a sair não fecha o exterior");
    });
    assert!(!ativo());
    let r = std::panic::catch_unwind(|| com_peso_exato(|| panic!("a meio")));
    assert!(r.is_err());
    assert!(!ativo(), "o pânico dentro do âmbito repõe o estado");
}

#[test]
fn o_desvio_do_estilo_vale_dentro_do_ambito() {
    let mut ts = TextSystem::without_system_fonts();
    let n = FontWeight::NORMAL;
    let normal = eixo(&mut ts, TextRendering::Default, n);
    set_active_text_style(UiTextStyle {
        weight: UiWeight::Light,
        ..UiTextStyle::default()
    });
    let leve = eixo(&mut ts, TextRendering::Default, n);
    let leve_chp_fora = eixo(&mut ts, CHP, n);
    let leve_chp_dentro = com_peso_exato(|| eixo(&mut ts, CHP, n));
    set_active_text_style(UiTextStyle::default());
    assert!(
        !normal.is_empty() && normal != leve,
        "controlo: o Light muda o eixo"
    );
    assert_ne!(
        leve_chp_fora, leve,
        "controlo: fora do âmbito o preset reforça"
    );
    assert_eq!(
        leve_chp_dentro, leve,
        "dentro: o Light vale, o reforço do preset não"
    );
}
