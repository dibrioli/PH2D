//! A costura do módulo 3D com o shell — **a cena, o gesto e o passe**.
//!
//! ⚠️ **A navegação orbital E o gesto de escultura moram AQUI, nunca numa
//! `Tool`.** Girar o modelo não é esculpir: o artista gira com o pincel na mão,
//! e uma `Tool` que capturasse o ponteiro para navegar teria de devolvê-lo a
//! cada gesto. É também o que mantém o contrato congelado intacto (ADR-0150) —
//! nenhum método novo em `Tool`.
//!
//! ⚠️ **Tudo isto é inerte sem a cena armada.** `AppGfx.sculpt3d` nasce `None` e
//! só o smoke a cria, então num run normal cada porta daqui devolve `false` no
//! primeiro `if` e o frame 2D é byte-idêntico.

use ph2d_light::LightRig;
use ph2d_mesh::{Hit, Mesh, Multires, Pose, Ray};
use ph2d_mesh_render::{Camera3d, MeshRenderer};
use ph2d_sculpt3d::{Brush, Dab, Grip, SculptStroke, Symmetry, Verb};

/// **A DOAÇÃO** — o carimbo, a rasterização e o interruptor de três posições.
/// Filho (`#[path]`) para alcançar os campos privados da cena; o corte é *o que
/// o escultor FAZ* (aqui) contra *o que a forma DOA* (lá).
#[path = "sculpt3d_donation.rs"]
pub(crate) mod donation;

/// **O GESTO** — as portas de ponteiro, roda e teclado. Filho (`#[path]`) para
/// alcançar os campos privados da cena; o corte é *o que a cena É* (aqui) contra
/// *o que a mão FAZ* (lá), o mesmo que separa a [`donation`].
#[path = "sculpt3d_input.rs"]
mod input;

/// **O TECLADO** — que tecla escolhe o quê. Irmão do [`input`], e o corte é
/// entre *o que a mão faz com o PONTEIRO* e *o que ela ESCOLHE com o teclado*;
/// ele nasceu quando a tabela de teclas levou o arquivo do gesto ao teto de LOC.
#[path = "sculpt3d_keys.rs"]
mod keys;

/// **O que a cena LEMBRA** — a pilha de níveis e a fila de desfazer. Filho
/// (`#[path]`) para alcançar os campos privados; o corte é *o que a cena É e o
/// que a mão faz* (aqui) contra *o que ela guarda para poder voltar* (lá).
#[path = "sculpt3d_history.rs"]
mod history;

use history::{Entry, StrokeUndo};

use donation::FormRole;
use donation::FormStamp;

/// Quantos radianos um pixel de arrasto vale.
///
/// Decisão de **smoke**, como a tolerância do RDP do Flip: 0,01 dá meia volta a
/// cada ~314 px, que é uma varredura confortável de trackpad. Não é um teto de
/// recurso, então não tem tabela de medição ao lado — tem o olho do Enio.
const ORBIT_RAD_PER_PX: f32 = 0.01;

/// O raio do pincel, em **pixels de tela**.
///
/// ⚠️ **Pixels, não fração do modelo** — o raio de MUNDO é derivado por dab
/// (`Camera3d::world_radius_for_screen_px`), então o pincel mantém o tamanho
/// aparente quando a câmera aproxima. É o `computeWorldRadius2` do SculptGL, e
/// é o que Blender e ZBrush entregam: aproximar É como se alcança detalhe fino,
/// e um raio ancorado no modelo tornava isso impossível (o pincel crescia junto
/// com a imagem).
///
/// **50 px é MEDIDO, não escolhido:** é o que reproduz o tamanho aparente do
/// default anterior (0,12 do span) na cena do smoke a 720p — ver
/// `ph2d-mesh-render/tests/measure_screen_radius.rs`.
const DEFAULT_RADIUS_PX: f32 = 50.0;

/// Passo das teclas de LUZ (`Q`/`E` giram, `R`/`F` sobem e descem), em graus
/// inteiros — que é a unidade em que o rig é autorado. Quinze graus porque o
/// gesto é *"ver a forma reacender"*, não afinar: um passo de 1° pediria vinte
/// toques para a mudança ficar óbvia.
const LIGHT_STEP_DEG: u16 = 15;

/// Passo do `[` / `]`. Multiplicativo pelo motivo do `dolly`: o gesto tem o
/// mesmo efeito *aparente* com pincel grande e pequeno.
const RADIUS_STEP: f32 = 1.15;

