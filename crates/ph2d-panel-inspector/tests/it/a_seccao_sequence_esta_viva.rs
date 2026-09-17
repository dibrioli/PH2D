//! ⭐⭐⭐ **A secção SEQUENCE é PINTADA e está VIVA sob o dedo** (TOP-20 #19, W3).
//!
//! # ⚠️ O gesto é REAL, e não um `WidgetEvent::Click`
//!
//! Um `Click` sintético entra por cima da checagem de focabilidade do despachante, logo passa sobre
//! um controlo **pintado, hit-registado e ausente do `populate`** — a família que esta crate já
//! pagou sete vezes, a última delas nas duas amostras de cor do emissor. Aqui o `Down`+`Up` cai no
//! rectângulo que a pintura de facto registou.
//!
//! # ⚠️ E o selector tem TRÊS juntas que ninguém vê da chamada
//!
//! `set_pending_seq_dd` → `take_pending_seq_dd` → `option_rect`. Qualquer uma partida dá o mesmo
//! sintoma — *«abre e não dá para escolher nada»* — e **compila**.
//!
//! # ⛔ E a ISCA é o que torna o gate honesto
//!
//! A cutscene alvo é a **SEGUNDA** do documento. Com uma só, um despacho que devolvesse sempre o
//! índice `0` — ou que lesse a tabela de ids errada — ficaria **inobservável**: é a mesma lição que
//! a fixtura da fase pagou por uma mutação sobrevivente.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::sequence_edits::{InspectorSequenceInfo, SequenceFieldEdit as E};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_sequence};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};
const SEC: u128 = 1_000_000_000;
const BITS: u64 = 0x5E_0DEC;
/// A posição da «Porta» no documento — ⚠️ **não é `0` de propósito** (ver o cabeçalho).
const ALVO: usize = 1;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// O objecto da cena de smoke: toca a «Porta», com um relógio a correr.
fn info() -> InspectorSequenceInfo {
    InspectorSequenceInfo {
        entity_bits: BITS,
        container: "Porta".into(),
        nomes: vec!["Isca".into(), "Porta".into()],
        escolhido: Some(ALVO),
        duracao_da_cutscene: 2.0,
        tem_relogio: true,
        a_correr: true,
        duracao_do_relogio: 2.0,
        t: 0.8,
        clock_playing: true,
        vista_deixa_correr: true,
        selected_count: 1,
    }
}

fn host(i: Option<InspectorSequenceInfo>) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_sequence(i);
    (h, InspectorState::default())
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

/// Um clique real no centro de `id`; devolve as acções que chegaram ao barramento.
fn clica(
    h: &mut MockPanelHost,
    st: &mut InspectorState,
    rects: &[(ph2d_a11y::NodeId, Rect)],
    id: ph2d_a11y::NodeId,
) -> Vec<EditorAction> {
    let r = rect_de(rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre {id:?} não virou evento — pintado e ausente do `populate`"
    );
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(st, ev);
    }
    h.drained_actions()
}

fn edicoes(acoes: &[EditorAction]) -> Vec<E> {
    acoes
        .iter()
        .filter_map(|a| match a {
            EditorAction::InspectorSequenceEdit { entity_bits, edit } => {
                assert_eq!(*entity_bits, BITS);
                Some(edit.clone())
            }
            _ => None,
        })
        .collect()
}

/// ⭐ **O chip é pintado, tem área, e é um `Dropdown` no store.**
///
/// ⚠️ As duas metades: sem área ele nunca é alcançado; sem o registo como `Dropdown` o despachante
/// genérico **não sabe abrir nada** — o clique chegaria e não faria coisa nenhuma.
///
/// **Mutação que deve sangrar:** apagar o `store.register(INSP_SEQ_PICK, …)` do
/// `populate_sequence`.
#[test]
fn o_chip_e_pintado_e_e_um_dropdown() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let r = rect_de(&rects, ids::INSP_SEQ_PICK).expect("o chip da cutscene não foi pintado");
    assert!(r.w > 0.0 && r.h > 0.0, "chip sem área: {r:?}");
    assert_eq!(
        h.dropdown_is_open(ids::INSP_SEQ_PICK),
        Some(false),
        "o chip não está registado como Dropdown — o despachante não o sabe abrir"
    );
    set_current_inspector_sequence(None);
}

/// ⚠️ **Sem o componente não se pinta nada da secção** — ADR-0166.
#[test]
fn sem_o_componente_nada_da_seccao_e_pintado() {
    let (mut h, mut st) = host(None);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rect_de(&rects, ids::INSP_SEQ_PICK).is_none());
    assert!(rect_de(&rects, ids::INSP_SEQ_CLEAR).is_none());
}

