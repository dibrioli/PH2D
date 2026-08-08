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
use ph2d_mesh::Pose;

use crate::camera::Camera3d;
use crate::lighting::RigRaw;
use crate::shade::ShadeRaw;
use crate::upload;

/// O fonte do shader, exposto para o gate poder afirmar o que ele DECLARA sem
/// precisar de device — o molde do `IMPASTO_LIGHT_WGSL` do `ph2d-render`.
pub(crate) const MESH_WGSL: &str = include_str!("shaders/mesh.wgsl");

/// O passe de tela cheia do AO de tela — ver [`crate::ssao`].
pub(crate) const SSAO_WGSL: &str = include_str!("shaders/ssao.wgsl");

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
    /// A CURVATURA DE MUNDO por vértice (`ph2d_mesh::world_curvature_at`) — o
    /// eixo `t = scatter·|κ|` da tabela do SSS. **Derivada como a irmã**, logo
    /// sem `Option`: toda malha tem uma.
    curv_world: wgpu::Buffer,
    thickness: wgpu::Buffer,
    /// O AO ASSADO por vértice — quanto do céu cada um enxerga.
    ///
    /// ⚠️ **Ao contrário da curvatura ao lado, ele é `Option` na malha**, e por
    /// isso passa pelo [`ao_of`]: nem toda peça foi assada, e a que não foi tem
    /// de subir o céu aberto para renderizar como sempre renderizou.
    ao: wgpu::Buffer,
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
    scratch_ao: Vec<f32>,
    scratch_thickness: Vec<f32>,
    // ---- o AO de TELA (`crate::ssao`) ----
    ssao_bgl: wgpu::BindGroupLayout,
    ssao_uniform: wgpu::Buffer,
    ssao_pipeline: wgpu::RenderPipeline,
    /// O layout do grupo 2 do passe de cor: a visibilidade que o barro amostra.
    ao_bgl: wgpu::BindGroupLayout,
    /// **O BRANCO 1×1** — a resposta a *"ninguém mediu oclusão nesta vista"*.
    ///
    /// ⚠️ Ele existe porque o grupo 2 é obrigatório no layout: um passe sem bind
    /// group posto é erro de validação, então a ausência de AO precisa de uma
    /// TEXTURA que diga *nada oclui*. Uma permutação de pipeline diria o mesmo ao
    /// preço de recompilar o shader quando o artista mexe num toggle.
    ao_white_bind: wgpu::BindGroup,
    ssao: Option<ssao_pass::SsaoTargets>,
    /// A medição é DESTA vista? Posta pelo `render_ssao`, consumida pelo `render`.
    ssao_fresh: bool,
    /// **A tabela pré-integrada do SSS** (`crate::sss`), no device.
    sss_lut: wgpu::Texture,
    /// O bind do grupo 3 — a tabela e o sampler. Construído uma vez.
    sss_bind: wgpu::BindGroup,
    /// A tabela já foi ASSADA e subiu? Ver [`Self::ensure_sss_lut`].
    sss_lut_ready: bool,
}

/// COMO A MALHA SOBE para o device — ver o módulo.
#[path = "pipeline_upload.rs"]
mod marshal;

/// OS ALVOS E O PASSE do AO de tela — ver o módulo.
#[path = "pipeline_ssao.rs"]
mod ssao_pass;

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

    /// Quantos vértices o último `upload_region` de fato copiou — a grandeza
    /// que separa "subi a pegada" de "subi a malha", e que um gate consegue ler
    /// sem device.
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
            pass.set_vertex_buffer(4, slot.gpu.ao.slice(..));
            pass.set_vertex_buffer(5, slot.gpu.curv_world.slice(..));
            pass.set_vertex_buffer(6, slot.gpu.thickness.slice(..));
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..count, 0, 0..1);
        }
    }

    /// Garante o depth-buffer do tamanho pedido (recria se mudou).
    /// **Garante a tabela do SSS no device** — assa e sobe, uma vez.
    ///
    /// ⚠️ **Lazy, e só quando alguém de fato pede espalhamento.** A tabela custa
    /// `128² × 512` avaliações da integral de Penner, ou seja **oito milhões de
    /// exponenciais**; assá-la no `new` faria toda cena pagar por um canal que
    /// nasce em zero. E ela não pode ser assada no `new` de qualquer forma: ele
    /// não tem `queue` (a restrição que fez o canal de AO guardar oclusão em vez
    /// de visibilidade), e aqui não há inversão que dispense o upload.
    ///
    /// ⚠️ **O guard é `strength > 0` e não `strength != 0`**: o valor já chegou
    /// clampado a `[0,1]` pela porta, então negativo não existe — e escrever a
    /// desigualdade estrita torna a leitura *"alguém pediu espalhamento"* em vez
    /// de *"o número não é exatamente zero"*.
    pub fn ensure_sss_lut(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shade: crate::Shade,
    ) {
        let _ = device;
        if self.sss_lut_ready || shade.sss.strength <= 0.0 {
            return;
        }
        let n = crate::sss::LUT_SIZE;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.sss_lut,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &crate::sss::bake_lut(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(n * 4),
                rows_per_image: Some(n),
            },
            wgpu::Extent3d {
                width: n,
                height: n,
                depth_or_array_layers: 1,
            },
        );
        self.sss_lut_ready = true;
    }

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
            // ⚠️ **`TEXTURE_BINDING` além do anexo, e não é folga:** o AO de tela
            // LÊ esta textura para saber onde cada pixel está. Sem a flag o wgpu
            // recusa o bind group — e a flag não custa nada a quem não a usa.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.depth = Some(tex.create_view(&wgpu::TextureViewDescriptor::default()));
        self.depth_size = size;
        // ⚠️ **Trocar a profundidade INVALIDA o grupo de entrada do AO de tela**,
        // que aponta para a view antiga. Derrubar os alvos aqui é o que garante
        // que os dois sejam sempre do mesmo enquadramento; deixá-los de pé faria
        // o passe ler uma textura morta — e a validação do wgpu pegaria isso, mas
        // só na máquina de quem redimensionasse a janela.
        self.ssao = None;
        self.ssao_fresh = false;
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
        self.ensure_sss_lut(device, queue, shade);
        // ⚠️ **A frescura é consumida AQUI**, antes de qualquer empréstimo do
        // passe: quem não renovou a medição nesta vista desenha sem oclusão de
        // tela, nunca com a do frame passado.
        let fresh = std::mem::take(&mut self.ssao_fresh);

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
        pass.set_bind_group(2, self.ao_bind_for(fresh), &[]);
        pass.set_bind_group(3, &self.sss_bind, &[]);
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
        // O G-buffer não lê oclusão nenhuma — mas o grupo 2 está no layout que os
        // três pipelines partilham, e um passe com um grupo do layout por pôr é
        // erro de validação. O branco é a resposta inerte.
        pass.set_bind_group(2, &self.ao_white_bind, &[]);
        // ⚠️ **E o grupo 3 pela MESMA razão, não porque ele o leia.** O `fs_gbuffer`
        // devolve a normal e a cobertura e mais nada; a tabela do SSS entra aqui
        // só para o layout ficar completo. Uma decisão de projeto está sendo
        // cobrada: os três pipelines partilham UM layout de propósito (senão o
        // G-buffer poderia divergir da tela sobre culling ou formato de depth), e
        // o preço é este bind inerte.
        pass.set_bind_group(3, &self.sss_bind, &[]);
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
