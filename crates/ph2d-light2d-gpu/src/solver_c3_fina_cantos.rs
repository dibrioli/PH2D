//! ⭐ W6g r5 — o canto (`Config::canto`) num passe PRÓPRIO (`Config::adia_o_canto`; handoff W6g §12): a
//! `c3_fina_adia` (a fina compilada sem o código do canto) lista os texels de duas faces, o
//! `c3_fina_cantos` calcula-os com o canto num buffer seu e a `c3_fina_junta` grava-os nas texturas da
//! fina. Módulo-filho de `solver_c3.rs` (o tecto de 700 LOC).

use super::*;
use crate::passlist::{NOME_FINA_ADIADOS, NOME_FINA_CANTOS_LUZ};

/// As constantes que tiram o código do canto da fina e a fazem parar no texel de duas faces.
const SEM_CANTO: [(&str, f64); 5] = [
    ("CANTO_NORMAIS", 0.0),
    ("CANTO_LUZ", 0.0),
    ("CANTO_FRACAO", 0.0),
    ("CANTO_UM_LADO", 0.0),
    ("CANTO_ADIA", 1.0),
];

pub(super) struct FinaCantos {
    lista: wgpu::Buffer,
    /// O grupo 2 da fina SEM as texturas de saída (o do passe dos cantos).
    entradas: wgpu::BindGroup,
    /// Os grupos 3: a lista escrita · a lista lida e a luz escrita · as duas lidas.
    grupos: [wgpu::BindGroup; 3],
    cantos: Par,
    junta: wgpu::ComputePipeline,
}

/// O grupo 2 da fina: o layout, as ligações e o layout montado.
pub(super) type GrupoDaFina<'a> = (
    &'a [wgpu::BindGroupLayoutEntry],
    &'a [wgpu::BindGroupEntry<'a>],
    &'a wgpu::BindGroupLayout,
);

/// Uma entrada de buffer de armazenamento do grupo 3.
fn armazenamento(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

impl FinaCantos {
    /// A `c3_fina_adia` (no lugar da fina) e o passe dos cantos, sobre os grupos 0–1 da fina e o grupo
    /// 2 dela (de onde saem as saídas 4 e 6); `constantes` = as próprias da fina.
    pub(super) fn nova<F>(
        e: &Entradas<'_>,
        pipeline_em: &F,
        [cena, lampadas]: [&wgpu::BindGroupLayout; 2],
        (entradas_g2, grupo_g2, g2): GrupoDaFina<'_>,
        constantes: &[(&str, f64)],
    ) -> (Par, Self)
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
        let saida = |b: u32| b == 4 || b == 6;
        let le: Vec<_> = entradas_g2
            .iter()
            .filter(|l| !saida(l.binding))
            .copied()
            .collect();
        let g2_entradas = layout(device, "luz2d c3 fina entradas", &le);
        let ligacoes: Vec<_> = grupo_g2
            .iter()
            .filter(|g| !saida(g.binding))
            .cloned()
            .collect();
        let entradas = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c3 fina entradas"),
            layout: &g2_entradas,
            entries: &ligacoes,
        });
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
        let (lista, luz) = (buffer(NOME_FINA_ADIADOS), buffer(NOME_FINA_CANTOS_LUZ));
        let g3 = [
            vec![(21, false, &lista)],
            vec![(22, true, &lista), (23, false, &luz)],
            vec![(22, true, &lista), (24, true, &luz)],
        ]
        .map(|b| {
            let entradas: Vec<_> = b
                .iter()
                .map(|&(binding, so, _)| armazenamento(binding, so))
                .collect();
            let l = layout(device, "luz2d c3 fina cantos", &entradas);
            let ligacoes: Vec<_> = b
                .iter()
                .map(|&(binding, _, buf)| wgpu::BindGroupEntry {
                    binding,
                    resource: buf.as_entire_binding(),
                })
                .collect();
            let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 fina cantos"),
                layout: &l,
                entries: &ligacoes,
            });
            (l, g)
        });
        let par = |rotulo: &str,
                   g2: &wgpu::BindGroupLayout,
                   g3: &wgpu::BindGroupLayout,
                   entrada: &str,
                   proprias: &[(&str, f64)]| {
            Par::em_paralelo(e.cfg.distantes, |d| {
                let c: Vec<_> = constantes
                    .iter()
                    .chain(proprias)
                    .copied()
                    .chain([("DISTANTES", d)])
                    .collect();
                pipeline_em(
                    rotulo,
                    &[Some(cena), Some(lampadas), Some(g2), Some(g3)],
                    entrada,
                    &c,
                )
            })
        };
        let (principal, cantos, junta) = std::thread::scope(|s| {
            let principal =
                s.spawn(|| par("luz2d c3 fina", g2, &g3[0].0, "c3_fina_adia", &SEM_CANTO));
            let junta = s.spawn(|| par("luz2d c3 fina junta", g2, &g3[2].0, "c3_fina_junta", &[]));
            let cantos = par(
                "luz2d c3 fina cantos",
                &g2_entradas,
                &g3[1].0,
                "c3_fina_cantos",
                &[],
            );
            (
                principal.join().expect("a compilação da fina sem o canto"),
                cantos,
                junta.join().expect("a compilação da junta dos cantos").sem,
            )
        });
        let [a, b, c] = g3;
        let fc = Self {
            lista,
            entradas,
            grupos: [a.1, b.1, c.1],
            cantos,
            junta,
        };
        (principal, fc)
    }

    /// A lista vazia antes do quadro (o despacho `[0, 1, 1]`, zero texels).
    pub(super) fn zera(&self, gpu: &GpuContext) {
        gpu.queue
            .write_buffer(&self.lista, 0, bytemuck::cast_slice(&[0u32, 1, 1, 0]));
    }

    /// O grupo 3 da `c3_fina_adia` (a lista escrita).
    pub(super) fn grupo(&self) -> &wgpu::BindGroup {
        &self.grupos[0]
    }

    /// Depois da `c3_fina_adia`: o passe dos cantos e a junta (`grupo_fina` = o grupo 2 da fina).
    pub(super) fn depois(
        &self,
        p: &mut wgpu::ComputePass<'_>,
        distantes: bool,
        grupo_fina: &wgpu::BindGroup,
    ) {
        p.set_pipeline(self.cantos.de(distantes));
        p.set_bind_group(2, &self.entradas, &[]);
        p.set_bind_group(3, &self.grupos[1], &[]);
        p.dispatch_workgroups_indirect(&self.lista, 0);
        p.set_pipeline(&self.junta);
        p.set_bind_group(2, grupo_fina, &[]);
        p.set_bind_group(3, &self.grupos[2], &[]);
        p.dispatch_workgroups_indirect(&self.lista, 0);
    }
}
