//! A costura do módulo 3D com o shell — **a cena, o gesto e o passe**.
//!
//! ⚠️ **A navegação orbital E o gesto de escultura moram AQUI, nunca numa
//! `Tool`.** Girar o modelo não é esculpir: o artista gira com o pincel na mão,
//! e uma `Tool` que capturasse o ponteiro para navegar teria de devolvê-lo a
//! cada gesto. É também o que mantém o contrato congelado intacto (ADR-0150) —
//! nenhum método novo em `Tool`.
//!
//! ⚠️ **Tudo isto é inerte sem a cena armada.** `AppGfx.sculpt3d` nasce `None`, então enquanto
//! ninguém a cria cada porta daqui devolve `false` no primeiro `if` e o frame 2D é byte-idêntico.
//! ⚠️ **Três coisas a criam, e a terceira é a que o artista alcança:** o smoke (`PH2D_SCULPT3D_SMOKE`),
//! um projeto que traz uma escultura dentro, e o **pill SCULPT** — ver [`mode`].

use ph2d_light::LightRig;
use ph2d_mesh::{Hit, Mesh, Multires, Pose, Ray};
use ph2d_mesh_render::{Camera3d, MeshRenderer};
use ph2d_sculpt3d::{Brush, Dab, Grip, SculptStroke, Symmetry, Verb};

/// **A DOAÇÃO** — o carimbo, a rasterização e o interruptor de três posições.
/// Filho para alcançar os campos privados da cena; o corte é *o que
/// o escultor FAZ* (aqui) contra *o que a forma DOA* (lá).
pub(crate) mod donation;

/// **O GESTO** — as portas de ponteiro, roda e teclado. Filho para
/// alcançar os campos privados da cena; o corte é *o que a cena É* (aqui) contra
/// *o que a mão FAZ* (lá), o mesmo que separa a [`donation`].
/// ⭐⭐ **QUEM TOMA O GESTO** — o pen-down, separado do que o gesto FAZ. Ver o
/// cabeçalho dele: um é arbitragem, o outro é execução.
mod input_down;

mod input;
/// ⭐ **As três portas do gesto que só precisam da CENA** (W2/L3-A2) — a shell procura-a (é
/// ela que tem o `gfx`) e estas aplicam a lei. Ver a nota no fim do [`input`]: as outras nove
/// continuam em `impl App` porque leem janela, e essa é a lista que o substrato tem de cobrir.
pub(crate) use input::{flush_grab, pointer_move, pointer_up};

/// **O TRANSFORM PONDERADO PELA MÁSCARA** — mover, girar e escalar a parte
/// LIVRE. Filho pelo motivo dos vizinhos; o corte é *o que a mão na
/// tela quer dizer* (lá) contra *a LEI* (no kernel, `ph2d-sculpt3d`).
mod transform;

/// **O FILTRO** — o verbo corrente na malha INTEIRA, com o arrasto a dar a
/// força. Irmão do [`transform`], e o corte é o mesmo: a LEI mora no kernel
/// (`ph2d_sculpt3d::stroke_filter`), e o que mora aqui é *o que a mão na tela
/// quer dizer*.
mod filter;

/// **O TECLADO** — que tecla escolhe o quê. Irmão do [`input`], e o corte é
/// entre *o que a mão faz com o PONTEIRO* e *o que ela ESCOLHE com o teclado*;
/// ele nasceu quando a tabela de teclas levou o arquivo do gesto ao teto de LOC.
mod keys;

/// ⭐⭐⭐ **O GIZMO DE TRANSFORMAÇÃO** — as alças que se agarram (ordem do Enio,
/// 2026-09-08). Filho pelo motivo dos vizinhos; a LEI das alças é a
/// do módulo de modelagem ([`crate::field3d_gizmo`]) e o que mora aqui é **esta
/// câmera e a restrição que cada alça impõe ao gesto**.
mod gizmo;

