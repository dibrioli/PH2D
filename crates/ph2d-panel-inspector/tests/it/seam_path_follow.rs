//! **Varredura de SEAM da secção PATH FOLLOW** (suplente #23).
//!
//! Irmã do [`super::seam_tween`] e com a MESMA disciplina: uma **varredura**, não uma amostra, e
//! todo gesto passa pelo `click_at` REAL. *Um `WidgetEvent` sintético salta a verificação de
//! focalizabilidade do store, logo um chip deixado de fora do `populate` fica pintado,
//! hit-registado e **morto sob o dedo**, com um teste verde ao lado.*
//!
//! # ⚠️⚠️ E há DUAS espécies de controlo aqui, com DOIS eventos
//!
//! Um chip emite `Click`; uma **caixa de marcar** emite `Toggled`. É a lição que a W9 do tween
//! pagou nesta mesma linha: os dois interruptores do relógio nasceram no ramo do clique — pintados,
//! hit-registados, vivos sob o dedo — e com o braço do despacho **inalcançável**, com a suíte
//! inteira verde. ⇒ este ficheiro varre as duas espécies, cada uma com o evento dela.
//!
//! # ⭐ E as TRÊS caixas não escrevem no mesmo sítio
//!
//! O `Repeat` e o `Autostart` saem por `InspectorTimerEdit` (o relógio é do `Timers`, e a porta tem
//! três chamadores); o `Face Path` sai por `InspectorComponentEdit`. *Um gate que só olhasse para
//! uma das duas portas leria metade dos controlos como mortos.*

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::path_follow_edits::{InspectorPathFollowInfo, PathFollowFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{TimerFieldEdit, ids as core_ids};
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, ids, set_current_inspector_path_follow,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x7CEE_0023;
/// ⚠️ **O relógio NÃO é o `0`** — com o índice zero, uma edição que mandasse sempre `0` passaria
/// despercebida, que é a mesma armadilha da fixtura de um elemento (paga três vezes no #22).
const SLOT: u8 = 2;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// Um seguidor SÃO, e a fixtura declara-o: **com nome, com forma e com relógio**. Sem isso a queixa
/// toma a primeira linha e o bloco do relógio nem é pintado.
fn info() -> InspectorPathFollowInfo {
    InspectorPathFollowInfo {
        entity_bits: ENTITY,
        caminho: "Trilho".into(),
        nome_existe: true,
        nome_tem_forma: true,
        relogio: SLOT,
        relogios: 4,
        duracao_us: Some(400_000),
        repeat: false,
        autostart: false,
        ciclo: 0,
        familia: 0,
        modo: 0,
        ao_acabar: 0,
        deslocamento: 0.0,
        alinha: true,
        angulo: 0.0,
        lado: 0.0,
        clock_playing: true,
        selected_count: 1,
    }
}

/// O gesto REAL, com a prova de que ele produziu o evento que aquele widget deve produzir.
fn dispara(
    id: ph2d_a11y::NodeId,
    esperado: impl Fn(&WidgetEvent, ph2d_a11y::NodeId) -> bool,
) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_path_follow(Some(info()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("a seccao PATH FOLLOW nunca pintou o controlo {id:?}"));
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        events.iter().any(|e| esperado(e, id)),
        "clicar no meio de {id:?} produziu {events:?} — ele e' pintado e hit-registado, mas nao \
         produziu o evento que o despacho espera: esta' MORTO SOB O DEDO"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_path_follow(None);
    out
}

fn clica(id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    dispara(
        id,
        |e, alvo| matches!(e, WidgetEvent::Click(c) if *c == alvo),
    )
}

fn marca(id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    dispara(
        id,
        |e, alvo| matches!(e, WidgetEvent::Toggled(c) if *c == alvo),
    )
}

/// O que a edição de componente que chegou ao barramento diz, ou o porquê de não ter chegado.
fn edicao(acoes: Vec<EditorAction>, id: ph2d_a11y::NodeId) -> PathFollowFieldEdit {
    acoes
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::PathFollow(e),
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

/// ⭐⭐⭐ **TODO chip de TODA família chega ao barramento com a SUA tag.**
///
/// ⚠️ **A tag é a POSIÇÃO no array**: um despacho que mandasse sempre `0` deixaria todo chip *vivo*
/// e todo clique a escrever a **primeira** opção — e o painel leria-se como *«o botão não faz
/// nada»* para três das quatro famílias.
///
/// **Mutações que devem sangrar:** tirar um array do `populate` · tirar um braço do despacho ·
/// mandar `0` em vez de `n`.
#[test]
fn todo_chip_da_seccao_path_follow_chega_ao_barramento_com_a_sua_tag() {
    #[allow(clippy::type_complexity)]
    let familias: [(
        &str,
        &[ph2d_a11y::NodeId],
        &dyn Fn(u8) -> PathFollowFieldEdit,
    ); 4] = [
        ("ciclo", &ids::INSP_PF_CICLO, &PathFollowFieldEdit::Ciclo),
        (
            "fim",
            &ids::INSP_PF_AO_ACABAR,
            &PathFollowFieldEdit::AoAcabar,
        ),
        (
            "curva",
            &ids::INSP_PF_FAMILIA,
            &PathFollowFieldEdit::Familia,
        ),
        ("ease", &ids::INSP_PF_MODO, &PathFollowFieldEdit::Modo),
    ];
    // ⛔ Piso de população: uma família vazia satisfaz o laço em silêncio.
    for (nome, ids_, _) in &familias {
        assert!(!ids_.is_empty(), "a familia «{nome}» nao tem chip nenhum");
    }
    for (nome, ids_, faz) in familias {
        for (n, id) in ids_.iter().enumerate() {
            let tag = u8::try_from(n).expect("as familias desta seccao cabem num u8");
            assert_eq!(
                edicao(clica(*id), *id),
                faz(tag),
                "o chip {n} da familia «{nome}» escreveu outra coisa"
            );
        }
    }
}

/// ⭐⭐⭐ **A caixa `Face Path` emite `Toggled` e escreve o INVERTIDO do que está no ecrã.**
///
/// ⚠️ **No ramo do CLIQUE ela ficaria viva sob o dedo e com o braço inalcançável** — a lição da W9,
/// e a razão de este gate existir separado dos chips.
#[test]
fn a_caixa_do_alinhar_emite_toggled_e_inverte_o_que_esta_no_ecra() {
    let id = ids::INSP_PF_ALINHA;
    assert_eq!(
        edicao(marca(id), id),
        // O snapshot diz `alinha: true` ⇒ a edição é o contrário.
        PathFollowFieldEdit::Alinha(false)
    );
}

/// ⭐⭐⭐ **As DUAS caixas do relógio saem pela porta dos TIMERS, com o ÍNDICE que o seguidor
/// declara.**
///
/// ⚠️⚠️ **O índice é o discriminador**, e a fixtura usa `2` de propósito: com `0` uma edição que
/// mandasse sempre o primeiro relógio passaria — a armadilha da fixtura de um elemento, paga três
/// vezes no suplente #22.
#[test]
fn as_caixas_do_relogio_escrevem_no_timer_do_indice_do_seguidor() {
    for (id, esperado) in [
        (
            ids::INSP_PF_REPEAT,
            TimerFieldEdit::Repeat(SLOT, true), // o snapshot diz `false`
        ),
        (
            ids::INSP_PF_AUTOSTART,
            TimerFieldEdit::Autostart(SLOT, true),
        ),
    ] {
        let achou = marca(id).into_iter().find_map(|a| match a {
            EditorAction::InspectorTimerEdit { entity_bits, edit } if entity_bits == ENTITY => {
                Some(edit)
            }
            _ => None,
        });
        assert_eq!(
            achou,
            Some(esperado),
            "a caixa {id:?} nao chegou a porta dos TIMERS com o indice {SLOT}"
        );
    }
}

/// ⭐⭐ **A secção TEM cabeçalho dobrável, e ele está na tabela das secções vivas.**
///
/// ⛔ Quem falta naquela tabela pinta o chevron e **a dobra não pode acontecer** — o defeito que a
/// FACTORY e a LIFECYCLE shiparam em 2026-09-14.
#[test]
fn a_seccao_tem_cabecalho_dobravel_na_tabela_das_vivas() {
    assert!(
        core_ids::LIVE_SECTION_IDS.contains(&core_ids::INSP_LIVE_PATHFOLLOW_SECTION),
        "o cabecalho da seccao PATH FOLLOW nao esta' na LIVE_SECTIONS"
    );
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_path_follow(Some(info()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let pintou = rects
        .iter()
        .any(|(n, _)| *n == core_ids::INSP_LIVE_PATHFOLLOW_SECTION);
    set_current_inspector_path_follow(None);
    assert!(pintou, "o cabecalho da seccao nunca foi pintado");
}

/// ⭐⭐ **O censo: toda família de chip da secção está NESTE ficheiro.**
///
/// ⚠️ *Um gate de costura que alguém tem de se lembrar de estender é um gate que envelhece na
/// primeira wave seguinte.* A régua conta os arrays que o `populate` regista, lendo-o.
///
/// **Mutação que deve sangrar:** acrescentar uma família ao `populate` sem a trazer para o gate
/// acima.
#[test]
fn toda_familia_de_chip_da_seccao_esta_varrida() {
    const POPULATE: &str = include_str!("../../src/populate_path_follow.rs");
    /// O que o `populate` regista e **não** é uma família de chip — cada um com o seu gate: o nome
    /// da forma (texto), os cinco números, e as três caixas (duas delas por outra porta).
    const NAO_SAO_CHIPS: [&str; 9] = [
        "CAMINHO",
        "RELOGIO",
        "DURACAO",
        "DESLOCAMENTO",
        "ANGULO",
        "LADO",
        "REPEAT",
        "AUTOSTART",
        "ALINHA",
    ];
    const VARRIDAS: [&str; 4] = ["CICLO", "AO_ACABAR", "FAMILIA", "MODO"];

    let mut achadas: Vec<String> = POPULATE
        .split("ids::INSP_PF_")
        .skip(1)
        .map(|s| {
            s.chars()
                .take_while(|c| c.is_ascii_uppercase() || *c == '_')
                .collect::<String>()
        })
        .filter(|s| !s.is_empty() && !NAO_SAO_CHIPS.contains(&s.as_str()))
        .collect();
    achadas.sort_unstable();
    achadas.dedup();
    // ⛔ **Piso de população:** uma extracção partida devolve zero e o `is_empty` fica verde.
    assert!(
        achadas.len() >= VARRIDAS.len(),
        "a extraccao leu {achadas:?} — ela partiu-se, e um censo que le^ zero aprova tudo"
    );
    for nome in &achadas {
        assert!(
            VARRIDAS.contains(&nome.as_str()),
            "a familia «{nome}» esta' no populate e NAO e' varrida por este ficheiro"
        );
    }
}
