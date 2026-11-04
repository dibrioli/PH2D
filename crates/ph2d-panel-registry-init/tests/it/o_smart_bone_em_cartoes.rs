//! ⭐ **As linhas de lista do Bones no padrão do cartão** (foto do dono 03/11): em Smart Bone, *Object*
//! e *Action* continuavam no estilo antigo (nome à esquerda noutro tom, chip fora do padrão), e a
//! *Curve Tip* era da mesma família. A cura: rótulo por cima e a caixa na largura do cartão, como o
//! Fill da galeria. Tudo por clique real no hero montado.
#![cfg(feature = "panel-skeleton")]

use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_skeleton::ids as ids_do_painel;
use ph2d_panel_skeleton::state;
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 1400.0,
};

fn pinta(h: &mut HeroScreen, n: usize) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..n {
        scene.reset();
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn rect(h: &HeroScreen, id: NodeId) -> Rect {
    h.hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{id:?} não foi pintado"))
}

fn clica_em(h: &mut HeroScreen, x: f32, y: f32) {
    let arena = bumpalo::Bump::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let evs: Vec<_> =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena)
                .to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
    pinta(h, 3);
}

fn clica(h: &mut HeroScreen, id: NodeId) {
    let r = rect(h, id);
    clica_em(h, r.x + r.w * 0.5, r.y + r.h * 0.5);
}

fn chegou_ao_barramento(h: &mut HeroScreen, id: NodeId) -> bool {
    h.bus
        .drain()
        .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id))
}

/// O Bones com um osso curvo em foco, Smart Bone com duas acções e a lista da ponta com duas opções.
fn hero() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut osso = ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0);
    osso.segments = 4;
    state::set_current_bone(Some(osso));
    state::set_current_bone_handles(Some(1));
    state::set_current_bone_tip(Some(state::TipView {
        rotulos: vec!["From Chain".to_owned(), "Nobody".to_owned()],
        ligado: 0,
        escondidos: 0,
    }));
    state::set_current_bone_smart(Some(state::SmartBoneView {
        from: 0.0,
        to: 90.0,
        clip: "Walk".to_owned(),
        target: String::new(),
        picking: false,
    }));
    state::set_current_bone_actions(vec!["Walk".to_owned(), "Wave".to_owned()]);
    state::set_current_bone_ik(Some((1.0, 0.0, 2.0, ph2d_skeleton::BendSide::default())));
    let mut h = HeroScreen::new(NodeId(1));
    let todos: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in todos {
        h.panel_visibility.insert(n, n == "skeleton");
    }
    pinta(&mut h, 60);
    let _ = h.bus.drain().count();
    h
}

/// A caixa ocupa a largura do cartão: o início do campo da esquerda e o fim do da direita.
fn na_largura_do_cartao(caixa: Rect, esq: Rect, dir: Rect, nome: &str) {
    assert!(
        (caixa.x - esq.x).abs() < 0.5,
        "{nome} não começa na coluna dos campos: {} × {}",
        caixa.x,
        esq.x
    );
    let (fim, alvo) = (caixa.x + caixa.w, dir.x + dir.w);
    assert!(
        (fim - alvo).abs() < 0.5,
        "{nome} não acaba na borda dos campos: {fim} × {alvo}"
    );
}

