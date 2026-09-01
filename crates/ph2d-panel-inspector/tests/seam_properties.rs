//! **Varredura de SEAM do cartão de PROPRIEDADES** (report do Enio, 2026-08-31).
//!
//! Irmã da `seam_anim.rs`, com a MESMA disciplina: todo clique passa pelo `click_at` REAL, e não
//! por um `WidgetEvent` sintético. Um evento fabricado pula a checagem de focabilidade do store,
//! então um widget deixado de fora do `populate` fica pintado, hit-registrado e **morto sob o
//! mouse**, com um teste verde ao lado.
//!
//! # ⚠️ O que só se mede aqui
//!
//! A lei (`variant_axes_tests`) diz **que fileiras** o cartão tem; os gates da shell dizem **de que
//! nome** elas saem. Nenhum dos dois responde: *o chip é pintado? está no hit-index? o clique nasce
//! e chega ao `swap`?* — e o cartão mudou de dono nesta wave, que é exactamente quando essa metade
//! se perde.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::screens::hero::{
    InspectorNameInfo, InspectorPropertiesInfo, VariantChoice, variant_axes::VariantAxis,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_name, set_current_inspector_properties,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_0077;
const ROOT: u64 = 0x5EED_0078;
const MINE: u64 = 11;
const OTHER: u64 = 22;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn choice(master: u64, label: &str, current: bool) -> VariantChoice {
    VariantChoice {
        master,
        label: label.into(),
        current,
    }
}

/// Uma fileira que PERGUNTA (`Size`, dois valores) e uma que só DECLARA (`State`, um valor).
///
/// ⚠️ **As duas na mesma fixtura, de propósito**: com só uma, um pintor que tratasse as duas
/// espécies igual ficaria verde por não haver a outra para onde errar.
fn info() -> InspectorPropertiesInfo {
    InspectorPropertiesInfo {
        entity_bits: ENTITY,
        root_bits: ROOT,
        rows: vec![
            VariantAxis {
                name: "Size".into(),
                options: vec![choice(MINE, "Small", true), choice(OTHER, "Big", false)],
            },
            VariantAxis {
                name: "State".into(),
                options: vec![choice(0, "Idle", true)],
            },
        ],
        beyond: 0,
        // A fixtura é uma CÓPIA (tem `root_bits`), então as propriedades são do componente.
        source_name: Some("Casa".into()),
    }
}

fn host() -> (MockPanelHost, InspectorState) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Casa {Size=Small, State=Idle}".into(),
    }));
    set_current_inspector_properties(Some(info()));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    (host, state)
}

fn clear() {
    set_current_inspector_properties(None);
    set_current_inspector_name(None);
}

/// ⭐⭐⭐ **O chip de uma pergunta CHEGA ao `swap`, pelo ponteiro.**
///
/// **Mutação que deve sangrar:** apagar o `hit_index.register(id, host)` do pintor, ou o braço do
/// `variant_click` no despachante.
#[test]
fn a_chip_of_a_real_question_reaches_the_swap() {
    let (mut host, mut state) = host();
    let id = ids::INSP_INSTANCE_AXIS_OPTION[0][1];
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .expect("o chip «Big» nunca foi pintado nem registado");
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        !events.is_empty(),
        "clicar no chip não produziu evento — ele está morto sob o mouse (fora do `populate`)"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let sent = host.drained_actions();
    assert!(
        sent.iter().any(|a| matches!(
            a,
            EditorAction::InspectorSwapVariant { root_bits, master }
                if *root_bits == ROOT && *master == OTHER
        )),
        "o clique não chegou ao swap com o mestre certo: {sent:?}"
    );
    clear();
}

