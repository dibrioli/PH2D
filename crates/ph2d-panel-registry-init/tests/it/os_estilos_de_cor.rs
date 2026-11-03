//! ⭐ **D28/D30 (CartoesUI): os estilos de cor no cartão Fill** — no hero montado: o ⠿ do cabeçalho
//! abre a lista «Color styles»; o «+» pede um estilo novo com a cor da camada; clicar numa linha
//! aplica-o; o botão direito na linha abre o menu dela; e a camada ligada mostra o NOME do estilo.
#![cfg(feature = "panel-inspector")]

use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::panel::cards::estilos::{self, Estilo, Pedido};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_text::TextSystem;
use ph2d_tool_vector::appearance::{self, BasePaint, PaintRow};

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn pinta(h: &mut HeroScreen) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..12 {
        scene.reset();
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn mao(h: &mut HeroScreen, r: Rect, button: PointerButton) {
    let arena = bumpalo::Bump::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x: r.x + r.w * 0.5,
            y: r.y + r.h * 0.5,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button,
            timestamp_ns: 0,
        };
        let evs: Vec<_> =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena)
                .to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

fn uma_camada() {
    appearance::set_base_paints(Some([BasePaint::Absent, BasePaint::Absent]));
    appearance::set_current_appearance(Some(appearance::Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![PaintRow {
            is_fill: true,
            color: [0x4C, 0x8D, 0xF0, 255],
            width: 0.0,
            enabled: true,
            opacity: 1.0,
            blend: ph2d_vec_scene::BlendMode::Normal,
            offset: [0.0, 0.0],
            dilate: 0.0,
            dilate_join: 0,
            mixed: Default::default(),
            ramp: None,
            align: ph2d_vec_scene::StrokeAlign::Centre,
        }],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
}

fn hero() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    ph2d_panel_inspector::set_current_inspector_transform(Some(
        ph2d_editor_core::screens::hero::InspectorTransformInfo {
            entity_bits: 1,
            translation: [0.0, 0.0],
            rotation_rad: 0.0,
            scale: [1.0, 1.0],
            skew_rad: [0.0, 0.0],
        },
    ));
    uma_camada();
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    pinta(&mut h);
    h
}

fn rect(h: &HeroScreen, id: NodeId, que: &str) -> Rect {
    h.hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{que} não se pintou"))
}

#[test]
fn the_styles_icon_opens_the_list_and_its_rows_act() {
    let mut h = hero();
    let _ = estilos::take_pedidos();
    estilos::publica(
        vec![Estilo {
            id: 7,
            name: "Brand orange".into(),
            rgba: [0xE8, 0x74, 0x3B, 0xFF],
            usos: 0,
        }],
        vec![None],
    );
    let botao = estilos::botao_de(ph2d_panel_inspector::CARD_FILL);
    let r = rect(&h, botao, "o ⠿ do Fill");
    mao(&mut h, r, PointerButton::Primary);
    pinta(&mut h);
    // A lista nasce por BAIXO do cabeçalho do Fill (maquete E), nunca sobre o título.
    let cabeca = rect(
        &h,
        ph2d_editor_core::ids::grip_de(ph2d_panel_inspector::CARD_FILL),
        "o cabeçalho do Fill",
    );
    let r = rect(&h, estilos::MAIS, "o «+» da lista");
    assert!(
        r.y >= cabeca.y + cabeca.h,
        "a lista abriu sobre o título: «+» em y={} e o cabeçalho acaba em {}",
        r.y,
        cabeca.y + cabeca.h
    );
    // O «+» pede um estilo novo para a camada 0.
    mao(&mut h, r, PointerButton::Primary);
    let p = estilos::take_pedidos();
    assert!(
        matches!(p.as_slice(), [Pedido::Novo { camada: 0, .. }]),
        "o «+» pediu {p:?}"
    );
    pinta(&mut h);
    // O botão direito numa linha abre o menu DELA.
    let linha = rect(&h, estilos::linha_de(7), "a linha Brand orange");
    mao(&mut h, linha, PointerButton::Secondary);
    assert_eq!(
        h.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::ColorStyleRow { style: 7 }),
        "o botão direito não abriu o menu do estilo"
    );
    // A linha «Duplicate» do menu pede a cópia com o nome «… copy».
    h.store.close_context_menu();
    h.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(
        ph2d_editor_core::ids::CTX_MENU_STYLE_DUPLICATE,
    ));
    assert_eq!(
        estilos::take_pedidos(),
        vec![Pedido::Duplicar {
            estilo: 7,
            nome: "Brand orange copy".into()
        }]
    );
    pinta(&mut h);
    // Clicar na linha aplica o estilo à camada e fecha a lista.
    let linha = rect(&h, estilos::linha_de(7), "a linha Brand orange");
    mao(&mut h, linha, PointerButton::Primary);
    assert_eq!(
        estilos::take_pedidos(),
        vec![Pedido::Aplicar {
            estilo: 7,
            camada: 0
        }]
    );
    pinta(&mut h);
    assert_eq!(
        h.hit_index.rect_for(estilos::linha_de(7)),
        None,
        "a lista não fechou"
    );
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

