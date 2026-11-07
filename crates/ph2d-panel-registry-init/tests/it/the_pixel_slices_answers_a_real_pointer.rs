//! **A janela SLICES responde a um PONTEIRO e a um TECLADO reais:** a tela inteira pintada, com todos
//! os painéis registados; Down+Up no rect pintado e as teclas no campo viram o pedido que a família
//! recebe. O caminho do pedido ao documento prova-se em `shells/pixel-scenes/tests/it/slices.rs`.

use bumpalo::Bump;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL_SLICES, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids as nucleo};
use ph2d_host::{
    KeyEvent, KeyKind, Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource,
};
use ph2d_panel_pixel_slices::canal::{self, Campo, Contexto, Linha, Menu, Pedido, Vista};
use ph2d_panel_pixel_slices::ids;
use ph2d_pixel_core::SliceId;

const JANELA: NodeId = nucleo::PIXEL_SLICES_PANEL;
const KEY_ENTER: u32 = 0x0D;

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

fn assenta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    for _ in 0..60 {
        let mut scene = ph2d_vector::VectorScene::new();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || paint_hero_screen(hero, vp, &mut scene, ts),
        );
    }
}

fn centro(hero: &HeroScreen, id: NodeId, nome: &str) -> (f32, f32) {
    let r = hero
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"));
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

fn clica(hero: &mut HeroScreen, id: NodeId, nome: &str) {
    let (x, y) = centro(hero, id, nome);
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    assert!(
        ev.contains(&WidgetEvent::Click(id)),
        "{nome}: o ponteiro deu {ev:?}; em ({x},{y}) acerta {:?}",
        hero.hit_index.hit_with_rect(x, y),
    );
    for e in ev {
        hero.apply_event(e);
    }
}

/// Clica no campo, escreve `t` onde o cursor fica (no fim; o 2.º clique depressa seleciona tudo), e `Enter`.
fn escreve(hero: &mut HeroScreen, id: NodeId, t: &str) {
    let (x, y) = centro(hero, id, "o campo");
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    assert_eq!(
        hero.store.focus_id(),
        Some(id),
        "o clique não focou o campo"
    );
    for c in t.chars() {
        ev.extend_from_slice(hero.handle_text_input(c, &arena));
    }
    ev.extend_from_slice(hero.handle_key(
        KeyEvent {
            keycode: KEY_ENTER,
            modifiers: Modifiers::default(),
            kind: KeyKind::Down,
            timestamp_ns: 0,
        },
        &arena,
    ));
    for e in ev {
        hero.apply_event(e);
    }
}

fn linha(id: u32, nome: String, selecionada: bool) -> Linha {
    Linha {
        id: SliceId(id),
        nome,
        selecionada,
    }
}

/// `button` selecionada, `hero` não.
fn uma_selecionada() -> Contexto {
    Contexto {
        na_mao: true,
        linhas: vec![
            linha(1, "button".to_owned(), true),
            linha(2, "hero".to_owned(), false),
        ],
        vista: Some(Vista {
            nome: Some("button".to_owned()),
            cor: Some([0, 0, 255, 255]),
            dados: Some(String::new()),
            bounds: [Some(4), Some(6), Some(20), Some(10)],
            ..Vista::default()
        }),
        transform: false,
    }
}

fn hero_com(ts: &mut ph2d_text::TextSystem, x: Contexto) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let _ = canal::take_pedidos();
    let _ = canal::take_menu();
    let _ = canal::take_mostrar();
    canal::publish_contexto(x);
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL_SLICES, true);
    assenta(&mut hero, ts);
    hero
}

#[test]
fn a_lista_seleciona_e_all_none_transform_chegam_a_familia() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com(&mut ts, uma_selecionada());
    clica(&mut hero, ids::linha(1), "a linha hero");
    assert_eq!(canal::take_pedidos(), [Pedido::Seleciona(SliceId(2))]);
    clica(&mut hero, ids::ALL, "All");
    assert_eq!(canal::take_pedidos(), [Pedido::Todas]);
    clica(&mut hero, ids::NONE, "None");
    assert_eq!(canal::take_pedidos(), [Pedido::Nenhuma]);
    clica(&mut hero, ids::TRANSFORM, "Transform");
    assert_eq!(canal::take_pedidos(), [Pedido::Transform]);
    clica(&mut hero, ids::DUPLICATE, "Duplicate");
    assert_eq!(canal::take_pedidos(), [Pedido::Duplica]);
    clica(&mut hero, ids::DELETE, "Delete");
    assert_eq!(canal::take_pedidos(), [Pedido::Apaga]);
}