/// O piso do raio, em pixels. **Quem aperta é a TELA, não a malha** — e isso é
/// medição, não herança: a régua antiga dizia *"menor que uma aresta não pega
/// vértice"*, e na cena do smoke um disco de **0,5 px já pega um vértice**
/// (`measure_screen_radius.rs`), porque a malha é densa. O que de fato quebra
/// abaixo de um pixel é o artista **ver onde está mirando**.
/// Quantos passos um clique de blur/sharpen dá.
///
/// ⚠️ **Número de SMOKE, não teto de recurso.** Um passo é pequeno de propósito
/// (`BLUR_MIX = 0,5`, para o gesto não apagar a própria borda de uma vez), então
/// o clique precisa de vários para o artista ver a diferença — e clicar de novo
/// borra mais, que é o que o gesto significa.
const MASK_OP_PASSES: u32 = 6;

const RADIUS_MIN_PX: f32 = 1.0;

/// O teto do raio, em fração da ALTURA do viewport.
///
/// ⚠️ **Fração da tela, e não um número fixo de pixels, porque um teto fixo muda
/// de SIGNIFICADO com a resolução:** medido, 160 px cobre **91% da altura do
/// modelo a 1280×720 e 45% a 2560×1440**. `0,125` é a mesma promessa do teto
/// antigo (*acima de meio modelo o "pincel" é um deformador global, que é outra
/// ferramenta*) escrita no recurso que de fato aperta — com o enquadramento
/// padrão o modelo ocupa 49% da altura, então 1/8 de tela é meio modelo.
const RADIUS_MAX_FRAC_OF_HEIGHT: f32 = 0.125;

/// A zona morta do **Twist**, em pixels de tela.
///
/// ⚠️ **Ela não é conforto, é a fronteira onde a grandeza deixa de existir:**
/// perto da âncora a direção *âncora → cursor* é RUÍDO, e um tremor de um pixel
/// a um pixel de distância vale meio radiano. Trinta é o número do SculptGL
/// (`Twist.js:92`), e como todo número de gesto deste arquivo ele é decisão de
/// **smoke** — não é teto de recurso nenhum.
const TWIST_DEADZONE_PX: f32 = 30.0;

/// Quanto de escala vale um pixel de arrasto horizontal no **Local Scale**
/// (`+1` dobra o raio da pegada). Cem pixels dobram; decisão de smoke, como o
/// [`ORBIT_RAD_PER_PX`].
const SCALE_PER_PX: f32 = 0.01;

/// **AS CENAS DO SMOKE** — a fixture de cada uma. Filho (`#[path]`) pelo motivo
/// dos outros três: o corte é de responsabilidade, e a lista de cenas cresce uma
/// entrada por wave.
#[path = "sculpt3d_scenes.rs"]
mod scenes;

/// **AS MALHAS DE FIXTURE** — como cada modelo de smoke é esculpido. Irmão das
/// cenas, e o corte é entre *que cena o smoke monta* e *como a malha dela é
/// FEITA*; ele nasceu quando o arquivo das cenas cruzou o cap de LOC.
#[path = "sculpt3d_fixtures.rs"]
mod fixtures;

pub(crate) use scenes::{
    announce, bake_scene, donation_scene, holes_scene, remesh_scene, reversion_scene,
    scene_objects, smoke_armed, smoke_mesh, turn_scene, wants_canvas,
};

/// O que o arrasto está fazendo.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Drag {
    Orbit,
    Pan,
    Sculpt,
}

/// **O ângulo VARRIDO desde o pen-down**, acumulado evento a evento.
///
/// ⚠️ **Acumulado, e não um `atan2` da direção inicial à atual** — este é o
/// único jeito de uma varredura passar de meia volta. Um ângulo com sinal
/// satura em `±π`, então a 181° ele voltaria a `−179°` e a torção **inverteria**
/// no meio do gesto. Somando os deltas (que são pequenos) o total cresce sem
/// teto, e a soma é EXATA: ângulos se somam, então subdividir o caminho não
/// muda o resultado — que é o que o [`Grip::Turn`] exige do gesto que o
/// alimenta.
struct TwistSweep {
    /// A última direção unitária *âncora → cursor*, em componentes de CÂMERA.
    /// `None` enquanto o cursor está dentro da zona morta: sem direção não há
    /// delta a somar, e a próxima saída re-semeia sem inventar um salto.
    last: Option<[f32; 2]>,
    total: f32,
}

/// **A MÁSCARA** — as quatro operações que agem na malha inteira. Filho
/// (`#[path]`) pelo motivo dos outros: o corte é de responsabilidade.
#[path = "sculpt3d_mask.rs"]
mod mask;

use mask::MaskOp;

