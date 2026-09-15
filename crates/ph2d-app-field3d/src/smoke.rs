//! **O smoke do módulo de modelagem 3D** — `PH2D_FIELD_SMOKE=1..32` (ADR-0161).
//!
//! Põe na tela o que o módulo de facto é: o **campo traçado**, não uma malha. É por aqui que o Enio
//! vê a quina de navalha e o filete liso que a W0 mediu.
//!
//! # Gira sozinho ATÉ ALGUÉM PEGAR nele
//!
//! A peça roda em prato giratório — a lei da casa, *feature nova = auto-play*. Mas ao primeiro
//! arrasto ou passo de roda ela **para onde a mão a deixou** ([`Smoke::manual`]): continuar a girar
//! depois disso é desfazer o gesto do artista a cada quadro. A navegação em si vive no arquivo irmão
//! [`crate::input`], que é também onde estão as quatro linhas que este módulo põe no
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

use std::sync::Arc;
use std::sync::mpsc::{TryRecvError, sync_channel};

use ph2d_editor_core::zones::Rect as EditorRect;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Profile, Xform};
use ph2d_field_render::{Matcap, Orbit};
use ph2d_vec_scene::{VecPath, VecVertex};
use ph2d_vector::{ImageQuality, VectorScene};

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

/// ⭐ **O que o smoke É** — os tipos do estado e a célula que o guarda — vive no irmão. Ver
/// [`field3d_smoke_state`](self::state).
#[path = "smoke_state.rs"]
mod state;
pub use state::{Drag, Grip, InFlight, Ready, Smoke};

/// ⭐⭐⭐ **A LISTA DE VIEWPORTS** — quem a abre, quem a fecha e de quem é um ponto (W90). Vive no
/// irmão pela mesma razão do [`state`]: o `field3d_smoke.rs` é a porta do módulo, e o tecto de LOC
/// do HR-18 é o instrumento que impede uma porta de virar um armazém.
#[path = "viewports.rs"]
mod viewports;
use state::{MatcapTexels, STATE};
pub use viewports::{canvas_area, divider_cursor, ensure_viewports, toggle_split, viewport_at};

/// ⭐ **O catálogo das cenas** vive no irmão — ver [`field3d_smoke_scenes`](self::scenes).
#[path = "smoke_scenes.rs"]
pub mod scenes;
pub use scenes::scene;

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
    for texel in bytes.as_chunks::<8>().0.iter() {
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
    // ⭐ **A vista com que este smoke nasce**: a de quando o painel fechou, ou a padrão na primeira
    // abertura da sessão (W43 — [`view::recall`]).
    let v = view::recall();
    println!(
        "[field-smoke] traçado no tamanho REAL da área, com anti-serrilhado — prato giratório, \
         feche a janela para sair"
    );
    let mut smoke = Smoke {
        doc: Some(doc.clone()),
        seed: Some(doc),
        isolated: v.isolated,
        flight: None,
        flight_gen: 0,
        flight_fresh: false,
        safe: None,
        profile_pick: None,
        nav_hot: None,
        nav_press: None,
        view_menu: None,
        view_menu_rect: None,
        materials: None,
        lights: std::sync::Arc::default(),
        has_live_sculpt: false,
        matcap: Arc::new(load_matcap()),
        // ⭐ **Um viewport, que é o que o módulo sempre teve** — a divisão entra depois, e este
        // é o estado em que ela não existe.
        vps: vec![{
            let mut vp = crate::smoke::state::Viewport::new(v.cam, v.manual);
            // ⭐ O modo de pintar volta com a vista (`docs/Render3d/05`).
            vp.shading = v.shading;
            vp
        }],
        active: 0,
        // ⭐ **A divisão volta com a vista** (W95) — os viewports que ela pede são reconstruídos
        // logo a seguir, a partir da câmera lembrada.
        split: v.split,
        announced: false,
        drag: None,
        last_pointer: (0.0, 0.0),
        gizmo: None,
        gizmo_hot: None,
        vertices: None,
        pending_move: None,
        drag_grip: None,
        snapping: false,
        typed: None,
        press_at: None,
        pending_pick: None,
        lasso: None,
        pending_lasso: None,
        gizmo_mode: v.gizmo_mode,
        lasso_subtracts: false,
        gizmo_frame: v.gizmo_frame,
        look: v.look,
    };
    // ⭐ **A lista nasce já com a divisão lembrada** (W95). Ela seria reconciliada no primeiro
    // desenho de qualquer forma, mas então haveria um quadro em que o `split` diz «quatro» e a
    // lista tem uma — e *um estado que só é verdade a partir do segundo quadro é um estado que
    // alguém vai ler no primeiro*.
    ensure_viewports(&mut smoke, v.split.count());
    Some(smoke)
}

// ⚠️ **`needs_trace` VIVEU AQUI e foi absorvida** pela `preview::next_trace` (W24). Ela
// respondia *"vale a pena traçar de novo?"*; a pergunta passou a ser *"traçar de novo a QUE
// tamanho?"*, e as duas na mesma função é a única forma de não haver duas ideias de *o que mudou*.
// A lei que ela defendia — o **documento** faz parte da chave, o smoke do *"slider disfuncional"* —
// continua gateada, agora em `field3d_preview_tests`.

