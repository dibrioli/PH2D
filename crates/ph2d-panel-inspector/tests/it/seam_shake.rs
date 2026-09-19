//! **Varredura de SEAM das DUAS secções do ABANÃO** (suplente #25).
//!
//! Irmã do [`super::seam_path_follow`] e com a MESMA disciplina: uma **varredura**, não uma
//! amostra, e todo gesto passa pelo `click_at` REAL. *Um `WidgetEvent` sintético salta a
//! verificação de focalizabilidade do store, logo um chip deixado de fora do `populate` fica
//! pintado, hit-registado e **morto sob o dedo**, com um teste verde ao lado.*
//!
//! # ⚠️⚠️ E aqui a régua tem uma metade a mais: o EXPOENTE não manda o índice
//!
//! Nas irmãs a tag de um chip **é** a posição no array. Aqui não: a faixa da lei começa em
//! [`ph2d_shake::EXPOENTE_MIN`] e o despacho manda `i + MIN`. ⛔ Um gate que só verificasse *«chegou
//! uma edição»* passaria com um despacho que mandasse `0`, que é o valor que a lei **recusa por
//! apagar o trauma** — e o abanão ficaria com a amplitude cheia até morrer num degrau.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::shake_edits::{
    EmitterFieldEdit, InspectorEmitterInfo, InspectorEmitterRow, InspectorShakeInfo, ShakeFieldEdit,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, ids, set_current_inspector_emitter, set_current_inspector_shake,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5AAE_0025;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// ⚠️ **O expoente da fixtura é o MÁXIMO**, e é deliberado: com o de fábrica no meio da faixa, um
/// despacho que mandasse sempre o valor de fábrica passaria em metade dos chips.
fn info_camera() -> InspectorShakeInfo {
    InspectorShakeInfo {
        entity_bits: ENTITY,
        amplitude: 0.25,
        frequencia: 20.0,
        decaimento: 2.0,
        expoente: ph2d_shake::EXPOENTE_MAX,
        semente: 1,
        trauma: 0.0,
        activa: true,
        clock_playing: true,
        selected_count: 1,
    }
}

/// ⚠️ **DUAS fontes, e a aberta é a `1`** — *uma fixtura com um elemento não pode testar um índice*
/// (`get(0)` e `first()` são a mesma coisa), a lição que esta linha pagou três vezes no #22.
fn info_emissor() -> InspectorEmitterInfo {
    let f = |on: &str| InspectorEmitterRow {
        on: on.into(),
        de: 0,
        forca: 0.6,
        dentro: 3.0,
        fora: 12.0,
    };
    InspectorEmitterInfo {
        entity_bits: ENTITY,
        rows: vec![f("primeira"), f("aberta")],
        ha_camera_que_treme: true,
        clock_playing: true,
        selected_count: 1,
    }
}

/// O índice da fonte que o gate abre. Ver [`info_emissor`].
const FONTE_ABERTA: u8 = 1;

/// O gesto REAL, com a prova de que ele produziu o evento que aquele widget deve produzir.
fn clica(id: ph2d_a11y::NodeId, abrir_fonte: bool) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    if abrir_fonte {
        state.emitter_selected = usize::from(FONTE_ABERTA);
    }
    set_current_inspector_shake(Some(info_camera()));
    set_current_inspector_emitter(Some(info_emissor()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("as seccoes do ABANAO nunca pintaram o controlo {id:?}"));
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "clicar no meio de {id:?} produziu {events:?} — ele e' pintado e hit-registado, mas nao \
         produziu o evento que o despacho espera: esta' MORTO SOB O DEDO"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_shake(None);
    set_current_inspector_emitter(None);
    out
}

fn edicao_camera(acoes: Vec<EditorAction>, id: ph2d_a11y::NodeId) -> ShakeFieldEdit {
    acoes
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Shake(e),
            } if entity_bits == ENTITY => Some(e),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "o controlo {id:?} chegou ao store e NAO produziu edicao nenhuma — falta o braco \
                 dele na tabela do despacho, e nenhum outro gate desta seccao o ve^"
            )
        })
}

fn edicao_emissor(acoes: Vec<EditorAction>, id: ph2d_a11y::NodeId) -> EmitterFieldEdit {
    acoes
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::ShakeEmitter(e),
            } if entity_bits == ENTITY => Some(e),
            _ => None,
        })
        .unwrap_or_else(|| panic!("o controlo {id:?} nao produziu edicao de EMISSOR nenhuma"))
}

