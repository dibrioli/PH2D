//! **O QUE UM FIO PODE ALCANÇAR NUMA FORMA, E O QUE ACONTECE QUANDO A FORMA MUDA.**
//!
//! O report do Enio, 2026-08-27: *"Shape expõe todos os parâmetros de todos os tipos de shape,
//! mas deveria expor apenas os da shape selecionada. Se o usuário trocar de shape e a nova
//! shape não tiver um parâmetro linkado, o link deve ser quebrado."*
//!
//! São **duas** leis, e nenhuma implica a outra:
//! - o menu de largar oferece só o que aquela espécie lê (senão o fio nasce morto);
//! - trocar a espécie SOLTA o fio que ficou órfão (senão ele fica morto depois de nascer vivo).
//!
//! ⚠️ **A régua é o caminho REAL nos dois casos** — a lista que o painel publica
//! (`card_hidden_ports`) e a fila de intenções que a shell drena (`apply_param_edits`), nunca
//! as funções internas que eles chamam: *um gate sobre a função que o executor chama fica
//! verde no dia em que ele deixar de a chamar.*

use super::*;
use crate::motion_state::MotionState;
use ph2d_node_motion_shape::ShapeKind;
use ph2d_panel_motion_params::MotionParamIntent;

/// O `kind` de uma forma, como o painel o escreve (um índice de enum em `f32`).
fn set_kind(m: &mut MotionState, n: ph2d_nodegraph::graph::NodeId, k: ShapeKind) {
    m.doc.graph.set_param(n, "kind", k as i32 as f32);
}

/// Os params que um fio largado no card de `n` pode alcançar, pela porta que o painel lê.
fn offered(m: &MotionState, n: ph2d_nodegraph::graph::NodeId) -> Vec<&'static str> {
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    fold::fold(m, &mut snap);
    ph2d_panel_motion_graph::card_hidden_ports(n.0)
        .inputs
        .iter()
        .filter_map(|p| match p.target {
            ph2d_panel_motion_graph::ChoiceTarget::Param(name) => Some(name),
            ph2d_panel_motion_graph::ChoiceTarget::Port(_) => None,
        })
        .collect()
}

/// **O MENU OFERECE SÓ O QUE AQUELA ESPÉCIE LÊ.**
///
/// O `tooth_depth` é da engrenagem (`ParamGate` do nó). Num **círculo** ele não existe no
/// painel, e até 2026-08-27 o menu de largar oferecia-o na mesma — o fio ligava e conduzia um
/// número que a receita do círculo **nunca lê**.
///
/// ⚠️ **Os dois sentidos, e é o par que faz a lei:** oferecer um knob morto é o defeito
/// reportado; ESCONDER um vivo é o defeito oposto, e é pior — o artista fica sem gesto nenhum
/// para o alcançar. (É a mesma simetria do gate do kernel
/// `no_kind_hides_a_live_knob_or_shows_a_dead_one`.)
#[test]
fn the_drop_menu_offers_only_the_knobs_the_chosen_shape_reads() {
    let mut m = MotionState::new();
    let sh = m.doc.graph.add_node("source.shape");

    set_kind(&mut m, sh, ShapeKind::Gear);
    let gear = offered(&m, sh);
    assert!(
        gear.contains(&"tooth_depth"),
        "a engrenagem TEM dente, e o menu tem de o oferecer; ofereceu {gear:?}"
    );

    set_kind(&mut m, sh, ShapeKind::Circle);
    let circle = offered(&m, sh);
    assert!(
        !circle.contains(&"tooth_depth"),
        "um circulo nao tem dente — o menu nao pode oferecer `tooth_depth`; ofereceu {circle:?}"
    );
    // E o menu não ficou vazio: o `size` é de toda espécie, e continua alcançável.
    assert!(
        circle.contains(&"size"),
        "o `size` vale para toda forma e tem de continuar oferecido"
    );
}

/// **TROCAR A ESPÉCIE SOLTA O FIO QUE FICOU ÓRFÃO** — e a régua é a fila de intenções que a
/// shell drena, o caminho por onde o artista de facto muda a forma.
///
/// ⚠️ **O CONTROLO está no mesmo teste**: o fio no `size` — que toda espécie lê —
/// **sobrevive**. Sem essa metade, um gate que só medisse a queda passaria com uma
/// implementação que solta TUDO ao mexer em qualquer param, que é a destruição de trabalho
/// com cara de cura.
#[test]
fn switching_the_shape_drops_the_wire_the_new_shape_cannot_read() {
    let mut m = MotionState::new();
    let sh = m.doc.graph.add_node("source.shape");
    let num = m.doc.graph.add_node("value.number");
    set_kind(&mut m, sh, ShapeKind::Gear);
    m.doc
        .graph
        .drive_param(sh, "tooth_depth", (num, 0))
        .expect("na engrenagem o dente aceita fio");
    m.doc
        .graph
        .drive_param(sh, "size", (num, 0))
        .expect("o tamanho aceita fio em toda espécie");

    // O gesto: o painel escreve o `kind` pela fila de intenções.
    let mut toasts = ph2d_editor::ToastQueue::default();
    let store = ph2d_editor::interaction::WidgetStore::default();
    ph2d_panel_motion_params::push_param_intent(MotionParamIntent::SetParam {
        node: sh.0,
        param: "kind",
        value: f64::from(ShapeKind::Circle as i32),
    });
    params::apply_param_edits_for_tests(&mut m, &store, &mut toasts);

    let sources = m.doc.graph.param_sources(sh);
    assert!(
        sources.is_none_or(|s| !s.contains_key("tooth_depth")),
        "o circulo nao le o dente — o fio tem de cair"
    );
    assert!(
        sources.is_some_and(|s| s.contains_key("size")),
        "o `size` vale para o circulo tambem — este fio NAO pode cair"
    );
    assert!(
        !toasts.is_empty(),
        "soltar um fio do artista em silencio e' o app a desfazer uma edicao as escondidas"
    );
}

