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

pub mod aseprite;
/// Compor a folha em retângulos DADOS — a metade que o bake precisa. Irmão do [`pack`].
pub mod compose;
pub mod pack;
pub use aseprite::to_aseprite_json;
pub use compose::compose;
pub use pack::{Layout, LayoutItem, PackError, PackInput, PackOptions, layout, pack};

use ph2d_asset::AssetId;
use serde::{Deserialize, Serialize};

/// A versão do documento.
///
/// ⚠️ **Bumpe-a quando qualquer tipo dentro do blob mudar de forma.** O postcard é POSICIONAL: um
/// campo novo lido por um binário velho não falha — devolve lixo bem-formado. Aqui esse lixo
/// seriam *pixels*, então esta versão é a única coisa entre um artista e uma imagem embaralhada.
///
/// - **v1** — só [`SpritePixelDoc`] (os pixels próprios de um sprite `Individual`).
/// - **v2** — junta [`AuthoredSheet`]: as FOLHAS hand-packed, com as regiões nomeadas.
///
/// ⚠️ **E este bump é a prova do desenho, não uma nota:** ele acrescentou uma capacidade inteira
/// ao formato de arquivo e o `PROJECT_SCHEMA` **não se moveu** — logo nenhum projeto já salvo foi
/// recusado. Era exatamente para isto que o campo nasceu como blob auto-versionado.
pub const SHEET_DOC_VERSION: u32 = 3;

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

/// Uma região nomeada dentro de uma folha, em **pixels da folha**, com `(0, 0)` no canto
/// superior-esquerdo — a mesma convenção do `Asset::ImageRgba8` e do `Sprite::region_rect`, para
/// que o retângulo viaje até o extract sem nenhuma conversão pelo caminho.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetRegion {
    /// O nome que o artista deu (a chave do `frames` no JSON do Aseprite/TexturePacker),
    /// preservado verbatim. É o que o Inspector mostra; o que o `Sprite` guarda é o ÍNDICE.
    pub name: String,
    /// `[x, y, w, h]` em pixels da folha.
    pub rect: [u32; 4],
}

/// Uma **folha hand-packed**: uma imagem partilhada por N sprites, com as regiões que cada um usa.
///
/// ## Por que o id é um `u32` e não o [`AssetId`] dos pixels próprios
///
/// Uma folha é um **documento AUTORADO**: o artista arrasta uma região, e os pixels mudam. Um id
/// de conteúdo mudaria a cada gesto e obrigaria a re-carimbar todo sprite que a usa. O `u32` é
/// caller-supplied e estável ao longo da edição — exatamente o espírito do `ph2d_ecs::PaintedDoc`.
/// *Dois tempos de vida diferentes, não uma inconsistência* (plano `docs/Sprite_projeto/17` §3.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoredSheet {
    /// Identidade estável da folha. É o que `SpriteSource::HandPacked { sheet, .. }` guarda.
    pub id: u32,
    /// Nome legível (o do arquivo importado, ou o que a ferramenta de empacotar deu). Só para o
    /// Inspector — a identidade é o `id`.
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// RGBA8 justo: exatamente `width * height * 4` bytes.
    pub rgba: Vec<u8>,
    /// As regiões, **ordenadas por nome** — vide [`AuthoredSheet::new`]. O índice nesta lista é a
    /// referência durável que o `Sprite` guarda.
    pub regions: Vec<SheetRegion>,
    /// **Os bytes estão pré-multiplicados?** (v3, 2026-08-19.)
    ///
    /// ⚠️ **Não é um detalhe de formato — é o que decide a BORDA.** A amostragem bilinear
    /// interpola os texeis *antes* de o shader tocar neles, e interpolar alfa reto mistura a cor
    /// dos texeis transparentes na do vizinho opaco: medido, **50 de 255** de diferença no meio do
    /// gradiente de uma borda. Interpolar pré-multiplicado dá a resposta certa — e é também o que
    /// o gerador de mipmaps assume, pelo próprio cabeçalho dele.
    ///
    /// Uma folha **assada aqui** nasce pré-multiplicada, como as texturas do app já são; uma folha
    /// **importada** de um `.png` é reta, porque é isso que um PNG é. A bandeira viaja com os
    /// bytes para que quem os liga a um sprite não tenha de adivinhar.
    #[serde(default)]
    pub premultiplied: bool,
}

