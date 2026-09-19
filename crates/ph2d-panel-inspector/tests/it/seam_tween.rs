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
        repeat: true,
        autostart: true,
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
    dispara(
        id,
        |e, alvo| matches!(e, WidgetEvent::Click(c) if *c == alvo),
    )
}

/// O gesto REAL, com a prova de que ele produziu o evento que aquele widget deve produzir.
fn dispara(
    id: ph2d_a11y::NodeId,
    esperado: impl Fn(&WidgetEvent, ph2d_a11y::NodeId) -> bool,
) -> Vec<EditorAction> {
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
        events.iter().any(|e| esperado(e, id)),
        "clicar no meio de {id:?} produziu {events:?} — ele e' pintado e hit-registado, mas nao \
         produziu o evento que o despacho espera: esta' MORTO SOB O DEDO"
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
    /// gate: a lista, os dois blocos de campos numéricos, os dois botões da lista, os TRÊS
    /// controlos do RELÓGIO (que escrevem no `Timers`, por outra porta) e as DUAS amostras de COR
    /// (que abrem o selector, e têm gates próprios neste ficheiro).
    const NAO_SAO_CHIPS: [&str; 10] = [
        "ROW",
        "DE",
        "PARA",
        "ADD",
        "REMOVE",
        "COR_DE",
        "COR_PARA",
        "DURACAO",
        "REPEAT",
        "AUTOSTART",
    ];

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

// ─────────────────────────────────────────────────────────────────────────────
// O RELÓGIO, dentro da secção (W9) — report do dono: *«por que não embutir na própria secção?»*.
// ─────────────────────────────────────────────────────────────────────────────

/// Duas linhas, com o relógio da SEGUNDA **diferente** da primeira.
///
/// ⚠️⚠️ **É isto que torna os gates do relógio testes de ÍNDICE:** *uma fixtura com um elemento não
/// pode testar um índice* — com uma linha só, `i` é `0` e a mutação *«manda o índice `0`»* é um
/// no-op. A W6 desta linha já pagou esta lição com uma mutação SOBREVIVENTE, e a 1.ª redacção
/// deste ficheiro voltou a pagá-la.
fn duas_linhas() -> InspectorTweenInfo {
    let mut segunda = linha();
    segunda.repeat = false;
    segunda.autostart = false;
    segunda.duracao_us = Some(1_500_000);
    InspectorTweenInfo {
        entity_bits: ENTITY,
        rows: vec![linha(), segunda],
        tem_sprite: true,
        selected_count: 1,
    }
}

/// Abre a SEGUNDA linha da lista (pelo clique REAL nela) e devolve o anfitrião pronto.
fn na_segunda_linha() -> (MockPanelHost, InspectorState) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_tween(Some(duas_linhas()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let alvo = ids::INSP_TWEEN_ROW[1];
    let r = rects
        .iter()
        .find(|(n, _)| *n == alvo)
        .map(|(_, r)| *r)
        .expect("a lista nunca pintou a segunda linha");
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let _ = host.drained_actions(); // a escolha da linha NÃO vai ao barramento
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    (host, state)
}

/// ⭐⭐⭐ **OS TRÊS CONTROLOS DO RELÓGIO CHEGAM AO BARRAMENTO — e pela porta dos TIMERS.**
///
/// ⛔⛔ **É isto que prova que não são uma segunda lei:** a edição que sai é uma
/// [`TimerFieldEdit`] no `Timers` do MESMO índice — a mesma que a secção TIMERS emite. Quem satura
/// no `TIMER_MAX_US`, quem recusa e quem grava continua a ser um só.
///
/// ⚠️⚠️ **A fixtura tem DUAS linhas e a aberta é a SEGUNDA** — ver [`duas_linhas`]. Com uma só, a
/// mutação que manda o índice `0` seria um no-op, e o gate afirmaria menos do que promete.
///
/// **Mutações que devem sangrar:** tirar qualquer um dos três do despacho · mandar o índice `0`
/// em vez do slot aberto · ler o estado da caixa do store em vez do instantâneo · pôr as caixas no
/// evento errado (`Click` em vez de `Toggled`).
#[test]
fn os_tres_controlos_do_relogio_chegam_a_porta_dos_timers() {
    use ph2d_editor_core::screens::hero::TimerFieldEdit;

    // As duas CAIXAS: o clique inverte o que o INSTANTÂNEO diz — e a segunda linha tem-nas
    // DESLIGADAS, logo o esperado é `true` (o contrário da primeira, que é o discriminador).
    for (id, esperado, nome) in [
        (
            ids::INSP_TWEEN_REPEAT,
            TimerFieldEdit::Repeat(1, true),
            "repeat",
        ),
        (
            ids::INSP_TWEEN_AUTOSTART,
            TimerFieldEdit::Autostart(1, true),
            "autostart",
        ),
    ] {
        let (mut host, mut state) = na_segunda_linha();
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("a seccao nunca pintou a caixa «{nome}»"));
        let evs = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Toggled(c) if *c == id)),
            "a caixa «{nome}» nao produziu `Toggled`: {evs:?} — ela esta' MORTA SOB O DEDO"
        );
        for ev in evs {
            let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
        }
        let acoes = host.drained_actions();
        set_current_inspector_tween(None);
        assert_eq!(
            acoes,
            [EditorAction::InspectorTimerEdit {
                entity_bits: ENTITY,
                edit: esperado,
            }],
            "a caixa «{nome}» do relogio nao chegou a` porta dos TIMERS com o INDICE da linha aberta"
        );
    }

    // E a DURAÇÃO, que é um campo numérico: ela sai em SEGUNDOS, como a da secção irmã.
    let (mut host, mut state) = na_segunda_linha();
    host.set_number_value(ids::INSP_TWEEN_DURACAO, 2.5);
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::INSP_TWEEN_DURACAO),
    );
    let acoes = host.drained_actions();
    set_current_inspector_tween(None);
    assert_eq!(
        acoes,
        [EditorAction::InspectorTimerEdit {
            entity_bits: ENTITY,
            edit: TimerFieldEdit::DurationSecs(1, 2.5),
        }],
        "a duracao da seccao TWEEN nao chegou a` porta dos TIMERS com o INDICE da linha aberta"
    );
}

