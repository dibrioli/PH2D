//! **A ESCRITA do formato do artista** — uma folha nossa vira `folha.png` + `folha.json` que o
//! Aseprite, o TexturePacker e qualquer engine que os leia entendem.
//!
//! ## Por que escrever, e não só ler
//!
//! Ler já sabíamos (`ph2d_asset::parse_atlas_meta`). Escrever é o que torna a ferramenta de
//! empacotar **reversível**: a folha que o PH2D produz sai da ferramenta como um par de arquivos
//! normais, o artista abre-os no Aseprite, e o import do PH2D lê-os de volta. Sem isto a folha
//! ficaria presa dentro do arquivo de projeto — e uma ferramenta de autoria que só sabe escrever
//! para dentro de si mesma é um beco.
//!
//! ## E é o que torna a ferramenta testável contra si própria
//!
//! `pack → to_aseprite_json → parse_atlas_meta` tem de devolver os MESMOS retângulos. É um
//! round-trip contra o leitor que já existe (e não contra uma segunda cópia da nossa opinião
//! sobre o formato), então ele falha se qualquer das duas metades derivar.
//!
//! ## O `serde_json` não é conforto
//!
//! Os nomes das regiões vêm do artista e podem conter aspas, barras e acentos. Montar o JSON com
//! `format!` produziria um arquivo inválido no primeiro nome com `"` — o tipo de bug que só
//! aparece no ficheiro de outra pessoa.

use crate::AuthoredSheet;

/// A "app" que declaramos no `meta`, para quem abrir o arquivo saber de onde veio.
const APP: &str = "https://github.com/dibrioli/PH2D";

