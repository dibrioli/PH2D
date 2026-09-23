//! ⭐⭐⭐ **BAIXAR AS SAÍDAS** (doc 119 W3) — o fim do cozimento: cada saída que o plano encenou
//! vira instâncias, uma a seguir à outra, no MESMO buffer, e a partição de texturas/misturas cobre
//! o buffer inteiro.
//!
//! Partido do `lib.rs` (a 6 linhas do tecto) ao longo da costura que lá estava: o laço dos
//! estágios produz CORRENTES; daqui para a frente é *como as correntes viram desenho*.
//!
//! ⚠️ **Com UMA saída isto é o caminho de sempre, byte a byte**: deslocamento `0`, a partição
//! procurada pelo comprimento entre todas as fronteiras, a contagem devolvida é a da corrente.

use crate::plan::GpuPlan;
use crate::stream::GpuStream;
use crate::{GpuCook, GpuCookError, tex_runs};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::graph::NodeId;
use ph2d_render::{GpuTexRun, SinkStyle};
use std::collections::BTreeMap;

impl GpuCook {
    /// Baixa as [`GpuPlan::sinks`] (um estilo por saída, na mesma ordem) e devolve a contagem
    /// TOTAL das correntes das saídas — com uma saída, a da corrente dela, como sempre.
    ///
    /// ⛔⛔ **A reserva do total vem ANTES da primeira escrita.** Crescer o buffer substitui-o (ver
    /// [`GpuCook::encode_lowering`]), logo uma saída que o crescesse a meio apagaria, em silêncio,
    /// as que já escreveram — *uma cena a desenhar só a última saída, sem erro nenhum*.
    ///
    /// ⚠️ **O deslocamento de cada saída é o que FOI escrito**, não a soma das contagens: uma saída
    /// que a lei do dono cala (o `veredito_do_dispositivo`) não escreve nada, e somar a contagem
    /// dela deixaria um buraco de lixo entre as vizinhas.
    #[allow(clippy::too_many_arguments)] // o fim do cook: o plano, as correntes e os defaults dele
    pub(crate) fn baixar_as_saidas(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        plan: &GpuPlan,
        streams: &BTreeMap<NodeId, GpuStream>,
        boundary_streams: &[(NodeId, &Stream)],
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
        styles: &[SinkStyle],
    ) -> Result<u32, GpuCookError> {
        // Uma saída que a placa não encena não está em `sinks` — e o caso de UMA saída nesse
        // estado é o de sempre: nenhuma corrente, e o lowering corre sobre a vazia no
        // deslocamento `0` (garante o buffer e deixa-o com `len = 0`).
        let saidas: Vec<GpuStream> = if plan.sinks.is_empty() {
            vec![GpuStream::default()]
        } else {
            plan.sinks
                .iter()
                .map(|s| streams.get(s).cloned().unwrap_or_default())
                .collect()
        };
        if styles.len() != saidas.len() {
            return Err(GpuCookError::SinkStyleMismatch {
                sinks: saidas.len(),
                styles: styles.len(),
            });
        }
        let total: u64 = saidas.iter().map(|s| u64::from(s.count)).sum();
        // The instance buffer is the one binding that can outgrow the device's
        // storage-binding limit below the id ceiling (184 B × count; every
        // stream column caps at 16 B × ID_WRAP ≈ 268 MB). Refuse BEFORE the
        // bind group turns it into a validation panic. ⚠️ Sobre o TOTAL: é um buffer só.
        let instance_bytes = total * std::mem::size_of::<ph2d_render::RenderInstance>() as u64;
        // wgpu 29: `max_storage_buffer_binding_size` is already `u64`.
        let binding_limit = gpu.device.limits().max_storage_buffer_binding_size;
        if instance_bytes > binding_limit {
            return Err(GpuCookError::BindingTooLarge {
                bytes: instance_bytes,
                limit: binding_limit,
            });
        }
        let total = u32::try_from(total).map_err(|_| GpuCookError::BindingTooLarge {
            bytes: instance_bytes,
            limit: binding_limit,
        })?;
        if saidas.len() > 1 {
            self.ensure_instance_capacity(gpu, total.max(1));
        }

        let uma_so = saidas.len() == 1;
        let mut partes: Vec<(u32, u32, Vec<GpuTexRun>)> = Vec::with_capacity(saidas.len());
        for (k, (stream, &style)) in saidas.iter().zip(styles).enumerate() {
            let primeiro = if k == 0 {
                0
            } else {
                self.instances.as_ref().map_or(0, |i| i.len)
            };
            // A 1.ª baixa ocupa a vaga de sempre (`stages.len()`); as seguintes vão PARA LÁ da
            // faixa das compactações (`stages.len() + 1 + stage_idx`, até `2·stages.len()`), porque
            // duas escritas na mesma vaga chegam ambas à última (`write_buffer` é à submissão).
            let vaga = if k == 0 {
                plan.stages.len()
            } else {
                2 * plan.stages.len() + k
            };
            let escreveu = self.encode_lowering(
                gpu,
                encoder,
                vaga,
                stream,
                primeiro,
                default_uv_rect,
                default_size,
                style,
            );
            debug_assert!(
                escreveu,
                "a reserva do total vem antes da 1.a escrita, logo nenhuma baixa recusa"
            );
            let escrito = self
                .instances
                .as_ref()
                .map_or(0, |i| i.len.saturating_sub(primeiro));
            // A partição DESTA saída. Com uma saída, a lei de sempre (a fronteira acha-se pelo
            // comprimento, e a contagem é a da corrente); com várias, só a fronteira da linhagem
            // dela — duas do mesmo comprimento seriam indistinguíveis — e só o que foi escrito.
            let mut runs = Vec::new();
            if uma_so {
                tex_runs::texture_runs_from_boundary(
                    boundary_streams,
                    stream.count,
                    style.blend,
                    style.sampling,
                    &mut runs,
                );
            } else {
                let dela = plan.sinks.get(k).and_then(|&s| plan.lineage_boundary(s));
                let fronteira: Vec<(NodeId, &Stream)> = boundary_streams
                    .iter()
                    .filter(|(n, _)| Some(*n) == dela)
                    .map(|(n, s)| (*n, *s))
                    .collect();
                tex_runs::texture_runs_from_boundary(
                    &fronteira,
                    escrito,
                    style.blend,
                    style.sampling,
                    &mut runs,
                );
            }
            partes.push((primeiro, escrito, runs));
        }
        self.tex_runs.clear();
        juntar_as_particoes(&partes, &mut self.tex_runs);
        Ok(total)
    }
}

