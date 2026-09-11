//! Seam do **AUTO LAYOUT** (plano UI/UX W2, ADR-0153) — os chips estão vivos sob o MOUSE, os
//! campos existem, e o que **não se aplica não é pintado**.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a lacuna que já deixou as 36 células da matriz de física e os dez chips de ferramenta
//! do Painter *pintados, hit-registrados e mortos sob o ponteiro*.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{LayoutFlow, LayoutItem, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

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

/// Um fluxo em LINHA, tudo no neutro — a publicação que faz a seção existir.
fn row_flow() -> LayoutFlow {
    LayoutFlow {
        size: [
            ids::VECTOR_LAYOUT_SIZE_W_FIXED,
            ids::VECTOR_LAYOUT_SIZE_H_FIXED,
        ],
        min: [0.0; 2],
        max: [0.0; 2],
        dir: ids::VECTOR_LAYOUT_DIR_ROW,
        gap: [0.0, 0.0],
        pad: [0.0; 4],
        align: ids::VECTOR_LAYOUT_ALIGN_START,
        justify: ids::VECTOR_LAYOUT_JUSTIFY_START,
        columns: 2.0,
    }
}

/// A MESMA moldura, em grade — o que muda é a direção, e é isso que faz a fileira do Justify
/// encolher e a de colunas nascer.
fn grid_flow() -> LayoutFlow {
    LayoutFlow {
        dir: ids::VECTOR_LAYOUT_DIR_GRID,
        ..row_flow()
    }
}

fn clear() {
    state::set_frame_clip(None);
    state::set_frame_present(false);
    state::set_layout_flow(None);
    state::set_layout_item(None);
}

/// Clica de verdade no widget `id` e exige que o Click chegue ao barramento.
fn click_reaches_bus(id: ph2d_a11y::NodeId, what: &str) {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
        )),
        "o Click de {what} nao chegou ao bus — ele acende sob o mouse e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}

/// O widget `id` foi pintado com área clicável?
fn painted(id: ph2d_a11y::NodeId) -> bool {
    rect(id).is_some()
}

/// O retângulo em que o painel pintou `id`.
fn rect(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
}

fn rect_of(id: ph2d_a11y::NodeId, what: &str) -> Rect {
    rect(id).unwrap_or_else(|| panic!("{what} nao foi pintado"))
}

/// A calha de rótulo que a versão anterior dava a TODO campo: um caractere (`Spacing::Md`) mais
/// o vão até o campo (`Spacing::Xs`). Literal aqui de propósito — é o número do DEFEITO, e o
/// gate tem de continuar a falhar se alguém reintroduzir a constante com outro nome.
const OLD_FIXED_GUTTER_PX: f32 = 8.0 + 4.0;

/// **O REPRO da label sobreposta** (Enio 2026-08-02: *"caixas de input numérico grande e label
/// sobreposta"*).
///
/// A calha era fixa em oito pixels — um caractere —, então `paint_text` recortava "Gap" em "G"
/// e o campo era desenhado por cima do resto. Nasceu VERMELHO em exactamente `12.0`.
#[test]
fn a_multi_letter_label_gets_more_room_than_one_character() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    let inner_x = rect_of(ids::VECTOR_LAYOUT_DIR_OFF, "o chip Off").x;
    let gap = rect_of(ids::VECTOR_LAYOUT_GAP_MAIN, "o campo Gap");
    assert!(
        gap.x - inner_x > OLD_FIXED_GUTTER_PX,
        "a calha do rotulo 'Gap' e' de {:.1} px — o campo comeca em cima do proprio rotulo",
        gap.x - inner_x
    );
    clear();
}

/// **E a calha SEGUE o rótulo** — um rótulo mais largo empurra o campo mais para a direita.
///
/// ⚠️ É a metade que distingue *medir* de *escolher uma constante maior*: com um número fixo
/// (qualquer que seja) os dois campos começariam no MESMO x, e o próximo rótulo mais longo que
/// ele voltaria a ser recortado.
#[test]
fn a_wider_label_pushes_its_field_further_right() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    state::set_layout_item(Some(LayoutItem {
        absolute: false,
        in_flow: true,
        grow: 0.0,
        shrink: 1.0,
        parent_is_grid: false,
    }));
    // "Gap" e "Grow" são os dois campos da coluna ESQUERDA — mesma origem de célula, então o x
    // deles é a calha e mais nada.
    let gap = rect_of(ids::VECTOR_LAYOUT_GAP_MAIN, "o campo Gap");
    let grow = rect_of(ids::VECTOR_LAYOUT_ITEM_GROW, "o campo Grow");
    assert!(
        grow.x > gap.x,
        "'Grow' e 'Gap' comecam no mesmo x ({:.1}) — a calha e' uma constante, nao a medida",
        gap.x
    );
    clear();
}

