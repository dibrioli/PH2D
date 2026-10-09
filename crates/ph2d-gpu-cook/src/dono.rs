//! ⭐⭐ **A ORIGEM de um objecto Motion, na placa** (doc ObjectsAndComponents 01 §13, F5; escolha do
//! dono, 08/10: mover o objecto Motion leva o que ele gera).
//!
//! Depois do cozido, a faixa de cada sink nos dois buffers (as cópias de sprite e as formas) recebe a
//! afim local → mundo do objecto dono dela: `world_pos' = L·p + t`, `basis' = L·B` — a mesma conta da
//! CPU (`motion_objeto::origem`). Um passe de cálculo por faixa, no sítio: a placa recozinha e
//! reescreve os buffers a CADA quadro (sem salto com a cena pausada), por isso nunca se aplica duas
//! vezes. Sem objecto deslocado não há passe nenhum.
//!
//! A disposição dos buffers (passo e onde está a base) é desta crate, e é lida aqui das constantes
//! que os escrevem: [`crate::lower::INSTANCE_WORDS`] e [`crate::lower_forma::FORMA_WORDS`].

use std::collections::BTreeMap;
use std::ops::Range;

use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::NodeId;

use crate::GpuCook;

/// Uma afim `[a, b, c, d, tx, ty]` (colunas `(a, b)`, `(c, d)`; translação `(tx, ty)`).
pub type Afim = [f32; 6];

const WGSL: &str = r"
struct Faixa {
    inicio: u32,
    fim: u32,
    passo: u32,
    base: u32,
    l: vec4<f32>,
    t: vec4<f32>,
};
@group(0) @binding(0) var<storage, read_write> dados: array<f32>;
@group(0) @binding(1) var<uniform> f: Faixa;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) n: vec3<u32>) {
    let i = f.inicio + gid.x + gid.y * n.x * 64u;
    if (i >= f.fim) {
        return;
    }
    let o = i * f.passo;
    let l = mat2x2<f32>(f.l.xy, f.l.zw);
    let p = l * vec2<f32>(dados[o], dados[o + 1u]) + f.t.xy;
    dados[o] = p.x;
    dados[o + 1u] = p.y;
    let b = o + f.base;
    let c0 = l * vec2<f32>(dados[b], dados[b + 1u]);
    let c1 = l * vec2<f32>(dados[b + 2u], dados[b + 3u]);
    dados[b] = c0.x;
    dados[b + 1u] = c0.y;
    dados[b + 2u] = c1.x;
    dados[b + 3u] = c1.y;
}
";

/// O passe compilado (um por dispositivo; quem o guarda é o chamador).
#[derive(Debug)]
pub struct PasseDoDono {
    pipeline: wgpu::ComputePipeline,
}

impl PasseDoDono {
    /// Compila o passe.
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        Self {
            pipeline: crate::encode::create_pipeline(gpu, WGSL, "ph2d-gpu-cook dono"),
        }
    }
}

/// O passo (em palavras) e onde começa a base 2×2, num buffer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Disposicao {
    passo: u32,
    base: u32,
}

/// `RenderInstance`: `world_pos` na palavra 0, `basis` no byte 48.
const SPRITES: Disposicao = Disposicao {
    passo: crate::lower::INSTANCE_WORDS,
    base: 12,
};
/// `ShapeInstance`: `pos` na palavra 0, `basis` no byte 16.
const FORMAS: Disposicao = Disposicao {
    passo: crate::lower_forma::FORMA_WORDS,
    base: 4,
};

/// O tecto do `dispatch` por dimensão (WebGPU): acima dele, a 2.ª dimensão.
const MAX_GRUPOS: u32 = 65_535;

impl GpuCook {
    /// ⭐ **Aplica a afim de cada sink** à faixa dele nos dois buffers do último cozido. `afins` só
    /// traz os sinks com afim que não é a identidade (vazio ⇒ nada a fazer). Um submit.
    pub fn aplica_donos(
        &self,
        gpu: &GpuContext,
        passe: &PasseDoDono,
        afins: &BTreeMap<NodeId, Afim>,
    ) {
        if afins.is_empty() {
            return;
        }
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-gpu-cook dono"),
            });
        let mut algum = false;
        if let Some(i) = self.instances.as_ref() {
            algum |= aplica(gpu, passe, &mut encoder, &i.buffer, SPRITES, &i.faixas, afins);
        }
        if let Some(f) = self.formas.as_ref() {
            algum |= aplica(gpu, passe, &mut encoder, &f.buffer, FORMAS, &f.faixas, afins);
        }
        if algum {
            gpu.queue.submit(Some(encoder.finish()));
        }
    }

    /// As faixas `(sink, cópias)` do último cozido no buffer das sprites e no das formas.
    #[must_use]
    pub fn faixas_dos_sinks(&self) -> (&[(NodeId, Range<u32>)], &[(NodeId, Range<u32>)]) {
        (
            self.instances.as_ref().map_or(&[][..], |i| &i.faixas),
            self.formas.as_ref().map_or(&[][..], |f| &f.faixas),
        )
    }
}