/// ⛔⛔ **UM VALOR SÓ NÃO É UM BOTÃO** — ele não entra no hit-index, logo não há clique a morrer.
///
/// *Um controlo que se pode carregar e que não faz nada é a 1.ª espécie de knob morto da caça de
/// 2026-08-30* — e aqui ela seria sistemática: toda propriedade declarada de todo objecto solto.
///
/// **Mutação que deve sangrar:** o pintor desenhar o valor único como `Button` + `register`.
#[test]
fn a_declared_value_is_text_and_never_a_dead_button() {
    let (mut host, mut state) = host();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let id = ids::INSP_INSTANCE_AXIS_OPTION[1][0];
    assert!(
        !rects.iter().any(|(n, _)| *n == id),
        "a fileira de UM valor registou um hit-rect — é um botão que o artista carrega para nada"
    );
    clear();
}

/// ⭐⭐⭐ **CARREGAR NO VALOR ACESO ABRE-O PARA ESCRITA, E O ENTER GRAVA NA RECEITA.**
///
/// # ⛔⛔⛔ O gesto que faltava era um clique MORTO
///
/// Report do Enio (2026-08-31, a quarta vez): *«Que inferno!!!»*. Ele escrevia `{Size=Big}` no nome
/// da **cópia** para dar nome ao valor, e o modelo ignorava-o — correctamente, porque uma
/// propriedade é do COMPONENTE. *O defeito é que autorar o valor obrigava a seleccionar OUTRO
/// objecto do que aquele que se está a olhar.*
///
/// E o clique no chip já aceso era um **no-op silencioso** — exactamente onde o dedo dele estava.
///
/// ⚠️ **Pelo `click_at` REAL**: um `WidgetEvent` sintético salta a checagem de focabilidade do
/// store, e um campo que nasce sem foco come as teclas sem as mostrar.
#[test]
fn clicking_the_current_value_opens_it_for_writing_and_enter_saves_it() {
    let (mut host, mut state) = host();
    let id = ids::INSP_INSTANCE_AXIS_OPTION[0][0]; // o `Small`, que é o VIGENTE
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .expect("o chip vigente nunca foi pintado");
    for ev in host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    assert!(
        host.drained_actions().is_empty(),
        "carregar no vigente nao pode levantar uma TROCA — ele ja' esta' escolhido"
    );

    // ⭐ O campo nasce com o valor dentro e SELECCIONADO — a lição do `CatalogHeroes`.
    match host.store().get(ids::INSP_INSTANCE_VALUE_EDIT) {
        Some(InteractiveState::TextInput {
            text,
            selection_anchor,
            ..
        }) => {
            assert_eq!(text, "Small");
            assert_eq!(*selection_anchor, Some(0), "abriu sem seleccionar o valor");
        }
        other => panic!("o campo nao abriu: {other:?}"),
    }
    // E ele é pintado, no lugar do chip.
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ids::INSP_INSTANCE_VALUE_EDIT),
        "o campo nao foi pintado nem registado"
    );

    // ⭐⭐ O Enter grava — **na RECEITA vigente**, e com a chave da FILEIRA.
    // ⚠️ Escreve pelo mesmo caminho que uma tecla escreveria — o arnês não expõe o store mutável,
    // e é bom que não exponha: um gate que semeia por dentro mede o que ele próprio pôs lá.
    host.set_text(ids::INSP_INSTANCE_VALUE_EDIT, "Big");
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Submit(ids::INSP_INSTANCE_VALUE_EDIT),
    );
    let sent = host.drained_actions();
    assert!(
        sent.iter().any(|a| matches!(
            a,
            EditorAction::InspectorRenameVariantValue { master, key, value, .. }
                if *master == MINE && key == "Size" && value == "Big"
        )),
        "o Enter nao gravou o valor na receita: {sent:?}"
    );
    clear();
}

