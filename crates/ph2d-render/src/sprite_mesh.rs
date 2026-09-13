//! ⭐⭐⭐ **UMA SPRITE DESENHADA COMO MALHA** — o primitivo que põe a imagem presa ao esqueleto
//! DENTRO do passe de sprites (plano `docs/Skeleton/03_plano_a_pele_no_passe_de_sprites.md`).
//!
//! # Porque não há pipeline nova
//!
//! O `vs_main` do `sprite.wgsl` calcula tudo a partir de dois atributos por vértice e da
//! instância: `local = anchor + quad_pos · size`, `world = world_pos + basis · local`, e a UV sai
//! do `quad_uv` (espelhado, repetido, `uv_xform`, `mix(atlas_uv)`). ⇒ um vértice da malha leva o
//! `quad_uv` DE REPOUSO e o `quad_pos` que devolve a posição POSADA, e a malha herda tinta,
//! opacidade, mistura, pré-multiplicação, repetição e o shader de marca do recorte — tudo o que
//! uma sprite tem, sem uma linha de WGSL.
//!
//! As 10 pipelines são `TriangleStrip` sem culling ⇒ `N` triângulos entram como UMA tira com
//! degenerados de ligação: `5N − 2` vértices.
//!
//! # ⭐⭐ Porque não há costuras
//!
//! Dois triângulos que partilham uma aresta são rasterizados pela regra de canto: cada centro de
//! pixel pertence a UM deles. Não há anti-aliasing por aresta a compor `1 − a·b`, nem faixa
//! desenhada duas vezes — que é o que o caminho do Vello (um recorte por triângulo) não conseguia
//! (`tests/it/skin_pieces_gpu_cost.rs`).
//!
//! # ⚠️ Divergência DECLARADA: a tinta por canto
//!
//! O shader calcula a tinta por canto POR VÉRTICE (bilinear sobre o `quad_uv`) e interpola-a. O
//! quad tem 4 vértices e interpola-a em dois triângulos; uma malha fina aproxima-a do bilinear
//! verdadeiro. Com a tinta por canto uniforme — quase toda sprite — os dois coincidem.

use crate::sprite::{QuadVertex, RenderInstance};
use ph2d_gpu::GpuContext;

/// ⭐ **A malha de uma sprite, posada** — componente de APRESENTAÇÃO, na mesma entidade da
/// [`RenderInstance`] que ela substitui.
///
/// - `local`: a posição POSADA de cada vértice, em metros no espaço LOCAL da sprite (o espaço em que
///   o quad de repouso é `anchor ± size/2`).
/// - `uv`: a coordenada DE REPOUSO de cada vértice na imagem, `0..1`, com `v = 0` em cima (a
///   convenção do [`QuadVertex::QUAD_STRIP`]).
/// - `tris`: triângulos por índice em `local`/`uv`.
///
/// ⚠️ **Se a malha não puder ser desenhada** (comprimentos diferentes, `size` zero, nenhum triângulo
/// válido), a instância desenha o QUAD de repouso — visível, nunca calada.
#[derive(bevy_ecs::component::Component, Clone, Debug, Default, PartialEq)]
pub struct SpriteMesh {
    pub local: Vec<[f32; 2]>,
    pub uv: Vec<[f32; 2]>,
    pub tris: Vec<[u32; 3]>,
}

/// As malhas de UMA chamada de render: os vértices costurados e o intervalo de cada uma.
#[derive(Default)]
pub(crate) struct MeshFrame {
    pub(crate) vertices: Vec<QuadVertex>,
    pub(crate) ranges: Vec<(u32, u32)>,
    /// Scratch dos vértices de UMA malha antes da costura (reutilizado entre malhas).
    work: Vec<QuadVertex>,
}

impl MeshFrame {
    pub(crate) fn clear(&mut self) {
        self.vertices.clear();
        self.ranges.clear();
    }

    /// Acumula `malha` convertida para o quad desta instância e devolve a marca (`1..`), ou `0`
    /// se ela não puder ser desenhada.
    pub(crate) fn push(&mut self, malha: &SpriteMesh, anchor: [f32; 2], size: [f32; 2]) -> u32 {
        if malha.local.len() != malha.uv.len() {
            return 0;
        }
        self.work.clear();
        for (l, uv) in malha.local.iter().zip(&malha.uv) {
            let Some(pos) = quad_pos(*l, anchor, size) else {
                return 0;
            };
            self.work.push(QuadVertex { pos, uv: *uv });
        }
        let antes = self.vertices.len();
        let Some(intervalo) = stitch(&mut self.vertices, &self.work, &malha.tris) else {
            self.vertices.truncate(antes);
            return 0;
        };
        let marca = self.ranges.len() + 1;
        let cabe = u32::try_from(marca)
            .ok()
            .filter(|m| *m <= RenderInstance::MESH_MASK >> RenderInstance::MESH_SHIFT);
        let Some(marca) = cabe else {
            self.vertices.truncate(antes);
            return 0;
        };
        self.ranges.push(intervalo);
        marca
    }
}

