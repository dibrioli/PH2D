//! ⭐⭐ **O QUE UM CARTÃO DESENHA, e a memória disso** — irmão por ASSUNTO do
//! [`super::asset_index_build`], que bateu no tecto de 600 LOC do shell.
//!
//! A junção responde *«que assets existem?»*; este ficheiro responde *«com que cara»* — a cor
//! dominante, a miniatura, e a memória por CONTEÚDO que torna as duas pagáveis.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_asset_index::Thumb;
use std::collections::BTreeMap;

/// Quantos pixels a média amostra, no máximo.
///
/// ⚠️ **É um teto de RELÓGIO, e a conta está aqui:** a média percorre a imagem com passo, então
/// uma textura de 4096² custa o mesmo que uma de 64² — `4096` amostras, ~4 µs. Sem o passo, a
/// mesma textura custaria 16,7 M amostras (~14 ms, um quadro inteiro) **por textura**.
/// ⚠️ E o resultado é guardado por `AssetId` na [`CardArt`], então a conta corre **uma vez por
/// conteúdo**, não uma vez por quadro.
pub(crate) const SWATCH_SAMPLES: usize = 4096;

/// ⭐ **A memória do que um cartão DESENHA**, chaveada por CONTEÚDO — o que a torna reutilizável
/// entre quadros, entre entidades e depois de um undo.
///
/// ⚠️ **Ela é obrigatória, não uma optimização.** A `TextureLibrary` reescreve a entrada de cada
/// textura **a cada quadro** (é assim que um nome novo chega lá), então sem esta memória a média
/// de cor e a redução da miniatura correriam 60×/s por textura. A cor tem tecto de amostras; a
/// miniatura **não pode ter** — ela lê a imagem inteira, e é exactamente por isso que a resposta
/// se guarda.
#[derive(Default)]
pub(crate) struct CardArt {
    /// A cor dominante já calculada.
    swatches: BTreeMap<AssetId, [u8; 4]>,
    /// ⭐⭐ A miniatura já reduzida (wave A6). ⚠️ O `Arc` guardado aqui é o que faz a igualdade
    /// `O(1)` do [`Thumb`] funcionar a jusante: enquanto o conteúdo não muda, o painel recebe **o
    /// mesmo ponteiro** e não reconstrói a textura de GPU.
    thumbs: BTreeMap<AssetId, Thumb>,
}

