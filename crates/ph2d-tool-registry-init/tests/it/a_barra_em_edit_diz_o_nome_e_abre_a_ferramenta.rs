//! Report do dono, 04/11: «em Edit Mode da Sprite o botão Remove Background desapareceu». O botão
//! estava lá com outro nome: a legenda da fila em cartões dizia «Cutout», e «Color», «Equal», «Raster»
//! na família inteira (`docs/CartoesUI/15_CHROME_CENSO.md`). Estes gates seguem o gesto real, com o
//! registry do boot: o nome inteiro na fila, o clique que abre a ferramenta, e o `⋯` numa janela estreita.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::object_mode::TOOLS_OPENED_BY_A_MODE;
use ph2d_editor_core::screens::hero::{HeroScreen, paint_hero_screen, tool_bar};
use ph2d_editor_core::widget::lista_flutuante::force_cromo_on_this_thread;
use ph2d_editor_core::widget::{RailAxis, ToolRailEntry, entry_advance, entry_gap_px};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;
use ph2d_tool_registry::{Registry, hash_node_id};

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn registry() -> &'static Registry {
    let mut reg = Registry::default();
    ph2d_tool_registry_init::register_all(&mut reg);
    reg.build().expect("o registry do boot tem de construir");
    ph2d_editor_core::install_registry(reg);
    ph2d_editor_core::installed_registry().expect("acabou de ser instalado")
}

fn pinta(hero: &mut HeroScreen, ts: &mut TextSystem, w: f32, h: f32) {
    let mut scene = ph2d_vector::VectorScene::new();
    paint_hero_screen(hero, Rect::new(0.0, 0.0, w, h), &mut scene, ts);
}

/// Um Sprite em Edit Mode, com a seleção de que uma ferramenta de uma só passagem precisa.
fn em_edit(ts: &mut TextSystem, w: f32, h: f32) -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    registry();
    let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
    hero.image_edit.mode_on = true;
    hero.gizmo.selection = Some(0xDEAD_BEEF);
    pinta(&mut hero, ts, w, h);
    pinta(&mut hero, ts, w, h);
    hero
}

/// Down+Up no centro de `r`, depois o despacho que o quadro faz com cada evento.
fn clica(hero: &mut HeroScreen, r: Rect) -> Vec<EditorAction> {
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    for e in ev {
        hero.apply_event(e);
    }
    hero.bus.drain().collect()
}

fn abriu_a_remocao(acoes: &[EditorAction]) -> bool {
    acoes
        .iter()
        .any(|a| matches!(a, EditorAction::ActivateTool { tool_id } if *tool_id == "bgremoval"))
}

fn nome_e_legenda(e: &ToolRailEntry) -> (String, String) {
    let sub = match e {
        ToolRailEntry::Icon { sub, .. } | ToolRailEntry::Glyph { sub, .. } => sub.clone(),
        _ => String::new(),
    };
    (e.label().unwrap_or_default().to_owned(), sub)
}

/// ⭐⭐⭐ Na largura de fábrica o botão está na fila, diz «Remove Background» inteiro e abre a ferramenta.
#[test]
fn remove_background_esta_na_fila_com_o_nome_inteiro_e_abre_a_ferramenta() {
    force_cromo_on_this_thread(Some(true));
    let mut ts = TextSystem::without_system_fonts();
    let mut hero = em_edit(&mut ts, 1930.0, 1040.0);
    let id = ids::IMAGE_ACTION_BGREMOVAL;
    let chip = hero
        .hit_index
        .rect_for(id)
        .expect("o botão não está na fila");
    assert!(
        hero.store.tool_overflow().is_empty(),
        "a fila transborda na largura de fábrica"
    );

    let rail = tool_bar::bar_rail(&hero.store, &mut ts, false, true);
    let entrada = rail
        .entries
        .iter()
        .find(|e| e.node_id() == Some(id))
        .expect("o botão não está na lista da fila");
    let (nome, legenda) = nome_e_legenda(entrada);
    assert_eq!(nome, "Remove Background");
    assert_eq!(legenda, "Remove Background");
    // Na fila FINA (maquete C, BarraDeMenus onda 6) não há legenda: o nome inteiro vai na dica (o `label`
    // acima). Só a fila larga pinta a legenda, e aí ela tem de caber no passo do chip.
    if !tool_bar::fila_fina(&rail) {
        let passo = entry_advance(entrada, chip.h, RailAxis::Horizontal) + entry_gap_px();
        let w = ts.prefix_width(&legenda, ph2d_card_stack::metrics::LABEL_PX);
        assert!(
            w <= passo,
            "a legenda ({w}) não cabe no passo do chip ({passo})"
        );
    }

    assert!(
        abriu_a_remocao(&clica(&mut hero, chip)),
        "o clique real não abriu a ferramenta"
    );
    force_cromo_on_this_thread(None);
}

