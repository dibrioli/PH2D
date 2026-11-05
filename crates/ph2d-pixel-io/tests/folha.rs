//! A folha contra o LibreSprite corrido: dimensões, hash da imagem e o JSON, caso a caso.
//! Fora da comparação, com o motivo: `vertical` (ele devolve a horizontal), `rows_h32`/`columns_w48`
//! (ele ignora o limite no outro eixo), `crop` (ele ignora; não temos), `packed` (posição não é lei:
//! ver `packed_*`), `range_*` (o nome dele traz «#Tag»: o teste compara só o resto).

use ph2d_pixel_core::{Layer, Sprite};
use ph2d_pixel_io::ase;
use ph2d_pixel_io::render::{Camadas, Quadros};
use ph2d_pixel_io::sheet::{Limite, OpcoesFolha, OpcoesJson, Tipo, json_da_folha, montar};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn oraculo() -> Value {
    serde_json::from_str(include_str!("fixtures/oracle_export.json")).unwrap()
}

fn unhex(h: &str) -> Vec<u8> {
    (0..h.len() / 2)
        .map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap())
        .collect()
}

fn doc(o: &Value, n: &str) -> Sprite {
    ase::ler(&unhex(o["entradas_hex"][n].as_str().unwrap()))
        .unwrap()
        .0
}

/// O mesmo CLI do oráculo, para a tabela de casos ser a da fixtura.
fn opcoes(s: &Sprite, caso: &Value) -> OpcoesFolha {
    let mut o = OpcoesFolha::default();
    let mut largura = None;
    let mut altura = None;
    for lado in ["antes", "depois"] {
        let a: Vec<&str> = caso[lado]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let mut i = 0;
        while i < a.len() {
            let arg = |k: usize| a[i + k];
            match a[i] {
                "--sheet-type" => {
                    o.tipo = match arg(1) {
                        "horizontal" => Tipo::Horizontal,
                        "vertical" => Tipo::Vertical,
                        "columns" => Tipo::Columns,
                        _ => Tipo::Rows,
                    };
                    i += 1;
                }
                "--sheet-pack" => o.tipo = Tipo::Packed,
                "--sheet-width" => {
                    largura = arg(1).parse().ok();
                    i += 1;
                }
                "--sheet-height" => {
                    altura = arg(1).parse().ok();
                    i += 1;
                }
                "--trim" => o.trim_cels = true,
                "--ignore-empty" => o.ignore_empty = true,
                "--split-layers" => o.split_layers = true,
                "--all-layers" => o.camadas = Camadas::Todas,
                "--border-padding" => {
                    o.border = arg(1).parse().unwrap();
                    i += 1;
                }
                "--shape-padding" => {
                    o.shape = arg(1).parse().unwrap();
                    i += 1;
                }
                "--inner-padding" => {
                    o.inner = arg(1).parse().unwrap();
                    i += 1;
                }
                "--layer" => {
                    let n = arg(1).trim_start_matches("grp-");
                    let l: &Layer = s.layers().iter().find(|l| l.name == n).unwrap();
                    o.camadas = Camadas::Uma(l.id);
                    i += 1;
                }
                "--frame-tag" => {
                    let t = s.tags().iter().position(|t| t.name == arg(1)).unwrap();
                    o.quadros = Quadros::Tag(t);
                    i += 1;
                }
                "--frame-range" => {
                    let (x, y) = arg(1).split_once(',').unwrap();
                    o.quadros = Quadros::Intervalo {
                        de: x.parse().unwrap(),
                        ate: y.parse().unwrap(),
                    };
                    i += 1;
                }
                "--filename-format" => {
                    o.nome = Some(arg(1).to_owned());
                    i += 1;
                }
                _ => {}
            }
            i += 1;
        }
    }
    if o.tipo == Tipo::Columns {
        o.limite = altura.map_or(Limite::Nenhum, Limite::Altura);
    } else {
        o.limite = largura.map_or(Limite::Nenhum, Limite::Largura);
    }
    o
}

/// `layer_oculta`: pedir uma camada escondida pelo nome dá-lhe uma folha VAZIA (medido); o artista que a
/// escolheu na lista quer vê-la, e aqui sai desenhada.
const FORA: [&str; 8] = [
    "vertical",
    "rows_h32",
    "columns_w48",
    "crop",
    "packed",
    "nome",
    "modos",
    "layer_oculta",
];

