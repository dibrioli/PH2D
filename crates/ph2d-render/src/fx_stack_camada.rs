//! **A pilha sobre um RETÂNGULO da camada** (ciclo 20 W4 do Motion, doc 129 §5.7): a camada
//! filtrada do Motion tem o tamanho da JANELA, mas só o retângulo da CENA se vê (os painéis pintam
//! por cima do resto). A 4K a pilha na janela inteira custava `2,01 ms` e no retângulo da cena
//! `0,77 ms` — medido por `relogio_cada_forma_contra_camada_inteira` (`ph2d-fx-cells`).
//!
//! A cura só vale se o que se VÊ não muda: o retângulo cresce pelo que a pilha LÊ e ESPALHA em volta
//! de um pixel (um disco fora da cena projecta a sombra para dentro dela), e a origem alinha-se à
//! grelha da pirâmide do borrão, que a camada inteira põe no pixel `(0, 0)`. Gate:
//! `ph2d-render/tests/it/fx_stack_camada_rect_gpu.rs`.

use crate::fx_stack::{FxOpGpu, FxStackPass, Origem, Plan};
use crate::fx_stack_plan::{plan_of, stack_reach, stack_read};
use ph2d_gpu::GpuContext;

/// **O retângulo `[x, y, w, h]` em que a pilha `ops` tem de correr** para que o retângulo `cena`
/// saia igual ao da camada inteira `janela`: alargado pelo que a pilha lê mais o que espalha, com
/// a origem descida ao múltiplo de `2^niveis` da maior pirâmide, e preso à janela.
#[must_use]
pub fn retangulo_da_camada(cena: [u32; 4], janela: (u32, u32), ops: &[FxOpGpu]) -> [u32; 4] {
    let (l, t, r, b) = stack_reach(ops);
    let margem = stack_read(ops) + l.max(t).max(r).max(b);
    let passo = ops
        .iter()
        .map(|op| match plan_of(op, true) {
            Plan::Blur { niveis } => 1u32 << niveis,
            _ => 1,
        })
        .max()
        .unwrap_or(1);
    let x0 = cena[0].saturating_sub(margem) / passo * passo;
    let y0 = cena[1].saturating_sub(margem) / passo * passo;
    let x1 = (cena[0] + cena[2] + margem).min(janela.0);
    let y1 = (cena[1] + cena[3] + margem).min(janela.1);
    [
        x0.min(x1),
        y0.min(y1),
        x1.saturating_sub(x0),
        y1.saturating_sub(y0),
    ]
}

impl FxStackPass {
    /// **A camada pré-multiplicada do Motion, só no retângulo `rect`** de `src` — o [`Self::run_layer`]
    /// lido a partir de `(rect.x, rect.y)`, com o resultado no canto de `dst` (que tem pelo menos
    /// `rect.w × rect.h`). `Origem::Ponto` está em pixels de `src` e é trazido para o retângulo.
    #[allow(clippy::too_many_arguments)]
    pub fn run_layer_rect(
        &mut self,
        gpu: &GpuContext,
        src: &wgpu::Texture,
        rect: [u32; 4],
        dst: &wgpu::Texture,
        ops: &[FxOpGpu],
        origem: Origem<'_>,
    ) {
        #[allow(clippy::cast_precision_loss)]
        let origem = match origem {
            Origem::Ponto([px, py]) => Origem::Ponto([px - rect[0] as f32, py - rect[1] as f32]),
            o => o,
        };
        #[allow(clippy::cast_possible_wrap)]
        let src_org = [rect[0] as i32, rect[1] as i32];
        self.run_with(
            gpu,
            src,
            src_org,
            dst,
            rect[2],
            rect[3],
            ops,
            &[],
            true,
            origem,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn borrao(sigma_px: f32) -> FxOpGpu {
        FxOpGpu {
            sigma_px,
            ..FxOpGpu::resolve(
                &ph2d_fx_op::FxOp::new(ph2d_fx_op::FxOp::BLUR),
                [1.0, 0.0, 0.0, 1.0],
            )
        }
    }

    #[test]
    fn without_ops_the_rect_is_the_scene() {
        assert_eq!(
            retangulo_da_camada([100, 50, 300, 200], (1600, 972), &[]),
            [100, 50, 300, 200]
        );
    }

    #[test]
    fn the_rect_grows_by_what_the_stack_reads_and_spreads_and_stays_in_the_window() {
        let ops = [borrao(4.0)];
        let m = stack_read(&ops) + stack_reach(&ops).0;
        assert!(m > 0);
        assert_eq!(
            retangulo_da_camada([100, 50, 300, 200], (1600, 972), &ops),
            [100 - m, 50 - m, 300 + 2 * m, 200 + 2 * m]
        );
        assert_eq!(
            retangulo_da_camada([0, 0, 1600, 972], (1600, 972), &ops),
            [0, 0, 1600, 972]
        );
    }

    #[test]
    fn a_pyramid_blur_aligns_the_origin_to_its_grid() {
        let ops = [borrao(100.0)];
        let Plan::Blur { niveis } = plan_of(&ops[0], true) else {
            panic!("um borrão")
        };
        assert!(niveis > 0);
        let r = retangulo_da_camada([1000, 900, 300, 200], (3840, 2160), &ops);
        let passo = 1u32 << niveis;
        assert_eq!((r[0] % passo, r[1] % passo), (0, 0), "{r:?}");
        assert!(r[0] + stack_read(&ops) <= 1000 && r[0] + r[2] >= 1300);
    }
}
