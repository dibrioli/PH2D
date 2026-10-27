//! ⭐ **A geometria da cerda e da marca** (Óleo Real W3 K3b, `docs/Painter/Pinceis/W3_K3b_cerda.md`): as cerdas do
//! Shape (o tapete, uma lei em píxeis) crescem com o pincel pelo mesmo `(D ÷ ref)^γ` do grão `Tiled` (K2), e a ponta
//! desfiada com um grão preso à tela ESFARELA pelo grão em vez de abrir em fios. O neutro (`γ = 0`; sem grão `Tiled`)
//! é o gate da fábrica inteira, `presets_de_fabrica::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`.

use super::exame_bancada::{Fonte, pinceis, pinta_cena_com};
use super::textura_do_tamanho::{dabber, tamanho_do_grao, tinta};
use ph2d_brush_presets::Preset;
use ph2d_tool_painter::miniatura_do_preset;
use serde_json::json;

/// O Dabber de fábrica (as cerdas são o Shape Bristles do tapete) ao diâmetro `d`, com o expoente `gama`.
fn cerdas(d: f64, gama: Option<f64>) -> Preset {
    let mut p = dabber();
    p.parametros.volumes.size_px = Some(d);
    // Um tapete de pontas ESPARSAS (o do Dabber 3): o tamanho do grão lê as pontas, não o corpo cheio.
    for (k, v) in [("bristleCount", 400.0), ("bristleSize", 2.0)] {
        p.parametros.engine.insert(k.into(), v);
    }
    if let Some(g) = gama {
        p.parametros.engine.insert("textureSizeExp".into(), g);
        p.parametros.engine.insert("textureSizeRef".into(), 24.0);
    }
    p
}

/// As cerdas crescem com o pincel: de 24 para 96 px (×4) o tamanho do grão com `γ = 1` cresce bem mais que com o
/// tapete preso aos px (`γ = 0`). E `γ = 0` escrito é o preset sem a chave, ao byte.
#[test]
fn as_cerdas_crescem_com_o_pincel() {
    assert_eq!(
        miniatura_do_preset(&cerdas(96.0, Some(0.0))),
        miniatura_do_preset(&cerdas(96.0, None)),
        "γ = 0 tem de ser o tapete de sempre, ao byte"
    );
    let fixo = tamanho_do_grao(&cerdas(96.0, None)) / tamanho_do_grao(&cerdas(24.0, None));
    let segue =
        tamanho_do_grao(&cerdas(96.0, Some(1.0))) / tamanho_do_grao(&cerdas(24.0, Some(1.0)));
    assert!(
        segue > 1.5 * fixo,
        "o tamanho das cerdas 96 ÷ 24: γ = 1 {segue:.3}, γ = 0 {fixo:.3}"
    );
}

