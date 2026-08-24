//! **Varredura de SEAM da §11 Animation** — a que faltava.
//!
//! Irmã de `seam_player.rs`, com a MESMA disciplina: todo clique passa pelo `click_at` REAL, e
//! não por um `WidgetEvent` sintético. Um evento fabricado pula a checagem de focabilidade do
//! store, então um widget deixado de fora do `populate` fica pintado, hit-registrado e **morto sob
//! o mouse**, com um teste verde ao lado.
//!
//! # ⚠️ A caixa «Playing» pergunta à CENA, nunca à própria memória
//!
//! É o defeito que o Enio reportou em 2026-08-23 — *«às vezes preciso clicar mais de uma vez para
//! checar Playing»* — e ele é uma **dupla fonte de verdade**: a §11 *pinta* a caixa a partir do
//! snapshot (`info.playing`, que é o mundo) e *decidia* a partir do valor guardado no
//! `WidgetStore`. Os dois concordam enquanto só o painel escrever — e o `playing` é justamente o
//! campo que o **motor** escreve por conta própria: uma animação que não repete põe-se a `false`
//! sozinha ao chegar ao fim. A partir daí a caixa desenhada e a caixa lembrada discordam, e o
//! primeiro clique manda o oposto do que o artista vê.
//!
//! *Uma caixa que se lembra do que mostrou mente no dia em que outra pessoa muda o facto.*

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{
    AnimFieldEdit, InspectorAnimInfo, InspectorAnimRow, InspectorNameInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_anim, set_current_inspector_name,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_0011;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// Uma sprite com tocador e duas animações — o estado em que TODOS os controlos são pintados.
///
/// ⚠️ **Duas linhas e não uma**: com uma só, um despacho que ignorasse o índice da linha aberta
/// ficaria verde por não haver outra para onde errar. *Uma fixtura que não contém o fenómeno mede
/// silêncio.*
fn anim(playing: bool, autoplay: bool) -> InspectorAnimInfo {
    InspectorAnimInfo {
        entity_bits: ENTITY,
        rows: vec![
            InspectorAnimRow {
                name: "walk".into(),
                from: 0,
                to: 3,
                frame_ms: 90,
                direction_tag: 0,
                repeat: 0,
                hold_ms: 40,
                repeat_delay_ms: 250,
                signal_on_finish: String::new(),
                signal_on_loop: String::new(),
                per_frame_ms: Vec::new(),
            },
            InspectorAnimRow {
                name: "attack".into(),
                from: 4,
                to: 7,
                frame_ms: 110,
                direction_tag: 2,
                repeat: 1,
                hold_ms: 0,
                repeat_delay_ms: 0,
                signal_on_finish: String::new(),
                signal_on_loop: String::new(),
                per_frame_ms: Vec::new(),
            },
        ],
        player_present: true,
        cells: 8,
        current: "walk".into(),
        playing,
        autoplay,
        // ⚠️ Longe de `1.0` (o seed do `populate`), pela razão do `the_rows_show_what_was_authored`
        // da §14: um número igual ao de fábrica deixa o espelho verde por coincidência.
        speed: 2.5,
        direction_override_tag: 0,
        loop_override_tag: 0,
        frame: 1,
        selected_count: 1,
    }
}

/// A mesma sprite **sem** tocador — a face que oferece um botão e mais nada.
fn no_player() -> InspectorAnimInfo {
    InspectorAnimInfo {
        player_present: false,
        ..anim(false, false)
    }
}

/// Um host já pintado uma vez, com o NOME publicado.
///
/// ⚠️ **O nome faz parte da fixtura, e a premissa é declarada:** o `entity_changed` do sync — a
/// porta que decide re-semear as caixas — lê a entidade do transform/nome/visibilidade, nunca do
/// info da §11. Sem esta publicação o gate mediria um sync que nunca correu.
fn host_with(info: InspectorAnimInfo) -> (MockPanelHost, InspectorState) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Hero".into(),
    }));
    set_current_inspector_anim(Some(info));
    let _ = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    (host, state)
}

fn clear() {
    set_current_inspector_anim(None);
    set_current_inspector_name(None);
}