impl AuthoredSheet {
    /// Constrói a partir de pares `(nome, [x, y, w, h])`.
    ///
    /// ⚠️ **Ordena por nome, e é isso que torna o índice uma referência estável:** o parser do
    /// Aseprite entrega um `BTreeMap` (já ordenado) e a ferramenta de empacotar entrega o que o
    /// artista arranjou — passar as duas por esta porta faz o mesmo `.json` produzir sempre a
    /// mesma folha, byte-a-byte (HR-5). É o que o teste de round-trip afirma.
    pub fn new(
        id: u32,
        name: String,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        regions: impl IntoIterator<Item = (String, [u32; 4])>,
    ) -> Self {
        Self::new_with_alpha(id, name, width, height, rgba, regions, false)
    }

    /// Como [`Self::new`], dizendo explicitamente se os bytes estão pré-multiplicados.
    ///
    /// ⚠️ O `new` continua a assumir **reto** de propósito: é o que um `.png` é, e o import — o
    /// consumidor mais antigo — não devia mudar de significado por causa desta wave.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_alpha(
        id: u32,
        name: String,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        regions: impl IntoIterator<Item = (String, [u32; 4])>,
        premultiplied: bool,
    ) -> Self {
        let mut regions: Vec<SheetRegion> = regions
            .into_iter()
            .map(|(name, rect)| SheetRegion { name, rect })
            .collect();
        regions.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            id,
            name,
            width,
            height,
            rgba,
            regions,
            premultiplied,
        }
    }

    /// A região de índice `i`, ou `None`. O Inspector usa isto para NOMEAR o que o sprite mostra —
    /// `Hand-packed · hero · idle_0` em vez de dois números crus.
    pub fn region(&self, index: u32) -> Option<&SheetRegion> {
        self.regions.get(index as usize)
    }

    /// **Esta folha precisa do recuo de meio texel na amostragem?**
    ///
    /// `true` quando duas regiões ficam a menos de um pixel uma da outra. Aí a amostragem
    /// bilinear — que alcança meio texel para lá da borda — puxaria o desenho vizinho, e o recuo
    /// (`Sprite::region_filter_clip`) é a defesa.
    ///
    /// ⚠️ **Mas ele não é grátis, e é por isso que esta pergunta existe:** o recuo come meio texel
    /// da própria região, e num sprite com borda suavizada esse meio texel é a parte mais fraca do
    /// contorno. Ligá-lo numa folha com folga custa fidelidade de borda **em troca de nada** — foi
    /// exatamente isso que o Enio viu ao assar (2026-08-19: *"a borda transparente muda"*).
    ///
    /// *Uma defesa que se liga sem olhar para o que ela corta é um palpite com custo.* Aqui a
    /// resposta é DERIVADA da folha, então uma folha do empacotador (que separa por `padding`) não
    /// paga, e uma folha colada de fora paga — cada uma pelo que de facto é.
    #[must_use]
    pub fn regions_need_filter_clip(&self) -> bool {
        // O `padding` de uma folha nossa é 2 px, então o caso comum sai por aqui sem comparar
        // nada quando há 0 ou 1 região.
        for (i, a) in self.regions.iter().enumerate() {
            for b in &self.regions[i + 1..] {
                if rects_within_one_pixel(a.rect, b.rect) {
                    return true;
                }
            }
        }
        false
    }

    fn validate(&self) -> Result<(), SheetDocError> {
        let expected = (self.width as usize)
            .saturating_mul(self.height as usize)
            .saturating_mul(4);
        if self.rgba.len() != expected {
            return Err(SheetDocError::SheetPixelCountMismatch {
                sheet: self.id,
                expected,
                found: self.rgba.len(),
            });
        }
        for r in &self.regions {
            let [x, y, w, h] = r.rect;
            // Soma em `u64` de propósito: `x + w` em `u32` daria a volta e um retângulo absurdo
            // passaria a "caber" dentro da folha.
            if u64::from(x) + u64::from(w) > u64::from(self.width)
                || u64::from(y) + u64::from(h) > u64::from(self.height)
            {
                return Err(SheetDocError::RegionOutsideSheet {
                    sheet: self.id,
                    name: r.name.clone(),
                });
            }
        }
        Ok(())
    }
}

/// Dois retângulos a menos de um pixel um do outro — encostados, sobrepostos, ou separados por
/// zero pixels de folga.
///
/// ⚠️ **A conta é feita EXPANDINDO um deles em 1 px** e testando interseção, em `i64`: expandir em
/// `u32` daria a volta num retângulo colado à origem, e o teste diria «longe» sobre um vizinho
/// colado.
fn rects_within_one_pixel(a: [u32; 4], b: [u32; 4]) -> bool {
    let ax0 = i64::from(a[0]) - 1;
    let ay0 = i64::from(a[1]) - 1;
    let ax1 = i64::from(a[0]) + i64::from(a[2]) + 1;
    let ay1 = i64::from(a[1]) + i64::from(a[3]) + 1;
    let bx0 = i64::from(b[0]);
    let by0 = i64::from(b[1]);
    let bx1 = bx0 + i64::from(b[2]);
    let by1 = by0 + i64::from(b[3]);
    ax0 < bx1 && bx0 < ax1 && ay0 < by1 && by0 < ay1
}

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

