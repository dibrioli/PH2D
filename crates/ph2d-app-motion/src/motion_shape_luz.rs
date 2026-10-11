//! ⭐ **As formas do Motion na luz 2D** (W5e, `docs/Luz2D/01_plano.md` §2.4) — um objeto Motion com
//! `BloqueiaLuz` faz as formas dele bloquearem a luz: as MESMAS cópias que a placa desenha neste
//! quadro (do buffer do dispositivo, ou as da CPU), num passe irmão do dela (a mesma variante e o
//! mesmo contorno, no formato da camada da luz), com o afim da grade da luz. A cobertura do passe é a de área (o alfa da forma).
//!
//! ⚠️ A placa não sabe de que objeto é cada cópia: um objeto Motion que bloqueia faz bloquear TODAS as
//! formas da placa (aberto no handoff W5e). Sem a placa (a rota do Vello) não há captura.

use ph2d_gpu::GpuContext;
use ph2d_light2d_gpu::CamadaDeOclusao;
use ph2d_shape_gpu::{Copias, ShapeView};

use super::{GeometriasDaPlaca, PlacaDeFormas, contorno_por_ordem};

/// As formas do Motion do quadro, para a luz.
pub struct OclusaoDoMotion<'a> {
    pub placa: &'a mut PlacaDeFormas,
    pub geometrias: &'a GeometriasDaPlaca,
    /// O buffer das cópias no dispositivo (o cozimento vivo), se é dele que a placa lê.
    pub dispositivo: Option<&'a wgpu::Buffer>,
    /// As entidades (`to_bits`) dos objetos Motion (`MotionGenerator`).
    pub objetos: Vec<u64>,
}

impl<'a> OclusaoDoMotion<'a> {
    /// O Motion deste quadro: a placa já decidida, o estado do Motion e o mundo (os objetos).
    #[must_use]
    pub fn do_quadro(
        placa: &'a mut PlacaDeFormas,
        motion: &'a crate::motion_state::MotionState,
        world: &ph2d_ecs::World,
    ) -> Self {
        let objetos = world
            .try_query_filtered::<ph2d_ecs::Entity, ph2d_ecs::With<ph2d_ecs::MotionGenerator>>()
            .map(|mut q| q.iter(world).map(|e| e.to_bits()).collect())
            .unwrap_or_default();
        Self {
            placa,
            geometrias: &motion.placa_geometrias,
            dispositivo: motion
                .gpu_cook
                .formas()
                .filter(|_| motion.gpu_live)
                .map(|f| f.buffer()),
            objetos,
        }
    }
}

impl CamadaDeOclusao for OclusaoDoMotion<'_> {
    fn desenhar(
        &mut self,
        gpu: &GpuContext,
        alvo: &wgpu::TextureView,
        tamanho: (u32, u32),
        mundo: [f64; 4],
        quem: &[u64],
    ) -> bool {
        if !self.objetos.iter().any(|b| quem.contains(b)) {
            return false;
        }
        let p = &mut *self.placa;
        if !p.ativa {
            return false;
        }
        if p.luz.is_none() {
            let variante = if p.variante.constantes.is_empty() {
                ph2d_shape_gpu::VarianteDoPasse::do_ambiente()
            } else {
                p.variante.clone()
            };
            let mut passe = ph2d_shape_gpu::ShapePass::da_variante(
                gpu,
                ph2d_light2d_gpu::FORMATO_DA_OCLUSAO,
                &variante,
            );
            p.afina(&mut passe);
            p.luz = Some(passe);
        }
        let Some(passe) = p.luz.as_mut() else {
            return false;
        };
        passe.set_geometries(gpu, self.geometrias.prontas());
        passe.com_contorno(contorno_por_ordem() && !p.sem_contorno);
        let (buffer, count) = match p.do_dispositivo {
            Some(n) => match self.dispositivo {
                Some(b) => (b.clone(), n),
                None => return false,
            },
            None => {
                passe.upload_instances(gpu, &p.copias);
                match (passe.uploaded(), u32::try_from(p.copias.len())) {
                    (Some(b), Ok(n)) => (b.clone(), n),
                    _ => return false,
                }
            }
        };
        if count == 0 {
            return false;
        }
        // Mundo → píxel do alvo: x para a direita, y para BAIXO a partir do topo (`mundo[3]`).
        let [x0, _, x1, y1] = mundo;
        let k = f64::from(tamanho.0) / (x1 - x0);
        #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
        #[expect(clippy::cast_precision_loss, reason = "um alvo cabe num f32")]
        let vista = ShapeView {
            lin: [k as f32, 0.0, 0.0, -k as f32],
            t: [(-x0 * k) as f32, (y1 * k) as f32],
            alvo: [tamanho.0.max(1) as f32, tamanho.1.max(1) as f32],
        };
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("luz2d/formas_do_motion"),
            });
        passe.draw(
            gpu,
            &mut enc,
            alvo,
            wgpu::LoadOp::Load,
            vista,
            Copias {
                buffer: &buffer,
                count,
            },
        );
        gpu.queue.submit([enc.finish()]);
        true
    }
}

#[cfg(test)]
#[path = "motion_shape_luz_tests.rs"]
mod testes;
