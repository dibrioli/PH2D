//! **Varredura de SEAM da secção TWEEN** (suplente #22, W8).
//!
//! Irmã do [`super::seam_player`] e com a MESMA disciplina: uma **varredura**, não uma amostra, e
//! todo clique passa pelo `click_at` REAL. *Um `WidgetEvent` sintético salta a verificação de
//! focalizabilidade do store, logo um chip deixado de fora do `populate` fica pintado,
//! hit-registado e **morto sob o dedo**, com um teste verde ao lado.*
//!
//! # ⛔⛔ Porque este ficheiro nasceu, e o que ele acusou
//!
//! A secção shipou em 2026-09-19 com **cinco famílias de chip** (canal · curva · ease · fim ·
//! preset) e **zero** gates de costura: os gates dela medem a porta da queixa e o dreno das
//! edições, e os dois entram **abaixo** do store. ⇒ um braço esquecido na tabela do despacho — ou
//! um array esquecido no `populate` — dava um chip que acende, não faz nada, e não reprova nada.
//!
//! É a SÉTIMA ocorrência deste defeito no repo (os treze chips da escultura · as duas amostras do
//! emissor · os quatro chips da booleana do vector · …), e a wave do **ciclo** — que acrescenta a
//! sexta família — foi onde ela deixou de ser barata de ignorar.
//!
//! # ⭐⭐ A régua é a FAMÍLIA, e não uma lista escrita à mão
//!
//! Cada bloco percorre o array de ids **inteiro** e afirma as duas metades: o clique **chega** (o
//! store considera o chip focalizável) e ele **produz a edição certa, com a tag certa**. Uma
//! família nova que não entre aqui deixa o censo do fim reprovar.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tween_edits::{InspectorTweenInfo, InspectorTweenRow, TweenFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_tween};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x7CEE_0022;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// Um tween SÃO, e a fixtura declara-o: **com relógio e com sprite**, senão a queixa toma a
/// primeira linha do editor e empurra tudo o resto para baixo — o que ainda pintaria, mas é uma
/// premissa que vale a pena estar escrita.
fn linha() -> InspectorTweenRow {
    InspectorTweenRow {
        canal: ph2d_tween::Canal::Opacity.tag(),
        de: [1.0, 0.0, 0.0, 0.0],
        para: [0.0, 0.0, 0.0, 0.0],
        familia: 0,
        modo: 0,
        ao_acabar: ph2d_tween::AoAcabar::Hold.tag(),
        ciclo: ph2d_tween::Ciclo::Reinicia.tag(),
        duracao_us: Some(400_000),
    }
}

fn info() -> InspectorTweenInfo {
    InspectorTweenInfo {
        entity_bits: ENTITY,
        rows: vec![linha()],
        tem_sprite: true,
        selected_count: 1,
    }
}

/// Carrega no MEIO do chip e devolve o que foi ao barramento.
fn clica(id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_tween(Some(info()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("a secção TWEEN nunca pintou o chip {id:?}"));
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "clicar no meio de {id:?} produziu {events:?} — o chip e' pintado e hit-registado, mas o \
         store nao o considera focalizavel: ele esta' MORTO SOB O DEDO"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_tween(None);
    out
}

/// O que a edição que chegou ao barramento diz, ou o porquê de não ter chegado.
fn edicao(id: ph2d_a11y::NodeId) -> TweenFieldEdit {
    let acoes = clica(id);
    acoes
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Tween(e),
            } if entity_bits == ENTITY => Some(e),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "o chip {id:?} chegou ao store e NAO produziu edicao nenhuma — falta o braco dele \
                 na tabela do despacho, e nenhum outro gate desta seccao o ve^"
            )
        })
}