/// A janela flutua sobre o desenho e o ponteiro dentro dela é dela; o título arrasta-a.
#[test]
fn a_janela_flutua_e_o_titulo_a_arrasta() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com(&mut ts, uma_selecionada());
    let (x, y) = centro(&hero, ids::BOUNDS_X, "X");
    assert_eq!(hero.chrome_panel_at(x, y), Some(JANELA));
    let pega = centro(&hero, ids::SLICES_DRAG, "a alça");
    let arena = Bump::new();
    let passo = (-40.0, 20.0);
    let (a, b) = (pega.0 + passo.0, pega.1 + passo.1);
    let _ = hero.handle_pointer(ptr(PointerKind::Down, pega.0, pega.1), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Move, a, b), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Up, a, b), &arena);
    assenta(&mut hero, &mut ts);
    let depois = centro(&hero, ids::BOUNDS_X, "X");
    assert_eq!((depois.0 - x, depois.1 - y), passo, "o X ficou para trás");
    clica(&mut hero, ids::ALL, "All no sítio novo");
    assert_eq!(canal::take_pedidos(), [Pedido::Todas]);
}

/// Bounds na mesma linha; o que se escreve segue como foi escrito, só ao confirmar, e só se mudou.
#[test]
fn os_campos_seguem_ao_confirmar_e_so_se_mudaram() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com(&mut ts, uma_selecionada());
    let linha_y = |h: &HeroScreen, id| h.hit_index.rect_for(id).expect("campo").y;
    let y = linha_y(&hero, ids::BOUNDS_X);
    for id in [ids::BOUNDS_Y, ids::BOUNDS_W, ids::BOUNDS_H] {
        assert_eq!(linha_y(&hero, id), y, "os quatro dividem a linha");
    }
    assert_eq!(hero.store.text(ids::BOUNDS_W), Some("20"));

    escreve(&mut hero, ids::BOUNDS_X, "");
    assert_eq!(hero.store.text(ids::BOUNDS_X), Some("4"));
    assert_eq!(canal::take_pedidos(), [], "Enter sem mudar não é passo");
    escreve(&mut hero, ids::BOUNDS_X, "5");
    assert_eq!(
        canal::take_pedidos().first(),
        Some(&Pedido::Campo(Campo::X, "5".to_owned()))
    );
    assenta(&mut hero, &mut ts);
    assert_eq!(
        hero.store.text(ids::BOUNDS_X),
        Some("4"),
        "volta a mostrar o documento"
    );
    escreve(&mut hero, ids::SLICE_NAME, "!");
    assert_eq!(
        canal::take_pedidos().first(),
        Some(&Pedido::Nome("button!".to_owned()))
    );
    escreve(&mut hero, ids::USER_DATA, "hi");
    assert_eq!(
        canal::take_pedidos().first(),
        Some(&Pedido::Dados("hi".to_owned()))
    );
}

/// Valor misto aparece vazio; escrever segue para todas, e confirmar sem escrever não é passo (§10).
#[test]
fn o_valor_misto_aparece_vazio_e_escrever_vale() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut x = uma_selecionada();
    if let Some(v) = &mut x.vista {
        v.nome = None;
        v.bounds = [None, Some(6), Some(20), None];
    }
    let mut hero = hero_com(&mut ts, x);
    assert_eq!(hero.store.text(ids::SLICE_NAME), Some(""));
    assert_eq!(hero.store.text(ids::BOUNDS_X), Some(""));
    assert_eq!(hero.store.text(ids::BOUNDS_Y), Some("6"));
    escreve(&mut hero, ids::BOUNDS_X, "");
    assert_eq!(canal::take_pedidos(), []);
    escreve(&mut hero, ids::BOUNDS_X, "7");
    assert_eq!(
        canal::take_pedidos().first(),
        Some(&Pedido::Campo(Campo::X, "7".to_owned()))
    );
}

