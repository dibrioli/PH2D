//! **A família `sculpt3d` a sair da shell** — a semente (W2/L3, 2026-09-11).
//!
//! Esta crate é o destino da família `sculpt3d`, que vivia inteira em
//! `shells/desktop/src/` (111 ficheiros, 31 847 LOC) e é hoje UMA pasta
//! (`shells/desktop/src/sculpt3d/`). A Fase A traz para cá **só o que já não precisa da
//! shell**; o corte do resto é a Fase B, pelo molde da `line/app-host`.
//!
//! ⚠️ **Porque é que a shell depende disto SEM `optional`** — e não é detalhe de
//! empacotamento: a `App` declarava, com a razão escrita ao lado de cada campo, que os
//! pedidos da escultura **não** levam `#[cfg(feature = "sculpt3d")]`. O motivo mais forte é
//! o [`Sculpt3dRequests::doc`]: um binário construído sem a escultura tem de ser um
//! **passa-adiante** dos bytes de um documento já gravado — carregá-los do load ao save sem
//! os ler — e não um triturador. Uma dependência opcional apagaria o campo nesse binário e
//! o artista perderia a escultura ao gravar. ⇒ esta crate tem **zero dependências** e é
//! barata o suficiente para toda a gente a pagar.
//!
//! O que **não** veio, e porquê (a lista que a Fase B fecha):
//! - `sculpt3d_pending`, `sculpt3d_rows`, `sculpt3d_dup`, `sculpt3d_sel` guardam tipos do
//!   módulo (`LoadedPiece`, `SculptRowsSeen`) que ainda vivem na shell. Eles estão
//!   agrupados do outro lado, num `Sculpt3dShellState` gateado que atravessa a fronteira
//!   inteiro quando aqueles tipos mudarem de casa.

/// **AS RÉGUAS DO GESTO** — quanto um pixel de arrasto vale, e onde uma grandeza deixa de
/// existir. Lei pura, zero dependências; era `shells/desktop/src/sculpt3d_rulers.rs`.
pub mod rulers;

// ⚠️ **O glob mantém-se de propósito**: os ~30 filhos leem `super::ORBIT_RAD_PER_PX` como
// sempre leram, e a travessia de crate não move um caminho — tal como o corte de ficheiro não
// movia. ⛔ **E ele estava declarado DUAS vezes** até 2026-09-11 (W2/L3-B): o `mod.rs` da shell
// trazia o dele, e ao ser fundido neste `lib.rs` os dois ficaram lado a lado com **90 linhas
// entre eles**. Um glob duplicado não dá erro e não muda o produto — o `unused import` do
// segundo é o único sinal que existe, e ele lê-se como *«ninguém usa as réguas»*, que é o
// contrário da verdade.
pub use rulers::*;

mod requests;
pub use requests::FAMILY;
pub use requests::Sculpt3dRequests;

// A costura do módulo 3D com o shell — **a cena, o gesto e o passe**.
//
// ⚠️ **A navegação orbital E o gesto de escultura moram AQUI, nunca numa
// `Tool`.** Girar o modelo não é esculpir: o artista gira com o pincel na mão,
// e uma `Tool` que capturasse o ponteiro para navegar teria de devolvê-lo a
// cada gesto. É também o que mantém o contrato congelado intacto (ADR-0150) —
// nenhum método novo em `Tool`.
//
// ⚠️ **Tudo isto é inerte sem a cena armada.** `AppGfx.sculpt3d` nasce `None`, então enquanto
// ninguém a cria cada porta daqui devolve `false` no primeiro `if` e o frame 2D é byte-idêntico.
// ⚠️ **Três coisas a criam, e a terceira é a que o artista alcança:** o smoke (`PH2D_SCULPT3D_SMOKE`),
// um projeto que traz uma escultura dentro, e o **pill SCULPT** — ver [`mode`].

use ph2d_light::LightRig;
use ph2d_mesh::{Hit, Mesh, Multires, Pose, Ray};
use ph2d_mesh_render::{Camera3d, MeshRenderer};
use ph2d_sculpt3d::{Brush, Dab, Grip, SculptStroke, Symmetry, Verb};

/// **A DOAÇÃO** — o carimbo, a rasterização e o interruptor de três posições.
/// Filho para alcançar os campos privados da cena; o corte é *o que
/// o escultor FAZ* (aqui) contra *o que a forma DOA* (lá).
pub mod donation;

