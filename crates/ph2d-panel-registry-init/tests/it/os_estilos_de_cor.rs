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

/// O clique como a shell o faz: a porta da pressão ANTES do despacho, nas duas pontas.
fn clique(h: &mut HeroScreen, (x, y): (f32, f32), button: PointerButton) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static RELOGIO: AtomicU64 = AtomicU64::new(1);
    let arena = bumpalo::Bump::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button,
            // Um segundo entre cliques: dois no mesmo sítio não viram duplo clique.
            timestamp_ns: u128::from(RELOGIO.fetch_add(1, Ordering::Relaxed)) * 1_000_000_000,
        };
        h.premido_fisico(ev);
        let evs = h.handle_pointer(ev, &arena).to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
    pinta(h);
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// Um ponto do canvas onde não há nada: é aqui que o dono clicou.
fn vazio(h: &HeroScreen) -> (f32, f32) {
    let c = h.last_layout.expect("houve um quadro").canvas;
    // O meio da largura: a coluna do Inspector (e o «Edit style» aberto) fica à direita, sobre o canvas.
    let p = (c.x + c.w * 0.5, c.y + c.h * 0.9);
    assert_eq!(
        h.hit_index.hit_with_rect(p.0, p.1),
        None,
        "o ponto não está vazio"
    );
    p
}

fn um_estilo() {
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
}

fn abre(h: &mut HeroScreen) {
    let r = rect(
        h,
        estilos::botao_de(ph2d_panel_inspector::CARD_FILL),
        "o ⠿ do Fill",
    );
    clique(h, centro(r), PointerButton::Primary);
    assert!(aberta(h), "o ⠿ não abriu a lista");
}

fn aberta(h: &HeroScreen) -> bool {
    h.hit_index.rect_for(estilos::linha_de(7)).is_some()
}

