//! **O smoke do módulo de modelagem 3D** — `PH2D_FIELD_SMOKE=1..3` (ADR-0161).
//!
//! Põe na tela o que o módulo de facto é: o **campo traçado**, não uma malha. É por aqui que o Enio
//! vê a quina de navalha e o filete liso que a W0 mediu.
//!
//! # Gira sozinho ATÉ ALGUÉM PEGAR nele
//!
//! A peça roda em prato giratório — a lei da casa, *feature nova = auto-play*. Mas ao primeiro
//! arrasto ou passo de roda ela **para onde a mão a deixou** ([`Smoke::manual`]): continuar a girar
//! depois disso é desfazer o gesto do artista a cada quadro. A navegação em si vive no arquivo irmão
//! [`crate::field3d_input`], que é também onde estão as quatro linhas que este módulo põe no
//! `input_dispatch.rs`.
//!
//! # Estado contido, de propósito
//!
//! O estado vive **neste arquivo**, num `thread_local`, em vez de num campo do `App`. Não é
//! preguiça: `app_state.rs` é compartilhado e a `line/sculpt3d` edita-o — um campo novo lá é uma
//! colisão por conveniência. A porta é [`with_smoke`].
//!
//! # A requisição em voo é UMA, e só se traça o que MUDOU
//!
//! Traçar custa dezenas de milissegundos (medido, `docs/3DModeling/05_resultados_imagem.md`), e
//! fazê-lo dentro do laço de quadro comeria o orçamento inteiro (HR-4). Então o traçado roda **fora**
//! da thread de UI, com **uma requisição em voo por vez** — a mesma disciplina que o modelador
//! original pagou para descobrir (`docs/3DModeling/00_plano_port.md` §1.2.7): as respostas que
//! chegam durante a espera já nasceram velhas, e só a última interessa.
//!
//! E a requisição só sai quando a câmera ou o tamanho mudaram. Com o prato a girar isso é todo
//! quadro; com a mão no controlo, **uma peça parada custa zero**.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use ph2d_editor::zones::Rect as EditorRect;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Profile, Xform};
use ph2d_field_render::{Matcap, Orbit, shade, trace};
use ph2d_vec_scene::{VecPath, VecVertex};
use ph2d_vector::{Affine, ImageQuality, VectorScene};

/// O menor traçado que ainda é uma imagem — só para não pedir zero pixels a uma área degenerada.
const MIN_TRACE: u32 = 16;

/// Quanto a peça gira **por segundo**, em radianos.
///
/// ⚠️ **Por segundo, e não por quadro** (correção do smoke de 19/08). Com um passo por quadro, a
/// velocidade da peça era função do custo do traçado: baixar a resolução acelerava a rotação e
/// subi-la travava-a. Isso confunde as duas perguntas que um prato giratório responde — *"a forma
/// está certa?"* e *"isto corre depressa?"* — e faz a segunda mentir sobre a primeira.
const SPIN_RATE: f32 = 0.5;

/// O fundo do quadro: **transparente**.
///
/// ⚠️ **Correção de um smoke do Enio (19/08):** *"o fundo está cinza escuro e acima do canvas"*.
/// Um cinza opaco aqui era eu **inventando uma cor** — e uma cor de fundo inventada num app com
/// tema é a segunda resposta a uma pergunta que o tema já responde (HR-15). Com alfa zero o canvas
/// do app aparece por baixo, e o módulo deixa de ter opinião sobre o fundo.
const BACKGROUND: [u8; 4] = [0, 0, 0, 0];