/// ⭐⭐⭐ **OS QUATRO VIEWPORTS** — frente, lado, cima e a vista do artista, ao
/// mesmo tempo (ordem do Enio, 2026-09-08). Filho pelo motivo dos
/// vizinhos; a lei da DIVISÃO é a do módulo de modelagem
/// ([`crate::field3d_layout`]) e o que mora aqui é **uma câmera por quadrante**.
mod viewports;

/// ⭐⭐ **O GIZMO DA VIEWPORT** — as seis bolas de eixo no canto e as seis vistas
/// nomeadas (ordem do Enio, 2026-09-08). Filho pelo motivo dos
/// vizinhos; a LEI do widget é a do módulo de modelagem
/// ([`crate::field3d_navball`]) e o que mora aqui é **a câmera desta cena**.
mod navball;

/// ⭐⭐⭐ **A PONTE com a árvore do editor** — uma peça ⟺ uma entidade; ver [`entities`].
pub(crate) mod entities;

/// **O CURSOR** — onde a mão está mirando, na tela (W12). Irmão dos três abaixo,
/// e o mais estreito: *onde o gesto vai pousar*, e nada além.
mod cursor;

/// **ENTRAR E SAIR** — o pill SCULPT. Irmão do [`input`] e do [`keys`], e o corte é o mesmo com
/// outro sujeito: aqueles perguntam *o que a mão faz com o barro*, este *quem é dono da tela*.
mod mode;
pub(crate) use mode::sync_pill;

pub(crate) use cursor::{OFF_SURFACE_RGBA, ON_SURFACE_RGBA};

/// **O PAINEL** — o retrato que ele pinta e o gesto que ele devolve (W12).
/// Terceiro irmão do [`input`] e do [`keys`]: o mesmo corte, com um vocabulário
/// próprio (o gesto chega como DADO, um frame depois, pela fila de intents).
mod panel;
pub(crate) use panel::Sculpt3dFrameRequest;

/// **COMO O BARRO É MOSTRADO** — o desenho e as opções de vista. Filho pelo mesmo motivo dos vizinhos, e o corte é *o que a cena É*
/// (aqui) contra *como ela APARECE* (lá): as duas metades crescem por motivos
/// diferentes, e foi um canal de sombreamento novo que cruzou o teto de LOC.
mod view;

/// **O que a cena LEMBRA** — a pilha de níveis e a fila de desfazer. Filho para alcançar os campos privados; o corte é *o que a cena É e o
/// que a mão faz* (aqui) contra *o que ela guarda para poder voltar* (lá).
mod history;

use history::{Entry, StrokeUndo, legacy_requested, retopo_line};

use donation::FormRole;
use donation::FormStamp;

// **AS RÉGUAS DO GESTO** — quanto um pixel de arrasto vale, e onde uma grandeza deixa de
// existir. ⭐ **Saíram para a `ph2d-app-sculpt3d` em 2026-09-11 (W2/L3-A4)**: já eram lei pura
// e não precisavam da shell para nada. O corte por ASSUNTO que as criou continua a valer —
// aqui diz-se *o que a CENA é*, lá *com que régua a mão fala com ela*.
//
// O glob mantém-se de propósito: os filhos leem `super::ORBIT_RAD_PER_PX` como sempre leram,
// e a travessia de crate não move um caminho — tal como o corte de arquivo não movia.
use ph2d_app_sculpt3d::rulers::*;

/// **AS CENAS DO SMOKE** — a fixture de cada uma. Filho pelo motivo
/// dos outros três: o corte é de responsabilidade, e a lista de cenas cresce uma
/// entrada por wave.
mod scenes;

/// O QUE A CENA DIZ ao artista — ver o módulo.
#[path = "announce.rs"]
mod announce_mod;
pub(crate) use announce_mod::announce;

/// **AS MALHAS DE FIXTURE** — como cada modelo de smoke é esculpido. Irmão das
/// cenas, e o corte é entre *que cena o smoke monta* e *como a malha dela é
/// FEITA*; ele nasceu quando o arquivo das cenas cruzou o cap de LOC.
mod fixtures;

/// **O CORPUS DE BENCHMARK do remesher** — ver [`corpus`]. `#[cfg(test)]`.
#[cfg(test)]
mod corpus;

