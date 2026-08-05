//! O passe wgpu que põe a malha na tela.
//!
//! ⚠️ **Dois vertex buffers, não um interleaved — e é a decisão que paga.** A
//! `Mesh` guarda posição e normal em vetores SEPARADOS (SoA), e intercalar para
//! subir custaria uma cópia da malha inteira a cada upload. Com um buffer por
//! atributo, o upload é `write_buffer` direto sobre o slice que já existe, e o
//! caminho que a W2 vai querer — *um dab mexeu em posições, a cor não mudou* —
//! sobe **só o que mudou** em vez de reintercalar tudo.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use ph2d_mesh::{Mesh, Pose};
use wgpu::util::DeviceExt as _;

use crate::camera::Camera3d;
use crate::lighting::RigRaw;
use crate::shade::ShadeRaw;
use crate::upload;

/// O fonte do shader, exposto para o gate poder afirmar o que ele DECLARA sem
/// precisar de device — o molde do `IMPASTO_LIGHT_WGSL` do `ph2d-render`.
pub(crate) const MESH_WGSL: &str = include_str!("shaders/mesh.wgsl");

/// O uniform da câmera. `mat4x4` alinha em 16 B, então não há padding a declarar.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraRaw {
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
}

/// O uniform do objeto: **onde ele está**. Uma `mat4x4` alinha em 16 B, então
/// não há padding a declarar.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ObjectRaw {
    model: [[f32; 4]; 4],
}

/// Os buffers da malha no device.
struct MeshGpu {
    positions: wgpu::Buffer,
    normals: wgpu::Buffer,
    /// A MÁSCARA por vértice — o canal de autoria, `0 = livre`.
    ///
    /// ⚠️ **Um `f32` e não um `vec3` de material.** O SculptGL guarda a máscara
    /// no canal `z` de um atributo de material cujos outros dois canais são
    /// rugosidade e metalness; portá-lo inteiro agora subiria **dois canais que
    /// ninguém escreve e ninguém lê** — 12 B/vértice contra 4, e dois controles
    /// mortos. O material chega com o Paint (W7/W9), e o canal é aditivo.
    masks: wgpu::Buffer,
    /// A CURVATURA por vértice — o canal que faz a forma ser LIDA
    /// (`ph2d_mesh::curvature`). ⚠️ **Ela é DERIVADA**, então ao contrário da
    /// máscara não há `Option` a resolver: toda malha tem uma, sempre, e o
    /// buffer sobe direto do slice que já existe.
    curvatures: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
    vert_capacity: usize,
    index_capacity: usize,
    /// **AS ARESTAS**, para o passe de wireframe — construídas SOB DEMANDA.
    ///
    /// ⚠️ `None` até o artista armar a malha, e derrubada por todo
    /// [`MeshRenderer::upload_at`] (que é o caminho por onde a topologia muda).
    /// Ela custa até 24 B por vértice — 24 MB num milhão — e a maioria esculpe
    /// com ela desligada: construí-la junto com a malha faria todo artista pagar
    /// pela vista de um.
    ///
    /// ⚠️ E ela **compartilha os buffers de posição e normal** com o passe de
    /// faces: só os ÍNDICES são outros. É isso que faz o upload incremental de
    /// um dab servir os dois passes sem saber que o segundo existe.
    wire: Option<wgpu::Buffer>,
    wire_count: u32,
}

/// **UM OBJETO no device** — a geometria e a pose que a põe no mundo.
///
/// ⚠️ O `model` é um buffer por objeto, e não um offset dinâmico num buffer só:
/// um uniform dinâmico exige alinhamento de 256 B por entrada (`min_uniform_
/// buffer_offset_alignment`), então uma cena de blocagem com uma dúzia de peças
/// pagaria 3 KB de padding para poupar uma dúzia de alocações de 64 B. O dia em
/// que a lista tiver centenas de objetos a conta se inverte — e aí a mudança é
/// interna a este arquivo.
struct Slot {
    gpu: MeshGpu,
    model: wgpu::Buffer,
    bind: wgpu::BindGroup,
}

