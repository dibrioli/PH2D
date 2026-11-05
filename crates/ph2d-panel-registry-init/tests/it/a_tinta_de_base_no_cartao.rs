//! ⭐ **A tinta de BASE nos cartões Fill e Stroke** (F44b, `c47127e2da`) — no hero montado, com cliques
//! reais: a forma de todo dia não tem camada nenhuma, e o cartão tem de mostrar a cor dela.
#![cfg(feature = "panel-inspector")]

use super::cartoes_mao::{InspectorNoHost, centro, clica, mao, pinta};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::estilos;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_host::PointerKind;
use ph2d_panel_inspector as insp;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::appearance::{self, Appearance, BasePaint, PaintRow};
use ph2d_tool_vector::ids as vids;
use ph2d_vec_scene::{BlendMode, MAX_PAINT_LAYERS, StackItem, StrokeAlign};

const AZUL: [u8; 4] = [0x4C, 0x8D, 0xF0, 255];
const LARANJA: [u8; 4] = [0xE8, 0x74, 0x3B, 128];
const VERDE: [u8; 4] = [0x2F, 0xA8, 0x5A, 255];
const HEX_FILL: usize = MAX_PAINT_LAYERS;
const HEX_STROKE: usize = HEX_FILL + 1;

fn camada(is_fill: bool, color: [u8; 4], opacity: f32) -> PaintRow {
    PaintRow {
        is_fill,
        color,
        width: 3.0,
        enabled: true,
        opacity,
        blend: BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 0,
        mixed: Default::default(),
        ramp: None,
        align: StrokeAlign::Centre,
    }
}

/// Publica a forma: as duas bases, a pilha de camadas e a ordem (vazia = a neutra).
fn forma(base: [BasePaint; 2], layers: Vec<PaintRow>, order: Vec<StackItem>) {
    appearance::set_base_paints(Some(base));
    appearance::set_current_vector_style(Some(VectorStyleSnapshot {
        fill: AZUL,
        stroke: LARANJA,
        ..Default::default()
    }));
    appearance::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: BlendMode::Normal,
        layers,
        order,
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
    }));
}

/// Larga o que a thread publicou, mesmo quando o teste cai.
struct Limpa;
impl Drop for Limpa {
    fn drop(&mut self) {
        appearance::set_base_paints(None);
        appearance::set_current_vector_style(None);
        appearance::set_current_appearance(None);
        insp::set_current_inspector_transform(None);
    }
}

fn publica_objeto() -> Limpa {
    insp::set_current_inspector_sprite(None);
    insp::set_current_inspector_transform(Some(
        ph2d_editor_core::screens::hero::InspectorTransformInfo {
            entity_bits: 1,
            translation: [0.0, 0.0],
            rotation_rad: 0.0,
            scale: [1.0, 1.0],
            skew_rad: [0.0, 0.0],
        },
    ));
    estilos::publica(Vec::new(), Vec::new());
    Limpa
}

fn hero() -> (HeroScreen, Limpa) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    insp::set_cards_on(true);
    let limpa = publica_objeto();
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    pinta(&mut h, 6);
    (h, limpa)
}

fn y_de(h: &HeroScreen, id: NodeId, que: &str) -> f32 {
    h.hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{que} não se pintou"))
        .y
}

fn pintado(h: &HeroScreen, id: NodeId) -> bool {
    h.hit_index.rect_for(id).is_some()
}

fn hex_na_caixa(store: &ph2d_editor_core::interaction::WidgetStore, i: usize) -> String {
    match store.get(insp::cards_hex_id(i)) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        o => panic!("a caixa de hex {i} é {o:?}"),
    }
}

/// O clique no id chega ao barramento como o `Click` que a ponte do Vector consome.
fn clique_chega(h: &mut HeroScreen, id: NodeId, que: &str) {
    let _ = h.bus.drain().count();
    clica(h, id);
    let chegou = h
        .bus
        .drain()
        .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id));
    assert!(chegou, "{que}: o clique não chegou ao barramento");
}

