//! ⭐ **As formas do Motion na luz 2D** (W5e, `docs/Luz2D/01_plano.md` §2.4) — um objeto Motion com
//! `BloqueiaLuz` faz as formas dele bloquearem a luz: as MESMAS cópias que a placa desenha neste
//! quadro (do buffer do dispositivo, ou as da CPU), num passe irmão do dela (a mesma variante e o
//! mesmo contorno, no formato da camada da luz), com o afim da grade da luz. A cobertura do passe é a de área (o alfa da forma).
//!
//! ⭐ W5j: cada cópia tem DONO — a faixa do sink na lista que a placa desenha, e o objeto Motion dono
//! do sink; só as faixas dos objetos que a luz pediu bloqueiam. Sem a placa (a rota do Vello) não há
//! captura.

use std::ops::Range;

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
    /// `(entidade do objeto Motion dono, as cópias dele)` na lista que a placa desenha ([`donos`]).
    pub donos: Vec<(u64, Range<u32>)>,
}

impl<'a> OclusaoDoMotion<'a> {
    /// O Motion deste quadro: a placa já decidida e o estado do Motion (os donos vivos).
    #[must_use]
    pub fn do_quadro(
        placa: &'a mut PlacaDeFormas,
        motion: &'a crate::motion_state::MotionState,
    ) -> Self {
        let donos = donos(motion, placa);
        Self {
            placa,
            geometrias: &motion.placa_geometrias,
            dispositivo: motion
                .gpu_cook
                .formas()
                .filter(|_| motion.gpu_live)
                .map(|f| f.buffer()),
            donos,
        }
    }
}

/// ⭐ **O dono de cada cópia** da placa: `(entidade, cópias)` por sink com dono vivo, pela ordem da
/// lista — no dispositivo, as faixas do cozido; na CPU, a faixa `vetores` do sink (já depois do LOD)
/// lida pelo índice de origem de cada cópia ([`PlacaDeFormas::fonte`]).
#[must_use]
pub fn donos(
    motion: &crate::motion_state::MotionState,
    placa: &PlacaDeFormas,
) -> Vec<(u64, Range<u32>)> {
    let dono = |sink| crate::motion_objeto::origem::dono_do_no(motion, sink).map(|d| d.entity);
    if placa.do_dispositivo.is_some() {
        let (_, formas) = motion.gpu_cook.faixas_dos_sinks();
        return formas
            .iter()
            .filter_map(|(sink, r)| Some((dono(*sink)?, r.clone())))
            .collect();
    }
    let em = |k: usize| {
        let k = u32::try_from(k).unwrap_or(u32::MAX);
        u32::try_from(placa.fonte.partition_point(|&i| i < k)).unwrap_or(u32::MAX)
    };
    motion
        .pump
        .faixas
        .iter()
        .filter_map(|f| Some((dono(f.sink)?, em(f.vetores.start)..em(f.vetores.end))))
        .collect()
}

/// As faixas dos objetos em `quem`, cortadas a `n` e juntas quando se tocam.
fn pedidas(donos: &[(u64, Range<u32>)], quem: &[u64], n: u32) -> Vec<Range<u32>> {
    let mut r: Vec<Range<u32>> = donos
        .iter()
        .filter(|(e, _)| quem.contains(e))
        .map(|(_, r)| r.start.min(n)..r.end.min(n))
        .filter(|r| !r.is_empty())
        .collect();
    r.sort_by_key(|r| r.start);
    let mut juntas: Vec<Range<u32>> = Vec::with_capacity(r.len());
    for f in r {
        match juntas.last_mut() {
            Some(u) if f.start <= u.end => u.end = u.end.max(f.end),
            _ => juntas.push(f),
        }
    }
    juntas
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
        if !self.donos.iter().any(|(e, _)| quem.contains(e)) {
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
        let pedidas = pedidas(&self.donos, quem, count);
        if pedidas.is_empty() {
            return false;
        }
        // Nem todas pedidas: só as faixas delas, numa lista à parte (a CPU sobe-as; no dispositivo, uma
        // cópia de placa para placa).
        let (buffer, count) = if pedidas[..] == [0..count] {
            (buffer, count)
        } else if p.do_dispositivo.is_none() {
            let so: Vec<_> = pedidas
                .iter()
                .flat_map(|r| p.copias[r.start as usize..r.end as usize].iter().copied())
                .collect();
            passe.upload_instances(gpu, &so);
            match (passe.uploaded(), u32::try_from(so.len())) {
                (Some(b), Ok(n)) => (b.clone(), n),
                _ => return false,
            }
        } else {
            let n: u32 = pedidas.iter().map(|r| r.end - r.start).sum();
            let passo = std::mem::size_of::<ph2d_shape_gpu::ShapeInstance>() as u64;
            let bytes = u64::from(n) * passo;
            if p.luz_copias.as_ref().is_none_or(|b| b.size() < bytes) {
                p.luz_copias = Some(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("luz2d/formas_do_motion/pedidas"),
                    size: bytes.next_power_of_two(),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
            }
            let destino = p.luz_copias.clone().expect("criado acima");
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            let mut em = 0_u64;
            for r in &pedidas {
                let tam = u64::from(r.end - r.start) * passo;
                enc.copy_buffer_to_buffer(&buffer, u64::from(r.start) * passo, &destino, em, tam);
                em += tam;
            }
            gpu.queue.submit([enc.finish()]);
            (destino, n)
        };
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