/// **O QUE este passe desenha** — a forma ou a malha de arestas dela.
///
/// ⚠️ Um enum e não um `bool`, e não é cerimônia: os dois passes leem os MESMOS
/// buffers de vértice e diferem só no índice, então um `wire: bool` no meio do
/// laço leria como *"desenhe de outro jeito"* quando o que ele decide é **qual
/// geometria**. E é um parâmetro do `draw_all` e não um segundo laço porque
/// aquele laço é a única resposta a *"o que este renderizador desenha"* — uma
/// cópia dele passaria a descrever uma cena diferente no dia em que a lista
/// ganhasse uma regra.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Draw {
    /// Os triângulos da forma.
    Faces,
    /// As arestas, por cima dela.
    Wire,
}

/// **COMO o passe é montado** — ver [`build`].
#[path = "pipeline_build.rs"]
mod build;

/// O renderizador da malha.
pub struct MeshRenderer {
    pipeline: wgpu::RenderPipeline,
    gbuffer_pipeline: wgpu::RenderPipeline,
    /// O passe de ARESTAS — `LineList`, com viés de profundidade. Ver
    /// [`crate::wire`] para por que ele não é `PolygonMode::Line`.
    wire_pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    rig_uniform: wgpu::Buffer,
    /// As opções de sombreamento do barro — ver [`crate::shade`].
    shade_uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    /// O layout do grupo 1 — guardado porque um `Slot` novo nasce a cada objeto
    /// que a cena ganha, e cada um precisa do seu bind group.
    obj_bgl: wgpu::BindGroupLayout,
    depth: Option<wgpu::TextureView>,
    depth_size: (u32, u32),
    /// **A LISTA.** Um objeto por posição; o índice é o mesmo que a cena usa.
    slots: Vec<Slot>,
    /// A pose de cada objeto, do lado da CPU.
    ///
    /// ⚠️ Ela é escrita ao device **no render**, e não no `set_pose`, porque
    /// mover um objeto não é um evento de GPU: um arrasto de gizmo emitiria uma
    /// escrita por movimento do mouse para um valor que só o próximo frame lê.
    /// O vetor pode ser mais longo que `slots` (uma pose autorada antes de a
    /// malha subir), e sobra é ignorada.
    poses: Vec<Pose>,
    scratch_indices: Vec<[u32; 3]>,
    /// O rascunho da lista de ARESTAS (`crate::wire_indices`) — plano, e irmão
    /// do `scratch_indices` (que é por-triângulo). Reusado entre rebuilds.
    scratch_indices_flat: Vec<u32>,
    scratch_moved: Vec<u32>,
    scratch_runs: Vec<(u32, u32)>,
    /// Zeros para uma malha que ninguém mascarou — ver [`masks_of`].
    scratch_masks: Vec<f32>,
}

/// A máscara da malha, ou **zeros** para uma que ninguém mascarou.
///
/// ⚠️ O plano é `Option` na malha (uma esfera intocada não paga 4 B/vértice), e
/// o device quer um buffer sempre presente: um pipeline por presença de máscara
/// seriam DUAS respostas a *"como esta malha é desenhada"*, divergindo no único
/// lugar onde ninguém lê um número de volta.
fn masks_of<'a>(mesh: &'a Mesh, scratch: &'a mut Vec<f32>) -> &'a [f32] {
    match mesh.masks() {
        Some(m) => m,
        None => {
            scratch.clear();
            scratch.resize(mesh.vert_count(), ph2d_mesh::DEFAULT_MASK);
            scratch
        }
    }
}