fn path_tags(h: &mut HeroScreen) -> Vec<u8> {
    pinta(h, 6);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = ph2d_text::TextSystem::without_system_fonts();
    ph2d_editor_core::screens::hero::paint_hero_screen(
        h,
        ph2d_editor_core::zones::Rect {
            x: 0.0,
            y: 0.0,
            w: 1600.0,
            h: 900.0,
        },
        &mut scene,
        &mut text,
    );
    scene
        .inner()
        .encoding()
        .path_tags
        .iter()
        .map(|t| t.0)
        .collect()
}

/// A forma de todo dia: uma linha em cada cartão, com o código da cor dela, e camada nenhuma.
/// *Mutação: `amostra_de` sempre o id da camada ⇒ a amostra da base não se pinta.*
#[test]
fn a_shape_with_base_paints_shows_one_row_in_each_card() {
    let (mut h, _l) = hero();
    forma([BasePaint::Shown, BasePaint::Shown], Vec::new(), Vec::new());
    let ((), medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| pinta(&mut h, 12));
    let cartao_fill = y_de(&h, core_ids::grip_de(insp::CARD_FILL), "o título do Fill");
    let cartao_stroke = y_de(
        &h,
        core_ids::grip_de(insp::CARD_STROKE),
        "o título do Stroke",
    );
    let fill = y_de(&h, vids::VECTOR_FILL_SWATCH, "a amostra do Fill");
    let stroke = y_de(&h, vids::VECTOR_STROKE_SWATCH, "a amostra do Stroke");
    assert!(
        cartao_fill < fill && fill < cartao_stroke && cartao_stroke < stroke,
        "cada linha no seu cartão: Fill {cartao_fill} · {fill} · Stroke {cartao_stroke} · {stroke}"
    );
    for (texto, i) in [("4C8DF0", HEX_FILL), ("E8743B", HEX_STROKE)] {
        assert_eq!(hex_na_caixa(&h.store, i), texto);
        assert!(
            medidos.iter().any(|m| m.texto == texto && m.coube()),
            "o código {texto} não saiu inteiro no cartão"
        );
    }
    assert!(
        (0..MAX_PAINT_LAYERS).all(|i| !pintado(&h, vids::vector_paint_swatch_id(i))),
        "sem camadas, o cartão pintou uma linha de camada"
    );
}

/// O olho e o «−» da base dão no MESMO `Click` do painel Vector: é o que a ponte traduz em gesto.
#[test]
fn the_base_eye_and_minus_reach_the_bus_like_the_vector_panel() {
    let (mut h, _l) = hero();
    forma([BasePaint::Shown, BasePaint::Shown], Vec::new(), Vec::new());
    pinta(&mut h, 12);
    for (id, que) in [
        (vids::VECTOR_FILL_EYE, "o olho do Fill"),
        (vids::VECTOR_FILL_PRESENT, "o «−» do Fill"),
        (vids::VECTOR_STROKE_EYE, "o olho do Stroke"),
        (vids::VECTOR_STROKE_PRESENT, "o «−» do Stroke"),
    ] {
        clique_chega(&mut h, id, que);
    }
}

