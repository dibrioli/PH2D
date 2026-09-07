//! **A PARTIDA DO FOCO como PORTA** — os gates do [`super::super::blur`].
//!
//! ⚠️ O defeito que a criou não vive aqui, vive na shell (um consumidor de canvas que devolve antes
//! do despachante e deixa o foco preso num chip numérico para o resto da sessão — `Delete` e
//! `Ctrl+Z` morrem com ele). O que estes gates prendem é a metade que é **desta** crate: *a porta
//! faz o MESMO que o clique em espaço morto faria*, senão o segundo chamador tem a sua própria lei.

use super::*;

/// Um chip numérico focado com um buffer **por confirmar**.
fn focused_chip_with_pending_buffer() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(41),
        InteractiveState::NumberInput {
            state: crate::widget::TextInputState::Focused,
            value: 1.0,
            buffer: "2.5".to_string(),
            caret: 3,
            last_committed: 1.0,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(NodeId(41)));
    store
}

/// ⭐⭐ **A porta COMPROMETE o número, apaga o cursor de texto e larga o teclado.**
///
/// As três metades num gate só, e de propósito: elas são o **mesmo acto** (a partida do foco), e
/// separá-las convidaria a uma cura que faz uma e esquece as outras — que é literalmente o defeito
/// que esta porta existe para não repetir noutro sítio.
#[test]
fn a_partida_do_foco_compromete_o_numero_e_larga_o_teclado() {
    let mut store = focused_chip_with_pending_buffer();
    let arena = Bump::new();
    let evts = crate::interaction::blur_focus(&mut store, &arena);

    assert_eq!(store.focus_id(), None, "o foco ficou preso no chip");
    let (v, st) = match store.get(NodeId(41)) {
        Some(InteractiveState::NumberInput { value, state, .. }) => (*value, *state),
        _ => panic!("o chip deixou de ser um NumberInput"),
    };
    assert!(
        (v - 2.5).abs() < f64::EPSILON,
        "o numero digitado evaporou em vez de ser comprometido: {v}"
    );
    assert_eq!(
        st,
        crate::widget::TextInputState::Normal,
        "o campo continua a desenhar-se focado sem ter o teclado"
    );
    assert!(
        evts.iter()
            .any(|e| matches!(e, WidgetEvent::Blur(id) if *id == NodeId(41))),
        "ninguem foi avisado da partida: {evts:?}"
    );
    assert!(
        evts.iter()
            .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(41))),
        "o valor mudou e o painel nao soube: {evts:?}"
    );
}

/// ⚠️ **IDEMPOTENTE** — é isto que deixa a shell chamá-la à frente de um consumidor que talvez não
/// consuma: sem foco não há partida, e uma segunda chamada não emite nada.
///
/// ⛔ Sem esta propriedade a cura da shell duplicaria eventos em todo clique de canvas que **não**
/// fosse consumido (o despachante corre a seguir), e um `ValueChanged` a dobrar num chip com slider
/// ligado é uma escrita a mais no documento.
#[test]
fn sem_foco_a_porta_e_um_no_op_exacto_e_repeti_la_tambem() {
    let mut store = focused_chip_with_pending_buffer();
    let arena = Bump::new();
    let primeira = crate::interaction::blur_focus(&mut store, &arena).len();
    let segunda = crate::interaction::blur_focus(&mut store, &arena).len();
    assert!(primeira > 0, "a primeira partida nao emitiu nada");
    assert_eq!(segunda, 0, "a segunda partida emitiu {segunda} eventos");

    let mut vazio = WidgetStore::with_capacity(1);
    let arena2 = Bump::new();
    assert_eq!(
        crate::interaction::blur_focus(&mut vazio, &arena2).len(),
        0,
        "uma loja sem foco nenhum emitiu eventos"
    );
}

/// ⭐ **O CLIQUE EM ESPAÇO MORTO CONTINUA A LARGAR O FOCO** — o outro chamador da porta.
///
/// ⚠️ Este gate existe porque a cura MOVEU aquele bloco do `dispatch_down` para dentro da porta. Um
/// refactor que deixasse a porta certa e o despachante a não a chamar passaria em tudo o resto:
/// *duas leis idênticas até ao dia em que uma delas deixa de correr.*
#[test]
fn o_clique_em_espaco_morto_continua_a_largar_o_foco() {
    let mut store = focused_chip_with_pending_buffer();
    let hits = HitIndex::default();
    let arena = Bump::new();
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 5.0, 5.0),
        &arena,
    );
    assert_eq!(
        store.focus_id(),
        None,
        "o despachante deixou de largar o foco em espaco morto"
    );
    assert!(
        evts.iter()
            .any(|e| matches!(e, WidgetEvent::Blur(id) if *id == NodeId(41))),
        "o despachante nao emitiu o Blur: {evts:?}"
    );
}
