#![forbid(unsafe_code)]
//! **OS PIXELS PRÓPRIOS de um sprite** — os que não vivem no atlas dinâmico, como o arquivo os
//! guarda.
//!
//! Crate fina de propósito (só `ph2d-asset` pela identidade, `serde` e `postcard`): a shell, o
//! painel do Inspector e a futura ferramenta de empacotar consomem este documento, e nenhum dos
//! três conhece os outros.
//!
//! ## O problema que ele existe para resolver
//!
//! `SpriteSource::Individual { texture_id }` guarda um **id de alocação da GPU** dentro de um
//! componente **persistido**. O `IndividualTextureStore` recomeça a numerar em `1` a cada
//! processo, então noutra sessão aquele id ou não existe (o sprite **some**) ou pertence a outra
//! textura (o sprite exibe **os pixels de outro**). Aqui ficam os bytes, e o
//! [`ph2d_ecs::SpritePixels`] carimbado no sprite é o nome deles.
//!
//! ## A identidade é o CONTEÚDO
//!
//! O id é o [`AssetId`] (blake3 dos pixels, HR-6), e isso não é decoração: dois sprites com os
//! mesmos pixels **partilham uma entrada** no arquivo, e re-salvar sem editar produz o mesmo
//! documento byte-a-byte.
//!
//! ⚠️ **Isto vale porque estes pixels são um SNAPSHOT imutável.** Uma folha *autorada* (o
//! hand-packed do plano §6-§7) muda a cada arrasto do artista, e um id de conteúdo obrigaria a
//! re-carimbar todo sprite a cada gesto — por isso ela virá com um id estável de **documento**, no
//! espírito do `PaintedDoc`, e não com este. São dois tempos de vida diferentes, não uma
//! inconsistência.
//!
//! ## Ele carrega a própria versão
//!
//! [`SHEET_DOC_VERSION`] mora **dentro** do blob, então este módulo evolui muitas waves sem tocar
//! o `PROJECT_SCHEMA` — o precedente exato do `TimelineDoc` e do documento de escultura. O
//! `PROJECT_SCHEMA` bumpa **uma vez**, quando o campo nasce, e é isso. ⚠️ É esta escolha que fará
//! o hand-packed (que acrescenta as regiões a este mesmo documento) custar **zero** recusa de
//! projeto salvo — sem ela, cada wave recusaria todo arquivo do artista.
//!
//! ## Um documento ilegível RECUSA o load inteiro
//!
//! A mesma lei do documento de escultura, e aqui mais afiada porque isto **são os pixels**: abrir
//! sem eles mostraria uma cena que parece certa com os sprites em branco, e o **próximo `Ctrl+S`
//! gravaria esse vazio por cima do arquivo**. A obra não sumiria por um bug; sumiria porque o app
//! abriu, mentiu e salvou. O parse acontece **antes** de qualquer mutação da sessão, então recusar
//! não custa nada ao documento aberto.

use ph2d_asset::AssetId;
use serde::{Deserialize, Serialize};

/// A versão do documento de pixels próprios.
///
/// ⚠️ **Bumpe-a quando qualquer tipo dentro do blob mudar de forma.** O postcard é POSICIONAL: um
/// campo novo lido por um binário velho não falha — devolve lixo bem-formado. Aqui esse lixo
/// seriam *pixels*, então esta versão é a única coisa entre um artista e uma imagem embaralhada.
pub const SHEET_DOC_VERSION: u32 = 1;

/// Os pixels próprios de um sprite, como o arquivo os guarda.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpritePixelDoc {
    /// A identidade durável: blake3 dos pixels (o mesmo `AssetId` que o `AssetDb` cunha), e o
    /// valor que o `ph2d_ecs::SpritePixels` carrega no sprite.
    pub id: AssetId,
    pub width: u32,
    pub height: u32,
    /// RGBA8 justo: exatamente `width * height * 4` bytes
    /// ([`SheetDocError::PixelCountMismatch`]).
    pub rgba: Vec<u8>,
    /// `true` ⇒ estes bytes estão PREMULTIPLICADOS (o resultado de um Apply do BG-Removal).
    ///
    /// ⚠️ **Ele TEM de viajar aqui, e a razão é que ele não viaja em mais lado nenhum:**
    /// `Sprite::premultiplied` é `#[serde(skip)]` — é uma dica de runtime que sempre volta
    /// `false` do `WorldSnapshot`. Sem este campo, reabrir um sprite com fundo removido
    /// devolveria bytes premultiplicados marcados como alfa reto, e a franja escura na borda
    /// anti-serrilhada voltaria: exatamente o bug que o `commit_edited_texture` existe para
    /// impedir, ressuscitado pelo caminho do arquivo.
    ///
    /// Guardado ao lado dos bytes (e não derivado depois) porque é um facto SOBRE eles —
    /// derivar significaria adivinhar, e adivinhar erra no meio-alfa.
    pub premultiplied: bool,
}

