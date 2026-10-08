//! O formato `*.ph2d-brush.png`: round-trip ao byte, campo desconhecido, preset mínimo, recusas.

use ph2d_brush_presets::{Erro, Miniatura, Origem, Parametros, Preset, escrever, ler};
use std::collections::BTreeMap;

fn miniatura(lado: u32) -> Miniatura {
    let rgba = (0..lado * lado)
        .flat_map(|i| {
            let (x, y) = (i % lado, i / lado);
            [
                (x * 7) as u8,
                (y * 5) as u8,
                ((x ^ y) * 3) as u8,
                200 + (x % 50) as u8,
            ]
        })
        .collect();
    Miniatura {
        largura: lado,
        altura: lado,
        rgba,
    }
}

fn preset_cheio() -> Preset {
    let mut p = Parametros::novo("WetPaint", "Fluido", "Aguada «molhada»");
    p.volumes.size_px = Some(40.0);
    p.volumes.opacity = Some(0.85);
    p.volumes.water = Some(1.0);
    p.paint_mode = Some("Paint".to_owned());
    p.engine.insert("pigmentPerDab".to_owned(), 800.0);
    p.engine.insert("paperGate".to_owned(), 0.4);
    p.engine.insert("felt".to_owned(), 0.03);
    p.engine.insert("gravity".to_owned(), 1.0 / 3.0);
    p.brush.insert("spacing".to_owned(), serde_json::json!(0.1));
    p.brush
        .insert("stroke_method".to_owned(), serde_json::json!("Space"));
    p.brush.insert(
        "paper".to_owned(),
        serde_json::json!({"kind": "Canvas", "scale": 1.25}),
    );
    p.width_table = vec![[2.0, 1.0], [15.3, 10.0], [46.6, 52.0]];
    let mut texturas = BTreeMap::new();
    texturas.insert("grao-1".to_owned(), vec![1, 2, 3, 0, 255]);
    Preset {
        parametros: p,
        origem: Origem::Fabrica,
        miniatura: miniatura(64),
        texturas,
    }
}

#[test]
fn um_preset_volta_ao_byte() {
    let p = preset_cheio();
    let b = escrever(&p).unwrap();
    let lido = ler(&b).unwrap();
    assert_eq!(lido, p, "ler(escrever(p)) == p");
    assert_eq!(escrever(&lido).unwrap(), b, "escrever(ler(b)) == b");
}

#[test]
fn a_mesma_entrada_da_os_mesmos_bytes() {
    assert_eq!(
        escrever(&preset_cheio()).unwrap(),
        escrever(&preset_cheio()).unwrap()
    );
}

/// Um PNG à mão com o JSON dado (o escritor de uma versão futura), 2×2 RGBA, filtro 0.
fn png_com(json: Option<&str>) -> Vec<u8> {
    fn chunk(out: &mut Vec<u8>, tipo: &[u8; 4], d: &[u8]) {
        out.extend_from_slice(&(d.len() as u32).to_be_bytes());
        let mut h = crc32fast::Hasher::new();
        h.update(tipo);
        h.update(d);
        out.extend_from_slice(tipo);
        out.extend_from_slice(d);
        out.extend_from_slice(&h.finalize().to_be_bytes());
    }
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&2u32.to_be_bytes());
    ihdr.extend_from_slice(&2u32.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    if let Some(j) = json {
        let mut d = b"ph2d-brush\0\0\0\0\0".to_vec();
        d.extend_from_slice(j.as_bytes());
        chunk(&mut out, b"iTXt", &d);
    }
    let cru = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 0, 9, 9, 9, 9, 7, 7, 7, 7];
    chunk(
        &mut out,
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(&cru, 6),
    );
    chunk(&mut out, b"IEND", &[]);
    out
}

#[test]
fn um_campo_desconhecido_nao_falha_e_sobrevive_a_regravacao() {
    let json = r#"{
        "schema": 1, "media": "WetPaint", "name": "Futuro",
        "volumes": {"size_px": 30.0, "blow": 0.25},
        "dual_brush": {"count": 3, "curve": [[0.0, 0.0], [1.0, 0.5]]},
        "bristles": {"thin": 0.52}
    }"#;
    let p = ler(&png_com(Some(json))).expect("campo desconhecido não falha");
    assert_eq!(p.parametros.volumes.size_px, Some(30.0));
    assert_eq!(
        p.parametros.volumes.extra.get("blow"),
        Some(&serde_json::json!(0.25))
    );
    assert_eq!(
        p.parametros.extra.get("dual_brush"),
        Some(&serde_json::json!({"count": 3, "curve": [[0.0, 0.0], [1.0, 0.5]]}))
    );
    assert!(p.parametros.extra.contains_key("bristles"));
    // E sobrevive: regravado por ESTA versão, relido, os campos estão lá.
    let de_novo = ler(&escrever(&p).unwrap()).unwrap();
    assert_eq!(de_novo.parametros, p.parametros);
    // Sem `tEXt ph2d-origin` o ficheiro é importado.
    assert_eq!(p.origem, Origem::Importado);
}

#[test]
fn um_preset_minimo_le_se_com_nada_fixado() {
    let p = ler(&png_com(Some(
        r#"{"schema": 1, "media": "WetPaint", "name": "Só o nome"}"#,
    )))
    .unwrap();
    assert_eq!(p.parametros.group, "");
    assert_eq!(p.parametros.volumes, Default::default());
    assert!(p.parametros.engine.is_empty() && p.parametros.brush.is_empty());
    assert_eq!(
        p.miniatura.rgba,
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9, 9, 7, 7, 7, 7]
    );
}

#[test]
fn as_recusas_dizem_porque() {
    assert_eq!(ler(b"GIF89a"), Err(Erro::NaoEPng));
    assert_eq!(ler(&png_com(None)), Err(Erro::SemParametros));
    assert_eq!(
        ler(&png_com(Some(
            r#"{"schema": 2, "media": "WetPaint", "name": "x"}"#
        ))),
        Err(Erro::SchemaFuturo(2))
    );
    assert!(matches!(
        ler(&png_com(Some(r#"{"media": "WetPaint", "name": "x"}"#))),
        Err(Erro::Json(_))
    ));
    // Um byte trocado nos parâmetros: o CRC apanha-o.
    let mut b = escrever(&preset_cheio()).unwrap();
    let i = b.windows(4).position(|w| w == b"iTXt").unwrap() + 20;
    b[i] ^= 0x40;
    assert!(matches!(ler(&b), Err(Erro::ChunkCorrompido(c)) if c.contains("iTXt")));
    // Truncado a meio da imagem.
    let b = escrever(&preset_cheio()).unwrap();
    assert!(matches!(
        ler(&b[..b.len() - 40]),
        Err(Erro::ChunkCorrompido(_))
    ));
}

#[test]
fn um_valor_nao_finito_nao_se_escreve() {
    let mut p = preset_cheio();
    p.parametros.engine.insert("drag".to_owned(), f64::NAN);
    assert!(matches!(escrever(&p), Err(Erro::Json(_))));
    let mut p = preset_cheio();
    p.miniatura.rgba.pop();
    assert!(matches!(escrever(&p), Err(Erro::MiniaturaIlegivel(_))));
}

#[test]
fn uma_textura_cinzenta_volta_igual() {
    let lum: Vec<u8> = (0..37 * 11).map(|i| (i * 7 % 256) as u8).collect();
    let b = ph2d_brush_presets::escrever_textura(&lum, 37, 11);
    assert_eq!(ph2d_brush_presets::ler_textura(&b).unwrap(), (lum, 37, 11));
}