/// Numa janela estreita nada some: o botão vai para o `⋯`, a lista diz o nome inteiro e abre a ferramenta.
#[test]
fn numa_janela_estreita_o_botao_mora_no_transbordo_e_abre_a_ferramenta() {
    force_cromo_on_this_thread(Some(true));
    let mut ts = TextSystem::without_system_fonts();
    let id = ids::IMAGE_ACTION_BGREMOVAL;
    // A fila fina cabe mais botões que a larga: a janela estreita é a primeira em que o botão transborda.
    let h = 744.0;
    let (w, mut hero) = [560.0_f32, 480.0, 420.0, 380.0, 340.0]
        .into_iter()
        .map(|w| (w, em_edit(&mut ts, w, h)))
        .find(|(_, hero)| {
            hero.store
                .tool_overflow()
                .iter()
                .any(|e| e.node_id() == Some(id))
        })
        .expect("controlo: numa janela estreita o botão tem de estar no transbordo");
    let over: Vec<_> = hero.store.tool_overflow().to_vec();
    let entrada = over
        .iter()
        .find(|e| e.node_id() == Some(id))
        .expect("controlo: nesta largura o botão tem de estar no transbordo");
    assert_eq!(nome_e_legenda(entrada).0, "Remove Background");

    let dots = hero
        .hit_index
        .rect_for(ids::TOOL_BAR_OVERFLOW)
        .expect("há transbordo e não há `⋯`");
    clica(&mut hero, dots);
    assert_eq!(
        hero.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::ToolBarOverflow),
        "o `⋯` não abriu"
    );
    pinta(&mut hero, &mut ts, w, h);
    let linha = hero
        .hit_index
        .rect_for(id)
        .expect("a linha do botão não está no menu");
    let nome_w = ts.prefix_width("Remove Background", ph2d_tokens::TypeToken::Sm.px());
    assert!(
        linha.w >= nome_w,
        "a linha ({}) não cabe o nome ({nome_w})",
        linha.w
    );
    assert!(
        linha.x >= 0.0 && linha.x + linha.w <= w,
        "a lista sai da janela: {linha:?}"
    );
    assert!(
        abriu_a_remocao(&clica(&mut hero, linha)),
        "a linha do `⋯` não abriu a ferramenta"
    );
    assert!(hero.store.context_menu().is_none(), "o menu não fechou");
    force_cromo_on_this_thread(None);
}

/// Toda ferramenta da fila, a de amanhã incluída, diz o nome inteiro em cartões, nunca sigla nem
/// palavra que não diz o que faz. CONTROLO: `PH2D_CARDS=0` fica com a sigla de sempre.
#[test]
fn toda_ferramenta_da_fila_diz_o_nome_inteiro() {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let reg = registry();
    let store = HeroScreen::new(ph2d_a11y::NodeId(1)).store;
    let mut ts = TextSystem::without_system_fonts();
    let vagas = ["Cutout", "Color", "Equal", "Raster", "Reshape", "Square"];
    let ferramentas: Vec<_> = reg
        .cluster("image_tools")
        .iter()
        .filter(|m| !TOOLS_OPENED_BY_A_MODE.iter().any(|(id, _)| *id == m.id))
        .collect();
    assert_eq!(ferramentas.len(), 9, "controlo: as nove da foto de 04/11");
    for m in ferramentas {
        let id = hash_node_id(m.id);
        let legenda_em = |on: bool, ts: &mut TextSystem| {
            force_cromo_on_this_thread(Some(on));
            let rail = tool_bar::bar_rail(&store, ts, false, true);
            force_cromo_on_this_thread(None);
            rail.entries
                .iter()
                .find(|e| e.node_id() == Some(id))
                .map(nome_e_legenda)
                .expect("a ferramenta não está na fila")
        };
        let (nome, legenda) = legenda_em(true, &mut ts);
        assert_eq!(nome, legenda, "{}", m.id);
        assert!(
            legenda.chars().any(char::is_lowercase) && !vagas.contains(&legenda.as_str()),
            "{}: «{legenda}» não diz o que faz",
            m.id
        );
        let dica = ph2d_i18n::tr(&m.label_key.replace(".label", ".tooltip"));
        assert!(
            dica.starts_with(&nome),
            "{}: a dica «{dica}» não começa pelo nome",
            m.id
        );
        let (_, sigla) = legenda_em(false, &mut ts);
        assert!(
            sigla.chars().all(|c| !c.is_lowercase()) && sigla.len() <= 5,
            "{}: fora de cartões mudou: «{sigla}»",
            m.id
        );
    }
}