pub(crate) struct Smoke {
    /// A peça a traçar, **cozida da cena** (`field3d_scene`). `None` quando não há geometria
    /// nenhuma — apagar o último filho na Hierarquia é um gesto normal, e o resultado normal dele
    /// é a tela ficar vazia.
    pub(crate) doc: Option<FieldDoc>,
    matcap: Arc<MatcapTexels>,
    pub(crate) cam: Orbit,
    /// O último quadro pronto — com o tamanho a que foi traçado, para o desenhar sem esticar
    /// enquanto o próximo (já do tamanho novo) não chega.
    frame: Option<(Arc<Vec<u8>>, u32, u32)>,
    inflight: Option<Receiver<Ready>>,
    /// Quando o pedido em voo saiu — é o relógio que faz a rotação ser por SEGUNDO.
    since: std::time::Instant,
    /// **O que já foi pedido**: a câmera, o tamanho **e o DOCUMENTO**.
    ///
    /// ⚠️ **O documento faz parte da chave, e não fazia** — foi um smoke reprovado (Enio,
    /// 2026-08-19: *"slider disfuncional"*). O controle mudava o raio, o documento mudava, o painel
    /// mostrava o número novo... e a peça na tela **não se mexia**, porque a pergunta *"mudou
    /// alguma coisa?"* olhava só a câmera e o tamanho.
    ///
    /// *Um cache que não conhece uma das entradas não é um cache — é um congelador.* E o sintoma
    /// culpava o controle, que estava certo.
    requested: Option<(Orbit, u32, u32, FieldDoc)>,
    /// Quanto custou o último traçado — o número que o painel mostra, porque quem mexe num raio
    /// é quem paga o traçado seguinte.
    pub(crate) last_trace_ms: f32,
    /// Já anunciou o primeiro quadro? Ver a nota do `boot`.
    announced: bool,
    /// **A área onde o quadro foi desenhado da última vez** — é ela que responde *"este clique é
    /// meu?"*. Sem isto a cena engoliria gestos de qualquer canto da janela.
    pub(crate) area: Option<EditorRect>,
    /// O arrasto em curso, se houver ([`crate::field3d_input`]).
    pub(crate) drag: Option<Drag>,
    /// A última posição do ponteiro, para o delta do arrasto.
    pub(crate) last_pointer: (f32, f32),
    /// ⭐ **O prato para de girar assim que o artista toca nele.**
    ///
    /// A lei da casa é *feature nova = auto-play*: a peça gira sozinha para provar que existe. Mas
    /// continuar a girar **depois** de alguém a ter posto num ângulo é desfazer o gesto dele a cada
    /// quadro — a auto-demonstração deixa de ser um convite e passa a ser uma disputa.
    pub(crate) manual: bool,
    /// ⭐ **Onde o gizmo está**, publicado pela ponte com a cena — que é quem tem o mundo. `None`
    /// quando nada de modelagem está selecionado.
    ///
    /// ⚠️ A âncora atravessa o quadro em vez de ser recalculada aqui de propósito: o `draw` não tem
    /// mundo nenhum, e dar-lhe um significaria o traçado passar a depender do ECS.
    pub(crate) gizmo: Option<crate::field3d_gizmo::Anchor>,
    /// A alça sob o cursor. Só realce — quem manda no arrasto é o `drag`.
    pub(crate) gizmo_hot: Option<crate::field3d_gizmo::Handle>,
    /// O que o arrasto pediu e a ponte ainda não aplicou: `(entidade, pedido)`.
    ///
    /// ⚠️ **Acumula**, e não substitui: entre dois quadros chegam vários eventos de ponteiro, e
    /// guardar só o último faria a peça andar menos do que a mão — devagar, e só quando o rato vai
    /// depressa, que é o defeito mais difícil de acreditar. Cada verbo acumula à maneira dele
    /// (`Motion::merge`).
    pub(crate) pending_move: Option<(u64, crate::field3d_gizmo::Motion)>,
    /// ⭐ **Que verbo o gizmo está a oferecer** — mover, rodar ou escalar.
    ///
    /// ⚠️ É estado de **vista**, e não do documento: por isso vive aqui e não num componente. O
    /// painel mostra-o e as teclas `G`/`R`/`S` trocam-no.
    pub(crate) gizmo_mode: crate::field3d_gizmo::Mode,
}

/// O gesto de navegação em curso.
///
/// ⚠️ Os botões são os **mesmos** do módulo de escultura (`sculpt3d_input.rs`): esquerdo e direito
/// orbitam, o do meio faz pan. Não é herança por analogia — são duas janelas 3D no mesmo app, e uma
/// mão que aprendeu a girar numa tem de girar na outra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Drag {
    Orbit,
    Pan,
    /// ⭐ Uma alça do gizmo 3D está agarrada. ⚠️ **Ela ganha do botão esquerdo**, e é o único sítio
    /// onde a navegação cede: uma seta que orbitasse a câmera em vez de mover a peça seria uma alça
    /// pintada e morta.
    Gizmo(crate::field3d_gizmo::Handle),
}

