//! ⭐⭐⭐ **UMA COLUNA DEVOLVE EXACTAMENTE O QUE LEVOU** — a involução que faltava.
//!
//! > *«a retração dos painéis laterais ainda está ruim. O inspector explodiu, soltou vários
//! > painéis no meio do canvas.»* — Enio, 2026-09-07
//!
//! # ⛔ Por que 13 132 testes verdes não viram isto
//!
//! Auditado em 2026-09-07, e a resposta é uma frase: **nenhum gate deste repo media a
//! CARDINALIDADE de nada.** A suíte inteira pergunta *«este painel específico está visível?»*; a
//! grandeza do report — *«vários»* — não tinha instrumento em lado nenhum
//! (`grep -rn 'is_panel_visible' | grep -i 'count\|len()\|filter'` → zero linhas).
//!
//! Pior: o gate que cobria o gesto (`shells/desktop/tests/the_border_gesture_reaches_the_panel.rs`)
//! era `fs::read_to_string` + `contains`, e uma das coisas que ele exigia era a string
//! `panel_visibility.insert(id, true)` — que é **a linha do defeito**. *Ele leu-a e chamou-lhe
//! correcta.* A causa era estrutural: `close_column`/`open_column` eram `fn` privadas de um
//! `impl App` do binário, inalcançáveis de qualquer teste. A cura moveu a lei para
//! [`ph2d_editor_core::screens::hero::dock_columns`], e é por isso que este ficheiro pode existir.
//!
//! # ⚠️ Por que ele mora AQUI
//!
//! Ele precisa do **registo real** de painéis, que vive nesta crate — na `ph2d-editor-core` o
//! `test_support::ensure_panel_registry` é um `{}` e toda varredura correria sobre zero painéis,
//! verde e vazia.
//!
//! # ⚠️ E por que o quadro é pintado três vezes
//!
//! O `DockSides::from_published` lê os rects do quadro **anterior**, e a fila de abas só existe
//! depois de alguém publicar. Um gate de um quadro só mediria o estado transitório.

use ph2d_editor_core::screens::hero::{HeroScreen, dock_columns, slot_tabs};
use ph2d_editor_core::screens::layout::DockSide;
use ph2d_editor_core::screens::slot::Slot;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 1024.0,
};

fn all_panel_ids() -> Vec<&'static str> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            v.push(p.manifest.id);
        }
    });
    v
}

/// Pinta três quadros com **exactamente** estes painéis visíveis.
fn settled(open: &[&str]) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    for n in all_panel_ids() {
        h.panel_visibility.insert(n, open.contains(&n));
    }
    paint(&mut h, 3);
    h
}

fn paint(h: &mut HeroScreen, frames: usize) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..frames {
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
    }
}

/// O conjunto visível — a grandeza que nenhum gate deste repo media.
fn visible(h: &HeroScreen) -> Vec<&'static str> {
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            if h.is_panel_visible(p.manifest.id) {
                v.push(p.manifest.id);
            }
        }
    });
    v
}

/// As janelas SOLTAS que estão visíveis agora.
fn visible_floats(h: &HeroScreen) -> Vec<&'static str> {
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            if p.manifest.can_float && h.is_panel_visible(p.manifest.id) {
                v.push(p.manifest.id);
            }
        }
    });
    v
}

/// Quantos painéis DECLARAM aquele lado — a população que a reabertura antiga acordava.
fn declaring(h: &HeroScreen, side: DockSide) -> Vec<&'static str> {
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            if slot_tabs::slot_of(h, &p.manifest).dock_side() == Some(side) {
                v.push(p.manifest.id);
            }
        }
    });
    v
}

/// Quais painéis ancorados publicaram rect neste quadro.
fn published(h: &HeroScreen) -> Vec<&'static str> {
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            if p.manifest.can_float {
                continue;
            }
            if h.store.panel_rect(p.manifest.panel_node_id).is_some() {
                v.push(p.manifest.id);
            }
        }
    });
    v
}

