//! Sonda (ignorada) da R2-F (Blender Grainy, L-2): o preset de estudo `pranchas/mistura/presets/blender_grainy` e as
//! variantes do grão (medidas sobre o preset J6 `0319d50dcf`; o preset JÁ é a v7) na cena `sobre`, gravadas em PPM em `SONDA_R2F_DIR` (relatório `docs/Painter/Pinceis/R2_F_blender_grainy.md`).

use super::sobre_a_escala;
use ph2d_brush_presets::Parametros;
use serde_json::json;

type Muta = fn(&mut Parametros);

fn lado(p: &mut Parametros, l: f64) {
    if let Some(t) = p.brush.get_mut("texture") {
        t["size"] = json!([l, l]);
    }
}

const VARIANTES: &[(&str, Muta)] = &[
    ("v0", |_| {}),
    ("v1", |p| lado(p, 6.0)),
    ("v2", |p| lado(p, 3.0)),
    ("v3", |p| {
        p.brush
            .insert("texture".to_owned(), json!({"kind": "None"}));
        p.engine.insert("paperGate".to_owned(), 0.8);
    }),
    ("v4", |p| {
        lado(p, 6.0);
        p.engine.insert("paperGate".to_owned(), 0.7);
    }),
    // Ronda 2: o grão fino de v1 com o toque espalhado (os lados da coluna rasgam-se como no Textured de fábrica).
    ("v5", |p| {
        lado(p, 6.0);
        p.brush.insert("jitter".to_owned(), json!(0.35));
    }),
    ("v6", |p| {
        lado(p, 6.0);
        p.brush.insert("jitter".to_owned(), json!(0.2));
        p.engine.insert("paperGate".to_owned(), 0.55);
    }),
    ("v7", |p| {
        lado(p, 4.0);
        p.brush.insert("jitter".to_owned(), json!(0.35));
    }),
];

#[test]
#[ignore = "sonda: grava a cena sobre de cada variante"]
fn sonda_r2f_blender_grainy() {
    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let bytes = std::fs::read(format!(
        "{raiz}/docs/Painter/Pinceis/pranchas/mistura/presets/blender_grainy.ph2d-brush.png"
    ))
    .unwrap();
    let base = ph2d_brush_presets::ler(&bytes).unwrap();
    // `SONDA_R2F_GRAVA=<variante>`: a variante escolhida grava-se no preset de estudo (a miniatura fica).
    if let Ok(v) = std::env::var("SONDA_R2F_GRAVA") {
        let (_, muta) = VARIANTES.iter().find(|x| x.0 == v).expect("variante");
        let mut p = base.clone();
        muta(&mut p.parametros);
        std::fs::write(
            format!(
                "{raiz}/docs/Painter/Pinceis/pranchas/mistura/presets/blender_grainy.ph2d-brush.png"
            ),
            ph2d_brush_presets::escrever(&p).unwrap(),
        )
        .unwrap();
        return;
    }
    let dir = std::env::var("SONDA_R2F_DIR").unwrap_or_else(|_| "/tmp/claude-1000/r2f".into());
    let so = std::env::var("SONDA_R2F").unwrap_or_default();
    std::fs::create_dir_all(&dir).unwrap();
    std::thread::scope(|s| {
        for &(nome, muta) in VARIANTES
            .iter()
            .filter(|v| so.is_empty() || so.split(',').any(|x| x == v.0))
        {
            let mut p = base.clone();
            muta(&mut p.parametros);
            let dir = dir.clone();
            s.spawn(move || {
                let px = sobre_a_escala("Blender Grainy", Some(p), 1.0);
                let w = 1000usize;
                let h = px.len() / 4 / w;
                let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
                for c in px.as_chunks::<4>().0 {
                    out.extend_from_slice(&c[..3]);
                }
                std::fs::write(format!("{dir}/{nome}.ppm"), out).unwrap();
                println!("[sonda r2f] {nome}: {w}×{h}");
            });
        }
    });
}