/// O que uma requisição de traçado devolve.
struct Ready {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    hits: usize,
    edges: usize,
    millis: f64,
}

struct MatcapTexels {
    side: u32,
    rgb: Vec<f32>,
}

thread_local! {
    static STATE: RefCell<Option<Option<Smoke>>> = const { RefCell::new(None) };
}

/// **Um contorno desenhado como a caneta o desenharia** — âncoras de quina com **raio vivo** — e
/// depois cozido em perfil.
///
/// ⭐ É esta função que fecha a costura da W3: o caminho é `VecPath` → `cooked()` (os Live Corners
/// do editor vetorial correm aqui) → `Profile`. Nenhum arco é escrito à mão; o arredondamento das
/// arestas verticais do sólido **é** o *corner widget* do editor de vetores.
fn drawn_profile(pts: &[([f64; 2], f64)]) -> Profile {
    let path = VecPath {
        verts: pts
            .iter()
            .map(|&(p, radius)| VecVertex {
                corner_radius: radius,
                ..VecVertex::corner(p)
            })
            .collect(),
        closed: true,
        ..VecPath::default()
    };
    ph2d_field_profile::cook_path_auto(&path).expect("os contornos do smoke são perfis válidos")
}

/// As cenas. ⚠️ **Cada uma imprime o que montou** — se a linha não aparecer no terminal, o smoke
/// não chegou a construir nada, e a tela vazia é sintoma disso e não da geometria.
pub(crate) fn scene(n: u32) -> FieldDoc {
    let combine = |op: Op, children: Vec<NodeId>| Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine { op, children },
    };
    let leaf = |p: Primitive, x: Xform| Node {
        xform: x,
        kind: NodeKind::Leaf(p),
    };
    let s = std::f32::consts::FRAC_1_SQRT_2;

    let doc = match n {
        2 => {
            println!("[field-smoke] cena 2 — cubo com aresta arredondada (r = 0,08)");
            FieldDoc::new(
                vec![leaf(
                    Primitive::Box {
                        half: [0.45; 3],
                        round: 0.08,
                    },
                    Xform::IDENTITY,
                )],
                NodeId(0),
            )
        }
        3 => {
            println!(
                "[field-smoke] cena 3 — caixa arredondada MENOS cilindro, boca do furo em 0,05"
            );
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Box {
                            half: [0.5; 3],
                            round: 0.06,
                        },
                        Xform::IDENTITY,
                    ),
                    leaf(
                        Primitive::Cylinder {
                            radius: 0.24,
                            half_height: 1.2,
                            round: 0.0,
                        },
                        Xform::IDENTITY,
                    ),
                    combine(
                        Op::Difference(Blend::Exact { radius: 0.05 }),
                        vec![NodeId(0), NodeId(1)],
                    ),
                ],
                NodeId(2),
            )
        }
        4 => {
            // A cantoneira: o perfil é DESENHADO (quinas com raio vivo), extrudado com aro
            // arredondado, e furado por dois cilindros cuja boca ganha filete.
            let profile = drawn_profile(&[
                ([-0.35, -0.35], 0.05),
                ([0.35, -0.35], 0.05),
                ([0.35, -0.13], 0.05),
                ([-0.10, -0.13], 0.05),
                ([-0.10, 0.35], 0.05),
                ([-0.35, 0.35], 0.05),
            ]);
            println!(
                "[field-smoke] cena 4 — cantoneira DESENHADA: perfil de {} arestas (quinas vivas \
                 r=0,05), extrusão com aro 0,03, dois furos com a boca em 0,02",
                profile.segment_count()
            );
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Extrude {
                            profile,
                            half_height: 0.14,
                            round: 0.03,
                        },
                        Xform::IDENTITY,
                    ),
                    leaf(
                        Primitive::Cylinder {
                            radius: 0.07,
                            half_height: 1.0,
                            round: 0.0,
                        },
                        Xform::at(0.18, -0.24, 0.0),
                    ),
                    leaf(
                        Primitive::Cylinder {
                            radius: 0.07,
                            half_height: 1.0,
                            round: 0.0,
                        },
                        Xform::at(-0.225, 0.20, 0.0),
                    ),
                    combine(
                        Op::Difference(Blend::Exact { radius: 0.02 }),
                        vec![NodeId(0), NodeId(1), NodeId(2)],
                    ),
                ],
                NodeId(3),
            )
        }
        5 => {
            // O torno: o mesmo tipo de contorno, agora GIRADO em torno de Y. A silhueta é a de um
            // vaso oco — parede externa a subir, lábio, parede interna a descer, e o fecho no eixo.
            let profile = drawn_profile(&[
                ([0.00, -0.45], 0.0),
                ([0.26, -0.45], 0.05),
                ([0.30, -0.34], 0.05),
                ([0.15, -0.10], 0.06),
                ([0.33, 0.22], 0.06),
                ([0.27, 0.44], 0.04),
                ([0.33, 0.52], 0.02),
                ([0.27, 0.52], 0.02),
                ([0.21, 0.44], 0.04),
                ([0.09, -0.08], 0.05),
                ([0.19, -0.32], 0.04),
                ([0.00, -0.32], 0.0),
            ]);
            println!(
                "[field-smoke] cena 5 — TORNO: o mesmo contorno desenhado ({} arestas) girado em \
                 torno de Y — um vaso oco, com a parede a sair do desenho",
                profile.segment_count()
            );
            FieldDoc::new(
                vec![leaf(Primitive::Revolve { profile }, Xform::IDENTITY)],
                NodeId(0),
            )
        }
        _ => {
            println!(
                "[field-smoke] cena 1 — junção de 3 cilindros: filete interno 0,12 + aros externos 0,05"
            );
            let cyl = |rot: [f32; 4]| {
                leaf(
                    Primitive::Cylinder {
                        radius: 0.22,
                        half_height: 0.78,
                        round: 0.05,
                    },
                    Xform {
                        rotation: rot,
                        ..Xform::IDENTITY
                    },
                )
            };
            FieldDoc::new(
                vec![
                    cyl([0.0, 0.0, 0.0, 1.0]),
                    cyl([s, 0.0, 0.0, s]),
                    cyl([0.0, s, 0.0, s]),
                    combine(
                        Op::Union(Blend::Exact { radius: 0.12 }),
                        vec![NodeId(0), NodeId(1), NodeId(2)],
                    ),
                ],
                NodeId(3),
            )
        }
    };
    doc.expect("as cenas do smoke são documentos válidos")
}

