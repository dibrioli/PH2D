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

/// `(spray_count, jitter, jitter_scale, particle, veu, veu_escala)` sobre o disco opaco.
type Gota = (&'static str, &'static str, &'static str, f64, f64, f64);

/// Variantes do J-3 Splatter (`aerografo::EM_ESTUDO[0]`) em gotas OPACAS redondas; `s0` = a receita de hoje.
const SPLATTER: &[(&str, Option<Gota>)] = &[
    ("s0", None),
    ("s1", Some(("4", "0.9", "0.9", 0.3, 0.15, 2.5))),
    ("s2", Some(("6", "1.2", "0.95", 0.2, 0.1, 4.0))),
    ("s3", Some(("3", "0.7", "0.8", 0.45, 0.0, 1.0))),
    // Ronda 2: a gota com TECTO na tela (`particle_max_px`), para a 200 px não virar disco gigante.
    ("s4", Some(("3", "0.7", "0.8", 0.45, 0.0, 1.0))),
    ("s5", Some(("4", "0.9", "0.9", 0.3, 0.15, 2.5))),
    // Volta 3: a MANCHA recortada de hoje (rodada e espelhada), opaca, com o véu e o tecto da s5.
    ("m1", Some(("4", "0.9", "0.9", 0.3, 0.15, 2.5))),
    ("m2", Some(("3", "0.7", "0.8", 0.45, 0.15, 2.0))),
    ("m3", Some(("6", "1.0", "0.9", 0.25, 0.2, 3.0))),
    // m4 = m3 com o tecto a 72 px (a mancha ocupa ~½ da caixa da imagem).
    ("m4", Some(("6", "1.0", "0.9", 0.25, 0.2, 3.0))),
];

#[test]
#[ignore = "sonda R2-F: o Splatter em gotas opacas (SONDA_R2F_DIR)"]
fn sonda_r2f_splatter() {
    use crate::exame_bancada::{Fonte, Pincel, bateria, pinta_cena};
    let dir = std::path::PathBuf::from(
        std::env::var("SONDA_R2F_DIR").unwrap_or_else(|_| "/tmp/claude-1000/r2f".into()),
    );
    std::fs::create_dir_all(&dir).unwrap();
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let so = std::env::var("SONDA_R2F").unwrap_or_default();
    let base = crate::presets_de_fabrica::preset_do_meio(&crate::familias::aerografo::MEIOS[0]);
    for &(nome, gota) in SPLATTER
        .iter()
        .filter(|v| so.is_empty() || so.split(',').any(|x| x == v.0))
    {
        let mut p = base.clone();
        if let Some((n, jit, js, particle, veu, veu_escala)) = gota {
            let b = &mut p.parametros.brush;
            let mancha = nome.starts_with('m');
            if !mancha {
                b.insert("shape".into(), json!({"kind": "None"}));
            }
            b.insert("spray_count".into(), serde_json::from_str(n).unwrap());
            b.insert("jitter".into(), serde_json::from_str(jit).unwrap());
            b.insert("jitter_scale".into(), serde_json::from_str(js).unwrap());
            b.insert(
                "dab_jitter".into(),
                json!({"across": 0.0, "angle_per_stroke": 0.0, "count": 0.6, "flip_x": if mancha { 0.5 } else { 0.0 }, "flip_y": 0.0,
                       "opacity": 0.0, "particle": particle, "roundness": 0.0, "veu": veu,
                       "veu_escala": veu_escala, "veu_opacidade": 1.0}),
            );
            if mancha || nome >= "s4" {
                b.get_mut("dab_jitter").unwrap()["particle_max_px"] =
                    json!(if nome == "m4" { 72.0 } else { 36.0 });
            }
            p.parametros.volumes.opacity = Some(1.0);
            if !mancha {
                p.texturas.clear();
            }
        }
        let pincel = Pincel {
            nome: nome.into(),
            fonte: Fonte::Preset {
                preset: Box::new(p),
                relativo: format!("r2f/{nome}.ph2d-brush.png"),
            },
        };
        for cn in ["gestos", "tamanhos"] {
            let cena = bat["cenas"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["nome"] == cn)
                .unwrap()
                .clone();
            let px = pinta_cena(&pincel, None, &cena, tela, papel, 0).1;
            let mut out = format!("P6\n{} {}\n255\n", tela[0], tela[1]).into_bytes();
            for c in px.as_chunks::<4>().0 {
                out.extend_from_slice(&c[..3]);
            }
            std::fs::write(dir.join(format!("{nome}_{cn}.ppm")), out).unwrap();
            println!("[r2f] {nome} {cn}");
        }
    }
}
