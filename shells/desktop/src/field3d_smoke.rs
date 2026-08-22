//! **O smoke do módulo de modelagem 3D** — `PH2D_FIELD_SMOKE=1..6` (ADR-0161).
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
use ph2d_field_render::{Matcap, Orbit, shade};
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

/// ⭐ **O traçado que está em VOO** — e a bandeira que o pode abandonar (W32).
///
/// ⚠️ O `size` viaja com ele porque a decisão de cancelar precisa de saber **o que** está a correr:
/// um refinamento cede à mão, um traçado de movimento nunca (ver `field3d_preview::cancels_the_inflight`).
pub(crate) struct InFlight {
    pub(crate) rx: Receiver<Ready>,
    pub(crate) cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub(crate) size: (u32, u32),
}

pub(crate) struct Smoke {
    /// A peça a traçar, **cozida da cena** (`field3d_scene`). `None` quando não há geometria
    /// nenhuma — apagar o último filho na Hierarquia é um gesto normal, e o resultado normal dele
    /// é a tela ficar vazia.
    pub(crate) doc: Option<FieldDoc>,
    /// ⭐ **A SEMENTE** — a cena inicial, que a ponte planta **uma vez** e depois deita fora.
    ///
    /// ⚠️ **Ela é separada do `doc` de propósito, e a mistura era um bug.** A ponte oferecia o `doc`
    /// **cozido** como semente, e o comentário ao lado afirmava que ele *"deixa de existir"* — o que
    /// nunca foi verdade: ele é reescrito a cada quadro. O sintoma: apagar a peça na Hierarquia
    /// **fazia-a voltar no quadro seguinte**, porque a ponte não encontrava raiz e replantava o que
    /// tinha acabado de cozer.
    ///
    /// *Uma semente usa-se uma vez.* Depois de plantada, a cena é a fonte — e apagar a peça apaga-a.
    pub(crate) seed: Option<FieldDoc>,
    matcap: Arc<MatcapTexels>,
    pub(crate) cam: Orbit,
    /// O último quadro pronto — com o tamanho a que foi traçado, para o desenhar sem esticar
    /// enquanto o próximo (já do tamanho novo) não chega.
    frame: Option<(Arc<Vec<u8>>, u32, u32)>,
    inflight: Option<InFlight>,
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
    /// ⭐ **O que o último traçado custou, COM o tamanho a que foi medido** — a entrada do laço que
    /// escolhe a resolução do preview ([`crate::field3d_preview`]).
    ///
    /// ⚠️ Separado do `last_trace_ms` de propósito: aquele é para o artista ler, este é para a
    /// máquina decidir, e um tempo sem os pixels ao lado não prevê coisa nenhuma.
    pub(crate) measured: Option<crate::field3d_preview::Measured>,
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
    /// ⭐ **O arrasto do gizmo em curso**, congelado no instante da pegada.
    ///
    /// ⚠️ **A âncora é congelada de propósito.** Ela é republicada a cada quadro a partir da pose do
    /// objeto — que o próprio arrasto está a mudar. Medir o total contra uma âncora que se move
    /// seria medir contra o resultado, e o gesto perseguiria a própria cauda.
    ///
    /// ⭐ E é por medir o **total desde a pegada**, e não incrementos, que prender à grelha funciona:
    /// um total preso é exato, enquanto uma soma de incrementos presos acumula o erro de cada um.
    pub(crate) drag_grip: Option<Grip>,
    /// ⭐ **O gesto está preso à grelha?** — o `Ctrl`, lido no início de cada movimento.
    ///
    /// ⚠️ Lido a cada movimento, e não guardado na pegada: o Blender deixa entrar e sair da grelha a
    /// meio do arrasto, e é o que se quer — mira-se à mão até perto e prende-se no fim. Congelar o
    /// modificador na pegada obrigaria a soltar e repetir o gesto.
    pub(crate) snapping: bool,
    /// ⭐ **O número que está a ser DIGITADO no meio do gesto** (W26, [`crate::field3d_typed`]).
    ///
    /// ⚠️ **Enquanto ele existe, o rato deixa de mandar** — e é isso que faz digitar funcionar: sem
    /// essa cedência, o movimento seguinte do ponteiro sobrescreveria o número no quadro a seguir a
    /// alguém o escrever, e o defeito leria como *"digitar não faz nada"*.
    ///
    /// `None` = não há entrada; o gesto é do ponteiro, como sempre foi.
    pub(crate) typed: Option<String>,
    /// Onde o botão desceu — é o que distingue um **clique** (selecionar) de um **arrasto**
    /// (orbitar). ⚠️ Sem ele, todo clique na peça seria também um giro de zero graus, e a única
    /// forma de selecionar seria a Hierarquia.
    pub(crate) press_at: Option<(f32, f32)>,
    /// Um clique que ainda não foi resolvido: o pixel, no referencial da área desenhada.
    ///
    /// ⚠️ Resolver aqui é impossível — a pergunta *"de quem é este ponto?"* precisa do MUNDO, e o
    /// ponteiro corre fora do quadro. Ele viaja para a ponte pelo mesmo cano dos intents do painel.
    pub(crate) pending_pick: Option<[f32; 2]>,
    /// ⭐ **Que verbo o gizmo está a oferecer** — mover, rodar ou escalar.
    ///
    /// ⚠️ É estado de **vista**, e não do documento: por isso vive aqui e não num componente. O
    /// painel mostra-o e as teclas `G`/`R`/`S` trocam-no.
    pub(crate) gizmo_mode: crate::field3d_gizmo::Mode,
    /// ⭐ **Em que referencial os eixos do gizmo apontam** — do mundo, ou do próprio objeto.
    /// Estado de **vista**, como o verbo.
    pub(crate) gizmo_frame: crate::field3d_gizmo::Frame,
}