/// `Hidden` fecha o olho e a linha fica; `Absent` tira a linha.
/// *Mutações: `base_row` a ignorar `Absent` ⇒ a linha fica; `visivel` sempre `true` ⇒ o olho igual.*
#[test]
fn hidden_closes_the_eye_and_absent_removes_the_row() {
    let (mut h, _l) = hero();
    forma([BasePaint::Shown, BasePaint::Shown], Vec::new(), Vec::new());
    let aberto = path_tags(&mut h);
    assert_eq!(aberto, path_tags(&mut h), "dois quadros iguais diferem");
    forma(
        [BasePaint::Hidden, BasePaint::Shown],
        Vec::new(),
        Vec::new(),
    );
    let fechado = path_tags(&mut h);
    assert_ne!(aberto, fechado, "o olho escondido pinta-se igual ao aberto");
    for id in [
        vids::VECTOR_FILL_SWATCH,
        vids::VECTOR_FILL_EYE,
        vids::VECTOR_FILL_PRESENT,
    ] {
        assert!(pintado(&h, id), "`Hidden` tirou a linha: {id:?}");
    }
    forma(
        [BasePaint::Absent, BasePaint::Shown],
        Vec::new(),
        Vec::new(),
    );
    pinta(&mut h, 12);
    for id in [vids::VECTOR_FILL_SWATCH, vids::VECTOR_FILL_EYE] {
        assert!(!pintado(&h, id), "`Absent` deixou a linha: {id:?}");
    }
    assert!(pintado(&h, vids::VECTOR_STROKE_SWATCH), "o Stroke sumiu");
    // Com uma camada o cartão tem corpo: quem tira a linha é a própria linha da base, e a camada fica.
    forma(
        [BasePaint::Absent, BasePaint::Shown],
        vec![camada(true, VERDE, 1.0)],
        Vec::new(),
    );
    pinta(&mut h, 12);
    assert!(
        pintado(&h, vids::vector_paint_swatch_id(0)),
        "a camada sumiu com a base"
    );
    assert!(
        !pintado(&h, vids::VECTOR_FILL_SWATCH) && !pintado(&h, vids::VECTOR_FILL_EYE),
        "`Absent` com camada deixou a linha da base"
    );
}

/// O «+» do Fill: dá a BASE quando ela falta; senão uma camada; com a pilha cheia, nada.
#[test]
fn the_plus_gives_the_base_then_a_layer_and_stops_when_full() {
    let cheia = || vec![camada(true, VERDE, 1.0); MAX_PAINT_LAYERS];
    let no_cabecalho = |h: &HeroScreen, id: NodeId| {
        let cab = h
            .hit_index
            .rect_for(core_ids::grip_de(insp::CARD_FILL))
            .expect("o título do Fill");
        let r = h.hit_index.rect_for(id).expect("o «+» do Fill");
        assert!(
            r.y + r.h > cab.y && r.y < cab.y + cab.h,
            "o «+» {r:?} não está no cabeçalho {cab:?}"
        );
    };
    // A base falta, sem camadas de fill (o «+» do cartão vazio) e com uma (o «+» entre as acções).
    // (Um hero por caso: dois cliques no mesmo sítio seriam um duplo clique.)
    for layers in [Vec::new(), vec![camada(true, VERDE, 1.0)]] {
        let (mut h, _l) = hero();
        forma([BasePaint::Absent, BasePaint::Shown], layers, Vec::new());
        pinta(&mut h, 12);
        no_cabecalho(&h, vids::VECTOR_FILL_PRESENT);
        assert!(
            !pintado(&h, vids::VECTOR_PAINT_ADD_FILL),
            "com a base em falta o «+» dá a base, não uma camada"
        );
        clique_chega(&mut h, vids::VECTOR_FILL_PRESENT, "o «+» do Fill");
    }
    // A base presente: uma camada.
    let (mut h, _l) = hero();
    forma([BasePaint::Shown, BasePaint::Shown], Vec::new(), Vec::new());
    pinta(&mut h, 12);
    no_cabecalho(&h, vids::VECTOR_PAINT_ADD_FILL);
    // Pilha cheia: nenhum «+»; a base em falta ainda se pode dar.
    forma([BasePaint::Shown, BasePaint::Shown], cheia(), Vec::new());
    pinta(&mut h, 12);
    assert!(
        !pintado(&h, vids::VECTOR_PAINT_ADD_FILL) && !pintado(&h, vids::VECTOR_PAINT_ADD_STROKE),
        "com a pilha cheia ainda há «+»"
    );
    forma([BasePaint::Absent, BasePaint::Shown], cheia(), Vec::new());
    pinta(&mut h, 12);
    no_cabecalho(&h, vids::VECTOR_FILL_PRESENT);
}

