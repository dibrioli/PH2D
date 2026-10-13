//! ⭐⭐ **AS IMAGENS DE UMA SAÍDA QUE MISTURA EM GRUPO** (P4, doc 132 §4 do Motion) — as linhas que o
//! baixamento das sprites escreveu para uma saída `Add`/`Multiply`/`Screen` entre as cópias, tiradas
//! do buffer das sprites e entregues à placa de formas, que as mistura com o MESMO alcance e no
//! MESMO tom das formas (`ph2d-app-motion`, `motion_shape_placa_imagens`).
//!
//! Até aqui (B6) o passe de sprites desenhava-as — luz linear, sem alcance — e uma vigia contava-as na
//! placa para devolver a saída à CPU um a dois quadros DEPOIS. ⇒ a decisão já não depende de valores
//! que só a placa vê: a saída em grupo vai sempre ao dispositivo, e cada linha dela desenha-se pela
//! média que a mistura sabe compor.
//!
//! A separação corre DEPOIS do passe do dono (a origem de cada objecto, que reescreve o buffer no
//! sítio) e ANTES do passe de sprites: cópia das faixas para o buffer da placa e `clear_buffer` delas
//! no das sprites — uma linha toda a zero tem tamanho `0` e não pinta nada. As linhas de FORMA que a
//! faixa também tem já vinham caladas (tamanho e opacidade `0`), e no buffer da placa continuam mudas.

use crate::GpuCook;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::NodeId;
use std::ops::Range;

/// As saídas em grupo e o que a última separação deixou.
#[derive(Default)]
pub(crate) struct Grupo {
    /// Uma por [`crate::GpuPlan::sinks`], na mesma ordem. Vazio = nenhuma (o caminho de sempre).
    saidas: Vec<bool>,
    imagens: Option<GpuImagensDoGrupo>,
}

impl Grupo {
    /// A saída `k` mistura em grupo.
    pub(crate) fn em_grupo(&self, k: usize) -> bool {
        self.saidas.get(k).copied().unwrap_or(false)
    }
}

/// As linhas de imagem das saídas em grupo do último quadro: `[RenderInstance; len]`, a faixa de
/// cada sink nelas e a textura de cada troço (o bind da CPU, como o [`ph2d_render::GpuTexRun`]).
pub struct GpuImagensDoGrupo {
    buffer: wgpu::Buffer,
    capacity: u32,
    len: u32,
    faixas: Vec<(NodeId, Range<u32>)>,
    texturas: Vec<(Range<u32>, u32)>,
}

impl GpuImagensDoGrupo {
    /// O buffer (`STORAGE`), com as linhas na disposição do [`ph2d_render::RenderInstance`].
    #[must_use]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    /// Quantas linhas a última separação copiou.
    #[must_use]
    pub fn len(&self) -> u32 {
        self.len
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// A faixa de cada sink em grupo neste buffer, na ordem do das sprites.
    #[must_use]
    pub fn faixas(&self) -> &[(NodeId, Range<u32>)] {
        &self.faixas
    }
    /// `(linhas, texture_id)` — a textura de cada troço, pela partição das sprites.
    #[must_use]
    pub fn texturas(&self) -> &[(Range<u32>, u32)] {
        &self.texturas
    }
}

/// **A textura de cada troço das faixas copiadas** — a partição das sprites (`runs`, faixas
/// absolutas no buffer delas; vazia = tudo no átlas) recortada a cada faixa e levada para o lugar
/// dela no buffer da placa. Vizinhos da mesma textura juntam-se. Pura, para o gate a medir sem placa.
#[must_use]
pub fn texturas_das_faixas(
    faixas: &[Range<u32>],
    runs: &[ph2d_render::GpuTexRun],
) -> Vec<(Range<u32>, u32)> {
    let mut out: Vec<(Range<u32>, u32)> = Vec::new();
    let mut destino = 0u32;
    for r in faixas {
        let n = r.end.saturating_sub(r.start);
        let mut troços: Vec<(Range<u32>, u32)> = if runs.is_empty() {
            vec![(destino..destino + n, 0)]
        } else {
            runs.iter()
                .filter_map(|run| {
                    let (a, b) = (run.start.max(r.start), run.end.min(r.end));
                    (a < b).then(|| {
                        (
                            destino + (a - r.start)..destino + (b - r.start),
                            run.texture_id,
                        )
                    })
                })
                .collect()
        };
        troços.sort_by_key(|(t, _)| t.start);
        for (t, id) in troços {
            match out.last_mut() {
                Some((u, uid)) if *uid == id && u.end == t.start => u.end = t.end,
                _ => out.push((t, id)),
            }
        }
        destino += n;
    }
    out
}

impl GpuCook {
    /// **As saídas que misturam em grupo**, uma por [`crate::GpuPlan::sinks`] — as linhas de imagem
    /// delas vão à placa de formas ([`Self::separa_as_imagens_do_grupo`]). Vazio = nenhuma.
    pub fn set_saidas_em_grupo(&mut self, saidas: Vec<bool>) {
        self.grupo.saidas = saidas;
    }

