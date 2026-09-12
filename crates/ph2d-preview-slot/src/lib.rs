//! **A RANHURA DE TEXTURA que segura a pré-visualização viva de uma ferramenta no canvas.**
//!
//! Uma ferramenta de raster (remoção de fundo, Painter, equalização de cor, upscale) mostra o
//! resultado *antes* de o aplicar. O lado da CPU disso é o `ph2d_tool_runtime::PreviewCache`; o
//! lado da GPU é a [`PreviewGpu`] daqui — a ranhura transitória de
//! `ph2d_render::IndividualTextureStore` para onde a ponte repete esses pixels, mais a escrituração
//! que lhe diz se o que está lá em cima ainda é o que a ferramenta tem.
//!
//! # ⛔⛔ Por que uma FOLHA, e não os dois sítios que pareciam óbvios
//!
//! Este tipo chamava-se `BgremovalPreviewGpu` e era uma `struct` em
//! `shells/desktop/src/app_state.rs`, **por inércia** — escrito pela remoção de fundo, que era da
//! shell, e depois emprestado ao Painter por um `type PainterPreviewGpu = BgremovalPreviewGpu`. O
//! doc dele já dizia *«tool-agnostic — shared by BgRemoval and the Painter»* por escrito. Quando a
//! família `painter` saiu para a `ph2d-app-painter` (W2 Fase D) ele era a âncora que prendia
//! **nove** ficheiros ao `app_state.rs`.
//!
//! Os dois destinos naturais foram **medidos e recusados**, e nenhum por gosto:
//!
//! | destino | por que parecia certo | por que NÃO |
//! |---|---|---|
//! | `ph2d-tool-runtime` | é onde o gémeo de CPU (`PreviewCache`) vive | tem um **teto de LOC de 650** e está em `633`. O gate diz-se *«the discipline mechanism»* contra esta crate virar god-crate, manda *«push the new logic back into the bridge»* e escreve que **a próxima subida exige ADR**. Os 17 restantes só chegariam apagando os comentários de campo — e o do `arc_token` **regista uma medição** (porquê um token e não um ponteiro). *Apagar prosa medida para caber num teto é gamar a disciplina que o teto existe para impor.* |
//! | `ph2d-render`, ao lado do `IndividualTextureStore` | o `texture_id` é uma ranhura DAQUELE store | o `individual.rs` está em **708** linhas com a folga do censo em **709** — uma linha. E a crate é o *renderer*: o `arc_token` e o `entity_bits` são escrituração da PONTE, não vocabulário de desenho. |
//!
//! ⇒ **uma folha por ASSUNTO** (`ESTADO_W2` §2), e o assunto diz-se sem a palavra *«comum»*: *a
//! ranhura onde a pré-visualização de uma ferramenta vive enquanto o artista decide*.
//!
//! ⚠️ **A `shells/desktop` mantém os dois `type`** (`BgremovalPreviewGpu`, `PainterPreviewGpu`)
//! como alias para cá, e é isso que deixa os ficheiros da remoção de fundo — e toda a prosa que os
//! cita — byte a byte iguais.

#![forbid(unsafe_code)]

/// **O companheiro do lado da GPU de um `ph2d_tool_runtime::PreviewCache`.**
///
/// Possui a ranhura transitória de `ph2d_render::IndividualTextureStore` que segura a
/// pré-visualização viva de uma ferramenta no canvas. **Agnóstico da ferramenta** — os campos não
/// carregam estado de nenhuma. O cache de CPU é a fonte de verdade (`Arc`-partilhado com o
/// `current_preview` da ferramenta); a ponte repete-o sobre esta textura sempre que o buffer do
/// `Arc` é trocado.
#[derive(Copy, Clone, Debug)]
pub struct PreviewGpu {
    /// Renderer-assigned id (a ranhura no `IndividualTextureStore`).
    pub texture_id: u32,
    /// Source-pixel width of the texture currently uploaded. Used to
    /// detect resize → `replace_pixels` will rebuild the entry.
    pub width: u32,
    /// Source-pixel height of the texture currently uploaded.
    pub height: u32,
    /// Opaque change token of the pixels most recently uploaded, `usize` so the
    /// struct stays `Send + Sync` — never dereferenced. A different value in the
    /// live cache means new pixels → re-upload; `0` means the slot was NOT
    /// CPU-seeded (the GPU producer's stamp), which forces the next CPU frame to a
    /// full upload. BgRemoval fills it with `Arc::as_ptr(rgba)`; the **Painter**
    /// fills it with the tool's monotonic `canvas_version()` — because keying on a
    /// pointer forced the shell to hold a clone of the live canvas, and holding
    /// that clone made `stamp_dabs`' `Arc::make_mut` copy the whole canvas every
    /// move (the CPU-bound FPS drop on a big canvas). A version lets the shell own
    /// its preview buffer and leave the tool sole owner of its canvas.
    pub arc_token: usize,
    /// Entity whose source produced the uploaded pixels. Used as a
    /// belt-and-suspenders check alongside `arc_token` so a
    /// coincidental token reuse can't paint the wrong sprite.
    pub entity_bits: u64,
}