/// A ordem é a da pilha PUBLICADA, com o topo em cima (`order` vai do chão ao topo).
/// *Mutação: `pilha` sem o `.rev()` ⇒ os dois casos trocam.*
#[test]
fn the_rows_follow_the_published_stack_with_the_top_above() {
    use StackItem::{BaseFill, BaseStroke, Entry};
    let (mut h, _l) = hero();
    let camadas = || vec![camada(true, VERDE, 1.0)];
    let linhas = |h: &HeroScreen| {
        (
            y_de(h, vids::VECTOR_FILL_SWATCH, "a linha da base"),
            y_de(h, vids::vector_paint_swatch_id(0), "a linha da camada"),
        )
    };
    forma(
        [BasePaint::Shown, BasePaint::Shown],
        camadas(),
        vec![BaseFill, BaseStroke, Entry(0)],
    );
    pinta(&mut h, 12);
    let (base, cam) = linhas(&h);
    assert!(cam < base, "camada no topo: camada {cam} · base {base}");
    forma(
        [BasePaint::Shown, BasePaint::Shown],
        camadas(),
        vec![Entry(0), BaseFill, BaseStroke],
    );
    pinta(&mut h, 12);
    let (base, cam) = linhas(&h);
    assert!(base < cam, "base no topo: camada {cam} · base {base}");
}

/// Digitar um hex é ESCOLHER a cor: o seletor abre na amostra daquela linha e fica com a cor; quando a
/// forma a mostra, ele fecha. 6 dígitos guardam a alfa; 8 trazem a sua.
#[test]
fn typing_a_hex_opens_the_picker_on_the_swatch_with_that_color() {
    let (mut h, _l) = hero();
    forma(
        [BasePaint::Shown, BasePaint::Shown],
        vec![camada(true, VERDE, 1.0)],
        Vec::new(),
    );
    pinta(&mut h, 12);
    let seletor = core_ids::INSP_BLENDER_PICKER;
    let escreve_e_confirma = |h: &mut HeroScreen, i: usize, codigo: &str| {
        let caixa = insp::cards_hex_id(i);
        let c = centro(h.hit_index.rect_for(caixa).expect("a caixa do código"));
        mao(h, PointerKind::Down, c);
        mao(h, PointerKind::Up, c);
        assert_eq!(h.store.focus_id(), Some(caixa), "a caixa não ganhou o foco");
        if let Some(InteractiveState::TextInput { text, caret, .. }) = h.store.get_mut(caixa) {
            codigo.clone_into(text);
            *caret = codigo.len();
        }
        h.apply_event(WidgetEvent::Submit(caixa));
    };
    let cor = |h: &HeroScreen| h.store.blender_picker(seletor).map(|(v, ..)| v.rgba);

    escreve_e_confirma(&mut h, HEX_STROKE, "#112233");
    assert_eq!(h.store.picker_target(), Some(vids::VECTOR_STROKE_SWATCH));
    assert_eq!(cor(&h), Some([0x11, 0x22, 0x33, LARANJA[3]]));
    h.store.set_picker_target(None);
    pinta(&mut h, 2);

    escreve_e_confirma(&mut h, 0, "11223344");
    assert_eq!(
        h.store.picker_target(),
        Some(vids::vector_paint_swatch_id(0))
    );
    assert_eq!(cor(&h), Some([0x11, 0x22, 0x33, 0x44]));
    h.store.set_picker_target(None);
    pinta(&mut h, 2);

    escreve_e_confirma(&mut h, HEX_FILL, "A1B2C3");
    assert_eq!(h.store.picker_target(), Some(vids::VECTOR_FILL_SWATCH));
    assert_eq!(cor(&h), Some([0xA1, 0xB2, 0xC3, 255]));
    // A forma passa a mostrar a cor (a ponte aplicou-a): o seletor fecha sozinho.
    appearance::set_current_vector_style(Some(VectorStyleSnapshot {
        fill: [0xA1, 0xB2, 0xC3, 255],
        stroke: LARANJA,
        ..Default::default()
    }));
    pinta(&mut h, 2);
    assert_eq!(h.store.picker_target(), None, "o seletor não fechou");
}