/// Lê do campo do arquivo de projeto.
///
/// ⚠️ **Todo erro daqui tem de RECUSAR o load inteiro** (vide o módulo): devolver uma lista vazia
/// abriria uma cena que parece certa com os sprites em branco, e o próximo `Ctrl+S` gravaria esse
/// vazio por cima do arquivo do artista.
pub fn decode(bytes: &[u8]) -> Result<(Vec<SpritePixelDoc>, Vec<AuthoredSheet>), SheetDocError> {
    if bytes.is_empty() {
        return Ok((Vec::new(), Vec::new()));
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
mod filter_clip_tests {
    use super::*;

    fn sheet(rects: &[[u32; 4]]) -> AuthoredSheet {
        AuthoredSheet::new(
            0,
            "s".into(),
            256,
            256,
            vec![0; 256 * 256 * 4],
            rects.iter().enumerate().map(|(i, r)| (format!("r{i}"), *r)),
        )
    }

    /// ⚠️ **O caso que devolve a borda ao artista.** Uma folha do empacotador separa por
    /// `padding` (2 px por omissão) — não há vizinho ao alcance da amostragem bilinear, e ligar o
    /// recuo custaria meio texel do contorno de cada peça em troca de nada.
    #[test]
    fn a_sheet_with_padding_needs_no_clip() {
        assert!(!sheet(&[[0, 0, 10, 10], [12, 0, 10, 10]]).regions_need_filter_clip());
    }

    /// ⚠️ E o caso que o exige: coladas, a amostragem puxa a vizinha pela borda.
    #[test]
    fn touching_regions_need_the_clip() {
        assert!(sheet(&[[0, 0, 10, 10], [10, 0, 10, 10]]).regions_need_filter_clip());
    }

    /// **UM pixel de folga já chega** — e a primeira versão deste teste afirmava o contrário.
    ///
    /// ⚠️ Eu tinha escrito *"a amostragem alcança meio texel para cada lado, e as duas metades
    /// encontram-se"*. Está errado, e a conta di-lo: a região `A` acaba em `x = 10.0`; amostrar na
    /// borda dela alcança `x = 10.5`, que cai no texel 10 — **a folga**, transparente. Para tocar
    /// no primeiro texel de `B` (o 11) seria preciso alcançar um texel inteiro, e a bilinear
    /// alcança meio. *Só um dos lados amostra; as metades não se somam.*
    ///
    /// O código estava certo e o teste errado — e isso só se descobre fazendo a conta com os
    /// índices na mão, que é o que este comentário guarda para a próxima vez.
    #[test]
    fn one_pixel_of_gap_is_already_enough() {
        assert!(!sheet(&[[0, 0, 10, 10], [11, 0, 10, 10]]).regions_need_filter_clip());
    }

    /// O eixo Y conta como o X — uma folha em coluna não pode escapar por o teste só olhar em X.
    #[test]
    fn the_vertical_axis_counts_too() {
        assert!(sheet(&[[0, 0, 10, 10], [0, 10, 10, 10]]).regions_need_filter_clip());
        assert!(!sheet(&[[0, 0, 10, 10], [0, 12, 10, 10]]).regions_need_filter_clip());
    }

    /// Uma região sozinha (ou nenhuma) não tem vizinho — nunca precisa.
    #[test]
    fn a_lone_region_never_needs_it() {
        assert!(!sheet(&[[0, 0, 10, 10]]).regions_need_filter_clip());
        assert!(!sheet(&[]).regions_need_filter_clip());
    }

    /// ⚠️ Colada à ORIGEM: expandir em `u32` daria a volta e o teste diria «longe» sobre um
    /// vizinho encostado. A conta é em `i64` por causa deste caso.
    #[test]
    fn a_region_at_the_origin_does_not_wrap() {
        assert!(sheet(&[[0, 0, 4, 4], [4, 0, 4, 4]]).regions_need_filter_clip());
    }
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
            rgba,
            premultiplied: false,
        }
    }

    #[test]
    fn round_trip_is_byte_identical() {
        let d = doc(4, 3, 0);
        let bytes = encode(std::slice::from_ref(&d), &[]).expect("encode");
        assert_eq!(decode(&bytes).expect("decode").0, vec![d]);
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
        bad.rgba.truncate(8);
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
