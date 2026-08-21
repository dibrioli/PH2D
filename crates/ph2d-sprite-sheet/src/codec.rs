//! **O CODEC do documento de folhas** — como ele vira bytes e volta, e a **migração** das versões
//! antigas. Irmão do [`crate`], que define *o que o documento É*.
//!
//! ⚠️ **Saiu de lá por medição** (2026-08-20): o `lib.rs` chegou a **901** linhas contra um tecto de
//! 700 quando o [`crate::PixelPayload`] e a migração da v4 entraram (plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)
//! W3). A regra registada deste projeto é **cortar, nunca alargar a allowlist**.
//!
//! O corte é por responsabilidade: lá ficam os TIPOS (o que um documento é, e o que o torna
//! válido); aqui fica a TRAVESSIA para bytes — que é onde vive a única coisa que pode partir um
//! ficheiro de um artista.

use serde::{Deserialize, Serialize};

use crate::{AuthoredSheet, PixelPayload, SHEET_DOC_VERSION, SpritePixelDoc};
use ph2d_asset::AssetId;

/// O documento como o arquivo o guarda.
#[derive(Serialize, Deserialize)]
struct SheetDoc {
    version: u32,
    pixels: Vec<SpritePixelDoc>,
    /// v2: as folhas hand-packed.
    sheets: Vec<AuthoredSheet>,
}

/// Por que um documento não pôde ser lido ou escrito.
#[derive(Debug, PartialEq, Eq)]
pub enum SheetDocError {
    /// Os bytes não são este documento (ou estão truncados).
    Postcard,
    /// O arquivo é de outra versão do formato. ⚠️ **Recusa o load inteiro** — vide o módulo.
    UnsupportedVersion { found: u32, expected: u32 },
    /// `rgba.len()` não bate com `width * height * 4`.
    PixelCountMismatch {
        id: AssetId,
        expected: usize,
        found: usize,
    },
    /// O mesmo, para uma folha autorada.
    SheetPixelCountMismatch {
        sheet: u32,
        expected: usize,
        found: usize,
    },
    /// Uma região aponta para fora da folha — quase sempre um `.json` e um `.png` que divergiram
    /// (o artista editou um sem re-exportar o outro).
    RegionOutsideSheet { sheet: u32, name: String },
}

impl std::fmt::Display for SheetDocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postcard => write!(f, "sprite pixel document is not readable"),
            Self::UnsupportedVersion { found, expected } => write!(
                f,
                "sprite pixel document version {found}, this build reads {expected}"
            ),
            Self::PixelCountMismatch {
                id,
                expected,
                found,
            } => write!(
                f,
                "sprite pixels {} declare {expected} bytes but carry {found}",
                id.to_hex()
            ),
            Self::SheetPixelCountMismatch {
                sheet,
                expected,
                found,
            } => write!(
                f,
                "sheet {sheet} declares {expected} pixel bytes but carries {found}"
            ),
            Self::RegionOutsideSheet { sheet, name } => write!(
                f,
                "region '{name}' extends past sheet {sheet} — image and metadata out of sync?"
            ),
        }
    }
}

impl std::error::Error for SheetDocError {}

/// Serializa para o campo do arquivo de projeto.
///
/// ⚠️ **Ordena por id e retira duplicados**, e as duas coisas são contrato, não arrumação: a
/// ordem faz dois saves da mesma cena produzirem bytes idênticos (HR-5), e o dedup é o que torna
/// o hash de conteúdo uma economia real quando N sprites partilham os mesmos pixels.
///
/// Uma lista vazia devolve `Vec` vazio — um projeto sem pixels próprios não paga bytes nem
/// versão, e o [`decode`] devolve vazio para vazio (é o que faz um projeto anterior carregar).
pub fn encode(
    pixels: &[SpritePixelDoc],
    sheets: &[AuthoredSheet],
) -> Result<Vec<u8>, SheetDocError> {
    if pixels.is_empty() && sheets.is_empty() {
        return Ok(Vec::new());
    }
    let mut pixels = pixels.to_vec();
    pixels.sort_by_key(|a| a.id);
    pixels.dedup_by(|a, b| a.id == b.id);
    for p in &pixels {
        p.validate()?;
    }
    let mut sheets = sheets.to_vec();
    sheets.sort_by_key(|s| s.id);
    sheets.dedup_by_key(|s| s.id);
    for s in &sheets {
        s.validate()?;
    }
    let doc = SheetDoc {
        version: SHEET_DOC_VERSION,
        pixels,
        sheets,
    };
    postcard::to_allocvec(&doc).map_err(|_| SheetDocError::Postcard)
}

