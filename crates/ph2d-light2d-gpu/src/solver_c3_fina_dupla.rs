//! ⭐ W6h §10 — as DUAS pipelines da fina (`Config::usa_fina_dupla`; handoff W6h §10): a tabela das áreas dos
//! blocos com cunha (`c3_fina_cunhas_linhas`/`_colunas`), a `c3_fina_marca` que lista os grupos 8×8 da fina
//! perto de uma cunha, a `c3_fina_enxuta` (`FINA_ENXUTA`: o `num_corpo` sem o ramo da cunha) sobre os outros e
//! a `c3_fina_listada` (a fina de sempre) por despacho indireto sobre os listados. Módulo-filho de
//! `solver_c3.rs` (o tecto de 700 LOC).

use super::*;
use crate::passlist::cantos::{NOME_FINA_CUNHAS, NOME_FINA_MARCAS};

pub(super) struct FinaDupla {
    marcas: wgpu::Buffer,
    /// Os grupos 3: as marcas e a tabela escritas · as marcas lidas.
    grupos: [wgpu::BindGroup; 2],
    linhas: wgpu::ComputePipeline,
    colunas: wgpu::ComputePipeline,
    marca: wgpu::ComputePipeline,
    enxuta: Par,
    listada: Par,
    /// Os blocos 8×8 da captura e os grupos da grade fina (o início da lista nas marcas).
    blocos: [u32; 2],
    grupos_da_grade: u32,
}

impl FinaDupla {
    /// As cinco pipelines sobre os grupos 0–2 da fina (`g2`); `constantes` = as próprias da fina.
    pub(super) fn nova<F>(
        e: &Entradas<'_>,
        pipeline_em: &F,
        [cena, lampadas]: [&wgpu::BindGroupLayout; 2],
        g2: &wgpu::BindGroupLayout,
        constantes: &[(&str, f64)],
    ) -> Self
    where
        F: Fn(
                &str,
                &[Option<&wgpu::BindGroupLayout>],
                &str,
                &[(&str, f64)],
            ) -> wgpu::ComputePipeline
            + Sync,
    {
        let device = e.placa;
        let buffer = |nome: &str| {
            let d = e
                .lista
                .buffers
                .iter()
                .find(|b| b.nome == nome)
                .expect("declarado na PassList");
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(nome),
                size: d.bytes,
                usage: d.usos,
                mapped_at_creation: false,
            })
        };
        let (marcas, cunhas) = (buffer(NOME_FINA_MARCAS), buffer(NOME_FINA_CUNHAS));
        let g3 = [
            vec![(25, false, &marcas), (27, false, &cunhas)],
            vec![(26, true, &marcas)],
        ]
        .map(|b| {
            let entradas: Vec<_> = b
                .iter()
                .map(|&(binding, so, _)| fina_cantos::armazenamento(binding, so))
                .collect();
            let l = layout(device, "luz2d c3 fina dupla", &entradas);
            let ligacoes: Vec<_> = b
                .iter()
                .map(|&(binding, _, buf)| wgpu::BindGroupEntry {
                    binding,
                    resource: buf.as_entire_binding(),
                })
                .collect();
            let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 fina dupla"),
                layout: &l,
                entries: &ligacoes,
            });
            (l, g)
        });
        let escrita = [Some(cena), Some(lampadas), Some(g2), Some(&g3[0].0)];
        let lida = [Some(cena), Some(lampadas), Some(g2), Some(&g3[1].0)];
        let par = |rotulo: &str, entrada: &str, proprias: &[(&str, f64)]| {
            Par::em_paralelo(e.cfg.distantes, |d| {
                let c: Vec<_> = constantes
                    .iter()
                    .chain(proprias)
                    .copied()
                    .chain([("DISTANTES", d)])
                    .collect();
                pipeline_em(rotulo, &lida, entrada, &c)
            })
        };
        let so = |rotulo: &str, entrada: &str| pipeline_em(rotulo, &escrita, entrada, constantes);
        let (enxuta, listada, [linhas, colunas, marca]) = std::thread::scope(|s| {
            let enxuta = s.spawn(|| {
                par(
                    "luz2d c3 fina enxuta",
                    "c3_fina_enxuta",
                    &[("FINA_ENXUTA", 1.0)],
                )
            });
            let listada = s.spawn(|| par("luz2d c3 fina listada", "c3_fina_listada", &[]));
            let pequenas = [
                ("luz2d c3 fina cunhas", "c3_fina_cunhas_linhas"),
                ("luz2d c3 fina cunhas", "c3_fina_cunhas_colunas"),
                ("luz2d c3 fina marca", "c3_fina_marca"),
            ]
            .map(|(r, entrada)| so(r, entrada));
            (
                enxuta.join().expect("a compilação da fina enxuta"),
                listada.join().expect("a compilação da fina listada"),
                pequenas,
            )
        });
        let op = Opcionais::de(e.cfg);
        let captura = e.lista.texturas[op.borda.expect("os cantos têm a borda")].tamanho;
        let gf = e.cfg.grade_da_fluencia();
        let [a, b] = g3;
        Self {
            marcas,
            grupos: [a.1, b.1],
            linhas,
            colunas,
            marca,
            enxuta,
            listada,
            blocos: captura.map(|v| v.div_ceil(8)),
            grupos_da_grade: gf.largura.div_ceil(8) * gf.altura.div_ceil(8),
        }
    }

    /// A lista vazia antes do quadro (o despacho `[0, 1, 1]`; o mapa reescreve-se inteiro).
    pub(super) fn zera(&self, gpu: &GpuContext) {
        gpu.queue.write_buffer(
            &self.marcas,
            0,
            bytemuck::cast_slice(&[0u32, 1, 1, self.grupos_da_grade]),
        );
    }

    /// No passe da fina (os grupos 0–2 ligados): a tabela, as marcas, a enxuta sobre os `grupos` da região e a
    /// fina de sempre sobre os listados.
    pub(super) fn gravar(&self, p: &mut wgpu::ComputePass<'_>, distantes: bool, grupos: [u32; 2]) {
        let [bx, by] = self.blocos;
        p.set_bind_group(3, &self.grupos[0], &[]);
        p.set_pipeline(&self.linhas);
        p.dispatch_workgroups(by, 1, 1);
        p.set_pipeline(&self.colunas);
        p.dispatch_workgroups(bx, 1, 1);
        p.set_pipeline(&self.marca);
        p.dispatch_workgroups((grupos[0] * grupos[1]).div_ceil(64), 1, 1);
        p.set_bind_group(3, &self.grupos[1], &[]);
        p.set_pipeline(self.enxuta.de(distantes));
        p.dispatch_workgroups(grupos[0], grupos[1], 1);
        p.set_pipeline(self.listada.de(distantes));
        p.dispatch_workgroups_indirect(&self.marcas, 0);
    }
}
