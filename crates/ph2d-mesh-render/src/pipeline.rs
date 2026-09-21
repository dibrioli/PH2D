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

/// O passe de tela cheia do AO de tela — ver [`crate::ssao`].
pub(crate) const SSAO_WGSL: &str = include_str!("shaders/ssao.wgsl");

/// O uniform da câmera. `mat4x4` alinha em 16 B, então não há padding a declarar.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct CameraRaw {
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    /// `(largura, altura, _, _)` em pixels — o que converte um deslocamento em
    /// PIXELS num deslocamento em NDC.
    ///
    /// ⚠️ Ele mora na CÂMERA e não no `Shade` porque é da VISTA, não do
    /// sombreamento: quem o lê pergunta *"quanto vale um pixel aqui?"*, que é a
    /// mesma pergunta que a projeção ao lado responde. Os dois últimos canais
    /// existem só para o alinhamento de 16 B que o WGSL exige de um `vec4`.
    viewport: [f32; 4],
}

/// `(largura, altura, 0, 0)` — ver [`CameraRaw::viewport`].
/// ⭐ **PRENDE UM PASSE A UMA VISTA** — o viewport (que transforma) e o scissor
/// (que recorta), sempre os dois. Ver [`Renderer::render_in`].
///
/// ⚠️ **Uma porta e não duas linhas em cada passe:** os três passes desta crate
/// têm de concordar sobre onde a vista está, e o modo de falha de um deles
/// discordar é oclusão medida num sítio e pintada noutro.
///
/// ⚠️ **Os DOIS, e não só o viewport:** o viewport mapeia o NDC ao rectângulo e
/// **não corta** — geometria que caia fora dele continuaria a escrever nos
/// vizinhos. Quem corta é o scissor.
pub(crate) fn set_area(pass: &mut wgpu::RenderPass<'_>, area: crate::ScreenRect, size: (u32, u32)) {
    // ⛔⛔ **O ALVO É ARGUMENTO, e é o que torna esta porta impossível de
    // chamar errado** — ver [`crate::ScreenRect::clip_to`], que nasceu de um
    // `panic` do dono ao desacoplar a janela maximizada. Os três passes desta
    // crate passam por aqui, e nenhum deles pode voltar a publicar um
    // rectângulo maior que a superfície.
    //
    // ⚠️ **O `set_viewport` é recortado TAMBÉM, e não só o scissor:** o `wgpu`
    // valida os dois contra o alvo. O preço é um quadro de transição com a peça
    // ligeiramente achatada — invisível ao lado de um `panic`.
    let Some(area) = area.clip_to(size) else {
        return;
    };
    pass.set_viewport(
        area.x as f32,
        area.y as f32,
        area.w as f32,
        area.h as f32,
        0.0,
        1.0,
    );
    pass.set_scissor_rect(area.x, area.y, area.w, area.h);
}

/// **A RÉGUA DA ÁREA** — `(largura, altura, x, y)` em pixels do ALVO.
///
/// ⚠️⚠️ **O `zw` deixou de ser padding em 2026-09-21, e a ORIGEM é obrigatória para quem converte
/// `@builtin(position)` em coordenada da ÁREA.** O `set_viewport` move o rasterizador e **não** move
/// a aritmética do shader: um fragmento dentro da área chega com a posição do ALVO, logo sem a
/// origem toda conta normalizada descreveria outro sítio da peça. *A mesma lição que o passe de
/// SSAO já tinha pago, escrita no doc dele.*
pub(crate) fn viewport_of(area: crate::ScreenRect) -> [f32; 4] {
    [
        area.w.max(1) as f32,
        area.h.max(1) as f32,
        area.x as f32,
        area.y as f32,
    ]
}

/// **O QUE UM OBJECTO OCUPA NO DEVICE** — ver [`slot`].
#[path = "pipeline_slot.rs"]
mod slot;

use slot::{Draw, MeshGpu, ObjectRaw, Slot};

/// **COMO o passe é montado** — ver [`build`].
#[path = "pipeline_build.rs"]
mod build;

/// **O QUE TEM DE ESTAR NO DEVICE** antes de desenhar — ver [`ensure`].
#[path = "pipeline_ensure.rs"]
mod ensure;

