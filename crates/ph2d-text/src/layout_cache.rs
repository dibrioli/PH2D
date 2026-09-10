//! ⭐⭐⭐ **A CACHE DE LAYOUTS MOLDADOS** — a chave, o tecto, e o que acontece ao cruzá-lo.
//!
//! Irmã do [`super::system`] por assunto, e o tecto de LOC foi o gatilho: aquele ficheiro
//! responde *como se molda um texto* e este *o que já foi moldado*. ⚠️ **A rotação é o miolo do
//! ficheiro** — ver o doc de [`LAYOUT_CACHE_CAP`], que traz a medição de 2026-09-10.

use parley::Layout;
use std::collections::BTreeMap;

/// Cache key for a shaped layout. The layout is fully determined by
/// text, size, wrap width, weight, and letter-spacing (the font stack
/// and opsz variation are fixed per `TextSystem` / derived from
/// `font_size`), so these fields are an exact identity — no collision
/// risk (unlike a hashed key). f32s are stored as raw bits so the key
/// is `Eq + Ord` (workspace clippy bans `HashMap` per ADR-0022, so this
/// is a `BTreeMap` key).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LayoutCacheKey {
    pub(crate) text: String,
    pub(crate) font_size_bits: u32,
    pub(crate) max_width_bits: u32,
    pub(crate) weight_bits: u32,
    pub(crate) letter_spacing_px_bits: u32,
}

/// ⭐⭐⭐ **O tecto de UMA GERAÇÃO da cache de layouts** — e o que acontece ao cruzá-lo é a
/// metade que importa.
///
/// # ⛔⛔ O penhasco que isto cura, medido em 2026-09-10
///
/// A cache era **deitada fora inteira** ao transbordar (`clear()`), com o argumento — escrito
/// aqui — de que texto sempre a mudar (um campo a ser escrito) gera chaves únicas e encheria a
/// memória. O argumento está certo para **esse** caso e é catastrófico para o outro: quando o
/// **conjunto de trabalho de um quadro** passa o tecto, o `clear` acontece **a meio do quadro**,
/// tudo o que vem depois falha, volta a encher, volta a transbordar — e o quadro seguinte molda
/// **tudo outra vez, para sempre**.
///
/// Medido no app com todos os painéis abertos (`shells/desktop`, debug):
///
/// | painéis | conteúdo | moldagens em REGIME | ms/quadro |
/// |---|---|---|---|
/// | 24 | 775 textos distintos | **0** | `4,87` |
/// | 26 | 1 110 textos distintos | **1 109** | `157,91` |
///
/// ⚠️ **`+43 %` de conteúdo, `29×` de relógio** — e a saída desenhada cresceu só `1,5×`. *Um
/// custo que não aparece no que se vê não é volume: é trabalho deitado fora.*
///
/// # ⭐ A cura é a ROTAÇÃO, e não um número maior
///
/// Subir o tecto move o penhasco um painel para a frente e deixa-o lá — *um limite legítimo diz
/// de que recurso ele é* (`CLAUDE.md` §0.0), e este é de **memória**. ⇒ ao encher, a geração
/// quente **roda** para [`TextSystem::layout_cold`] e uma nova nasce vazia; um falhanço consulta
/// a fria e **promove**. O residente fica entre `CAP` e `2 × CAP`, um conjunto de trabalho até
/// `2 × CAP` acerta ~sempre, e **nada é perdido de uma vez** — cruzar o tecto passa a custar uma
/// promoção, não a moldagem do ecrã inteiro.
///
/// ⚠️ **O preço é MEMÓRIA e está declarado:** no pior caso guardam-se `2 × CAP` layouts moldados
/// em vez de `CAP`. É esse o recurso que este número mede, e é por isso que ele continua a
/// existir. // LITERAL-OK: cache budget
pub const LAYOUT_CACHE_CAP: usize = 1024;

/// ⭐⭐ **Duas gerações de layouts moldados, e um contador de moldagens.**
///
/// ⚠️ **`get` é `&mut` de propósito:** um acerto na geração FRIA **promove** para a quente, e é
/// essa metade que faz a rotação valer alguma coisa. *Sem a promoção, rodar é um `clear` com
/// mais passos* — e há mutação a prová-lo.
#[derive(Default)]
pub(crate) struct LayoutCache {
    hot: BTreeMap<LayoutCacheKey, Layout<()>>,
    cold: BTreeMap<LayoutCacheKey, Layout<()>>,
    shapes: u64,
}

impl LayoutCache {
    /// O layout deste texto, se alguma das duas gerações o tiver.
    pub(crate) fn get(&mut self, key: &LayoutCacheKey) -> Option<Layout<()>> {
        if !self.hot.contains_key(key)
            && let Some(warm) = self.cold.remove(key)
        {
            self.hot.insert(key.clone(), warm);
        }
        self.hot.get(key).cloned()
    }

    /// Guarda um layout acabado de moldar, **rodando** a geração quando ela enche.
    pub(crate) fn insert(&mut self, key: LayoutCacheKey, layout: Layout<()>) {
        if self.hot.len() >= LAYOUT_CACHE_CAP {
            self.cold = core::mem::take(&mut self.hot);
        }
        self.hot.insert(key, layout);
    }

    /// Marca que um texto foi MOLDADO — o instrumento do penhasco.
    pub(crate) fn note_shape(&mut self) {
        self.shapes = self.shapes.saturating_add(1);
    }

    pub(crate) fn shapes(&self) -> u64 {
        self.shapes
    }

    /// O residente das duas gerações.
    pub(crate) fn len(&self) -> usize {
        self.hot.len() + self.cold.len()
    }

    /// Quantos a geração QUENTE guarda — a régua dos testes de identidade de chave.
    ///
    /// ⚠️ `cfg(test)`: ele mede a geração de dentro, que **só** um teste de identidade de
    /// chave quer saber. O produto pergunta o residente ([`Self::len`]), e expor as duas ao
    /// produto seria oferecer duas respostas a *«quanto a cache guarda?»*.
    #[cfg(test)]
    pub(crate) fn hot_len(&self) -> usize {
        self.hot.len()
    }
}