/// **Um campo SOZINHO ocupa meia largura, não a linha inteira** (a outra metade do report: o
/// *"input numérico grande"*).
///
/// O oráculo é a borda direita do painel, lida do último chip da fileira de direção — nenhum
/// número escrito à mão, então ele continua certo se a largura do painel mudar.
#[test]
fn a_lone_number_field_sits_in_half_the_row() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    let wrap = rect_of(ids::VECTOR_LAYOUT_DIR_WRAP, "o chip Wrap");
    let inner_right = wrap.x + wrap.w;
    let gap = rect_of(ids::VECTOR_LAYOUT_GAP_MAIN, "o campo Gap");
    let right = gap.x + gap.w;
    assert!(
        right < inner_right - gap.w * 0.5,
        "o campo Gap vai ate' {right:.1} e a linha acaba em {inner_right:.1} — ele tomou a \
         largura inteira do painel"
    );
    clear();
}

/// **Os quatro chips de DIREÇÃO estão vivos numa moldura**, incluindo o Off.
#[test]
fn the_four_direction_chips_are_reachable_and_reach_the_bus() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    for (id, what) in [
        (ids::VECTOR_LAYOUT_DIR_OFF, "o chip Off"),
        (ids::VECTOR_LAYOUT_DIR_ROW, "o chip Row"),
        (ids::VECTOR_LAYOUT_DIR_COL, "o chip Column"),
        (ids::VECTOR_LAYOUT_DIR_WRAP, "o chip Wrap"),
        (ids::VECTOR_LAYOUT_DIR_GRID, "o chip Grid"),
    ] {
        click_reaches_bus(id, what);
    }
    clear();
}

/// **O COMMIT de um campo numérico chega ao barramento** — e não o clique.
///
/// ⚠️ A distinção não é cerimónia: um número mora no COMPONENTE, então o que a shell precisa de
/// ouvir é o `ValueChanged` do commit, não o `Click` do foco. É a cicatriz do Z-index, escrita no
/// `event.rs` — *"o campo era pintado, registado e vivo sob o mouse; o artista clicava, digitava,
/// via o número mudar, e o commit caía no catch-all"*. Um gate que medisse o clique aqui ficaria
/// **VERDE sobre exactamente esse defeito**.
fn commit_reaches_bus(id: ph2d_a11y::NodeId, what: &str) {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    host.apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::ValueChanged(id));
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, _)) if c == id
        )),
        "o commit de {what} nao chegou ao bus — o campo aceita teclas e nao fala com ninguem          (falta a lista no event.rs)"
    );
}

/// **A contagem de colunas nasce com o modo que a lê** — a mesma cerca do vão transversal.
///
/// ⚠️ E o campo tem de estar VIVO sob o mouse, não só pintado: um id registado que ninguém pode
/// clicar é o defeito que este arquivo existe para pegar, e ele já apareceu três vezes neste
/// painel.
#[test]
fn the_column_count_is_born_with_the_grid_and_reaches_the_bus() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    assert!(
        !painted(ids::VECTOR_LAYOUT_COLUMNS),
        "a contagem de colunas foi pintada numa LINHA, onde ela nao move um pixel"
    );
    state::set_layout_flow(Some(grid_flow()));
    assert!(
        painted(ids::VECTOR_LAYOUT_COLUMNS),
        "a grade sem o numero de colunas e' uma grade que o artista nao consegue descrever"
    );
    commit_reaches_bus(ids::VECTOR_LAYOUT_COLUMNS, "o campo de colunas");
    clear();
}

/// ⭐ **As duas DISTRIBUIÇÕES não são oferecidas na grade** — com colunas iguais não sobra espaço
/// para repartir, e dois chips que não movem um pixel são o item-de-menu-morto que esta política
/// existe para impedir.
///
/// ⚠️ O gate afirma as DUAS metades: elas somem na grade **e continuam lá** na linha. Só a
/// primeira seria satisfeita por as apagar de vez.
#[test]
fn the_two_distributions_are_not_offered_in_a_grid_but_survive_in_a_row() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    for id in [
        ids::VECTOR_LAYOUT_JUSTIFY_BETWEEN,
        ids::VECTOR_LAYOUT_JUSTIFY_AROUND,
    ] {
        state::set_layout_flow(Some(row_flow()));
        assert!(
            painted(id),
            "a distribuicao sumiu de uma LINHA, onde ela e' viva"
        );
        state::set_layout_flow(Some(grid_flow()));
        assert!(
            !painted(id),
            "a distribuicao foi oferecida numa GRADE, onde ela nao pode fazer nada"
        );
    }
    // Os três que ficam continuam vivos — a fileira encolhe, não desaparece.
    state::set_layout_flow(Some(grid_flow()));
    for id in [
        ids::VECTOR_LAYOUT_JUSTIFY_START,
        ids::VECTOR_LAYOUT_JUSTIFY_CENTER,
        ids::VECTOR_LAYOUT_JUSTIFY_END,
    ] {
        assert!(painted(id), "a grade perdeu um chip que ela HONRA");
    }
    clear();
}

