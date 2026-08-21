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
/// **O codec + a migração** — irmão pelo tecto de LOC. Veja os docs do módulo.
pub mod codec;
/// Compor a folha em retângulos DADOS — a metade que o bake precisa. Irmão do [`pack`].
pub mod compose;
pub mod pack;
pub use aseprite::to_aseprite_json;
pub use codec::{SheetDocError, decode, encode};
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
/// - **v3** — a folha declara o seu `premultiplied`.
/// - **v4** — o payload de [`SpritePixelDoc`] vira [`PixelPayload`], que sabe ser de 16 bits
///   (plano [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)).
///   ⚠️ **Este bump traz MIGRAÇÃO**, ao contrário dos anteriores: ver [`decode`].
///
/// ⚠️ **O bump da v2 é a prova do desenho, não uma nota:** ele acrescentou uma capacidade inteira
/// ao formato de arquivo e o `PROJECT_SCHEMA` **não se moveu** — logo nenhum projeto já salvo foi
/// recusado. Era exatamente para isto que o campo nasceu como blob auto-versionado. A v4 gasta a
/// mesma moeda: 16 bits nos pixels **não** move o `PROJECT_SCHEMA`.
pub const SHEET_DOC_VERSION: u32 = 4;

/// **Os pixels de um sprite, e a variante É a precisão.**
///
/// ⚠️ **Por que um enum e não um campo `precision` ao lado de `rgba`:** um campo separado pode
/// discordar do payload — um documento com `precision = Rgba16` e `4·w·h` bytes é representável, e
/// alguém teria de o validar em toda a leitura. Aqui esse estado **não existe**. É a mesma lei que
/// o projeto já paga noutros sítios: *a representação apaga o caso especial*.
///
/// ⚠️ **`Rgba16` são bits de MEIO-FLOAT em espaço LINEAR**, não inteiros de 16 bits e não sRGB —
/// vide [`ph2d_color::precision`]. Guardar aqui os bytes sRGB promovidos a `u16` seria um
/// documento que abre e renderiza mais claro.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelPayload {
    /// RGBA8 sRGB justo: `width · height · 4` elementos.
    Rgba8(Vec<u8>),
    /// Meio-float linear justo: `width · height · 4` elementos (⇒ o dobro dos bytes).
    Rgba16(Vec<u16>),
}

impl PixelPayload {
    /// Elementos por pixel — quatro, em qualquer das duas. ⚠️ O que muda entre elas é o tamanho de
    /// **cada** elemento, e por isso a validação conta ELEMENTOS e não bytes: contar bytes daria a
    /// resposta certa para 8 bits e o dobro do esperado para 16.
    const CHANNELS: usize = 4;

    /// Quantos elementos o payload tem.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Rgba8(v) => v.len(),
            Self::Rgba16(v) => v.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// A precisão que esta variante É — o mesmo tipo que o `Asset` e a conversão usam, para não
    /// haver dois nomes para a mesma coisa.
    #[must_use]
    pub fn precision(&self) -> ph2d_color::Precision {
        match self {
            Self::Rgba8(_) => ph2d_color::Precision::Rgba8,
            Self::Rgba16(_) => ph2d_color::Precision::Rgba16,
        }
    }
}

/// Os pixels próprios de um sprite, como o arquivo os guarda.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpritePixelDoc {
    /// A identidade durável: blake3 dos pixels (o mesmo `AssetId` que o `AssetDb` cunha), e o
    /// valor que o `ph2d_ecs::SpritePixels` carrega no sprite.
    pub id: AssetId,
    pub width: u32,
    pub height: u32,
    /// Os pixels, justos: exatamente `width · height · 4` **elementos**
    /// ([`SheetDocError::PixelCountMismatch`]). A variante diz a precisão.
    pub pixels: PixelPayload,
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
        // ⚠️ Conta ELEMENTOS, não bytes: um payload de 16 bits tem os mesmos `w·h·4` elementos e o
        // DOBRO dos bytes. Contar bytes reprovaria toda imagem de 16 bits válida.
        let expected = (self.width as usize)
            .saturating_mul(self.height as usize)
            .saturating_mul(PixelPayload::CHANNELS);
        if self.pixels.len() != expected {
            return Err(SheetDocError::PixelCountMismatch {
                id: self.id,
                expected,
                found: self.pixels.len(),
            });
        }
        Ok(())
    }

    /// A precisão destes pixels — atalho para [`PixelPayload::precision`].
    #[must_use]
    pub fn precision(&self) -> ph2d_color::Precision {
        self.pixels.precision()
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