pub(crate) use scenes::shading::env_scene;
pub(crate) use scenes::{
    alpha_image_scene, alpha_scene, bake_scene, cavity_scene, directional_alpha_scene,
    donation_scene, dyntopo_scene, extract_scene, flatten_scene, flatten_scene_counts, fuse_scene,
    holes_scene, mask_channel_numbers, mask_channel_scene, masked_dome_counts, remesh_scene,
    reopen_scene, reversion_scene, scene_objects, smoke_armed, smoke_mesh, soft_masked_counts,
    transform_scene, turn_scene, wants_canvas,
};

/// ⭐ **O VOCABULÁRIO DO ARRASTO** — o que um botão em baixo significa, e o
/// acumulador de ângulo que a torção partilha. Irmão (`#[path]`) pelo tecto de
/// LOC, e o corte é por RESPONSABILIDADE: aqui está *o que a cena É* e ali *como
/// se nomeia um gesto em curso*.
#[path = "drag.rs"]
mod drag_kinds;
use drag_kinds::{Drag, TwistSweep};

/// **A MÁSCARA** — as quatro operações que agem na malha inteira. Filho pelo motivo dos outros: o corte é de responsabilidade.
mod mask;

use mask::MaskOp;

/// **OS VERBOS DA LISTA** — acrescentar, duplicar, apagar. Filho
/// pelo motivo dos outros: o corte é de responsabilidade.
mod objects;
#[path = "preview.rs"]
mod sculpt3d_preview;

pub(crate) use objects::{Extracted, Merge, Primitive};

/// **ONDE as coisas estão** — as portas de espaço. Filho pelo motivo
/// dos outros: o corte é de responsabilidade, e este é o assunto que a lista de
/// objetos inventou.
mod space;

/// **O QUE O DEVICE TEM** — a tabela de slots e o upload. Filho pelo
/// motivo dos outros; ele saiu da [`donation`] quando o isolamento tornou *quem
/// mora em cada slot* uma pergunta com resposta não-óbvia.
mod dyntopo;
mod slots;

/// **O OBJETO MISTO (O2)** — a forma acende um SPRITE da cena, e continua
/// acendendo depois de a malha sair. Filho e irmão da [`donation`]:
/// lá a forma acende a tela do Painter, aqui um objeto da cena — duas perguntas
/// diferentes, e só a segunda sobrevive à escultura.
pub(crate) mod bake;

/// **OS VERBOS QUE PUXAM** — Grab, Snake Hook, Twist, Local Scale. Filho pelo motivo dos outros: o corte é de responsabilidade, e o deles
/// é uma LEI própria (a pegada é presa no pen-down, e o alvo é função do puxão
/// TOTAL, nunca da soma dos passos).
mod pull;

/// **O DOCUMENTO** — a cena como bytes, e os bytes como cena. Filho pelo mesmo
/// motivo: ele lê `objects`/`active`/`next_id` e as filas de desfazer.
mod doc;

/// **A PORTA DE ENTRADA** — um arquivo de malha vira peças. Filho pelo mesmo
/// motivo: ele constrói `SceneObject`s e mexe na lista.
///
/// ⚠️ **`pub(crate)` desde a W22**: o módulo de modelagem implícita entra por aqui também
/// (`field3d_import`), e ele **não** duplica o leitor — o `read_pieces` é a única resposta da casa a
/// *"que malha há neste arquivo?"*, e uma segunda diria "cor preservada" sobre um STL no dia em que
/// alguém trocasse o leitor.
pub(crate) mod import;

/// **A PORTA DE SAÍDA** — a cena vira um arquivo que outro programa abre. Irmão
/// da entrada, e o par dela: sem isto a escultura entra, salva e não sai.
mod export;