/// ⭐ Dono 03/11: «dropdown color styles não fecha sozinho ao clicar fora dele».
/// *Mutações: a porta vazia; sem a excepção do ⠿; sem a da lista; sem a do menu; o botão do meio.*
#[test]
fn a_press_outside_closes_the_styles_list() {
    let mut h = hero();
    um_estilo();
    let p = vazio(&h);

    abre(&mut h);
    clique(&mut h, p, PointerButton::Primary);
    assert!(!aberta(&h), "o clique no vazio não fechou a lista");

    // Noutro widget: fecha também (o clique segue para ele).
    abre(&mut h);
    let outro = rect(
        &h,
        ph2d_editor_core::ids::grip_de(ph2d_panel_inspector::CARD_TRANSFORM),
        "o título do Transform",
    );
    clique(&mut h, centro(outro), PointerButton::Primary);
    assert!(!aberta(&h), "o clique noutro cartão não fechou a lista");

    // O próprio ⠿ fecha, e não reabre no mesmo clique.
    abre(&mut h);
    let botao = rect(
        &h,
        estilos::botao_de(ph2d_panel_inspector::CARD_FILL),
        "o ⠿",
    );
    clique(&mut h, centro(botao), PointerButton::Primary);
    assert!(!aberta(&h), "o ⠿ reabriu a lista no clique que a fechava");

    // Dentro dela não é fora: a busca, o fundo e o direito numa linha.
    abre(&mut h);
    let busca = rect(&h, estilos::BUSCA, "a busca");
    clique(&mut h, centro(busca), PointerButton::Primary);
    assert!(aberta(&h), "o clique na busca fechou a lista");
    let mais = rect(&h, estilos::MAIS, "o «+»");
    clique(
        &mut h,
        (busca.x + 4.0, mais.y + 2.0),
        PointerButton::Primary,
    );
    assert!(aberta(&h), "o clique no título da lista fechou-a");
    h.store.set_focus(None);

    // A mão que move a vista não fecha.
    clique(&mut h, p, PointerButton::Middle);
    assert!(aberta(&h), "o botão do meio fechou a lista");

    // O menu da linha: a pressão nele é do menu, e o Apply ainda sabe a camada.
    let linha = rect(&h, estilos::linha_de(7), "a linha");
    clique(&mut h, centro(linha), PointerButton::Secondary);
    assert!(aberta(&h), "o direito numa linha fechou a lista");
    let apply = rect(
        &h,
        ph2d_editor_core::ids::CTX_MENU_STYLE_APPLY,
        "o Apply do menu",
    );
    clique(&mut h, centro(apply), PointerButton::Primary);
    assert_eq!(
        estilos::take_pedidos(),
        vec![Pedido::Aplicar {
            estilo: 7,
            camada: 0
        }],
        "o Apply do menu perdeu a camada"
    );
    assert!(!aberta(&h), "o Apply não fechou a lista");

    // O direito fora também fecha.
    abre(&mut h);
    clique(&mut h, p, PointerButton::Secondary);
    h.store.close_context_menu();
    pinta(&mut h);
    assert!(!aberta(&h), "o direito no vazio não fechou a lista");
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

/// O «Edit style» fecha junto, e o que estava escrito sem Enter não se perde.
/// *Mutação: sem o `Submit` antes de fechar.*
#[test]
fn a_press_outside_closes_the_style_editor_and_keeps_what_was_typed() {
    let mut h = hero();
    um_estilo();
    let p = vazio(&h);
    for (caixa, escrito, esperado) in [
        (
            estilos::NOME,
            "Novo nome",
            Pedido::Renomear {
                estilo: 7,
                nome: "Novo nome".into(),
            },
        ),
        (
            estilos::HEX,
            "112233",
            Pedido::Cor {
                estilo: 7,
                rgba: [0x11, 0x22, 0x33, 0xFF],
            },
        ),
    ] {
        estilos::abre_o_editor_do_primeiro();
        pinta(&mut h);
        let r = rect(&h, caixa, "a caixa do «Edit style»");
        clique(&mut h, centro(r), PointerButton::Primary);
        assert!(
            h.hit_index.rect_for(caixa).is_some(),
            "o clique na caixa fechou o «Edit style»"
        );
        assert_eq!(h.store.focus_id(), Some(caixa), "a caixa não ganhou o foco");
        let Some(ph2d_editor_core::interaction::InteractiveState::TextInput {
            text, caret, ..
        }) = h.store.get_mut(caixa)
        else {
            panic!("a caixa não é de texto");
        };
        escrito.clone_into(text);
        *caret = escrito.len();
        clique(&mut h, p, PointerButton::Primary);
        assert_eq!(
            estilos::take_pedidos(),
            vec![esperado],
            "o escrito perdeu-se"
        );
        assert_eq!(
            h.hit_index.rect_for(caixa),
            None,
            "o «Edit style» não fechou"
        );
        assert!(!aberta(&h), "a lista não fechou");
        assert_eq!(
            h.store.focus_id(),
            None,
            "o foco ficou numa caixa invisível"
        );
    }
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

/// A busca não fica com o teclado depois de a lista fechar. *Mutação: sem soltar o foco.*
#[test]
fn a_press_outside_takes_the_keyboard_from_the_search_box() {
    let mut h = hero();
    um_estilo();
    let p = vazio(&h);
    abre(&mut h);
    let busca = rect(&h, estilos::BUSCA, "a busca");
    clique(&mut h, centro(busca), PointerButton::Primary);
    assert_eq!(h.store.focus_id(), Some(estilos::BUSCA));
    clique(&mut h, p, PointerButton::Primary);
    assert_eq!(
        h.store.focus_id(),
        None,
        "a busca fechada ficou com o teclado"
    );
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

/// O cartão SOLTO no canvas: é o da captura do dono.
#[test]
fn a_press_outside_closes_the_list_of_a_loose_card() {
    let mut h = hero();
    um_estilo();
    super::cartoes_mao::solta(&mut h, ph2d_panel_inspector::CARD_FILL);
    pinta(&mut h);
    let p = vazio(&h);
    abre(&mut h);
    clique(&mut h, p, PointerButton::Primary);
    assert!(!aberta(&h), "a lista do cartão solto não fechou");
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

/// A mesma família: a lista Blend do cartão Appearance. *Mutação: a pressão na opção conta como fora.*
#[test]
fn a_press_outside_closes_a_card_dropdown_but_not_on_its_options() {
    use ph2d_editor_core::panel::cards::dropdown;
    let mut h = hero();
    let p = vazio(&h);
    let dd = ph2d_tool_registry::hash_node_id("cards.insp.blend.dd");
    let caixa = rect(&h, dd, "a caixa do Blend");
    clique(&mut h, centro(caixa), PointerButton::Primary);
    assert_eq!(dropdown::open(), Some(dd), "a caixa não abriu");
    clique(&mut h, p, PointerButton::Primary);
    assert_eq!(
        dropdown::open(),
        None,
        "o clique no vazio não fechou o Blend"
    );

    // A própria caixa fecha, e não reabre no mesmo clique.
    let caixa = rect(&h, dd, "a caixa do Blend");
    clique(&mut h, centro(caixa), PointerButton::Primary);
    assert_eq!(dropdown::open(), Some(dd), "a caixa não abriu");
    let caixa = rect(&h, dd, "a caixa do Blend");
    clique(&mut h, centro(caixa), PointerButton::Primary);
    assert_eq!(
        dropdown::open(),
        None,
        "a caixa reabriu a lista que fechava"
    );

    // Os cartões ainda assentam nos primeiros quadros: a caixa lê-se de novo.
    let caixa = rect(&h, dd, "a caixa do Blend");
    clique(&mut h, centro(caixa), PointerButton::Primary);
    assert_eq!(dropdown::open(), Some(dd), "a caixa não reabriu");
    let caixa = rect(&h, dd, "a caixa do Blend");
    let opcao = (centro(caixa).0, caixa.y + caixa.h + 12.0);
    let (id, _) = h
        .hit_index
        .hit_with_rect(opcao.0, opcao.1)
        .expect("a 1.ª opção");
    assert_ne!(id, dd);
    let down = PointerEvent {
        x: opcao.0,
        y: opcao.1,
        pressure: 1.0,
        kind: PointerKind::Down,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    h.premido_fisico(down);
    assert_eq!(
        dropdown::open(),
        Some(dd),
        "premir numa opção fechou a lista antes do clique"
    );
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

/// ⛔ A pressão no canvas é tomada pelos ramos da shell e nunca chega ao despacho: a porta tem de
/// estar antes do primeiro `return` da porta única do clique. *Mutação: mover a chamada para depois.*
#[test]
fn the_shell_calls_the_press_door_before_any_canvas_branch() {
    let fonte = include_str!("../../../../shells/desktop/src/input_dispatch.rs");
    let corpo = &fonte[fonte.find("fn pointer_button").expect("a porta do clique")..];
    let porta = corpo
        .find("ramo_aperto_solta_teclado(state, evt)")
        .expect("a shell não chama o ramo da pressão");
    let primeiro = corpo.find("return;").expect("o primeiro ramo");
    assert!(
        porta < primeiro,
        "a porta da pressão está depois de um ramo que devolve"
    );
    let ramo =
        include_str!("../../../../shells/desktop/src/input_dispatch/despacho_clique_prologo.rs");
    let ramo = &ramo[ramo.find("fn ramo_aperto_solta_teclado").expect("o ramo")..];
    let fim = ramo.find("\n    }\n").expect("o fim do ramo");
    assert!(
        ramo[..fim].contains("premido_fisico(evt)"),
        "o ramo não chama a porta da pressão"
    );
}

/// A shell dá o ponto da JANELA: a qualquer escala, premir dentro da lista continua a ser dentro.
/// *Mutação: sem o `to_logical` do `premido_fisico`.*
#[test]
fn a_press_outside_is_read_in_window_pixels() {
    let mut h = hero();
    um_estilo();
    abre(&mut h);
    h.escala_do_ecra = 2.0;
    let r = h
        .escala()
        .rect_to_physical(rect(&h, estilos::BUSCA, "a busca"));
    h.premido_fisico(PointerEvent {
        x: centro(r).0,
        y: centro(r).1,
        pressure: 1.0,
        kind: PointerKind::Down,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    });
    assert!(
        estilos::aberta_em(ph2d_panel_inspector::CARD_FILL),
        "o ponto da janela foi lido como lógico: premir na busca fechou a lista"
    );
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}