/// ⭐⭐ **Aberto, o selector põe CADA cutscene sob o ponteiro — e publica o popover.**
///
/// ⚠️ **Três coisas, porque são três juntas distintas.** As opções só chegam ao índice de acerto se
/// o rect do chip atravessou o slot até ao passe diferido; e o rect do POPOVER só chega ao store se
/// o fim da pintura o publicar — sem isso, clicar fora **não fecha** o selector.
///
/// **Mutações que devem sangrar:** apagar o `set_pending_seq_dd` · apagar o bloco da cutscene no
/// `paint_deferred_popovers`.
#[test]
fn aberto_o_selector_poe_cada_cutscene_sob_o_ponteiro() {
    let (mut h, mut st) = host(Some(info()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_dropdown_open(ids::INSP_SEQ_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);

    let mut vistos: Vec<Rect> = Vec::new();
    for (i, &id) in ids::INSP_SEQ_OPT
        .iter()
        .take(info().nomes.len())
        .enumerate()
    {
        let r = rect_de(&rects, id).unwrap_or_else(|| {
            panic!(
                "a entrada {i} do selector não chegou ao índice de acerto — o popover abre e não \
                 se escolhe nada"
            )
        });
        assert!(r.w > 0.0 && r.h > 0.0, "entrada {i} sem área: {r:?}");
        for (j, a) in vistos.iter().enumerate() {
            assert!(
                r.y >= a.y + a.h || a.y >= r.y + r.h,
                "as entradas {j} e {i} ocupam a mesma banda — uma delas é inalcançável"
            );
        }
        vistos.push(r);
    }
    // ⛔ E as opções ALÉM das cutscenes que existem NÃO são pintadas: uma linha em branco num
    // popover é um clique que não escreve nada e não diz porquê.
    assert!(
        rect_de(&rects, ids::INSP_SEQ_OPT[info().nomes.len()]).is_none(),
        "o selector pintou uma opção para uma cutscene que o documento não tem"
    );

    let (dono, painel) = h
        .store()
        .dropdown_popover()
        .expect("o rect do popover não foi publicado — clicar FORA não fecharia o selector");
    assert_eq!(
        dono,
        ids::INSP_SEQ_PICK,
        "o popover publicado é de outro selector"
    );
    for (i, r) in vistos.iter().enumerate() {
        assert!(
            painel.contains(r.x + r.w * 0.5, r.y + r.h * 0.5),
            "a entrada {i} cai FORA do painel publicado — um clique nela seria lido como «clicou \
             fora»"
        );
    }
    set_current_inspector_sequence(None);
}

/// ⭐⭐⭐ **Escolher uma cutscene CHEGA ao barramento com o NOME dela — e fecha a lista.**
///
/// Este é o gate que mede a costura inteira, com o gesto real: ponteiro → despachante → evento →
/// `apply_event` do painel → `EditorAction`.
///
/// ⚠️ **O discriminador é o NOME, e o alvo é o índice `1`:** um `position` sobre a tabela errada, ou
/// um despacho que devolvesse sempre a primeira, mandaria a `Isca` — e uma fixtura de UMA cutscene
/// não os distinguiria.
///
/// **Mutações que devem sangrar:** apagar o `register_button_ids(store, &INSP_SEQ_OPT)` do
/// `populate_sequence` (o ponteiro deixa de virar evento) · trocar o índice lido no despacho ·
/// apagar o `fecha`.
#[test]
fn escolher_uma_cutscene_chega_ao_barramento_com_o_nome_dela() {
    let (mut h, mut st) = host(Some(info()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_dropdown_open(ids::INSP_SEQ_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);

    let a = clica(&mut h, &mut st, &rects, ids::INSP_SEQ_OPT[ALVO]);
    assert_eq!(edicoes(&a), [E::Container("Porta".into())]);
    assert_eq!(
        h.dropdown_is_open(ids::INSP_SEQ_PICK),
        Some(false),
        "o popover ficou aberto depois da escolha — ele tapa as secções por baixo dele"
    );
    set_current_inspector_sequence(None);
}

/// ⭐⭐ **Largar a cutscene chega ao barramento VAZIA** — o caminho de volta.
///
/// ⚠️ **Com o CONTROLO ao lado:** sem cutscene escolhida o botão **não é pintado**, senão ele seria
/// um controlo que nunca faz nada.
///
/// **Mutação que deve sangrar:** apagar o braço do `INSP_SEQ_CLEAR` no `event_sequence`.
#[test]
fn largar_a_cutscene_chega_ao_barramento_vazia_e_o_botao_so_existe_com_uma() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let a = clica(&mut h, &mut st, &rects, ids::INSP_SEQ_CLEAR);
    assert_eq!(edicoes(&a), [E::Container(String::new())]);

    let vazio = InspectorSequenceInfo {
        container: String::new(),
        escolhido: None,
        ..info()
    };
    let (mut h, mut st) = host(Some(vazio));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        rect_de(&rects, ids::INSP_SEQ_CLEAR).is_none(),
        "sem cutscene escolhida o botão de largar é um controlo que nunca faz nada"
    );
    assert!(
        rect_de(&rects, ids::INSP_SEQ_PICK).is_some(),
        "o selector tem de continuar lá — é por ele que se escolhe a primeira"
    );
    set_current_inspector_sequence(None);
}