/// ⭐⭐⭐ **TODO chip de TODA família chega ao barramento com a SUA tag.**
///
/// ⚠️ **A tag é a POSIÇÃO no array**, e é isso que a segunda metade mede: um despacho que mandasse
/// sempre `0` deixaria todo chip *vivo* e todo clique a escrever a **primeira** opção — e o painel
/// leria-se como *«o botão não faz nada»* para cinco das seis famílias.
///
/// **Mutações que devem sangrar:** tirar um array do `populate` · tirar um braço da tabela do
/// despacho · mandar `0` em vez de `n`.
#[test]
fn todo_chip_da_seccao_tween_chega_ao_barramento_com_a_sua_tag() {
    #[allow(clippy::type_complexity)]
    let familias: [(&str, &[ph2d_a11y::NodeId], &dyn Fn(u8) -> TweenFieldEdit); 6] = [
        ("canal", &ids::INSP_TWEEN_CANAL, &|n| {
            TweenFieldEdit::Canal(0, n)
        }),
        ("curva", &ids::INSP_TWEEN_FAMILIA, &|n| {
            TweenFieldEdit::Familia(0, n)
        }),
        ("ease", &ids::INSP_TWEEN_MODO, &|n| {
            TweenFieldEdit::Modo(0, n)
        }),
        ("fim", &ids::INSP_TWEEN_AO_ACABAR, &|n| {
            TweenFieldEdit::AoAcabar(0, n)
        }),
        ("ciclo", &ids::INSP_TWEEN_CICLO, &|n| {
            TweenFieldEdit::Ciclo(0, n)
        }),
        ("preset", &ids::INSP_TWEEN_PRESET, &|n| {
            TweenFieldEdit::Preset(0, n)
        }),
    ];
    // ⛔ Piso de população: uma família vazia satisfaz o laço em silêncio.
    for (nome, ids_, _) in &familias {
        assert!(!ids_.is_empty(), "a familia «{nome}» nao tem chip nenhum");
    }
    for (nome, ids_, faz) in familias {
        for (n, id) in ids_.iter().enumerate() {
            let tag = u8::try_from(n).expect("as familias desta seccao cabem num u8");
            assert_eq!(
                edicao(*id),
                faz(tag),
                "o chip {n} da familia «{nome}» escreveu outra coisa"
            );
        }
    }
}

/// ⭐⭐ **O censo: toda família de chip da secção está NESTE ficheiro.**
///
/// ⚠️ *Um gate de costura que alguém tem de se lembrar de estender é um gate que envelhece na
/// primeira wave seguinte* — e foi exactamente assim que a secção chegou a ter cinco famílias e
/// zero costura. A régua conta os arrays que o `populate` regista, lendo-o.
///
/// **Mutação que deve sangrar:** acrescentar uma família ao `populate` sem a trazer para o gate
/// acima.
#[test]
fn toda_familia_de_chip_da_seccao_esta_varrida() {
    const POPULATE: &str = include_str!("../../src/populate_tween.rs");
    /// O que o `populate` regista e **não** é uma família de chip — cada um com a sua lei e o seu
    /// gate: a lista, os dois blocos de campos numéricos e os dois botões da lista.
    const NAO_SAO_CHIPS: [&str; 5] = ["ROW", "DE", "PARA", "ADD", "REMOVE"];

    let mut familias: Vec<&str> = POPULATE
        .match_indices("ids::INSP_TWEEN_")
        .map(|(i, m)| {
            let resto = &POPULATE[i + m.len()..];
            let fim = resto
                .find(|c: char| !c.is_ascii_uppercase() && c != '_')
                .unwrap_or(resto.len());
            &resto[..fim]
        })
        .filter(|n| !NAO_SAO_CHIPS.contains(n))
        .collect();
    familias.sort_unstable();
    familias.dedup();

    // ⛔ CONTROLO da extracção: se ela devolvesse vazio, o `assert` de baixo lia-se como um gate
    //    a passar sobre uma secção sem chip nenhum.
    assert!(
        familias.len() >= 5,
        "a extraccao leu {} familias — ela partiu-se: {familias:?}",
        familias.len()
    );
    assert_eq!(
        familias.len(),
        6,
        "o `populate_tween` regista {} familias de chip ({familias:?}) e o gate acima varre 6 — \
         uma familia fora da varredura pode morrer sob o dedo sem nada reprovar",
        familias.len()
    );
}
