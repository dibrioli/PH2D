//! ⭐ **O solver nasce sob os dois tetos do DISPOSITIVO** (W5e): o lado de textura
//! (`max_texture_dimension_2d`, conferido em TODA textura da [`PassList`] antes de alocar — a W5d só
//! conferia a fluência, e a captura fina, 2× mais larga, dava um erro de validação a 2160p e a luz
//! saía apagada) e a MEMÓRIA, que o wgpu não diz quanta há: diz só que RECUSOU (`OutOfMemory`, do
//! driver ou do `MemoryBudgetThresholds` da instância). Recusada, a montagem sai como
//! [`ErroSolver::SemMemoria`] com os bytes que pedia — o que o governador lê.

use ph2d_gpu::GpuContext;

use super::Solver;
use crate::config::Config;
use crate::erro::ErroSolver;
use crate::passlist::PassList;

impl Config {
    /// Toda textura da lista desta configuração cabe no lado `limite` do dispositivo?
    ///
    /// # Errors
    /// [`ErroSolver::GradeGrandeDemais`] com o maior lado que não cabe.
    pub fn cabe_na_placa(&self, limite: u32) -> Result<(), ErroSolver> {
        let cfg = Self {
            grade: self.grade_efetiva(),
            ..*self
        };
        let lado = PassList::de(&cfg, false)
            .texturas
            .iter()
            .map(|t| t.tamanho[0].max(t.tamanho[1]))
            .max()
            .unwrap_or(0);
        if lado > limite {
            return Err(ErroSolver::GradeGrandeDemais { lado, limite });
        }
        Ok(())
    }
}

impl Solver {
    /// [`Self::novo`] com a [`crate::Fabrica`] partilhada: as pipelines de uma grade anterior com as
    /// mesmas constantes voltam sem recompilar (a janela redimensionada custava ~2 s por quadro).
    ///
    /// # Errors
    /// As de [`Self::novo`], e [`ErroSolver::SemMemoria`] quando o dispositivo recusa alocar.
    pub fn novo_com(
        gpu: &GpuContext,
        pedido: Config,
        fabrica: &crate::Fabrica,
    ) -> Result<Self, ErroSolver> {
        Self::nascer(gpu, pedido, fabrica, None)
    }

    /// ⭐ W6c — [`Self::novo_com`] com a FLUÊNCIA de um solver que morreu ([`Self::fluencia`]), zerada: os
    /// mesmos bits de uma nova, sem alocar outra enquanto quem a lia (os bind groups dos receptores)
    /// ainda a segura. Uma textura que não serve à configuração (outro tamanho ou formato) é largada.
    ///
    /// # Errors
    /// As de [`Self::novo_com`].
    pub fn novo_adotando(
        gpu: &GpuContext,
        pedido: Config,
        fabrica: &crate::Fabrica,
        fluencia: wgpu::Texture,
    ) -> Result<Self, ErroSolver> {
        let solver = Self::nascer(gpu, pedido, fabrica, Some(fluencia.clone()))?;
        if let Some(formato) = formato_wgsl(solver.fluencia().format())
            && *solver.fluencia() == fluencia
        {
            zerar(gpu, &fluencia, formato);
        }
        Ok(solver)
    }

    fn nascer(
        gpu: &GpuContext,
        pedido: Config,
        fabrica: &crate::Fabrica,
        adotada: Option<wgpu::Texture>,
    ) -> Result<Self, ErroSolver> {
        // A validação por fora: depois de uma recusa de memória a montagem continua e cria vistas das
        // texturas inválidas — erros que são CONSEQUÊNCIA da recusa (o gate
        // `a_luz_recua_quando_a_placa_recusa_a_memoria` viu-os escapar).
        let validacao = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let memoria = gpu.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let feito = Self::montar(gpu, pedido, fabrica, adotada);
        let sem_memoria = pollster::block_on(memoria.pop()).is_some();
        let invalido = pollster::block_on(validacao.pop());
        if let (false, Some(e)) = (sem_memoria, &invalido) {
            // Sem recusa, é um defeito: dito, e a luz apaga-se com o aviso (nunca calado).
            eprintln!("[luz2d] o dispositivo recusou a montagem do solver: {e}");
            return Err(ErroSolver::ConfigInvalida(
                "o dispositivo recusou a montagem",
            ));
        }
        match sem_memoria {
            false => feito,
            true => Err(ErroSolver::SemMemoria {
                bytes: PassList::de(
                    &Config {
                        grade: pedido.grade_efetiva(),
                        ..pedido
                    },
                    false,
                )
                .bytes_alocados(),
            }),
        }
    }
}

/// A textura `i` da lista: a ADOTADA na fluência quando serve à descrição `t`, senão uma nova.
pub(super) fn textura(
    device: &wgpu::Device,
    t: &crate::passlist::Textura,
    i: usize,
    adotada: &mut Option<wgpu::Texture>,
) -> wgpu::Texture {
    let serve = |a: &mut wgpu::Texture| {
        let tamanho = wgpu::Extent3d {
            width: t.tamanho[0],
            height: t.tamanho[1],
            depth_or_array_layers: 1,
        };
        (
            a.size(),
            a.format(),
            a.usage(),
            a.mip_level_count(),
            a.sample_count(),
        ) == (tamanho, t.formato, t.usos, t.mips, 1)
            && a.dimension() == wgpu::TextureDimension::D2
            && formato_wgsl(t.formato).is_some()
    };
    match adotada.take_if(|a| i == crate::passlist::FLUENCIA && serve(a)) {
        Some(a) => a,
        None => super::criar_textura(device, t),
    }
}

/// O formato de armazenamento da fluência no WGSL (`formato_cascata`); `None` = não se adota.
fn formato_wgsl(f: wgpu::TextureFormat) -> Option<&'static str> {
    match f {
        wgpu::TextureFormat::Rgba16Float => Some("rgba16float"),
        wgpu::TextureFormat::Rgba32Float => Some("rgba32float"),
        _ => None,
    }
}

/// Zera a fluência adotada (a nova do wgpu nasce zerada) — um passe de escrita: ela não é `COPY_DST`.
fn zerar(gpu: &GpuContext, alvo: &wgpu::Texture, formato: &str) {
    let modulo = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("luz2d zera a fluência adotada"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "@group(0) @binding(0) var alvo: texture_storage_2d<{formato}, write>;\n\
                 @compute @workgroup_size(8, 8)\n\
                 fn zerar(@builtin(global_invocation_id) id: vec3<u32>) {{\n\
                 let d = textureDimensions(alvo);\n\
                 if id.x < d.x && id.y < d.y {{ textureStore(alvo, id.xy, vec4<f32>(0.0)); }}\n\
                 }}"
                )
                .into(),
            ),
        });
    let pipeline = gpu
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("luz2d zera a fluência adotada"),
            layout: None,
            module: &modulo,
            entry_point: Some("zerar"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
    let vista = alvo.create_view(&wgpu::TextureViewDescriptor::default());
    let grupo = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("luz2d zera a fluência adotada"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&vista),
        }],
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("luz2d zera a fluência adotada"),
        });
    {
        let mut passe = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luz2d zera a fluência adotada"),
            timestamp_writes: None,
        });
        passe.set_pipeline(&pipeline);
        passe.set_bind_group(0, &grupo, &[]);
        let s = alvo.size();
        passe.dispatch_workgroups(s.width.div_ceil(8), s.height.div_ceil(8), 1);
    }
    gpu.queue.submit([enc.finish()]);
}
