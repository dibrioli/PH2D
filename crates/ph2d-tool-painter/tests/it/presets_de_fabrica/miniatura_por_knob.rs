//! ⭐ **Cada knob da lei do grão chega à miniatura** (B07.6, GT-03): o alarme `a_miniatura_de_fabrica_e_a_que_o_motor_
//! pinta_hoje` só diz «mudou», e o par-a-par dele vê presets inteiros — um `softness`/`tip_depth`/`fill` sem leitor
//! passava. Sobre o MESMO Pastel (Height, os três ligados), cada knob nos dois extremos pinta miniaturas diferentes;
//! controlo: o mesmo valor pinta ao bit.

use super::*;
use crate::familias::csp_secos;
use serde_json::json;

/// O Pastel com `chave` da lei do grão em `v`.
fn com_lei(base: &Preset, chave: &str, v: &Value) -> Preset {
    let mut p = base.clone();
    p.parametros
        .brush
        .get_mut("texture")
        .and_then(|t| t.get_mut("lei"))
        .and_then(Value::as_object_mut)
        .expect("a lei do grão do Pastel")
        .insert(chave.to_owned(), v.clone());
    p
}

#[test]
fn cada_knob_da_lei_do_grao_muda_a_miniatura() {
    let r = csp_secos::MEIOS
        .iter()
        .find(|r| r.nome == "Pastel")
        .expect("a receita do Pastel");
    let base = preset_do_meio(r);
    let mini = |p: &Preset| miniatura_do_preset(p).rgba;
    for (chave, a, z) in [
        ("softness", json!(0.0), json!(1.0)),
        ("tip_depth", json!(false), json!(true)),
        ("fill", json!(0.0), json!(1.0)),
    ] {
        let pa = mini(&com_lei(&base, chave, &a));
        assert_eq!(
            pa,
            mini(&com_lei(&base, chave, &a)),
            "controlo: {chave} = {a} pinta duas miniaturas diferentes"
        );
        assert_ne!(
            pa,
            mini(&com_lei(&base, chave, &z)),
            "{chave} {a} × {z} pinta a mesma miniatura: o knob não tem leitor"
        );
    }
}
