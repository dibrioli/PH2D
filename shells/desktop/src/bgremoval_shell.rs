//! ⭐⭐⭐ **O ESTADO DE SHELL da Remoção de fundo, numa casa só** — o molde dos `*Live` do vetor.
//!
//! ⛔⛔ **Ele nasceu porque a `App` estourou o tecto de campos** (2026-09-15), e a catraca diz a
//! cura por escrito: *«um campo novo tem DONO: ponha-o no estado da família do assunto dele»*.
//! Aqui eram **cinco** campos soltos com o mesmo prefixo — a prova de que a família já existia e
//! ninguém lhe tinha dado casa. ⇒ `189 → 185` campos, e o tecto desce com eles.
//!
//! ⚠️ **O que ele NÃO é:** o estado da FERRAMENTA. Tudo aqui é o que a **shell** guarda *sobre* a
//! ferramenta entre quadros — a cache de CPU da prévia, as duas ranhuras de GPU, a instância que o
//! passe de sprites desenha e o memo de quem foi empurrado. O documento e os parâmetros vivem na
//! `ph2d_tool_bgremoval::BgRemovalTool`, do outro lado do downcast (ADR-0040 §3).

use crate::app_state::{BgremovalPreview, BgremovalPreviewGpu};

/// Ver o cabeçalho do módulo.
#[derive(Default)]
pub(crate) struct BgremovalShell {
    /// `entity_bits` da sprite cujo RGBA foi empurrado por último para o snapshot da ferramenta.
    /// `None` até ao primeiro empurrão; reposto a `None` quando o artista (re)activa a ferramenta,
    /// para o quadro seguinte re-empurrar contra a selecção de agora. O laço lê-o a cada quadro
    /// para saltar empurrões redundantes (a miniatura e a re-corrida do pipeline são trabalho).
    pub(crate) last_pushed_entity: Option<u64>,
    /// A prévia de resolução inteira, em CPU. Recalculada só quando os params, a fonte ou a
    /// activação mudam; o quadro reusa o `Arc` (sem cópia de pixels). ⚠️ **Não é uma edição
    /// destrutiva** — a textura da sprite fica intacta, e é por isso que o Apply (que relê a fonte
    /// original) e o undo continuam certos.
    pub(crate) preview: Option<BgremovalPreview>,
    /// A ranhura de GPU que a prévia ocupa (Lens F, 2026-05-26). O `sim_extract` emite um
    /// `PreviewOverride` que **troca a textura da MESMA instância** ⇒ a prévia passa pelo mesmo
    /// `sprite.wgsl` que o Apply (paridade byte a byte) **e herda a malha** de uma arte presa ao
    /// esqueleto, de graça.
    pub(crate) preview_gpu: Option<BgremovalPreviewGpu>,
    /// ⭐⭐⭐ **A ranhura da TINTA da máscara de protecção** (2026-09-15) — gémea da de cima.
    ///
    /// ⛔⛔ Ela nasceu porque a tinta era desenhada pelo **Vello com o afim do quad de repouso**:
    /// sobre uma arte presa ao esqueleto e dobrada, a prévia (deformada) e a tinta que a anota
    /// apareciam em sítios DIFERENTES. O caminho por um recorte do Vello por triângulo está medido
    /// e **refutado** (costuras numa arte translúcida, que dilatar os recortes **piora**; e os
    /// buffers do Vello degradam em SILÊNCIO) — ver o doc da
    /// `render_loop::bgremoval_preview_gpu::upload_tint`.
    pub(crate) tint_gpu: Option<BgremovalPreviewGpu>,
    /// A tinta como instância do passe de sprites, **com a malha da arte**. Vazia sem prévia.
    /// ⚠️ Vive aqui, e não numa `Vec` local, pela razão dos vizinhos (HR-3): é lixo de quadro sobre
    /// uma lista quase sempre vazia.
    pub(crate) tint_extra: ph2d_render::LiftedInstances,
}