/// Carrega um matcap da casa e converte para linear f32.
///
/// ⚠️ **Os matcaps moram na `ph2d-mesh-render`, com os assets e a licença** — e é de lá que se
/// pegam, em vez de sintetizar aqui um sombreamento novo. O acoplamento é do **smoke**, não do
/// módulo: a `ph2d-field-render` recebe os texels por parâmetro e não conhece aquela crate.
#[cfg(feature = "sculpt3d")]
fn load_matcap() -> MatcapTexels {
    let id = 0usize;
    let side = ph2d_mesh_render::matcap::MATCAPS[id].side;
    let bytes = ph2d_mesh_render::matcap::decode(id);
    let n = (side as usize) * (side as usize);
    let mut rgb = Vec::with_capacity(n * 3);
    for texel in bytes.chunks_exact(8) {
        // RGBA em `f16` little-endian; o alfa é descartado (é 1 em toda parte, por construção).
        for c in 0..3 {
            let bits = u16::from_le_bytes([texel[c * 2], texel[c * 2 + 1]]);
            rgb.push(half::f16::from_bits(bits).to_f32());
        }
    }
    MatcapTexels { side, rgb }
}

/// Sem o módulo de escultura compilado não há matcap — e um cinza plano seria uma forma ilegível.
#[cfg(not(feature = "sculpt3d"))]
fn load_matcap() -> MatcapTexels {
    println!("[field-smoke] ⚠️ sem a feature `sculpt3d` não há matcap; o smoke fica sem cor");
    MatcapTexels {
        side: 0,
        rgb: Vec::new(),
    }
}