/// **Os nove chips de alinhamento/distribuição estão vivos com a moldura FLUINDO.**
#[test]
fn the_alignment_chips_are_reachable_and_reach_the_bus() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    for id in [
        ids::VECTOR_LAYOUT_ALIGN_START,
        ids::VECTOR_LAYOUT_ALIGN_CENTER,
        ids::VECTOR_LAYOUT_ALIGN_END,
        ids::VECTOR_LAYOUT_ALIGN_STRETCH,
        ids::VECTOR_LAYOUT_JUSTIFY_START,
        ids::VECTOR_LAYOUT_JUSTIFY_CENTER,
        ids::VECTOR_LAYOUT_JUSTIFY_END,
        ids::VECTOR_LAYOUT_JUSTIFY_BETWEEN,
        ids::VECTOR_LAYOUT_JUSTIFY_AROUND,
    ] {
        click_reaches_bus(id, "um chip de alinhamento");
    }
    clear();
}

/// **Com a moldura PARADA só a fileira de direção é pintada.**
///
/// ⚠️ É a metade que impede cinco controles que não mudam um pixel: vão, recuo, alinhamento e
/// distribuição sobre uma moldura que não empilha não têm o que fazer.
#[test]
fn a_frame_that_does_not_flow_paints_only_the_direction_row() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    assert!(
        painted(ids::VECTOR_LAYOUT_DIR_OFF),
        "a direcao e' oferecida"
    );
    for id in [
        ids::VECTOR_LAYOUT_GAP_MAIN,
        ids::VECTOR_LAYOUT_PAD_ALL,
        ids::VECTOR_LAYOUT_ALIGN_START,
        ids::VECTOR_LAYOUT_JUSTIFY_START,
    ] {
        assert!(
            !painted(id),
            "um controle de fluxo foi pintado numa moldura que nao flui"
        );
    }
    clear();
}

/// **O par All/Each TROCA os campos pintados** — nunca quatro campos espelhando um número.
#[test]
fn the_padding_mode_swaps_which_fields_are_painted() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));

    // Modo *All* (o default): um campo, e os quatro lados NÃO estão na tela.
    assert!(painted(ids::VECTOR_LAYOUT_PAD_ALL));
    assert!(!painted(ids::VECTOR_LAYOUT_PAD_T));

    // O chip Each é panel-local: ele muda o que é pintado sem passar pela shell.
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_LAYOUT_PAD_EACH_MODE)
        .expect("o chip Each e' pintado");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_LAYOUT_PAD_EACH_MODE)),
        "o chip Each esta' morto sob o mouse"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }

    for id in [
        ids::VECTOR_LAYOUT_PAD_T,
        ids::VECTOR_LAYOUT_PAD_R,
        ids::VECTOR_LAYOUT_PAD_B,
        ids::VECTOR_LAYOUT_PAD_L,
    ] {
        assert!(painted(id), "os quatro lados tem de aparecer no modo Each");
    }
    assert!(
        !painted(ids::VECTOR_LAYOUT_PAD_ALL),
        "o campo unico tem de SAIR — dois campos para o mesmo numero nao dizem em qual se digita"
    );

    // Volta ao *All*, senão o modo vaza para os outros gates deste binário (é thread-local).
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_LAYOUT_PAD_ALL_MODE)
        .expect("o chip All e' pintado");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    clear();
}

/// **O vão TRANSVERSAL só existe no `Wrap`** — em linha ou coluna há uma faixa só, e não há entre
/// o que ele ficaria.
#[test]
fn the_cross_gap_is_born_with_the_mode_that_uses_it() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    assert!(painted(ids::VECTOR_LAYOUT_GAP_MAIN));
    assert!(
        !painted(ids::VECTOR_LAYOUT_GAP_CROSS),
        "o vao entre FAIXAS foi pintado numa linha unica"
    );

    state::set_layout_flow(Some(LayoutFlow {
        size: [
            ids::VECTOR_LAYOUT_SIZE_W_FIXED,
            ids::VECTOR_LAYOUT_SIZE_H_FIXED,
        ],
        min: [0.0; 2],
        max: [0.0; 2],
        dir: ids::VECTOR_LAYOUT_DIR_WRAP,
        ..row_flow()
    }));
    assert!(painted(ids::VECTOR_LAYOUT_GAP_CROSS));
    clear();
}

