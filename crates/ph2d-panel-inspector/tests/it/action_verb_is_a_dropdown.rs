//! ⭐⭐⭐ **O VERBO de uma acção escolhe-se num SELETOR, e o seletor está VIVO.**
//!
//! # Porque este ficheiro existe
//!
//! Report do dono, 2026-09-09: *«as actions deveriam ficar num dropdown e não em muitos botões»*.
//! A fileira de cinco botões saiu e entrou um chip com popover — e a troca **muda a espécie do
//! defeito possível**, que é a razão de um gate novo em vez de um gate emendado:
//!
//! | antes (fileira) | depois (seletor) |
//! |---|---|
//! | os 5 ids eram pintados e hit-registados **em todo quadro** | só o CHIP o é; as 5 opções só existem enquanto o popover está aberto |
//! | um id por fazer no `populate` morria **à vista** | um id por fazer no `populate` morre **dentro de uma lista que o artista teve de abrir** |
//! | o rect vinha do `segment_rects`, ao lado da chamada | o rect vem de um **passe diferido**, com um slot de estado pelo meio |
//!
//! ⇒ o novo caminho tem **três** juntas que ninguém vê da chamada: `set_pending_action_dd` →
//! `take_pending_action_dd` → `option_rect`. Qualquer uma partida dá o mesmo sintoma — *«abre e
//! não dá para escolher nada»* — e **compila**.
//!
//! # ⚠️ O gesto é REAL, e não um `WidgetEvent::Click`
//!
//! Um `Click` sintético entra por cima da checagem de focabilidade do dispatcher, logo passa sobre
//! um chip **pintado, hit-registado e ausente do `populate`** — a família que os dez chips do
//! impasto e as 36 células da matriz de física já pagaram, e a mesma que o `seam_bool.rs` do vector
//! nomeia. Aqui o `Down`+`Up` cai no rectângulo que a pintura de facto registou.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::hero::{ActionFieldEdit, InspectorActionInfo, InspectorActionRow};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_action};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;
/// A posição de `Show` em `SignalVerb::ALL` — **e a posição É a tag** (ver o doc de
/// `ids::inspector_action`). ⚠️ Escolhido de propósito diferente do verbo da fixtura: um alvo igual
/// ao valor de partida não distingue *«a escolha chegou»* de *«nada aconteceu»*.
const TAG_SHOW: u8 = 2;

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

/// ⚠️ **Os rótulos vêm no snapshot** porque o painel não conhece o enum dos verbos (ADR-0029). A
/// fixtura escreve-os à mão de propósito: quem os liga à fonte (`SignalVerb::ALL`) é um gate da
/// shell, e repetir aqui essa ligação faria dois sítios a responder à mesma pergunta.
fn actions() -> InspectorActionInfo {
    InspectorActionInfo {
        entity_bits: 0xABCD_1234,
        rows: vec![InspectorActionRow {
            on: "abre".into(),
            target: "Porta".into(),
            // `Hide`, para o alvo do clique (`Show`) ser outro.
            verb_tag: 3,
            arg: String::new(),
            uses_arg: false,
            target_mode: 0,
            from_tag: 0,
            target_tag: None,
            target_tag_path: String::new(),
        }],
        verb_labels: vec![
            "Start Timer".into(),
            "Stop Timer".into(),
            "Show".into(),
            "Hide".into(),
            "Toggle Visibility".into(),
            "Play Sound".into(),
            "Stop Sound".into(),
            "Add to Counter".into(),
            "Destroy".into(),
        ],
        selected_count: 1,
    }
}

/// ⚠️ **A fixtura tem de acompanhar o modelo, e ela DIZ-O em vez de derivar sozinha.**
///
/// Os rótulos são escritos à mão de propósito (ver o doc de [`actions`]), então um verbo novo
/// deixa-a curta — e o sintoma, sem esta guarda, é um *«a entrada 5 não foi pintada»* que parece um
/// defeito do painel. ⚠️ **Aconteceu**: os verbos de som (TOP-20 #4) levaram a lista de 5 a 7 e os
/// dois gates reprovaram sobre um painel correcto. *Uma fixtura escrita à mão precisa de uma cerca
/// que diga que ela é que envelheceu.*
fn assert_fixture_covers_the_model(i: &InspectorActionInfo) {
    assert_eq!(
        i.verb_labels.len(),
        ph2d_panel_inspector::ids::INSP_ACTION_VERB.len(),
        "a FIXTURA e' que esta' velha: o modelo tem {} verbos e ela escreve {} rotulos",
        ph2d_panel_inspector::ids::INSP_ACTION_VERB.len(),
        i.verb_labels.len()
    );
}

/// Um host com o Inspector populado e a secção SIGNAL ACTIONS no snapshot.
fn host() -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    let info = actions();
    assert_fixture_covers_the_model(&info);
    set_current_inspector_action(Some(info));
    (h, InspectorState::default())
}