/// **OS VERBOS DA LISTA** — acrescentar, duplicar, apagar. Filho (`#[path]`)
/// pelo motivo dos outros: o corte é de responsabilidade.
#[path = "sculpt3d_objects.rs"]
mod objects;

pub(crate) use objects::Primitive;

/// **ONDE as coisas estão** — as portas de espaço. Filho (`#[path]`) pelo motivo
/// dos outros: o corte é de responsabilidade, e este é o assunto que a lista de
/// objetos inventou.
#[path = "sculpt3d_space.rs"]
mod space;

/// **O OBJETO MISTO (O2)** — a forma acende um SPRITE da cena, e continua
/// acendendo depois de a malha sair. Filho (`#[path]`) e irmão da [`donation`]:
/// lá a forma acende a tela do Painter, aqui um objeto da cena — duas perguntas
/// diferentes, e só a segunda sobrevive à escultura.
#[path = "sculpt3d_bake.rs"]
pub(crate) mod bake;

/// **OS VERBOS QUE PUXAM** — Grab, Snake Hook, Twist, Local Scale. Filho
/// (`#[path]`) pelo motivo dos outros: o corte é de responsabilidade, e o deles
/// é uma LEI própria (a pegada é presa no pen-down, e o alvo é função do puxão
/// TOTAL, nunca da soma dos passos).
#[path = "sculpt3d_pull.rs"]
mod pull;

/// **O DOCUMENTO** — a cena como bytes, e os bytes como cena. Filho pelo mesmo
/// motivo: ele lê `objects`/`active`/`next_id` e as filas de desfazer.
#[path = "sculpt3d_doc.rs"]
mod doc;

/// **A PORTA DE ENTRADA** — um arquivo de malha vira peças. Filho pelo mesmo
/// motivo: ele constrói `SceneObject`s e mexe na lista.
#[path = "sculpt3d_import.rs"]
mod import;

/// **A PORTA DE SAÍDA** — a cena vira um arquivo que outro programa abre. Irmão
/// da entrada, e o par dela: sem isto a escultura entra, salva e não sai.
#[path = "sculpt3d_export.rs"]
mod export;

pub(crate) use import::is_mesh_file;

// ⚠️ Só o que ATRAVESSA a fronteira do módulo: o `SCULPT_DOC_VERSION` e o
// `SculptDocError` são assunto de dentro (o load só formata o `Display` do
// erro), e re-exportá-los seria superfície que ninguém pede.
pub(crate) use doc::{LoadedPiece, decode as decode_doc};

// ⚠️ O ESCRITOR atravessa a fronteira só para os gates: as fixtures de
// `project_tests` precisam de um documento de escultura VÁLIDO, e montá-lo à mão
// lá seria um segundo escritor — que concordaria com este exatamente onde ele
// erra. O `cfg(test)` é o que diz que a superfície é isso e nada mais.
#[cfg(test)]
pub(crate) use doc::encode as encode_doc;

/// **UM OBJETO da cena** — a pilha de níveis dele e onde ele está.
///
/// ⚠️ `uploaded` e `dirty` são POR OBJETO, e não da cena: subir a malha de um
/// não limpa a do outro, e um par compartilhado deixaria o segundo objeto
/// desenhado com a geometria de antes do dab — sem erro, sem warning, e com
/// todos os gates de CPU verdes.
/// **O QUE UMA PEÇA É** — ela mora com os verbos da LISTA (`objects`), que é o
/// assunto de que ela é o elemento; re-exportada aqui porque meio módulo a
/// nomeia por `super::SceneObject`.
pub(crate) use objects::{ObjectId, SceneObject};

/// A cena 3D viva: os objetos, a câmera, o pincel e o pipeline que a desenha.
pub(crate) struct Sculpt3dScene {
    /// **A CENA é uma LISTA.** Nunca vazia — a invariante que torna
    /// [`Sculpt3dScene::obj`] total.
    pub(crate) objects: Vec<SceneObject>,
    /// Quem a mão está trabalhando. Sempre `< objects.len()`.
    ///
    /// ⚠️ **Ele NÃO é um modo escondido:** quem o move é o `aim` do PEN-DOWN
    /// (mirar uma peça a torna ativa), então "a ativa" é sempre *a última que
    /// você tocou* — e é por isso que a cena não precisa de um realce de seleção
    /// para ser honesta.
    ///
    /// ⚠️ **E ele não se move DENTRO de um gesto**, o que não é preferência: o
    /// `SculptStroke` dimensiona os planos por-vértice na malha em que o traço
    /// começou, então trocar de peça no meio escreve índices de uma malha noutra
    /// — mudo enquanto a nova for menor, **pânico** assim que for maior. Ver
    /// `Sculpt3dScene::aim`.
    pub(crate) active: usize,
    /// O próximo [`ObjectId`] a cunhar. Ele **nunca reusa**: um id reciclado
    /// faria uma entrada de undo velha nomear uma peça nova, que é exatamente o
    /// que o índice já fazia.
    next_id: u32,
    pub(crate) camera: Camera3d,
    renderer: MeshRenderer,
    drag: Option<Drag>,
    last: (f32, f32),
    viewport: (u32, u32),