fn boot() -> Option<Smoke> {
    let n = armed_scene()?;
    let doc = scene(n);
    println!(
        "[field-smoke] traçado no tamanho REAL da área, com anti-serrilhado — prato giratório, \
         feche a janela para sair"
    );
    Some(Smoke {
        doc: Some(doc),
        matcap: Arc::new(load_matcap()),
        cam: Orbit::default(),
        frame: None,
        inflight: None,
        since: std::time::Instant::now(),
        requested: None,
        last_trace_ms: 0.0,
        announced: false,
        area: None,
        drag: None,
        last_pointer: (0.0, 0.0),
        manual: false,
        gizmo: None,
        gizmo_hot: None,
        pending_move: None,
        gizmo_mode: crate::field3d_gizmo::Mode::default(),
    })
}

/// **Vale a pena traçar de novo?** — a pergunta inteira, num só sítio e pura.
///
/// ⚠️ **As TRÊS entradas contam, e a terceira foi esquecida.** A primeira versão comparava só a
/// câmera e o tamanho; mexer num raio mudava o documento, o painel mostrava o número novo, e a peça
/// na tela **não se mexia** (Enio, 2026-08-19: *"slider disfuncional"*). *Um cache que não conhece
/// uma das entradas não é um cache — é um congelador*, e o sintoma culpa o controle, que está certo.
///
/// Pura e separada de propósito: é o que a torna gateável sem janela nem GPU.
fn needs_trace(
    requested: Option<&(Orbit, u32, u32, FieldDoc)>,
    cam: &Orbit,
    w: u32,
    h: u32,
    doc: &FieldDoc,
    has_frame: bool,
) -> bool {
    // Sem quadro nenhum ainda: traçar, mesmo que nada tenha "mudado".
    if !has_frame {
        return true;
    }
    requested.is_none_or(|(c, rw, rh, rdoc)| c != cam || *rw != w || *rh != h || rdoc != doc)
}

/// **O painel abre sozinho na primeira vez** que o smoke desenha, e só nessa.
///
/// ⭐ *Feature nova = auto-play* é a lei da casa, e um painel que exigisse aprender uma tecla para
/// aparecer é uma feature que ninguém alcança. Abrir **uma vez** é o que reconcilia isso com o botão
/// de fechar: reabri-lo todo quadro faria o X não funcionar, que é a forma mais irritante de duas
/// portas discordarem.
pub(crate) fn take_open_panel_request() -> bool {
    thread_local! {
        static PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    }
    // Só pede se o smoke está de facto armado — senão o painel de modelagem abriria em toda sessão
    // do app, ocupando o encaixe da direita para não mostrar nada.
    if with_smoke(|_| ()).is_none() {
        return false;
    }
    PENDING.with(|p| p.replace(false))
}