/// Serializa a folha na forma **Aseprite "Hash"** (a que o [`ph2d_asset::parse_atlas_meta`] lê).
///
/// `image_filename` é o nome do PNG irmão — só o nome, sem pasta: o formato resolve-o **relativo
/// ao próprio `.json`**, e gravar um caminho absoluto aqui quebraria o arquivo assim que o artista
/// movesse a pasta.
pub fn to_aseprite_json(sheet: &AuthoredSheet, image_filename: &str) -> String {
    let mut frames = serde_json::Map::new();
    for r in &sheet.regions {
        let [x, y, w, h] = r.rect;
        frames.insert(
            r.name.clone(),
            serde_json::json!({
                "frame": { "x": x, "y": y, "w": w, "h": h },
                // Campos que o Aseprite emite e que consumidores de terceiros esperam. Nós não
                // rodamos nem recortamos ao empacotar, então são constantes — mas omiti-los faria
                // ferramentas mais estritas recusarem o arquivo.
                "rotated": false,
                "trimmed": false,
                "spriteSourceSize": { "x": 0, "y": 0, "w": w, "h": h },
                "sourceSize": { "w": w, "h": h },
            }),
        );
    }
    let doc = serde_json::json!({
        "frames": serde_json::Value::Object(frames),
        "meta": {
            "app": APP,
            "image": image_filename,
            "format": "RGBA8888",
            "size": { "w": sheet.width, "h": sheet.height },
            "scale": "1",
        }
    });
    // `to_string_pretty` porque este arquivo é para HUMANOS tanto quanto para máquinas — o artista
    // abre-o para conferir um nome, e o `git diff` de uma folha versionada tem de ser legível.
    serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::{PackInput, PackOptions, pack};

    fn img(name: &str, w: u32, h: u32) -> PackInput {
        PackInput {
            name: name.to_string(),
            width: w,
            height: h,
            rgba: vec![0x40; (w * h * 4) as usize],
        }
    }

    /// ⚠️ **O round-trip que fecha o ciclo da ferramenta**: empacotar, exportar, e reler pelo
    /// leitor que o import de facto usa. Se qualquer das duas metades derivar, isto fica vermelho.
    #[test]
    fn a_packed_sheet_round_trips_through_our_own_reader() {
        let sheet = pack(
            7,
            "heroi".into(),
            vec![img("andar_0", 24, 32), img("parado_0", 16, 16)],
            PackOptions::default(),
        )
        .expect("pack");
        let json = to_aseprite_json(&sheet, "heroi.png");
        let meta = ph2d_asset::parse_atlas_meta(json.as_bytes()).expect("o nosso leitor aceita");
        assert_eq!(meta.image_filename, "heroi.png");
        assert_eq!(meta.image_size, (sheet.width, sheet.height));
        assert_eq!(meta.regions.len(), sheet.regions.len());
        for r in &sheet.regions {
            let back = meta.region(&r.name).expect("regiao presente");
            assert_eq!(
                [back.x, back.y, back.w, back.h],
                r.rect,
                "o retangulo de '{}' mudou pelo caminho",
                r.name
            );
        }
    }

    /// ⚠️ Os nomes vêm do artista. Montar o JSON com `format!` produziria um arquivo inválido no
    /// primeiro nome com aspas — e o modo de falha seria no ficheiro de OUTRA pessoa.
    #[test]
    fn a_region_name_with_quotes_and_backslashes_survives() {
        let awkward = r#"he said "hi"\path"#;
        let sheet = pack(
            1,
            "s".into(),
            vec![img(awkward, 8, 8)],
            PackOptions::default(),
        )
        .expect("pack");
        let json = to_aseprite_json(&sheet, "s.png");
        let meta = ph2d_asset::parse_atlas_meta(json.as_bytes()).expect("JSON valido");
        assert!(meta.region(awkward).is_some(), "o nome tem de voltar igual");
    }

    /// O caminho da imagem é **só o nome**: o formato resolve-o relativo ao `.json`, e um
    /// caminho absoluto quebraria assim que o artista movesse a pasta.
    #[test]
    fn the_image_reference_is_a_bare_filename() {
        let sheet =
            pack(1, "s".into(), vec![img("a", 8, 8)], PackOptions::default()).expect("pack");
        let json = to_aseprite_json(&sheet, "s.png");
        let meta = ph2d_asset::parse_atlas_meta(json.as_bytes()).expect("parse");
        assert!(!meta.image_filename.contains('/'));
        assert!(!meta.image_filename.contains('\\'));
    }

    /// Exportar é determinístico, como empacotar: a mesma folha dá o mesmo arquivo.
    #[test]
    fn exporting_twice_gives_the_same_bytes() {
        let sheet = pack(
            1,
            "s".into(),
            vec![img("b", 8, 8), img("a", 8, 8)],
            PackOptions::default(),
        )
        .expect("pack");
        assert_eq!(
            to_aseprite_json(&sheet, "s.png"),
            to_aseprite_json(&sheet, "s.png")
        );
    }
}

/// Um quadro do JSON de exportação do Pixel Art.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonFrame {
    pub filename: String,
    pub frame: [u32; 4],
    pub trimmed: bool,
    pub source: [u32; 4],
    pub source_size: [u32; 2],
    pub duration_ms: u32,
}

/// Uma tag do JSON. `repeat` 0 = para sempre, e nesse caso o campo SOME (a convenção dos importadores).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonTag {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub direction: &'static str,
    pub repeat: u16,
    pub color: [u8; 4],
}

/// Uma camada do JSON. `blend`/`opacity` `None` num grupo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonLayer {
    pub name: String,
    pub group: Option<String>,
    pub opacity: Option<u8>,
    pub blend: Option<&'static str>,
}

/// O par `.json` inteiro de uma folha exportada, na forma do Aseprite: chaves pela ordem dele, quadros
/// pela ordem da linha do tempo. `array` = `json-array`, senão `json-hash`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SheetJson {
    pub frames: Vec<JsonFrame>,
    pub array: bool,
    pub image: String,
    /// `RGBA8888` ou `I8`.
    pub format: &'static str,
    pub size: [u32; 2],
    /// `None` = a lista não foi pedida, e a chave não sai (como `--list-tags`).
    pub tags: Option<Vec<JsonTag>>,
    pub layers: Option<Vec<JsonLayer>>,
    pub slices: Option<Vec<JsonSlice>>,
}

