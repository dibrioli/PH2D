//! ⭐⭐ **A BARRA DE MENUS REALOJA VERBOS — e este gate mede o REALOJAMENTO, não o desenho.**
//!
//! A barra nasceu em 2026-08-30, no lugar dos 29 pills (Enio: *«pode tirar também os botões do
//! topo para começarmos a trabalhar a barra superior»*). A promessa dela é a **D2**: cada comando
//! tem um sítio canónico, e a barra **não constrói** verbos — ela dá casa aos que já existiam.
//!
//! ⛔ **Uma barra bonita e muda é exactamente o defeito que este repo já pagou** (o pill `[SHEET]`,
//! os quatro pills de vetor, o botão Redo): pintar é a metade fácil, e nenhum gate de pintura vê a
//! outra. Por isso cada teste aqui **carrega no botão** e mede o efeito.
//!
//! # As quatro formas de esta barra falhar em silêncio
//!
//! | # | falha | quem a apanha |
//! |---|---|---|
//! | 1 | um título pintado que não abre menu nenhum | [`pressing_a_title_opens_its_own_menu`] |
//! | 2 | uma linha do menu *Window* que ninguém despacha | [`every_window_row_reaches_a_consumer`] |
//! | 3 | o menu que **não fecha** ao escolher (o painel consome o clique antes do chrome) | [`choosing_a_row_closes_the_menu`] |
//! | 4 | a barra a **flutuar** sobre o desenho em vez de subtrair altura | [`the_bar_subtracts_height_it_never_floats`] |

use bumpalo::Bump;
use ph2d_editor_core::interaction::{ContextMenuKind, HitIndex, WidgetEvent, dispatch_pointer};
use ph2d_editor_core::screens::hero::menu_bar::{self, MENUS};
use ph2d_editor_core::screens::hero::{HeroScreen, ids, menu_rows::menu_rows};
use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;

fn hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    HeroScreen::new(ph2d_editor_core::NodeId(1))
}

fn bar() -> Rect {
    Rect::new(0.0, 0.0, 1366.0, menu_bar::MENU_BAR_H)
}

/// **Carregar num título abre o MENU DELE** — pela cadeia real de ponteiro, não chamando o
/// handler à mão.
///
/// ⚠️ A âncora sai do **rectângulo** do título, e o `y` do menu tem de cair por baixo da barra:
/// um menu ancorado no cursor saltaria de sítio conforme onde no título se carregou.
///
/// ⛔ E os quatro juntos de propósito: os cinco blocos gémeos que esta tabela substituiu eram
/// exactamente a forma de um `kind` copiado do vizinho.
#[test]
fn pressing_a_title_opens_its_own_menu() {
    let mut text = TextSystem::without_system_fonts();
    let rects = menu_bar::menu_rects(bar(), &mut text);
    for (id, title, r) in rects {
        let mut h = hero();
        let mut hit = HitIndex::default();
        hit.register(id, r);
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut h.store,
            &hit,
            PointerEvent {
                x: r.x + r.w * 0.5,
                y: r.y + r.h * 0.5,
                pressure: 1.0,
                kind: PointerKind::Down,
                source: PointerSource::Mouse,
                button: PointerButton::Primary,
                timestamp_ns: 0,
            },
            &arena,
        );
        let open = h
            .store
            .context_menu()
            .unwrap_or_else(|| panic!("{title}: o título é pintado e não abre menu nenhum"));
        let want = MENUS
            .iter()
            .find(|(mid, ..)| *mid == id)
            .expect("o id veio da tabela")
            .2;
        assert_eq!(open.kind, want, "{title} abriu o menu do vizinho");
        assert!(
            open.y >= r.y + r.h,
            "{title}: o menu abriu POR CIMA do título (y={}, barra acaba em {})",
            open.y,
            r.y + r.h
        );
    }
}

/// **E um id que não é título não abre menu nenhum** — o controlo negativo do `find`, que é o que
/// impede um `_ =>` de ter engolido o mundo.
#[test]
fn an_unrelated_press_opens_nothing() {
    let mut h = hero();
    let mut hit = HitIndex::default();
    let r = Rect::new(400.0, 400.0, 40.0, 20.0);
    hit.register(ids::TOOL_UNDO, r);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut h.store,
        &hit,
        PointerEvent {
            x: r.x + 1.0,
            y: r.y + 1.0,
            pressure: 1.0,
            kind: PointerKind::Down,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        },
        &arena,
    );
    assert!(h.store.context_menu().is_none());
}