/// `grp-mult` é o nome achatado dele; o `{frame01}` ele não conhece.
fn nome_dele(n: &str) -> String {
    n.replace("grp-", "")
}

#[test]
fn a_folha_bate_com_o_libresprite() {
    let o = oraculo();
    let mut conferidos = 0;
    for caso in o["folhas"].as_array().unwrap() {
        let nome = caso["name"].as_str().unwrap();
        if FORA.contains(&nome) || caso["sheet"].is_null() {
            continue;
        }
        let (fonte, titulo) = if nome.starts_with("ix_") {
            ("indexado", "indexado")
        } else if nome == "um_quadro" {
            ("um_quadro_indexado", "um_quadro_indexado")
        } else {
            ("folha", "folha")
        };
        let s = doc(&o, fonte);
        let f = montar(&s, &opcoes(&s, caso), titulo, "ase");
        assert_eq!(
            (u64::from(f.w), u64::from(f.h)),
            (
                caso["sheet"]["w"].as_u64().unwrap(),
                caso["sheet"]["h"].as_u64().unwrap()
            ),
            "{nome}: dimensões"
        );
        let dele: Value =
            serde_json::from_str(caso["json-array"]["json"].as_str().unwrap()).unwrap();
        let df = dele["frames"].as_array().unwrap();
        assert_eq!(f.entradas.len(), df.len(), "{nome}: quantos quadros");
        for (e, d) in f.entradas.iter().zip(df) {
            let r = |k: &str, v: &Value| {
                [
                    v[k]["x"].as_u64().unwrap() as u32,
                    v[k]["y"].as_u64().unwrap() as u32,
                    v[k]["w"].as_u64().unwrap() as u32,
                    v[k]["h"].as_u64().unwrap() as u32,
                ]
            };
            assert_eq!(e.frame, r("frame", d), "{nome}: {}", e.nome);
            assert_eq!(e.origem, r("spriteSourceSize", d), "{nome}: {}", e.nome);
            assert_eq!(e.trimmed, d["trimmed"].as_bool().unwrap(), "{nome}");
            assert_eq!(
                u64::from(e.duration_ms),
                d["duration"].as_u64().unwrap(),
                "{nome}"
            );
            if !nome.starts_with("range") {
                assert_eq!(e.nome, nome_dele(d["filename"].as_str().unwrap()), "{nome}");
            }
        }
        let mut px = f.rgba.clone();
        // O RGB de alfa 0 não se vê: o PNG dele traz zero.
        for c in px.as_chunks_mut::<4>().0 {
            if c[3] == 0 {
                *c = [0; 4];
            }
        }
        let hex: String = Sha256::digest(&px)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            hex,
            caso["sheet"]["rgba_sha256"].as_str().unwrap(),
            "{nome}: pixels"
        );
        conferidos += 1;
    }
    assert!(conferidos >= 30, "só {conferidos} casos conferidos");
}