/// *Mutação: voltar a `c.legacy` com as linhas antigas ⇒ a caixa cai na coluna de valor e sem rótulo
/// por cima, e as duas medidas reprovam.*
#[test]
fn object_action_and_curve_tip_are_card_rows() {
    let h = hero();
    let (de, ate) = (
        rect(&h, ids::VECTOR_BONE_SMART_FROM),
        rect(&h, ids::VECTOR_BONE_SMART_TO),
    );
    let objecto = rect(&h, ids::VECTOR_BONE_SMART_PICK);
    let accao = rect(&h, ids_do_painel::VECTOR_BONE_SMART_CLIP);
    na_largura_do_cartao(objecto, de, ate, "Object");
    na_largura_do_cartao(accao, de, ate, "Action");
    // O rótulo por cima: o vão entre as três linhas é o mesmo, porque cada uma leva o seu rótulo.
    let vao_accao = accao.y - (objecto.y + objecto.h);
    let vao_campos = de.y - (accao.y + accao.h);
    assert!(
        (vao_accao - vao_campos).abs() < 0.5,
        "Action não tem o rótulo por cima como From/To: vão {vao_accao} × {vao_campos}"
    );
    assert!(
        vao_accao >= objecto.h * 0.5,
        "não sobra lugar para o rótulo por cima de Action: {vao_accao}"
    );
    // E o da 1.ª linha: do título do cartão ao botão, o mesmo vão do Mix, que leva rótulo.
    let vao_do_titulo = |cartao, campo: Rect| campo.y - rect(&h, ids::grip_de(cartao)).y;
    let (smart, ik) = (
        vao_do_titulo(ph2d_panel_skeleton::cards::CARD_SMART, objecto),
        vao_do_titulo(
            ph2d_panel_skeleton::cards::CARD_IK,
            rect(&h, ids::VECTOR_BONE_IK_MIX),
        ),
    );
    assert!(
        (smart - ik).abs() < 0.5,
        "Object não tem o rótulo por cima como o Mix: {smart} × {ik}"
    );
    let ponta = rect(&h, ids_do_painel::VECTOR_BONE_TIP);
    let (len, seg) = (
        rect(&h, ids::VECTOR_BONE_LENGTH),
        rect(&h, ids::VECTOR_BONE_SEGMENTS),
    );
    na_largura_do_cartao(ponta, len, seg, "Curve Tip");
}

/// *Mutação: tirar o `dropdown::declare` ⇒ a caixa não abre e a opção nunca é pintada.*
#[test]
fn the_action_list_opens_and_the_choice_reaches_the_bus() {
    let mut h = hero();
    let (caixa, wave) = (
        ids_do_painel::VECTOR_BONE_SMART_CLIP,
        ids::VECTOR_BONE_SMART_CLIP_IDS[1],
    );
    assert!(h.hit_index.rect_for(wave).is_none(), "fixtura: fechada");
    clica(&mut h, caixa);
    assert!(h.hit_index.rect_for(wave).is_some(), "a caixa não abriu");
    clica(&mut h, wave);
    assert!(
        chegou_ao_barramento(&mut h, wave),
        "a escolha não chegou ao osso"
    );
    assert!(h.hit_index.rect_for(wave).is_none(), "a lista não fechou");
    // A lista abre e um clique fora fecha sem escolher.
    clica(&mut h, caixa);
    assert!(h.hit_index.rect_for(wave).is_some(), "não reabriu");
    let fora = rect(&h, ids::VECTOR_BONE_SMART_FROM);
    clica_em(&mut h, fora.x + 2.0, fora.y - 2.0);
    assert!(
        h.hit_index.rect_for(wave).is_none(),
        "o clique fora não fechou"
    );
    assert!(
        !chegou_ao_barramento(&mut h, wave),
        "o clique fora escolheu"
    );
}

#[test]
fn the_curve_tip_list_and_the_object_button_answer_a_click() {
    let mut h = hero();
    let (caixa, opcao) = (ids_do_painel::VECTOR_BONE_TIP, ids::VECTOR_BONE_TIP_IDS[1]);
    clica(&mut h, caixa);
    clica(&mut h, opcao);
    assert!(
        chegou_ao_barramento(&mut h, opcao),
        "a escolha da ponta não chegou ao osso"
    );
    clica(&mut h, ids::VECTOR_BONE_SMART_PICK);
    assert!(
        chegou_ao_barramento(&mut h, ids::VECTOR_BONE_SMART_PICK),
        "o botão Object não armou a escolha"
    );
}