/// **Grow/Shrink seguem o FILHO, não a moldura** — e os dois blocos COEXISTEM.
#[test]
fn the_item_rows_follow_the_child_and_coexist_with_the_frame_block() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    assert!(
        !painted(ids::VECTOR_LAYOUT_ITEM_GROW),
        "Grow apareceu sem o filho estar num fluxo"
    );

    state::set_layout_item(Some(LayoutItem {
        absolute: false,
        in_flow: true,
        grow: 0.0,
        shrink: 0.0,
        parent_is_grid: false,
    }));
    assert!(painted(ids::VECTOR_LAYOUT_ITEM_GROW));
    assert!(painted(ids::VECTOR_LAYOUT_ITEM_SHRINK));
    // A moldura ANINHADA: os dois blocos ao mesmo tempo.
    assert!(painted(ids::VECTOR_LAYOUT_DIR_ROW));
    clear();
}

/// **Sem moldura E sem filho de fluxo a seção não existe.**
#[test]
fn the_layout_section_is_absent_without_a_subject() {
    clear();
    for id in [
        ids::VECTOR_LAYOUT_DIR_OFF,
        ids::VECTOR_LAYOUT_DIR_ROW,
        ids::VECTOR_LAYOUT_GAP_MAIN,
        ids::VECTOR_LAYOUT_ITEM_GROW,
    ] {
        assert!(
            !painted(id),
            "a secao Layout foi pintada sem moldura nem filho de fluxo"
        );
    }
}

/// **Só o filho selecionado: a seção existe com o bloco de ITEM sozinho.**
///
/// ⚠️ Sem esta metade o artista que seleciona uma forma dentro de uma moldura não teria onde
/// escrever o Grow — a moldura não está selecionada, então o outro bloco não aparece.
#[test]
fn a_selected_child_alone_still_gets_its_two_rows() {
    clear();
    state::set_layout_item(Some(LayoutItem {
        absolute: false,
        in_flow: true,
        grow: 1.0,
        shrink: 0.0,
        parent_is_grid: false,
    }));
    assert!(painted(ids::VECTOR_LAYOUT_ITEM_GROW));
    assert!(
        !painted(ids::VECTOR_LAYOUT_DIR_OFF),
        "o bloco da MOLDURA nao pode aparecer sem moldura selecionada"
    );
    clear();
}

/// **UMA FILEIRA QUE NÃO CABE QUEBRA EM LINHAS** (Enio 2026-08-02: *"botões não se adaptam à
/// largura do painel e se amontoam (veja Between e Around)"*).
///
/// A fileira de distribuição tem CINCO opções, e os rótulos `Between`/`Around` não cabem num
/// quinto da largura do painel: eles eram espremidos e o texto invadia o vizinho.
///
/// ⚠️ **O par é o que torna o gate honesto:** a de distribuição (5) tem de quebrar, e a de
/// alinhamento (4) tem de continuar numa linha só. Um gate que só exigisse a quebra passaria com
/// um refluxo que quebra TUDO, e o painel inteiro viraria uma coluna de botões.
#[test]
fn a_row_that_does_not_fit_wraps_and_one_that_fits_does_not() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));

    let first = rect_of(
        ids::VECTOR_LAYOUT_JUSTIFY_START,
        "o chip Start da distribuição",
    );
    let last = rect_of(ids::VECTOR_LAYOUT_JUSTIFY_AROUND, "o chip Around");
    assert!(
        last.y > first.y,
        "os cinco chips ficaram na MESMA linha ({:.1}) — 'Between' e 'Around' espremidos",
        first.y
    );

    let a_first = rect_of(
        ids::VECTOR_LAYOUT_ALIGN_START,
        "o chip Start do alinhamento",
    );
    let a_last = rect_of(ids::VECTOR_LAYOUT_ALIGN_STRETCH, "o chip Stretch");
    assert!(
        (a_last.y - a_first.y).abs() < 0.5,
        "a fileira de QUATRO quebrou sem precisar: {:.1} vs {:.1}",
        a_first.y,
        a_last.y
    );
    clear();
}