/// Os campos do centro e do pivot só aparecem ligados; o botão liga-os.
#[test]
fn nove_e_pivot_ligam_e_mostram_os_campos() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com(&mut ts, uma_selecionada());
    assert!(hero.hit_index.rect_for(ids::CENTER_X).is_none());
    assert!(hero.hit_index.rect_for(ids::PIVOT_X).is_none());
    clica(&mut hero, ids::NINE, "9-Slices");
    assert_eq!(canal::take_pedidos(), [Pedido::Nove(true)]);
    clica(&mut hero, ids::PIVOT, "Pivot");
    assert_eq!(canal::take_pedidos(), [Pedido::Pivot(true)]);

    let mut x = uma_selecionada();
    if let Some(v) = &mut x.vista {
        (v.nove, v.algum_nove, v.pivot, v.algum_pivot) = (true, true, true, true);
        v.centro = [Some(1), Some(1), Some(18), Some(8)];
        v.ponto = [Some(-3), Some(0)];
    }
    canal::publish_contexto(x);
    assenta(&mut hero, &mut ts);
    assert_eq!(hero.store.text(ids::CENTER_W), Some("18"));
    assert_eq!(hero.store.text(ids::PIVOT_X), Some("-3"));
    clica(&mut hero, ids::NINE, "9-Slices");
    assert_eq!(canal::take_pedidos(), [Pedido::Nove(false)]);
    escreve(&mut hero, ids::PIVOT_Y, "2");
    assert_eq!(
        canal::take_pedidos().first(),
        Some(&Pedido::Campo(Campo::PivotY, "02".to_owned()))
    );
}

/// Sem seleção nada se edita, nem Delete nem campo, e a janela diz porquê; a lista continua viva.
#[test]
fn sem_selecao_os_campos_estao_apagados() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut x = uma_selecionada();
    x.vista = None;
    let mut hero = hero_com(&mut ts, x);
    for id in [
        ids::SLICE_NAME,
        ids::BOUNDS_X,
        ids::DELETE,
        ids::DUPLICATE,
        ids::NINE,
        ids::PIVOT,
        ids::COLOR,
    ] {
        assert!(
            hero.hit_index.rect_for(id).is_none(),
            "{id:?} ficou clicável"
        );
    }
    clica(&mut hero, ids::linha(0), "a lista continua");
    assert_eq!(canal::take_pedidos(), [Pedido::Seleciona(SliceId(1))]);
}

/// A amostra abre o seletor partilhado; a cor dele segue à família, e fechar diz-lho.
#[test]
fn a_cor_vem_do_seletor_partilhado() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com(&mut ts, uma_selecionada());
    let (x, y) = centro(&hero, ids::COLOR, "a amostra");
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    for e in ev {
        hero.apply_event(e);
    }
    assert_eq!(hero.store.picker_target(), Some(ids::COLOR));
    assenta(&mut hero, &mut ts);
    assert_eq!(canal::take_pedidos(), [], "abrir não é escolher");
    // O valor que o arrasto e o hex do seletor escrevem.
    hero.store.set_blender_value(
        nucleo::INSP_BLENDER_PICKER,
        ph2d_tokens::ColorValue::from_rgba8(255, 0, 255, 255),
    );
    assenta(&mut hero, &mut ts);
    assert_eq!(
        canal::take_pedidos().first(),
        Some(&Pedido::Cor([255, 0, 255, 255]))
    );
}