thread_local! {
    /// O pill do topo pediu o módulo ligado.
    static PILL_ARMED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// **A porta de ARMAR**, escrita pelo shell a partir da visibilidade do painel.
///
/// ⭐ O módulo passa a ter **duas** entradas: a variável de ambiente (a do smoke dirigido) e o
/// **pill do topo** — que é a que um artista encontra. Enquanto a única porta era a `env`, o módulo
/// não existia para quem abre o app (Enio, 2026-08-19: *"não temos um Pill no topo"*).
///
/// ⚠️ Uma porta que só quem já sabe consegue abrir é o mesmo que não existir — a lição que o pill do
/// SCULPT já tinha registado ao lado, e que este módulo repetiu.
pub(crate) fn set_armed_by_panel(open: bool) {
    // ⚠️ **Trava ligado, e não segue o painel.** Fechar o painel fecha o PAINEL; a peça continua na
    // cena, porque ela é um objeto do documento — está na Hierarquia, é salva e é desfeita. Fazer o
    // X do painel apagar o modelo da tela seria um segundo significado para o mesmo gesto, e o
    // artista perderia a peça sem a ter apagado.
    //
    // Desligar o módulo é apagar a entidade, que é trabalho da Hierarquia — e é a única porta que
    // pode fazê-lo sem ambiguidade.
    if open {
        PILL_ARMED.with(|c| c.set(true));
    }
}

fn armed_scene() -> Option<u32> {
    if let Ok(v) = std::env::var("PH2D_FIELD_SMOKE") {
        return Some(v.parse().unwrap_or(1));
    }
    // Aberto pelo pill: a cena 1 é a que mostra os dois arredondamentos de uma vez.
    PILL_ARMED.with(std::cell::Cell::get).then_some(1)
}

/// **A porta única para o estado do smoke**, e é por ela que a metade de entrada chega.
///
/// ⚠️ O estado vive num `thread_local` deste arquivo, e não num campo do `App`, de propósito: o
/// `app_state.rs` é compartilhado e a `line/sculpt3d` edita-o. Um campo novo lá seria uma colisão
/// por conveniência. Isto custa uma função e não custa um conflito.
///
/// Devolve `None` quando o smoke não está armado — e é isso que faz cada gancho de entrada ser
/// **inerte** (e portanto invisível) fora dele.
pub(crate) fn with_smoke<R>(f: impl FnOnce(&mut Smoke) -> R) -> Option<R> {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        // ⚠️ **Re-tenta enquanto não nasceu**, e não uma vez só: com o pill, o módulo pode ser
        // armado a meio da sessão. Um `get_or_insert_with` puro guardaria o `None` da primeira
        // pergunta e o pill nunca acenderia nada — o mesmo defeito de "a porta existe e não abre"
        // que este módulo acabou de pagar noutro sítio.
        if slot.as_ref().is_none_or(Option::is_none) {
            *slot = Some(boot());
        }
        slot.as_mut().and_then(Option::as_mut).map(f)
    })
}

/// ⭐ **Há um gesto de AUTORIA em curso?** — a pergunta que o undo faz.
///
/// ⚠️ Só o arrasto do **gizmo** conta. Orbitar e deslocar a vista não tocam no documento: suprimir
/// o undo neles não estragaria nada, mas afirmaria uma coisa falsa sobre o que eles fazem.
///
/// ⚠️ **Sem isto, um arrasto vira N passos de undo — um por quadro.** O `post_frame_undo` já tem a
/// lei («um gesto em andamento espera o fim»), e ela lê o `held_button` do shell — que **este
/// módulo nunca chega a pôr**, porque o gancho do ponteiro consome o `Down` e volta antes da linha
/// que o escreve. A lei estava certa e não alcançava este gesto.
pub(crate) fn gesture_in_progress() -> bool {
    with_smoke(|s| matches!(s.drag, Some(Drag::Gizmo(_)))).unwrap_or(false)
}