/// **A LEI É UM INVARIANTE, NÃO UM DIFF — logo é IDEMPOTENTE.**
///
/// Correr a reparação de novo, sobre um documento já reparado, não pode soltar mais nada nem
/// falar outra vez. Sem isto, um slider que re-emite a intenção a cada quadro de um arrasto
/// encheria a tela de toasts.
#[test]
fn the_repair_says_nothing_the_second_time() {
    let mut m = MotionState::new();
    let sh = m.doc.graph.add_node("source.shape");
    let num = m.doc.graph.add_node("value.number");
    set_kind(&mut m, sh, ShapeKind::Gear);
    m.doc
        .graph
        .drive_param(sh, "tooth_depth", (num, 0))
        .unwrap();

    let mut toasts = ph2d_editor::ToastQueue::default();
    let store = ph2d_editor::interaction::WidgetStore::default();
    let fire = |m: &mut MotionState, toasts: &mut ph2d_editor::ToastQueue| {
        ph2d_panel_motion_params::push_param_intent(MotionParamIntent::SetParam {
            node: sh.0,
            param: "kind",
            value: f64::from(ShapeKind::Circle as i32),
        });
        params::apply_param_edits_for_tests(m, &store, toasts);
    };
    fire(&mut m, &mut toasts);
    let after_first = toasts.len();
    assert!(after_first > 0, "a primeira vez fala");
    fire(&mut m, &mut toasts);
    assert_eq!(
        toasts.len(),
        after_first,
        "a segunda vez nao tem nada a soltar, logo nada a dizer"
    );
}

/// **UM LIMIAR CONTÍNUO NÃO SOLTA FIO — e esta assimetria é deliberada.**
///
/// ⚠️ O `Trim End` do `source.shape` pende do `Stroke Width` passar de zero
/// (`ParamGateAbove`), e o `Stroke Width` é um **slider**: a mão varre-o para baixo e para
/// cima no mesmo gesto. Soltar o fio na travessia apagaria a ligação do artista num arrasto,
/// e voltar a subir não a repõe — destruição de trabalho disfarçada de arrumação.
///
/// A lei que fica: **o menu esconde pelas três famílias de gate, a queda do fio só pela
/// discreta.** *Um controle que não se vê não se pode usar, logo não se oferece; mas só um
/// controle que deixou de EXISTIR justifica destruir a ligação que ia até ele.*
///
/// ⚠️ **A 1.ª redacção da cura usava as três famílias nos dois sítios**, e este teste é o que
/// a mediu — os seis params do `source.shape` que pendem daquele slider (as duas cores, o
/// tracejado e as duas pontas do Trim) caíam todos ao arrastar o traço até zero.
#[test]
fn sweeping_a_threshold_through_zero_keeps_the_wire() {
    let mut m = MotionState::new();
    let sh = m.doc.graph.add_node("source.shape");
    let num = m.doc.graph.add_node("value.number");
    m.doc.graph.set_param(sh, "stroke_width", 0.2);
    m.doc
        .graph
        .drive_param(sh, "trim_end", (num, 0))
        .expect("com traco, o Trim aceita fio");

    // O menu, por outro lado, DEIXA de o oferecer sem traço — as duas metades da assimetria
    // medidas no mesmo teste, ou uma delas passa a ser opinião.
    m.doc.graph.set_param(sh, "stroke_width", 0.0);
    assert!(
        !offered(&m, sh).contains(&"trim_start"),
        "sem traco o Trim nao se ve', logo nao se oferece"
    );

    // E agora o gesto real: o slider a chegar a zero.
    let mut toasts = ph2d_editor::ToastQueue::default();
    let store = ph2d_editor::interaction::WidgetStore::default();
    ph2d_panel_motion_params::push_param_intent(MotionParamIntent::SetParam {
        node: sh.0,
        param: "stroke_width",
        value: 0.0,
    });
    params::apply_param_edits_for_tests(&mut m, &store, &mut toasts);

    assert!(
        m.doc
            .graph
            .param_sources(sh)
            .is_some_and(|s| s.contains_key("trim_end")),
        "um limiar continuo e' reversivel — o fio TEM de sobreviver"
    );
    assert!(toasts.is_empty(), "e nada aconteceu, logo nada ha' a dizer");
}
