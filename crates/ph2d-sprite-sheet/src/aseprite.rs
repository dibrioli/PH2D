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
        let sheet = pack(1, "s".into(), vec![img(awkward, 8, 8)], PackOptions::default())
            .expect("pack");
        let json = to_aseprite_json(&sheet, "s.png");
        let meta = ph2d_asset::parse_atlas_meta(json.as_bytes()).expect("JSON valido");
        assert!(meta.region(awkward).is_some(), "o nome tem de voltar igual");
    }

    /// O caminho da imagem é **só o nome**: o formato resolve-o relativo ao `.json`, e um
    /// caminho absoluto quebraria assim que o artista movesse a pasta.
    #[test]
    fn the_image_reference_is_a_bare_filename() {
        let sheet = pack(1, "s".into(), vec![img("a", 8, 8)], PackOptions::default()).expect("pack");
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