impl MeshRenderer {
    /// O formato do depth-buffer da cena 3D.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// O formato do **G-buffer** da doação: `xyz` = normal no espaço do rig, `w` = cobertura.
    ///
    /// ⚠️ Ponto flutuante, e não um formato normalizado: as componentes de uma normal vivem em
    /// `[-1, 1]`, e um unorm exigiria codificar `n * 0.5 + 0.5` de um lado e decodificar do outro —
    /// duas metades que precisam concordar, num canal onde a discordância é uma luz levemente torta que
    /// ninguém consegue nomear.
    pub const GBUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    /// **A pose de um objeto** — onde ele está no mundo.
    ///
    /// Aceita um índice além da lista de malhas (a cena pode autorar a pose
    /// antes de a geometria subir); o excedente é ignorado no desenho.
    pub fn set_pose(&mut self, index: usize, pose: Pose) {
        if self.poses.len() <= index {
            self.poses.resize(index + 1, Pose::IDENTITY);
        }
        self.poses[index] = pose;
    }

    /// Quantos objetos o device tem.
    #[must_use]
    pub fn object_count(&self) -> usize {
        self.slots.len()
    }

    /// Esquece os objetos a partir de `n`.
    ///
    /// ⚠️ Existe porque apagar um objeto da cena **sem** isto o deixaria
    /// desenhado para sempre: os `slots` são a verdade do device, e uma lista
    /// que só cresce descreveria uma cena que o artista já desmontou.
    pub fn truncate_objects(&mut self, n: usize) {
        self.slots.truncate(n);
        self.poses.truncate(n);
    }