/// ⭐⭐⭐ **Todo chip do EXPOENTE chega ao barramento com o valor da LEI, e não com o índice.**
///
/// **Mutações que devem sangrar:** tirar o array do `populate` · tirar o braço do despacho · mandar
/// `i` em vez de `i + EXPOENTE_MIN` · mandar sempre o mesmo `n`.
#[test]
fn todo_chip_do_expoente_chega_ao_barramento_com_o_valor_da_lei() {
    assert!(
        !ids::INSP_SHAKE_EXPOENTE.is_empty(),
        "piso de populacao: um array vazio satisfaz o laco em silencio"
    );
    for (i, &id) in ids::INSP_SHAKE_EXPOENTE.iter().enumerate() {
        let esperado = u8::try_from(i).unwrap() + ph2d_shake::EXPOENTE_MIN;
        assert_eq!(
            edicao_camera(clica(id, false), id),
            ShakeFieldEdit::Expoente(esperado),
            "o chip {i} tem de mandar o expoente {esperado}, e nao o indice"
        );
    }
    // ⛔ **E o array cobre a faixa INTEIRA** — um chip a menos é um expoente que o artista nunca
    // escolhe, e que nenhum gate de «o clique chega» vê.
    let faixa = usize::from(ph2d_shake::EXPOENTE_MAX - ph2d_shake::EXPOENTE_MIN) + 1;
    assert_eq!(ids::INSP_SHAKE_EXPOENTE.len(), faixa);
}

/// ⭐⭐⭐ **Todo chip da CERCA chega com a SUA tag, e na fonte ABERTA.**
///
/// ⚠️ **As duas metades:** a tag certa **e** o índice certo. Um despacho que mandasse `0` como
/// índice escreveria sempre na primeira fonte — e com uma fixtura de um elemento isso passaria.
#[test]
fn todo_chip_da_cerca_chega_ao_barramento_na_fonte_aberta() {
    assert!(!ids::INSP_EMITTER_DE.is_empty(), "piso de populacao");
    for (i, &id) in ids::INSP_EMITTER_DE.iter().enumerate() {
        let tag = u8::try_from(i).unwrap();
        assert_eq!(
            edicao_emissor(clica(id, true), id),
            EmitterFieldEdit::De(FONTE_ABERTA, tag),
            "o chip {i} tem de mandar a tag {tag} na fonte {FONTE_ABERTA}"
        );
    }
}

/// ⭐⭐ **Os dois BOTÕES da lista chegam ao barramento** — e o `Remove` com o índice da ABERTA.
#[test]
fn os_botoes_da_lista_do_emissor_chegam_ao_barramento() {
    assert_eq!(
        edicao_emissor(clica(ids::INSP_EMITTER_ADD, true), ids::INSP_EMITTER_ADD),
        EmitterFieldEdit::Add
    );
    assert_eq!(
        edicao_emissor(
            clica(ids::INSP_EMITTER_REMOVE, true),
            ids::INSP_EMITTER_REMOVE
        ),
        EmitterFieldEdit::Remove(FONTE_ABERTA),
        "o `Remove` tem de apagar a fonte ABERTA, nunca a primeira"
    );
}

/// ⭐ **Clicar numa LINHA da lista abre-a, e isso NÃO vai ao barramento** — qual fonte se edita é um
/// facto da UI. ⛔ Uma edição aqui seria um passo de `Ctrl+Z` por clique.
#[test]
fn clicar_numa_linha_abre_a_e_nao_publica_nada() {
    let acoes = clica(ids::INSP_EMITTER_ROW[0], true);
    assert!(
        !acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::ShakeEmitter(_),
                ..
            }
        )),
        "escolher a linha aberta nao pode publicar uma edicao: {acoes:?}"
    );
}

/// ⛔⛔ **Uma linha ALÉM da lista não pode ser alcançável** — o array tem `16` ids e a fixtura tem
/// `2` fontes; clicar no terceiro id seria escolher uma fonte que não existe.
#[test]
fn as_linhas_alem_da_lista_nao_sao_pintadas() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_shake(Some(info_camera()));
    set_current_inspector_emitter(Some(info_emissor()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let pintadas = ids::INSP_EMITTER_ROW
        .iter()
        .filter(|id| rects.iter().any(|(n, _)| n == *id))
        .count();
    set_current_inspector_shake(None);
    set_current_inspector_emitter(None);
    assert_eq!(
        pintadas,
        info_emissor().rows.len(),
        "so' as fontes que existem podem ser pintadas"
    );
}