/// **A v3 tal como o arquivo a gravou** — o payload era `Vec<u8>` cru, sem variante.
///
/// ⚠️ Esta cópia existe para **não se mexer nela nunca mais**. O tipo vivo evolui; este é a forma
/// que já está gravada no disco de alguém, e um `postcard` não é auto-descritivo: mudar um campo
/// aqui não dá erro de leitura, dá **pixels embaralhados**.
#[derive(Deserialize)]
struct SpritePixelDocV3 {
    id: AssetId,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    premultiplied: bool,
}

/// O `SheetDoc` da v3. As folhas não mudaram, por isso reusa [`AuthoredSheet`].
#[derive(Deserialize)]
struct SheetDocV3 {
    /// ⚠️ **Nunca lido, e tem de estar aqui na mesma.** O `postcard` não é auto-descritivo: os
    /// campos consomem-se por posição, e retirar este faria o `pixels` ler os bytes da versão.
    /// O valor já veio do [`VersionProbe`] — quem o lê é o `decode`, antes de escolher a forma.
    #[allow(
        dead_code,
        reason = "consumido posicionalmente pelo postcard; lido via VersionProbe"
    )]
    version: u32,
    pixels: Vec<SpritePixelDocV3>,
    sheets: Vec<AuthoredSheet>,
}

/// Só o cabeçalho, para saber que forma esperar antes de a decodificar.
///
/// ⚠️ `postcard::take_from_bytes` é o que torna isto possível: ele lê os campos que este tipo pede
/// e **devolve o resto** em vez de reprovar por bytes a mais, que é o que `from_bytes` faria.
#[derive(Deserialize)]
struct VersionProbe {
    version: u32,
}