/// O que a forma tem aparece nas caixas: a opacidade da base e a de cada camada.
#[test]
fn the_row_boxes_are_seeded_from_the_shape() {
    let (mut h, _l) = hero();
    forma(
        [BasePaint::Shown, BasePaint::Shown],
        vec![camada(true, VERDE, 0.25)],
        Vec::new(),
    );
    pinta(&mut h, 12);
    let valor = |id| h.store.number_value(id).expect("a caixa está registada");
    assert!((valor(vids::VECTOR_FILL_OPACITY_NUM) - 100.0).abs() < 1e-3);
    let meio = f64::from(LARANJA[3]) * 100.0 / 255.0;
    assert!((valor(vids::VECTOR_STROKE_OPACITY_NUM) - meio).abs() < 1e-3);
    assert!((valor(vids::vector_paint_row_opacity_id(0)) - 25.0).abs() < 1e-3);
    for id in [
        vids::VECTOR_FILL_OPACITY_NUM,
        vids::VECTOR_STROKE_OPACITY_NUM,
        vids::vector_paint_row_opacity_id(0),
    ] {
        assert!(pintado(&h, id), "a caixa {id:?} não se pintou");
    }
}

/// ⛔ A caixa com foco não é pisada pela semente; sem foco, volta à forma.
/// *Mutação: `semeia_as_linhas` sem a guarda do foco ⇒ o número e o código da mão somem.*
#[test]
fn the_seed_does_not_step_on_the_focused_box() {
    let _l = publica_objeto();
    forma(
        [BasePaint::Shown, BasePaint::Shown],
        vec![camada(true, VERDE, 0.25)],
        Vec::new(),
    );
    let mut p = InspectorNoHost::novo();
    p.pinta();
    p.pinta();
    for (id, mundo) in [
        (vids::VECTOR_FILL_OPACITY_NUM, 100.0),
        (vids::vector_paint_row_opacity_id(0), 25.0),
    ] {
        let store = p.host.store_mut();
        store.set_focus(Some(id));
        store.set_number_value(id, 77.0);
        p.pinta();
        assert_eq!(p.host.store().number_value(id), Some(77.0), "pisou {id:?}");
        p.host.store_mut().set_focus(None);
        p.pinta();
        assert_eq!(p.host.store().number_value(id), Some(mundo), "{id:?}");
    }
    let caixa = insp::cards_hex_id(HEX_FILL);
    let store = p.host.store_mut();
    store.set_focus(Some(caixa));
    if let Some(InteractiveState::TextInput { text, .. }) = store.get_mut(caixa) {
        "ABC".clone_into(text);
    }
    p.pinta();
    assert_eq!(hex_na_caixa(p.host.store(), HEX_FILL), "ABC", "pisou o hex");
    p.host.store_mut().set_focus(None);
    p.pinta();
    assert_eq!(hex_na_caixa(p.host.store(), HEX_FILL), "4C8DF0");
}

fn publica_um_estilo(ligados: Vec<Option<u32>>) {
    estilos::publica(
        vec![estilos::Estilo {
            id: 7,
            name: "Brand orange".into(),
            rgba: [0xE8, 0x74, 0x3B, 255],
            usos: 1,
        }],
        ligados,
    );
}

