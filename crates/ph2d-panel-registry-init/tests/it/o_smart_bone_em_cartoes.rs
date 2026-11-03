//! ⭐ **As linhas de lista do Bones no padrão do cartão** (foto do dono 03/11): em Smart Bone, *Object*
//! e *Action* continuavam no estilo antigo (nome à esquerda noutro tom, chip fora do padrão), e a
//! *Curve Tip* era da mesma família. A cura: rótulo por cima e a caixa na largura do cartão, como o
//! Fill da galeria. Tudo por clique real no hero montado.
#![cfg(feature = "panel-skeleton")]

use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
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