    /// As imagens que a última separação entregou à placa — `None` sem linha nenhuma.
    #[must_use]
    pub fn imagens_do_grupo(&self) -> Option<&GpuImagensDoGrupo> {
        self.grupo.imagens.as_ref().filter(|i| i.len > 0)
    }

    /// ⭐⭐ **Tira as faixas das saídas em grupo do buffer das sprites e põe-nas no da placa** — depois
    /// do passe do dono, antes do desenho. Sem saída em grupo, nada (e a última separação esquece-se).
    pub fn separa_as_imagens_do_grupo(&mut self, gpu: &GpuContext) {
        if let Some(i) = self.grupo.imagens.as_mut() {
            i.len = 0;
            i.faixas.clear();
            i.texturas.clear();
        }
        let Some(sprites) = self.instances.as_ref() else {
            return;
        };
        let faixas: Vec<(NodeId, Range<u32>)> = sprites
            .faixas
            .iter()
            .enumerate()
            .filter(|(k, (_, r))| self.grupo.em_grupo(*k) && r.end > r.start)
            .map(|(_, f)| f.clone())
            .collect();
        let total: u32 = faixas.iter().map(|(_, r)| r.end - r.start).sum();
        if total == 0 {
            return;
        }
        let linha = u64::from(crate::lower::INSTANCE_WORDS) * 4;
        let cabe = self
            .grupo
            .imagens
            .as_ref()
            .is_some_and(|i| i.capacity >= total);
        if !cabe {
            let mut capacity = self.grupo.imagens.as_ref().map_or(1, |i| i.capacity.max(1));
            while capacity < total {
                capacity = capacity.saturating_mul(2);
            }
            self.grupo.imagens = Some(GpuImagensDoGrupo {
                buffer: gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ph2d-gpu-cook imagens do grupo"),
                    size: u64::from(capacity) * linha,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }),
                capacity,
                len: 0,
                faixas: Vec::new(),
                texturas: Vec::new(),
            });
        }
        let destino = self.grupo.imagens.as_mut().expect("garantido acima");
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-gpu-cook imagens do grupo"),
            });
        let mut o = 0u32;
        for (sink, r) in &faixas {
            let n = r.end - r.start;
            enc.copy_buffer_to_buffer(
                &sprites.buffer,
                u64::from(r.start) * linha,
                &destino.buffer,
                u64::from(o) * linha,
                u64::from(n) * linha,
            );
            destino.faixas.push((*sink, o..o + n));
            o += n;
        }
        // ⚠️ As limpezas DEPOIS das cópias, no mesmo encoder: os comandos correm por ordem.
        for (_, r) in &faixas {
            enc.clear_buffer(
                &sprites.buffer,
                u64::from(r.start) * linha,
                Some(u64::from(r.end - r.start) * linha),
            );
        }
        gpu.queue.submit(Some(enc.finish()));
        let so_faixas: Vec<Range<u32>> = faixas.iter().map(|(_, r)| r.clone()).collect();
        destino.texturas = texturas_das_faixas(&so_faixas, &self.tex_runs);
        destino.len = total;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(texture_id: u32, start: u32, end: u32) -> ph2d_render::GpuTexRun {
        ph2d_render::GpuTexRun {
            texture_id,
            start,
            end,
            blend: 0,
            sampling: 0,
        }
    }

    /// A partição vazia é o átlas; os runs recortam-se a cada faixa e mudam de lugar; vizinhos da
    /// mesma textura juntam-se (também entre duas faixas).
    #[test]
    fn a_textura_de_cada_troco_segue_a_particao_das_sprites() {
        assert_eq!(texturas_das_faixas(&[2..5, 9..10], &[]), vec![(0..4, 0)]);
        let runs = [run(7, 0, 4), run(0, 4, 8), run(9, 8, 12)];
        assert_eq!(
            texturas_das_faixas(&[2..6, 10..12], &runs),
            vec![(0..2, 7), (2..4, 0), (4..6, 9)]
        );
        // Ordenados por textura (a ordem da CPU), o recorte continua pela posição.
        let ordenados = [run(0, 4, 8), run(7, 0, 4)];
        assert_eq!(
            texturas_das_faixas(&[Range { start: 3, end: 5 }], &ordenados),
            vec![(0..1, 7), (1..2, 0)]
        );
    }
}