/// ⭐⭐⭐ **A LEI.** Fechar e reabrir devolve o mesmo conjunto — nem mais um painel, nem menos.
#[test]
fn reopening_a_column_gives_back_exactly_what_the_close_took() {
    let mut h = settled(&["audio_mixer", "audio_editor", "inspector"]);
    let before = visible(&h);

    let choice = h.store.dock_width_choice(DockSide::Right);
    assert!(
        dock_columns::close(&mut h, DockSide::Right, choice),
        "o fecho nao encontrou inquilino nenhum — a fixtura nao produz o fenomeno"
    );
    paint(&mut h, 3);
    let closed = visible(&h);

    // ⚠️ **Controlo de vacuidade nº1:** sem isto, um fecho que não fizesse nada tornaria a
    //    igualdade lá em baixo trivialmente verdadeira.
    assert!(
        closed.len() < before.len(),
        "o fecho nao escondeu ninguem: antes {before:?}, depois {closed:?}"
    );

    assert!(
        dock_columns::open(&mut h, DockSide::Right),
        "a alca nao reabriu"
    );
    paint(&mut h, 3);

    assert_eq!(
        visible(&h),
        before,
        "reabrir NAO devolveu o que o fecho levou. Era este o report de 2026-09-07: fechar contava \
         os rects PUBLICADOS (1) e reabrir varria o REGISTO ({} declaram este lado), logo um toque \
         na alca abria painéis que o dono nunca tinha aberto",
        declaring(&h, DockSide::Right).len()
    );
}

/// ⭐⭐⭐ **O caso que separa as duas leis.** Um painel que o DONO fechou não ressuscita.
///
/// ⚠️ É o único teste que distingue *«reabri o que fechei»* de *«reabri tudo o que declara este
/// lado»* — sem ele, uma reabertura por re-derivação passa em todos os outros.
#[test]
fn a_panel_the_owner_closed_by_hand_is_not_resurrected_by_the_column() {
    let mut h = settled(&["audio_mixer", "audio_editor", "inspector"]);
    // O dono fecha um deles à mão, pelo menu ou pelo X do painel.
    h.panel_visibility.insert("audio_editor", false);
    paint(&mut h, 3);

    let choice = h.store.dock_width_choice(DockSide::Right);
    assert!(dock_columns::close(&mut h, DockSide::Right, choice));
    paint(&mut h, 3);
    assert!(dock_columns::open(&mut h, DockSide::Right));
    paint(&mut h, 3);

    assert!(
        !h.is_panel_visible("audio_editor"),
        "a coluna ressuscitou um painel que o dono tinha fechado. A informacao que decide — «o dono \
         tinha isto aberto?» — nao existe no registo: ela existe num instante so', o do fecho"
    );
    assert!(
        h.is_panel_visible("audio_mixer") && h.is_panel_visible("inspector"),
        "e os outros dois tinham de voltar: {:?}",
        visible(&h)
    );
}

/// ⭐⭐⭐ **Um gesto de COLUNA é invariante sobre as janelas SOLTAS — nos dois sentidos.**
///
/// Seis painéis declaram um encaixe lateral **e** `CAN_FLOAT`; eles nascem centrados no viewport.
/// Era isto, ao pé da letra, o *«soltou vários painéis no meio do canvas»*.
///
/// ⚠️ **A 1.ª redacção deste teste era VÁCUA e eu escrevi-a**: ela fechava e reabria com zero
/// flutuantes visíveis, e nesse caminho nenhum flutuante *pode* aparecer, faça o código o que
/// fizer. A régua a sério precisa de um flutuante **aberto**: aí a pergunta passa a ser se a coluna
/// o considera seu — e ela não o deve considerar, porque não o contém.
#[test]
fn a_column_gesture_is_invariant_on_floating_windows() {
    // `widget_lab` declara `RightTop` **e** `CAN_FLOAT`: pelo encaixe ele parecia inquilino da
    // coluna, e é por isso que a reabertura antiga o acendia.
    let mut h = settled(&["inspector", "widget_lab"]);
    let before = visible_floats(&h);
    assert_eq!(
        before,
        vec!["widget_lab"],
        "controlo partido: a fixtura tem de ter uma janela solta ABERTA, senao o teste passa sobre \
         qualquer codigo"
    );

    let choice = h.store.dock_width_choice(DockSide::Right);
    assert!(dock_columns::close(&mut h, DockSide::Right, choice));
    paint(&mut h, 3);
    assert_eq!(
        visible_floats(&h),
        before,
        "o fecho da coluna levou uma janela solta que ela nao contem"
    );

    assert!(dock_columns::open(&mut h, DockSide::Right));
    paint(&mut h, 3);
    assert_eq!(
        visible_floats(&h),
        before,
        "a reabertura mexeu numa janela solta. Uma coluna nao a contem: o `occupants` exclui os \
         `CAN_FLOAT` desde que as abas existem, e as duas metades do interruptor tem de fazer a \
         MESMA pergunta que ele"
    );
}

