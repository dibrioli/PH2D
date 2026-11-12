//! **O envio sem cópia dos planos da luz** — a dobra escreve nos buffers de envio mapeados e a luz copia-os para
//! as texturas no mesmo submit (filho de [`super`], LOC cap).

use super::{ImpastoLightError, ImpastoLightInput, ImpastoLightPass};
use ph2d_gpu::GpuContext;

/// Os planos, pela ordem de [`PlanosEmEnvio`]: relevo, cobertura, mat0, mat1, papel.
pub(super) const N_PLANOS: usize = 5;
pub(super) const PAPEL: usize = 4;
/// Bytes por texel de cada plano (R32F, R8, RGBA8, RGBA8, R32F).
const TEXEL: [u32; N_PLANOS] = [4, 1, 4, 4, 4];
/// Buffers livres guardados por plano: o do quadro em voo e o do anterior, a voltar mapeado.
const ENVIOS_GUARDADOS: usize = 2;

/// Um buffer de envio (`MAP_WRITE | COPY_SRC`) e se ele está mapeado, pronto a escrever.
pub(super) struct Envio {
    buffer: wgpu::Buffer,
    pronto: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

/// **Os planos de uma janela na memória de envio** — a dobra escreve AQUI, e a luz copia daqui para as texturas
/// (`copy_buffer_to_texture` no mesmo submit). Sem `Vec` intermédio nem o `memcpy` do `write_texture`, que no
/// wgpu-core 30 corre com o `pending_writes` preso (14 ms no fio da UI no Ctrl+Z 4K).
///
/// Nasce de [`ImpastoLightPass::planos_para_envio`], escreve-se por faixas ([`Self::faixa`], em paralelo) e entrega-se a
/// [`ImpastoLightPass::run_enviado`].
pub struct PlanosEmEnvio {
    pub(super) region: crate::layer_compositor::Region,
    pub(super) envios: [Option<(Envio, u32)>; N_PLANOS],
}

/// As vistas mapeadas de uma faixa de linhas da janela, uma por plano. Nasce e morre no fio que a escreve
/// ([`PlanosEmEnvio::faixa`]): só a vista, nunca a memória, e só de escrita (pode ser write-combining).
pub struct FaixaDeEnvio {
    vistas: [Option<(wgpu::BufferViewMut, usize, usize)>; N_PLANOS],
}

impl FaixaDeEnvio {
    /// Escreve a linha `j` da faixa nos cinco planos (`papel` só quando foi pedido em
    /// [`ImpastoLightPass::planos_para_envio`]); o padding do `stride` não se toca.
    ///
    /// # Panics
    ///
    /// Uma linha que não tem a largura da janela, ou um `papel` que não foi pedido.
    pub fn escreve(
        &mut self,
        j: usize,
        relief: &[f32],
        cover: &[u8],
        mat0: &[u8],
        mat1: &[u8],
        papel: Option<&[f32]>,
    ) {
        let fontes: [Option<&[u8]>; N_PLANOS] = [
            Some(bytemuck::cast_slice(relief)),
            Some(cover),
            Some(mat0),
            Some(mat1),
            papel.map(bytemuck::cast_slice),
        ];
        for (destino, fonte) in self.vistas.iter_mut().zip(fontes) {
            match (destino, fonte) {
                (Some((v, stride, row)), Some(f)) => {
                    v.slice(j * *stride..j * *stride + *row).copy_from_slice(f);
                }
                (None, None) => {}
                _ => panic!("o plano do papel foi pedido ≠ entregue"),
            }
        }
    }
}

impl PlanosEmEnvio {
    /// As vistas das linhas `linhas` (relativas à janela) em cada plano mapeado. Faixas disjuntas podem abrir-se
    /// ao mesmo tempo em fios diferentes (o `Buffer` é `Sync` e as vistas não se sobrepõem).
    #[must_use]
    pub fn faixa(&self, linhas: std::ops::Range<u32>) -> FaixaDeEnvio {
        let w = self.region.w;
        FaixaDeEnvio {
            vistas: std::array::from_fn(|k| {
                let (e, stride) = self.envios[k].as_ref()?;
                let s = u64::from(*stride);
                let vista = e
                    .buffer
                    .slice(u64::from(linhas.start) * s..u64::from(linhas.end) * s)
                    .get_mapped_range_mut()
                    .expect("o envio está mapeado: nasce mapeado ou só volta à reserva quando o mapa chega");
                Some((vista, *stride as usize, (w * TEXEL[k]) as usize))
            }),
        }
    }
}

impl ImpastoLightPass {
    /// **Os buffers de envio de uma janela**, mapeados, para a dobra escrever direto neles (o `relief`, o `cover`…
    /// das fatias de [`ImpastoLightInput`] ficam vazios e [`Self::run_enviado`] copia daqui). Reaproveita um
    /// buffer livre já mapeado de tamanho bastante; senão cria um (`mapped_at_creation`).
    pub fn planos_para_envio(
        &mut self,
        gpu: &GpuContext,
        plane_region: crate::layer_compositor::Region,
        com_papel: bool,
    ) -> PlanosEmEnvio {
        use std::sync::atomic::Ordering;
        let envios = std::array::from_fn(|k| {
            if k == PAPEL && !com_papel {
                return None;
            }
            let stride = (plane_region.w * TEXEL[k]).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let need = u64::from(stride) * u64::from(plane_region.h);
            let livres = &mut self.envios_livres[k];
            let achado = livres
                .iter()
                .position(|e| e.pronto.load(Ordering::Acquire) && e.buffer.size() >= need);
            let envio = achado.map_or_else(
                || Envio {
                    buffer: gpu.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("ph2d-render impasto_light envio"),
                        size: need,
                        usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: true,
                    }),
                    pronto: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
                },
                |i| livres.swap_remove(i),
            );
            Some((envio, stride))
        });
        PlanosEmEnvio {
            region: plane_region,
            envios,
        }
    }

    /// [`Self::run`] com os planos já escritos em `envio` ([`Self::planos_para_envio`]); as fatias de planos de
    /// `input` não são lidas. Copia-os para as texturas no mesmo submit da luz e devolve os buffers à reserva,
    /// a remapear.
    ///
    /// # Errors
    ///
    /// Os de [`Self::run`]; `PlaneSize` se a janela do envio não é `input.plane_region` ou falta o papel.
    pub fn run_enviado(
        &mut self,
        gpu: &GpuContext,
        src: &wgpu::Texture,
        input: &ImpastoLightInput<'_>,
        envio: PlanosEmEnvio,
    ) -> Result<&wgpu::Texture, ImpastoLightError> {
        self.run_com(gpu, src, input, Some(envio))
    }

    /// Desmapeia os buffers de envio e grava a cópia de cada um para a JANELA da sua textura.
    pub(super) fn copy_planes(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        envio: PlanosEmEnvio,
    ) -> [Option<Envio>; N_PLANOS] {
        let c = self.canvas.as_ref().expect("just ensured");
        let r = envio.region;
        let alvos = [&c.relief, &c.cover, &c.mat0, &c.mat1, &c.papel];
        let mut envios = envio.envios;
        std::array::from_fn(|k| {
            let (e, stride) = envios[k].take()?;
            e.buffer.unmap();
            encoder.copy_buffer_to_texture(
                wgpu::TexelCopyBufferInfo {
                    buffer: &e.buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(stride),
                        rows_per_image: Some(r.h),
                    },
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &alvos[k].texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: r.x,
                        y: r.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: r.w,
                    height: r.h,
                    depth_or_array_layers: 1,
                },
            );
            Some(e)
        })
    }

    /// Depois do submit: cada buffer volta à reserva e pede o mapa de novo; fica `pronto` quando a cópia acaba.
    /// Guarda os [`ENVIOS_GUARDADOS`] maiores por plano (o do Ctrl+Z da tela inteira serve a qualquer janela).
    pub(super) fn devolve(&mut self, enviados: [Option<Envio>; N_PLANOS]) {
        use std::sync::atomic::Ordering;
        for (k, e) in enviados.into_iter().enumerate() {
            let Some(e) = e else { continue };
            e.pronto.store(false, Ordering::Release);
            let pronto = std::sync::Arc::clone(&e.pronto);
            e.buffer
                .slice(..)
                .map_async(wgpu::MapMode::Write, move |r| {
                    pronto.store(r.is_ok(), Ordering::Release);
                });
            let livres = &mut self.envios_livres[k];
            livres.push(e);
            if livres.len() > ENVIOS_GUARDADOS {
                let menor = (0..livres.len())
                    .min_by_key(|&i| livres[i].buffer.size())
                    .expect("não vazio");
                livres.swap_remove(menor);
            }
        }
    }
}