/// O renderizador da malha.
/// ⭐ **A DOAÇÃO** — irmão (`#[path]`) cortado por responsabilidade: aqui o passe
/// que desenha a peça, lá o que entrega o plano de normais a quem pinta.
#[path = "pipeline_gbuffer.rs"]
mod gbuffer;

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
    /// O material e o céu que o modo `Pbr` lê — ver [`crate::pbr`].
    pbr_uniform: wgpu::Buffer,
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
    /// Zeros para um objeto sem preview armado — ver `preview_of`.
    scratch_preview: Vec<f32>,
    scratch_ao: Vec<f32>,
    scratch_thickness: Vec<f32>,
    /// O rascunho da COR — irmão do [`Self::scratch_ao`], e `[f32; 3]` porque
    /// o canal é um albedo. Ver `colors_of`.
    scratch_colors: Vec<[f32; 3]>,
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
    /// O bind do grupo 3 — a tabela, o sampler e a imagem do matcap.
    ///
    /// ⚠️ **Construído uma vez, inclusive quando o matcap troca.** Trocar de
    /// chip reescreve os TEXELS da mesma textura (`ensure_matcap`), e um bind
    /// aponta para a textura, não para o conteúdo dela — recriá-lo aqui seria
    /// trabalho que ninguém pediu.
    sss_bind: wgpu::BindGroup,
    /// A tabela já foi ASSADA e subiu? Ver [`Self::ensure_sss_lut`].
    sss_lut_ready: bool,
    /// **A imagem do matcap**, no device — ver [`crate::matcap`].
    matcap_tex: wgpu::Texture,
    /// **QUAL matcap está residente**, no índice de [`crate::matcap::MATCAPS`].
    ///
    /// `None` é *"a textura está vazia"* — o estado em que ela nasce, e o que
    /// faz o primeiro `render` com matcap ligado pagar o upload. Guardar o
    /// índice (e não um `bool`) é o que torna a troca de chip barata: o
    /// `ensure_matcap` compara e sai sem tocar no device quando nada mudou, que
    /// é TODO frame menos aquele em que o artista clicou.
    matcap_ready: Option<usize>,
    /// O layout e o sampler do grupo 3, guardados para o bind poder ser
    /// RECONSTRUÍDO quando a imagem do matcap muda de lado.
    ///
    /// ⚠️ **Só o LADO obriga a reconstruir**, não a troca de matcap: reescrever
    /// os texels de uma textura não move o bind, e é por isso que trocar entre
    /// dois matcaps do mesmo tamanho continua custando um `write_texture` e mais
    /// nada.
    sss_bgl: wgpu::BindGroupLayout,
    sss_sampler: wgpu::Sampler,
    /// ⭐⭐⭐ **A FONTE DO ALBEDO** — os pixels que o BAKE vai acender, no device.
    ///
    /// ⚠️ **Ela existe porque o visor e a sprite tinham albedos DIFERENTES**, e essa era a última
    /// coisa a separá-los: medido em 2026-09-21, a lei, o enquadramento e a oclusão de tela somavam
    /// `0,52` códigos de desvio e o albedo sozinho valia **`31,68`** (`61×`). *Nenhuma correcção de
    /// LUZ fecha uma diferença de MATÉRIA.*
    ///
    /// Ela nasce `1×1` BRANCA e inerte: sem fonte posta, o modo `Pbr` continua a pintar o `CLAY` do
    /// shader, byte a byte como antes.
    albedo_tex: wgpu::Texture,
    /// O tamanho da fonte, para a projecção saber o aspecto dela. `None` = ninguém a pôs.
    albedo_size: Option<(u32, u32)>,
}