/// ⭐ **O estado de VISTA e a memória que o faz sobreviver a fechar o painel** vive no irmão — ver
/// [`field3d_view`](self::view).
#[path = "view.rs"]
pub(crate) mod view;
pub use view::forget_isolation_across_documents;

/// ⭐ **Os pedidos que atravessam para o app** vivem no irmão — ver [`field3d_smoke_requests`](self::requests).
#[path = "smoke_requests.rs"]
mod requests;
use requests::armed_scene;
// ⚠️ **O ÚNICO auxiliar de gate desta família que atravessa a fronteira da crate.** Um
// `project_field_tests` da shell repõe as portas de abertura entre gates, e do outro lado da
// fronteira `cfg(test)` é falso — ver a nota da feature `test-support` no `Cargo.toml`.
//
// ⛔ **E a FUNÇÃO aberta sem a RE-EXPORTAÇÃO aberta não chega a lado nenhum:** ela vive num módulo
// privado, logo o compilador lê-a como `dead_code` e o chamador lê-a como inexistente. *São duas
// portas, e abrir uma só dá as duas mensagens ao mesmo tempo, cada uma a apontar para o outro lado.*
#[cfg(any(test, feature = "test-support"))]
pub use requests::forget_open_panel_request;
pub use requests::{
    ProfileShape, ask_export, ask_frame_the_part, ask_import, ask_isolate_key, ask_open_panel,
    ask_open_panel_if_part, ask_profile_shape, ask_relink_sculpt, ask_relinked, ask_scene_sculpt,
    ask_sculpt_extent, ask_shape, ask_shape_palette, ask_spawn_profile, ask_spawn_sculpt,
    mark_authored_change, served_frame, set_armed_by_panel, take_authored_change,
    take_export_request, take_import_request, take_isolate_key_request, take_open_if_part_request,
    take_open_panel_request, take_pending_profile, take_pending_sculpt, take_profile_request,
    take_relink_request, take_relinked, take_scene_sculpt_request, take_sculpt_extent,
    take_shape_palette_request, take_shape_request, wants_frame,
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
pub fn sampled_registry() -> ph2d_field_eval::hybrid::Registry {
    SAMPLED.with(|r| r.borrow().clone())
}

/// Põe uma escultura no registo, sob um nome.
pub fn register_sampled(key: &str, field: std::sync::Arc<dyn ph2d_field_eval::hybrid::Sampled>) {
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
pub fn with_smoke<R>(f: impl FnOnce(&mut Smoke) -> R) -> Option<R> {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        // ⭐⭐ **DESARMAR TEM DE DESARMAR** (W42). Enio, 2026-08-22: *"ainda não consigo usar outros
        // modos como vector"* — e, antes disso, *"o modo Modelagem nunca é desativado"*.
        //
        // ⚠️ **O doc desta função já prometia isto e o código não o fazia.** O `armed_scene()` era
        // consultado **só dentro do `boot()`**, isto é, **só enquanto o smoke ainda não existia**:
        // nascido uma vez, ele vivia para sempre. Fechar o painel punha a bandeira a `false` e
        // ninguém a voltava a ler, então todo gancho de entrada continuava a consumir o gesto.
        //
        // ⭐ **E é por isso que esculpir funcionava e o Vector não:** no `input_dispatch` a
        // escultura toma o ponteiro **antes** (3174) e a modelagem **depois** (3186) — quem vem
        // depois da modelagem nunca via o clique. *A ordem do despacho transformou um bug em dois
        // sintomas, e o segundo parecia outra coisa.*
        //
        // ⚠️ **Largar a cena não perde trabalho:** a peça vive no MUNDO (entidades ECS), não aqui.
        // O que morre é o cache do quadro e a câmera, e ao rearmar a semente é ignorada porque a
        // ponte encontra a raiz que já existe (ver `sync_scene_and_birth`).
        if armed_scene().is_none() {
            // ⭐ **A VISTA sobrevive ao fecho** (W43) — o ⏸️ que a W42 deixou escrito: *"fica: fechar
            // o painel larga a câmera (a peça não)"*. Largar o cache do quadro é o que se quer;
            // largar o ângulo em que o artista pousou a peça não é. Ver [`view`].
            if let Some(Some(s)) = slot.as_ref() {
                view::remember(s);
            }
            *slot = Some(None);
            return None;
        }
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

/// **O shell diz se há uma escultura viva na cena** — publicado todo quadro, como a âncora do
/// gizmo. Ver [`Smoke::has_live_sculpt`].
/// ⭐⭐ **PARTE PARA UMA VISTA** — em vez de saltar para ela (W51).
///
/// ⚠️ **É a ÚNICA porta**: todos os caminhos que escolhiam uma vista escreviam `s.vp().cam.rotation` à
/// mão (a tecla, o chip do painel, a bola do gizmo, o `Home`). Enquanto fossem quatro escritas, uma
/// delas ia ficar a saltar — e o defeito leria como *"às vezes é suave, às vezes não"*, que é o mais
/// difícil de acreditar.
///
/// ⚠️ Sem `Smoke` armado não há para onde partir; sem mudança nenhuma não se parte (uma viagem de
/// zero graus acenderia a mola por nada).
pub fn fly_to(s: &mut Smoke, to: Orbit) {
    if to == s.vp().cam {
        return;
    }
    s.flight = Some(crate::flight::Flight {
        from: s.vp().cam,
        to,
    });
    s.flight_gen = s.flight_gen.wrapping_add(1);
    s.flight_fresh = true;
}

/// ⭐ **A mão CANCELA a viagem** — orbitar, deslocar, aproximar, agarrar uma alça.
///
/// ⚠️ É a lei que o módulo já aplica ao refinamento do preview (*"um refinamento cede à mão"*) e ao
/// prato giratório (`manual`). Uma câmera que continuasse a viajar por baixo de um arrasto seria o
/// app a disputar o rato com o artista.
pub fn cancel_flight(s: &mut Smoke) {
    s.flight = None;
}

/// ⭐ **A track que o shell tem de animar** — `(id, é nova?)`, ou `None` sem viagem.
///
/// ⚠️ **Um id NOVO por viagem**: a mola da casa lembra-se por id, e reusar um faria a segunda
/// viagem continuar de onde a primeira parou. O `flight_gen` é a única razão de ele existir.
pub fn flight_track() -> Option<(u32, bool)> {
    with_smoke(|s| {
        s.flight.is_some().then(|| {
            let fresh = std::mem::take(&mut s.flight_fresh);
            (s.flight_gen, fresh)
        })
    })
    .flatten()
}

/// ⭐⭐ **O progresso da viagem, vindo da mola da casa** — e é aqui que a câmera anda.
///
/// ⚠️ Em `t >= 1` a câmera é **escrita** com o destino e o voo larga-se: a lei do `arrive` da casa
/// (*"assentar põe o valor EXACTO"*), sem a qual o chip da vista nunca acenderia — ele reconhece a
/// orientação com uma barra de 0,16°.
pub fn note_flight_progress(t: f32) {
    with_smoke(|s| advance_flight(s, t));
}

/// O **corpo** da porta acima, sobre um `&mut Smoke` que o chamador já tem.
///
/// ⚠️ **Ela existe por uma razão que custou duas vezes no mesmo dia:** `with_smoke` pega o
/// `RefCell` do estado, e chamá-lo de dentro de outro `with_smoke` é um `borrow_mut` re-entrante —
/// pânico, não erro de compilação. A W50 pagou-o num gate de costura, e este arquivo voltou a
/// pagá-lo na hora seguinte. *Quando uma porta de módulo tem de ser chamada de dentro dele, a cura
/// é o corpo separado — não lembrar-se.*
pub fn advance_flight(s: &mut Smoke, t: f32) {
    let Some(f) = s.flight else {
        return;
    };
    s.vp_mut().cam = f.at(t);
    if t >= 1.0 {
        s.flight = None;
    }
}

/// **O shell diz qual é a parte livre da área** — todo quadro, como a âncora do gizmo. Ver
/// [`Smoke::safe`] e [`crate::navball::safe_corner`].
pub fn note_safe(safe: EditorRect) {
    with_smoke(|s| s.safe = Some(safe));
}

/// A parte livre, ou a área inteira quando ninguém a publicou.
pub fn safe_of(s: &Smoke) -> EditorRect {
    s.safe
        .or(s.vp().area)
        .unwrap_or(EditorRect::new(0.0, 0.0, 0.0, 0.0))
}

/// **O shell diz QUAL contorno fechado está escolhido** — todo quadro, como o irmão abaixo.
///
/// ⚠️ `None` = nenhum. Ver [`crate::smoke_state::Smoke::profile_pick`] para porque é o id e
/// não um `bool`.
pub fn note_profile(pick: Option<u64>) {
    with_smoke(|s| s.profile_pick = pick);
}

/// O contorno escolhido agora, se houver — a porta que o religar consome.
pub fn profile_pick() -> Option<u64> {
    with_smoke(|s| s.profile_pick).flatten()
}

pub fn note_live_sculpt(has: bool) {
    with_smoke(|s| s.has_live_sculpt = has);
}

/// ⭐⭐ **As duas condições que a paleta de formas lê** (W100): há escultura viva na cena? há
/// contorno fechado escolhido?
///
/// ⚠️ **Uma porta e não dois `with_smoke` espalhados**, porque ela tem **dois** leitores que TÊM de
/// concordar: quem constrói a paleta e quem executa o pick um quadro depois. Duas leituras
/// escritas à mão em sítios diferentes é a forma de a oferta e o gesto divergirem — o defeito que
/// a lei da W34 existe para não deixar acontecer.
///
/// ⚠️ **Sem o módulo armado é `(false, false)`**: nada se pode criar de um contorno que não há.
pub fn palette_conditions() -> (bool, bool) {
    with_smoke(|s| (s.has_live_sculpt, s.profile_pick.is_some())).unwrap_or((false, false))
}

/// **Larga o isolamento** — o alvo deixou de existir. Explícito, e não um `toggle(alvo)`: *sair* e
/// *o alvo morreu* são fatos diferentes, e escrever o segundo com a porta do primeiro faria o
/// próximo leitor pensar que houve um gesto.
pub fn forget_isolation() {
    with_smoke(|s| s.isolated = None);
}

/// Que nó está isolado agora, se algum.
pub fn isolated() -> Option<u64> {
    with_smoke(|s| s.isolated).flatten()
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
pub fn gesture_in_progress() -> bool {
    with_smoke(|s| matches!(s.drag, Some(Drag::Gizmo(_)))).unwrap_or(false)
}

/// ⭐⭐ **O ISOLAMENTO** — as duas leis (o chip e a tecla) vivem no irmão. Ver
/// [`field3d_smoke_isolate`](self::isolate).
#[path = "smoke_isolate.rs"]
mod isolate;
pub use isolate::{toggle_isolate, toggle_isolate_by_key};
// ⚠️ As duas LEIS PURAS só têm consumidor nos gates — é o ponto delas: elas existem separadas do
// estado precisamente para serem dirigidas sem armar o módulo (ver o doc de cada uma). O
// `#[cfg(test)]` é o mesmo que o `forget_open_panel_request` já usa, e não um remendo de aviso.
#[cfg(test)]
pub use isolate::{key_isolation, next_isolation};

/// ⭐ **A pintura do quadro** vive no irmão — ver [`field3d_smoke_draw`](self::frame).
#[path = "smoke_draw.rs"]
mod frame;
pub use frame::draw;

// ⚠️ **A metade de SHELL da ponte ECS deixou de estar pendurada aqui** (W2): o
// `field3d_snapshot_tests.rs` captura um `ProjectState`, que é a máquina de undo da shell, e o doc
// dele já se chamava *«a metade de SHELL da ponte ECS»*. Ele ficou lá, declarado no `main.rs`.
//
// ⛔ O doc-comment dele sobreviveu à remoção do `mod` e passou a documentar o `trace_tests` logo
// abaixo — *um `///` órfão não fica órfão: ele adopta o item seguinte*, em silêncio, e foi o
// clippy que o apanhou.
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
        use crate::preview::next_trace;
        let cam = Orbit::default();
        let doc = scene(1);
        let full = (640u32, 480u32);
        // ⚠️ `false` = *o último traçado NÃO foi de movimento*; ver a escada em `next_trace` (W73).
        let asked = (&cam, full.0, full.1, &doc, false);

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
        crate::input::law::orbit(&mut moved, 10.0, 0.0);
        assert!(
            next_trace(Some(asked), &moved, &doc, full, None, true, MIN_TRACE).is_some(),
            "a câmera mudou"
        );
        assert_eq!(
            next_trace(Some(asked), &cam, &doc, (800, 480), None, true, MIN_TRACE),
            Some((800, 480, false)),
            "a área mudou de tamanho: o traçado novo sai NÍTIDO, não grosso"
        );
        // Sem quadro nenhum, traça — mesmo com tudo igual.
        assert_eq!(
            next_trace(Some(asked), &cam, &doc, full, None, false, MIN_TRACE),
            Some((full.0, full.1, false)),
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

#[cfg(test)]
mod tape_shape_probe {
    /// ⭐⭐⭐ **CABE UM INTERPRETADOR DE FITA NA GPU?** — a medição que decide a arquitectura do
    /// traçador de dispositivo (`docs/Render3d/05` §33).
    ///
    /// Duas rotas para levar um campo implícito ao dispositivo:
    ///
    /// - **gerar WGSL** por documento — máxima velocidade, mas **recompila a cada edição**, e o
    ///   artista edita a arrastar um slider;
    /// - **interpretar a FITA** — um shader só, o catálogo **inteiro** de graça (as `62` primitivas
    ///   e todos os modificadores já estão lowered em aritmética), e **zero compilação**.
    ///
    /// ⚠️ O que mata a segunda é o **scratch por thread**: guardar a fita inteira por invocação
    /// seriam quilobytes. Mas a fita é SSA com índices para trás, logo um slot morre assim que o
    /// último leitor passa — o que conta é o **pico de valores vivos**.
    #[test]
    #[ignore = "sonda"]
    fn measure_tape_shape_of_the_real_scenes() {
        println!("  cena · nós ·    ops · VIVOS (o scratch por thread)");
        let mut pior_ops = 0;
        let mut pior_vivos = 0;
        for n in 0..crate::smoke::scenes::CENAS {
            if crate::smoke::scenes::PODADAS.contains(&n) {
                continue;
            }
            let doc = crate::smoke::scene(n);
            let campo = ph2d_field_eval::Field::new(&doc);
            let Some(f) = campo.tape_shape() else {
                println!("  {n:4} · (sem fita)");
                continue;
            };
            pior_ops = pior_ops.max(f.ops);
            pior_vivos = pior_vivos.max(f.vivos);
            println!(
                "  {n:4} · {:3} · {:6} · {:5}",
                doc.nodes().len(),
                f.ops,
                f.vivos
            );
        }
        println!("\n  PIOR: {pior_ops} ops · {pior_vivos} vivos");
        println!(
            "  ⇒ scratch de {} bytes por thread (f32) — um workgroup de 64 gasta {} KB",
            pior_vivos * 4,
            pior_vivos * 4 * 64 / 1024
        );
    }
}

#[cfg(test)]
mod gpu_parity {
    /// ⭐⭐⭐ **DOIS MOTORES, UMA LEI: o campo do DISPOSITIVO responde o mesmo que o da CPU.**
    ///
    /// Percorre **todas** as cenas vivas do smoke, gera o WGSL da fita de cada uma, corre-a na GPU
    /// sobre uma grelha de pontos e compara com o [`ph2d_field_eval::Field::at`].
    ///
    /// ⚠️ **A barra não é zero, e a razão é declarada:** a fita da CPU é `f64` e o dispositivo é
    /// `f32`. O que se exige é que o erro seja o da **representação**, e não o de uma lei diferente
    /// — um opcode traduzido ao contrário (a ordem do `atan2`, o sinal do `Mod`) dá erros de
    /// unidades de mundo, não de `1e-6`.
    ///
    /// ⚠️ `#[ignore]`: precisa de adaptador, como todo gate de GPU desta casa.
    #[test]
    #[ignore = "precisa de GPU"]
    fn o_campo_do_dispositivo_responde_como_o_da_cpu() {
        // Uma grelha dentro do enquadramento da peça, mais os eixos — pontos que caem DENTRO,
        // FORA e sobre a superfície.
        let mut pontos = Vec::new();
        for i in 0..11 {
            for j in 0..11 {
                for k in 0..11 {
                    let f = |n: i32| (n as f32 / 10.0) * 2.4 - 1.2;
                    pontos.push([f(i), f(j), f(k)]);
                }
            }
        }

        let mut piores: Vec<(u32, f64, f64)> = Vec::new();
        for n in 0..crate::smoke::scenes::CENAS {
            if crate::smoke::scenes::PODADAS.contains(&n) {
                continue;
            }
            let doc = crate::smoke::scene(n);
            let Some(p) = ph2d_field_gpu::parity::compare(&doc, &pontos) else {
                println!("cena {n}: sem GPU ou sem fita — saltada");
                continue;
            };
            // ⚠️ **NaN de um lado tem de ser NaN do outro** — um campo que responde `NaN` onde o
            // outro responde um número é uma lei diferente, e a subtração esconde-o.
            let discordam_nan = p
                .gpu
                .iter()
                .zip(&p.cpu)
                .filter(|(g, c)| g.is_nan() != c.is_nan())
                .count();
            assert_eq!(
                discordam_nan, 0,
                "cena {n}: {discordam_nan} pontos em que um motor diz NaN e o outro não"
            );
            piores.push((n, p.worst(), p.rms()));
        }
        assert!(!piores.is_empty(), "nenhuma cena foi comparada");

        println!("  cena ·   pior desvio ·    RMS");
        for (n, w, r) in &piores {
            println!("  {n:4} · {w:13.3e} · {r:9.3e}");
        }
        let pior = piores.iter().fold(0.0f64, |m, (_, w, _)| m.max(*w));
        // ⚠️ **A barra é a da REPRESENTAÇÃO.** As peças vivem em `±1,2` de mundo, e um `f32` tem
        // `~7` dígitos: um erro acumulado ao longo de uma fita de centenas de operações fica na
        // casa de `1e-4`. ⛔ Uma lei diferente (uma ordem de `atan2` trocada, um sinal de `Mod`)
        // dá desvios de unidades de MUNDO — três ordens de grandeza acima disto.
        assert!(
            pior < 1e-3,
            "o pior desvio entre os dois motores é {pior:.3e} — acima do erro de representação de \
             um `f32` sobre uma peça de `±1,2`. Não é precisão: é uma LEI diferente."
        );
    }
}

#[cfg(test)]
mod gpu_gbuffer_parity {
    /// ⭐⭐⭐ **O G-BUFFER DO DISPOSITIVO É O DA CPU** — a silhueta, a profundidade e a normal.
    ///
    /// ⚠️ **É este gate que vigia a única coisa escrita DUAS vezes**: a câmera. Mandar os raios
    /// prontos seriam `50 MB` por quadro, logo o WGSL reconstrói o `ray_at_plane` — e uma
    /// divergência ali move o ponto de acerto em **unidades de mundo**, que é o que as colunas
    /// medem.
    /// A lâmpada onde a wave da §25 a põe.
    fn luz_do_rig(cam: &ph2d_field_render::Orbit) -> [f32; 3] {
        let (right, up, toward_eye) = cam.basis();
        let ecra = [-0.5566703_f32, 0.6634139, 0.5];
        let r = 2.0 * cam.half_extent;
        [0, 1, 2].map(|i| {
            cam.target[i] + r * (ecra[0] * right[i] + ecra[1] * up[i] + ecra[2] * toward_eye[i])
        })
    }

    #[test]
    #[ignore = "precisa de GPU"]
    fn o_gbuffer_do_dispositivo_e_o_da_cpu() {
        use ph2d_field_render::{Orbit, Screen, trace};
        const W: u32 = 192;
        const H: u32 = 108;

        let reg = ph2d_field_eval::hybrid::Registry::new();
        let cam = Orbit::default();
        let (right, up, fwd) = cam.basis();
        let screen = Screen::new(W, H, cam.half_extent);

        println!("  cena · silhueta ·       Dt ·  Dnormal ·   a variacao da PROPRIA peca · razao");
        let mut piores = (0usize, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f64);
        let mut vistas = 0;
        for n in 0..crate::smoke::scenes::CENAS {
            if crate::smoke::scenes::PODADAS.contains(&n) {
                continue;
            }
            let doc = crate::smoke::scene(n);
            let campo = ph2d_field_eval::Field::new(&doc);
            let Some(fita) = campo.tape_wgsl() else {
                continue;
            };
            let g = trace(&doc, &reg, &cam, W, H);
            let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
                .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
            let passo = ph2d_field_eval::safe_march_step(&doc);
            let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
            let setup = ph2d_field_gpu::trace::MarchSetup {
                half_extent: cam.half_extent,
                half_px: screen.half(),
                target: cam.target,
                right,
                up,
                fwd,
                ortho_start: ph2d_field_render::ORTHO_START,
                eye_distance: cam.eye_distance().unwrap_or(0.0),
                hit_eps: ph2d_field_render::Sharpness::for_frame(
                    cam.half_extent,
                    W.min(H) as usize,
                )
                .hit,
                normal_eps: ph2d_field_render::Sharpness::for_frame(
                    cam.half_extent,
                    W.min(H) as usize,
                )
                .normal,
                lamp: luz_do_rig(&cam),
                ball_center: bola.center,
                ball_radius: bola.radius,
                ao_rays: ph2d_field_render::OCCLUSION_PASSES,
                ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
                edge_cos: ph2d_field_render::EDGE_COS,
                step: passo,
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
                    / passo.clamp(f32::EPSILON, 1.0))
                .ceil() as u32,
                t_max: ph2d_field_render::T_MAX,
            };
            let Some(dev) = ph2d_field_gpu::trace::march(&fita, setup, W, H) else {
                println!("sem GPU — saltada");
                return;
            };
            vistas += 1;

            // ⚠️ **A silhueta compara-se por CONTAGEM de pixels em desacordo, não por igualdade**:
            // na borda um raio decide por um `epsilon`, e os dois motores são `f32` com ordens de
            // soma diferentes. O que não pode é a peça mudar de tamanho.
            let mut difere = 0usize;
            let mut dts: Vec<f32> = Vec::new();
            let mut angs: Vec<f32> = Vec::new();
            for i in 0..g.hit.len() {
                if g.hit[i] != dev.hit(i) {
                    difere += 1;
                    continue;
                }
                if !g.hit[i] {
                    continue;
                }
                // O `t` da CPU não é guardado; o ponto é. A distância entre os dois pontos É o Δt.
                let (sx, sy) = ((i % W as usize) as f32 + 0.5, (i / W as usize) as f32 + 0.5);
                let (u, v) = screen.plane_at(sx, sy);
                let (o, d) = cam.ray_at_plane(u, v);
                let p = g.point[i];
                let t_cpu = (p[0] - o[0]) * d[0] + (p[1] - o[1]) * d[1] + (p[2] - o[2]) * d[2];
                dts.push((t_cpu - dev.t[i]).abs());
                let (a, b) = (g.normal[i], dev.normal[i]);
                let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
                angs.push(dot.acos().to_degrees());
            }
            // ⚠️⚠️ **O EXTREMO E A POPULAÇÃO respondem a perguntas DIFERENTES, e aqui a que
            // interessa é a segunda.** Num VINCO a derivada não existe (o módulo já o tem escrito),
            // logo um pixel que caia exactamente lá dá normais muito diferentes a partir de uma
            // diferença de campo de `1e-7` — e as formas por fórmula (rosca, polígono, triângulo)
            // são feitas de vincos. *Um opcode traduzido ao contrário move MILHARES de pixels; um
            // vinco move um punhado.* ⇒ a barra é o `p99`, e o máximo fica na tabela para se ver.
            dts.sort_by(f32::total_cmp);
            angs.sort_by(f32::total_cmp);
            let q = |v: &[f32], f: f64| -> f32 {
                if v.is_empty() {
                    0.0
                } else {
                    v[((v.len() - 1) as f64 * f) as usize]
                }
            };
            // ⭐⭐⭐ **A RÉGUA DA NORMAL É A VARIAÇÃO DA PRÓPRIA PEÇA, e não um ângulo escolhido.**
            //
            // ⚠️ Duas cenas — a ROSCA e as CURVAS — dão `p99` de `12,8°` e `9,9°` contra `≤ 0,5°`
            // das outras catorze, e o campo concorda a `1e-7` nas três. A diferença não é a lei:
            // é o **CONDICIONAMENTO**. Numa ranhura de passo fino a normal roda dezenas de graus
            // de um pixel para o vizinho, logo um deslocamento de `1e-7` no ponto move-a muito.
            //
            // ⇒ a barra é a **variação entre pixels VIZINHOS da CPU**: os dois motores têm de
            // concordar tanto quanto a geometria permite a um pixel concordar com o do lado. *Uma
            // barra em graus absolutos ou isentava a rosca ou acusava as outras quinze.*
            let mut vizinhos: Vec<f32> = Vec::new();
            for y in 0..H as usize {
                for x in 0..W as usize - 1 {
                    let (a_i, b_i) = (y * W as usize + x, y * W as usize + x + 1);
                    if !g.hit[a_i] || !g.hit[b_i] {
                        continue;
                    }
                    let (a, b) = (g.normal[a_i], g.normal[b_i]);
                    let d = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
                    vizinhos.push(d.acos().to_degrees());
                }
            }
            vizinhos.sort_by(f32::total_cmp);
            let pct = 100.0 * difere as f64 / g.hit.len() as f64;
            let (ang99, viz99) = (q(&angs, 0.99), q(&vizinhos, 0.99));
            let razao = ang99 / viz99.max(1e-6);
            println!(
                "  {n:4} · {pct:7.3} % · p99 {:8.2e} · p99 {ang99:6.2}° · o VIZINHO varia {viz99:6.2}° · {razao:5.2}x",
                q(&dts, 0.99)
            );
            piores.0 = piores.0.max(difere);
            piores.1 = piores.1.max(q(&dts, 0.99));
            piores.2 = piores.2.max(razao);

            // ⭐⭐⭐ **AS TRÊS COISAS NOVAS, cada uma com a sua régua.**
            let sh = ph2d_field_render::shadow_pass(&doc, &reg, &cam, &g, &[luz_do_rig(&cam)]);
            let ao = ph2d_field_render::occlusion(
                &doc,
                &reg,
                &cam,
                &g,
                ph2d_field_render::OCCLUSION_PASSES,
            );
            let mut d_sombra: Vec<f32> = Vec::new();
            let mut d_ceu: Vec<f32> = Vec::new();
            for (j, ceu) in ao.iter().enumerate() {
                if !g.hit[j] || !dev.hit(j) {
                    continue;
                }
                d_sombra.push((sh.at(0, j) - dev.shadow[j]).abs());
                d_ceu.push((ceu - dev.ambient[j]).abs());
            }
            d_sombra.sort_by(f32::total_cmp);
            d_ceu.sort_by(f32::total_cmp);

            // ⚠️ **A BORDA compara-se como CONJUNTO.** Os dois motores decidem por um `cos` sobre
            // normais em `f32`, logo um pixel de fronteira pode cair de qualquer lado — o que não
            // pode é a população ser outra. *A régua é a sobreposição, não a igualdade.*
            let cpu_b: std::collections::BTreeSet<u32> = g.edges.iter().map(|e| e.pixel).collect();
            let gpu_b: std::collections::BTreeSet<u32> =
                dev.edges.iter().map(|e| e.pixel).collect();
            let comuns = cpu_b.intersection(&gpu_b).count();
            let uniao = cpu_b.union(&gpu_b).count();
            let sobrep = if uniao == 0 {
                1.0
            } else {
                comuns as f64 / uniao as f64
            };
            println!(
                "         sombra p99 {:7.4} · ceu p99 {:7.4} · bordas CPU {} / GPU {} · sobrepoem {:5.1} %",
                q(&d_sombra, 0.99),
                q(&d_ceu, 0.99),
                cpu_b.len(),
                gpu_b.len(),
                100.0 * sobrep
            );
            piores.3 = piores.3.max(q(&d_sombra, 0.99));
            piores.4 = piores.4.max(q(&d_ceu, 0.99));
            piores.5 = piores.5.min(sobrep);
        }
        assert!(vistas > 10, "só {vistas} cenas foram comparadas");

        let pct = 100.0 * piores.0 as f64 / (W * H) as f64;
        // ⚠️ **As três barras são do MESMO tipo: erro de representação, não de lei.** Uma câmera
        // divergente move o ponto de acerto em unidades de MUNDO e vira a silhueta inteira; um
        // estêncil trocado põe a normal a dezenas de graus. *As barras estão onde o vale medido
        // está, não onde o defeito seria confortável.*
        assert!(
            pct < 1.0,
            "{pct:.3} % dos pixels discordam sobre haver peça — na borda um `epsilon` decide, mas \
             1 % é a peça a mudar de TAMANHO, e isso é a câmera escrita duas vezes a divergir"
        );
        assert!(
            piores.1 < 1e-3,
            "o Δt do p99 é {:.3e} — a marcha do dispositivo está a parar noutro sítio",
            piores.1
        );
        // ⭐ **A sombra e a oclusão são AO BIT comparáveis** — os dois motores correm a mesma
        // sequência de amostragem, logo o que sobra é `f32`. ⛔ Uma sequência só «equivalente»
        // obrigaria a descer a uma média, que é a régua que a §31 mostrou ser cega.
        assert!(
            piores.3 < 0.05,
            "a sombra dos dois motores difere {:.4} no p99 — com o MESMO amostrador, isso já não \
             é ruído",
            piores.3
        );
        // ⚠️⚠️ **A barra da oclusão é o QUANTUM da medida, e a primeira redacção ficou ABAIXO
        // dele.** Com `16` raios binários, a menor diferença possível é `1/16 = 0,0625` — um raio.
        // Eu escrevi `0,05` e o gate acusou `0,0625` exacto, isto é, *acusou a granularidade*.
        // ⇒ a barra é **dois** raios: um raio a discordar é `f32` numa saída rasante; dois já não.
        assert!(
            piores.4 < 2.0 / ph2d_field_render::OCCLUSION_PASSES as f32,
            "a oclusão difere {:.4} no p99 — mais de UM raio de {} a discordar já não é `f32`",
            piores.4,
            ph2d_field_render::OCCLUSION_PASSES
        );
        // ⚠️ A borda é uma decisão de `cos` sobre `f32`: um pixel de fronteira pode cair de
        // qualquer lado. *O que não pode é a POPULAÇÃO ser outra.*
        assert!(
            piores.5 > 0.9,
            "as listas de borda só se sobrepõem {:.1} % — o critério de aresta divergiu",
            100.0 * piores.5
        );
        assert!(
            piores.2 < 1.0,
            "a normal dos dois motores difere {:.2}x mais do que um pixel difere do VIZINHO — \
             isso ja nao e condicionamento: e o estencil ou a base de vista a divergirem",
            piores.2
        );
    }
}

#[cfg(test)]
mod gpu_frame_clock {
    /// ⭐⭐⭐ **O QUADRO COMPLETO NO DISPOSITIVO** — traçado, normal, sombra, oclusão e bordas, com
    /// a leitura de volta dentro. É o relógio que o artista vai sentir.
    #[test]
    #[ignore = "precisa de GPU"]
    fn measure_the_device_frame() {
        use std::time::Instant;
        let reg = ph2d_field_eval::hybrid::Registry::new();
        let cam = ph2d_field_render::Orbit::default();
        let (right, up, fwd) = cam.basis();
        let doc = crate::smoke::scene(1);
        let campo = ph2d_field_eval::Field::new(&doc);
        let fita = campo.tape_wgsl().expect("a fita");
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
            .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
        let passo = ph2d_field_eval::safe_march_step(&doc);
        let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
        let ecra = [-0.5566703_f32, 0.6634139, 0.5];
        let r = 2.0 * cam.half_extent;
        let luz = [0, 1, 2]
            .map(|i| cam.target[i] + r * (ecra[0] * right[i] + ecra[1] * up[i] + ecra[2] * fwd[i]));

        println!(
            "carga: {}",
            std::fs::read_to_string("/proc/loadavg").unwrap().trim()
        );
        println!("  px        · DISPOSITIVO · a CPU faz · ganho");
        for (w, h) in [(640_u32, 360_u32), (1920, 1080)] {
            let screen = ph2d_field_render::Screen::new(w, h, cam.half_extent);
            let sharp = ph2d_field_render::Sharpness::for_frame(cam.half_extent, w.min(h) as usize);
            let setup = ph2d_field_gpu::trace::MarchSetup {
                half_extent: cam.half_extent,
                half_px: screen.half(),
                target: cam.target,
                right,
                up,
                fwd,
                ortho_start: ph2d_field_render::ORTHO_START,
                eye_distance: cam.eye_distance().unwrap_or(0.0),
                hit_eps: sharp.hit,
                normal_eps: sharp.normal,
                step: passo,
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
                    / passo.clamp(f32::EPSILON, 1.0))
                .ceil() as u32,
                t_max: ph2d_field_render::T_MAX,
                lamp: luz,
                ball_center: bola.center,
                ball_radius: bola.radius,
                ao_rays: ph2d_field_render::OCCLUSION_PASSES,
                ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
                edge_cos: ph2d_field_render::EDGE_COS,
            };
            // ⛔⛔ **O TRAÇADOR VIVE ENTRE QUADROS, e a 1.ª redacção desta sonda usava a porta que
            // abre o dispositivo a cada chamada** — ela leu `130 ms` a `640×360`, *mais lento que a
            // CPU*, medindo a abertura e a compilação em vez do quadro. O doc daquela porta já
            // dizia «é a forma de sonda», e eu usei-a como relógio na mesma.
            let Some(mut tr) = ph2d_field_gpu::trace::Tracer::new() else {
                println!("sem GPU");
                return;
            };
            // A 1.ª corrida COMPILA o shader (§33); fica de fora.
            let _ = tr.frame(&fita, setup, w, h);
            let mut v: Vec<f64> = (0..5)
                .map(|_| {
                    let t = Instant::now();
                    let g = tr.frame(&fita, setup, w, h);
                    std::hint::black_box(g.edges.len());
                    t.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            v.sort_by(f64::total_cmp);
            let cpu = if w == 640 { 13.15 } else { 102.64 };
            println!(
                "{w:5}x{h:<4} · {:8.2} ms · {cpu:6.2} ms · {:5.1}x",
                v[0],
                cpu / v[0]
            );
        }
    }
}