/// ⭐ D28/D30: a forma de todo dia (só bases) tem o ⠿ nos DOIS cartões; o clique abre a lista e
/// escolher pede o estilo para a BASE daquele cartão.
/// *Mutação: o ⠿ só com camada ⇒ não se pinta.*
#[test]
fn a_shape_with_only_base_paints_has_the_styles_icon_in_both_cards() {
    let (mut h, _l) = hero();
    let _ = estilos::take_pedidos();
    forma([BasePaint::Shown, BasePaint::Shown], Vec::new(), Vec::new());
    publica_um_estilo(Vec::new());
    pinta(&mut h, 6);
    for (card, linha) in [(insp::CARD_FILL, HEX_FILL), (insp::CARD_STROKE, HEX_STROKE)] {
        let botao = estilos::botao_de(card);
        assert!(pintado(&h, botao), "o ⠿ do cartão {card:?} não se pintou");
        clica(&mut h, botao);
        assert!(
            pintado(&h, estilos::MAIS),
            "o clique no ⠿ não abriu a lista"
        );
        clica(&mut h, estilos::linha_de(7));
        assert_eq!(
            estilos::take_pedidos(),
            vec![estilos::Pedido::Aplicar {
                estilo: 7,
                camada: linha
            }],
            "escolher não pediu a tinta de base"
        );
        assert!(!pintado(&h, estilos::linha_de(7)), "a lista não fechou");
    }
    // Sem tinta nem camada não há linha, e o ⠿ sai com ela.
    forma(
        [BasePaint::Absent, BasePaint::Shown],
        Vec::new(),
        Vec::new(),
    );
    pinta(&mut h, 6);
    assert!(
        !pintado(&h, estilos::botao_de(insp::CARD_FILL)),
        "um cartão vazio mostra o ⠿"
    );
    assert!(pintado(&h, estilos::botao_de(insp::CARD_STROKE)));
}

/// O ⠿ vale para a linha de CIMA da pilha do cartão, salvo a camada aberta desse tipo.
#[test]
fn the_styles_icon_targets_the_open_layer_or_the_top_row() {
    use StackItem::{BaseFill, BaseStroke, Entry};
    let (mut h, _l) = hero();
    let _ = estilos::take_pedidos();
    publica_um_estilo(Vec::new());
    let botao = estilos::botao_de(insp::CARD_FILL);
    let escolhe = |h: &mut HeroScreen| {
        pinta(h, 6);
        clica(h, botao);
        clica(h, estilos::linha_de(7));
        match estilos::take_pedidos().as_slice() {
            [estilos::Pedido::Aplicar { camada, .. }] => *camada,
            o => panic!("escolher pediu {o:?}"),
        }
    };
    let camadas = || vec![camada(true, VERDE, 1.0), camada(true, AZUL, 1.0)];
    // A camada 1 por cima da base e da 0.
    forma(
        [BasePaint::Shown, BasePaint::Shown],
        camadas(),
        vec![BaseFill, BaseStroke, Entry(0), Entry(1)],
    );
    assert_eq!(escolhe(&mut h), 1, "a linha de cima é a camada do topo");
    // A base por cima das camadas.
    forma(
        [BasePaint::Shown, BasePaint::Shown],
        camadas(),
        vec![Entry(0), Entry(1), BaseStroke, BaseFill],
    );
    assert_eq!(escolhe(&mut h), HEX_FILL, "a linha de cima é a base");
    // A camada aberta manda.
    appearance::set_open_layer(Some(0));
    assert_eq!(escolhe(&mut h), 0, "a camada aberta perdeu");
    appearance::set_open_layer(None);
}