    brush: Brush,
    /// O raio autorado, em **pixels de tela** — ver [`DEFAULT_RADIUS_PX`]. O raio
    /// de MUNDO é derivado por dab, contra a câmera e o ponto de acerto.
    radius_px: f32,
    /// Onde o traço carimbou pela última vez, em pixels — **a âncora do
    /// espaçamento**, e ela é separada do `last` de propósito: o `last` é o
    /// delta de TODO arrasto (a órbita precisa dele por evento) e esta só anda
    /// quando um dab de fato saiu. Colapsá-las apagaria o carry.
    stroke_anchor: [f32; 2],
    /// Onde a mão **pegou** — o ponto de mundo do pen-down e o pixel dele. É a
    /// âncora dos DOIS grips que puxam, e é dela que os dois derivam o mundo:
    /// o [`Grip::Hold`] mede o puxão total até aqui, o [`Grip::Hook`] mede o
    /// incremento entre dois passos do caminho.
    grab: Option<([f32; 3], (f32, f32))>,
    /// **O ângulo já varrido** pelo gesto do Twist — ver [`TwistSweep`]. `None`
    /// fora de um gesto; o pen-down o zera.
    twist: Option<TwistSweep>,
    symmetry: Symmetry,
    /// **O rig de luz do artista** — as mesmas quatro lâmpadas que acendem a tinta
    /// do Painter (`ph2d-light`).
    ///
    /// ⚠️ A cena guarda uma INSTÂNCIA porque hoje ela é um viewport solto, e o
    /// viewport é o documento dela. Quando a escultura virar uma camada de um
    /// documento do Painter (W3.M4) o rig passa a ser o DELE — a estrutura já tem
    /// um dono só, e o que falta unificar é o dado. Um segundo rig permanente
    /// aqui seria exatamente o que `docs/3D/05.2` proíbe.
    rig: LightRig,
    stroke: SculptStroke,
    undo: Vec<Entry>,
    /// **O futuro guardado** — o que um Ctrl+Z tirou e um Ctrl+Shift+Z devolve.
    ///
    /// ⚠️ Ela é populada por [`Sculpt3dScene::undo_stroke`] e esvaziada por
    /// [`Sculpt3dScene::record`], nunca por quem edita: uma edição nova torna
    /// este futuro inalcançável, e a lei mora na porta que grava.
    redo: Vec<Entry>,
    /// Quantas vezes a MALHA mudou. Entra no carimbo da doação — ver
    /// `Sculpt3dScene::mesh_changed`, a porta única que o move.
    edits: u64,
    /// **O interruptor da doação** — ver [`FormRole`]. Nasce em `Clay` porque a
    /// primeira coisa que se faz com uma escultura é esculpi-la; a doação é o
    /// passo seguinte, e o `D` o dá.
    role: FormRole,
    /// O carimbo da última doação entregue — `None` enquanto nada foi doado.
    donated: Option<FormStamp>,
}

impl Sculpt3dScene {
    /// **O rig que esta cena tem na mão.** A luz é dela enquanto ela existe.
    ///
    /// ⚠️ Ele é lido pelo bake para AUTORAR o rig do objeto assado — ver
    /// [`bake::follow_live_rig`]. O objeto guarda uma CÓPIA porque ele sobrevive
    /// à cena; enquanto os dois existem, quem manda é esta.
    pub(crate) fn rig(&self) -> &LightRig {
        &self.rig
    }
}