/// **O que um arrasto de gizmo guarda** desde a pegada até soltar.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Grip {
    /// A âncora **no instante da pegada** — ver a nota de [`Smoke::drag_grip`].
    pub(crate) anchor: crate::field3d_gizmo::Anchor,
    /// O pixel da pegada, no referencial da área desenhada.
    pub(crate) from: [f32; 2],
    /// O que o mundo **já recebeu** deste gesto. O que falta aplicar é `total.since(applied)`.
    pub(crate) applied: crate::field3d_gizmo::Motion,
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
pub(crate) struct Ready {
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

/// ⭐ **O catálogo das cenas** vive no irmão — ver [`field3d_smoke_scenes`](self::scenes).
#[path = "field3d_smoke_scenes.rs"]
pub(crate) mod scenes;
pub(crate) use scenes::scene;

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
        doc: Some(doc.clone()),
        seed: Some(doc),
        matcap: Arc::new(load_matcap()),
        cam: Orbit::default(),
        frame: None,
        inflight: None,
        since: std::time::Instant::now(),
        requested: None,
        last_trace_ms: 0.0,
        measured: None,
        announced: false,
        area: None,
        drag: None,
        last_pointer: (0.0, 0.0),
        manual: false,
        gizmo: None,
        gizmo_hot: None,
        pending_move: None,
        drag_grip: None,
        snapping: false,
        typed: None,
        press_at: None,
        pending_pick: None,
        gizmo_mode: crate::field3d_gizmo::Mode::default(),
        gizmo_frame: crate::field3d_gizmo::Frame::default(),
    })
}

// ⚠️ **`needs_trace` VIVEU AQUI e foi absorvida** pela `field3d_preview::next_trace` (W24). Ela
// respondia *"vale a pena traçar de novo?"*; a pergunta passou a ser *"traçar de novo a QUE
// tamanho?"*, e as duas na mesma função é a única forma de não haver duas ideias de *o que mudou*.
// A lei que ela defendia — o **documento** faz parte da chave, o smoke do *"slider disfuncional"* —
// continua gateada, agora em `field3d_preview_tests`.

