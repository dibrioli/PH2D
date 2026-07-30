//! **Toda row que a fórmula dobra tem um widget** (FASE C.1 do plano 12) — o U2 da
//! auditoria de 2026-07-29, que é o defeito por trás de *"não tem scroll nem barra de
//! scroll"* e do screenshot com `+1 more rows`.
//!
//! **O mecanismo medido:** `BODY_SLOTS = 12`, uma row custava `1 + knobs`, e quando
//! `used + need > BODY_SLOTS` o pintor imprimia `+N more rows` e fazia **`return`**. Com 4
//! rows de Turbulence as rows 0-1 eram pintadas e as **rows 2-3 ficavam com ZERO widgets**
//! — sem hit rect, sem store, sem clique — *enquanto a fórmula que o objeto roda continha
//! as quatro*. Uma `Fade by Distance` sozinha comia **9 dos 12 slots**.
//!
//! ⚠️ **Por que os gates de row eram todos verdes:** todos usam UMA ou DUAS rows. A fixture
//! não continha o fenômeno, e não havia como ela conter por acidente.
//!
//! **A cura não foi scroll**, e a medição é o argumento: toda row de knob tinha **128 px
//! mortos** (40% do sheet — `ctrl_w` computado como 168 e descartado no braço numérico).
//! Dois knobs numéricos por linha levam o pior caso do catálogo de **5 slots para 3** — 4
//! rows garantidas, 6 no caso típico — sem um 2º eixo de scroll dentro de um painel que já
//! rola.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn publish_one_track(entity: u64, prop: ph2d_timeline::PropKind) -> u64 {
    use ph2d_timeline::{TimelineIntent, TimelineState, TimelineViewSnapshot, apply_intent};
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut st, &mut ph, TimelineIntent::Bind { entity, prop });
    let target = st.doc.binding_for(entity, prop).unwrap().target.get();
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    ph2d_panel_timeline::set_current_timeline(Some(snap));
    target
}

/// Abre o card e empilha `ids`, pela porta que o artista usa (a galeria).
fn card_with(host: &mut MockPanelHost, state: &mut TimelinePanelState, target: u64, ids_: &[&str]) {
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 50.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu();
    host.apply_panel_event::<TimelinePanel>(state, WidgetEvent::Click(ids::CTX_MENU_TL_EXPR));
    for id in ids_ {
        host.apply_panel_event::<TimelinePanel>(
            state,
            WidgetEvent::Click(ids::expr_gallery_id(id)),
        );
    }
}