impl Sculpt3dScene {
    pub(crate) fn new(device: &wgpu::Device, mesh: Mesh, aspect: f32) -> Self {
        let mut camera = Camera3d {
            yaw: 0.6,
            pitch: 0.35,
            ..Camera3d::default()
        };
        camera.frame(mesh.bounds(), aspect);
        Self {
            objects: vec![SceneObject::new(ObjectId(0), mesh, Pose::IDENTITY)],
            active: 0,
            next_id: 1,
            camera,
            renderer: MeshRenderer::new(device, ph2d_render::GameRt::FORMAT),
            drag: None,
            last: (0.0, 0.0),
            viewport: (1, 1),
            brush: Brush::default(),
            radius_px: DEFAULT_RADIUS_PX,
            stroke_anchor: [0.0, 0.0],
            grab: None,
            twist: None,
            // ⚠️ **DESLIGADA por default, e é decisão do smoke.** O ZBrush
            // nasce com espelho ligado — e MOSTRA isso. Aqui o artista clicava
            // de um lado e via uma segunda protuberância do outro, sem nada na
            // tela explicando por quê: *"o local onde está esculpindo não
            // coincide com a posição do mouse"*. Um default que só se descobre
            // por acidente é pior que um default menos ambicioso; o `X` liga.
            symmetry: Symmetry::default(),
            rig: LightRig::default(),
            stroke: SculptStroke::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            edits: 0,
            role: FormRole::Clay,
            donated: None,
        }
    }

    /// Desenha a malha sobre o que já está no alvo. O upload acontece na
    /// primeira passagem — é aqui que o device é conhecido.
    pub(crate) fn render(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        size: (u32, u32),
    ) {
        self.viewport = size;
        // ⚠️ O viewport é atualizado ANTES da recusa: ele é o que converte um
        // clique em raio, e um viewport parado faria o pincel cair no lugar
        // errado no instante em que o artista voltasse ao barro.
        if !self.shows_clay() {
            return;
        }
        self.sync_mesh(&gpu.device, &gpu.queue);
        // O rig é RESOLVIDO por frame, não guardado resolvido: a resolução é
        // barata (quatro lâmpadas) e uma cópia resolvida seria uma segunda
        // verdade sobre onde a luz está — a que fica velha no frame seguinte ao
        // artista mexer no card.
        let resolved = ph2d_light::resolve(&self.rig);
        self.renderer.render(
            &gpu.device,
            &gpu.queue,
            encoder,
            color,
            &self.camera,
            resolved.as_ref(),
            size,
        );
    }

    /// O raio do cursor, pela câmera desta cena.
    fn ray_at(&self, x: f32, y: f32) -> Ray {
        self.camera.ray_through(x, y, self.viewport)
    }

    /// Aplica um dab onde o cursor aponta. Devolve `false` se o raio errou a
    /// malha — e errar é normal: a mão sai do modelo o tempo todo.
    fn sculpt_at(&mut self, x: f32, y: f32) -> bool {
        // Na peça ATIVA — quem a escolheu foi o `aim` do pen-down. Ver o doc
        // dele: um traço pertence a uma peça, e trocar no meio é um pânico.
        let Some(hit) = self.pick_active(x, y) else {
            return false;
        };
        let ray = self.ray_at(x, y);
        if std::env::var("PH2D_SCULPT3D_DIAG").ok().as_deref() == Some("1") {
            // ⚠️ **O instrumento que responde *"o pincel cai onde o cursor
            // aponta?"* com um NÚMERO.** Ele reprojeta o acerto pela porta
            // `project` — o inverso exato do `ray_through` — e imprime o erro em
            // pixels. Um desvio grande acusa a fiação (viewport, escala, um
            // flip); zero acusa a percepção, e aí a causa é outra.
            let back = self
                .camera
                .project(self.pose().point_to_world(hit.point), self.viewport);
            let err = back.map(|(bx, by)| ((bx - x).hypot(by - y), bx, by));
            eprintln!(
                "[sculpt3d] clique ({x:.1}, {y:.1}) viewport {:?} -> acerto {:?} \
                 -> volta {err:?}",
                self.viewport, hit.point
            );
        }
        let brush = self.armed_brush(hit.point);
        let eye = self.dir_to_local(ray.dir());
        self.stroke.dab(
            self.objects[self.active].stack.mesh_mut(),
            &brush,
            // ⚠️ **O olho é o `dir` do raio que ACABOU de produzir este acerto**,
            // e não uma direção derivada da câmera de novo: duas respostas para
            // *"de onde se está olhando"* divergem no frame em que a câmera se
            // move entre o pick e o dab.
            &Dab::at(hit.point, brush.radius, eye),
            self.symmetry,
        );
        Self::mesh_changed(
            &mut self.objects[self.active].dirty,
            &mut self.edits,
            // ⚠️ **`last_gpu_dirty`, não `last_refreshed`.** Um traço de máscara
            // não move geometria, então ele não refresca normal nenhuma — e
            // perguntar *"o que refresquei?"* devolveria VAZIO, deixando a
            // máscara invisível na GPU com todos os gates de CPU verdes.
            self.stroke.last_gpu_dirty(),
        );
        true
    }
}