/// **O GESTO** — as portas de ponteiro, roda e teclado. Filho para
/// alcançar os campos privados da cena; o corte é *o que a cena É* (aqui) contra
/// *o que a mão FAZ* (lá), o mesmo que separa a [`donation`].
/// ⭐⭐ **QUEM TOMA O GESTO** — o pen-down, separado do que o gesto FAZ. Ver o
/// cabeçalho dele: um é arbitragem, o outro é execução.
pub mod input_down;

pub mod input;
/// ⭐ **As três portas do gesto que só precisam da CENA** (W2/L3-A2) — a shell procura-a (é
/// ela que tem o `gfx`) e estas aplicam a lei. Ver a nota no fim do [`input`]: as outras nove
/// continuam em `impl App` porque leem janela, e essa é a lista que o substrato tem de cobrir.
pub use input::{flush_grab, pointer_move, pointer_up};

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
pub mod keys;

/// ⭐⭐⭐ **O GIZMO DE TRANSFORMAÇÃO** — as alças que se agarram (ordem do Enio,
/// 2026-09-08). Filho pelo motivo dos vizinhos; a LEI das alças é a
/// do módulo de modelagem ([`ph2d_viewport3d::gizmo`]) e o que mora aqui é **esta
/// câmera e a restrição que cada alça impõe ao gesto**.
mod gizmo;

/// ⭐⭐⭐ **OS QUATRO VIEWPORTS** — frente, lado, cima e a vista do artista, ao
/// mesmo tempo (ordem do Enio, 2026-09-08). Filho pelo motivo dos
/// vizinhos; a lei da DIVISÃO é a do módulo de modelagem
/// ([`ph2d_viewport3d::layout`]) e o que mora aqui é **uma câmera por quadrante**.
mod viewports;

/// ⭐⭐ **O GIZMO DA VIEWPORT** — as seis bolas de eixo no canto e as seis vistas
/// nomeadas (ordem do Enio, 2026-09-08). Filho pelo motivo dos
/// vizinhos; a LEI do widget é a do módulo de modelagem
/// ([`ph2d_viewport3d::navball`]) e o que mora aqui é **a câmera desta cena**.
mod navball;

/// ⭐⭐⭐ **A PONTE com a árvore do editor** — uma peça ⟺ uma entidade; ver [`entities`].
pub mod entities;

/// **O CURSOR** — onde a mão está mirando, na tela (W12). Irmão dos três abaixo,
/// e o mais estreito: *onde o gesto vai pousar*, e nada além.
mod cursor;

/// **ENTRAR E SAIR** — o pill SCULPT. Irmão do [`input`] e do [`keys`], e o corte é o mesmo com
/// outro sujeito: aqueles perguntam *o que a mão faz com o barro*, este *quem é dono da tela*.
pub mod mode;
pub use mode::sync_pill;

pub use cursor::{OFF_SURFACE_RGBA, ON_SURFACE_RGBA};

/// **O PAINEL** — o retrato que ele pinta e o gesto que ele devolve (W12).
/// Terceiro irmão do [`input`] e do [`keys`]: o mesmo corte, com um vocabulário
/// próprio (o gesto chega como DADO, um frame depois, pela fila de intents).
mod panel;
pub use panel::Sculpt3dFrameRequest;

/// ⭐ **A FASE da ponte do painel** — o 112.º ficheiro a sair da shell (W2/L3-B).
///
/// ⚠️ Ele vivia no `render_loop/` e não na pasta da família, e por isso não estava no alvo de
/// 111 medido em 11/09. Ele atravessa porque o que ele toca — o [`ph2d_editor::screens::hero::HeroScreen`]
/// — é de uma crate-MÓDULO, não da shell: *o que decide se um ficheiro sai não é a pasta em que
/// ele estava, é o fecho de compilação dele.*
pub mod panel_bridge;

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

/// **AS CENAS DO SMOKE** — a fixture de cada uma. Filho pelo motivo
/// dos outros três: o corte é de responsabilidade, e a lista de cenas cresce uma
/// entrada por wave.
pub mod scenes;

#[cfg(test)]
#[path = "scenes_router_tests.rs"]
mod scenes_router_tests;

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
pub mod bake;