/// **A textura de um matcap de lado `side`** — a porta ÚNICA, para o `new` e o
/// `ensure_matcap` não descreverem a mesma textura de dois jeitos.
///
/// ⚠️ **`Rgba16Float`, e o formato é a wave inteira:** os matcaps do Blender são
/// meio-float na fonte, e guardá-los em 8 bits erra ~1 nível de 255 de volta em
/// linear (medido — ver [`crate::matcap`]). Como o conteúdo é LINEAR, não há
/// `…Srgb` a pedir: a conversão dos que nascem sRGB acontece na decodificação.
pub(crate) fn matcap_texture(device: &wgpu::Device, side: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-mesh matcap"),
        size: wgpu::Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// **A TEXTURA DA FONTE DO ALBEDO** — a porta única, para o nascimento e a troca de tamanho não a
/// descreverem de dois jeitos.
///
/// ⭐⭐ **`Rgba8UnormSrgb`, como o matcap — e a razão continua a ser a LEI do bake.**
///
/// A [`ph2d_form_pbr::imagem`] **descodifica** o byte da sprite (`codigo::para_luz`) e **codifica**
/// o resultado: para ela os pixels de um objecto assado são **códigos sRGB**, que é a convenção
/// declarada da ranhura onde eles vão viver. ⇒ aqui o formato tem de deixar o hardware
/// descodificar, senão o shader recebe um código onde a lei irmã recebe luz.
///
/// ⛔⛔⛔ **E até 2026-09-20 este ficheiro dizia o CONTRÁRIO, com a premissa escrita:** *«a
/// `ph2d_form_pbr::imagem` lê o byte como `px / 255,0` … para ela os pixels são LINEARES»*. Era
/// **verdade no dia em que foi escrita** e morreu no dia em que o assado passou a atravessar a
/// curva nas duas pontas.
///
/// ⭐⭐⭐ **E o mesmo parágrafo já nomeava o defeito que a troca em falta produziria:** *«o modo
/// de falha seria MUDO e para o lado errado — a peça no visor sairia mais CLARA que a sprite, sem
/// erro nenhum»*. Foi exactamente isso que aconteceu, e mediu **`25` códigos de oito bits** sobre
/// uma matéria fria e clara — ⚠️ **e ZERO sobre uma tela BRANCA**, que é ponto fixo da curva e o
/// enquadramento em que o dono estava a olhar. *Quem espelha uma lei copia a CONVENÇÃO dela, não a
/// do vizinho — e quando a convenção da lei muda, o espelho muda com ela.*
///
/// ⚠️ **Quem o apanhou** foi o `os_dois_lados_leem_o_mesmo_byte_no_ecra`, que lê os DOIS lados
/// depois do [`ph2d_render::Tonemap`]; ⛔ o irmão que compara em VALORES não o podia ver.
pub(crate) fn albedo_texture(device: &wgpu::Device, size: (u32, u32)) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-mesh albedo source"),
        size: wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// COMO A MALHA SOBE para o device — ver o módulo.
/// ⭐⭐ **A tinta fina no device.** Irmã do [`upload`] e pelo mesmo motivo:
/// ela toca os `slots`, que são privados a este módulo.
#[path = "tinta_gpu.rs"]
pub mod tinta_gpu;

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

    /// O formato do **segundo** alvo da doação: a OCLUSÃO DE FORMA, um escalar.
    ///
    /// ⚠️ **Um alvo próprio e não um canal do primeiro**, porque o primeiro não tem vaga — `xyz` é a
    /// normal e `w` é a cobertura. A tentação é reconstruir `z` por `sqrt(1 − x² − y²)` (o
    /// `canvas_normal` vira a normal para a frente, então o sinal é conhecido) e usar a vaga: isso
    /// **funciona na álgebra e falha na SILHUETA**, onde `z → 0` e a derivada da raiz vai a infinito —
    /// a mesma classe de mal-condicionamento que o doc 24 do Painter mediu e recusou ao fundir a razão
    /// K/S. Aqui o sintoma seria uma normal levemente torta exatamente na borda da peça.
    ///
    /// **Preço: 2 B/texel contra os 8 do primeiro** (+25% de leitura de volta), e o número que ele move
    /// está medido no [`Self::form_plane`].
    pub const OCCLUSION_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

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
                    wire_cull: if slot.gpu.closed { 1.0 } else { 0.0 },
                    _pad: [0.0; 3],
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
            pass.set_vertex_buffer(7, slot.gpu.preview.slice(..));
            pass.set_vertex_buffer(8, slot.gpu.colors.slice(..));
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..count, 0, 0..1);
        }
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
        self.render_in(
            device,
            queue,
            encoder,
            color_view,
            camera,
            rig,
            shade,
            size,
            crate::ScreenRect::full(size),
        );
    }

    /// ⭐⭐⭐ **DESENHA N VISTAS** — a porta dos quatro viewports, e a **única**
    /// forma correcta de desenhar mais do que uma.
    ///
    /// ⛔⛔⛔ **REPORT DO ENIO, 2026-09-08:** *«com 4 views o mesh não está
    /// correspondendo às views e ao tentar esculpir nas outras views o pincel
    /// tem drift ou offset»* — **dois sintomas, uma causa.**
    ///
    /// Este renderizador tem **UM** buffer de uniform de câmera, e o
    /// [`Self::render_in`] escreve-o com `queue.write_buffer` antes de gravar o
    /// passe. ⚠️⚠️ **Mas `write_buffer` não é gravado no encoder:** ele é
    /// agendado na **fila**, e toda escrita feita antes de um `submit` acontece
    /// antes de **qualquer** comando desse submit. ⇒ quatro `render_in` num
    /// encoder só desenham as quatro vistas com a câmera da **última**.
    ///
    /// E é isso que produz os dois sintomas de uma vez: a imagem de um
    /// quadrante é a da última câmera, e o **pick** daquele quadrante usa a
    /// câmera **dele** — logo o pincel cai onde a peça *estaria* e não onde ela
    /// *está desenhada*.
    ///
    /// ⇒ **um encoder e um `submit` POR VISTA**, aqui dentro. As escritas de
    /// cada vista ficam entre dois submits, que é o que as põe na ordem certa.
    ///
    /// ⚠️ **A porta existe para o perigo não ter como ser repetido.** Um
    /// contrato em prosa a dizer *«submeta entre as chamadas»* é exactamente o
    /// tipo de coisa que o próximo chamador não lê — e o modo de falha dele não
    /// dá erro nenhum, dá uma imagem plausível na vista errada.
    ///
    /// ⚠️ **Medir e desenhar alternam DENTRO de cada vista**, e não em duas
    /// varreduras: a frescura do AO (`ssao_fresh`) é uma para o renderizador
    /// inteiro e é o `render_in` que a consome.
    ///
    /// `ssao` a `None` salta a medição de oclusão — é o que o `Shade::ssao == 0`
    /// pede.
    #[allow(clippy::too_many_arguments)]
    pub fn render_views(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_view: &wgpu::TextureView,
        camera_by_view: &[(crate::ScreenRect, Camera3d)],
        rig: Option<&ph2d_light::ResolvedRig>,
        shade: crate::Shade,
        size: (u32, u32),
        ssao: Option<crate::SsaoParams>,
    ) {
        for (area, cam) in camera_by_view {
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-mesh view"),
            });
            if let Some(p) = ssao {
                self.render_ssao_in(device, queue, &mut enc, cam, p, size, *area);
            }
            self.render_in(
                device, queue, &mut enc, color_view, cam, rig, shade, size, *area,
            );
            queue.submit([enc.finish()]);
        }
    }

    /// ⭐⭐⭐ **O MESMO, NUM SUB-RECTÂNGULO DO ALVO** — a metade que os quatro
    /// viewports da escultura pedem (ordem do Enio, 2026-09-08).
    ///
    /// `size` continua a ser o **alvo** (é ele que dimensiona a profundidade, que
    /// tem de casar com a cor); `area` é **onde** desenhar.
    ///
    /// ⚠️⚠️ **A profundidade é limpa no passe INTEIRO, e isso é correcto e
    /// load-bearing:** um `LoadOp::Clear` não obedece ao scissor, então a vista
    /// `k+1` apaga a profundidade da `k`. Ela pode: a cor da `k` já foi escrita
    /// (`LoadOp::Load` preserva-a) e ninguém volta a testar profundidade contra
    /// ela. *Partilhar um buffer de profundidade entre vistas seria errado; o que
    /// se partilha é o buffer, não os valores.*
    ///
    /// ⚠️ **`set_scissor_rect` E `set_viewport`, os dois.** O viewport mapeia o
    /// NDC ao rectângulo (é ele que faz a peça caber na vista) e o scissor
    /// **recorta**: sem o segundo, geometria fora do frustum lateral ainda
    /// escreveria nos vizinhos, porque o viewport não corta, só transforma.
    ///
    /// ⛔⛔⛔ **DUAS CHAMADAS DESTAS NO MESMO ENCODER DESENHAM AS DUAS COM A
    /// ÚLTIMA CÂMERA** — o uniform é **um** e o `queue.write_buffer` corre na
    /// fila, não no encoder. Para mais do que uma vista use o
    /// [`Self::render_views`], que submete entre elas. *(Report do Enio,
    /// 2026-09-08; gate `duas_vistas_no_mesmo_quadro_mostram_duas_cameras`.)*
    #[allow(clippy::too_many_arguments)]
    pub fn render_in(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        camera: &Camera3d,
        rig: Option<&ph2d_light::ResolvedRig>,
        shade: crate::Shade,
        size: (u32, u32),
        area: crate::ScreenRect,
    ) {
        // ⛔ **A área é RECORTADA ao alvo à entrada** — ver
        // [`crate::ScreenRect::clip_to`]. Sem vista não se desenha: um quadro
        // de redimensionamento em que o painel ainda publica o rectângulo da
        // janela maximizada punha o `wgpu` a entrar em pânico (report do dono,
        // 2026-09-14).
        let Some(area) = area.clip_to(size) else {
            return;
        };
        if !self.has_mesh() || size.0 == 0 || size.1 == 0 {
            return;
        }
        self.ensure_depth(device, size);
        self.ensure_sss_lut(device, queue, shade);
        self.ensure_matcap(device, queue, shade);
        // ⚠️ **A frescura é consumida AQUI**, antes de qualquer empréstimo do
        // passe: quem não renovou a medição nesta vista desenha sem oclusão de
        // tela, nunca com a do frame passado.
        let fresh = std::mem::take(&mut self.ssao_fresh);

        // ⚠️ **O aspecto e o uniform de viewport são os da VISTA**, nunca os do
        // alvo: o segundo é o que converte pixels em NDC dentro do shader (o
        // contorno do wireframe, o empurrão do `depth bias`), e num quadrante
        // com metade da largura ele daria o dobro do deslocamento.
        let aspect = area.aspect();
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&CameraRaw {
                view_proj: camera.view_proj(aspect).to_cols_array_2d(),
                view: camera.view().to_cols_array_2d(),
                viewport: viewport_of(area),
            }),
        );
        queue.write_buffer(&self.rig_uniform, 0, bytemuck::bytes_of(&RigRaw::pack(rig)));
        queue.write_buffer(
            &self.shade_uniform,
            0,
            bytemuck::bytes_of(&ShadeRaw::pack(shade)),
        );
        // ⭐⭐⭐ **O MATERIAL e o CÉU que este rig produz** — o modo `Pbr`.
        //
        // ⚠️ **O `prepare()` corre aqui, uma vez por quadro, e isso é a divisão certa:** ele é por
        // MATERIAL e não por pixel, e correr no dispositivo o que já está calculado poria a mesma
        // conta a dar o mesmo número um milhão de vezes (o doc do `ph2d_material::wgsl::pack` diz
        // isso por escrito).
        //
        // ⚠️ **Sem lâmpada acesa o `plano` é zero e a rampa é toda zero** — o céu apaga com o rig,
        // que é a metade que torna a tradução do piso relativo honesta.
        queue.write_buffer(
            &self.pbr_uniform,
            0,
            bytemuck::bytes_of(&crate::pbr::PbrRaw::pack(
                &shade.material.prepare(),
                rig.map_or([0.0; 3], ph2d_light::flat_response),
                shade.look,
                // ⭐⭐⭐ **A FONTE DO ALBEDO, se o app a pôs** — ver [`MeshRenderer::set_albedo_source`].
                // Sem ela o `Pbr` continua a pintar o `CLAY` do shader, byte a byte.
                self.albedo_size().map(|(fw, fh)| {
                    let (vw, vh) = (area.w.max(1) as f32, area.h.max(1) as f32);
                    crate::pbr::Projeccao {
                        razao_dos_aspectos: (vw / vh) / (fw.max(1) as f32 / fh.max(1) as f32),
                        inv_w: 1.0 / vw,
                        inv_h: 1.0 / vh,
                    }
                }),
            )),
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
        set_area(&mut pass, area, size);
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
}

/// A matriz que o uniform carrega, exposta para o gate poder afirmar o que sobe
/// ao device sem abrir uma render pass.
#[must_use]
pub fn camera_uniform_bytes(camera: &Camera3d, aspect: f32, size: (u32, u32)) -> [u8; 144] {
    let raw = CameraRaw {
        view_proj: camera.view_proj(aspect).to_cols_array_2d(),
        view: camera.view().to_cols_array_2d(),
        viewport: viewport_of(crate::ScreenRect::full(size)),
    };
    let mut out = [0u8; 144];
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
pub fn view_proj_from_bytes(bytes: &[u8]) -> Mat4 {
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