impl CardArt {
    /// Uma memória vazia. ⚠️ **Só os gates a constroem** — o caminho do produto tem UMA, viva na
    /// sessão, e ela nasce do literal `const` do `thread_local` abaixo (uma cache que se pudesse
    /// criar em qualquer sítio seria a segunda, e a segunda não acerta em nada).
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

/// A cor do cartão, com memória por conteúdo.
pub(crate) fn swatch_for(db: &AssetDb, id: AssetId, cache: &mut CardArt) -> Option<[u8; 4]> {
    if let Some(hit) = cache.swatches.get(&id) {
        return Some(*hit);
    }
    let asset = db.get(&id)?;
    // ⭐ **Pela PORTA** (`image_rgba8`), que cobre as DUAS variantes — e isso **fecha a dívida** que
    // a 1.ª versão declarava aqui (*«uma imagem de 16 bits fica com a cor neutra»*). ⚠️ Numa imagem
    // de 8 bits ela devolve `Cow::Borrowed`, então o caminho comum não copia um byte; só a de 16
    // bits paga a conversão, e paga-a **uma vez por conteúdo** graças à memória.
    let (w, h, pixels) = asset.image_rgba8()?;
    let rgba = mean_rgba8(w, h, &pixels)?;
    cache.swatches.insert(id, rgba);
    Some(rgba)
}

/// ⭐⭐ **A miniatura do cartão, com memória por conteúdo** (wave A6).
///
/// ⚠️ **Sem tecto de amostras, ao contrário da cor — e isso é a razão da cache, não um descuido.**
/// Uma média de cor pode saltar pixels porque a resposta é UM número; uma miniatura é a forma, e
/// saltar pixels apaga exactamente o que se quer ver. ⇒ ela lê a imagem inteira **uma vez por
/// conteúdo** e a resposta fica guardada. A `TextureLibrary` reescreve a entrada a cada quadro, e
/// sem esta memória seria uma passagem completa por textura, 60×/s.
///
/// ⚠️ **Devolve sempre o MESMO `Arc` para o mesmo conteúdo** — é isso que deixa o painel decidir
/// em `O(1)` que a imagem não mudou e não reconstruir a textura de GPU. ⛔ Um `Arc` novo por quadro
/// faria o `vello` reenviar cada cartão ao atlas **todo o quadro**.
pub(crate) fn thumb_for(
    db: &AssetDb,
    id: AssetId,
    cache: &mut CardArt,
    budget: &mut u64,
) -> Option<Thumb> {
    if let Some(hit) = cache.thumbs.get(&id) {
        return Some(hit.clone());
    }
    // ⚠️ **Um acerto na memória NÃO gasta orçamento** — o teste é só para quem vai de facto
    // reduzir. Cobrar o acerto faria o painel parar de mostrar miniaturas que já existem.
    if *budget == 0 {
        return None;
    }
    let asset = db.get(&id)?;
    // ⚠️ **Aqui a porta é a `image_rgba8`, e não o `match` que a cor usa** — ela cobre as DUAS
    // variantes (a de 16 bits sai convertida), e é isso que fecha a dívida que o `swatch_for`
    // declara no `_`: *«uma imagem de 16 bits fica com a cor neutra até a A6 desenhar a miniatura
    // a sério»*. A conversão custa uma descodificação inteira, que aqui já se paga na mesma — e
    // **uma vez só**, por conteúdo.
    let (w, h, pixels) = asset.image_rgba8()?;
    if w == 0 || h == 0 || pixels.len() < (w as usize) * (h as usize) * 4 {
        return None;
    }
    // ⚠️ A conta é subtraída ANTES: a primeira redução do quadro corre sempre (`budget` chegou
    // aqui > 0), e depois dela o orçamento pode ficar a zero — que é exactamente o que se quer
    // quando uma só textura vale o quadro inteiro.
    *budget = budget.saturating_sub(u64::from(w) * u64::from(h));
    let (rgba, tw, th) = crate::thumbnail::reduce(&pixels, w, h);
    let thumb = Thumb { rgba, w: tw, h: th };
    cache.thumbs.insert(id, thumb.clone());
    Some(thumb)
}

/// A média em **luz linear**, ponderada por alfa.
///
/// ⚠️ Ponderada por alfa porque uma sprite recortada é quase toda transparente, e a média crua
/// dela é a cor do NADA (preto), não a cor do desenho. Foi isso que a primeira versão devolveu.
fn mean_rgba8(width: u32, height: u32, pixels: &[u8]) -> Option<[u8; 4]> {
    let total = (width as usize).checked_mul(height as usize)?;
    if total == 0 || pixels.len() < total * 4 {
        return None;
    }
    let stride = total.div_ceil(SWATCH_SAMPLES).max(1);
    let (mut r, mut g, mut b, mut wsum) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut alpha_sum = 0.0f64;
    let mut taken = 0.0f64;
    for i in (0..total).step_by(stride) {
        let p = &pixels[i * 4..i * 4 + 4];
        let a = f64::from(p[3]) / 255.0;
        r += srgb_to_linear(p[0]) * a;
        g += srgb_to_linear(p[1]) * a;
        b += srgb_to_linear(p[2]) * a;
        wsum += a;
        alpha_sum += a;
        taken += 1.0;
    }
    if taken == 0.0 {
        return None;
    }
    // Tudo transparente: não há cor a reportar.
    if wsum <= f64::EPSILON {
        return Some([0x50, 0x50, 0x58, 0xFF]);
    }
    Some([
        linear_to_srgb(r / wsum),
        linear_to_srgb(g / wsum),
        linear_to_srgb(b / wsum),
        // A opacidade média entra no alfa do cartão — uma textura quase vazia desenha-se quase
        // vazia, que é informação.
        ((alpha_sum / taken) * 255.0).round().clamp(0.0, 255.0) as u8,
    ])
}

pub(crate) fn dimensions(db: &AssetDb, id: AssetId) -> Option<(u32, u32)> {
    // ⚠️ **Pela PORTA, e não por um `match` na variante.** O `ph2d_asset::Asset` é
    // `#[non_exhaustive]`: um `match` aceita uma imagem de 16 bits **em silêncio** e o compilador
    // não avisa — o sintoma seria o `128x128` a sumir do cartão, sem erro nenhum.
    db.get(&id)?.image_dimensions()
}

fn srgb_to_linear(v: u8) -> f64 {
    let c = f64::from(v) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f64) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

impl CardArt {
    /// ⭐ **A memória vazia da SESSÃO** — o literal `const` que o `thread_local` da junção precisa.
    ///
    /// ⚠️ Ela existe porque os campos são privados de propósito: quem os escreve são as duas portas
    /// deste ficheiro, e expô-los faria a junção poder semear uma cor sem passar pela lei que a
    /// calcula.
    pub(crate) const EMPTY: Self = Self {
        swatches: BTreeMap::new(),
        thumbs: BTreeMap::new(),
    };
}

#[cfg(test)]
impl CardArt {
    /// Quantas cores estão na memória. ⚠️ **Só para os gates** — os campos ficam privados porque
    /// quem os escreve são as duas portas deste ficheiro, e expô-los faria a junção poder semear
    /// uma cor sem passar pela lei que a calcula.
    pub(crate) fn swatch_len(&self) -> usize {
        self.swatches.len()
    }

    /// Quantas miniaturas estão na memória. Só para os gates — ver [`Self::swatch_len`].
    pub(crate) fn thumb_len(&self) -> usize {
        self.thumbs.len()
    }
}