/// **OS VERBOS QUE PUXAM** — Grab, Snake Hook, Twist, Local Scale. Filho pelo motivo dos outros: o corte é de responsabilidade, e o deles
/// é uma LEI própria (a pegada é presa no pen-down, e o alvo é função do puxão
/// TOTAL, nunca da soma dos passos).
mod pull;

/// **O DOCUMENTO** — a cena como bytes, e os bytes como cena. Filho pelo mesmo
/// motivo: ele lê `objects`/`active`/`next_id` e as filas de desfazer.
pub mod doc;

/// **A PORTA DE ENTRADA** — um arquivo de malha vira peças. Filho pelo mesmo
/// motivo: ele constrói `SceneObject`s e mexe na lista.
///
/// ⚠️ **`pub(crate)` desde a W22**: o módulo de modelagem implícita entra por aqui também
/// (`field3d_import`), e ele **não** duplica o leitor — o `read_pieces` é a única resposta da casa a
/// *"que malha há neste arquivo?"*, e uma segunda diria "cor preservada" sobre um STL no dia em que
/// alguém trocasse o leitor.
pub mod import;

/// **A PORTA DE SAÍDA** — a cena vira um arquivo que outro programa abre. Irmão
/// da entrada, e o par dela: sem isto a escultura entra, salva e não sai.
pub mod export;

/// ⭐ **O aviso do que cada formato NÃO carrega**, partilhado com a modelagem 3D
/// ([`crate::field3d_export`]). Uma segunda cópia lá diria *"cor preservada"*
/// sobre um STL no dia em que alguém trocasse o escritor — e um aviso errado é
/// pior que aviso nenhum, porque o artista confia nele. **Uma tabela, um aviso.**
pub use import::is_mesh_file;

// ⚠️ Só o que ATRAVESSA a fronteira do módulo: o `SCULPT_DOC_VERSION` e o
// `SculptDocError` são assunto de dentro (o load só formata o `Display` do
// erro), e re-exportá-los seria superfície que ninguém pede.
pub use doc::{LoadedPiece, decode as decode_doc};

// ⚠️ O ESCRITOR atravessa a fronteira só para os gates: as fixtures de
// `project_tests` precisam de um documento de escultura VÁLIDO, e montá-lo à mão
// lá seria um segundo escritor — que concordaria com este exatamente onde ele
// erra. O `cfg(test)` é o que diz que a superfície é isso e nada mais.
// ⚠️⚠️ **`any(test, feature = "test-support")` desde 2026-09-11 (W2/L3-B), e é a armadilha
// §2.5 do HOWTO à letra:** um `#[cfg(test)]` é **invisível do outro lado de uma crate**. O
// `project_sculpt_tests` da shell consome este escritor, e enquanto a família vivia lá dentro
// o `cfg(test)` dele bastava. ⛔ A cura NÃO é publicá-lo sem cerca — a superfície é exactamente
// isto e nada mais, e a feature é o que o diz.
#[cfg(any(test, feature = "test-support"))]
pub use doc::encode as encode_doc;

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

/// ⭐ **A CENA VIVA** — os 55 campos do que uma escultura é, cortados para o irmão em
/// 2026-09-11 (W2/L3-B) pelo tecto de LOC. ⚠️ Ver o cabeçalho dele: a nota da Fase A dizia que
/// este tecto *«não se cura por corte»*, e estava certa **enquanto a família vivia na shell**.
mod cena;
pub use cena::Sculpt3dScene;

mod birth;

/// ⭐ **O ESTADO DA FAMÍLIA que vive na `App`** — os quatro campos que eram soltos lá
/// (W2/L3-A2). Ver o cabeçalho dele: a fronteira com o irmão não-gateado da crate é imposta
/// pela `cfg`, não escolhida.
pub mod shell_state;
pub use shell_state::Sculpt3dShellState;

/// ⭐⭐ **O teclado da CÂMERA da escultura** — a divisão em quatro e as seis vistas.
/// Irmão do [`keys`] pelo tecto de LOC; ver o cabeçalho dele.
///
/// ⚠️ **Era `crate::sculpt3d_keys_view`, declarado à parte no `main.rs`** — o único
/// ficheiro da família fora desta árvore. Passou a filho em 2026-09-11 (W2/L3-A3), e é
/// por isso que os testes dele mudaram de `sculpt3d_keys_view::…` para
/// `sculpt3d::keys_view::…`: o nome da função não mudou, a casa dela sim.
pub mod keys_view;