/// A camada ligada mostra o NOME do estilo e a corrente partida solta-a (maquete F).
#[test]
fn a_linked_layer_shows_the_style_name_and_detaches() {
    let mut h = hero();
    let _ = estilos::take_pedidos();
    estilos::publica(
        vec![Estilo {
            id: 3,
            name: "Ink".into(),
            rgba: [0x1F, 0x24, 0x30, 0xFF],
            usos: 1,
        }],
        vec![Some(3)],
    );
    pinta(&mut h);
    let solta = rect(&h, estilos::solta_de(0), "a corrente partida");
    assert!(
        h.hit_index
            .rect_for(ph2d_panel_inspector::cards_hex_id(0))
            .is_none(),
        "a camada ligada ainda oferece o código para editar"
    );
    mao(&mut h, solta, PointerButton::Primary);
    assert_eq!(estilos::take_pedidos(), vec![Pedido::Soltar { camada: 0 }]);
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

/// ⭐ Maquete F: com o nome do estilo inteiro, o rótulo «Opacity (%)» QUEBRA em duas linhas em vez
/// de sair «Opacity…».
#[test]
fn a_linked_layer_wraps_the_opacity_label_instead_of_cutting_it() {
    let mut h = hero();
    estilos::publica(
        vec![Estilo {
            id: 7,
            name: "Brand orange".into(),
            rgba: [0xE8, 0x74, 0x3B, 0xFF],
            usos: 1,
        }],
        vec![Some(7)],
    );
    let ((), medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| pinta(&mut h));
    let opacidade: Vec<_> = medidos
        .iter()
        .filter(|m| m.texto.starts_with("Opacity") || m.texto == "(%)")
        .collect();
    assert!(!opacidade.is_empty(), "o rótulo da opacidade não se pintou");
    for m in &opacidade {
        assert!(
            m.coube(),
            "«{}» saiu «{}» ({} px)",
            m.texto,
            m.pintado,
            m.largura
        );
    }
    assert!(
        medidos
            .iter()
            .any(|m| m.texto == "Brand orange" && m.coube()),
        "o nome do estilo não saiu inteiro"
    );
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

fn busca(h: &mut HeroScreen, q: &str) {
    use ph2d_editor_core::interaction::InteractiveState;
    if let Some(InteractiveState::TextInput { text, caret, .. }) = h.store.get_mut(estilos::BUSCA) {
        q.clone_into(text);
        *caret = q.len();
    }
}

/// ⭐ Maquete E: a caixa «Search...» no topo da lista filtra pelo NOME (sem caixa alta/baixa); a
/// busca vazia mostra todos; ao reabrir a lista a busca recomeça vazia.
#[test]
fn the_search_box_filters_the_style_rows_by_name() {
    let mut h = hero();
    let _ = estilos::take_pedidos();
    let estilo = |id, name: &str| Estilo {
        id,
        name: name.into(),
        rgba: [0xE8, 0x74, 0x3B, 0xFF],
        usos: 0,
    };
    estilos::publica(
        vec![estilo(7, "Brand orange"), estilo(8, "Grass")],
        vec![None],
    );
    let botao = estilos::botao_de(ph2d_panel_inspector::CARD_FILL);
    let r = rect(&h, botao, "o ⠿ do Fill");
    mao(&mut h, r, PointerButton::Primary);
    pinta(&mut h);
    let caixa = rect(&h, estilos::BUSCA, "a caixa Search");
    mao(&mut h, caixa, PointerButton::Primary);
    assert_eq!(
        h.store.focus_id(),
        Some(estilos::BUSCA),
        "a busca não ganhou o foco"
    );
    assert!(
        estilos::take_pedidos().is_empty(),
        "clicar na busca pediu algo"
    );
    busca(&mut h, "ORAN");
    pinta(&mut h);
    assert!(
        h.hit_index.rect_for(estilos::linha_de(7)).is_some(),
        "«Brand orange» sumiu"
    );
    assert_eq!(
        h.hit_index.rect_for(estilos::linha_de(8)),
        None,
        "«Grass» não saiu"
    );
    busca(&mut h, "");
    pinta(&mut h);
    assert!(
        h.hit_index.rect_for(estilos::linha_de(8)).is_some(),
        "busca vazia esconde"
    );
    // Fecha e reabre: a busca recomeça vazia.
    busca(&mut h, "zzz");
    let r = rect(&h, botao, "o ⠿ do Fill");
    mao(&mut h, r, PointerButton::Primary);
    pinta(&mut h);
    assert_eq!(
        h.hit_index.rect_for(estilos::linha_de(7)),
        None,
        "o ⠿ não fechou a lista"
    );
    // (O 2.º clique no mesmo sítio seria duplo clique: o reabrir mostra o que a lista fechada guardou.)
    estilos::abre_o_primeiro();
    pinta(&mut h);
    assert!(
        h.hit_index.rect_for(estilos::linha_de(7)).is_some(),
        "a busca ficou ao reabrir"
    );
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}
