//! **Os ESTUDOS da família** (J4, 3.ª ronda): um preset de estudo = o ficheiro de um pincel de fábrica (`base`) com o que
//! a cura muda (volumes, knobs do motor, campos do pincel), escrito FORA da fábrica para a bancada o ler pelo caminho
//! (`exame.sh <caminho>`). A tabela da largura fica a da base (uma cura de pigmento/água não a mexe a ±1 px).
//!
//! ```text
//! PH2D_EXAME_DIR=/tmp/ph2d_exame_aguarela3 PH2D_ESTUDO_JSON=<ficheiro> \
//!   cargo test -p ph2d-tool-painter --profile smoke --test it escreve_os_estudos_da_aguarela -- --ignored --nocapture
//! ```
//! O JSON: `[{"nome": "Mop", "base": "Wet Wash", "grupo": "Fluid", "volumes": {"water": 1.0}, "engine": {"strokeFilm": 1},
//! "brush": {"hardness": 0.9}}, …]` → `$PH2D_EXAME_DIR/estudo/<nome>.ph2d-brush.png`.

use crate::exame_bancada::{Fonte, pinceis};
use serde_json::Value;

#[test]
#[ignore = "bancada: escreve os presets de estudo (PH2D_EXAME_DIR, PH2D_ESTUDO_JSON)"]
fn escreve_os_estudos_da_aguarela() {
    let dir = std::path::PathBuf::from(std::env::var("PH2D_EXAME_DIR").expect("PH2D_EXAME_DIR"))
        .join("estudo");
    let f = std::env::var("PH2D_ESTUDO_JSON").expect("PH2D_ESTUDO_JSON");
    let lista: Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
    for e in lista.as_array().unwrap() {
        // `"receita": "<nome>"` = uma receita da família (`WET`/`EM_ESTUDO`) feita como o gerador a faz, com a tabela da
        // largura MEDIDA (a de um ficheiro base mente quando a dureza muda: o Mop a 40 px pintava 62).
        let mut p = if let Some(nome) = e["receita"].as_str() {
            // J5: e as do guache e do óleo (o agente molhado examina-as com a lei molhado × molhado).
            let r = super::WET
                .iter()
                .chain(super::EM_ESTUDO)
                .chain(crate::familias::guache::WET)
                .chain(crate::familias::guache::EM_ESTUDO)
                .chain(crate::familias::oleo_acrilica::EM_ESTUDO)
                .find(|r| r.nome == nome)
                .expect("receita da família");
            let mut p = ph2d_brush_presets::Preset {
                parametros: crate::presets_de_fabrica::parametros(r),
                origem: ph2d_brush_presets::Origem::Fabrica,
                miniatura: ph2d_brush_presets::Miniatura {
                    largura: 1,
                    altura: 1,
                    rgba: vec![255; 4],
                },
                texturas: Default::default(),
            };
            p.parametros.width_table = crate::presets_de_fabrica::tabela_de_largura(&p);
            p
        } else {
            let base = e["base"].as_str().unwrap();
            let Fonte::Preset { preset, .. } = pinceis(base).remove(0).fonte else {
                unreachable!()
            };
            *preset
        };
        p.parametros.name = e["nome"].as_str().unwrap().to_owned();
        if let Some(g) = e["grupo"].as_str() {
            p.parametros.group = g.to_owned();
        }
        for (k, v) in e["volumes"].as_object().into_iter().flatten() {
            let v = v.as_f64();
            match k.as_str() {
                "water" => p.parametros.volumes.water = v,
                "opacity" => p.parametros.volumes.opacity = v,
                "size_px" => p.parametros.volumes.size_px = v,
                outro => panic!("volume {outro}?"),
            }
        }
        for (k, v) in e["engine"].as_object().into_iter().flatten() {
            assert!(
                ph2d_wet_paint::tuning::KNOB_DEFS.iter().any(|d| d.key == k),
                "knob {k}?"
            );
            p.parametros.engine.insert(k.clone(), v.as_f64().unwrap());
        }
        for (k, v) in e["brush"].as_object().into_iter().flatten() {
            assert!(
                ph2d_tool_painter::CAMPOS_DO_PINCEL.contains(&k.as_str()),
                "campo {k}?"
            );
            p.parametros.brush.insert(k.clone(), v.clone());
        }
        let rel = format!("{}.ph2d-brush.png", p.parametros.name);
        let caminho = ph2d_brush_presets::gravar(&dir, &rel, &p).unwrap();
        println!("[estudo] {}", caminho.display());
    }
}