fn ponteiro(h: &mut HeroScreen, kind: PointerKind, x: f32, y: f32) {
    let arena = bumpalo::Bump::new();
    let ev = PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    let evs: Vec<_> =
        ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena)
            .to_vec();
    for e in evs {
        h.apply_event(e);
    }
}

fn valor(h: &HeroScreen, id: NodeId) -> f64 {
    match h.store.get(id) {
        Some(InteractiveState::NumberInput { value, .. }) => *value,
        _ => panic!("{id:?} não é um campo numérico"),
    }
}

/// Arrasta na pega de `id`: `doc` faz de documento (guarda a contagem inteira e devolve-a pela porta
/// do painel), e a pintura entre dois Moves repõe o número, como cada quadro do app.
struct Arrasto {
    id: NodeId,
    x: f32,
    y: f32,
}

impl Arrasto {
    fn comeca(h: &mut HeroScreen, id: NodeId) -> Self {
        let r = rect(h, id);
        let (x, y) = (r.x + 4.0, r.y + r.h * 0.5);
        ponteiro(h, PointerKind::Down, x, y);
        // O 1.º Move só atravessa o limiar: não muda o valor.
        ponteiro(h, PointerKind::Move, x + 8.0, y);
        Self { id, x: x + 8.0, y }
    }

    fn anda(&mut self, h: &mut HeroScreen, moves: usize, dx: f32, doc: &mut impl FnMut(f64)) {
        for _ in 0..moves {
            self.x += dx;
            ponteiro(h, PointerKind::Move, self.x, self.y);
            doc(valor(h, self.id));
            pinta(h, 1);
        }
    }
}

/// O dono (03/11): arrastar no ícone de Segments não fazia nada. O documento guarda inteiro, o quadro
/// devolve-o ao campo e o resíduo de cada Move (0,1 por px) perdia-se.
/// *Mutação: a base do Move volta a ser o valor do campo ⇒ a contagem não sobe.*
#[test]
fn dragging_the_icon_of_a_count_field_walks_both_ways() {
    use ph2d_skeleton::bend::{BoneSpec, segments_of};
    let mut h = hero();
    let mut osso = BoneSpec::straight(20.0, 1.0);
    osso.segments = 4;
    let lido = std::cell::Cell::new(4u8);
    let mut doc = |v: f64| {
        osso.segments = segments_of(v.round() as u8);
        lido.set(osso.segments);
        state::set_current_bone(Some(osso));
    };
    let mut a = Arrasto::comeca(&mut h, ids::VECTOR_BONE_SEGMENTS);
    a.anda(&mut h, 40, 2.0, &mut doc);
    assert_eq!(lido.get(), 12, "80 px a 0,1 por px: 4 + 8");
    a.anda(&mut h, 40, -2.0, &mut doc);
    assert_eq!(lido.get(), 4, "a volta desfaz");
    // Para lá do piso e de volta: sem zona morta do tamanho do que se andou contra ele.
    a.anda(&mut h, 60, -2.0, &mut doc);
    assert_eq!(lido.get(), 1, "o piso");
    a.anda(&mut h, 30, 2.0, &mut doc);
    assert!(
        lido.get() >= 4,
        "a inversão no piso ficou presa: {}",
        lido.get()
    );
    ponteiro(&mut h, PointerKind::Up, a.x, a.y);
    state::set_current_bone(Some({
        let mut o = BoneSpec::straight(20.0, 1.0);
        o.segments = 4;
        o
    }));
}