/// Clica no meio do widget e devolve o que chegou ao barramento.
fn click(
    host: &mut MockPanelHost,
    state: &mut InspectorState,
    id: ph2d_a11y::NodeId,
) -> Vec<EditorAction> {
    let rects = host.paint::<InspectorPanel>(state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("a §11 nunca pintou o widget {id:?}"));
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        !events.is_empty(),
        "clicar no meio de {id:?} não produziu evento nenhum — ele é pintado e \
         hit-registrado, mas o store não o considera focável: está morto sob o mouse"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(state, ev);
    }
    host.drained_actions()
}

/// Clica num widget de uma sprite recém-pintada e devolve o barramento.
fn click_fresh(info: InspectorAnimInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let (mut host, mut state) = host_with(info);
    let out = click(&mut host, &mut state, id);
    clear();
    out
}

fn commit(info: InspectorAnimInfo, id: ph2d_a11y::NodeId, v: f64) -> Vec<EditorAction> {
    let (mut host, mut state) = host_with(info);
    host.set_number_value(id, v);
    let _ = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::ValueChanged(id));
    let out = host.drained_actions();
    clear();
    out
}

#[track_caller]
fn expect(actions: &[EditorAction], edit: AnimFieldEdit, what: &str) {
    assert_eq!(
        actions,
        [EditorAction::InspectorAnimEdit {
            entity_bits: ENTITY,
            edit,
        }],
        "{what} não levantou a edição que devia"
    );
}

// ── O DEFEITO REPORTADO ──────────────────────────────────────────────────────────────────────

/// **A caixa «Playing» decide pelo que está DESENHADO, e não pelo que ela lembra.**
///
/// O caso do meio é o defeito: a cena parou sozinha (uma animação de uma volta chegou ao fim)
/// **sem** que a entidade ou a linha aberta mudassem, então a semente do sync não voltou a correr
/// e o valor guardado no store ficou a dizer «marcada». O artista vê a caixa vazia — porque o
/// pintor lê o snapshot — clica nela, e o despacho antigo lia o store: mandava `Playing(false)`
/// sobre uma cena já parada. Nada acontecia. Só o **segundo** clique tocava.
///
/// **Mutação que deve sangrar:** voltar a derivar o `on` de `host.store().checkbox(id)`.
#[test]
fn the_playing_box_asks_the_scene_not_its_own_memory() {
    // 1. A cena TOCA e o artista desmarca: `Playing(false)`.
    expect(
        &click_fresh(anim(true, false), ids::INSP_ANIM_PLAYING),
        AnimFieldEdit::Playing(false),
        "desmarcar sobre uma cena que toca",
    );

    // 2. A cena está PARADA e o artista marca: `Playing(true)`.
    expect(
        &click_fresh(anim(false, false), ids::INSP_ANIM_PLAYING),
        AnimFieldEdit::Playing(true),
        "marcar sobre uma cena parada",
    );

    // 3. ⚠️ **O DEFEITO:** a cena tocava, e parou-se **sozinha** — mesma entidade, mesma linha.
    let (mut host, mut state) = host_with(anim(true, false));
    set_current_inspector_anim(Some(anim(false, false)));
    let out = click(&mut host, &mut state, ids::INSP_ANIM_PLAYING);
    clear();
    expect(
        &out,
        AnimFieldEdit::Playing(true),
        "marcar uma caixa que a CENA esvaziou por conta própria (a animação de uma \
         volta chegou ao fim) — o primeiro clique tem de tocar, não de mandar parar \
         uma cena já parada",
    );
}

/// **O mesmo para «Autoplay»** — a lei é da caixa, não de quem hoje lhe mexe.
///
/// ⚠️ Hoje só o painel escreve o `autoplay`, então o defeito é **latente** aqui e vivo no irmão.
/// Prendê-lo agora é o que impede a próxima porta que o escreva de reabrir o mesmo buraco a partir
/// de um sítio que ninguém liga a esta caixa.
#[test]
fn the_autoplay_box_asks_the_scene_not_its_own_memory() {
    expect(
        &click_fresh(anim(false, true), ids::INSP_ANIM_AUTOPLAY),
        AnimFieldEdit::Autoplay(false),
        "desmarcar o autoplay",
    );
    let (mut host, mut state) = host_with(anim(false, true));
    set_current_inspector_anim(Some(anim(false, false)));
    let out = click(&mut host, &mut state, ids::INSP_ANIM_AUTOPLAY);
    clear();
    expect(
        &out,
        AnimFieldEdit::Autoplay(true),
        "marcar um autoplay que a cena mudou por baixo da caixa",
    );
}

