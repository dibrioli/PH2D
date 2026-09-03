//! **O smoke do módulo de modelagem 3D** — `PH2D_FIELD_SMOKE=1..17` (ADR-0161).
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

use std::sync::Arc;
use std::sync::mpsc::{TryRecvError, channel};

use ph2d_editor::zones::Rect as EditorRect;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Profile, Xform};
use ph2d_field_render::{Matcap, Orbit, shade};
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
#[path = "field3d_smoke_state.rs"]
mod state;
pub(crate) use state::{Drag, Grip, InFlight, Ready, Smoke};

/// ⭐⭐⭐ **A LISTA DE VIEWPORTS** — quem a abre, quem a fecha e de quem é um ponto (W90). Vive no
/// irmão pela mesma razão do [`state`]: o `field3d_smoke.rs` é a porta do módulo, e o tecto de LOC
/// do HR-18 é o instrumento que impede uma porta de virar um armazém.
#[path = "field3d_viewports.rs"]
mod viewports;
use state::{MatcapTexels, STATE};
pub(crate) use viewports::{
    canvas_area, divider_cursor, ensure_viewports, toggle_split, viewport_at,
};

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
        has_live_sculpt: false,
        matcap: Arc::new(load_matcap()),
        // ⭐ **Um viewport, que é o que o módulo sempre teve** — a divisão entra depois, e este
        // é o estado em que ela não existe.
        vps: vec![crate::field3d_smoke::state::Viewport::new(v.cam, v.manual)],
        active: 0,
        // ⭐ **A divisão volta com a vista** (W95) — os viewports que ela pede são reconstruídos
        // logo a seguir, a partir da câmera lembrada.
        split: v.split,
        announced: false,
        drag: None,
        last_pointer: (0.0, 0.0),
        gizmo: None,
        gizmo_hot: None,
        pending_move: None,
        drag_grip: None,
        snapping: false,
        typed: None,
        press_at: None,
        pending_pick: None,
        lasso: None,
        pending_lasso: None,
        gizmo_mode: v.gizmo_mode,
        gizmo_frame: v.gizmo_frame,
    };
    // ⭐ **A lista nasce já com a divisão lembrada** (W95). Ela seria reconciliada no primeiro
    // desenho de qualquer forma, mas então haveria um quadro em que o `split` diz «quatro» e a
    // lista tem uma — e *um estado que só é verdade a partir do segundo quadro é um estado que
    // alguém vai ler no primeiro*.
    ensure_viewports(&mut smoke, v.split.count());
    Some(smoke)
}

// ⚠️ **`needs_trace` VIVEU AQUI e foi absorvida** pela `field3d_preview::next_trace` (W24). Ela
// respondia *"vale a pena traçar de novo?"*; a pergunta passou a ser *"traçar de novo a QUE
// tamanho?"*, e as duas na mesma função é a única forma de não haver duas ideias de *o que mudou*.
// A lei que ela defendia — o **documento** faz parte da chave, o smoke do *"slider disfuncional"* —
// continua gateada, agora em `field3d_preview_tests`.

/// ⭐ **O estado de VISTA e a memória que o faz sobreviver a fechar o painel** vive no irmão — ver
/// [`field3d_view`](self::view).
#[path = "field3d_view.rs"]
mod view;
pub(crate) use view::forget_isolation_across_documents;