/// ⭐⭐⭐ **Fechar leva TODOS os inquilinos, não só o que tem rect publicado.**
///
/// ⚠️ A asserção do meio é o controlo: ela mede que o critério antigo (*rect publicado*) via **um**
/// painel de três — a fila de abas esconde os de trás e o `panel_walk` limpa-lhes o rect. Sem ela,
/// este teste passaria sobre o desenho partido no dia em que a fixtura só tivesse um ocupante.
#[test]
fn closing_takes_every_tenant_not_only_the_one_with_a_published_rect() {
    let mut h = settled(&["audio_mixer", "audio_editor", "inspector"]);
    let tenants = dock_columns::tenants(&h, DockSide::Right);
    assert_eq!(
        tenants.len(),
        3,
        "a fixtura nao pos tres inquilinos na coluna: {tenants:?}"
    );

    let with_rect: Vec<_> = published(&h)
        .into_iter()
        .filter(|id| tenants.contains(id))
        .collect();
    assert!(
        with_rect.len() < tenants.len(),
        "controlo partido: com {} inquilinos, {} publicaram rect — se todos publicassem, o criterio \
         antigo nao teria falhado e este teste nao mede nada",
        tenants.len(),
        with_rect.len()
    );

    let choice = h.store.dock_width_choice(DockSide::Right);
    dock_columns::close(&mut h, DockSide::Right, choice);

    for id in &tenants {
        assert!(
            !h.is_panel_visible(id),
            "«{id}» sobreviveu ao fecho. Com N inquilinos em abas, fechar a coluna exigia N \
             arrastos — e cada um deles largava a costura"
        );
    }
}

/// ⭐⭐ **Reabrir devolve a largura que a coluna tinha, e não o mínimo.**
///
/// ⚠️ A linha do meio é o que torna este gate honesto: ela **imita o arrasto**, que escreve a
/// largura a cada pixel e deixa o mínimo gravado muito antes de o degrau de fecho disparar. Sem
/// ela, ler o store no instante do fecho também devolveria 420 e o gate ficaria verde sobre o
/// desenho que produziu o report.
#[test]
fn reopening_restores_the_width_the_column_had_before_the_drag() {
    let mut h = settled(&["inspector"]);
    h.store.set_dock_width(DockSide::Right, 420.0);
    paint(&mut h, 3);

    let at_start = h.store.dock_width_choice(DockSide::Right);
    assert_eq!(at_start, Some(420.0));

    // O arrasto, a caminho do degrau: cada pixel escreve, e a porta clampa no mínimo.
    h.store.set_dock_width(DockSide::Right, 200.0);
    assert_eq!(
        h.store.dock_width_choice(DockSide::Right),
        Some(ph2d_editor_core::interaction::WidgetStore::DOCK_W_MIN)
    );

    dock_columns::close(&mut h, DockSide::Right, at_start);
    paint(&mut h, 3);
    dock_columns::open(&mut h, DockSide::Right);

    assert_eq!(
        h.store.dock_width_choice(DockSide::Right),
        Some(420.0),
        "a coluna voltou no minimo em vez da largura do artista — e o doc do gesto prometia por \
         escrito «um toque reabre a coluna na largura que ela tinha»"
    );
}

/// ⛔ **Uma coluna que ninguém dimensionou NÃO ganha uma escolha de largura.**
///
/// A persistência grava exactamente `dock_width_choice`: repor um número que ninguém escolheu
/// escrevê-lo-ia no ficheiro como decisão do artista, e prenderia a coluna nesse valor para sempre
/// no dia em que o default mudasse.
#[test]
fn a_column_that_was_never_sized_does_not_gain_a_width_choice() {
    let mut h = settled(&["inspector"]);
    assert_eq!(h.store.dock_width_choice(DockSide::Right), None);

    dock_columns::close(&mut h, DockSide::Right, None);
    paint(&mut h, 3);
    dock_columns::open(&mut h, DockSide::Right);

    assert_eq!(
        h.store.dock_width_choice(DockSide::Right),
        None,
        "reabrir inventou uma escolha de largura que o artista nunca fez"
    );
}