// ── A VARREDURA ──────────────────────────────────────────────────────────────────────────────

/// **Todo controlo do TOCADOR chega ao barramento pelo ponteiro real.**
#[test]
fn every_player_control_reaches_the_bus() {
    expect(
        &click_fresh(anim(true, false), ids::INSP_ANIM_REWIND),
        AnimFieldEdit::Rewind,
        "Rewind",
    );
    expect(
        &commit(anim(true, false), ids::INSP_ANIM_SPEED, -1.5),
        AnimFieldEdit::Speed(-1.5),
        "Speed",
    );
    // ⚠️ **A POSIÇÃO no array é a tag** — o despacho deriva-a de `position()`. Um array reordenado
    // faria cada botão escrever a direção do vizinho, e compila.
    for (i, &id) in ids::INSP_ANIM_DIR_OVERRIDE.iter().enumerate() {
        expect(
            &click_fresh(anim(true, false), id),
            AnimFieldEdit::DirectionOverride(i as u8),
            &format!("Direction override [{i}]"),
        );
    }
    for (i, &id) in ids::INSP_ANIM_LOOP_OVERRIDE.iter().enumerate() {
        expect(
            &click_fresh(anim(true, false), id),
            AnimFieldEdit::LoopOverride(i as u8),
            &format!("Loop override [{i}]"),
        );
    }
}

/// **Todo controlo da BIBLIOTECA chega ao barramento** — e cada número o seu campo.
#[test]
fn every_library_control_reaches_the_bus() {
    expect(
        &click_fresh(anim(true, false), ids::INSP_ANIM_ADD),
        AnimFieldEdit::Add,
        "+ Add Animation",
    );
    expect(
        &click_fresh(anim(true, false), ids::INSP_ANIM_REMOVE),
        AnimFieldEdit::Remove(0),
        "× Remove Animation",
    );
    for (id, v, edit) in [
        (ids::INSP_ANIM_FROM, 2.0, AnimFieldEdit::From(0, 2)),
        (ids::INSP_ANIM_TO, 6.0, AnimFieldEdit::To(0, 6)),
        (ids::INSP_ANIM_FRAME_MS, 33.0, AnimFieldEdit::FrameMs(0, 33)),
        (ids::INSP_ANIM_HOLD_MS, 120.0, AnimFieldEdit::HoldMs(0, 120)),
        (ids::INSP_ANIM_DELAY_MS, 75.0, AnimFieldEdit::DelayMs(0, 75)),
        (ids::INSP_ANIM_REPEAT, 3.0, AnimFieldEdit::Repeat(0, 3)),
    ] {
        expect(&commit(anim(true, false), id, v), edit, &format!("{id:?}"));
    }
    for (i, &id) in ids::INSP_ANIM_DIR.iter().enumerate() {
        expect(
            &click_fresh(anim(true, false), id),
            AnimFieldEdit::Direction(0, i as u8),
            &format!("Direction [{i}]"),
        );
    }
}

/// **Clicar numa linha escolhe a animação que TOCA** — e o índice vai para a linha certa.
///
/// ⚠️ As duas metades: a linha clicada manda o NOME dela (não o da aberta), e a edição seguinte de
/// um número passa a apontar para o índice novo. Sem a segunda, um despacho que escrevesse sempre
/// `0` ficaria verde.
#[test]
fn clicking_a_row_picks_what_plays_and_moves_the_editor_with_it() {
    let (mut host, mut state) = host_with(anim(true, false));
    let out = click(&mut host, &mut state, ids::INSP_ANIM_ROW[1]);
    expect(
        &out,
        AnimFieldEdit::SetCurrent("attack".into()),
        "clicar na segunda linha",
    );
    // E agora o editor edita a SEGUNDA.
    host.set_number_value(ids::INSP_ANIM_FRAME_MS, 42.0);
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::INSP_ANIM_FRAME_MS),
    );
    let out = host.drained_actions();
    clear();
    expect(
        &out,
        AnimFieldEdit::FrameMs(1, 42),
        "o número seguinte edita a linha que se abriu",
    );
}

