//! **Fase do quadro: OS CATAVENTOS** — a rota B (`docs/3D/02.2`) do lado do quadro.
//!
//! ⚠️ **Ela corre DEPOIS da irmã [`super::fase_relight_baked_forms`], e a ordem é LEI:** um
//! catavento é também um objecto assado (a matéria e o slot vêm do `BakedForm`), logo a irmã
//! também o vê. Quem escreve por último ganha o slot do sprite, e a rota B tem de ser essa.
//! ⭐ E a porta **carimba** o `lit_with` do assado com o rig que acabou de usar, o que faz a irmã
//! saltá-lo no quadro seguinte — *o carimbo diz a verdade literal, não um remendo*.
//!
//! ⛔ **Atrás da feature, ao contrário da irmã:** a rota A promete acender **sem** o módulo 3D no
//! build (a forma dela viaja no documento); esta RASTERIZA por quadro, logo precisa da malha.
//! O corpo vive na [`ph2d_app_sculpt3d::vivo_fase`] — aqui está a composição e nada mais.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    #[cfg(feature = "sculpt3d")]
    pub(super) fn fase_cataventos(&mut self) {
        // ⚠️ Lido ANTES do empréstimo do `gfx` — o `FrameGfx::of` toma-o inteiro.
        let segundos = self.playhead.time() as f32;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sculpt3d,
            baked_forms,
            formas_vivas,
            baked_light,
            surface,
            renderer,
            sim,
            ..
        } = FrameGfx::of(gfx);

        // Sem cena 3D armada não há malha para rasterizar — e sem `Mesh3D` no mundo a porta varre
        // um mapa vazio e devolve zeros, sem tocar a placa.
        let Some(scene) = sculpt3d.as_mut() else {
            return;
        };
        let conta = ph2d_app_sculpt3d::vivo_fase::acende_os_cataventos(
            scene,
            baked_forms,
            formas_vivas,
            ph2d_app_sculpt3d::vivo_fase::Bancada {
                gpu: surface.gpu(),
                renderer,
                passes: baked_light,
            },
            sim,
            segundos,
        );
        // ⚠️ O readout é do DIAGNÓSTICO e não do quadro: ele imprime **só** quando alguma coisa
        // mudou de estado, senão seriam 60 linhas por segundo (o defeito que a auto-conferência do
        // HUD pagou). A CONTA das rasterizações é o que torna a economia do carimbo defensável.
        if conta.rasterizados > 0 || conta.largadas > 0 {
            eprintln!(
                "[catavento] acesos={} rasterizados={} largados={}",
                conta.acesos, conta.rasterizados, conta.largadas
            );
        }
    }
}
