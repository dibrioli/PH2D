//! ⭐⭐⭐ **UMA FORMA ENCODADA UMA VEZ, CARIMBADA `N` VEZES** — a porta que serve o carimbo do
//! Motion (`N` cópias da MESMA geometria, uma pose e uma tinta cada).
//!
//! ## O que já estava pago, e o que sobrava
//!
//! A porta de lote da `ph2d-vec-render` (`draw_shared_instances`) já **tessela** cada geometria
//! distinta UMA vez — é o congelamento das 160 k estrelas. O que sobrava era o **encode**: cada
//! cópia voltava a percorrer o `BezPath` elemento a elemento dentro do `Scene::fill`, e isso é
//! `O(segmentos)` por cópia, todos os quadros.
//!
//! ## A lei: o caminho é escrito em coordenadas LOCAIS
//!
//! ⭐ No Vello a pose de uma forma **não** entra no caminho: o `Scene::fill` escreve a
//! transformação num fluxo (`transforms`), o estilo noutro (`styles`), o caminho em mais dois
//! (`path_tags` / `path_data`) e o pincel noutros dois (`draw_tags` / `draw_data`). ⇒ **os bytes do
//! caminho são os mesmos para todas as cópias**, e a única coisa que muda por cópia é a
//! transformação e a cor.
//!
//! É isso que faz [`PreparedFill`] ser exacto e não uma aproximação: ela guarda os bytes que o
//! `encode_shape` escreveria, e o [`VectorScene::fill_prepared`] reproduz **passo a passo** o que o
//! `Scene::fill` faz — `encode_transform` · `encode_fill_style` · o caminho · `encode_brush` —, só
//! que o terceiro passo é um `extend_from_slice` em vez de um percurso.
//!
//! ⚠️ **A prova não é este parágrafo: é o gate.** `o_carimbo_preparado_escreve_os_MESMOS_bytes`
//! encoda `N` cópias pelas duas rotas e compara os **seis fluxos** mais os dois contadores. *Uma
//! rota rápida que produza outra imagem não é uma optimização, é um segundo produto* — e um desvio
//! de um byte num fluxo do Vello não dá erro nenhum: dá outro desenho.
//!
//! ⛔ **Fronteira DECLARADA:** com a feature `bump_estimate` do Vello ligada, o `Scene::fill`
//! alimenta também um estimador de memória de GPU, e esta porta **não o alimenta**. A `ph2d-vector`
//! pede o `vello` com `default-features = false` (`Cargo.toml`), logo a feature está desligada e o
//! caminho não existe — mas quem a ligar tem de voltar aqui, e é por isso que isto está escrito.

use crate::VectorScene;
use vello::kurbo::{Affine, BezPath};
use vello::peniko::{Brush, Fill};
use vello_encoding::{Encoding, PathTag, Transform};

/// **Uma forma já encodada** — os dois fluxos de caminho que o Vello escreve, mais a contabilidade
/// que ele leva junto. Construa-a UMA vez por geometria e carimbe-a com
/// [`VectorScene::fill_prepared`].
pub struct PreparedFill {
    /// O fluxo de tags do caminho, **sem** as marcas de transformação e de estilo (elas não são do
    /// caminho: são escritas por cópia).
    tags: Vec<PathTag>,
    /// O fluxo de coordenadas.
    data: Vec<u32>,
    /// Quantos segmentos o caminho tem — a contabilidade que o `PathEncoder::finish` acumula.
    segments: u32,
    /// Quantos caminhos ele conta como (um, salvo caminho vazio).
    paths: u32,
    /// A regra de preenchimento, que é do caminho e não da cópia.
    fill: Fill,
}

impl PreparedFill {
    /// Encoda a forma **uma vez**, num `Encoding` de rascunho.
    ///
    /// ⚠️ O rascunho não leva transformação nem estilo de propósito: os dois escrevem uma marca no
    /// `path_tags`, e uma marca colhida aqui seria escrita **duas** vezes no carimbo.
    #[must_use]
    pub fn new(shape: &BezPath, fill: Fill) -> Self {
        let mut rascunho = Encoding::new();
        rascunho.encode_shape(shape, true);
        Self {
            tags: rascunho.path_tags,
            data: rascunho.path_data,
            segments: rascunho.n_path_segments,
            paths: rascunho.n_paths,
            fill,
        }
    }

    /// `true` quando a forma não tem um único segmento — o mesmo caso em que o `Scene::fill`
    /// **não** encoda o pincel.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments == 0
    }
}

impl VectorScene {
    /// **Carimba uma forma já preparada** com esta pose e esta tinta.
    ///
    /// Reproduz o `Scene::fill` (com `brush_transform = None`) passo a passo; ver o cabeçalho do
    /// módulo e o gate que o afirma em bytes.
    pub fn fill_prepared(&mut self, forma: &PreparedFill, transform: Affine, brush: &Brush) {
        let enc = self.inner_mut().encoding_mut();
        enc.encode_transform(Transform::from_kurbo(&transform));
        enc.encode_fill_style(forma.fill);
        if forma.is_empty() {
            // O `encode_shape` devolve `false` aqui e o `Scene::fill` NÃO encoda o pincel — a
            // transformação e o estilo ficam, que é o que ele deixa. Reproduzir isto importa: é o
            // que mantém os fluxos alinhados quando uma geometria degenerada entra no lote.
            return;
        }
        enc.path_tags.extend_from_slice(&forma.tags);
        enc.path_data.extend_from_slice(&forma.data);
        enc.n_paths += forma.paths;
        enc.n_path_segments += forma.segments;
        enc.encode_brush(brush, 1.0);
    }
}

#[cfg(test)]
#[path = "scene_prepared_tests.rs"]
mod tests;