/// **A face sem tocador oferece UM botão, e mais nada.**
///
/// A metade da ausência é a que carrega o desenho: pintar Playing/Speed/Rewind sobre uma sprite
/// que não tem estado de reprodução seria oferecer knobs que não vão a lado nenhum.
#[test]
fn the_empty_face_offers_only_the_gesture_that_creates_the_player() {
    expect(
        &click_fresh(no_player(), ids::INSP_ANIM_ADD_PLAYER),
        AnimFieldEdit::AddPlayer,
        "+ Add Animator",
    );
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_anim(Some(no_player()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    clear();
    for id in [
        ids::INSP_ANIM_PLAYING,
        ids::INSP_ANIM_AUTOPLAY,
        ids::INSP_ANIM_SPEED,
        ids::INSP_ANIM_REWIND,
        ids::INSP_ANIM_ADD,
        ids::INSP_ANIM_ROW[0],
    ] {
        assert!(
            !rects.iter().any(|(n, _)| *n == id),
            "a face sem tocador pintou {id:?}, que edita um estado que não existe"
        );
    }
}

/// **As rows NÃO são write-only** — o espelho existe, e mostra o documento.
///
/// ⚠️ Os sete números da fixtura são todos DIFERENTES do seed do `populate` (que é
/// `0/0/100/0/0/0/1`), senão o gate ficaria verde por coincidência.
#[test]
fn the_library_fields_show_what_was_authored_not_the_seed() {
    let (host, _state) = host_with(anim(true, false));
    let got: Vec<f64> = [
        ids::INSP_ANIM_FROM,
        ids::INSP_ANIM_TO,
        ids::INSP_ANIM_FRAME_MS,
        ids::INSP_ANIM_HOLD_MS,
        ids::INSP_ANIM_DELAY_MS,
        ids::INSP_ANIM_REPEAT,
        ids::INSP_ANIM_SPEED,
    ]
    .iter()
    .map(|&id| host.store().number_value(id).unwrap_or(f64::NAN))
    .collect();
    clear();
    let want = [0.0, 3.0, 90.0, 40.0, 250.0, 0.0, 2.5];
    for (g, w) in got.iter().zip(want) {
        assert!(
            (g - w).abs() < 1.0e-4,
            "o campo mostra {g} onde o documento diz {w} — ele é WRITE-ONLY: {got:?}"
        );
    }
}

/// **TODA edição que o modelo declara tem um gesto que a levanta.**
///
/// ⚠️ É a varredura que faltava quando a §11 nasceu, e é a razão de o defeito da caixa ter
/// shipado: a seção tinha gates da lei (no `ph2d-ecs`) e do commit (na shell), e **nenhum** que
/// carregasse num pixel. Um `AnimFieldEdit` novo que ninguém ligue fica exatamente assim —
/// compilando, testado dos dois lados, e inalcançável.
///
/// # ⚠️ O que este gate apanha, e o que NÃO apanha
///
/// O `match` de [`sample`] é exaustivo sobre o enum: uma variante nova **não compila** até alguém
/// lhe dar um braço. A asserção abaixo garante que tudo o que está na lista é **de facto**
/// levantado por um clique real. O ponto cego é a variante que ganha braço no `match` e não entra
/// na lista — por isso o braço e a amostra são o mesmo sítio, e a lista é derivada dele.
#[test]
fn every_edit_the_model_declares_is_reachable_by_a_gesture() {
    use AnimFieldEdit as E;
    // ⚠️ **UMA amostra por variante, e o `match` é o guarda.** A lista abaixo é a fonte; o `match`
    // logo a seguir força o compilador a reprovar uma variante que ninguém amostrou.
    let declared = vec![
        E::Add,
        E::Remove(0),
        E::Rename(0, String::new()),
        E::From(0, 0),
        E::To(0, 0),
        E::FrameMs(0, 0),
        E::HoldMs(0, 0),
        E::DelayMs(0, 0),
        E::Repeat(0, 0),
        E::Direction(0, 0),
        E::SignalOnFinish(0, String::new()),
        E::SignalOnLoop(0, String::new()),
        E::FrameMsAt(0, 0, 0),
        E::AddPlayer,
        E::SetCurrent(String::new()),
        E::Playing(false),
        E::Autoplay(false),
        E::Speed(0.0),
        E::DirectionOverride(0),
        E::LoopOverride(0),
        E::Rewind,
        E::SetFrame(0),
    ];
    fn sample(e: &AnimFieldEdit) -> u8 {
        match e {
            E::Add => 0,
            E::Remove(..) => 1,
            E::Rename(..) => 2,
            E::From(..) => 3,
            E::To(..) => 4,
            E::FrameMs(..) => 5,
            E::HoldMs(..) => 6,
            E::DelayMs(..) => 7,
            E::Repeat(..) => 8,
            E::Direction(..) => 9,
            E::SignalOnFinish(..) => 19,
            E::SignalOnLoop(..) => 20,
            E::FrameMsAt(..) => 21,
            E::AddPlayer => 10,
            E::SetCurrent(..) => 11,
            E::Playing(..) => 12,
            E::Autoplay(..) => 13,
            E::Speed(..) => 14,
            E::DirectionOverride(..) => 15,
            E::LoopOverride(..) => 16,
            E::Rewind => 17,
            E::SetFrame(..) => 18,
        }
    }
    let want: std::collections::BTreeSet<u8> = declared.iter().map(sample).collect();
    assert_eq!(
        want.len(),
        declared.len(),
        "duas amostras da MESMA variante — a lista tem de ter uma por variante"
    );

    // Agora os GESTOS, todos pelo ponteiro real (ou pelo commit real, nos campos de texto).
    let mut raised: std::collections::BTreeSet<u8> = std::collections::BTreeSet::new();
    let mut note = |acts: Vec<EditorAction>| {
        for a in acts {
            if let EditorAction::InspectorAnimEdit { edit, .. } = a {
                raised.insert(sample(&edit));
            }
        }
    };
    for id in [
        ids::INSP_ANIM_ADD,
        ids::INSP_ANIM_REMOVE,
        ids::INSP_ANIM_REWIND,
        ids::INSP_ANIM_PLAYING,
        ids::INSP_ANIM_AUTOPLAY,
        ids::INSP_ANIM_ROW[1],
        ids::INSP_ANIM_DIR[1],
        ids::INSP_ANIM_DIR_OVERRIDE[2],
        ids::INSP_ANIM_LOOP_OVERRIDE[1],
    ] {
        note(click_fresh(anim(true, false), id));
    }
    note(click_fresh(no_player(), ids::INSP_ANIM_ADD_PLAYER));
    note(drag_frame_bar(anim(true, false), 0.9));
    for (id, v) in [
        (ids::INSP_ANIM_FROM, 1.0),
        (ids::INSP_ANIM_TO, 2.0),
        (ids::INSP_ANIM_FRAME_MS, 33.0),
        (ids::INSP_ANIM_HOLD_MS, 10.0),
        (ids::INSP_ANIM_DELAY_MS, 20.0),
        (ids::INSP_ANIM_REPEAT, 2.0),
        (ids::INSP_ANIM_SPEED, 0.5),
        // A duração da célula que a barra mostra (§8.12) — o alvo é a CÉLULA, não a linha.
        (ids::INSP_ANIM_FRAME_MS_THIS, 250.0),
    ] {
        note(commit(anim(true, false), id, v));
    }
    // O NOME e os DOIS NOMES DE SINAL (§8.10), pela porta de texto.
    //
    // ⚠️ Os três pelo mesmo laço porque eles partilham o `TextChanged` — e foi por partilharem que
    // um `if` por campo faria o terceiro chamar o `Rename` do primeiro. O gate percorre-os para que
    // essa troca não possa passar.
    for (id, text) in [
        (ids::INSP_ANIM_NAME, "sprint"),
        (ids::INSP_ANIM_SIGNAL_FINISH, "attack_done"),
        (ids::INSP_ANIM_SIGNAL_LOOP, "footstep"),
    ] {
        let (mut h, mut s) = host_with(anim(true, false));
        h.set_text(id, text);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut s, WidgetEvent::TextChanged(id));
        note(h.drained_actions());
    }
    let (mut host, mut state) = host_with(anim(true, false));
    host.set_text(ids::INSP_ANIM_NAME, "sprint");
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::TextChanged(ids::INSP_ANIM_NAME),
    );
    let out = host.drained_actions();
    clear();
    note(out);

    let missing: Vec<u8> = want.difference(&raised).copied().collect();
    assert!(
        missing.is_empty(),
        "estas edições do modelo não têm gesto que as levante (índices do `sample`): \
         {missing:?} — elas são inalcançáveis no produto"
    );
}

