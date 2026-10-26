//! Sonda (ignorada) dos presets das ferramentas da água (N4, dono 25/10): cada candidato na cena `sobre`, contra a
//! foto do Rebelle (`fixtures/rebelle_bateria/{SMUDGE,BLEND}`, medidas por `exames/mistura_foto.py`).
//! `SONDA_AGUA=<texto>` corre só os candidatos cujo nome o contém.

use super::{Identidade, sobre_a_escala};
use ph2d_brush_presets::{Parametros, Preset};
use serde_json::json;

type Muta = fn(&mut Parametros);

fn smear(p: &mut Parametros, forca: f64) {
    p.paint_mode = Some("Smear".to_owned());
    p.engine.insert("smear".to_owned(), forca);
}

fn blend(p: &mut Parametros, forca: f64) {
    p.engine.insert("blendForce".to_owned(), forca);
}

fn pincel(p: &mut Parametros, campo: &str, v: serde_json::Value) {
    p.brush.insert(campo.to_owned(), v);
}

/// `(nome, escala, mutação sobre o Blender Soft de fábrica)`.
fn grao(p: &mut Parametros, kind: &str, lado: f64) {
    p.brush.insert(
        "texture".to_owned(),
        crate::receitas_de_fabrica::grao(kind, lado),
    );
}

fn mao(p: &mut Parametros, chaves: &[(&str, f64)]) {
    for (k, v) in chaves {
        p.engine.insert((*k).to_owned(), *v);
    }
}

/// `(nome, escala, mutação sobre o Blender Soft de fábrica)`.
const CANDIDATOS: &[(&str, f64, Muta)] = &[
    ("Blender Soft", 0.45, |_| {}),
    ("Blender Soft 2", 0.45, |p| grao(p, "Noise", 32.0)),
    ("Blender Soft 2 g64", 0.45, |p| grao(p, "Noise", 64.0)),
    ("Blender Soft 2 g128", 0.45, |p| grao(p, "Noise", 128.0)),
    ("Blender Noisy g64", 0.45, |p| {
        blend(p, 1.0);
        grao(p, "Noise", 64.0);
    }),
    ("Blender Noisy f.8 g64", 0.45, |p| {
        blend(p, 0.8);
        grao(p, "Noise", 64.0);
    }),
    ("Blender Noisy", 0.45, |p| {
        blend(p, 1.0);
        grao(p, "Noise", 16.0);
    }),
    ("Blender Textured", 0.45, |p| {
        blend(p, 1.0);
        pincel(p, "jitter", json!(0.35));
        pincel(p, "jitter_rotate", json!(0.5));
    }),
    ("Blender Dots", 0.45, |p| {
        blend(p, 1.0);
        pincel(p, "jitter", json!(0.125));
        pincel(p, "jitter_rotate", json!(0.5));
        grao(p, "Noise", 8.0);
    }),
    ("Blender Textured f.3", 0.45, |p| {
        blend(p, 0.3);
        pincel(p, "jitter", json!(0.35));
        pincel(p, "jitter_rotate", json!(0.5));
    }),
    ("Blender Dots f.3", 0.45, |p| {
        blend(p, 0.3);
        pincel(p, "jitter", json!(0.125));
        pincel(p, "jitter_rotate", json!(0.5));
    }),
    ("Blender Dots f.5", 0.45, |p| {
        blend(p, 0.5);
        pincel(p, "jitter", json!(0.125));
        pincel(p, "jitter_rotate", json!(0.5));
    }),
    ("Blender Hard", 0.45, |p| {
        blend(p, 0.6);
        mao(p, &[("pressureOpacity", 0.8), ("pressureFloor", 0.2)]);
    }),
    ("Smear Soft", 0.5, |p| {
        smear(p, 1.0);
        mao(p, &[("pressureSize", 0.0), ("pressureOpacity", 1.0)]);
    }),
    ("Smear Spikey", 0.5, |p| {
        smear(p, 1.0);
        mao(
            p,
            &[
                ("pressureSize", 1.0),
                ("pressureOpacity", 0.7),
                ("pressureFloor", 0.3),
            ],
        );
        grao(p, "Noise", 8.0);
    }),
    ("Smear Flat", 0.5, |p| {
        smear(p, 0.95);
        pincel(p, "dab_flatten", json!(0.3));
        pincel(p, "dab_angle_deg", json!(45));
        mao(p, &[("pressureSize", 0.3), ("pressureOpacity", 1.0)]);
    }),
    ("Smear Shattered", 0.5, |p| {
        smear(p, 1.0);
        pincel(p, "jitter", json!(0.25));
        pincel(p, "jitter_rotate", json!(1.0));
        grao(p, "Noise", 16.0);
    }),
    ("Smear Shattered s.6", 0.5, |p| {
        smear(p, 0.6);
        pincel(p, "jitter", json!(0.25));
        pincel(p, "jitter_rotate", json!(1.0));
    }),
    ("Smear Shattered s.4", 0.5, |p| {
        smear(p, 0.4);
        pincel(p, "jitter", json!(0.25));
        pincel(p, "jitter_rotate", json!(1.0));
    }),
    ("Smear Wet", 0.5, |p| {
        smear(p, 1.0);
        p.engine.insert("smearWater".to_owned(), 0.9);
    }),
];

#[test]
#[ignore = "sonda: imprime as identidades dos candidatos"]
fn sonda_ferramentas_da_agua() {
    let filtro = std::env::var("SONDA_AGUA").unwrap_or_default();
    let base: Preset = crate::midias_de_fabrica::de_fabrica("Blender Soft");
    let escolhidos: Vec<_> = CANDIDATOS
        .iter()
        .filter(|c| c.0.contains(filtro.as_str()))
        .collect();
    std::thread::scope(|s| {
        let hs: Vec<_> = escolhidos
            .iter()
            .map(|&&(nome, escala, muta)| {
                let mut p = base.clone();
                muta(&mut p.parametros);
                s.spawn(move || {
                    let px = sobre_a_escala("Blender Soft", Some(p), escala);
                    (nome, Identidade::de(&px, 1000))
                })
            })
            .collect();
        for h in hs {
            let (nome, i) = h.join().unwrap();
            println!("[sonda agua] {nome}: {i:?}");
        }
    });
}

#[test]
#[ignore = "sonda: o estado depois de aplicar o Blender Soft sobre o Ink Brush"]
fn sonda_estado_do_blend() {
    use ph2d_tool_painter::PainterTool;
    let mut t = PainterTool::default();
    t.preset_aplicar(&crate::midias_de_fabrica::de_fabrica("Ink Brush"))
        .unwrap();
    t.preset_aplicar(&crate::midias_de_fabrica::de_fabrica("Blender Soft"))
        .unwrap();
    println!("ESTADO {:#?}", t.brush_settings());
}