/// ⭐ **Os pedidos que atravessam para o app** vivem no irmão — ver [`field3d_smoke_requests`](self::requests).
#[path = "field3d_smoke_requests.rs"]
mod requests;
use requests::armed_scene;
#[cfg(test)]
pub(crate) use requests::forget_open_panel_request;
pub(crate) use requests::{
    ProfileShape, ask_export, ask_frame_the_part, ask_import, ask_isolate_key, ask_open_panel,
    ask_open_panel_if_part, ask_profile_shape, ask_relink_sculpt, ask_relinked, ask_scene_sculpt,
    ask_sculpt_extent, ask_shape, ask_shape_palette, ask_spawn_profile, ask_spawn_sculpt,
    served_frame, set_armed_by_panel, take_export_request, take_import_request,
    take_isolate_key_request, take_open_if_part_request, take_open_panel_request,
    take_pending_profile, take_pending_sculpt, take_profile_request, take_relink_request,
    take_relinked, take_scene_sculpt_request, take_sculpt_extent, take_shape_palette_request,
    take_shape_request, wants_frame,
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
pub(crate) fn fly_to(s: &mut Smoke, to: Orbit) {
    if to == s.vp().cam {
        return;
    }
    s.flight = Some(crate::field3d_flight::Flight {
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
pub(crate) fn cancel_flight(s: &mut Smoke) {
    s.flight = None;
}

/// ⭐ **A track que o shell tem de animar** — `(id, é nova?)`, ou `None` sem viagem.
///
/// ⚠️ **Um id NOVO por viagem**: a mola da casa lembra-se por id, e reusar um faria a segunda
/// viagem continuar de onde a primeira parou. O `flight_gen` é a única razão de ele existir.
pub(crate) fn flight_track() -> Option<(u32, bool)> {
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
pub(crate) fn note_flight_progress(t: f32) {
    with_smoke(|s| advance_flight(s, t));
}

/// O **corpo** da porta acima, sobre um `&mut Smoke` que o chamador já tem.
///
/// ⚠️ **Ela existe por uma razão que custou duas vezes no mesmo dia:** `with_smoke` pega o
/// `RefCell` do estado, e chamá-lo de dentro de outro `with_smoke` é um `borrow_mut` re-entrante —
/// pânico, não erro de compilação. A W50 pagou-o num gate de costura, e este arquivo voltou a
/// pagá-lo na hora seguinte. *Quando uma porta de módulo tem de ser chamada de dentro dele, a cura
/// é o corpo separado — não lembrar-se.*
pub(crate) fn advance_flight(s: &mut Smoke, t: f32) {
    let Some(f) = s.flight else {
        return;
    };
    s.vp_mut().cam = f.at(t);
    if t >= 1.0 {
        s.flight = None;
    }
}

/// **O shell diz qual é a parte livre da área** — todo quadro, como a âncora do gizmo. Ver
/// [`Smoke::safe`] e [`crate::field3d_navball::safe_corner`].
pub(crate) fn note_safe(safe: EditorRect) {
    with_smoke(|s| s.safe = Some(safe));
}

/// A parte livre, ou a área inteira quando ninguém a publicou.
pub(crate) fn safe_of(s: &Smoke) -> EditorRect {
    s.safe
        .or(s.vp().area)
        .unwrap_or(EditorRect::new(0.0, 0.0, 0.0, 0.0))
}

/// **O shell diz QUAL contorno fechado está escolhido** — todo quadro, como o irmão abaixo.
///
/// ⚠️ `None` = nenhum. Ver [`crate::field3d_smoke_state::Smoke::profile_pick`] para porque é o id e
/// não um `bool`.
pub(crate) fn note_profile(pick: Option<u64>) {
    with_smoke(|s| s.profile_pick = pick);
}

/// O contorno escolhido agora, se houver — a porta que o religar consome.
pub(crate) fn profile_pick() -> Option<u64> {
    with_smoke(|s| s.profile_pick).flatten()
}

pub(crate) fn note_live_sculpt(has: bool) {
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
pub(crate) fn palette_conditions() -> (bool, bool) {
    with_smoke(|s| (s.has_live_sculpt, s.profile_pick.is_some())).unwrap_or((false, false))
}

/// **Larga o isolamento** — o alvo deixou de existir. Explícito, e não um `toggle(alvo)`: *sair* e
/// *o alvo morreu* são fatos diferentes, e escrever o segundo com a porta do primeiro faria o
/// próximo leitor pensar que houve um gesto.
pub(crate) fn forget_isolation() {
    with_smoke(|s| s.isolated = None);
}

/// Que nó está isolado agora, se algum.
pub(crate) fn isolated() -> Option<u64> {
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
pub(crate) fn gesture_in_progress() -> bool {
    with_smoke(|s| matches!(s.drag, Some(Drag::Gizmo(_)))).unwrap_or(false)
}

/// ⭐⭐ **O ISOLAMENTO** — as duas leis (o chip e a tecla) vivem no irmão. Ver
/// [`field3d_smoke_isolate`](self::isolate).
#[path = "field3d_smoke_isolate.rs"]
mod isolate;
pub(crate) use isolate::{toggle_isolate, toggle_isolate_by_key};
// ⚠️ As duas LEIS PURAS só têm consumidor nos gates — é o ponto delas: elas existem separadas do
// estado precisamente para serem dirigidas sem armar o módulo (ver o doc de cada uma). O
// `#[cfg(test)]` é o mesmo que o `forget_open_panel_request` já usa, e não um remendo de aviso.
#[cfg(test)]
pub(crate) use isolate::{key_isolation, next_isolation};

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
        crate::field3d_input::law::orbit(&mut moved, 10.0, 0.0);
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