/// ⭐ **O chip existe, com área, e é um `Dropdown` no store.**
///
/// ⚠️ As duas metades: sem área ele nunca é alcançado; sem o registo como `Dropdown` o dispatcher
/// genérico não sabe **abrir** nada — o clique chegaria e não faria coisa nenhuma.
///
/// **Mutação que deve sangrar:** apagar o `store.register(INSP_ACTION_VERB_PICK, …)` do
/// `populate_action`.
#[test]
fn the_verb_chip_is_painted_and_is_a_dropdown() {
    let (mut h, mut st) = host();
    let r = h
        .painted_rect::<InspectorPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_inspector::ids::INSP_ACTION_VERB_PICK,
        )
        .expect("o chip do verbo nao foi PINTADO com area clicavel");
    assert!(r.w > 0.0 && r.h > 0.0, "chip sem area: {r:?}");
    assert_eq!(
        h.dropdown_is_open(ph2d_panel_inspector::ids::INSP_ACTION_VERB_PICK),
        Some(false),
        "o chip nao esta' registado como Dropdown — o dispatcher nao o sabe abrir"
    );
}

/// ⭐⭐ **Aberto, o seletor põe as CINCO entradas sob o ponteiro — e o popover é publicado.**
///
/// ⚠️ **Três coisas, porque são três juntas distintas.** As opções só chegam ao índice de acerto se
/// o rect do chip atravessou o slot até ao passe diferido; e o rect do POPOVER só chega ao store se
/// o fim da pintura o publicar — sem isso, clicar fora **não fecha** o seletor (a lei do
/// `pointer_down`, pedida pelo dono em 2026-06-24).
///
/// **Mutações que devem sangrar:** apagar o `set_pending_action_dd` · apagar o bloco do verbo no
/// `paint_deferred_popovers` · apagar o `set_dropdown_popover` no fim do `paint`.
#[test]
fn opening_it_makes_every_verb_reachable_and_publishes_the_popover() {
    let (mut h, mut st) = host();
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_dropdown_open(ph2d_panel_inspector::ids::INSP_ACTION_VERB_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);

    let mut vistos: Vec<Rect> = Vec::new();
    for (i, &id) in ph2d_panel_inspector::ids::INSP_ACTION_VERB
        .iter()
        .enumerate()
    {
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| {
                panic!(
                    "a entrada {i} do seletor nao chegou ao indice de acerto — o popover abre e \
                     nao se escolhe nada"
                )
            });
        assert!(r.w > 0.0 && r.h > 0.0, "entrada {i} sem area: {r:?}");
        for (j, a) in vistos.iter().enumerate() {
            assert!(
                r.y >= a.y + a.h || a.y >= r.y + r.h,
                "as entradas {j} e {i} ocupam a mesma banda ({a:?} contra {r:?}) — uma delas e' \
                 inalcancavel"
            );
        }
        vistos.push(r);
    }

    let (dono, painel) = h
        .store()
        .dropdown_popover()
        .expect("o rect do popover nao foi publicado — clicar FORA nao fecharia o seletor");
    assert_eq!(
        dono,
        ph2d_panel_inspector::ids::INSP_ACTION_VERB_PICK,
        "o popover publicado e' de outro seletor"
    );
    for (i, r) in vistos.iter().enumerate() {
        assert!(
            painel.contains(r.x + r.w * 0.5, r.y + r.h * 0.5),
            "a entrada {i} cai FORA do painel publicado ({painel:?}) — um clique nela seria lido \
             como «clicou fora» e fecharia o seletor em vez de escolher"
        );
    }
}

/// ⭐⭐⭐ **Escolher um verbo no seletor CHEGA ao barramento — e fecha a lista.**
///
/// Este é o gate que mede a costura inteira, com o gesto real: ponteiro → dispatcher → evento →
/// `apply_event` do painel → `EditorAction`.
///
/// ⚠️ **A segunda metade não é decoração:** um popover que não fecha depois da escolha fica a tapar
/// as secções por baixo dele, que é a queixa de que este trabalho nasceu.
///
/// **Mutações que devem sangrar:** apagar o `close_verb_popover` · apagar o
/// `register_button_ids(store, &ids::INSP_ACTION_VERB)` do `populate_action` (o ponteiro deixa de
/// virar evento nenhum).
#[test]
fn picking_a_verb_reaches_the_bus_and_closes_the_list() {
    let (mut h, mut st) = host();
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_dropdown_open(ph2d_panel_inspector::ids::INSP_ACTION_VERB_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let alvo = ph2d_panel_inspector::ids::INSP_ACTION_VERB[TAG_SHOW as usize];
    let (_, r) = rects
        .iter()
        .find(|(n, _)| *n == alvo)
        .copied()
        .expect("a entrada `Show` nao foi pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);

    let _ = h.drained_actions();
    // ⚠️ **As DUAS fases**: um `Button` fala no `Up`, e enumerar só uma passaria para a família que
    // fala nela e reprovaria a outra — a lição que o seam do painel autorado já pagou.
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre a entrada `Show` nao virou evento nenhum — ela esta' desenhada e nao \
         existe para o dispatcher (falta o registo no populate)"
    );
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }

    let acoes = h.drained_actions();
    let escrito = acoes.iter().any(|a| {
        matches!(
            a,
            EditorAction::InspectorActionEdit {
                edit: ActionFieldEdit::Verb(0, t),
                ..
            } if *t == TAG_SHOW
        )
    });
    assert!(
        escrito,
        "escolher `Show` nao chegou ao barramento; o que chegou foi {acoes:?}"
    );
    assert_eq!(
        h.dropdown_is_open(ph2d_panel_inspector::ids::INSP_ACTION_VERB_PICK),
        Some(false),
        "o seletor ficou ABERTO depois da escolha — a lista continuaria a tapar as seccoes de baixo"
    );
    assert!(
        !matches!(
            h.store().get(alvo),
            Some(InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Pressed
            })
        ),
        "a entrada ficou `Pressed` depois do clique — ela reabriria acesa"
    );
}

