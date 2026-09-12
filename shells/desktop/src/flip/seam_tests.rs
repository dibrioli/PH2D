//! **A COSTURA da `App` com a família Flip** — o que ficou deste lado quando ela saiu
//! (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! ⭐ **Os testes seguem o SUJEITO, não o ficheiro** (HOWTO §1.2). Estes quatro viviam nos
//! `*_tests.rs` do `select_segment` e do `colorize`; vieram para cá porque o que eles
//! AFIRMAM é sobre a **`App`** — que o invólucro recusa quando não há `AppGfx`, e que a
//! guarda do hover morre com a troca de modo e com um gesto vivo. A LEI que eles exercitam
//! ficou na crate (`ph2d_app_flip::select_segment` / `::colorize`), e é isso que torna a
//! repartição honesta em vez de arbitrária.
//!
//! ⚠️ **E eles NÃO podem viver em `shells/desktop/tests/it/`**: aquilo é uma crate de
//! integração e o `App::flip_state` é `pub(crate)` — um gate de lá não o alcança. O sítio de
//! um teste que dirige a `App` headless é dentro do `src/`, e é aqui.

use ph2d_core::Vec2;
use ph2d_tool_flip::FlipMode;

// ── O passe de HOVER (§4.C): a GUARDA ────────────────────────────────────────────
//
// O caminho POSITIVO do hover (cursor → pedaço) precisa de `gfx` (câmera/janela) e é
// coberto pelos gates de unidade (`hover_piece`, `piece_halo_path`) + o smoke. O que se
// gateia aqui é a GUARDA — a parte que apodrece: o preview não pode sobreviver a uma troca
// de modo nem competir com um arrasto. O `App` é dirigível sem janela (`crate::App::new()`).
//
// ⚠️ **O observável destes gates é `flip_segment_hover_at`, NÃO `flip_segment_hover`.** Sem
// `gfx` o caminho positivo devolve `None` de qualquer jeito (o pick precisa da câmera),
// então `flip_segment_hover` seria `None` com OU sem a guarda — um gate sobre ele ficaria
// verde com a mutação aplicada ([[feedback_a_green_gate_may_be_green_by_accident]], a
// armadilha que a mutação 2 pegou ao vivo). O que distingue "a guarda BARROU" de "a guarda
// PASSOU" é o carimbo do cursor: a guarda, ao barrar, o zera (`= None`); ao passar, ela o
// estampa (`= Some(cursor)`) antes de tentar o pick. E cada gate deixa `flip_wants_edit()`
// e o domínio de modo que SÓ a condição sob teste decida (senão outra condição limpa por
// ela e o teste não isola nada).

/// Um `App` ARMADO no modo Segment (tool Flip ativa, modo Edit) — o estado em que só a
/// condição sob teste (domínio ≠ Segment, ou gesto ativo) decide se a guarda barra.
fn app_armed_in_segment() -> crate::App {
    let mut app = crate::App::new();
    app.flip_state.active = true;
    app.flip_state.style = Some(ph2d_tool_flip::FlipStyleSnapshot {
        mode: ph2d_tool_flip::FlipMode::Edit,
        edit_domain: ph2d_tool_flip::EditDomain::Segment,
        ..Default::default()
    });
    app.last_pointer = (5.0, 5.0);
    // Um carimbo DIFERENTE do cursor atual, senão a guarda "cursor parado" curto-circuita
    // antes de a condição sob teste ser avaliada.
    app.flip_state.segment_hover_at = Some((1.0, 1.0));
    app
}