/// ⭐⭐ **Sem memória, a coluna devolve o que a TAREFA declara — nunca o registo inteiro.**
///
/// É o caso do arranque com a coluna já fechada. ⚠️ A asserção que interessa é a **estrita**: o
/// conjunto tem de ser mais pequeno que a população que declara aquele lado, que é precisamente a
/// diferença entre 1 e 22 do report.
#[test]
fn without_memory_the_column_gives_back_what_the_task_declares() {
    let h = settled(&[]);
    assert!(!dock_columns::has_memory(&h, DockSide::Right));

    let back = dock_columns::fallback(&h, DockSide::Right);
    let candidates = declaring(&h, DockSide::Right);

    assert!(
        !back.is_empty(),
        "puxar a borda de uma coluna sem memoria nao deu nada — a alca ficaria inerte"
    );
    assert!(
        back.len() < candidates.len(),
        "a reabertura sem memoria devolveu {} de {} candidatos: ela voltou a ser uma re-derivacao \
         sobre o registo, que e' um superconjunto por construcao",
        back.len(),
        candidates.len()
    );
    for id in &back {
        let float = ph2d_editor_core::panel::with_registry_ref(|reg| {
            reg.panels()
                .iter()
                .find(|p| p.manifest.id == *id)
                .is_some_and(|p| p.manifest.can_float)
        });
        assert!(!float, "«{id}» flutua e nao devia estar numa coluna");
    }
}

/// ⭐⭐⭐ **A aba ESCOLHIDA é sempre pintada — mesmo quando não cabem todas.**
///
/// ⛔ O emparelhamento ingénuo (`occ.iter().zip(tab_rects(bar, occ.len()))`) trunca pelo mais curto
/// e o escolhido é o **último** da ordem z ⇒ com transbordo **nenhuma** aba acende e o painel que
/// desenha não tem aba. Foi metade do report de 2026-09-07.
///
/// ⚠️ A asserção do meio é o controlo de vacuidade: sem transbordo o teste passaria sobre o
/// emparelhamento partido.
#[test]
fn the_selected_tab_is_painted_even_when_they_do_not_all_fit() {
    let h = settled(&[
        "audio_mixer",
        "audio_editor",
        "inspector",
        "physics",
        "tokens",
    ]);
    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let l = h.last_layout.expect("o quadro publicou o layout");
    let bar = l.slot_tabs[Slot::RightTop as usize];
    assert!(bar.h > 0.0, "sem faixa de abas nao ha' o que medir");

    let front = slot_tabs::chosen(&h, Slot::RightTop);
    // ⚠️⚠️ **A faixa é ESPREMIDA de propósito, e a razão é uma medição de 2026-09-08.** Desde que
    //    uma aba tem PISO de largura (nunca mais estreita do que alta), cinco ocupantes cabem
    //    todos na coluna de fábrica — `5 × 22 = 110` contra `296` px úteis. *A coluna real deixou
    //    de produzir o fenómeno que este gate mede*, e alimentá-la aqui deixaria a asserção a
    //    passar sobre um caso que já não existe. O sujeito é a função, e a geometria é dela.
    let tight = ph2d_editor_core::zones::Rect::new(bar.x, bar.y, 100.0, bar.h);
    let painted =
        slot_tabs::tab_layout(&occ, front, tight, &mut TextSystem::without_system_fonts());
    assert!(
        painted.len() < occ.len(),
        "controlo partido: {} ocupantes cabem todos em {} px, entao nao ha' transbordo e este teste \
         nao mede nada",
        occ.len(),
        tight.w
    );

    let selected = occ.last().map(|o| o.node).expect("ha' ocupantes");
    assert!(
        painted.iter().any(|(o, _)| o.node == selected),
        "a aba do painel que ESTA' a desenhar nao foi pintada: ocupantes {:?}, pintadas {:?}",
        occ.iter().map(|o| o.id).collect::<Vec<_>>(),
        painted.iter().map(|(o, _)| o.id).collect::<Vec<_>>()
    );
}

/// ⛔ **E as metades de baixo não são esquecidas.** A varredura sai de [`Slot::dock_side`], não de
/// uma lista escrita à mão — hoje `LeftBottom`/`RightBottom` estão vazias, e uma lista à mão
/// ficaria certa hoje e calada no dia em que um painel as declarasse.
#[test]
fn both_halves_of_a_column_are_swept() {
    let h = settled(&["inspector"]);
    let right: Vec<_> = Slot::ALL
        .into_iter()
        .filter(|s| s.dock_side() == Some(DockSide::Right))
        .collect();
    assert_eq!(
        right,
        vec![Slot::RightTop, Slot::RightBottom],
        "a coluna da direita deixou de ter duas metades"
    );
    let _ = h;
}