/// ⛔⛔ **Trocar de versão FECHA o campo.**
///
/// ⚠️ Sem isto ele ficaria aberto sobre um valor que já não é o vigente, e o `Blur` seguinte
/// gravaria o texto na propriedade **errada** — um defeito que só aparece no segundo gesto.
#[test]
fn switching_versions_closes_the_open_field() {
    let (mut host, mut state) = host();
    // abre no vigente…
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let cur = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_INSTANCE_AXIS_OPTION[0][0])
        .map(|(_, r)| *r)
        .expect("o vigente");
    for ev in host.click_at(cur.x + cur.w * 0.5, cur.y + cur.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let _ = host.drained_actions();
    // …e carrega no OUTRO.
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let other = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_INSTANCE_AXIS_OPTION[0][1])
        .map(|(_, r)| *r)
        .expect("o outro");
    for ev in host.click_at(other.x + other.w * 0.5, other.y + other.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ids::INSP_INSTANCE_VALUE_EDIT),
        "o campo ficou aberto depois de trocar de versao"
    );
    clear();
}

/// ⛔⛔⛔ **A TROCA DE SELEÇÃO LARGA O CAMPO — e um Blur tardio não grava no objecto novo.**
///
/// Auditoria de 2026-08-31 (A1): abrir o campo no objecto A, trocar a seleção para B e clicar em
/// qualquer coisa gravava **o texto de A na receita de B** — o `commit` relia o snapshot vigente e
/// endereçava por índice. As duas metades da cura: o `sync` abandona na troca, e o `commit`
/// confere a IDENTIDADE (entidade + nome do eixo).
///
/// (Mutações: tirar o `abandon` do sync ⇒ RED na 2.ª asserção; tirar a conferência de identidade
/// do commit ⇒ RED na 3.ª.)
#[test]
fn a_selection_change_abandons_the_field_and_a_late_blur_writes_nothing() {
    let (mut host, mut state) = host();
    // Abre no vigente de A…
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let cur = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_INSTANCE_AXIS_OPTION[0][0])
        .map(|(_, r)| *r)
        .expect("o vigente");
    for ev in host.click_at(cur.x + cur.w * 0.5, cur.y + cur.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let _ = host.drained_actions();
    assert!(state.value_edit.is_some(), "o campo nem abriu");

    // …e o mundo troca para o objecto B (outra entidade, outro cartão).
    let b_entity = 0x5EED_00FF_u64;
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: b_entity,
        name: "Outra {Color=Red}".into(),
    }));
    set_current_inspector_properties(Some(InspectorPropertiesInfo {
        entity_bits: b_entity,
        root_bits: 0x5EED_0100,
        rows: vec![VariantAxis {
            name: "Color".into(),
            options: vec![choice(33, "Red", true), choice(34, "Blue", false)],
        }],
        beyond: 0,
        source_name: Some("Outra".into()),
    }));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    // O sync viu a entidade mudar ⇒ o campo foi LARGADO.
    assert!(
        state.value_edit.is_none(),
        "o campo sobreviveu à troca de seleção — hibernaria com o texto de A"
    );
    // E um Blur tardio não grava NADA — nem em A nem em B.
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Blur(ids::INSP_INSTANCE_VALUE_EDIT),
    );
    let sent = host.drained_actions();
    assert!(
        sent.is_empty(),
        "um Blur tardio gravou depois da troca de seleção: {sent:?}"
    );
    clear();
}

/// ⛔⛔ **O painel ESCONDIDO larga o campo e o foco** — senão um `TextInput` invisível come as
/// teclas do app inteiro (auditoria de 2026-08-31, A3; é a porta homóloga do molde do navegador
/// de assets).
///
/// (Mutação: tirar o `abandon` do ramo escondido do `paint` ⇒ RED.)
#[test]
fn hiding_the_panel_abandons_the_field_and_releases_the_keyboard() {
    let (mut host, mut state) = host();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let cur = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_INSTANCE_AXIS_OPTION[0][0])
        .map(|(_, r)| *r)
        .expect("o vigente");
    for ev in host.click_at(cur.x + cur.w * 0.5, cur.y + cur.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let _ = host.drained_actions();
    assert!(state.value_edit.is_some());
    assert_eq!(host.store().focus_id(), Some(ids::INSP_INSTANCE_VALUE_EDIT));

    host.paint_hidden::<InspectorPanel>(&mut state, VIEWPORT);
    assert!(
        state.value_edit.is_none(),
        "fechar o painel deixou o campo armado"
    );
    assert_ne!(
        host.store().focus_id(),
        Some(ids::INSP_INSTANCE_VALUE_EDIT),
        "fechar o painel deixou um TextInput invisivel com o teclado"
    );
    clear();
}