    /// Sobe a malha do objeto `index`. Reusa os buffers quando cabem — um dab
    /// que só move vértices não realoca nada.
    pub fn upload_at(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        index: usize,
        mesh: &Mesh,
    ) {
        mesh.triangle_indices(&mut self.scratch_indices);
        let verts = mesh.vert_count();
        let idx = self.scratch_indices.len();
        let index_count = (idx * 3) as u32;

        let fits = self
            .slots
            .get(index)
            .is_some_and(|s| s.gpu.vert_capacity >= verts && s.gpu.index_capacity >= idx);

        if fits {
            let g = &mut self
                .slots
                .get_mut(index)
                .expect("acabou de ser conferido")
                .gpu;
            queue.write_buffer(&g.positions, 0, bytemuck::cast_slice(mesh.positions()));
            queue.write_buffer(&g.normals, 0, bytemuck::cast_slice(mesh.normals()));
            queue.write_buffer(
                &g.masks,
                0,
                bytemuck::cast_slice(masks_of(mesh, &mut self.scratch_masks)),
            );
            queue.write_buffer(&g.curvatures, 0, bytemuck::cast_slice(mesh.curvatures()));
            queue.write_buffer(&g.indices, 0, bytemuck::cast_slice(&self.scratch_indices));
            g.index_count = index_count;
            // ⚠️ **A topologia pode ter mudado, então as arestas morrem aqui.**
            // Esta rota é a do buffer que CABE — e caber é sobre tamanho, não
            // sobre as faces serem as mesmas: um remesh que devolva a mesma
            // contagem reusa os buffers e reescreve os índices. Uma lista de
            // arestas sobrevivente descreveria a malha anterior, e o wireframe
            // desenharia um grafo que a superfície não tem.
            g.wire = None;
            g.wire_count = 0;
            return;
        }

        let vb = |label, data: &[u8]| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: data,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
        };
        let gpu = MeshGpu {
            positions: vb("ph2d-mesh pos", bytemuck::cast_slice(mesh.positions())),
            normals: vb("ph2d-mesh nrm", bytemuck::cast_slice(mesh.normals())),
            masks: vb(
                "ph2d-mesh mask",
                bytemuck::cast_slice(masks_of(mesh, &mut self.scratch_masks)),
            ),
            curvatures: vb("ph2d-mesh curv", bytemuck::cast_slice(mesh.curvatures())),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ph2d-mesh idx"),
                contents: bytemuck::cast_slice(&self.scratch_indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }),
            index_count,
            vert_capacity: verts,
            index_capacity: idx,
            wire: None,
            wire_count: 0,
        };
        if let Some(slot) = self.slots.get_mut(index) {
            // O bind group e o buffer de pose sobrevivem: só a geometria mudou.
            slot.gpu = gpu;
            return;
        }
        // Um objeto novo. ⚠️ Só o PRÓXIMO da lista pode nascer aqui — um índice
        // salteado deixaria um buraco que o desenho percorreria como se fosse
        // um objeto, e a pose de todos depois dele apontaria para a malha
        // errada. A cena constrói a lista em ordem; se este `if` disparar, é
        // porque alguém a construiu de outro jeito.
        if index != self.slots.len() {
            return;
        }
        let model = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh model"),
            size: size_of::<ObjectRaw>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh object bind"),
            layout: &self.obj_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model.as_entire_binding(),
            }],
        });
        self.slots.push(Slot { gpu, model, bind });
        if self.poses.len() < self.slots.len() {
            self.poses.resize(self.slots.len(), Pose::IDENTITY);
        }
    }

    /// Sobe **só os vértices que um dab moveu** — posições e normais.
    ///
    /// Devolve `false` quando não há com que reconciliar (nada subiu ainda, ou a
    /// contagem de vértices mudou), e aí o chamador cai no [`Self::upload`]
    /// cheio. Recusar é a resposta certa: subir uma região sobre um buffer de
    /// outra topologia escreveria bytes válidos nos vértices errados, e o
    /// sintoma seria geometria puxada para lugares que ninguém tocou.
    ///
    /// ⚠️ **Os ÍNDICES não são reenviados**, e é isso que torna o caminho barato:
    /// um dab move vértices e não muda a topologia. Quem mudar a topologia
    /// (remesh, subdivisão, dyntopo — a W4) passa pelo `upload` cheio.
    ///
    /// ⚠️ **A normal viaja junto com a posição, sempre.** São dois buffers, mas
    /// um fato só: o `refresh_region` já recomputou as normais de todo vértice
    /// que este `moved` contém, e subir metade deixaria a malha iluminada por
    /// normais de antes do dab — um erro que só aparece na tela, nunca num
    /// número.
    pub fn upload_region_at(
        &mut self,
        queue: &wgpu::Queue,
        index: usize,
        mesh: &Mesh,
        moved: &[u32],
    ) -> bool {
        let Some(slot) = self.slots.get(index) else {
            return false;
        };
        if slot.gpu.vert_capacity != mesh.vert_count() {
            return false;
        }
        if moved.is_empty() {
            return true;
        }
        self.scratch_moved.clear();
        self.scratch_moved.extend_from_slice(moved);
        self.scratch_moved.sort_unstable();
        self.scratch_moved.dedup();
        upload::plan_runs(&self.scratch_moved, &mut self.scratch_runs);
        let masks = masks_of(mesh, &mut self.scratch_masks);
        let g = &self.slots.get(index).expect("conferido acima").gpu;

        let stride = size_of::<[f32; 3]>() as u64;
        for &(a, b) in &self.scratch_runs {
            let (a, b) = (a as usize, (b as usize).min(mesh.vert_count()));
            if a >= b {
                continue;
            }
            let at = a as u64 * stride;
            queue.write_buffer(
                &g.positions,
                at,
                bytemuck::cast_slice(&mesh.positions()[a..b]),
            );
            queue.write_buffer(&g.normals, at, bytemuck::cast_slice(&mesh.normals()[a..b]));
            // ⚠️ **A máscara viaja na MESMA janela, sempre.** Um dab é de
            // geometria ou de máscara — nunca dos dois — então uma das duas
            // cópias reescreve bytes idênticos, e a alternativa (perguntar de
            // qual canal é este dab) seria o chamador conhecendo a regra que o
            // `last_gpu_dirty` existe para responder.
            queue.write_buffer(&g.masks, a as u64 * 4, bytemuck::cast_slice(&masks[a..b]));
            // **E a CURVATURA também**, pela mesma janela e pelo mesmo motivo.
            // ⚠️ Aqui ela é ainda mais obrigatória que a máscara: a lista que
            // chega é o `refreshed()` do `RegionScratch`, que é exatamente o
            // conjunto cuja curvatura foi recomputada. Deixá-la de fora daria uma
            // superfície cujo RELEVO é novo e cuja LEITURA de cavidade é a de
            // antes do traço — a fresta desenhada onde ela estava, no lugar exato
            // em que o artista está olhando.
            queue.write_buffer(
                &g.curvatures,
                a as u64 * 4,
                bytemuck::cast_slice(&mesh.curvatures()[a..b]),
            );
        }
        true
    }

    /// **Garante a lista de arestas do objeto `index`** — o buffer que o passe
    /// de wireframe desenha. No-op se ela já existe.
    ///
    /// ⚠️ **Porta SEPARADA do [`Self::upload_at`], e a razão é quem tem o quê.**
    /// A lista sai da adjacência da `Mesh`, que o device não guarda; e ela custa
    /// até 24 B por vértice, que ninguém deve pagar com o wireframe desligado.
    /// Então quem chama é a cena, no frame em que o artista arma a malha — e o
    /// custo aparece uma vez, ali, em vez de em todo `upload_at` de todo dab que
    /// muda topologia.
    ///
    /// Devolve `false` quando não há slot (a cena ainda não subiu o objeto).
    pub fn upload_wire_at(&mut self, device: &wgpu::Device, index: usize, mesh: &Mesh) -> bool {
        let Some(slot) = self.slots.get(index) else {
            return false;
        };
        if slot.gpu.wire.is_some() {
            return true;
        }
        crate::wire_indices(mesh, &mut self.scratch_indices_flat);
        let count = u32::try_from(self.scratch_indices_flat.len()).unwrap_or(u32::MAX);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ph2d-mesh wire"),
            contents: bytemuck::cast_slice(&self.scratch_indices_flat),
            usage: wgpu::BufferUsages::INDEX,
        });
        let g = &mut self.slots.get_mut(index).expect("conferido acima").gpu;
        g.wire = Some(buffer);
        g.wire_count = count;
        true
    }

    /// Quantos vértices o último [`Self::upload_region`] de fato copiou — a
    /// grandeza que separa "subi a pegada" de "subi a malha", e que um gate
    /// consegue ler sem device.
    #[must_use]
    pub fn last_region_verts(&self) -> u32 {
        upload::covered(&self.scratch_runs)
    }

    /// Há geometria para desenhar? (Qualquer objeto da lista serve.)
    #[must_use]
    pub fn has_mesh(&self) -> bool {
        self.slots.iter().any(|s| s.gpu.index_count > 0)
    }

    /// Escreve as poses e desenha **cada objeto**, com o pipeline dado.
    ///
    /// ⚠️ Porta única dos dois passes (a cena e o G-buffer), e por um motivo que
    /// já mordeu este arquivo: eles diferem em DOIS campos e em nada mais, então
    /// uma segunda cópia do laço seria a segunda resposta a *"o que este
    /// renderizador desenha"* — e o dia em que a lista ganhasse um objeto num
    /// dos dois, o G-buffer descreveria uma silhueta que a tela não mostra.
    fn draw_all(&self, queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'_>, what: Draw) {
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.gpu.index_count == 0 {
                continue;
            }
            // O passe de arestas pula quem ainda não a tem: a lista é construída
            // sob demanda, e um objeto que entrou na cena depois de o wireframe
            // ser armado a ganha no frame seguinte. Desenhar nada é a resposta
            // certa para *"ainda não sei quais são as arestas"*.
            let (indices, count) = match what {
                Draw::Faces => (&slot.gpu.indices, slot.gpu.index_count),
                Draw::Wire => match (&slot.gpu.wire, slot.gpu.wire_count) {
                    (Some(b), n) if n > 0 => (b, n),
                    _ => continue,
                },
            };
            let pose = self.poses.get(i).copied().unwrap_or(Pose::IDENTITY);
            queue.write_buffer(
                &slot.model,
                0,
                bytemuck::bytes_of(&ObjectRaw {
                    model: pose.to_cols_array_2d(),
                }),
            );
            pass.set_bind_group(1, &slot.bind, &[]);
            pass.set_vertex_buffer(0, slot.gpu.positions.slice(..));
            pass.set_vertex_buffer(1, slot.gpu.normals.slice(..));
            pass.set_vertex_buffer(2, slot.gpu.masks.slice(..));
            pass.set_vertex_buffer(3, slot.gpu.curvatures.slice(..));
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..count, 0, 0..1);
        }
    }

    /// Garante o depth-buffer do tamanho pedido (recria se mudou).
    pub fn ensure_depth(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        if self.depth.is_some() && self.depth_size == size {
            return;
        }
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-mesh depth"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.depth = Some(tex.create_view(&wgpu::TextureViewDescriptor::default()));
        self.depth_size = size;
    }

    /// Desenha a malha em `color_view`, PRESERVANDO o que já está lá
    /// (`LoadOp::Load`) — a cena 2D fica por baixo. No-op sem geometria.
    ///
    /// `rig` são as lâmpadas do ARTISTA, resolvidas ([`ph2d_light::resolve`]) — as mesmas que acendem
    /// a tinta. `None` é o artista com todas as luzes apagadas, e o shader devolve o barro cru.
    ///
    /// `cavity` é **quanto a curvatura escurece a fresta e clareia a crista**
    /// (`crate::shade`). `0` devolve o barro liso da W3, ao byte.
    ///
    /// ⚠️ Os dois são passados por frame, não guardados: o rig e a cavidade são do DOCUMENTO, e uma
    /// cópia aqui seria uma segunda verdade sobre como a cena é acesa — a que fica velha no frame
    /// seguinte ao card mudar.
    // Nove argumentos, e nenhum deles é agrupável sem inventar um tipo que existiria só para o
    // clippy: quatro são as alças do device (que o chamador possui) e cinco são o frame. O precedente
    // é o `body_desc` da `line/physics`.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        camera: &Camera3d,
        rig: Option<&ph2d_light::ResolvedRig>,
        shade: crate::Shade,
        size: (u32, u32),
    ) {
        if !self.has_mesh() || size.0 == 0 || size.1 == 0 {
            return;
        }
        self.ensure_depth(device, size);

        let aspect = size.0 as f32 / size.1 as f32;
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&CameraRaw {
                view_proj: camera.view_proj(aspect).to_cols_array_2d(),
                view: camera.view().to_cols_array_2d(),
            }),
        );
        queue.write_buffer(&self.rig_uniform, 0, bytemuck::bytes_of(&RigRaw::pack(rig)));
        queue.write_buffer(
            &self.shade_uniform,
            0,
            bytemuck::bytes_of(&ShadeRaw::pack(shade)),
        );

        let depth = self.depth.as_ref().expect("ensure_depth acabou de rodar");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-mesh pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind, &[]);
        self.draw_all(queue, &mut pass, Draw::Faces);
        // **AS ARESTAS, no MESMO passe de render.** Um segundo `begin_render_pass`
        // teria de recarregar cor e profundidade, e a profundidade que as linhas
        // precisam consultar é *a que este passe acabou de escrever* — carregá-la
        // de volta é trabalho de banda por nada. Trocar o pipeline no meio é a
        // operação que existe exatamente para isto.
        if shade.wireframe {
            pass.set_pipeline(&self.wire_pipeline);
            self.draw_all(queue, &mut pass, Draw::Wire);
        }
    }

    /// **A DOAÇÃO, do lado de quem doa** — rasteriza a malha num G-buffer de normais.
    ///
    /// `xyz` de cada texel é a normal no espaço do RIG (a mesma que o barro usa para se acender —
    /// porta única no shader, `canvas_normal`) e `w` é a **cobertura**: `1` onde há forma, `0` onde
    /// não há.
    ///
    /// É isto que `docs/3D/05.2` chama de *"uma SEGUNDA FONTE DE NORMAL para o sistema que já
    /// existe"*: o passe de luz da tinta deriva a normal de `∇h`; com este plano na mão ele pode
    /// escolher a fonte **por pixel**, e é a cobertura que torna essa escolha possível.
    ///
    /// ⚠️ **Limpa o alvo, ao contrário do [`Self::render`]**, e a assimetria é o ponto: o passe de cor
    /// preserva o que está embaixo (`LoadOp::Load`, a cena 2D), enquanto um G-buffer com resíduo do
    /// frame anterior descreveria uma forma que saiu de quadro. Um plano de normais é uma MEDIÇÃO, e
    /// uma medição velha é pior que medição nenhuma — a cobertura diria `1` onde não há forma.
    ///
    /// No-op sem geometria.
    pub fn render_gbuffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        normal_view: &wgpu::TextureView,
        camera: &Camera3d,
        size: (u32, u32),
    ) {
        if !self.has_mesh() || size.0 == 0 || size.1 == 0 {
            return;
        }
        self.ensure_depth(device, size);
        let aspect = size.0 as f32 / size.1 as f32;
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&CameraRaw {
                view_proj: camera.view_proj(aspect).to_cols_array_2d(),
                view: camera.view().to_cols_array_2d(),
            }),
        );

        let depth = self.depth.as_ref().expect("ensure_depth acabou de rodar");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-mesh gbuffer pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: normal_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.gbuffer_pipeline);
        pass.set_bind_group(0, &self.bind, &[]);
        self.draw_all(queue, &mut pass, Draw::Faces);
    }
}