/// Uma slice em `meta.slices`. Cor ausente e texto vazio não saem (medido, `P11_oraculos_arquivo.md` F5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonSlice {
    pub name: String,
    pub color: Option<[u8; 4]>,
    pub data: String,
    pub keys: Vec<JsonSliceKey>,
    /// `"ph2d.collider": true` — a slice é um collider do PH2D (P14e); `false` não sai.
    pub collider: bool,
}

impl JsonSlice {
    /// Lê uma entrada de `meta.slices` (a forma que [`sheet_json`] escreve); `None` sem `name`.
    #[must_use]
    pub fn ler(v: &serde_json::Value) -> Option<Self> {
        let caixa = |b: &serde_json::Value| -> Option<(i32, i32, u32, u32)> {
            let n = |k: &str| b[k].as_i64();
            Some((
                i32::try_from(n("x")?).ok()?,
                i32::try_from(n("y")?).ok()?,
                u32::try_from(n("w")?).ok()?,
                u32::try_from(n("h")?).ok()?,
            ))
        };
        let cor = |t: &str| -> Option<[u8; 4]> {
            let h = t.strip_prefix('#').filter(|h| h.len() == 8)?;
            let b = |i: usize| u8::from_str_radix(h.get(i..i + 2)?, 16).ok();
            Some([b(0)?, b(2)?, b(4)?, b(6)?])
        };
        let keys = v["keys"].as_array().map_or_else(Vec::new, |ks| {
            ks.iter()
                .filter_map(|k| {
                    let p = &k["pivot"];
                    Some(JsonSliceKey {
                        frame: u32::try_from(k["frame"].as_u64()?).ok()?,
                        bounds: caixa(&k["bounds"])?,
                        center: caixa(&k["center"]),
                        pivot: p["x"].as_i64().zip(p["y"].as_i64()).and_then(|(x, y)| {
                            Some((i32::try_from(x).ok()?, i32::try_from(y).ok()?))
                        }),
                    })
                })
                .collect()
        });
        Some(Self {
            name: v["name"].as_str()?.to_owned(),
            color: v["color"].as_str().and_then(cor),
            data: v["data"].as_str().unwrap_or_default().to_owned(),
            keys,
            collider: v["ph2d.collider"].as_bool() == Some(true),
        })
    }
}

/// `(x, y, w, h)` em `bounds` e `center`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JsonSliceKey {
    pub frame: u32,
    pub bounds: (i32, i32, u32, u32),
    pub center: Option<(i32, i32, u32, u32)>,
    pub pivot: Option<(i32, i32)>,
}

/// Uma slice, com os espaços que o Aseprite escreve.
fn json_slice(o: &mut String, s: &JsonSlice) {
    use std::fmt::Write as _;
    let caixa = |(x, y, w, h): (i32, i32, u32, u32)| {
        format!("{{\"x\": {x}, \"y\": {y}, \"w\": {w}, \"h\": {h} }}")
    };
    let _ = write!(o, "   {{ \"name\": {}", cadeia(&s.name));
    if let Some([r, g, b, a]) = s.color {
        let _ = write!(o, ", \"color\": \"#{r:02x}{g:02x}{b:02x}{a:02x}\"");
    }
    if !s.data.is_empty() {
        let _ = write!(o, ", \"data\": {}", cadeia(&s.data));
    }
    if !s.keys.is_empty() {
        o.push_str(", \"keys\": [");
        for (i, k) in s.keys.iter().enumerate() {
            if i > 0 {
                o.push_str(", ");
            }
            let _ = write!(
                o,
                "{{ \"frame\": {}, \"bounds\": {}",
                k.frame,
                caixa(k.bounds)
            );
            if let Some(c) = k.center {
                let _ = write!(o, ", \"center\": {}", caixa(c));
            }
            if let Some((x, y)) = k.pivot {
                let _ = write!(o, ", \"pivot\": {{\"x\": {x}, \"y\": {y} }}");
            }
            o.push_str(" }");
        }
        o.push(']');
    }
    if s.collider {
        o.push_str(", \"ph2d.collider\": true");
    }
    o.push_str(" }");
}