/// **A SELEÇÃO MÚLTIPLA diz-se, e ela EMPURRA o resto da seção.**
///
/// ⚠️ As edições da §11 não se espalham sobre a seleção — o índice que elas carregam só significa
/// alguma coisa na biblioteca da entidade ativa. Sem o aviso, marcar cinco goblins e renomear uma
/// animação muda **um**, em silêncio, e o artista descobre semanas depois.
///
/// ⚠️ **O oráculo é a GEOMETRIA, e não um id**: um aviso é texto, e texto que despacha mente. O
/// que se afirma é o que ele desloca — com dois selecionados, tudo o que vem a seguir desce.
///
/// **Mutação que deve sangrar:** trocar o `info.selected_count > 1` por `false`.
#[test]
fn a_multiple_selection_says_so_before_offering_any_control() {
    let first_control_y = |count: usize| -> f32 {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_anim(Some(InspectorAnimInfo {
            selected_count: count,
            ..anim(true, false)
        }));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        clear();
        rects
            .iter()
            .find(|(n, _)| *n == ids::INSP_ANIM_PLAYING)
            .map(|(_, r)| r.y)
            .expect("a caixa Playing é pintada nos dois casos")
    };
    let one = first_control_y(1);
    let many = first_control_y(2);
    assert!(
        many > one,
        "com dois selecionados a §11 tem de avisar ANTES de oferecer controlo nenhum \
         (Playing ficou em {many} contra {one} — o aviso não foi pintado)"
    );
    // ⚠️ E com UM selecionado o aviso NÃO existe: um painel que avisa sempre não avisa de nada.
    assert!(
        (many - one) > 4.0,
        "o deslocamento tem de ser o de uma linha de texto, e não ruído de layout"
    );
}

