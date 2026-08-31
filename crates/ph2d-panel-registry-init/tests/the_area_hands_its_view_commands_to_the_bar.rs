//! ⭐⭐⭐ **O PAINEL DEIXA DE SER O DEPÓSITO** — os comandos de vista do 3D vivem na fila.
//!
//! # ⛔⛔ O defeito, com número
//!
//! O painel `3D Model` tem **74 entradas** e só **8** são propriedades do objecto
//! (`docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md` §D2) — as outras **66 têm outro dono**. O
//! sintoma não é uma opinião: aquele painel precisou de **barra de rolagem** (report do Enio,
//! 2026-08-27), porque não cabia.
//!
//! ⇒ as **nove** primeiras mudam-se: seis vistas nomeadas + três gestos de câmera. Elas nunca foram
//! propriedades de nada.
//!
//! # ⚠️ O dono é a FILA, e é UM chip
//!
//! > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
//!
//! A faixa própria (o *cabeçalho de área* da D2) foi construída e revertida no mesmo dia: `28 px`
//! permanentes. E **nove chips** na fila levam-na a **2 linhas até no iPad 12,9"** — o maior dos
//! três alvos —, com `2` chips a transbordar (medido pela mutação 6).
//! *Poupar altura gastando largura não poupa nada.*
//! ⇒ um **pulldown**, cuja face é a leitura da vista actual.
//!
//! # ⚠️ E o gate carrega num PIXEL, duas vezes
//!
//! Um `apply_event(Click(id))` sintético passaria com o chip **morto sob o dedo** — foi assim que o
//! `⋯` nasceu (sem `InteractiveState`) e assim que os quatro chips do vetor morreram. E há uma
//! segunda costura que só o gesto real mede: **servir é fechar**, e o fecho vive no `pre_dispatch`
//! porque estes ids são de um PAINEL, que consome o clique antes de o chrome o ver.

use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::screens::hero::{HeroScreen, tool_bar};
use ph2d_editor_core::widget::RailButtonSize;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_model3d::{ModeChip, ModelIntent, ModelSnapshot};
use ph2d_text::TextSystem;

const TABLETS: [(&str, f32, f32); 3] = [
    ("iPad 12.9", 1366.0, 1024.0),
    ("iPad 11", 1194.0, 834.0),
    ("iPad mini", 1133.0, 744.0),
];

/// As seis vistas e os três gestos de câmera, na ordem em que o shell os publica.
fn snapshot_with_views() -> ModelSnapshot {
    let view = |key, active| ModeChip { key, active };
    ModelSnapshot {
        views: vec![
            view("panel.model3d.view.front", false),
            view("panel.model3d.view.back", false),
            view("panel.model3d.view.right", true),
            view("panel.model3d.view.left", false),
            view("panel.model3d.view.top", false),
            view("panel.model3d.view.bottom", false),
        ],
        camera: vec![
            view("panel.model3d.camera.ortho", false),
            view("panel.model3d.camera.frame", false),
            view("panel.model3d.camera.quad", false),
        ],
        view_label: "viewport.model3d.view.right",
        ..ModelSnapshot::default()
    }
}

fn hero(w: f32, h: f32, armed: bool) -> (HeroScreen, Rect) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.view.legacy_chrome = false;
    ph2d_panel_model3d::publish(snapshot_with_views());
    ph2d_panel_model3d::publish_area_bar(&mut hero.store, armed);
    (hero, Rect::new(0.0, 0.0, w, h))
}

fn paint(h: &mut HeroScreen, vp: Rect) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..2 {
        ph2d_editor_core::screens::hero::paint_hero_screen(h, vp, &mut scene, &mut text);
    }
}

fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

/// ⚠️ Down **e** Up pelo `dispatch_pointer`, como o `⋯` — um `Click` sintético salta exactamente a
/// metade que falha (o `HitIndex` resolver o ponto).
fn click_at(h: &mut HeroScreen, r: Rect) {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let arena = bumpalo::Bump::new();
    for ev in [
        pointer(PointerKind::Down, x, y),
        pointer(PointerKind::Up, x, y),
    ] {
        let events =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena);
        let evs: Vec<_> = events.to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

/// ⭐⭐⭐ **O pulldown abre, serve, e fecha-se ao servir** — com o dedo, não com um `Click` fabricado.
#[test]
fn the_area_pulldown_opens_serves_a_view_and_closes() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    let _ = ph2d_panel_model3d::drain_intents();
    paint(&mut h, vp);

    let chip = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::AREA_COMMANDS)
        .expect("o pulldown da area nao foi registado no indice de acerto — ele nasceu invisivel");

    // ⛔ **Antes de abrir, os comandos NÃO estão em lado nenhum** — é isto que prova que eles
    // saíram do painel e não que ganharam um segundo sítio.
    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::model3d_view_button(0))
            .is_none(),
        "a vista continua registada com o menu fechado — ela nao saiu do painel, ganhou um 2.o sitio"
    );

    click_at(&mut h, chip);
    assert!(
        matches!(
            h.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::AreaCommands)
        ),
        "o chip nao abriu o menu da area — ele esta' morto sob o dedo"
    );
    paint(&mut h, vp);

    let row = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::model3d_view_button(0))
        .expect("a 1.a vista nao foi pintada no menu aberto");
    click_at(&mut h, row);

    let intents = ph2d_panel_model3d::drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, ModelIntent::SetView { slot: 0 })),
        "carregar na vista nao pediu nada ao shell: {intents:?}"
    );
    // ⭐ **Servir é fechar** — e esta metade só vive porque o fecho está no `pre_dispatch`: o painel
    // consome o clique e o `chrome::dispatch_all` nunca chega a correr.
    assert!(
        h.store.context_menu().is_none(),
        "o menu ficou aberto POR CIMA da coisa que o clique acabou de fazer"
    );
}