/// Desenha o smoke sobre a área dada. No-op silencioso quando a variável não está posta.
pub(crate) fn draw(area: EditorRect, theme: ph2d_tokens::Theme, scene_out: &mut VectorScene) {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        let smoke = slot.get_or_insert_with(boot);
        let Some(smoke) = smoke.as_mut() else {
            return;
        };

        // Colhe o traçado que ficou pronto, se ficou.
        if let Some(rx) = &smoke.inflight {
            match rx.try_recv() {
                Ok(r) => {
                    if !smoke.announced {
                        smoke.announced = true;
                        // ⚠️ Uma linha, uma vez. É ela que separa "o smoke subiu" de "o smoke
                        // DESENHOU": o boot já imprime acima, e um boot sem quadro é exatamente o
                        // modo de falha em que a janela fica vazia e ninguém sabe de quem é a culpa.
                        // Zero pixels aqui = a peça está fora do quadro ou o campo saiu vazio.
                        println!(
                            "[field-smoke] primeiro quadro desenhado — {}x{}, {} pixels de peça, \
                             {} de borda re-amostrada, {:.1} ms",
                            r.width, r.height, r.hits, r.edges, r.millis
                        );
                    }
                    smoke.last_trace_ms = r.millis as f32;
                    smoke.frame = Some((Arc::new(r.rgba), r.width, r.height));
                    smoke.inflight = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => smoke.inflight = None,
            }
        }

        // ⚠️ **O traçado sai no tamanho REAL da área** (correção do smoke de 19/08: *"render
        // pixelado"*). Antes ele era fixo em 640×480 e o desenho reamostrava para a área — e uma
        // imagem reamostrada é uma imagem com metade da informação, num módulo cuja razão de
        // existir é a nitidez da aresta. *A resolução do traçado é a da tela, não a de um número
        // que alguém escolheu.*
        let (tw, th) = (
            (area.w.round().max(1.0) as u32).max(MIN_TRACE),
            (area.h.round().max(1.0) as u32).max(MIN_TRACE),
        );

        smoke.area = Some(area);

        // ⚠️ **Peça vazia é uma resposta, não um erro** — e a tela tem de a mostrar. Guardar o
        // último quadro válido faria a imagem mentir sobre a cena, que é a irmã exacta do
        // congelador que este módulo já pagou no cache do traçado.
        let Some(doc) = smoke.doc.clone() else {
            smoke.frame = None;
            smoke.requested = None;
            return;
        };

        if !smoke.manual && smoke.inflight.is_none() {
            let dt = smoke.since.elapsed().as_secs_f32();
            smoke.since = std::time::Instant::now();
            // Trava o passo: se a janela ficou minimizada meia hora, a peça não dá vinte voltas
            // de uma vez.
            // ⚠️ Em torno do Y do **MUNDO**, e não de um eixo da câmera: um prato giratório é
            // exatamente isso — a peça a girar sobre a mesa, com o horizonte parado. É o único
            // sítio deste módulo onde um eixo do mundo entra na conta, e é de propósito.
            smoke
                .cam
                .turn_world([0.0, 1.0, 0.0], -SPIN_RATE * dt.min(0.25));
        }

        // ⭐ **Só se traça o que MUDOU.** Uma requisição em voo por vez, e só quando a câmera ou o
        // tamanho já não são os do último pedido — senão uma cena parada re-traçaria o mesmo quadro
        // para sempre, queimando um núcleo por nada. Com o prato a girar a câmera muda todo quadro,
        // então isto é invisível ali; com a mão no controlo, uma peça parada custa **zero**.
        let stale = needs_trace(
            smoke.requested.as_ref(),
            &smoke.cam,
            tw,
            th,
            &doc,
            smoke.frame.is_some(),
        );
        if smoke.inflight.is_none() && stale {
            smoke.requested = Some((smoke.cam, tw, th, doc.clone()));
            let (tx, rx) = channel::<Ready>();
            let cam = smoke.cam;
            let matcap = Arc::clone(&smoke.matcap);
            std::thread::spawn(move || {
                let t0 = std::time::Instant::now();
                let g = trace(&doc, &cam, tw, th);
                let rgba = shade(
                    &g,
                    &Matcap {
                        side: matcap.side,
                        rgb_linear: &matcap.rgb,
                    },
                    BACKGROUND,
                );
                // O receptor pode ter sumido (janela fechada): descartar é a resposta certa.
                let _ = tx.send(Ready {
                    rgba,
                    width: tw,
                    height: th,
                    hits: g.hits(),
                    edges: g.edges.len(),
                    millis: t0.elapsed().as_secs_f64() * 1000.0,
                });
            });
            smoke.inflight = Some(rx);
        }

        if let Some((frame, fw, fh)) = &smoke.frame {
            // O quadro cobre a área toda — ele foi traçado com a proporção dela. Enquanto o
            // primeiro traçado do tamanho novo não chega, o anterior estica; é um quadro só, e
            // esticar é melhor do que piscar.
            scene_out.draw_image_rgba_premultiplied_transformed(
                frame,
                *fw,
                *fh,
                Affine::translate((f64::from(area.x), f64::from(area.y)))
                    * Affine::scale_non_uniform(
                        f64::from(area.w) / f64::from(*fw),
                        f64::from(area.h) / f64::from(*fh),
                    ),
                // ⚠️ **Bilinear, não bicúbico.** No caso normal o mapeamento é 1:1 e os dois são a
                // identidade — mas o bicúbico **toca** (*ringing*) numa aresta de alto contraste, e
                // agora que a aresta sai anti-serrilhada do traçador, um halo posto pelo filtro
                // seria o próprio artefato que se acabou de remover.
                ImageQuality::Medium,
            );
        }

        // ⭐ **O gizmo por cima da peça**, e no referencial da área.
        //
        // ⚠️ Ele é desenhado **depois** do quadro traçado e **sem teste de profundidade**: uma alça
        // escondida por trás da superfície que ela move seria inalcançável exatamente quando o
        // artista precisa dela. É o que todo modelador faz, e a razão é essa.
        if let Some(anchor) = smoke.gizmo {
            let screen = ph2d_field_render::Screen::new(tw, th, smoke.cam.half_extent);
            let handles =
                crate::field3d_gizmo::project(anchor, &smoke.cam, screen, smoke.gizmo_mode);
            let hot = crate::field3d_input::hot_handle(smoke);
            scene_out.push_clip(&ph2d_vector::Rect::new(
                f64::from(area.x),
                f64::from(area.y),
                f64::from(area.x + area.w),
                f64::from(area.y + area.h),
            ));
            crate::field3d_gizmo_paint::paint(scene_out, &handles, hot, theme, [area.x, area.y]);
            scene_out.pop_layer();
        }
    });
}