/// O Filbert de fábrica (ponta desfiada, `bristleTipRagged` 1,6) num traço recto de 120 px, com ou sem um grão
/// `Noise` preso à tela (Depth 0,1: o corpo quase não muda). Devolve a ANISOTROPIA das duas pontas: as transições
/// tinta↔papel ao longo de x ÷ ao longo de y (fios ao longo do traço ⇒ ≪ 1; grumos ⇒ ≈ 1).
fn anisotropia_das_pontas(com_grao: bool) -> f64 {
    let mut pincel = pinceis("Filbert").remove(0);
    if com_grao {
        let Fonte::Preset { preset, .. } = &mut pincel.fonte else {
            unreachable!("o Filbert é um preset");
        };
        let b = &mut preset.parametros.brush;
        b.insert(
            "texture".into(),
            json!({"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0],
                   "params": [1.0, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [60.0, 60.0],
                   "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}),
        );
        b.insert("grain_depth".into(), json!(0.1));
    }
    let (w, h) = (480u32, 220u32);
    let pts: Vec<_> = (0..=100)
        .map(|i| json!([90.0 + 3.0 * f64::from(i), 110.0, 1.0]))
        .collect();
    let cena = json!({"nome": "ponta", "ops": [
        {"op": "pincel", "quem": "preset"},
        {"op": "cor", "rgb": [23, 101, 179]},
        {"op": "traco", "size_px": 120, "pts": pts},
        {"op": "simula", "quadros": 30},
        {"op": "foto", "nome": "A"},
    ]});
    let (_, px) = pinta_cena_com(&pincel, None, &cena, [w, h], [250, 250, 250], 0, false);
    let m: Vec<bool> = px.chunks(4).map(|p| tinta(p) >= 0.25).collect();
    let (w, h) = (w as usize, h as usize);
    let colunas: Vec<usize> = (0..w)
        .filter(|&x| (0..h).filter(|&y| m[y * w + x]).count() > h / 20)
        .collect();
    let (x0, x1) = (colunas[0], *colunas.last().unwrap());
    let linhas: Vec<usize> = (0..h)
        .filter(|&y| (x0..=x1).filter(|&x| m[y * w + x]).count() > (x1 - x0) / 4)
        .collect();
    let (y0, y1) = (linhas[0], *linhas.last().unwrap());
    let largura = (y1 - y0 + 1) / 4;
    let (mut ex, mut ey) = (0usize, 0usize);
    for xs in [x0..x0 + largura, x1 + 1 - largura..x1 + 1] {
        for y in y0..y1 {
            for x in xs.clone() {
                ex += usize::from(x + 1 < w && m[y * w + x] != m[y * w + x + 1]);
                ey += usize::from(m[y * w + x] != m[(y + 1) * w + x]);
            }
        }
    }
    assert!(ey > 0, "as pontas não têm transições");
    ex as f64 / ey as f64
}

/// A ponta ESFARELA pelo grão (o Metallic Flat do Rebelle: o recto acaba em grumos, sem fios): com o grão `Tiled`
/// as pontas ficam isotrópicas (medido 1,02; o Rebelle lê 0,89–1,00 na régua da K3b), sem ele abrem em fios ao
/// longo do traço (medido 0,72).
#[test]
fn a_ponta_esfarela_pelo_grao() {
    let (fios, grumos) = (anisotropia_das_pontas(false), anisotropia_das_pontas(true));
    assert!(fios < 0.8, "sem grão a ponta tem de abrir em fios: {fios:.2}");
    assert!(
        grumos > 0.9,
        "com o grão preso à tela a ponta tem de esfarelar (isotrópica): {grumos:.2} (sem grão {fios:.2})"
    );
}

/// As transições tinta↔papel da miniatura ao longo de x e de y.
fn transicoes(p: &Preset) -> (f64, f64) {
    let m = miniatura_do_preset(p);
    let lado = m.largura as usize;
    let t: Vec<bool> = m.rgba.chunks(4).map(|px| tinta(px) >= 0.25).collect();
    let (mut ex, mut ey) = (0usize, 0usize);
    for (i, &v) in t.iter().enumerate() {
        ex += usize::from(i % lado + 1 < lado && v != t[i + 1]);
        ey += usize::from(i + lado < t.len() && v != t[i + lado]);
    }
    (ex as f64, ey as f64)
}

/// ⭐ **O expoente ATRAVÉS** (W3 K5, `textureSizeExpAcross`): o 2.º eixo do Size das cerdas cresce pelo seu próprio
/// expoente. Negativo escrito (o default) = sem a chave, ao byte; a `0` com `γ = 1` no 1.º eixo, o 2.º fica nos px de
/// tela — as cerdas crescem num eixo só e a razão das transições muda (medida no relatório).
#[test]
fn o_eixo_atraves_tem_expoente_proprio() {
    let com = |atraves: Option<f64>| {
        let mut p = cerdas(96.0, Some(1.0));
        if let Some(a) = atraves {
            p.parametros.engine.insert("textureSizeExpAcross".into(), a);
        }
        p
    };
    assert_eq!(
        miniatura_do_preset(&com(Some(-0.01))),
        miniatura_do_preset(&com(None)),
        "o expoente através negativo tem de ser o de sempre, ao byte"
    );
    let (ax, ay) = transicoes(&com(None));
    let (bx, by) = transicoes(&com(Some(0.0)));
    let (dois, um) = (ay / ax, by / bx);
    println!("y÷x: os dois eixos {dois:.3} · só o 1.º {um:.3} (x {ax} → {bx}, y {ay} → {by})");
    assert!(um > 1.3 * dois, "y÷x com o 2.º eixo preso aos px {um:.3}, com os dois a crescer {dois:.3}");
}

/// ⭐ **A nuvem do spray pousa o pigmento de UM dab** (W3 K5): quatro marcas de `particle` 0,1 do candidato Metallic
/// Splats (pigmento 50, cerdas fracas) eram DISCOS PÁLIDOS — o depósito é por célula e, juntas, pousavam 4 % do
/// pigmento de um dab. A tinta média dos píxeis da nuvem é a de marcas ESCURAS (o Rebelle), não de um véu.
#[test]
fn a_nuvem_do_spray_pousa_o_pigmento_de_um_dab() {
    let mut p = dabber();
    p.parametros.volumes.size_px = Some(96.0);
    for (k, v) in [("bristleStrength", 0.0), ("felt", 0.05), ("pigmentPerDab", 50.0)] {
        p.parametros.engine.insert(k.into(), v);
    }
    p.parametros.brush.insert("spray_count".into(), json!(4));
    let mut j = p.parametros.brush.get("dab_jitter").cloned().unwrap_or_else(|| json!({}));
    j["particle"] = json!(0.1);
    j["across"] = json!(1.0);
    p.parametros.brush.insert("dab_jitter".into(), j);
    let v: Vec<f64> = miniatura_do_preset(&p).rgba.chunks(4).map(tinta).filter(|&t| t > 0.02).collect();
    let media = v.iter().sum::<f64>() / v.len().max(1) as f64;
    println!("tinta média dos píxeis da nuvem: {media:.3} ({} píxeis)", v.len());
    assert!(media > 0.15, "as marcas do spray são discos pálidos: tinta média {media:.3}");
}
