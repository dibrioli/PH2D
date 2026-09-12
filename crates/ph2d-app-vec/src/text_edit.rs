//! **A SESSÃO de texto em curso** (`DrawMode::Text`) — o TIPO. As acções que a conduzem ficam na
//! shell (`vec_text.rs`, `impl App`), que é quem tem a cena e a câmera.
//!
//! ⚠️ **Desceu da shell em 2026-09-12** (`line/render-loop`, A9 da auditoria de arquitectura): a
//! `App` guardava-o num campo solto (`vec_text_edit`), e um campo cujo TIPO mora na shell não pode
//! juntar-se ao [`crate::state::VecState`]. O tipo é só dados
//! (`Paint`/`StrokeSpec`/`VecPathId`/`TextAlign`), então descer não levou lei nenhuma consigo.

use ph2d_vec_scene::{Paint, StrokeSpec, VecPathId};
use ph2d_vec_text::TextAlign;

/// Uma sessão de digitação de texto no canvas (`DrawMode::Text`). O texto vive como
/// UM `VecPath` compound na cena (campo [`Self::id`]); esta struct guarda o ponto de
/// inserção + os parâmetros, re-cozinhando o compound a cada tecla/mudança. Ao
/// finalizar, o objeto de texto permanece (Live Shape) — a sessão só some.
pub struct VecTextEdit {
    /// Baseline da PRIMEIRA linha em MUNDO (o ponto do clique). A geometria nasce na
    /// baseline local [0,0] e é centrada no local 0 (o pivô); o `Transform` da
    /// entidade = `origin + center`, então a baseline fica no clique e o pivô no centro.
    pub origin: [f64; 2],
    /// Tamanho em unidades de world.
    pub size: f64,
    /// Peso da fonte variável (eixo `wght`, ex. 100..900) aplicado ao contorno.
    pub weight: f32,
    /// Entrelinha como múltiplo do tamanho (leading).
    pub line_height: f64,
    /// Espaçamento entre glyphs como fração do tamanho (tracking, em).
    pub tracking: f64,
    /// Alinhamento horizontal do bloco (L/C/R) em relação à origem.
    pub align: TextAlign,
    /// Valores dos eixos de variação da fonte ALÉM do peso (opsz/wdth/slnt/…), na
    /// ordem que a fonte expõe (`vec_font::variation_axes`). Casa índice-a-índice com
    /// os campos da seção Axes do painel; reseedado quando a família muda.
    pub extra_axes: Vec<(ph2d_vector_font::AxisTag, f32)>,
    /// Família de fonte escolhida (`None` = a InterVariable embutida). Resolvida em
    /// `VariableFont` por `vec_font::resolve` a cada regen.
    pub family: Option<String>,
    /// Preenchimento dos glyphs (do Style do painel; `None` = sem fill).
    pub fill: Option<Paint>,
    /// Traço dos glyphs (do Style: cor/largura/cap/join/dash), como nas formas.
    pub stroke: Option<StrokeSpec>,
    /// Conteúdo digitado.
    pub text: String,
    /// A largura da caixa a que o texto REFLUI, em unidades de mundo. `None` = sem refluxo.
    /// Espelha o campo homónimo do [`ph2d_ecs::VecTextParams`] — a sessão é a face VIVA do
    /// componente, e as duas viajam pelo mesmo par `text_params`/`reopen_text_session`.
    pub wrap_width: Option<f64>,
    /// O ÚNICO `VecPath` compound do texto vivo na cena (todos os glyphs num path só —
    /// um objeto). `None` enquanto não há geometria (string vazia). Atualizado
    /// IN-PLACE a cada mudança para o id — e a entidade + o `VecShape` — ficarem
    /// estáveis (sem churn de despawn/respawn a cada tecla).
    pub id: Option<VecPathId>,
    /// Centro da bbox do layout (coords relativas à baseline, ANTES de centrar) — o
    /// deslocamento que centra a geometria no local 0. O `Transform` da entidade =
    /// `origin + center`, então a baseline fica no clique. Recalculado a cada regen.
    pub center: [f64; 2],
}