/// Lê do campo do arquivo de projeto.
///
/// ⚠️ **Todo erro daqui tem de RECUSAR o load inteiro** (vide o módulo): devolver uma lista vazia
/// abriria uma cena que parece certa com os sprites em branco, e o próximo `Ctrl+S` gravaria esse
/// vazio por cima do arquivo do artista.
///
/// # A migração, e por que ela existe aqui e não existia antes
///
/// Os bumps v1→v2→v3 **recusavam** o que não fosse a versão corrente, e podiam: eles aconteceram
/// dentro da mesma jornada em que o formato nasceu. A v4 é a primeira a chegar depois de haver
/// projetos gravados — e recusar aqui não devolve um erro simpático ao artista, **recusa o load
/// inteiro** (é o que o parágrafo acima manda). *Um formato só precisa de migração a partir do dia
/// em que alguém guardou alguma coisa nele.*
///
/// Um documento v3 sobe para v4 embrulhando o `rgba` em [`PixelPayload::Rgba8`] — que é exatamente
/// o que ele sempre foi, dito com o tipo novo. Sem perda, e com gate de ida-e-volta.
pub fn decode(bytes: &[u8]) -> Result<(Vec<SpritePixelDoc>, Vec<AuthoredSheet>), SheetDocError> {
    if bytes.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let (probe, _) =
        postcard::take_from_bytes::<VersionProbe>(bytes).map_err(|_| SheetDocError::Postcard)?;
    if probe.version == 3 {
        let old: SheetDocV3 = postcard::from_bytes(bytes).map_err(|_| SheetDocError::Postcard)?;
        let pixels: Vec<SpritePixelDoc> = old
            .pixels
            .into_iter()
            .map(|p| SpritePixelDoc {
                id: p.id,
                width: p.width,
                height: p.height,
                pixels: PixelPayload::Rgba8(p.rgba),
                premultiplied: p.premultiplied,
            })
            .collect();
        for p in &pixels {
            p.validate()?;
        }
        for s in &old.sheets {
            s.validate()?;
        }
        return Ok((pixels, old.sheets));
    }
    let doc: SheetDoc = postcard::from_bytes(bytes).map_err(|_| SheetDocError::Postcard)?;
    if doc.version != SHEET_DOC_VERSION {
        return Err(SheetDocError::UnsupportedVersion {
            found: doc.version,
            expected: SHEET_DOC_VERSION,
        });
    }
    for p in &doc.pixels {
        p.validate()?;
    }
    for s in &doc.sheets {
        s.validate()?;
    }
    Ok((doc.pixels, doc.sheets))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(w: u32, h: u32, seed: u8) -> Vec<u8> {
        (0..(w * h * 4))
            .map(|i| (i as u8).wrapping_add(seed))
            .collect()
    }

    fn doc(w: u32, h: u32, seed: u8) -> SpritePixelDoc {
        let rgba = rgba(w, h, seed);
        SpritePixelDoc {
            id: AssetId::from_bytes(&rgba),
            width: w,
            height: h,
            pixels: PixelPayload::Rgba8(rgba),
            premultiplied: false,
        }
    }

    /// O irmão de 16 bits do [`doc`], para os gates da v4.
    fn doc16(w: u32, h: u32, seed: u8) -> SpritePixelDoc {
        let rgba = rgba(w, h, seed);
        let halves = ph2d_color::rgba8_to_rgba16(&rgba);
        SpritePixelDoc {
            id: AssetId::from_bytes(&rgba),
            width: w,
            height: h,
            pixels: PixelPayload::Rgba16(halves),
            premultiplied: false,
        }
    }

    #[test]
    fn round_trip_is_byte_identical() {
        let d = doc(4, 3, 0);
        let bytes = encode(std::slice::from_ref(&d), &[]).expect("encode");
        assert_eq!(decode(&bytes).expect("decode").0, vec![d]);
    }

    /// A ida-e-volta de 16 bits, e que ela **não** se confunde com a de 8.
    #[test]
    fn sixteen_bit_pixels_round_trip_and_keep_their_variant() {
        let d = doc16(4, 3, 0);
        let bytes = encode(std::slice::from_ref(&d), &[]).expect("encode");
        let back = decode(&bytes).expect("decode").0;
        assert_eq!(back, vec![d.clone()]);
        assert_eq!(back[0].precision(), ph2d_color::Precision::Rgba16);
        // ⚠️ Controle: o payload de 16 bits ocupa o DOBRO dos bytes com o mesmo número de
        // elementos. Sem isto, uma implementação que gravasse 8 bits com uma etiqueta de 16
        // passaria a ida-e-volta e mentiria sobre a precisão.
        let eight = encode(&[doc(4, 3, 0)], &[]).expect("encode");
        assert!(
            bytes.len() > eight.len() + 40,
            "o documento de 16 bits ({} B) devia pesar ~o dobro do de 8 ({} B)",
            bytes.len(),
            eight.len()
        );
    }

    /// **Um documento v3 real continua a abrir** — o gate da migração.
    ///
    /// ⚠️ Os bytes são construídos com a forma v3 **verdadeira** (`rgba: Vec<u8>` cru, versão 3),
    /// não com o tipo vivo: um teste que serializasse o tipo de hoje e o lesse de volta provaria
    /// apenas que hoje concorda consigo próprio, que é a forma clássica de um gate de migração
    /// passar sem migrar nada.
    #[test]
    fn a_v3_document_still_opens_and_becomes_eight_bit() {
        #[derive(Serialize)]
        struct V3Pixels {
            id: AssetId,
            width: u32,
            height: u32,
            rgba: Vec<u8>,
            premultiplied: bool,
        }
        #[derive(Serialize)]
        struct V3Doc {
            version: u32,
            pixels: Vec<V3Pixels>,
            sheets: Vec<AuthoredSheet>,
        }
        let rgba = rgba(4, 3, 5);
        let id = AssetId::from_bytes(&rgba);
        let old = V3Doc {
            version: 3,
            pixels: vec![V3Pixels {
                id,
                width: 4,
                height: 3,
                rgba: rgba.clone(),
                premultiplied: true,
            }],
            sheets: Vec::new(),
        };
        let bytes = postcard::to_allocvec(&old).expect("encode v3");
        let (pixels, sheets) = decode(&bytes).expect("um projeto v3 tem de continuar a abrir");
        assert!(sheets.is_empty());
        assert_eq!(pixels.len(), 1);
        assert_eq!(pixels[0].id, id);
        assert_eq!(pixels[0].pixels, PixelPayload::Rgba8(rgba));
        assert_eq!(pixels[0].precision(), ph2d_color::Precision::Rgba8);
        assert!(
            pixels[0].premultiplied,
            "o `premultiplied` da v3 nao sobreviveu a' migracao — a franja escura do BG-Removal \
             voltaria em todo projeto ja' gravado"
        );
    }

    /// ⚠️ **Controle positivo da migração:** uma versão que não é nem a corrente nem a v3 continua
    /// a ser recusada. Sem isto, um `decode` que aceitasse tudo passaria o teste acima.
    #[test]
    fn an_unknown_version_is_still_refused() {
        let doc = SheetDoc {
            version: SHEET_DOC_VERSION + 1,
            pixels: Vec::new(),
            sheets: Vec::new(),
        };
        let bytes = postcard::to_allocvec(&doc).expect("encode");
        assert_eq!(
            decode(&bytes),
            Err(SheetDocError::UnsupportedVersion {
                found: SHEET_DOC_VERSION + 1,
                expected: SHEET_DOC_VERSION,
            })
        );
    }

    #[test]
    fn empty_encodes_to_nothing_and_decodes_back() {
        assert!(encode(&[], &[]).expect("encode").is_empty());
        assert_eq!(decode(&[]).expect("decode").0, Vec::new());
    }

    /// O dedup é a razão de a identidade ser o CONTEÚDO: dois sprites com os mesmos pixels
    /// custam uma entrada, não duas.
    #[test]
    fn identical_pixels_cost_one_entry() {
        let d = doc(4, 4, 7);
        let bytes = encode(&[d.clone(), d.clone()], &[]).expect("encode");
        assert_eq!(decode(&bytes).expect("decode").0, vec![d]);
    }

    /// A ordem é contrato: a MESMA cena declarada ao contrário grava os MESMOS bytes.
    #[test]
    fn encoding_is_order_independent() {
        let a = doc(2, 2, 1);
        let b = doc(2, 2, 9);
        assert_eq!(
            encode(&[a.clone(), b.clone()], &[]).expect("encode"),
            encode(&[b, a], &[]).expect("encode")
        );
    }

    #[test]
    fn pixels_that_do_not_match_the_declared_size_are_refused() {
        let mut bad = doc(4, 4, 0);
        let PixelPayload::Rgba8(ref mut v) = bad.pixels else {
            unreachable!("o `doc` de teste constroi 8 bits")
        };
        v.truncate(8);
        assert_eq!(
            encode(std::slice::from_ref(&bad), &[]),
            Err(SheetDocError::PixelCountMismatch {
                id: bad.id,
                expected: 64,
                found: 8,
            })
        );
    }

    /// A lei do módulo: um documento de outra versão RECUSA, nunca devolve vazio — senão o
    /// próximo `Ctrl+S` grava o vazio por cima da obra.
    #[test]
    fn a_document_from_another_version_is_refused_not_silently_empty() {
        let d = doc(1, 1, 0);
        let bytes = postcard::to_allocvec(&SheetDoc {
            version: SHEET_DOC_VERSION + 1,
            pixels: vec![d],
            sheets: Vec::new(),
        })
        .expect("encode");
        assert_eq!(
            decode(&bytes),
            Err(SheetDocError::UnsupportedVersion {
                found: SHEET_DOC_VERSION + 1,
                expected: SHEET_DOC_VERSION,
            })
        );
    }

    fn sheet(id: u32, w: u32, h: u32, regions: &[(&str, [u32; 4])]) -> AuthoredSheet {
        AuthoredSheet::new(
            id,
            format!("sheet{id}"),
            w,
            h,
            rgba(w, h, 0),
            regions.iter().map(|(n, r)| (n.to_string(), *r)),
        )
    }

    #[test]
    fn a_sheet_round_trips_byte_identical() {
        let sh = sheet(
            1,
            8,
            8,
            &[("idle_0", [0, 0, 4, 4]), ("idle_1", [4, 0, 4, 4])],
        );
        let bytes = encode(&[], std::slice::from_ref(&sh)).expect("encode");
        let (pixels, sheets) = decode(&bytes).expect("decode");
        assert!(pixels.is_empty());
        assert_eq!(sheets, vec![sh]);
    }

    /// ⚠️ **A lei que torna o ÍNDICE uma referência durável.** Um sprite hand-packed guarda
    /// `region: u32`; se a ordem dependesse de como o `.json` foi lido, re-importar a mesma folha
    /// re-apontaria cada sprite para o desenho errado — em silêncio.
    #[test]
    fn regions_are_sorted_by_name_so_the_index_is_stable() {
        let a = sheet(1, 8, 8, &[("zulu", [4, 0, 4, 4]), ("alpha", [0, 0, 4, 4])]);
        let b = sheet(1, 8, 8, &[("alpha", [0, 0, 4, 4]), ("zulu", [4, 0, 4, 4])]);
        assert_eq!(
            a, b,
            "a mesma folha, declarada ao contrario, e' a mesma folha"
        );
        assert_eq!(a.region(0).map(|r| r.name.as_str()), Some("alpha"));
        assert_eq!(a.region(1).map(|r| r.name.as_str()), Some("zulu"));
        assert_eq!(a.region(2), None);
    }

    /// Um `.png` e um `.json` que divergiram: o retangulo sai da folha.
    #[test]
    fn a_region_outside_the_sheet_is_refused() {
        let bad = sheet(9, 4, 4, &[("huge", [2, 2, 8, 8])]);
        assert_eq!(
            encode(&[], std::slice::from_ref(&bad)),
            Err(SheetDocError::RegionOutsideSheet {
                sheet: 9,
                name: "huge".to_string(),
            })
        );
    }

    /// ⚠️ `x + w` em `u32` daria a volta e o retangulo absurdo passaria a "caber".
    #[test]
    fn a_region_whose_rect_would_overflow_u32_is_refused() {
        let bad = sheet(11, 4, 4, &[("wrap", [u32::MAX, 0, 8, 4])]);
        assert!(matches!(
            encode(&[], std::slice::from_ref(&bad)),
            Err(SheetDocError::RegionOutsideSheet { .. })
        ));
    }

    /// As duas metades do documento viajam juntas e nao se confundem.
    #[test]
    fn pixels_and_sheets_coexist_in_one_document() {
        let d = doc(2, 2, 5);
        let sh = sheet(3, 4, 4, &[("a", [0, 0, 2, 2])]);
        let bytes = encode(std::slice::from_ref(&d), std::slice::from_ref(&sh)).expect("encode");
        let (pixels, sheets) = decode(&bytes).expect("decode");
        assert_eq!(pixels, vec![d]);
        assert_eq!(sheets, vec![sh]);
    }

    #[test]
    fn garbage_bytes_are_refused() {
        assert!(decode(&[0xff; 3]).is_err());
    }

    /// ⚠️ O flag de premultiplicado é a única cópia que existe — `Sprite::premultiplied` é
    /// `#[serde(skip)]`. Perdê-lo aqui devolve a franja escura do BG-Removal ao reabrir.
    #[test]
    fn the_premultiplied_flag_survives_the_round_trip() {
        let mut d = doc(2, 2, 3);
        d.premultiplied = true;
        let bytes = encode(std::slice::from_ref(&d), &[]).expect("encode");
        let back = decode(&bytes).expect("decode").0;
        assert!(back[0].premultiplied, "o flag tem de voltar `true`");
        assert_eq!(back, vec![d]);
    }
}