/// ⛔ **No modo PLANO o chip aceso NÃO abre campo nenhum** — um campo que abre, aceita texto e
/// nunca grava é um controlo que come trabalho em silêncio (auditoria de 2026-08-31, A4: a decisão
/// de ABRIR e a de GRAVAR têm de ler a mesma pergunta).
///
/// (Mutação: tirar o filtro do plano do `chip_click` ⇒ RED.)
#[test]
fn a_flat_lit_chip_never_opens_the_field() {
    // Um cartão em modo PLANO: eixo sem nome, chips com nomes de receita.
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Casa".into(),
    }));
    set_current_inspector_properties(Some(InspectorPropertiesInfo {
        entity_bits: ENTITY,
        root_bits: ROOT,
        rows: vec![VariantAxis {
            name: String::new(),
            options: vec![choice(MINE, "Casa", true), choice(OTHER, "Outra", false)],
        }],
        beyond: 0,
        source_name: Some("Casa".into()),
    }));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let cur = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_INSTANCE_AXIS_OPTION[0][0])
        .map(|(_, r)| *r)
        .expect("o chip plano vigente");
    for ev in host.click_at(cur.x + cur.w * 0.5, cur.y + cur.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    assert!(
        state.value_edit.is_none(),
        "o chip plano abriu um campo que o commit recusaria em silencio"
    );
    clear();
}

/// ⛔⛔ **A conferência de ENTIDADE do commit tem o SEU caso** — e ele não é o do abandono do sync.
///
/// A mutação que a apagava sobreviveu: com o abandono do sync no lugar, a troca de seleção limpa o
/// campo antes de qualquer Blur. A janela que SÓ ela cobre: o snapshot novo é publicado e o `Blur`
/// chega **antes do paint seguinte** (o sync ainda não correu) — e o objecto B tem um eixo com o
/// MESMO nome (`Size`), então a busca por nome não salva. Sem a conferência, o texto de A ia para
/// a receita de B.
///
/// (Mutação: tirar o `info.entity_bits != edit.entity_bits` do commit ⇒ RED.)
#[test]
fn a_blur_between_publish_and_paint_never_writes_into_the_new_object() {
    let (mut host, mut state) = host();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let cur = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_INSTANCE_AXIS_OPTION[0][0])
        .map(|(_, r)| *r)
        .expect("o vigente");
    for ev in host.click_at(cur.x + cur.w * 0.5, cur.y + cur.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let _ = host.drained_actions();
    host.set_text(ids::INSP_INSTANCE_VALUE_EDIT, "Enorme");

    // B chega — com um eixo chamado «Size» TAMBÉM — e o Blur corre ANTES do paint.
    let b_entity = 0x5EED_0BB0_u64;
    set_current_inspector_properties(Some(InspectorPropertiesInfo {
        entity_bits: b_entity,
        root_bits: 0x5EED_0BB1,
        rows: vec![VariantAxis {
            name: "Size".into(),
            options: vec![choice(55, "Mini", true), choice(56, "Maxi", false)],
        }],
        beyond: 0,
        source_name: Some("Outra".into()),
    }));
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Blur(ids::INSP_INSTANCE_VALUE_EDIT),
    );
    let sent = host.drained_actions();
    assert!(
        sent.is_empty(),
        "o texto de A foi gravado na receita de B pela janela publish→paint: {sent:?}"
    );
    clear();
}