/// O menu do botão direito: as linhas pintadas respondem; a da mudança só acende onde há mudança.
#[test]
fn o_menu_responde_e_a_mudanca_so_acende_onde_ha() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com(&mut ts, uma_selecionada());
    let onde = Menu {
        x: 300.0,
        y: 300.0,
        remover: false,
    };
    canal::abrir_menu(onde);
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(ids::MENU_REMOVE_KEY).is_none(),
        "apagada, sem clique"
    );
    assert!(
        hero.store.dropdown_popover().is_some(),
        "o menu não publicou o rect: {:?}",
        hero.store.get(ids::MENU)
    );
    let (x, y) = centro(&hero, ids::MENU_DUPLICATE, "Duplicate Slice");
    assert_eq!(
        hero.chrome_panel_at(x, y),
        Some(ids::MENU),
        "o menu cai no canvas"
    );
    clica(&mut hero, ids::MENU_DUPLICATE, "Duplicate Slice");
    assert_eq!(canal::take_pedidos(), [Pedido::Duplica]);
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(ids::MENU_DUPLICATE).is_none(),
        "fecha"
    );

    canal::abrir_menu(Menu {
        remover: true,
        ..onde
    });
    assenta(&mut hero, &mut ts);
    clica(
        &mut hero,
        ids::MENU_REMOVE_KEY,
        "Remove Change on This Frame",
    );
    assert_eq!(canal::take_pedidos(), [Pedido::RemoveMudanca]);

    canal::abrir_menu(onde);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, ids::MENU_DELETE, "Delete Slice");
    assert_eq!(canal::take_pedidos(), [Pedido::Apaga]);

    // `Slice Properties...` abre o cartão que o artista fechou.
    hero.store.set_collapsed(ids::SEC_PROPS, true);
    canal::abrir_menu(onde);
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(ids::BOUNDS_X).is_none(),
        "o controlo"
    );
    clica(&mut hero, ids::MENU_PROPERTIES, "Slice Properties...");
    assenta(&mut hero, &mut ts);
    assert!(hero.hit_index.rect_for(ids::BOUNDS_X).is_some());
    assert_eq!(canal::take_pedidos(), []);

    // Clicar fora fecha, sem pedido.
    canal::abrir_menu(onde);
    assenta(&mut hero, &mut ts);
    let arena = Bump::new();
    let _ = hero.handle_pointer(ptr(PointerKind::Down, 5.0, 5.0), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Up, 5.0, 5.0), &arena);
    assenta(&mut hero, &mut ts);
    assert!(hero.hit_index.rect_for(ids::MENU_DELETE).is_none());
}

/// Com a lista longa, `Slice Properties...` traz o cartão para dentro da janela.
#[test]
fn slice_properties_traz_o_cartao_para_a_vista() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut x = uma_selecionada();
    x.linhas = (0..200).map(|i| linha(i, format!("s{i}"), false)).collect();
    let mut hero = hero_com(&mut ts, x);
    let janela = hero.store.panel_rect(JANELA).expect("a janela");
    let dentro = |h: &HeroScreen| {
        h.hit_index
            .rect_for(ids::BOUNDS_X)
            .is_some_and(|r| r.y >= janela.y && r.y + r.h <= janela.y + janela.h)
    };
    assert!(!dentro(&hero), "o controlo: o cartão está abaixo da dobra");
    canal::abrir_menu(Menu {
        x: 300.0,
        y: 300.0,
        remover: false,
    });
    assenta(&mut hero, &mut ts);
    clica(&mut hero, ids::MENU_PROPERTIES, "Slice Properties...");
    assenta(&mut hero, &mut ts);
    // O tique da UI leva a rolagem viva ao alvo; aqui não há tique.
    for _ in 0..2 {
        hero.store
            .set_panel_scroll_live(JANELA, hero.store.panel_scroll_target(JANELA));
        assenta(&mut hero, &mut ts);
    }
    assert!(
        dentro(&hero),
        "{:?}",
        hero.hit_index.rect_for(ids::BOUNDS_X)
    );
}

/// `Esc` limpa a seleção: o menu aberto fecha com ela.
#[test]
fn sem_selecao_o_menu_fecha() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com(&mut ts, uma_selecionada());
    canal::abrir_menu(Menu {
        x: 300.0,
        y: 300.0,
        remover: false,
    });
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(ids::MENU_DELETE).is_some(),
        "o controlo"
    );
    let mut x = uma_selecionada();
    x.vista = None;
    canal::publish_contexto(x);
    assenta(&mut hero, &mut ts);
    assert!(hero.hit_index.rect_for(ids::MENU_DELETE).is_none());
}