/// ⚠️ A metade de shell da ponte ECS vive num arquivo irmão, pendurada aqui pelo padrão do
/// `joint_rig`: o que ela prova — que o componente sobrevive ao snapshot real — só o shell sabe.
#[cfg(test)]
#[path = "field3d_snapshot_tests.rs"]
mod snapshot_tests;

#[cfg(test)]
mod trace_tests {
    use super::*;

    /// ⭐ **Mudar o DOCUMENTO pede um traçado novo** — o gate do *"slider disfuncional"*.
    ///
    /// A primeira versão da pergunta *"mudou alguma coisa?"* olhava a câmera e o tamanho. Um raio
    /// editado mudava o documento, o painel mostrava o número novo, e a peça na tela ficava
    /// **congelada** — com o controle a levar a culpa.
    #[test]
    fn changing_the_document_asks_for_a_new_trace() {
        let cam = Orbit::default();
        let doc = scene(1);
        let asked = (cam, 640, 480, doc.clone());

        assert!(
            !needs_trace(Some(&asked), &cam, 640, 480, &doc, true),
            "nada mudou: traçar de novo seria queimar um núcleo por nada"
        );

        let mut edited = doc.clone();
        edited.set_radius(edited.root(), 0.2).expect("raio válido");
        assert!(
            needs_trace(Some(&asked), &cam, 640, 480, &edited, true),
            "o DOCUMENTO mudou e o traçado tem de correr — foi esta a linha que faltava"
        );

        // E as outras duas entradas continuam a contar.
        let mut moved = cam;
        crate::field3d_input::law::orbit(&mut moved, 10.0, 0.0);
        assert!(needs_trace(Some(&asked), &moved, 640, 480, &doc, true));
        assert!(needs_trace(Some(&asked), &cam, 800, 480, &doc, true));
        // Sem quadro nenhum, traça — mesmo com tudo igual.
        assert!(needs_trace(Some(&asked), &cam, 640, 480, &doc, false));
    }

    /// ⭐ **Toda cena do smoke constrói E DESENHA.**
    ///
    /// O modo de falha deste smoke não é o pânico: é a **janela vazia** — a peça fora do quadro, o
    /// perfil recusado, o campo que saiu sem interior. A linha *"primeiro quadro desenhado — N
    /// pixels"* existe para o Enio conseguir ver isso; este gate existe para ninguém precisar de
    /// abrir a janela para saber.
    #[test]
    fn every_smoke_scene_builds_and_draws_something() {
        for n in 1..=5 {
            let doc = scene(n);
            let g = ph2d_field_render::trace(&doc, &Orbit::default(), 160, 120);
            assert!(
                g.hits() > 200,
                "a cena {n} traçou só {} pixels de peça em 160x120 — a peça está fora do quadro \
                 ou o campo saiu vazio",
                g.hits()
            );
        }
    }
}