/// ⭐⭐ **Fechar o módulo tira o chip da fila no MESMO quadro.**
///
/// ⚠️ É a lei do `⋯`: publicado em todo quadro, vazio incluído. Sem o ramo vazio o chip ficava lá,
/// a despachar para um painel que já não está.
#[test]
fn closing_the_module_takes_the_pulldown_off_the_bar() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    paint(&mut h, vp);
    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::AREA_COMMANDS)
            .is_some(),
        "controlo: com o modulo armado o pulldown tem de estar la'"
    );

    ph2d_panel_model3d::publish_area_bar(&mut h.store, false);
    paint(&mut h, vp);
    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::AREA_COMMANDS)
            .is_none(),
        "o pulldown sobreviveu ao fecho do modulo"
    );
    assert!(
        h.store.area_entries().is_empty(),
        "os comandos da area sobreviveram ao fecho do modulo"
    );
}

/// ⭐⭐⭐ **A faixa continua a ser UMA linha nos três tablets, com o módulo armado.**
///
/// ⚠️ E o gate imprime **quantos** chips a área acrescenta: se um dia forem nove em vez de um, esta
/// é a linha que o diz antes de a largura o dizer.
#[test]
fn the_area_costs_one_chip_and_the_bar_is_still_one_line() {
    for (name, w, ht) in TABLETS {
        let (h, vp) = hero(w, ht, true);
        let bands = ph2d_editor_core::screens::layout::ChromeBands {
            rail_w: 0.0,
            top_bar_h: ph2d_editor_core::screens::hero::menu_bar::MENU_BAR_H,
            tool_bar_h: tool_bar::tool_bar_h(RailButtonSize::Small, 1),
            ..ph2d_editor_core::screens::layout::ChromeBands::DEFAULT
        };
        let area_w = ph2d_editor_core::screens::layout::HeroLayout::for_viewport_bands(
            vp,
            false,
            bands,
            ph2d_editor_core::screens::layout::CenterSplit::None,
            ph2d_editor_core::screens::layout::DockSides::BOTH,
        )
        .draw_area
        .w;
        let (rail, over) = tool_bar::bar_split(&h.store, false, false, area_w);
        let added = rail
            .entries
            .iter()
            .chain(over.iter())
            .filter(|e| e.node_id() == Some(ph2d_editor_core::ids::AREA_COMMANDS))
            .count();
        println!(
            "{name:11} a area acrescenta {added} chip(s); fila {:2} + {:2} atras do dots",
            rail.entries.len(),
            over.len()
        );
        assert_eq!(
            added, 1,
            "{name}: a area pos {added} chips na fila — ela tem de ser UM pulldown"
        );
        let lines =
            ph2d_editor_core::widget::horizontal_lines(&rail, area_w - 16.0, RailButtonSize::Small);
        assert_eq!(lines, 1, "{name}: a fila precisa de {lines} linhas");
    }
}

/// ⭐⭐⭐ **O CAMINHO SEM DEDO** — a paleta de comandos, que não tem `Down` nenhum.
///
/// ⛔⛔ **Este gate existe porque uma MUTAÇÃO sobreviveu.** Sob o ponteiro, quem fecha um menu ao
/// servir é a regra genérica do store (`pointer_down` + `click_belongs_to_the_open_menu`), e não o
/// handler do chip: apagar o interruptor de lá deixava o gate de gesto verde.
///
/// ⚠️ Mas a **paleta de comandos global** levanta `apply_event(Click(id))` sem ponteiro nenhum — e
/// ela projecta a lista que a fila pinta. Por esse caminho o fecho genérico nunca corre, e sem o
/// interruptor escolher o chip **re-abriria** um menu já aberto.
#[test]
fn the_palette_path_toggles_the_menu_because_it_has_no_pointer_down() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    paint(&mut h, vp);
    let click =
        ph2d_editor_core::interaction::WidgetEvent::Click(ph2d_editor_core::ids::AREA_COMMANDS);
    assert!(
        h.apply_event(click),
        "controlo: o chip nao respondeu ao clique sintetico"
    );
    assert!(
        matches!(
            h.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::AreaCommands)
        ),
        "a paleta nao abriu o menu"
    );
    assert!(h.apply_event(click));
    assert!(
        h.store.context_menu().is_none(),
        "pela paleta o 2.o pick RE-ABRIU o menu — o interruptor do chip e' a unica coisa que o fecha \
         quando nao ha' Down"
    );
}
