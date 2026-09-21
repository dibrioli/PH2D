//! **O QUE UM OBJECTO OCUPA NO DEVICE** — os uniforms, os buffers e a ranhura.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é a RESPONSABILIDADE: ali ficam os
//! PASSES (como se desenha), aqui o que cada objecto guarda para ser desenhado.
//!
//! ⛔ Saiu de lá por TECTO DE LOC (`705` contra `700`) quando a lista de arestas
//! passou a guardar **em que vista** foi construída — e o corte é melhor do que
//! o ficheiro era.

use bytemuck::{Pod, Zeroable};

/// O uniform do objeto: **onde ele está**, e **se ele é um sólido**.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ObjectRaw {
    pub(super) model: [[f32; 4]; 4],
    /// **O WIREFRAME PODE REMOVER LINHA ESCONDIDA PELA NORMAL?** `1` só numa
    /// malha FECHADA (`Mesh::is_closed`).
    ///
    /// ⚠️ **Ele é POR-OBJETO e não do quadro**, e a diferença é o produto: uma
    /// única casca aberta na cena devolveria o vazamento a TODAS as peças se a
    /// pergunta morasse no `Shade`.
    ///
    /// ⚠️ **E o default é `0`** — o mundo de antes desta wave, ao byte. Uma malha
    /// cuja lista de arestas nunca subiu não teve a pergunta feita, e o valor
    /// que ela carrega tem de significar *"não sei"*, não *"pode cortar"*.
    pub(super) wire_cull: f32,
    /// A `mat4x4` alinha em 16 B: o `f32` acima abre um vec4 que precisa fechar.
    pub(super) _pad: [f32; 3],
}