/// **Toda row do stack tem widgets pintados — e a fórmula contém todas elas.**
///
/// A fixture usa **QUATRO** rows da receita mais cara que o catálogo tem, que é exactamente
/// a forma que os gates de uma-ou-duas-rows não podiam alcançar.
#[test]
fn every_row_the_formula_folds_has_a_widget() {
    let target = publish_one_track(31, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    // `shake` tem 4 knobs (absorveu o Detail/Roughness do turbulence) — a receita mais
    // cara do catálogo, e a que o report do `+1 more rows` usava.
    card_with(
        &mut host,
        &mut state,
        target,
        &["shake", "shake", "shake", "shake"],
    );
    let m = state.expr_modal.as_ref().expect("o card abriu");
    assert_eq!(m.stack.rows.len(), 4, "PREMISSA: quatro rows no stack");
    let formula = m.stack.to_formula();
    assert_eq!(
        formula.matches("wiggle").count(),
        4,
        "PREMISSA: a fórmula que o objeto roda contém as QUATRO rows: {formula}"
    );

    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    for ri in 0..4 {
        // O X de remover é o widget mínimo: sem ele a row é inalcançável, e a única saída
        // do artista é apagar uma row ACIMA dela.
        let rm = ids::expr_remove_id(ri);
        assert!(
            regs.iter().any(|(id, _)| *id == rm),
            "a row {ri} dirige o objeto e não tem um pixel de UI — o `+N more rows` \
             deixava-a assim, e a fórmula continha-a"
        );
        // ...e os knobs dela também.
        for ki in 0..4 {
            let k = ids::expr_knob_id(ri, ki);
            assert!(
                regs.iter().any(|(id, _)| *id == k),
                "o knob {ki} da row {ri} não foi pintado"
            );
        }
    }
}

/// **Os knobs numéricos vêm em DUAS colunas, e a linha não tem mais 128 px mortos.**
///
/// O oráculo é a GEOMETRIA que o paint registrou: dois knobs na mesma faixa de `y` e em `x`
/// diferentes é o que "duas colunas" significa, e nenhuma outra afirmação sobre o layout
/// pode ser feita sem olhar os retângulos.
///
/// **Mutação que deve sangrar:** `knob_slot` devolvendo sempre `col = 0`.
#[test]
fn two_numeric_knobs_share_a_line() {
    let target = publish_one_track(32, ph2d_timeline::PropKind::TranslationY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    card_with(&mut host, &mut state, target, &["shake"]);
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);

    let rect_of = |ki: usize| {
        regs.iter()
            .find(|(id, _)| *id == ids::expr_knob_id(0, ki))
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o knob {ki} não foi pintado"))
    };
    let (a, b) = (rect_of(0), rect_of(1));
    assert_eq!(
        a.y, b.y,
        "os dois primeiros knobs numéricos do Shake têm de dividir a LINHA"
    );
    assert!(
        b.x > a.x + a.w,
        "e sentar em colunas separadas, sem sobrepor: {a:?} vs {b:?}"
    );
    // A segunda coluna tem de usar de fato a metade direita: os 128 px mortos eram lá.
    let sheet_right = a.x + (b.x - a.x) * 2.0;
    assert!(
        b.x + b.w > sheet_right - 100.0,
        "a coluna da direita tem de alcançar a borda do sheet — o defeito era 128 px \
         mortos exactamente aí ({:?})",
        b
    );
    // ...e a 2ª LINHA de knobs desce (o Shake tem 4 knobs = 2 linhas).
    let c = rect_of(2);
    assert!(
        c.y > a.y,
        "o terceiro knob abre uma linha nova: {a:?} vs {c:?}"
    );
    assert_eq!(c.x, a.x, "e volta para a coluna da esquerda");
}

/// **Um knob de TEXTO fica sozinho na linha.**
///
/// ⚠️ A metade que impede a cura de virar o defeito: parear dois campos de texto de 72 px
/// seria trocar um aperto por outro, e um `Link` carrega um NOME (`Ball.x`) enquanto um
/// `Text` carrega uma fórmula.
///
/// **Mutação que deve sangrar:** tratar `Link`/`Text` como numérico em `knob_slot`.
#[test]
fn a_text_knob_gets_the_whole_line() {
    let target = publish_one_track(33, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    // `follow` = 1 Link + 2 numéricos.
    card_with(&mut host, &mut state, target, &["follow"]);
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let rect_of = |ki: usize| {
        regs.iter()
            .find(|(id, _)| *id == ids::expr_knob_id(0, ki))
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o knob {ki} não foi pintado"))
    };
    let (link, mult, off) = (rect_of(0), rect_of(1), rect_of(2));
    assert!(
        mult.y > link.y,
        "o Target é um link: ninguém divide a linha com ele ({link:?} vs {mult:?})"
    );
    assert_eq!(
        mult.y, off.y,
        "e os dois numéricos que sobram dividem a próxima"
    );
    assert!(
        link.w > mult.w,
        "o campo de link é mais largo que uma caixa de coluna: {} vs {}",
        link.w,
        mult.w
    );
}

// ─────────── FASE C.4 — o corpo é função da JANELA, e a row é um cartão ───────────

/// Uma janela pequena o bastante para o card ficar no PISO (12 slots): 564 px é a altura
/// exacta em que ele cabe, então 560 está abaixo dela.
const SMALL_VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 560.0,
};

/// **O corpo cresce com a janela, entre um piso e um teto.**
///
/// ⚠️ O piso é o tamanho que o card SHIPAVA: uma janela pequena não pode ficar pior do que
/// era. O teto tem um recurso nomeado (a galeria, cujo conteúdo máximo é doze slots).
///
/// **Mutação que deve sangrar:** `body_slots` voltar a devolver a constante 12.
#[test]
fn the_body_is_a_function_of_the_window_between_a_floor_and_a_ceiling() {
    use ph2d_panel_timeline::expr_modal_paint::{body_slots, card_h};
    let tiny = Rect {
        h: 200.0,
        ..SMALL_VIEWPORT
    };
    let huge = Rect {
        h: 4000.0,
        ..SMALL_VIEWPORT
    };

    assert_eq!(
        body_slots(SMALL_VIEWPORT),
        12,
        "janela pequena = o corpo que o card sempre teve"
    );
    assert_eq!(
        body_slots(tiny),
        12,
        "e ele NUNCA encolhe abaixo disso — o piso é o que já shipava"
    );
    assert_eq!(
        body_slots(huge),
        20,
        "nem cresce sem teto: um card do tamanho da tela deixa de ler como card"
    );
    assert!(
        body_slots(VIEWPORT) > body_slots(SMALL_VIEWPORT),
        "e ENTRE os dois ele responde à janela: {} vs {}",
        body_slots(VIEWPORT),
        body_slots(SMALL_VIEWPORT)
    );

    // A altura é sempre um número inteiro de faixas — meia faixa no fim do corpo é
    // espaço que nada pode ocupar.
    for vp in [tiny, SMALL_VIEWPORT, VIEWPORT, huge] {
        let body = card_h(vp) - card_h(tiny) + ph2d_tokens::ROW_H_PX * 12.0;
        assert!(
            (body / ph2d_tokens::ROW_H_PX).fract().abs() < 1e-3,
            "o corpo de {vp:?} não é um número inteiro de faixas: {body}"
        );
    }
}

/// **Uma janela mais alta segura MAIS rows do stack — e é isso que a altura compra.**
///
/// O oráculo é o widget, não a aritmética: a row que não cabe é a que fica **sem UI**
/// enquanto a fórmula continua a contendo (o defeito U2 desta suíte). A receita é a mais
/// cara do catálogo (`distance-2d`, 5 slots — quatro knobs de LINK, e um knob largo toma a
/// linha inteira), porque é nela que a capacidade morde.
///
/// **Mutação que deve sangrar:** `body_slots` constante (aí as duas janelas seguram o mesmo).
#[test]
fn a_taller_window_holds_more_rows_of_the_most_expensive_recipe() {
    let target = publish_one_track(34, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    card_with(
        &mut host,
        &mut state,
        target,
        &["distance-2d", "distance-2d", "distance-2d"],
    );
    assert_eq!(
        state
            .expr_modal
            .as_ref()
            .expect("o card abriu")
            .stack
            .rows
            .len(),
        3,
        "PREMISSA: três rows no stack"
    );

    let rows_with_ui = |host: &mut MockPanelHost, state: &mut TimelinePanelState, vp: Rect| {
        let regs = host.paint::<TimelinePanel>(state, vp);
        (0..3)
            .filter(|ri| {
                let rm = ids::expr_remove_id(*ri);
                regs.iter().any(|(id, _)| *id == rm)
            })
            .count()
    };

    let small = rows_with_ui(&mut host, &mut state, SMALL_VIEWPORT);
    let big = rows_with_ui(&mut host, &mut state, VIEWPORT);
    assert_eq!(
        small, 2,
        "no piso, duas rows de `Distance` cabem — o número medido"
    );
    assert_eq!(big, 3, "e uma janela alta segura as três");
}

/// **Dois cartões de row não se encostam.**
///
/// O §5.2 nomeou *sem hierarquia* e *sem respiro* como dois defeitos; eles são um só, e é
/// este: a planilha lia como uma lista plana porque nada separava uma receita da seguinte.
///
/// O oráculo é a GEOMETRIA registrada — o olhinho da row 1 tem de começar depois do fim da
/// banda da row 0, não colado nela.
///
/// **Mutação que deve sangrar:** tirar o `cy += ROW_GAP` do fim do laço.
#[test]
fn two_row_cards_do_not_touch() {
    let target = publish_one_track(35, ph2d_timeline::PropKind::TranslationY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    // Duas rows de `Sway` (3 slots cada: cabeçalho + 2 linhas de knob).
    card_with(&mut host, &mut state, target, &["sway", "sway"]);
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let eye = |ri: usize| {
        regs.iter()
            .find(|(id, _)| *id == ids::expr_bypass_id(ri))
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o olhinho da row {ri} foi pintado"))
    };
    let (a, b) = (eye(0), eye(1));
    let band = ph2d_tokens::ROW_H_PX * 3.0;
    assert!(
        b.y > a.y + band,
        "a row 1 tem de começar DEPOIS da banda da row 0, com respiro: {a:?} vs {b:?}"
    );
    assert!(
        b.y - (a.y + band) < ph2d_tokens::ROW_H_PX,
        "e o respiro é uma calha, não uma faixa vazia: {}",
        b.y - (a.y + band)
    );
}

/// **O que o pintor admite CABE no corpo — a calha entre cartões entra na conta.**
///
/// ⚠️ O orçamento é em PIXELS e não em slots porque `ROW_GAP` não é múltiplo de
/// `ROW_H_PX`: contado em slots, o respiro fica de fora da aritmética, o pintor admite
/// uma row a mais e ela é desenhada **por baixo da fita**. É a mesma família do defeito
/// U2 (a fórmula contém o que a tela não mostra), só que pelo outro lado.
///
/// ⚠️ **A fixture TEM de conter o fenômeno, e a primeira não continha:** com `Sway`
/// (3 slots = 84 px) o corte cai no mesmo lugar com e sem a calha (6 rows nos dois casos),
/// então a mutação SOBREVIVIA. A calha só morde quando o acúmulo dela cruza uma faixa: com
/// uma receita de **2 slots** (56 px) o corpo de 560 px admite 9 rows cobrando a calha e
/// **10** sem cobrar — e essas dez são desenhadas em 596 px, 36 px por baixo da fita.
///
/// **Mutação que deve sangrar:** `need_px` sem o `+ ROW_GAP`.
#[test]
fn what_the_painter_admits_fits_inside_the_body() {
    let target = publish_one_track(36, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let ten = ["jitter"; 12];
    card_with(&mut host, &mut state, target, &ten);
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);

    let eyes: Vec<Rect> = (0..ten.len())
        .filter_map(|ri| {
            regs.iter()
                .find(|(id, _)| *id == ids::expr_bypass_id(ri))
                .map(|(_, r)| *r)
        })
        .collect();
    assert!(eyes.len() >= 2, "PREMISSA: o card admitiu mais de uma row");

    let band = ph2d_tokens::ROW_H_PX * 2.0; // `Jitter` = cabeçalho + 1 linha de knob
    let span = eyes.last().expect("há rows").y + band - eyes[0].y;
    let budget =
        ph2d_tokens::ROW_H_PX * ph2d_panel_timeline::expr_modal_paint::body_slots(VIEWPORT) as f32;
    assert!(
        span <= budget,
        "o pintor admitiu {} rows, que ocupam {span} px num corpo de {budget} px — a \
         última é desenhada por baixo da fita",
        eyes.len()
    );
}