/// Arrasta a barra de frames até `frac` do curso e devolve o barramento.
///
/// ⚠️ **Pelo `click_at` REAL**, e não por um `ValueChanged` sintético: é o `pointer_down` do
/// despachante que faz o salto-ao-clique de um `Slider`, e ele só o faz para um id que o store
/// tenha registado **como slider**. Um evento fabricado passaria verde sobre uma barra que continua
/// a ser dois retângulos pintados.
fn drag_frame_bar(info: InspectorAnimInfo, frac: f32) -> Vec<EditorAction> {
    let (mut host, mut state) = host_with(info);
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_ANIM_FRAME_SCRUB)
        .map(|(_, r)| *r)
        .expect("a §11 nunca pintou a barra de frames");
    let events = host.click_at(rect.x + rect.w * frac, rect.y + rect.h * 0.5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::ValueChanged(v) if *v == ids::INSP_ANIM_FRAME_SCRUB)),
        "carregar na barra produziu {events:?} — ela é pintada e hit-registada, mas o store não a \
         tem como Slider: está MORTA sob o rato"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    clear();
    out
}

/// **A BARRA DE FRAMES ARRASTA** (pedido do Enio, 2026-08-23) — e alcança as DUAS pontas.
///
/// ⚠️ **O que este gate mede é a COSTURA, e não a régua** — e a distinção foi paga: a primeira
/// versão dizia que a ponta esquerda apanhava a troca posição↔progresso, e **uma mutação provou o
/// contrário**. O caminho do clique (`x → 0..1 → célula`) não passa pelo pintor, então mudar só o
/// desenho não move nenhuma destas asserções. Quem prende a régua é o
/// `the_scrub_position_and_the_cell_are_inverses`, no modelo; **aqui prende-se que o clique nasce**.
///
/// A fixtura tem `walk` = células 0-3 sobre uma grelha de 8, e o snapshot está no frame 1.
///
/// **Mutação que deve sangrar:** tirar o `register` do `populate_anim` (a barra volta a ser dois
/// retângulos mortos), **ou** trocar o `round` de `scrub_cell` por truncagem.
#[test]
fn the_frame_bar_is_dragged_not_just_looked_at() {
    expect(
        &drag_frame_bar(anim(true, false), 0.0),
        AnimFieldEdit::SetFrame(0),
        "arrastar até ao começo",
    );
    expect(
        &drag_frame_bar(anim(true, false), 1.0),
        AnimFieldEdit::SetFrame(3),
        "arrastar até ao fim",
    );
    // ⚠️ **O ponto do MEIO, e ele não é decorativo:** com `round` a célula muda a meio caminho
    // entre duas, e com `as u32` (truncar) a última só apareceria no pixel final da trilha.
    expect(
        &drag_frame_bar(anim(true, false), 0.5),
        AnimFieldEdit::SetFrame(2),
        "arrastar até ao meio",
    );
}