/// ⭐⭐ **As partições de N saídas numa só, que cubra o buffer inteiro** (doc 119 W3).
///
/// Vazia é, para o desenho, *«o buffer inteiro no átlas, em `Mix`, com o sampler do projecto»* (o
/// ramo `runs.is_empty()` do `renderer_draw`). ⇒ se NENHUMA saída pediu um run, fica vazia (o
/// caminho de sempre); se UMA pediu, **todas** passam a ter runs — a que tinha a partição vazia
/// ganha o run explícito de átlas em `Mix` sobre a faixa dela. ⛔ Sem isso, os runs de uma saída
/// fariam o desenho saltar a outra inteira: o ramo dos runs só desenha o que os runs cobrem.
pub(crate) fn juntar_as_particoes(partes: &[(u32, u32, Vec<GpuTexRun>)], out: &mut Vec<GpuTexRun>) {
    if partes.iter().all(|(_, _, r)| r.is_empty()) {
        return;
    }
    for (primeiro, escrito, runs) in partes {
        if runs.is_empty() {
            if *escrito > 0 {
                out.push(GpuTexRun {
                    texture_id: 0,
                    start: *primeiro,
                    end: primeiro + escrito,
                    blend: 0,
                    sampling: 0,
                });
            }
            continue;
        }
        out.extend(runs.iter().map(|r| GpuTexRun {
            start: r.start + primeiro,
            end: r.end + primeiro,
            ..*r
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(t: u32, a: u32, b: u32, blend: u8) -> GpuTexRun {
        GpuTexRun {
            texture_id: t,
            start: a,
            end: b,
            blend,
            sampling: 0,
        }
    }

    /// Nenhuma saída pede um run ⇒ a partição fica vazia (o caminho de sempre).
    #[test]
    fn sem_runs_em_nenhuma_saida_a_particao_fica_vazia() {
        let mut out = Vec::new();
        juntar_as_particoes(&[(0, 3, vec![]), (3, 4, vec![])], &mut out);
        assert!(out.is_empty());
    }

    /// ⭐⭐ Uma saída pede runs ⇒ a outra ganha o run de átlas sobre a faixa dela, e os runs da
    /// primeira são DESLOCADOS para a faixa dela. Sem o preenchimento, o desenho saltaria a
    /// saída de partição vazia inteira.
    #[test]
    fn uma_saida_com_runs_obriga_as_outras_a_terem_run() {
        let mut out = Vec::new();
        juntar_as_particoes(
            &[
                (0, 3, vec![]),
                (3, 4, vec![run(0, 0, 4, 1)]),
                (7, 0, vec![]),
            ],
            &mut out,
        );
        assert_eq!(out, vec![run(0, 0, 3, 0), run(0, 3, 7, 1)]);
        // A cobertura é o buffer inteiro, sem buracos nem sobreposições.
        let mut fim = 0;
        for r in &out {
            assert_eq!(r.start, fim, "buraco ou sobreposição em {r:?}");
            fim = r.end;
        }
        assert_eq!(fim, 7);
    }
}