fn aplica(
    gpu: &GpuContext,
    passe: &PasseDoDono,
    encoder: &mut wgpu::CommandEncoder,
    buffer: &wgpu::Buffer,
    d: Disposicao,
    faixas: &[(NodeId, Range<u32>)],
    afins: &BTreeMap<NodeId, Afim>,
) -> bool {
    let mut algum = false;
    for (sink, r) in faixas {
        let Some(a) = afins.get(sink) else {
            continue;
        };
        let n = r.end.saturating_sub(r.start);
        if n == 0 {
            continue;
        }
        let mut u = [0u8; 48];
        for (k, w) in [r.start, r.end, d.passo, d.base].iter().enumerate() {
            u[k * 4..k * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        for (k, v) in [a[0], a[1], a[2], a[3], a[4], a[5], 0.0, 0.0].iter().enumerate() {
            u[16 + k * 4..16 + k * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        let uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-gpu-cook dono faixa"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&uniform, 0, &u);
        let bind = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-gpu-cook dono"),
            layout: &passe.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform.as_entire_binding(),
                },
            ],
        });
        let grupos = n.div_ceil(64);
        let x = grupos.min(MAX_GRUPOS);
        let y = grupos.div_ceil(x);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ph2d-gpu-cook dono"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&passe.pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(x, y, 1);
        algum = true;
    }
    algum
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_render::RenderInstance;

    fn ponto(a: &Afim, p: [f32; 2]) -> [f32; 2] {
        [a[0] * p[0] + a[2] * p[1] + a[4], a[1] * p[0] + a[3] * p[1] + a[5]]
    }

    fn base(a: &Afim, b: [f32; 4]) -> [f32; 4] {
        [
            a[0] * b[0] + a[2] * b[1],
            a[1] * b[0] + a[3] * b[1],
            a[0] * b[2] + a[2] * b[3],
            a[1] * b[2] + a[3] * b[3],
        ]
    }

    fn quase(a: f32, b: f32) -> bool {
        (a - b).abs() <= 1e-5 * (1.0 + b.abs())
    }

    /// ⭐⭐ **A placa faz a conta da CPU** (DIRETIVA §4): as faixas dos sinks com afim saem com
    /// `L·p + t` e `L·B`, nas sprites E nas formas; a faixa sem afim e o resto da cópia (cor, tamanho)
    /// ficam byte a byte. (Mutação: trocar `f.t.xy` por `0` no WGSL ⇒ RED.)
    #[test]
    #[ignore = "precisa de adapter de GPU"]
    fn a_placa_aplica_a_afim_como_a_cpu() {
        let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
            panic!("sem adapter — este gate mede a placa");
        };
        let a: Afim = [0.6, 0.8, -1.6, 1.2, 10.0, -3.0];
        let (s1, s2) = (NodeId(7), NodeId(9));
        let mut cook = GpuCook::new();
        // Sprites: 3 do sink 7 (com afim) e 2 do sink 9 (sem).
        let mut insts: Vec<RenderInstance> = (0..5)
            .map(|k| {
                let mut i: RenderInstance = bytemuck::Zeroable::zeroed();
                i.world_pos = [k as f32, 2.0 * k as f32 - 1.0];
                i.basis = [1.0 + k as f32, 0.5, -0.25, 2.0];
                i.size = [3.0, 4.0];
                i.tint = [0.1, 0.2, 0.3, 1.0];
                i
            })
            .collect();
        // ⚠️ Os buffers do produto não aceitam escrita da CPU (`COPY_DST`): o teste cria os seus.
        let buffer = |bytes: &[u8]| {
            let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("teste dono"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu.queue.write_buffer(&b, 0, bytes);
            b
        };
        cook.instances = Some(crate::instances::GpuInstances {
            buffer: buffer(bytemuck::cast_slice(&insts)),
            len: 5,
            capacity: 5,
            faixas: vec![(s1, 0..3), (s2, 3..5)],
        });
        // Formas: 2 do sink 7.
        let mut formas: Vec<f32> = vec![0.0; 2 * FORMAS.passo as usize];
        for k in 0..2 {
            let o = k * FORMAS.passo as usize;
            formas[o] = 5.0 + k as f32;
            formas[o + 1] = -2.0;
            formas[o + 4..o + 8].copy_from_slice(&[1.0, 0.0, 0.0, 1.0]);
        }
        cook.formas = Some(crate::GpuFormas {
            buffer: buffer(bytemuck::cast_slice(&formas)),
            len: 2,
            capacity: 2,
            faixas: vec![(s1, 0..2)],
        });

        let passe = PasseDoDono::new(&gpu);
        cook.aplica_donos(&gpu, &passe, &BTreeMap::from([(s1, a)]));

        let lidas = crate::read_instances(&gpu, cook.instances.as_ref().unwrap());
        for (k, (lida, orig)) in lidas.iter().zip(&insts).enumerate() {
            let (p, b) = if k < 3 {
                (ponto(&a, orig.world_pos), base(&a, orig.basis))
            } else {
                (orig.world_pos, orig.basis)
            };
            assert!(
                quase(lida.world_pos[0], p[0]) && quase(lida.world_pos[1], p[1]),
                "{k}: {:?} ≠ {p:?}",
                lida.world_pos
            );
            for c in 0..4 {
                assert!(quase(lida.basis[c], b[c]), "{k}: base {:?} ≠ {b:?}", lida.basis);
            }
            assert_eq!(lida.size, orig.size);
            assert_eq!(lida.tint, orig.tint);
        }
        let palavras = crate::read_formas(&gpu, cook.formas.as_ref().unwrap());
        for k in 0..2 {
            let o = k * FORMAS.passo as usize;
            let p = ponto(&a, [formas[o], formas[o + 1]]);
            let lido = [f32::from_bits(palavras[o]), f32::from_bits(palavras[o + 1])];
            assert!(quase(lido[0], p[0]) && quase(lido[1], p[1]), "forma {k}: {lido:?}");
            let b = base(&a, [1.0, 0.0, 0.0, 1.0]);
            assert!(quase(f32::from_bits(palavras[o + 4]), b[0]));
            assert!(quase(f32::from_bits(palavras[o + 7]), b[3]));
        }
        insts.clear();
    }
}