/// ⭐ **As linhas de *File*, *Edit* e *View* chegam a um consumidor.**
///
/// ⛔⛔ **E as treze do *Window* NÃO estão aqui — não por escolha, por ALCANCE.** Elas são
/// despachadas pelos **painéis**, e o `test_support::ensure_panel_registry` desta crate é um
/// `{}`: o registry vive na `ph2d-panel-registry-init`, que depende desta. Escrevê-las aqui daria
/// quatro acusações falsas (mixer, editor de áudio, galeria, grelha) sobre código correcto.
/// ⇒ o gate delas mora em
/// `ph2d-panel-registry-init/tests/the_window_menu_reaches_every_module.rs`, que é a crate mais
/// barata que enxerga as duas cadeias. *Um gate escrito de UMA camada deixa a outra por medir.*
#[test]
fn every_other_menu_bar_row_reaches_a_consumer() {
    let mut dead = Vec::new();
    for kind in [
        ContextMenuKind::MenuBarFile,
        ContextMenuKind::MenuBarEdit,
        ContextMenuKind::MenuBarView,
    ] {
        for (id, label, _) in menu_rows(kind) {
            let mut h = hero();
            if !h.apply_event(WidgetEvent::Click(*id)) {
                dead.push(*label);
            }
        }
    }
    assert!(dead.is_empty(), "linhas mudas: {dead:?}");
}

/// ⛔⛔ **Escolher TIRA o menu do ecrã — e é o teste que prova a ordem do `apply_event`.**
///
/// ⚠️ A propriedade é *«este menu deixa de estar aberto»*, e não *«não há menu nenhum»*: a linha
/// *New Image…* **substitui** o menu pelo modal dela, e exigir `None` acusaria a linha correcta.
/// (A primeira redacção deste gate fez exactamente isso.)
///
/// O registo de painéis é caminhado **antes** do `chrome::dispatch_all`, então um
/// `Click(TOPBAR_AUDIO_MIXER)` é consumido pelo painel do mixer e o chrome nunca o vê. Um fecho
/// escrito num handler de chrome ficaria morto exactamente nas treze linhas do menu *Window* — o
/// artista escolheria *Audio Mixer*, o painel abriria, e o menu ficaria pousado por cima dele.
#[test]
fn choosing_a_row_closes_the_menu() {
    for kind in [
        ContextMenuKind::MenuBarWindow,
        ContextMenuKind::MenuBarFile,
        ContextMenuKind::MenuBarView,
    ] {
        let (id, label, _) = menu_rows(kind)[0];
        let mut h = hero();
        h.store
            .open_context_menu(ph2d_editor_core::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind,
            });
        let _ = h.apply_event(WidgetEvent::Click(id));
        assert_ne!(
            h.store.context_menu().map(|r| r.kind),
            Some(kind),
            "{label} ({kind:?}): o menu ficou aberto depois de escolher"
        );
    }
}

/// **As DUAS linhas que abrem outra coisa substituem o menu, não o fecham.**
///
/// ⚠️ Elas estão excluídas do fecho antecipado de propósito — o `cascade_anchor` lê o menu ainda
/// aberto para saber de onde a cascata sai. Se o fecho as apanhasse, o submenu abriria no canto.
#[test]
fn the_cascading_rows_replace_the_menu() {
    for (row, want) in [
        (ids::MENUBAR_EDIT_PREFERENCES, ContextMenuKind::SettingsMenu),
        (ids::MENUBAR_VIEW_THEME, ContextMenuKind::ThemeSelector),
    ] {
        let mut h = hero();
        h.store
            .open_context_menu(ph2d_editor_core::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: ContextMenuKind::MenuBarView,
            });
        assert!(h.apply_event(WidgetEvent::Click(row)));
        assert_eq!(
            h.store.context_menu().map(|r| r.kind),
            Some(want),
            "a linha tinha de SUBSTITUIR o menu pelo dela"
        );
    }
}

/// **A linha *Rulers* mexe no MESMO interruptor que as réguas lêem.**
///
/// ⚠️ A régua tinha um dono só — uma caixa dentro do painel do vetor —, e ele deixou de fazer
/// sentido no dia em que as réguas passaram a valer em todos os modos. Duas portas, **um** valor:
/// um `bool` próprio aqui faria o menu dizer *ligada* sobre uma régua que o painel desligou.
#[test]
fn the_rulers_row_flips_the_switch_the_rulers_read() {
    let mut h = hero();
    let before = h.rulers_live();
    assert!(h.apply_event(WidgetEvent::Click(ids::MENUBAR_VIEW_RULERS)));
    assert_eq!(h.rulers_live(), !before, "a linha não mexeu na régua");
    assert!(h.apply_event(WidgetEvent::Click(ids::MENUBAR_VIEW_RULERS)));
    assert_eq!(h.rulers_live(), before, "e volta atrás");
}

