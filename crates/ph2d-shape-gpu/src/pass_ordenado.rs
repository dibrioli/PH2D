//! ⭐ doc 132 §4.1 (3) do Motion — **o desenho ORDENADO** das corridas `Normal` com as duas médias
//! ([`super::mistura::Corrida::ordenada`]): o segundo meio que o chamador regista e o `first_vertex`
//! que leva o deslocamento entre as faixas (`ordenado.wgsl`). Saiu do `pass.rs` pelo tecto de LOC.

use ph2d_gpu::GpuContext;

use super::{SHADER, ShapePass, mistura};

/// O viés do deslocamento entre as faixas de uma corrida ordenada, no `first_vertex` (`ordenado.wgsl`).
const VIES_DO_SEGUNDO: i64 = 1 << 28;

/// ⭐ doc 132 §4.1 (3) — **o `first_vertex` do desenho ordenado** de uma corrida cujas faixas começam
/// em `copias` (as formas) e `imagens` (o segundo meio): `6 · (imagens − copias + VIES)`. `None` se o
/// deslocamento não cabe (mais de `2²⁸` linhas — buffers de dezenas de GB).
#[must_use]
pub fn primeiro_vertice_ordenado(copias: u32, imagens: u32) -> Option<u32> {
    let d = i64::from(imagens) - i64::from(copias);
    (d.abs() < VIES_DO_SEGUNDO)
        .then(|| u32::try_from(6 * (d + VIES_DO_SEGUNDO)).ok())
        .flatten()
}

impl ShapePass {
    /// ⭐⭐ doc 132 §4.1 (3) — **o segundo meio das corridas ordenadas** ([`Corrida::ordenada`]): o
    /// WGSL dele (o contrato está em `ordenado.wgsl`, com os bindings no grupo `2`) e o layout desse
    /// grupo. O desenho de uma corrida ordenada liga os grupos `0` e `1` e entrega o passe ao segundo
    /// desenhador de [`Self::draw_em_corridas_com`], que liga o `2` e desenha com
    /// [`primeiro_vertice_ordenado`].
    pub fn com_segundo_meio(
        &mut self,
        gpu: &GpuContext,
        meio: &str,
        layout_do_meio: &wgpu::BindGroupLayout,
    ) {
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-shape-gpu (ordenado)"),
            source: wgpu::ShaderSource::Wgsl(Self::modulo_ordenado(meio).into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-shape-gpu (ordenado)"),
            bind_group_layouts: &[
                Some(&self.layout),
                Some(&self.contorno.leitura),
                Some(layout_do_meio),
            ],
            immediate_size: 0,
        });
        self.ordenado = Some(crate::contorno::Variantes::cria(&[], |opcoes| {
            mistura::pipeline_com_vertice(
                device,
                &pl,
                &module,
                ("vs_ordenado", "fs_ordenado"),
                mistura::Mistura::Normal,
                self.formato,
                opcoes,
            )
        }));
    }

    /// Um segundo meio já se registou ([`Self::com_segundo_meio`]).
    #[must_use]
    pub fn tem_segundo_meio(&self) -> bool {
        self.ordenado.is_some()
    }

    /// O WGSL inteiro do desenho ordenado com o `meio` — para o gate que o valida sem placa.
    #[must_use]
    pub fn modulo_ordenado(meio: &str) -> String {
        format!("{SHADER}{}{meio}", include_str!("ordenado.wgsl"))
    }
}