/// ⭐ Maquete F: a base ligada mostra o NOME do estilo, a corrente partida solta-a e não há caixa de
/// código (a da base solta volta). A dica da corrente é a da camada.
#[test]
fn a_linked_base_shows_the_style_name_and_detaches() {
    let (mut h, _l) = hero();
    let _ = estilos::take_pedidos();
    forma([BasePaint::Shown, BasePaint::Shown], Vec::new(), Vec::new());
    let mut ligados = vec![None; ph2d_vec_scene::LINHAS_DA_PILHA];
    ligados[HEX_FILL] = Some(7);
    publica_um_estilo(ligados);
    let ((), medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| pinta(&mut h, 12));
    assert!(
        medidos
            .iter()
            .any(|m| m.texto == "Brand orange" && m.coube()),
        "o nome do estilo não saiu inteiro na linha do Fill"
    );
    let solta = estilos::solta_de(HEX_FILL);
    assert!(pintado(&h, solta), "a corrente partida não se pintou");
    assert!(
        !pintado(&h, estilos::solta_de(HEX_STROKE)),
        "o Stroke solto tem corrente"
    );
    assert!(
        !pintado(&h, insp::cards_hex_id(HEX_FILL)),
        "a base ligada ainda oferece o código para editar"
    );
    assert!(
        pintado(&h, insp::cards_hex_id(HEX_STROKE)),
        "o Stroke solto perdeu o código"
    );
    assert_eq!(
        h.store.tooltip_for(solta),
        h.store.tooltip_for(estilos::solta_de(0)),
        "a dica da corrente da base difere da da camada"
    );
    assert!(
        h.store.tooltip_for(solta).is_some(),
        "a corrente da base sem dica"
    );
    clica(&mut h, solta);
    assert_eq!(
        estilos::take_pedidos(),
        vec![estilos::Pedido::Soltar { camada: HEX_FILL }]
    );
}

/// ⭐ C3: na pilha, cada linha tem as marcas dela (a chave é por camada) e só a 1.ª repete os nomes; o
/// clique no losango de uma linha de baixo pede essa linha.
/// *Mutação: as marcas só com rótulo em `compoe`, ou `so_marcas` fora de `row` ⇒ a 2.ª linha sem alvo.*
#[test]
fn every_row_of_a_paint_stack_has_its_own_marks() {
    use ph2d_editor_core::widget::key_mark;
    let (mut h, _l) = hero();
    forma(
        [BasePaint::Shown, BasePaint::Shown],
        vec![camada(true, VERDE, 1.0), camada(true, LARANJA, 0.5)],
        Vec::new(),
    );
    pinta(&mut h, 12);
    let linhas = [
        (
            vids::vector_paint_swatch_id(1),
            vids::vector_paint_row_opacity_id(1),
        ),
        (
            vids::vector_paint_swatch_id(0),
            vids::vector_paint_row_opacity_id(0),
        ),
        (vids::VECTOR_FILL_SWATCH, vids::VECTOR_FILL_OPACITY_NUM),
    ];
    key_mark::set_key_marks(
        linhas
            .iter()
            .flat_map(|(cor, op)| {
                [
                    (*cor, key_mark::KeyMark::Livre),
                    (*op, key_mark::KeyMark::Livre),
                ]
            })
            .collect(),
    );
    pinta(&mut h, 1);
    let mut topos = Vec::new();
    for (i, (cor, op)) in linhas.into_iter().enumerate() {
        let caixa = h.hit_index.rect_for(cor).expect("a linha pintou-se");
        for campo in [cor, op] {
            let marca = h
                .hit_index
                .rect_for(key_mark::key_mark_id(campo))
                .unwrap_or_else(|| panic!("a linha {i} sem a marca do campo {campo:?}"));
            assert!(
                marca.y + marca.h <= caixa.y && caixa.y - marca.y <= 2.0 * m::KEY_MARK_BOX,
                "a marca da linha {i} não está logo acima dela: {marca:?} × {caixa:?}"
            );
        }
        topos.push(caixa.y);
    }
    assert!(topos[0] < topos[1] && topos[1] < topos[2], "{topos:?}");
    let passo = topos[2] - topos[1];
    assert!(
        (passo - (m::FIELD_H + m::FIELD_ROW_GAP + m::KEY_MARK_BOX)).abs() < 0.5,
        "a linha sem nomes não é compacta: {passo}"
    );
    let marca = h
        .hit_index
        .rect_for(key_mark::key_mark_id(vids::VECTOR_FILL_SWATCH))
        .expect("a marca da base");
    mao(&mut h, PointerKind::Down, centro(marca));
    let c = key_mark::take_key_mark_click().expect("o clique no losango da base");
    assert_eq!(c.fields, vec![vids::VECTOR_FILL_SWATCH]);
}