/// ⚠️ **Uma janela do tamanho do ALVO** — o app é para tablet, e é ali que a lista de baixo do
/// Inspector deixa de caber. A do gate acima (1600×900) **não produz o fenómeno**: com 16 acções o
/// chip fica a `y = 609` e a lista pendurada abaixo acaba a `747`, ainda dentro de uma região que
/// vai até `900`. *Uma cerca que nunca morde não é uma cerca.*
const JANELA_TABLET: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1280.0,
    h: 720.0,
};

/// Uma secção com `n` acções — o que empurra o editor (e o chip) para o fundo do painel.
fn actions_n(n: usize) -> InspectorActionInfo {
    let mut i = actions();
    i.rows = (0..n).map(|_| i.rows[0].clone()).collect();
    i
}

/// ⭐⭐⭐ **A lista VIRA PARA CIMA quando abaixo do chip não cabe** — ela nunca sai da região.
///
/// Report do dono, 2026-09-09: *«o dropdown está abrindo fora da tela para baixo, não se adapta à
/// posição do widget»*. Os quatro seletores do Inspector penduravam a lista **sempre abaixo**
/// (`popover_rect`), enquanto o resto do app já usava o `popover_rect_clamped`.
///
/// ⚠️ **O gate tem DUAS metades, e a primeira é sobre a FIXTURA:** ele mede primeiro que, sem o
/// clamp, esta cena de facto transbordaria — senão a segunda metade passaria por a lista caber, e
/// não por o código a virar.
///
/// **Mutação que deve sangrar:** trocar o `popover_rect_clamped(chip, region)` da porta pelo
/// `popover_rect(chip)`.
#[test]
fn the_list_flips_above_when_below_would_leave_the_screen() {
    let regiao = HeroLayout::for_viewport(JANELA_TABLET).popover_region();
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let info = actions_n(16);
    assert_fixture_covers_the_model(&info);
    set_current_inspector_action(Some(info));
    let _ = h.paint::<InspectorPanel>(&mut st, JANELA_TABLET);
    h.set_dropdown_open(ph2d_panel_inspector::ids::INSP_ACTION_VERB_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, JANELA_TABLET);

    let (_, chip) = rects
        .iter()
        .find(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_ACTION_VERB_PICK)
        .copied()
        .expect("o chip do verbo nao foi pintado");

    // **Metade 1 — a fixtura produz o fenómeno.** Pendurada abaixo, a lista sairia da região.
    let altura_da_lista = chip.h * ph2d_panel_inspector::ids::INSP_ACTION_VERB.len() as f32;
    let fundo_se_abaixo = chip.y + chip.h + altura_da_lista;
    assert!(
        fundo_se_abaixo > regiao.y + regiao.h,
        "a fixtura NAO produz o fenomeno: pendurada abaixo a lista acabaria em {fundo_se_abaixo:.1}          e a regiao vai ate' {:.1} — este gate estaria a passar por caber, nao por virar",
        regiao.y + regiao.h
    );

    // **Metade 2 — nenhuma entrada sai da região.**
    for (i, &id) in ph2d_panel_inspector::ids::INSP_ACTION_VERB
        .iter()
        .enumerate()
    {
        let (_, r) = rects
            .iter()
            .find(|(n, _)| *n == id)
            .copied()
            .unwrap_or_else(|| panic!("a entrada {i} nao foi pintada"));
        assert!(
            r.y >= regiao.y && r.y + r.h <= regiao.y + regiao.h,
            "a entrada {i} cai FORA da regiao do chrome (y {:.1}..{:.1} contra {:.1}..{:.1}) — o              artista nao lhe chega",
            r.y,
            r.y + r.h,
            regiao.y,
            regiao.y + regiao.h
        );
    }
    set_current_inspector_action(None);
}