/// ⭐ **Os pedidos que atravessam para o app** vivem no irmão — ver [`field3d_smoke_requests`](self::requests).
#[path = "field3d_smoke_requests.rs"]
mod requests;
use requests::armed_scene;
pub(crate) use requests::{
    ask_export, ask_import, ask_sculpt_extent, ask_spawn_sculpt, set_armed_by_panel,
    take_export_request, take_import_request, take_open_panel_request, take_pending_sculpt,
    take_sculpt_extent,
};
thread_local! {
    /// ⭐ **O registo de esculturas: nome → campo amostrado.**
    ///
    /// ⚠️ **Ele é separado do documento de propósito.** Uma grade de 128³ pesa 12 MB; o documento é
    /// **cozido da cena a cada quadro**, e pô-la lá dentro faria cada quadro copiar isso. O documento
    /// guarda o NOME (`NodeKind::Sampled`), e é aqui que o nome vira campo.
    ///
    /// ⚠️ **`Arc` e não clone**: o traçado corre noutra thread, e o que viaja para lá é um `Arc` por
    /// escultura — o custo de mandar uma escultura para o worker é um incremento de contador.
    static SAMPLED: std::cell::RefCell<ph2d_field_eval::hybrid::Registry> =
        std::cell::RefCell::new(ph2d_field_eval::hybrid::Registry::new());
}

/// O registo, para quem vai avaliar. ⚠️ Devolve uma **cópia dos `Arc`**, que é o que atravessa a
/// fronteira da thread.
pub(crate) fn sampled_registry() -> ph2d_field_eval::hybrid::Registry {
    SAMPLED.with(|r| r.borrow().clone())
}