/// Os buffers da malha no device.
pub(super) struct MeshGpu {
    pub(super) positions: wgpu::Buffer,
    pub(super) normals: wgpu::Buffer,
    /// A MÁSCARA por vértice — o canal de autoria, `0 = livre`.
    ///
    /// ⚠️ **Um `f32` e não um `vec3` de material.** O SculptGL guarda a máscara
    /// no canal `z` de um atributo de material cujos outros dois canais são
    /// rugosidade e metalness; portá-lo inteiro agora subiria **dois canais que
    /// ninguém escreve e ninguém lê** — 12 B/vértice contra 4, e dois controles
    /// mortos. O material chega com o Paint (W7/W9), e o canal é aditivo.
    pub(super) masks: wgpu::Buffer,
    /// A CURVATURA por vértice — o canal que faz a forma ser LIDA
    /// (`ph2d_mesh::curvature`). ⚠️ **Ela é DERIVADA**, então ao contrário da
    /// máscara não há `Option` a resolver: toda malha tem uma, sempre, e o
    /// buffer sobe direto do slice que já existe.
    pub(super) curvatures: wgpu::Buffer,
    /// A CURVATURA DE MUNDO por vértice (`ph2d_mesh::world_curvature_at`) — o
    /// eixo `t = scatter·|κ|` da tabela do SSS. **Derivada como a irmã**, logo
    /// sem `Option`: toda malha tem uma.
    pub(super) curv_world: wgpu::Buffer,
    pub(super) thickness: wgpu::Buffer,
    /// O PREVIEW do padrão do pincel por vértice — o canal **transiente**.
    ///
    /// ⚠️ **Ele é irmão da máscara, não ela.** Os dois são `f32` e pintam um
    /// tinto, mas a máscara é AUTORADA (o artista a pinta para proteger) e este
    /// é DERIVADO do pincel vivo. Escrever o preview no buffer da máscara
    /// custaria zero VRAM e mostraria o padrão onde o artista espera ver a
    /// proteção — e devolvê-la depois é uma promessa que um `return` esquecido
    /// quebra em silêncio.
    pub(super) preview: wgpu::Buffer,
    /// O AO ASSADO por vértice — quanto do céu cada um enxerga.
    ///
    /// ⚠️ **Ao contrário da curvatura ao lado, ele é `Option` na malha**, e por
    /// isso passa pelo [`ao_of`]: nem toda peça foi assada, e a que não foi tem
    /// de subir o céu aberto para renderizar como sempre renderizou.
    pub(super) ao: wgpu::Buffer,
    /// A COR por vértice — o albedo que os pincéis de pintura escrevem.
    ///
    /// ⚠️ **Irmão do [`ao`](Self::ao) e não da máscara:** ele é `Option` na
    /// malha (uma esfera que ninguém pintou não paga 12 B/vértice) e a ausência
    /// sobe como [`ph2d_mesh::DEFAULT_COLOR`] — **branco**, o neutro do produto
    /// que multiplica —, porque uma peça que ninguém pintou tem de renderizar
    /// EXACTAMENTE como antes de este canal existir. Se a ausência subisse
    /// preta, toda peça nasceria apagada e o artista iria procurar o defeito no
    /// shader.
    ///
    /// ⭐ **Ele é o 9.º buffer de vértice, e o 9.º passa do piso do WebGPU** — o
    /// `max_vertex_buffers` de omissão é `8`. O limite sobe ao máximo do
    /// adaptador em [`ph2d_gpu`], onde a medição desta máquina (`32`) está
    /// escrita ao lado do argumento.
    pub(super) colors: wgpu::Buffer,
    pub(super) indices: wgpu::Buffer,
    pub(super) index_count: u32,
    pub(super) vert_capacity: usize,
    pub(super) index_capacity: usize,
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
    pub(super) wire: Option<wgpu::Buffer>,
    pub(super) wire_count: u32,
    /// **Em que VISTA a lista de arestas de cima foi construída.**
    ///
    /// ⚠️⚠️ **Sem ela o interruptor da vista seria MUDO**, e de forma calada: a
    /// [`MeshRenderer::upload_wire_at`] é idempotente por *«a lista já existe»*,
    /// logo trocar a vista deixaria o device a desenhar a lista da vista
    /// anterior até a topologia mudar por acaso. *Um estado guardado ao lado do
    /// buffer que ele descreve é o que torna a porta impossível de chamar
    /// errado.*
    pub(super) wire_grade: bool,
    /// **ESTA MALHA É UM SÓLIDO?** — a resposta de `Mesh::is_closed`, guardada
    /// no instante em que a lista de arestas sobe.
    ///
    /// ⚠️ **Ela mora AQUI e não no `Mesh`, e é ao lado da escrita que ela
    /// descreve:** o `upload_at` derruba a lista de arestas exatamente quando a
    /// topologia pode ter mudado, então guardar a resposta em qualquer outro
    /// lugar abriria a janela em que a peça tem uma topologia e o device carrega
    /// o veredito da anterior.
    pub(super) closed: bool,
    /// ⭐⭐⭐ **O PLANO DE TINTA FINA no device** — ver [`crate::tinta_gpu`].
    ///
    /// ⚠️ **Ele existe SEMPRE**, mesmo sem plano armado, e a razão é o LAYOUT:
    /// o bind group por objecto nasce com o slot, e um binding que aparece e
    /// desaparece obrigaria a reconstruir o layout — que é do PIPELINE, não do
    /// bind. Sem plano ele é de um elemento e o `armado` vale `0`.
    pub(super) tinta: crate::pipeline::tinta_gpu::TintaGpu,
}

/// **UM OBJETO no device** — a geometria e a pose que a põe no mundo.
///
/// ⚠️ O `model` é um buffer por objeto, e não um offset dinâmico num buffer só:
/// um uniform dinâmico exige alinhamento de 256 B por entrada (`min_uniform_
/// buffer_offset_alignment`), então uma cena de blocagem com uma dúzia de peças
/// pagaria 3 KB de padding para poupar uma dúzia de alocações de 64 B. O dia em
/// que a lista tiver centenas de objetos a conta se inverte — e aí a mudança é
/// interna a este arquivo.
pub(super) struct Slot {
    pub(super) gpu: MeshGpu,
    pub(super) model: wgpu::Buffer,
    pub(super) bind: wgpu::BindGroup,
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
pub(super) enum Draw {
    /// Os triângulos da forma.
    Faces,
    /// As arestas, por cima dela.
    Wire,
}
