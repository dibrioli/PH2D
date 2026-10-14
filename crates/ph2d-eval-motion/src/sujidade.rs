//! **A SUJIDADE DA BOMBA** — irmão por RESPONSABILIDADE (HR-18) do [`crate::MotionCookPump`]: quem
//! marca o próximo cozimento, e a contagem de cada marca.

use super::*;

impl MotionCookPump {
    /// Force a re-cook on the next [`Self::pump`], even at the same tick (call
    /// after editing the graph while paused). The M1 graph-edit path. Also
    /// **invalidates the scrub cache**: the recorded checkpoints are the sim of
    /// the OLD graph, so a later scrub must re-sim from the tick-0 seed under the
    /// edited graph (Blender/Houdini "edit invalidates the cache").
    pub fn mark_dirty(&mut self) {
        self.mark_dirty_keeping_ring();
        self.edicoes += 1;
        self.ring.clear();
    }

    /// ⭐⭐⭐ **Marca sujo SEM deitar fora o anel de scrub** — para quem precisa de re-cozinhar por
    /// uma razão que NÃO é uma edição do documento.
    ///
    /// O [`Self::mark_dirty`] existe para uma EDIÇÃO: o grafo mudou, logo tudo o que o anel
    /// guardava descreve outro documento e tem de sair. O LOD da forma
    /// ([`ph2d_app_motion::motion_shape_lod`]) precisa do contrário — o documento é o MESMO e o
    /// que mudou foi a CÂMARA: ele esvazia o lado crisp ao particionar, e sem um re-cozimento no
    /// quadro seguinte não há instâncias para ele mudar de ideias sobre, logo APROXIMAR nunca
    /// devolveria o desenho.
    ///
    /// ⚠️ **Usar o `mark_dirty` ali seria limpar o anel a cada quadro em que o LOD está armado** —
    /// e o doc do [`Self::is_dirty`] nomeia esse defeito por escrito (*«a stray `mark_dirty` …
    /// would re-cook the whole graph every frame of the gesture»*). O anel é o cache do scrub:
    /// deitá-lo fora por uma razão que não o invalidou faz o scrub seguinte cozinhar do zero.
    pub fn mark_dirty_keeping_ring(&mut self) {
        self.dirty = true;
        self.sujidades += 1;
    }

    /// Whether the next [`Self::pump`] will re-cook. Exposed so an edit that must
    /// NOT re-cook can prove it: the editor's decoration (the graph panel's group
    /// backdrops) is document state but cannot change what the graph cooks, and a
    /// stray `mark_dirty` on a backdrop drag would re-cook the whole graph every
    /// frame of the gesture. A test asserts this stays `false` across such an edit.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// **Quantas vezes o próximo cozimento foi marcado** — por uma edição, pelo LOD, pela camada
    /// isolada. Nenhuma rota a desfaz: a placa coze sem marchar a bomba e deixa o [`Self::is_dirty`]
    /// como estava, e é por esta contagem que uma macro gravada numa rota se confere na outra
    /// (ADR-0186).
    #[must_use]
    pub fn sujidades(&self) -> u64 {
        self.sujidades
    }
}