/// **Os quatro chips de TAMANHO estão vivos sob o mouse e chegam ao barramento.**
#[test]
fn the_size_chips_are_reachable_and_reach_the_bus() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    for (id, what) in [
        (ids::VECTOR_LAYOUT_SIZE_W_FIXED, "Width: Fixed"),
        (ids::VECTOR_LAYOUT_SIZE_W_HUG, "Width: Hug"),
        (ids::VECTOR_LAYOUT_SIZE_H_FIXED, "Height: Fixed"),
        (ids::VECTOR_LAYOUT_SIZE_H_HUG, "Height: Hug"),
    ] {
        click_reaches_bus(id, what);
    }
    clear();
}

/// **O toggle do fora-do-fluxo está vivo, e ESCONDE Grow/Shrink quando marcado.**
///
/// ⚠️ As duas metades no mesmo gate de propósito: um filho absoluto não reparte sobra nenhuma, e
/// oferecer-lhe os dois números seria o controlo morto que esta política existe para impedir. O
/// gate que só clicasse o toggle ficaria verde com os dois campos pintados ao lado.
#[test]
fn the_absolute_toggle_is_live_and_hides_grow_and_shrink() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    state::set_layout_item(Some(LayoutItem {
        grow: 0.0,
        shrink: 0.0,
        absolute: false,
        in_flow: true,
        parent_is_grid: false,
    }));
    click_reaches_bus(ids::VECTOR_LAYOUT_ITEM_ABSOLUTE, "o toggle Absolute");
    // No fluxo, os dois números existem.
    assert!(
        painted(ids::VECTOR_LAYOUT_ITEM_GROW),
        "no fluxo, Grow tem de ser pintado"
    );
    // Fora do fluxo, não.
    state::set_layout_item(Some(LayoutItem {
        grow: 0.0,
        shrink: 0.0,
        absolute: true,
        in_flow: true,
        parent_is_grid: false,
    }));
    assert!(
        !painted(ids::VECTOR_LAYOUT_ITEM_GROW) && !painted(ids::VECTOR_LAYOUT_ITEM_SHRINK),
        "um filho ABSOLUTO nao reparte sobra: os dois campos nao podem ser pintados"
    );
    assert!(
        painted(ids::VECTOR_LAYOUT_ITEM_ABSOLUTE),
        "e o proprio toggle TEM de continuar pintado — senao nao ha como desmarcar"
    );
    clear();
}

/// **Os quatro limites existem com a moldura a fluir.**
#[test]
fn the_four_bounds_are_painted_when_the_frame_flows() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    for (id, what) in [
        (ids::VECTOR_LAYOUT_MIN_W, "Min W"),
        (ids::VECTOR_LAYOUT_MAX_W, "Max W"),
        (ids::VECTOR_LAYOUT_MIN_H, "Min H"),
        (ids::VECTOR_LAYOUT_MAX_H, "Max H"),
    ] {
        assert!(painted(id), "{what} nao foi pintado");
    }
    clear();
}

/// **Sem fluxo no pai, o bloco do filho EXPLICA-SE em vez de sumir.**
///
/// ⚠️ É a cura do report do Enio no smoke da cena `=66` (*"não achei Absolute Position no painel
/// do quadrado âmbar"*): o toggle é escondido quando o pai não empilha — e escondê-lo **em
/// silêncio** deixava o artista a olhar para um painel que não dizia o que faltava.
///
/// As duas metades no mesmo gate, porque uma sem a outra é um defeito: **nenhum controlo é
/// oferecido** (eles não fariam nada) **e alguma coisa é pintada** (senão é o silêncio de volta).
#[test]
fn the_item_block_explains_itself_when_the_parent_does_not_flow() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    state::set_layout_flow(Some(row_flow()));
    state::set_layout_item(Some(LayoutItem {
        grow: 0.0,
        shrink: 0.0,
        absolute: false,
        in_flow: false,
        parent_is_grid: false,
    }));
    assert!(
        !painted(ids::VECTOR_LAYOUT_ITEM_ABSOLUTE)
            && !painted(ids::VECTOR_LAYOUT_ITEM_GROW)
            && !painted(ids::VECTOR_LAYOUT_ITEM_SHRINK),
        "sem fluxo no pai nenhum dos tres faz nada — nenhum pode ser oferecido"
    );
    // A outra metade: a seção EXISTE (o cabeçalho foi pintado), que é o que separa *explicar* de
    // *sumir*. Sem fluxo E sem moldura selecionada, ela seria a única coisa na seção.
    assert!(
        painted(ids::VECTOR_SECTION_LAYOUT),
        "a secao tem de existir para poder explicar"
    );
    clear();
}
