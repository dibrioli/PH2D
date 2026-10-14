//! A cena `PH2D_MACRO_SMOKE=2` ensina o que acontece: a macro gravada por script coze FORMAS no palco,
//! nos dois modos, e o vento move-as.

use super::em_cena::MacroEmCena;
use super::gate_tests::retrato;
use super::smoke::grava_o_tutorial;
use super::tocar::Modo;
use crate::motion_mundo::MundoDaCena;

#[test]
fn a_macro_da_cena_coze_formas_e_o_vento_as_move() {
    const DONO: u64 = 7;
    let (bytes, d) = grava_o_tutorial(DONO).expect("a gravação arma");
    for modo in [Modo::Fiel, Modo::Explicativo] {
        let mut m = MacroEmCena::de(&bytes, modo)
            .expect("lê-se")
            .expect("tem trilha do Motion");
        let mundo = MundoDaCena::default();
        let mut vistos = Vec::new();
        for f in [0.3, 0.6, 0.8, 1.0] {
            let r = m.segue(d * f, &mundo, None);
            eprintln!(
                "{modo:?} t={:.2} erro={r:?} sinks={} inst={} vec={}",
                d * f,
                m.palco().sinks.len(),
                m.palco().pump.instances.len(),
                m.palco().pump.vector_instances.len()
            );
            vistos.push(retrato(m.palco()));
        }
        assert!(m.erro().is_none(), "{modo:?}: {:?}", m.erro());
        assert!(vistos[3].len() > 1, "{modo:?}: o palco cozeu VAZIO");
        // A grelha 6×10 da cena: os params chegaram (sem eles, a de origem dá 3×3).
        assert_eq!(m.palco().pump.vector_instances.len(), 60, "{modo:?}");
        assert_ne!(vistos[2], vistos[3], "{modo:?}: o vento não move as formas");
    }
}