/// O Chain do IK é a mesma família: o documento guarda uma contagem inteira.
#[test]
fn dragging_the_icon_of_the_ik_chain_walks_too() {
    let mut h = hero();
    let lido = std::cell::Cell::new(2u32);
    let mut doc = |v: f64| {
        let n = v.max(0.0).round();
        lido.set(n as u32);
        state::set_current_bone_ik(Some((1.0, 0.0, n, ph2d_skeleton::BendSide::default())));
    };
    let mut a = Arrasto::comeca(&mut h, ids::VECTOR_BONE_IK_CHAIN);
    a.anda(&mut h, 40, 2.0, &mut doc);
    assert_eq!(lido.get(), 10, "80 px a 0,1 por px: 2 + 8");
    ponteiro(&mut h, PointerKind::Up, a.x, a.y);
}

/// O campo contínuo não muda: cada px anda um passo, como antes da cura da contagem.
#[test]
fn dragging_the_icon_of_a_continuous_field_still_walks_by_the_pixel() {
    let mut h = hero();
    let id = ids::VECTOR_BONE_LENGTH;
    let antes = valor(&h, id);
    let px_por_unidade = antes / 20.0;
    let lido = std::cell::Cell::new(antes);
    let mut doc = |v: f64| {
        lido.set(v);
        let osso = ph2d_skeleton::bend::BoneSpec {
            segments: 4,
            ..ph2d_skeleton::bend::BoneSpec::straight(v / px_por_unidade, 1.0)
        };
        state::set_current_bone(Some(osso));
    };
    let mut a = Arrasto::comeca(&mut h, id);
    a.anda(&mut h, 10, 3.0, &mut doc);
    ponteiro(&mut h, PointerKind::Up, a.x, a.y);
    assert!(
        (lido.get() - (antes + 30.0)).abs() < 1e-3,
        "30 px a 1 por px: {antes} → {}",
        lido.get()
    );
}

/// O Figma: a seta ↔ sobre o ícone e sobre a letra; sobre o número, não.
/// *Mutação: `on_handle` sempre falso ⇒ a pega não pede a seta.*
#[test]
fn the_arrow_shows_over_the_icon_and_the_letter_never_over_the_value() {
    use ph2d_editor_core::widget::card_field::{ew_resize_under, painted_letter};
    let mut h = hero();
    // Em Auto as alças somem; autoradas, os campos da curva levam letra.
    state::set_current_bone_handles(Some(0));
    pinta(&mut h, 60);
    assert!(
        painted_letter(ids::VECTOR_BONE_CURVE_IN_Y).is_some(),
        "fixtura: o campo da curva mostra uma letra"
    );
    for id in [ids::VECTOR_BONE_SEGMENTS, ids::VECTOR_BONE_CURVE_IN_Y] {
        let r = rect(&h, id);
        let (na_pega, no_valor) = (r.x + 4.0, r.x + r.w - 4.0);
        let y = r.y + r.h * 0.5;
        let (sob, caixa) = h.hit_index.hit_with_rect(na_pega, y).expect("a caixa");
        assert_eq!(sob, id, "o ponteiro sobre a pega acha o campo");
        assert!(
            ew_resize_under(&h.store, id, caixa, na_pega),
            "{id:?}: pega"
        );
        assert!(
            !ew_resize_under(&h.store, id, caixa, no_valor),
            "{id:?}: valor"
        );
    }
}

/// Todo ícone sem texto do Bones tem dica; controlo: ≥ 1 ícone declarado.
#[test]
fn every_icon_of_the_bones_cards_has_a_tooltip() {
    state::set_current_bone_limit(Some((-0.5, 0.5)));
    state::set_current_bone_ik(Some((1.0, 0.0, 2.0, ph2d_skeleton::BendSide::default())));
    let h = hero();
    let bones = ids::SKELETON_PANEL;
    let todos = ph2d_editor_core::panel::cards::dicas::icon_only_ids(bones);
    assert!(!todos.is_empty(), "controlo: nenhum ícone declarado");
    let mudos: Vec<_> = todos
        .iter()
        .filter(|id| h.store.tooltip_for(**id).is_none())
        .collect();
    assert!(mudos.is_empty(), "ícones sem dica: {mudos:?}");
}