#[test]
fn o_json_tem_a_forma_do_dele() {
    let o = oraculo();
    let s = doc(&o, "folha");
    let caso = o["folhas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "list")
        .unwrap();
    let f = montar(&s, &OpcoesFolha::default(), "folha", "ase");
    for (fmt, array) in [("json-hash", false), ("json-array", true)] {
        let meu: Value = serde_json::from_str(&json_da_folha(
            &s,
            &f,
            "list.png",
            "RGBA8888",
            OpcoesJson {
                array,
                tags: true,
                layers: true,
                slices: false,
            },
        ))
        .unwrap();
        let dele: Value = serde_json::from_str(caso[fmt]["json"].as_str().unwrap()).unwrap();
        assert_eq!(
            meu["frames"], dele["frames"],
            "{fmt}: frames, chaves e valores"
        );
        for k in ["image", "format", "size", "scale"] {
            assert_eq!(meu["meta"][k], dele["meta"][k], "{fmt}: meta.{k}");
        }
        let so_dele = |v: &Value| -> Vec<Value> {
            v["meta"]["frameTags"].as_array().unwrap().iter().map(|t| {
                serde_json::json!({"name": t["name"], "from": t["from"], "to": t["to"], "direction": t["direction"]})
            }).collect()
        };
        assert_eq!(
            so_dele(&meu),
            so_dele(&dele),
            "{fmt}: frameTags, os campos dele"
        );
        // O que ele não grava e os importadores leem: repeat em texto, e ausente se infinito.
        let walk = &meu["meta"]["frameTags"][2];
        assert_eq!(walk["repeat"], "2");
        assert!(meu["meta"]["frameTags"][0].get("repeat").is_none());
        assert_eq!(meu["meta"]["frameTags"][0]["color"], "#ff0000ff");
    }
}

#[test]
fn packed_nao_sobrepoe_e_preserva() {
    let o = oraculo();
    let s = doc(&o, "folha");
    let f = montar(
        &s,
        &OpcoesFolha {
            tipo: Tipo::Packed,
            trim_cels: true,
            shape: 2,
            border: 3,
            ..OpcoesFolha::default()
        },
        "t",
        "png",
    );
    let r: Vec<_> = f.entradas.iter().map(|e| e.frame).collect();
    for (i, a) in r.iter().enumerate() {
        for b in &r[i + 1..] {
            let longe = a[0] + a[2] <= b[0]
                || b[0] + b[2] <= a[0]
                || a[1] + a[3] <= b[1]
                || b[1] + b[3] <= a[1];
            assert!(longe, "{a:?} × {b:?}");
        }
        assert!(a[0] >= 3 && a[1] >= 3 && a[0] + a[2] + 3 <= f.w && a[1] + a[3] + 3 <= f.h);
    }
    assert!(f.w.is_power_of_two() || (f.w - 6).is_power_of_two());
}

#[test]
fn merge_extrude_2n_e_split_de_tags() {
    let o = oraculo();
    let s = doc(&o, "folha");
    let merge = montar(
        &s,
        &OpcoesFolha {
            merge_duplicates: true,
            ..OpcoesFolha::default()
        },
        "t",
        "png",
    );
    assert_eq!(merge.entradas.len(), 5);
    assert_eq!(
        merge.entradas[2].frame, merge.entradas[3].frame,
        "o quadro repetido partilha"
    );
    assert_eq!(merge.w, 24 * 4, "o repetido não ocupa lugar");

    let e = montar(
        &s,
        &OpcoesFolha {
            extrude: 2,
            trim_cels: true,
            tipo: Tipo::Horizontal,
            ..OpcoesFolha::default()
        },
        "t",
        "png",
    );
    let p = e.entradas[0].frame;
    let at = |x: u32, y: u32| &e.rgba[(y * e.w + x) as usize * 4..][..4];
    assert_eq!(at(p[0] - 1, p[1]), at(p[0], p[1]), "a borda repete-se");
    assert_eq!(at(p[0] - 2, p[1]), at(p[0], p[1]));

    let d = montar(
        &s,
        &OpcoesFolha {
            potencia_de_2: true,
            ..OpcoesFolha::default()
        },
        "t",
        "png",
    );
    assert_eq!((d.w, d.h), (128, 16));

    let t = montar(
        &s,
        &OpcoesFolha {
            split_tags: true,
            ..OpcoesFolha::default()
        },
        "t",
        "png",
    );
    let nomes: Vec<_> = t.entradas.iter().map(|e| e.nome.as_str()).collect();
    assert_eq!(
        nomes[..3],
        ["t #idle 0.png", "t #idle 1.png", "t #walk 0.png"]
    );
    assert_eq!(t.entradas.len(), 2 + 3 + 3);
}

#[test]
fn o_sprite_inspector_le_a_folha() {
    let o = oraculo();
    let s = doc(&o, "folha");
    let f = montar(
        &s,
        &OpcoesFolha {
            trim_cels: true,
            ..OpcoesFolha::default()
        },
        "t",
        "png",
    );
    let j = json_da_folha(
        &s,
        &f,
        "t.png",
        "RGBA8888",
        OpcoesJson {
            tags: true,
            layers: true,
            ..OpcoesJson::default()
        },
    );
    let meta = ph2d_asset::parse_atlas_meta(j.as_bytes()).unwrap();
    assert_eq!(meta.regions.len(), f.entradas.len());
}