/// Põe uma escultura no registo, sob um nome.
pub(crate) fn register_sampled(
    key: &str,
    field: std::sync::Arc<dyn ph2d_field_eval::hybrid::Sampled>,
) {
    SAMPLED.with(|r| r.borrow_mut().insert(key.to_string(), field));
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

/// ⭐ **A pintura do quadro** vive no irmão — ver [`field3d_smoke_draw`](self::frame).
#[path = "field3d_smoke_draw.rs"]
mod frame;
pub(crate) use frame::draw;

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
    ///
    /// ⚠️ A pergunta mudou de casa na W24 (passou a devolver **a que tamanho**), e este gate veio
    /// com ela: *uma lei não se apaga quando a função que a carregava é absorvida.*
    #[test]
    fn changing_the_document_asks_for_a_new_trace() {
        use crate::field3d_preview::next_trace;
        let cam = Orbit::default();
        let doc = scene(1);
        let full = (640u32, 480u32);
        let asked = (&cam, full.0, full.1, &doc);

        assert_eq!(
            next_trace(Some(asked), &cam, &doc, full, None, true, MIN_TRACE),
            None,
            "nada mudou e já está no tamanho cheio: traçar de novo seria queimar um núcleo por nada"
        );

        let mut edited = doc.clone();
        edited.set_radius(edited.root(), 0.2).expect("raio válido");
        assert!(
            next_trace(Some(asked), &cam, &edited, full, None, true, MIN_TRACE).is_some(),
            "o DOCUMENTO mudou e o traçado tem de correr — foi esta a linha que faltava"
        );

        // E as outras entradas continuam a contar.
        let mut moved = cam;
        crate::field3d_input::law::orbit(&mut moved, 10.0, 0.0);
        assert!(
            next_trace(Some(asked), &moved, &doc, full, None, true, MIN_TRACE).is_some(),
            "a câmera mudou"
        );
        assert_eq!(
            next_trace(Some(asked), &cam, &doc, (800, 480), None, true, MIN_TRACE),
            Some((800, 480)),
            "a área mudou de tamanho: o traçado novo sai NÍTIDO, não grosso"
        );
        // Sem quadro nenhum, traça — mesmo com tudo igual.
        assert_eq!(
            next_trace(Some(asked), &cam, &doc, full, None, false, MIN_TRACE),
            Some(full),
            "sem quadro nenhum traça, e traça CHEIO: o primeiro traçado é a medição"
        );
    }

    /// ⭐ **Toda cena do smoke constrói E DESENHA.**
    ///
    /// O modo de falha deste smoke não é o pânico: é a **janela vazia** — a peça fora do quadro, o
    /// perfil recusado, o campo que saiu sem interior. A linha *"primeiro quadro desenhado — N
    /// pixels"* existe para o Enio conseguir ver isso; este gate existe para ninguém precisar de
    /// abrir a janela para saber.
    /// ⭐ **A cena 6 é a PONTE, e o gate mede que ela é MISTA** — não uma peça analítica disfarçada.
    ///
    /// ⚠️ **A cena traçar alguma coisa não prova nada aqui.** Se a escultura não chegasse ao registo,
    /// o nome ficaria por resolver, leria como espaço vazio, e a subtração devolveria... vazio — o
    /// gate irmão apanharia isso. Mas se alguém trocasse a escultura por uma esfera analítica, tudo
    /// continuaria a passar e a ponte deixaria de ser exercitada por teste nenhum.
    #[test]
    fn the_bridge_scene_really_has_a_sculpture_in_it() {
        let doc = scene(6);
        let reg = sampled_registry();
        let h = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
        assert_eq!(
            h.sampled_count(),
            1,
            "a cena 6 tem de ter UMA escultura — se der 0, o nome não chegou ao registo"
        );
        assert_eq!(h.tape_count(), 1, "e o cilindro é a única árvore analítica");

        // ⭐ **E a SILHUETA prova que a caixa da grade não virou peça.**
        //
        // ⚠️ Este é o gate do smoke reprovado de 21/08 (*"um objeto texturizado dentro de um cubo
        // furado"*): a costura entre os dois regimes do campo amostrado caía a zero na parede da
        // caixa, e a marcha encontrava ali uma superfície. Medido, 640×480:
        //
        // | | pixels de peça | fração do quadro | relógio |
        // |---|---:|---:|---:|
        // | **cubo** falso (a costura caía a zero) | 215 921 | **70,3 %** | 20,0 ms |
        // | **plano** falso (a parede lia zero) | 128 608 | **41,9 %** | 23,1 ms |
        // | curado | 80 581 | **26,2 %** | 25,0 ms |
        //
        // ⚠️ **Os dois defeitos eram mais RÁPIDOS que o certo**, e é por isso que o relógio não
        // serve de gate aqui: os raios paravam mais cedo, na parede. Quem separa os três casos é a
        // **área**, e a barra fica entre 26,2 % e 41,9 %.
        let g = ph2d_field_render::trace(&doc, &reg, &Orbit::default(), 320, 240);
        let covered = g.hits() as f64 / (320.0 * 240.0);
        assert!(
            (0.18..0.35).contains(&covered),
            "a peça cobre {:.1} % do quadro — acima de 35 % é a caixa da grade a virar superfície \
             (plano a 41,9 %, cubo a 70,3 %), abaixo de 18 % é a escultura a não chegar",
            covered * 100.0
        );
    }

    #[test]
    fn every_smoke_scene_builds_and_draws_something() {
        for n in 1..=6 {
            let doc = scene(n);
            let g =
                ph2d_field_render::trace(&doc, &sampled_registry(), &Orbit::default(), 160, 120);
            assert!(
                g.hits() > 200,
                "a cena {n} traçou só {} pixels de peça em 160x120 — a peça está fora do quadro \
                 ou o campo saiu vazio",
                g.hits()
            );
        }
    }
}