fn cadeia(t: &str) -> String {
    serde_json::Value::from(t).to_string()
}

fn caixa(r: [u32; 4]) -> String {
    format!(
        "{{ \"x\": {}, \"y\": {}, \"w\": {}, \"h\": {} }}",
        r[0], r[1], r[2], r[3]
    )
}

/// Escreve o texto do JSON. À mão e com `serde_json` só nos textos: a ordem das chaves faz parte da
/// forma que os importadores conhecem, e o `Map` dele ordena por nome.
#[must_use]
pub fn sheet_json(j: &SheetJson) -> String {
    use std::fmt::Write as _;
    let mut o = String::from(if j.array {
        "{ \"frames\": [\n"
    } else {
        "{ \"frames\": {\n"
    });
    for (i, f) in j.frames.iter().enumerate() {
        let sep = if i + 1 == j.frames.len() { "" } else { "," };
        if j.array {
            let _ = write!(o, "   {{\n    \"filename\": {},\n", cadeia(&f.filename));
        } else {
            let _ = writeln!(o, "   {}: {{", cadeia(&f.filename));
        }
        let _ = write!(
            o,
            "    \"frame\": {},\n    \"rotated\": false,\n    \"trimmed\": {},\n    \"spriteSourceSize\": {},\n    \"sourceSize\": {{ \"w\": {}, \"h\": {} }},\n    \"duration\": {}\n   }}{sep}\n",
            caixa(f.frame),
            f.trimmed,
            caixa(f.source),
            f.source_size[0],
            f.source_size[1],
            f.duration_ms,
        );
    }
    o.push_str(if j.array { " ],\n" } else { " },\n" });
    let _ = write!(
        o,
        " \"meta\": {{\n  \"app\": {},\n  \"version\": {},\n  \"image\": {},\n  \"format\": {},\n  \"size\": {{ \"w\": {}, \"h\": {} }},\n  \"scale\": \"1\"",
        cadeia(APP),
        cadeia(env!("CARGO_PKG_VERSION")),
        cadeia(&j.image),
        cadeia(j.format),
        j.size[0],
        j.size[1],
    );
    if let Some(tags) = &j.tags {
        o.push_str(",\n  \"frameTags\": [\n");
        for (i, t) in tags.iter().enumerate() {
            let _ = write!(
                o,
                "   {{ \"name\": {}, \"from\": {}, \"to\": {}, \"direction\": {}",
                cadeia(&t.name),
                t.from,
                t.to,
                cadeia(t.direction)
            );
            if t.repeat != 0 {
                let _ = write!(o, ", \"repeat\": \"{}\"", t.repeat);
            }
            let [r, g, b, a] = t.color;
            let _ = write!(o, ", \"color\": \"#{r:02x}{g:02x}{b:02x}{a:02x}\" }}");
            o.push_str(if i + 1 == tags.len() { "\n" } else { ",\n" });
        }
        o.push_str("  ]");
    }
    if let Some(layers) = &j.layers {
        o.push_str(",\n  \"layers\": [\n");
        for (i, l) in layers.iter().enumerate() {
            let _ = write!(o, "   {{ \"name\": {}", cadeia(&l.name));
            if let Some(g) = &l.group {
                let _ = write!(o, ", \"group\": {}", cadeia(g));
            }
            if let Some(v) = l.opacity {
                let _ = write!(o, ", \"opacity\": {v}");
            }
            if let Some(b) = l.blend {
                let _ = write!(o, ", \"blendMode\": {}", cadeia(b));
            }
            o.push_str(" }");
            o.push_str(if i + 1 == layers.len() { "\n" } else { ",\n" });
        }
        o.push_str("  ]");
    }
    if let Some(slices) = &j.slices {
        o.push_str(",\n  \"slices\": [\n");
        for (i, s) in slices.iter().enumerate() {
            json_slice(&mut o, s);
            o.push_str(if i + 1 == slices.len() { "\n" } else { ",\n" });
        }
        o.push_str("  ]");
    }
    o.push_str("\n }\n}\n");
    o
}