/// ⭐ **`local → quad_pos`** — a inversa da lei do shader `local = anchor + quad_pos · size`.
///
/// `None` quando o `size` tem um lado nulo ou não finito: ali não há quad a que referir o vértice.
#[must_use]
pub(crate) fn quad_pos(local: [f32; 2], anchor: [f32; 2], size: [f32; 2]) -> Option<[f32; 2]> {
    let ok = |s: f32| s.is_finite() && s != 0.0;
    if !(ok(size[0]) && ok(size[1])) {
        return None;
    }
    Some([
        (local[0] - anchor[0]) / size[0],
        (local[1] - anchor[1]) / size[1],
    ])
}

/// ⭐ **Costura uma lista de triângulos numa tira** com degenerados de ligação, acrescentando a `out`.
///
/// Para cada triângulo depois do primeiro acrescenta `(último, a)` e depois `a b c`: as quatro
/// janelas de 3 que atravessam a ligação têm dois vértices iguais (área zero, nenhum fragmento).
/// Triângulos com índices fora de `verts` são **saltados**. `None` se nenhum triângulo for válido.
pub(crate) fn stitch(
    out: &mut Vec<QuadVertex>,
    verts: &[QuadVertex],
    tris: &[[u32; 3]],
) -> Option<(u32, u32)> {
    let start = out.len();
    let mut algum = false;
    for t in tris {
        let (Some(&a), Some(&b), Some(&c)) = (
            verts.get(t[0] as usize),
            verts.get(t[1] as usize),
            verts.get(t[2] as usize),
        ) else {
            continue;
        };
        if algum && let Some(&ultimo) = out.last() {
            out.push(ultimo);
            out.push(a);
        }
        out.extend_from_slice(&[a, b, c]);
        algum = true;
    }
    if !algum {
        return None;
    }
    Some((u32::try_from(start).ok()?, u32::try_from(out.len()).ok()?))
}

/// Limpa a marca de malha de uma instância — o que toda instância que NÃO veio da recolha precisa.
pub(crate) fn clear_mesh_tag(inst: &mut RenderInstance) {
    inst.flip_uv &= !RenderInstance::MESH_MASK;
}

/// ⭐ **Desenha um run**: o quad instanciado de sempre, ou — se o run é de uma malha — o intervalo
/// dela, trocando o buffer do slot 0 e repondo o quad a seguir.
///
/// ⚠️ **Uma porta para os TRÊS passes** (normal, recorte, máscara): uma malha que desenhasse num e
/// não nos outros perderia o recorte ou a máscara sem erro nenhum.
pub(crate) fn draw_run(
    pass: &mut wgpu::RenderPass<'_>,
    run: &crate::renderer::DrawRun,
    quad: &wgpu::Buffer,
    mesh: &wgpu::Buffer,
    ranges: &[(u32, u32)],
) {
    if run.mesh == 0 {
        pass.draw(0..4, run.start..run.end);
        return;
    }
    let Some(&(a, b)) = ranges.get(run.mesh as usize - 1) else {
        return;
    };
    pass.set_vertex_buffer(0, mesh.slice(..));
    pass.draw(a..b, run.start..run.end);
    pass.set_vertex_buffer(0, quad.slice(..));
}

/// O buffer de vértices das malhas de uma chamada — o gémeo do [`crate::InstanceBuffer`].
pub(crate) struct MeshVertexBuffer {
    buffer: wgpu::Buffer,
    capacity: u32,
}

impl MeshVertexBuffer {
    pub(crate) fn new(gpu: &GpuContext) -> Self {
        Self {
            buffer: Self::allocate(gpu, 1),
            capacity: 1,
        }
    }

    fn allocate(gpu: &GpuContext, capacity: u32) -> wgpu::Buffer {
        gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render sprite mesh vbo"),
            size: u64::from(capacity) * std::mem::size_of::<QuadVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub(crate) fn upload(&mut self, gpu: &GpuContext, vertices: &[QuadVertex]) {
        let Ok(needed) = u32::try_from(vertices.len()) else {
            return;
        };
        if needed > self.capacity {
            let mut cap = self.capacity.max(1);
            while cap < needed {
                cap = cap.saturating_mul(2);
            }
            self.buffer = Self::allocate(gpu, cap);
            self.capacity = cap;
        }
        if needed > 0 {
            gpu.queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(vertices));
        }
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

#[cfg(test)]
#[path = "sprite_mesh_tests.rs"]
mod tests;