/// ⭐⭐ **O campo mostra o número DO OBJECTO, e não o de fábrica** — a quinta vez que esta casa
/// escreve este gate, e a razão é sempre a mesma.
///
/// ⚠️ **A fixtura difere do default do `Timer`** (`0,4 s` contra `1 s`), senão um campo semeado no
/// ponto neutro passaria com a semente apagada — *um corpus no NEUTRO de um knob não testa esse
/// knob*.
///
/// **Mutação que deve sangrar:** tirar o bloco da duração do `sync_tween`.
#[test]
fn o_campo_da_duracao_mostra_o_relogio_do_objecto() {
    let (host, _state) = na_segunda_linha();
    let lido = host
        .store()
        .number_value(ids::INSP_TWEEN_DURACAO)
        .expect("o campo da duracao nem sequer esta' registado");
    set_current_inspector_tween(None);
    // ⚠️ **`1,5 s` é o relógio da SEGUNDA linha** — diferente do da primeira (`0,4`) e do default
    //    do `populate` (`1,0`), logo ele discrimina as três leituras possíveis de uma vez.
    assert!(
        (lido - 1.5).abs() < 1.0e-6,
        "o campo mostra {lido} s e o relogio da linha aberta tem 1,5 s — o painel esta' a mostrar \
         o valor de FABRICA do `populate` (ou o da linha errada)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// A COR (W10) — report do dono: *«por que usar cores em números se temos caixas selectoras?»*.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma linha cujo canal é uma COR — é ela que faz a secção pintar amostras em vez de campos.
fn linha_de_cor() -> InspectorTweenInfo {
    let mut r = linha();
    r.canal = ph2d_tween::Canal::Tint.tag();
    r.de = [1.0, 0.0, 0.0, 1.0];
    r.para = [0.0, 0.0, 1.0, 1.0];
    InspectorTweenInfo {
        entity_bits: ENTITY,
        rows: vec![r],
        tem_sprite: true,
        selected_count: 1,
    }
}

/// ⭐⭐⭐ **UM CANAL DE COR PINTA AMOSTRAS, E OS CAMPOS NUMÉRICOS DESAPARECEM.**
///
/// ⛔ **Os dois caminhos são EXCLUSIVOS**, e é a segunda metade que o afirma: um painel que
/// pintasse os dois daria duas respostas a *«que cor é esta?»*, e elas divergiriam no primeiro
/// arrasto de um dos campos.
///
/// **Mutações que devem sangrar:** trocar o `canal.e_cor()` por `false` · pintar os dois ramos.
#[test]
fn um_canal_de_cor_pinta_amostras_e_nao_campos() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_tween(Some(linha_de_cor()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let tem = |id| rects.iter().any(|(n, _)| *n == id);
    assert!(
        tem(ids::INSP_TWEEN_COR_DE) && tem(ids::INSP_TWEEN_COR_PARA),
        "um canal de COR nao pintou as duas amostras"
    );
    for id in ids::INSP_TWEEN_DE.iter().chain(ids::INSP_TWEEN_PARA.iter()) {
        assert!(
            !tem(*id),
            "o campo numerico {id:?} foi pintado AO LADO da amostra — duas respostas a` mesma cor"
        );
    }
    set_current_inspector_tween(None);

    // ⛔ O CONTROLO: um canal ESCALAR faz exactamente o contrário.
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_tween(Some(info()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let tem = |id| rects.iter().any(|(n, _)| *n == id);
    assert!(
        tem(ids::INSP_TWEEN_DE[0]) && !tem(ids::INSP_TWEEN_COR_DE),
        "controlo: um canal escalar tem de pintar CAMPOS e nenhuma amostra"
    );
    set_current_inspector_tween(None);
}

/// ⭐⭐⭐ **CARREGAR NUMA AMOSTRA ABRE O SELECTOR, semeado com a cor do documento.**
///
/// ⚠️ **É o gesto REAL** (`click_at`): uma amostra não carrega valor nenhum (a cor vive na tabela
/// lateral), logo ela pinta-se e hit-regista-se na mesma — e sem o registo o clique morre no
/// `is_focusable`, **sem o selector abrir**.
///
/// **Mutações que devem sangrar:** tirar as duas amostras do `populate` · tirar o braço do
/// despacho · semear o selector com a cor errada.
#[test]
fn carregar_numa_amostra_abre_o_selector_com_a_cor_do_documento() {
    for (id, esperada, nome) in [
        (ids::INSP_TWEEN_COR_DE, [255_u8, 0, 0, 255], "de"),
        (ids::INSP_TWEEN_COR_PARA, [0_u8, 0, 255, 255], "para"),
    ] {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_tween(Some(linha_de_cor()));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("a amostra «{nome}» nao foi pintada"));
        let evs = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "a amostra «{nome}» nao aceitou o clique: {evs:?} — ela esta' MORTA SOB O DEDO"
        );
        for ev in evs {
            let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
        }
        assert_eq!(
            host.store().picker_target(),
            Some(id),
            "a amostra «{nome}» nao abriu o selector"
        );
        assert_eq!(
            host.store().widget_color(id),
            Some(esperada),
            "o selector da amostra «{nome}» abriu com a cor errada"
        );
        set_current_inspector_tween(None);
    }
}

/// ⭐⭐⭐ **A COR ESCOLHIDA CHEGA AO TWEEN** — a metade em que o fio costuma morrer.
///
/// ⛔⛔ **E ela corre ANTES da saída antecipada da semente:** enquanto o artista escolhe, o
/// DOCUMENTO ainda não mudou, logo a assinatura é a mesma — e um `return` ali deixaria a cor morrer
/// no `widget_color`. *O fio estaria completo até ao último passo.*
///
/// **Mutações que devem sangrar:** pôr o bloco da cor DEPOIS da saída antecipada · não comparar
/// com o que está gravado (o barramento gira para sempre).
#[test]
fn a_cor_escolhida_chega_ao_tween() {
    use ph2d_editor_core::panel::PanelHostInternal as _;
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_tween(Some(linha_de_cor()));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    // O artista abre o selector e escolhe outra cor.
    host.store_mut()
        .set_picker_target(Some(ids::INSP_TWEEN_COR_DE));
    host.store_mut()
        .set_widget_color(ids::INSP_TWEEN_COR_DE, [0, 255, 0, 255]);
    let _ = host.drained_actions();
    // …e o quadro seguinte tem de a levar ao documento.
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let acoes = host.drained_actions();
    set_current_inspector_tween(None);

    let esperado = ph2d_editor_core::action_bus::EditorAction::InspectorComponentEdit {
        entity_bits: ENTITY,
        edit: ComponentEdit::Tween(TweenFieldEdit::Cor(0, false, [0.0, 1.0, 0.0, 1.0])),
    };
    assert!(
        acoes.contains(&esperado),
        "a cor escolhida NAO chegou ao tween: {acoes:?}"
    );
}