/// A matriz que o uniform carrega, exposta para o gate poder afirmar o que sobe
/// ao device sem abrir uma render pass.
#[must_use]
pub fn camera_uniform_bytes(camera: &Camera3d, aspect: f32) -> [u8; 128] {
    let raw = CameraRaw {
        view_proj: camera.view_proj(aspect).to_cols_array_2d(),
        view: camera.view().to_cols_array_2d(),
    };
    let mut out = [0u8; 128];
    out.copy_from_slice(bytemuck::bytes_of(&raw));
    out
}

/// A matriz de vista-projeção que o shader recebe, reconstruída dos bytes.
///
/// Existe para o gate provar que a coluna-major do `glam` é a coluna-major que
/// o WGSL espera — trocar isso transpõe a cena e nada avisa.
///
/// ⚠️ **O gate que esta frase prometia não existia até 2026-08-02**, e esta e a
/// [`camera_uniform_bytes`] eram duas `pub fn` sem chamador nenhum: um doc
/// afirmando um consumidor que não havia. Ele agora é o
/// `tests/camera_uniform_layout.rs`, e a metade que lhe dá dentes é a câmera
/// **enviesada** — numa câmera de frente a matriz é quase simétrica, e uma
/// transposição passaria em metade das entradas.
#[must_use]
pub fn view_proj_from_bytes(bytes: &[u8; 128]) -> Mat4 {
    let mut cols = [0f32; 16];
    for (i, c) in cols.iter_mut().enumerate() {
        *c = f32::from_ne_bytes([
            bytes[i * 4],
            bytes[i * 4 + 1],
            bytes[i * 4 + 2],
            bytes[i * 4 + 3],
        ]);
    }
    Mat4::from_cols_array(&cols)
}
