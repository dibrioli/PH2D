//! A água na PLACA dentro do worker (degraus A+B, docs/Painter/49_realshader/LIGACAO_PLACA.md).
//!
//! Todo caminho que lê ou escreve o motor passa por `bring_home`, isto é, o worker devolve-o: por isso
//! a placa só é dona do estado enquanto o motor está COM o worker. Ao receber: sobe as linhas que a CPU
//! mudou (hash por linha contra o espelho); ao devolver: desce as linhas da união das caixas (e as bordas).
//! Sem `PH2D_WET_GPU=1` = a CPU de sempre, ao byte (nada aqui corre).

use ph2d_wet_paint::painter::Engine;

#[cfg(feature = "gpu")]
static CONTEXTO: std::sync::OnceLock<ph2d_gpu::GpuContext> = std::sync::OnceLock::new();

impl crate::PainterTool {
    /// O device do Painter (o do app) para a água do Wet Paint; a 1.ª chamada vence.
    #[cfg(feature = "gpu")]
    pub fn instala_placa_do_wet(gpu: &ph2d_gpu::GpuContext) {
        let _ = CONTEXTO.set(gpu.clone());
    }
}

/// `PH2D_WET_GPU` lido UMA vez. ⚠️ Por ora só `1` liga a placa (o gate placa × CPU no tool ainda não
/// correu — LIGACAO_PLACA.md); a omissão passa a placa quando ele estiver verde.
pub(super) fn ligada() -> bool {
    static L: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *L.get_or_init(|| std::env::var("PH2D_WET_GPU").is_ok_and(|v| v.trim() == "1"))
}

/// O estado da placa de UM worker.
#[derive(Default)]
pub(super) struct Placa {
    #[cfg(feature = "gpu")]
    passo: Option<ph2d_wet_gpu::passo::PassoPlaca>,
    /// O motor actual está na placa (decidido ao receber; um traço nunca troca a meio).
    viva: bool,
}

#[cfg_attr(not(feature = "gpu"), allow(unused_variables))]
impl Placa {
    /// O motor chegou: decide placa × CPU e, se placa, sobe a grade.
    #[cfg_attr(not(feature = "gpu"), allow(clippy::needless_return))] // o `return` cobre o bloco `#[cfg(feature = "gpu")]` a seguir
    pub(super) fn recebe(&mut self, e: &mut Engine) {
        self.viva = false;
        if !ligada() {
            return;
        }
        #[cfg(feature = "gpu")]
        {
            let Some(gpu) = CONTEXTO.get() else { return };
            let idx = e.active_layer;
            if !ph2d_wet_gpu::passo::placa_sabe(&e.sim, &e.layers[idx].grid, &e.tuning) {
                return;
            }
            // A história: o passo da placa não sabe a caixa de antemão — marca a grade inteira.
            let _ = e.active_grid_mut();
            let g = &e.layers[idx].grid;
            match &mut self.passo {
                Some(p) if p.serve(g) => {
                    let _ = p.sobe_sujo(gpu, g);
                }
                _ => {
                    self.passo = Some(ph2d_wet_gpu::passo::PassoPlaca::new(
                        gpu, g, &e.sim, &e.tuning,
                    ))
                }
            }
            self.viva = true;
        }
    }

    /// UM passo inteiro na placa: `Some(ran)`, ou `None` = a CPU anda (estágio a estágio).
    pub(super) fn passo(&mut self, e: &mut Engine) -> Option<bool> {
        if !self.viva {
            return None;
        }
        #[cfg(feature = "gpu")]
        {
            let gpu = CONTEXTO.get()?;
            let p = self.passo.as_mut()?;
            let idx = e.active_layer;
            let g0 = &e.layers[idx].grid;
            let pre = g0.has_fluid.then_some((g0.bx0, g0.by0, g0.bx1, g0.by1));
            let Engine {
                sim,
                layers,
                tuning,
                ..
            } = e;
            let ran = p.passo(gpu, sim, &mut layers[idx].grid, tuning);
            if let Some((x0, y0, x1, y1)) = pre {
                e.merge_dirty(x0, y0, x1, y1);
            }
            let g = &e.layers[idx].grid;
            if g.has_fluid {
                let (x0, y0, x1, y1) = (g.bx0, g.by0, g.bx1, g.by1);
                e.merge_dirty(x0, y0, x1, y1);
            }
            return Some(ran);
        }
        #[allow(unreachable_code)]
        None
    }

    /// O motor vai voltar à UI: desce as linhas que a placa pode ter mudado.
    #[cfg_attr(not(feature = "gpu"), allow(clippy::needless_return))] // o `return` cobre o bloco `#[cfg(feature = "gpu")]` a seguir
    pub(super) fn devolve(&mut self, e: &mut Engine) {
        if !std::mem::take(&mut self.viva) {
            return;
        }
        #[cfg(feature = "gpu")]
        if let (Some(gpu), Some(p)) = (CONTEXTO.get(), &mut self.passo) {
            let idx = e.active_layer;
            p.desce_suja(gpu, &mut e.layers[idx].grid);
        }
    }
}