/// 🔴 **O hover some fora do modo Segment.** Trocar o domínio para Point/Stroke tem de
/// barrar o preview — senão um pedaço fantasma fica aceso no domínio errado.
///
/// Isola o `!is_segment`: `flip_wants_edit()` fica VERDADEIRO (armado no Edit) e só o
/// domínio muda. Mutação que sangra: tirar o termo `!is_segment` da condição de guarda — a
/// guarda passa em Point e estampa o cursor (`hover_at = Some`).
#[test]
fn the_hover_clears_when_the_domain_is_not_segment() {
    let mut app = app_armed_in_segment();
    app.flip_state.style = Some(ph2d_tool_flip::FlipStyleSnapshot {
        mode: ph2d_tool_flip::FlipMode::Edit,
        edit_domain: ph2d_tool_flip::EditDomain::Point, // ← só isto muda
        ..Default::default()
    });
    app.flip_segment_hover_refresh();
    assert_eq!(
        app.flip_state.segment_hover_at, None,
        "a guarda deixou passar fora do Segment (o cursor foi estampado)"
    );
}

/// 🔴 **O hover não é recomputado durante um GESTO** — armar/arrastar/soltar é o usuário
/// selecionando, não sondando; um preview competiria com o que ele arrasta.
///
/// Isola o `flip_edit_gesture.is_some()`: armado no Segment (as outras condições passam),
/// só o gesto barra. Mutação que sangra: tirar o termo do gesto — a guarda passa e estampa
/// o cursor.
#[test]
fn the_hover_is_suppressed_during_a_gesture() {
    let mut app = app_armed_in_segment();
    app.flip_state.edit_gesture = Some(ph2d_app_flip::edit_gesture::EditGesture::Click);
    app.flip_segment_hover_refresh();
    assert_eq!(
        app.flip_state.segment_hover_at, None,
        "a guarda deixou o hover competir com um gesto ativo (o cursor foi estampado)"
    );
}

/// 🔴 O irmão de PRESENÇA ([[feedback_absence_gate_needs_a_presence_sibling]]): armado no
/// Segment, SEM gesto, com o cursor MOVIDO, a guarda **PASSA** — estampa o cursor e segue
/// para o pick. Sem este gate, uma guarda que barrasse SEMPRE deixaria os dois de cima
/// verdes (o hover nunca competiria porque nunca existiria).
///
/// (`flip_segment_hover` fica `None` aqui — headless não tem `gfx` para o pick —, mas o
/// carimbo do cursor prova que a guarda deixou passar.)
#[test]
fn the_hover_proceeds_when_armed_and_the_cursor_moved() {
    let mut app = app_armed_in_segment();
    app.flip_segment_hover_refresh();
    assert_eq!(
        app.flip_state.segment_hover_at,
        Some((5.0, 5.0)),
        "a guarda barrou um hover legitimo (armado, sem gesto, cursor movido)"
    );
}

/// O rabisco de fixture — duas cores, dois traços fechados.
fn scr(n: u8) -> ([u8; 4], Vec<Vec2>) {
    (
        [n, 0, 0, 255],
        vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, f32::from(n))],
    )
}

#[test]
fn a_refused_apply_keeps_the_scribbles_the_artist_drew() {
    let mut app = crate::App::new();
    app.flip_state.active = true;
    app.flip_state.style = Some(ph2d_tool_flip::FlipStyleSnapshot {
        mode: FlipMode::Colorize,
        ..Default::default()
    });
    for n in [1u8, 2] {
        let (col, pts) = scr(n);
        app.flip_state.colorize.push_scribble(col, pts);
    }
    assert_eq!(app.flip_state.colorize.scribble_count(), 2, "semeado");

    // `App::new()` é headless (`gfx: None`), então o Apply RECUSA na 1ª saída.
    app.flip_colorize_apply();

    assert_eq!(
        app.flip_state.colorize.scribble_count(),
        2,
        "um Apply recusado devolveu {} rabiscos em vez de 2 — o artista perdeu o \
         trabalho e o Ctrl+Z não o traz de volta",
        app.flip_state.colorize.scribble_count()
    );
    // E o Ctrl+Z continua alcançável (o Colorize ainda é dono do atalho).
    assert!(
        app.flip_state.colorize.can_undo_scribble(),
        "com rabisco pendente o Colorize tem de seguir dono do Ctrl+Z"
    );
}