/// **A linha *New Image…* abre o modal que só a tecla abria.**
#[test]
fn the_new_image_row_opens_the_dialog() {
    let mut h = hero();
    assert!(h.apply_event(WidgetEvent::Click(ids::MENUBAR_FILE_NEW)));
    assert_eq!(
        h.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::NewImageDialog)
    );
}

/// ⛔⛔ **A barra SUBTRAI altura — ela nunca flutua sobre o desenho.**
///
/// É a spec §4 ao pé da letra: *«a barra global deixa de flutuar sobre o conteúdo e passa a
/// subtrair altura, como o trilho subtrai largura»*. Uma barra a flutuar reproduziria, num modelo
/// novo, o defeito de 29,4 % que a wave das réguas curou — e desta vez sobre a régua de cima, que
/// nasce exactamente na borda de cima da área de desenho.
#[test]
fn the_bar_subtracts_height_it_never_floats() {
    let viewport = Rect::new(0.0, 0.0, 1366.0, 1024.0);
    let layout = HeroLayout::for_viewport_bands(
        viewport,
        false,
        ChromeBands {
            rail_w: 0.0,
            top_bar_h: menu_bar::MENU_BAR_H,
            ..ChromeBands::DEFAULT
        },
        CenterSplit::None,
        DockSides::BOTH,
    );
    assert!(
        layout.top_bar.h > 0.0,
        "a barra tem de ter faixa — sem ela não há onde pintar os títulos"
    );
    assert!(
        layout.draw_area.y >= layout.top_bar.y + layout.top_bar.h,
        "a área de desenho começa DENTRO da barra: {:?} contra {:?}",
        layout.draw_area,
        layout.top_bar
    );
    for (name, col) in [
        ("hierarchy", layout.hierarchy),
        ("inspector", layout.inspector),
    ] {
        assert!(
            col.y >= layout.top_bar.y + layout.top_bar.h,
            "a coluna {name} começa por baixo da barra"
        );
    }
}

/// **E os títulos não se sobrepõem** — a porta única mede-os contra o texto real, e larguras
/// fixas por título dariam ou um buraco ou um recorte conforme a fonte.
#[test]
fn the_titles_are_laid_side_by_side_without_overlap() {
    let mut text = TextSystem::without_system_fonts();
    let rects = menu_bar::menu_rects(bar(), &mut text);
    for pair in rects.windows(2) {
        let (_, a_title, a) = pair[0];
        let (_, b_title, b) = pair[1];
        assert!(
            a.x + a.w <= b.x + f32::EPSILON,
            "{a_title} e {b_title} sobrepõem-se"
        );
        assert!(a.w > 0.0, "{a_title} tem largura zero");
    }
    let last = rects[rects.len() - 1].2;
    assert!(
        last.x + last.w <= bar().x + bar().w,
        "os títulos transbordam da barra"
    );
}

/// ⛔⛔ **O QUADRO REAL: os quatro títulos ficam agarráveis onde a barra os desenha.**
///
/// ⚠️ Os outros testes deste ficheiro montam o `HitIndex` à mão a partir da porta
/// ([`menu_bar::menu_rects`]) — provam a **lei**, e um pintor que se esquecesse de registar
/// passaria por todos eles. Este pinta o hero de verdade e pergunta ao índice do quadro.
///
/// ⭐ É a mesma lição que o trilho lateral ainda paga: lá o pintor e o registo de hit são **dois
/// laços** sobre a mesma lista, com a aritmética escrita duas vezes e nada a ligá-los.
///
/// *Mutação que sangra:* apagar o `hit_index.register` do `paint_menu_bar`.
#[test]
fn the_painted_bar_registers_every_title_where_it_drew_it() {
    let mut h = hero();
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, 1366.0, 1024.0);
    ph2d_editor_core::screens::hero::paint_hero_screen(&mut h, viewport, &mut scene, &mut text);
    assert!(
        !h.view.legacy_chrome,
        "precondição: sem chrome legado a barra de menus é o inquilino da faixa"
    );
    let layout = h.last_layout.expect("o quadro publicou o layout");
    for (id, title, want) in menu_bar::menu_rects(layout.top_bar, &mut text) {
        let got = h
            .hit_index
            .rect_for(id)
            .unwrap_or_else(|| panic!("{title}: pintado e SEM alvo no índice do quadro"));
        assert!(
            (got.x - want.x).abs() < 0.5 && (got.w - want.w).abs() < 0.5,
            "{title}: o alvo ({got:?}) não é onde a porta diz ({want:?})"
        );
        assert!(
            got.y + got.h <= layout.draw_area.y + 0.5,
            "{title}: o alvo entra na área de desenho"
        );
    }
}