impl SpritePixelDoc {
    /// Valida contra as próprias declarações. Chamada no encode **e** no decode: um documento
    /// inválido nunca chega ao disco, e um que lá esteja nunca chega à sessão.
    fn validate(&self) -> Result<(), SheetDocError> {
        let expected = (self.width as usize)
            .saturating_mul(self.height as usize)
            .saturating_mul(4);
        if self.rgba.len() != expected {
            return Err(SheetDocError::PixelCountMismatch {
                id: self.id,
                expected,
                found: self.rgba.len(),
            });
        }
        Ok(())
    }
}

/// O documento como o arquivo o guarda.
#[derive(Serialize, Deserialize)]
struct SheetDoc {
    version: u32,
    pixels: Vec<SpritePixelDoc>,
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
pub fn encode(pixels: &[SpritePixelDoc]) -> Result<Vec<u8>, SheetDocError> {
    if pixels.is_empty() {
        return Ok(Vec::new());
    }
    let mut pixels = pixels.to_vec();
    pixels.sort_by(|a, b| a.id.cmp(&b.id));
    pixels.dedup_by(|a, b| a.id == b.id);
    for p in &pixels {
        p.validate()?;
    }
    let doc = SheetDoc {
        version: SHEET_DOC_VERSION,
        pixels,
    };
    postcard::to_allocvec(&doc).map_err(|_| SheetDocError::Postcard)
}

/// Lê do campo do arquivo de projeto.
///
/// ⚠️ **Todo erro daqui tem de RECUSAR o load inteiro** (vide o módulo): devolver uma lista vazia
/// abriria uma cena que parece certa com os sprites em branco, e o próximo `Ctrl+S` gravaria esse
/// vazio por cima do arquivo do artista.
pub fn decode(bytes: &[u8]) -> Result<Vec<SpritePixelDoc>, SheetDocError> {
    if bytes.is_empty() {
        return Ok(Vec::new());
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
    Ok(doc.pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(w: u32, h: u32, seed: u8) -> Vec<u8> {
        (0..(w * h * 4)).map(|i| (i as u8).wrapping_add(seed)).collect()
    }

    fn doc(w: u32, h: u32, seed: u8) -> SpritePixelDoc {
        let rgba = rgba(w, h, seed);
        SpritePixelDoc {
            id: AssetId::from_bytes(&rgba),
            width: w,
            height: h,
            rgba,
            premultiplied: false,
        }
    }

    #[test]
    fn round_trip_is_byte_identical() {
        let d = doc(4, 3, 0);
        let bytes = encode(std::slice::from_ref(&d)).expect("encode");
        assert_eq!(decode(&bytes).expect("decode"), vec![d]);
    }

    #[test]
    fn empty_encodes_to_nothing_and_decodes_back() {
        assert!(encode(&[]).expect("encode").is_empty());
        assert_eq!(decode(&[]).expect("decode"), Vec::new());
    }

    /// O dedup é a razão de a identidade ser o CONTEÚDO: dois sprites com os mesmos pixels
    /// custam uma entrada, não duas.
    #[test]
    fn identical_pixels_cost_one_entry() {
        let d = doc(4, 4, 7);
        let bytes = encode(&[d.clone(), d.clone()]).expect("encode");
        assert_eq!(decode(&bytes).expect("decode"), vec![d]);
    }

    /// A ordem é contrato: a MESMA cena declarada ao contrário grava os MESMOS bytes.
    #[test]
    fn encoding_is_order_independent() {
        let a = doc(2, 2, 1);
        let b = doc(2, 2, 9);
        assert_eq!(
            encode(&[a.clone(), b.clone()]).expect("encode"),
            encode(&[b, a]).expect("encode")
        );
    }

    #[test]
    fn pixels_that_do_not_match_the_declared_size_are_refused() {
        let mut bad = doc(4, 4, 0);
        bad.rgba.truncate(8);
        assert_eq!(
            encode(std::slice::from_ref(&bad)),
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
        let bytes = encode(std::slice::from_ref(&d)).expect("encode");
        let back = decode(&bytes).expect("decode");
        assert!(back[0].premultiplied, "o flag tem de voltar `true`");
        assert_eq!(back, vec![d]);
    }
}