/// ⭐ **O aviso do que cada formato NÃO carrega**, partilhado com a modelagem 3D
/// ([`crate::field3d_export`]). Uma segunda cópia lá diria *"cor preservada"*
/// sobre um STL no dia em que alguém trocasse o escritor — e um aviso errado é
/// pior que aviso nenhum, porque o artista confia nele. **Uma tabela, um aviso.**
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
    /// **A peça ISOLADA** — `None` quando a cena inteira está à vista.
    ///
    /// ⚠️ **Um id, e não um `hidden: bool` por peça.** Os dois guardam a mesma
    /// coisa hoje, e só este é incapaz de guardar um estado que ninguém pode
    /// autorar: com bandeiras existe *duas escondidas e uma à vista sem ninguém
    /// ter isolado*, e a pergunta *"o artista está isolado?"* passaria a ser
    /// respondida contando. Aqui a visibilidade é DERIVADA — ver
    /// [`Sculpt3dScene::visible_pieces`].
    ///
    /// ⚠️ **É estado de VISTA, e por isso não entra na história nem no
    /// documento.** Isolar não muda um vértice; é o mesmo lugar em que o onion
    /// da timeline mora (`TimelineState.onion`, não serializado). Um Ctrl+Z que
    /// re-escondesse peças gastaria um passo de undo sem devolver trabalho
    /// nenhum.
    isolated: Option<ObjectId>,
    /// **O ARM da topologia dinâmica** — ver [`dyntopo`], que é onde as três
    /// consequências de ligar estão escritas.
    ///
    /// ⚠️ **Da CENA e não da peça**, como o `symmetry` ao lado: é um modo de
    /// TRABALHO, e um interruptor que mudasse de posição ao trocar de peça é o
    /// que faz o artista desenhar o traço errado para descobrir onde ele está.
    dyntopo: dyntopo::Dyntopo,
    /// A malha da peça ativa **como ela estava no pen-down**, e só quando a
    /// topologia dinâmica está armada.
    ///
    /// ⚠️ **É o preço declarado do modo:** um traço que muda a contagem de
    /// vértices não tem janela por-índice para desfazer, então o desfazer dele é
    /// a malha inteira (a entrada `Remeshed`, que já é uma troca simétrica).
    /// Vazio fora do gesto — ele não é um cache, é a metade de trás de UM passo.
    dyn_before: Option<Box<ph2d_mesh::Mesh>>,
    /// **De onde veio cada vértice que o último refino criou** — reusado entre
    /// dabs, e por isso um campo em vez de um `Vec` local.
    ///
    /// ⚠️ Mora aqui e não no [`dyntopo::Dyntopo`] porque aquele é o estado que o
    /// ARTISTA autora (o interruptor e o detalhe) e é `Copy`; isto é rascunho.
    dyn_births: Vec<ph2d_mesh::Birth>,
    /// A renumeração que o último colapso produziu — o canal irmão do
    /// `dyn_births`, e scratch pela mesma razão: reusá-lo entre dabs é o que
    /// mantém a topologia dinâmica sem alocação no caminho quente.
    dyn_remap: ph2d_mesh::Remap,
    /// **Os buffers do passe de região que o refino roda** — mesma razão do
    /// `dyn_births` acima: eles são do tamanho da MALHA e nascer por dab os
    /// tornaria uma alocação por movimento do mouse, que é o que o
    /// [`ph2d_mesh::RegionScratch`] existe para evitar.
    dyn_region: ph2d_mesh::RegionScratch,
    /// **A TABELA DE SLOTS DO DEVICE** — quem mora em cada índice do
    /// renderizador. Ver [`slots`], que é onde a lei dela está escrita.
    slots: Vec<ObjectId>,
    pub(crate) camera: Camera3d,
    renderer: MeshRenderer,
    drag: Option<Drag>,
    last: (f32, f32),
    /// ⭐⭐⭐ **O ESTADO DA JANELA 3D** — a divisão, as câmeras dos quadrantes, e
    /// os dois gizmos que vivem por cima dela.
    ///
    /// ⚠️ **Dez campos num tipo só, e não dez campos aqui**: eles são uma coisa
    /// (*como a peça é OLHADA*), nascem juntos, morrem juntos e são lidos pelos
    /// mesmos três módulos. Espalhados na cena eles eram indistinguíveis dos
    /// sessenta que descrevem *o que a peça É* — e foram eles que levaram este
    /// ficheiro ao tecto de LOC. Ver [`viewports::Janela`].
    janela: viewports::Janela,

    /// **Com que luz o barro é mostrado** — `None` é o RIG DO ARTISTA, `Some(i)`
    /// é o matcap `i`. Ver [`ph2d_mesh_render::Shade::matcap`].
    ///
    /// ⚠️ Estado de VISTA, não do documento: ele não é salvo, pelo mesmo motivo
    /// que o onion da timeline não é. Escolher com que luz olhar não muda a
    /// escultura, e um projeto que reabrisse em metal diria que alguém mexeu
    /// nela.
    matcap: Option<u8>,
    /// A malha desenhada por cima da forma. Vista, como o [`Self::matcap`].
    wireframe: bool,

    brush: Brush,
    /// **A REFERÊNCIA de cada verbo** (`RefMode`), na ordem do `Verb::ALL`.
    ///
    /// ⚠️ **Ela mora aqui e não só no pincel** porque o `Brush::mode` é o
    /// DERIVADO: o painel reconstrói o `Sculpt3dUi` a cada quadro a partir
    /// destes campos, então uma escolha que vivesse só no pincel seria perdida
    /// na primeira troca de ferramenta — e a troca é justamente o gesto que a
    /// re-resolve.
    /// ⚠️ **O TAMANHO É DERIVADO, e era um `16` literal** — a segunda cópia de
    /// uma contagem que o [`ph2d_sculpt3d::Verb::ALL`] já responde. Ela sobreviveu
    /// enquanto o catálogo não crescia, e o dia em que a W6 acrescentou a faixa
    /// ela virou erro de tipo em dois arquivos do shell.
    verb_slots: [ph2d_panel_sculpt3d::state::VerbSlot; ph2d_sculpt3d::Verb::ALL.len()],
    /// **COM QUE PROFUNDIDADE O PAINEL SE MOSTRA** — ver
    /// [`ph2d_panel_sculpt3d::UiLevel`].
    ///
    /// ⚠️ **A cena o guarda e NUNCA o lê**, e isso é o desenho e não descuido:
    /// o painel não guarda estado entre quadros (ele recebe um retrato novo a
    /// cada um), então a escolha tem de morar em algum lugar que sobreviva — e
    /// esse lugar é o mesmo que já guarda os outros valores autorados. Renderizar
    /// nada a partir dele é o que o mantém honesto: com que profundidade olhar
    /// não muda a escultura, e por isso ele também não é salvo.
    ui_level: ph2d_panel_sculpt3d::UiLevel,
    /// **O padrão do pincel é mostrado no barro?** — ver
    /// [`sculpt3d_preview::PreviewState`]. Nasce ligado.
    alpha_preview: bool,
    /// **A IMAGEM que o artista armou, e de que sprite ela veio** — o slot que
    /// sobrevive à troca de padrão.
    ///
    /// ⚠️ **Sem ele o chip do slot seria um controle que só sabe DESMARCAR-se:**
    /// escolher `Grain` tira o `Arc<AlphaImage>` do pincel, e o painel — que só
    /// vê o retrato — não teria o que re-armar. Lembrar é o que faz da fileira
    /// um SELETOR: os nove procedurais sobrevivem porque são nomes, e a imagem
    /// precisa de alguém que a segure.
    ///
    /// ⚠️ **UM campo com o par, e não dois `Option`**, porque um nome sem imagem
    /// (ou o contrário) é um estado que ninguém sabe pintar — e dois campos que
    /// têm de concordar é a forma como ele nasce.
    alpha_image: Option<(
        std::sync::Arc<ph2d_sculpt3d::AlphaImage>,
        std::sync::Arc<str>,
    )>,
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
    /// **ONDE O DEDO ESTÁ, ainda não carimbado** — o par do [`Self::grab`], e a
    /// razão de ele existir é que um `Grip::Hold` faz **um dab por EVENTO de
    /// ponteiro**: a ~1000 Hz de mouse e 60 fps são ~16 dabs por quadro, e o
    /// dab do `l-mode` custa 1,05 ms num pincel de 10 % (medido em
    /// `ph2d-sculpt3d/tests/it/measure_field_cost.rs`).
    ///
    /// ⚠️ **Descartar os intermediários é BYTE-IDÊNTICO, e isso é MEDIDO, não
    /// suposto** (`measure_what_coalescing_a_hold_gesture_changes`): 16 eventos
    /// contra 1, com puxões de 50 %, 100 % e 200 % do raio, dão **desvio máximo
    /// 0,000000** e a MESMA contagem de movidos — 17,9 ms viram 1,2.
    ///
    /// O mecanismo já estava escrito no `touched`: o `Grip::Hold` é **frozen**,
    /// então o laço percorre o conjunto congelado no pen-down e o alvo é medido
    /// contra o `pre`. A pegada é consultada nas posições VIVAS e por isso pode
    /// CRESCER — mas o que ela acrescenta pesa zero contra a posição congelada.
    /// *A ressalva que eu tinha escrito (`pode mudar quais vértices entram`)
    /// caiu na medição.*
    ///
    /// ⚠️ **É o mesmo argumento que o `Grip::Turn` já usa** para não andar o
    /// `walk` (`input.rs`): *N dabs com o mesmo total acumulado no
    /// mesmo lugar é trabalho idêntico repetido*. A diferença é que o `Turn`
    /// paga isso uma vez por evento e o `Hold` pagava dezesseis.
    pending_grab: Option<(f32, f32)>,
    /// **O ângulo já varrido** pelo gesto do Twist — ver [`TwistSweep`]. `None`
    /// fora de um gesto; o pen-down o zera.
    twist: Option<TwistSweep>,
    /// **O que o painel ARMOU** — `None` é o estado normal, em que o botão
    /// esquerdo esculpe.
    ///
    /// ⚠️ Estado de FERRAMENTA, não de gesto: ele sobrevive ao pen-up, como o
    /// verbo do pincel. Quem o desarma é o artista, clicando o mesmo botão.
    transform_arm: Option<ph2d_sculpt3d::TransformKind>,
    /// A sessão VIVA — a foto do pen-down. Vazia fora do gesto.
    transform: Option<ph2d_sculpt3d::MaskTransform>,
    /// Onde o dedo pousou: a âncora contra a qual todo gesto é medido.
    transform_from: (f32, f32),
    /// **O FILTRO está armado?** — o botão esquerdo roda o verbo corrente na
    /// malha INTEIRA em vez de esculpir sob o cursor.
    ///
    /// ⚠️ Estado de FERRAMENTA, como o [`Self::transform_arm`] acima, e
    /// **mutuamente exclusivo** com ele: os dois armam o MESMO botão, e um gesto
    /// que significasse as duas coisas não teria como escolher. Quem garante a
    /// exclusão são as duas portas de armar, uma vez cada.
    filter_arm: bool,
    /// **Onde o dedo pousou**, em x — a âncora da força.
    ///
    /// ⚠️ Só o eixo X, e não o par: a força é o arrasto HORIZONTAL, a lei da
    /// referência (o filtro de malha dela). Guardar o `y` seria carregar
    /// um número que ninguém lê.
    filter_from_x: f32,
    /// **QUAL lei o filtro roda** — escolhida no painel, não derivada do verbo.
    ///
    /// ⚠️ **É a mudança de premissa da W9b, e ela é visível:** enquanto a lei
    /// vinha de [`Verb::filter_kind`], três das sete eram INALCANÇÁVEIS por
    /// gesto nenhum (não existe pincel de Scale, de Sphere nem de Random). O
    /// verbo em mãos passa a apenas SEMEAR esta escolha ao armar — quem manda
    /// é o artista, e um verbo sem lei própria deixa a última escolha de pé.
    filter_law: ph2d_sculpt3d::FilterLaw,
    /// ⭐⭐⭐ **O QUE O ARTISTA AFINOU NO FILTRO DE TECIDO** — o referencial, os
    /// quatro números e os três eixos do *Force Axis*.
    ///
    /// ⚠️ **Um tipo e não cinco campos soltos**: eles são uma coisa (*como o
    /// filtro de tecido está afinado*), são GLOBAIS — um filtro não pertence a
    /// ferramenta nenhuma, e três dos cinco tipos não têm verbo — e são lidos
    /// pelo mesmo sítio. Soltos entre os sessenta campos que descrevem *o que a
    /// peça É* eram indistinguíveis do resto, e foram eles que levaram este
    /// ficheiro ao tecto de LOC. Ver [`filter::Tecido`].
    tecido: filter::Tecido,
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
    /// **A CAVIDADE** — quanto a curvatura escurece a fresta e clareia a crista
    /// (`ph2d_mesh_render::shade`, `docs/3D/05.1` §4).
    ///
    /// ⚠️ Nasce em [`ph2d_mesh_render::DEFAULT_CAVITY`], que é **zero**, e o
    /// motivo é o mesmo do `FormRole::Clay` logo abaixo: o barro liso é o que a
    /// W3 entregou e o Enio aprovou, e um canal de sombreamento que se arma
    /// sozinho muda a arte de todo mundo que já esculpiu. O `Shift+C` o liga.
    cavity: f32,
    /// **Quanto do AMBIENTE COM DIREÇÃO entra** — o piso da difusa deixando de
    /// ser o mesmo número em toda direção.
    ///
    /// ⚠️ **Nasce em ZERO, como a `cavity` logo acima e pela mesma razão**: é um
    /// canal que muda a leitura de toda escultura já feita. E dois gates de GPU
    /// cobraram isso — o `the_two_lights_agree_where_the_form_turns_away` afirma
    /// que a luz do barro e a da tinta concordam onde a forma vira, e um piso
    /// direcional só no barro as separa. Levantar o slider diverge, e isso é
    /// aceito (é a classe do matcap); entregá-lo divergido não.
    env: f32,
    /// Quanto do AO ASSADO entra no sombreamento — o irmão da `cavity`, e o
    /// oposto dela na origem: a cavidade é derivada e existe sempre, o AO só
    /// existe depois de o artista pedir um bake.
    ao: f32,
    /// Quanto do AO DE TELA entra — e ele **nasce LIGADO**, ao contrário dos dois
    /// vizinhos, porque é o único dos três que é MEDIDO a cada frame.
    ///
    /// ⚠️ Ligar por default não muda a tela sozinho: o barro amostra um canal que
    /// vale zero enquanto ninguém rodar o passe, então o `1.0` diz *"mostre o que
    /// foi medido"*. E é ele que decide se o passe roda — zero é a resposta
    /// completa para *"não quero pagar por isto"*, sem um segundo interruptor.
    ssao: f32,
    /// **O ESPALHAMENTO SUB-SUPERFICIAL** — quanto, e até onde a luz viaja dentro
    /// do material.
    ///
    /// ⚠️ Ele nasce em **zero**, como o AO assado e ao contrário do AO de tela, e
    /// a razão é de outra família: os dois AOs são MEDIÇÕES da forma (mostrá-las
    /// é honesto), e este é um **MATERIAL**. Barro não é pele; ligar o
    /// espalhamento por padrão mudaria a aparência de toda escultura já feita
    /// para uma que ninguém pediu.
    ///
    /// ⚠️ E o `scatter` **não** mora aqui: ele é semeado pelo tamanho da peça a
    /// cada frame (`Sculpt3dScene::shade`), pela mesma razão do raio do AO de
    /// tela — um número guardado seria uma segunda verdade sobre o tamanho da
    /// peça, a que fica velha exatamente quando o artista a faz crescer.
    sss: f32,
    /// **Até onde a luz viaja, como FRAÇÃO do maior lado da peça.**
    ///
    /// ⚠️ **Fração e não comprimento, e é ela que resolve a tensão entre duas
    /// coisas certas.** Um comprimento absoluto guardado seria uma segunda
    /// verdade sobre o tamanho da escultura — a que fica velha exatamente quando
    /// o artista a faz crescer. Mas o alcance é **o número que decide o LOOK**, e
    /// derivá-lo inteiramente deixava o artista sem como julgá-lo. Guardar a
    /// FRAÇÃO mantém as duas: o alcance segue sendo função da peça, e o slider é
    /// o veredito de aparência, que nenhuma medição responde.
    sss_scatter: f32,
    /// **O que o botão de extract vai fazer** — ver [`ph2d_mesh::Extract`].
    ///
    /// ⚠️ Autorado, e por isso ele mora aqui e não num argumento do gesto: o
    /// artista ajusta a espessura, olha, extrai de novo. O tipo é o do KERNEL —
    /// dois `f32` soltos seriam um segundo lugar para o default morar.
    extract: ph2d_mesh::Extract,
    /// **Em que resolução o botão RECONSTRUIR voxeliza** — o slider da seção
    /// Topology.
    ///
    /// ⚠️ Guardado como CONTAGEM (`u32`), e não como o `f32` da pista: a pista é
    /// contínua porque pistas são contínuas, e a grandeza é um número de células.
    /// O arredondamento e o clamp moram na fronteira (`apply_ui`), não aqui.
    remesh_res: u32,
    /// O lado do quad que a retopologia persegue, em unidades de objeto — ver
    /// [`Sculpt3dScene::quad_remesh`].
    quad_detail: f32,
    /// Quanto a densidade da retopologia segue a curvatura.
    quad_adapt: f32,
    /// ⭐ **QUAL MOTOR de retopologia o botão chama** — ver
    /// [`ph2d_panel_sculpt3d::state::RetopoMode`].
    retopo_mode: ph2d_panel_sculpt3d::state::RetopoMode,
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
    /// **A posição ANTERIOR do barro**, a testemunha de que o modo VIROU — ver
    /// [`Sculpt3dScene::take_clay_edge`], a porta única que a lê.
    ///
    /// ⚠️ **Nasce `false` mesmo com o papel nascendo em `Clay`**, e é isso que
    /// apaga o caso especial: o primeiro frame de uma cena nova produz a borda
    /// *entrou*, então "abrir ao nascer" e "abrir ao voltar" deixam de ser duas
    /// regras — são a MESMA.
    clay_was_on: bool,
    /// **O rig de quando alguém perguntou pela última vez** — a testemunha de que o artista MEXEU
    /// na lâmpada; ver [`Sculpt3dScene::take_rig_edge`], a porta única que a lê.
    ///
    /// ⚠️ **Nasce com o rig ATUAL**, ao contrário do [`Self::clay_was_on`] logo acima, e a
    /// assimetria é o conserto: uma cena recém-criada não mexeu em lâmpada nenhuma, e tratar o
    /// nascimento dela como um gesto reescreve a luz de todo objeto já assado no documento.
    rig_was: crate::baked_form::RigStamp,
    /// O carimbo da última doação entregue — `None` enquanto nada foi doado.
    donated: Option<FormStamp>,
}

mod birth;

/// ⭐ **O ESTADO DA FAMÍLIA que vive na `App`** — os quatro campos que eram soltos lá
/// (W2/L3-A2). Ver o cabeçalho dele: a fronteira com o irmão não-gateado da crate é imposta
/// pela `cfg`, não escolhida.
mod shell_state;
pub(crate) use shell_state::Sculpt3dShellState;

/// ⭐⭐ **O teclado da CÂMERA da escultura** — a divisão em quatro e as seis vistas.
/// Irmão do [`keys`] pelo tecto de LOC; ver o cabeçalho dele.
///
/// ⚠️ **Era `crate::sculpt3d_keys_view`, declarado à parte no `main.rs`** — o único
/// ficheiro da família fora desta árvore. Passou a filho em 2026-09-11 (W2/L3-A3), e é
/// por isso que os testes dele mudaram de `sculpt3d_keys_view::…` para
/// `sculpt3d::keys_view::…`: o nome da função não mudou, a casa dela sim.
mod keys_view;
