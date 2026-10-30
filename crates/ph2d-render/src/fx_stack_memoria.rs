//! A memória da pilha de FX sob o escopo de memória (F40, `docs/UI_New_and_Simple/F38_plano.md`).

use super::FxStackPass;
use crate::FxOpGpu;
use ph2d_gpu::GpuContext;

impl FxStackPass {
    /// Toda a memória da corrida, sob o escopo de memória (F40). Recusada, os recursos de antes
    /// voltam (os novos nasceram inválidos) e a pilha fica em recusa até ao fim do episódio.
    pub(super) fn reserva(
        &mut self,
        gpu: &GpuContext,
        (w, h): (u32, u32),
        ops: &[FxOpGpu],
        geom: &[[f32; 4]],
        total_slots: usize,
    ) -> Option<Vec<[u32; 4]>> {
        let antes = (
            self.work.clone(),
            (self.globals.clone(), self.globals_cap),
            (self.segs.clone(), self.segs_cap),
            (self.ramps.clone(), self.ramps_cap),
        );
        let (rampas, erro) = ph2d_gpu::memoria::sob_escopo_de_memoria(&gpu.device, || {
            self.ensure_work(gpu, w, h);
            self.ensure_segments(gpu, geom);
            self.ensure_globals(gpu, total_slots);
            crate::fx_stack_res::ensure_ramps(gpu, &mut self.ramps, &mut self.ramps_cap, ops)
        });
        let recusou = erro.is_some();
        #[cfg(test)]
        let recusou = recusou || crate::renasce::testes::consome_recusa_forcada();
        if !recusou {
            return Some(rampas);
        }
        let (work, globals, segs, ramps) = antes;
        self.work = work;
        (self.globals, self.globals_cap) = globals;
        (self.segs, self.segs_cap) = segs;
        (self.ramps, self.ramps_cap) = ramps;
        self.recusa = Some(ph2d_gpu::memoria::geracao());
        None
    }

    /// Põe a pilha em recusa até ao fim do episódio — o dono recusou a SAÍDA dela (`nasce_saida`).
    pub fn anota_recusa(&mut self) {
        self.recusa = Some(ph2d_gpu::memoria::geracao());
    }

    /// `true` enquanto a placa recusou a memória da pilha neste episódio: nenhuma corrida faz
    /// trabalho de placa (a saída fica com a imagem de antes).
    #[must_use]
    pub fn em_recusa(&self) -> bool {
        self.recusa == Some(ph2d_gpu::memoria::geracao())
    }
}
