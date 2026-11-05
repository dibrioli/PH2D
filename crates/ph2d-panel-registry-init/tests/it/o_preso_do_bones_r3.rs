//! ⭐ **R3 (05 §9, «Outros antigos»)**, no hero montado: com um cartão do Bones preso a A e B em foco,
//! as listas de Smart Bone e Tip são as de A; e um atalho depois de premir no preso é da seleção.
#![cfg(feature = "panel-skeleton")]

use super::o_cartao_solto_no_canvas::{
    bones, clica, hero_com_o_bones, mao, osso_em_foco, pinta, solta,
};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{dispatch_key, loose_pin_of, loose_rect_of};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_host::{KeyEvent, KeyKind, PointerKind};
use ph2d_panel_skeleton::cards::{CARD_BONE, CARD_SMART};
use ph2d_panel_skeleton::{self as skel, SmartBoneView, TipView};

const PRESO: u64 = 21;
const FOCO: u64 = 22;

fn publica(bits: u64, nome: &str, acoes: &[&str], pontas: &[&str]) {
    osso_em_foco(bits, nome, 0.5);
    skel::set_current_bone_smart(Some(SmartBoneView {
        from: 0.0,
        to: 90.0,
        clip: acoes[0].to_owned(),
        target: String::new(),
        picking: false,
    }));
    skel::set_current_bone_actions(acoes.iter().map(|s| (*s).to_owned()).collect());
    skel::set_current_bone_tip(Some(TipView {
        rotulos: pontas.iter().map(|s| (*s).to_owned()).collect(),
        ligado: 0,
        escondidos: 0,
    }));
}

fn o_preso() {
    publica(PRESO, "Bone A", &["a1", "a2", "a3"], &["t1", "t2", "t3"]);
}

fn o_foco() {
    publica(FOCO, "Bone B", &["b1"], &["u1"]);
}

/// Larga o que a thread publicou, mesmo quando o teste cai.
struct Limpa;
impl Drop for Limpa {
    fn drop(&mut self) {
        skel::set_current_bone_smart(None);
        skel::set_current_bone_actions(Vec::new());
        skel::set_current_bone_tip(None);
    }
}

/// `sec` solto, preso a A, com B em foco.
fn preso(sec: NodeId) -> (HeroScreen, Limpa) {
    let mut h = hero_com_o_bones();
    let limpa = Limpa;
    o_preso();
    pinta(&mut h, 20);
    solta(&mut h, sec);
    clica(&mut h, loose_pin_of(sec));
    assert_eq!(h.store.loose_card(sec).and_then(|c| c.pin), Some(PRESO));
    o_foco();
    skel::publica_presos(&h.store.loose_pins_of(bones()), |_| o_preso());
    pinta(&mut h, 5);
    (h, limpa)
}

fn centro(r: ph2d_editor_core::zones::Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// Clica no chip da lista dentro do solto e devolve o hero com ela aberta.
fn abre(h: &mut HeroScreen, sec: NodeId, chip: NodeId) {
    let r = h.store.panel_rect(loose_rect_of(sec)).expect("o solto");
    let c = centro(h.hit_index.rect_for(chip).expect("o chip"));
    assert!(r.contains(c.0, c.1), "o chip pintado não é o do preso");
    mao(h, PointerKind::Down, c);
    mao(h, PointerKind::Up, c);
    pinta(h, 3);
}

/// A lista de ações do Smart Bone aberta no preso tem as três de A, não a única de B.
/// *Mutação: o corpo do preso fora de `com_preso` ⇒ só uma opção.*
#[test]
fn the_action_list_of_a_pinned_card_is_of_its_bone() {
    let (mut h, _l) = preso(CARD_SMART);
    abre(&mut h, CARD_SMART, skel::ids::VECTOR_BONE_SMART_CLIP);
    assert!(
        h.hit_index
            .rect_for(core_ids::VECTOR_BONE_SMART_CLIP_IDS[2])
            .is_some(),
        "a lista do preso leu as ações do osso em foco"
    );
}

/// A lista de Tip aberta no preso tem as três pontas de A, e a pressão na opção fica com A.
#[test]
fn the_tip_list_of_a_pinned_card_is_of_its_bone() {
    let (mut h, _l) = preso(CARD_BONE);
    abre(&mut h, CARD_BONE, skel::ids::VECTOR_BONE_TIP);
    let opcao = h
        .hit_index
        .rect_for(core_ids::VECTOR_BONE_TIP_IDS[2])
        .expect("a lista do preso leu as pontas do osso em foco");
    mao(&mut h, PointerKind::Down, centro(opcao));
    assert_eq!(h.store.card_press_subject(), Some(PRESO));
}

fn tecla(h: &mut HeroScreen) {
    let arena = bumpalo::Bump::new();
    let ev = KeyEvent {
        keycode: 0x4B,
        modifiers: ph2d_host::Modifiers::default(),
        kind: KeyKind::Down,
        timestamp_ns: 0,
    };
    let _ = dispatch_key(&mut h.store, ev, &arena);
}

/// Prime na beira do solto e larga.
fn prime(h: &mut HeroScreen, sec: NodeId) {
    let r = h.store.panel_rect(loose_rect_of(sec)).expect("o solto");
    let p = (r.x + r.w - 12.0, r.y + r.h - 4.0);
    mao(h, PointerKind::Down, p);
    assert_eq!(h.store.card_press_subject(), Some(PRESO));
    mao(h, PointerKind::Up, p);
}

/// ⭐ Um atalho depois de premir no preso vai à seleção; a tecla de um campo, de uma lista aberta, do
/// seletor ou com a mão a meio do gesto continua a ser do preso (Shift a meio de um arrasto).
/// *Mutação: cada guarda de `note_card_key` fora ⇒ o seu caso cai; `note_card_key` fora ⇒ o atalho fica.*
#[test]
fn a_shortcut_after_a_press_on_a_pinned_card_is_of_the_selection() {
    let (mut h, _l) = preso(CARD_SMART);
    prime(&mut h, CARD_SMART);
    tecla(&mut h);
    assert_eq!(
        h.store.card_press_subject(),
        None,
        "o atalho ficou no preso"
    );

    let campo = core_ids::VECTOR_BONE_SMART_FROM;
    prime(&mut h, CARD_SMART);
    h.store.set_focus(Some(campo));
    tecla(&mut h);
    assert_eq!(h.store.card_press_subject(), Some(PRESO), "campo em foco");
    h.store.set_focus(None);

    h.store.set_active(Some(campo));
    tecla(&mut h);
    assert_eq!(
        h.store.card_press_subject(),
        Some(PRESO),
        "mão a meio do gesto"
    );
    h.store.set_active(None);

    h.store.open_picker_on(campo);
    tecla(&mut h);
    assert_eq!(h.store.card_press_subject(), Some(PRESO), "seletor aberto");
    h.store.set_picker_target(None);

    h.store
        .open_context_menu(ph2d_editor_core::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ph2d_editor_core::interaction::ContextMenuKind::CreateNote {
                panel: CARD_SMART,
                section: None,
            },
        });
    tecla(&mut h);
    assert_eq!(h.store.card_press_subject(), Some(PRESO), "menu aberto");
    h.store.close_context_menu();
    pinta(&mut h, 2);

    abre(&mut h, CARD_SMART, skel::ids::VECTOR_BONE_SMART_CLIP);
    assert!(h.store.dropdown_popover().is_some(), "a lista não abriu");
    tecla(&mut h);
    assert_eq!(h.store.card_press_subject(), Some(PRESO), "lista aberta");
}

/// ⭐ T36: um gesto sobre um cartão preso a um osso é desse osso; preso a um objecto que não é osso,
/// da seleção. E a shell pergunta por esta porta: ela perdeu-a no porte da linha de teste, e o preso
/// mostrava um osso e escrevia noutro. A shell lê-se no fonte (o dreno e a tecla não correm aqui).
/// *Mutação: `preso` ignorado; o dreno com `selected_bone`; a tecla da shell sem `note_card_key`.*
#[test]
fn the_requests_of_a_gesture_on_a_pinned_card_are_of_its_bone() {
    use ph2d_skeleton_live::bone::osso_do_gesto;
    let mut sim = ph2d_ecs::SimWorld::new();
    let osso = |sim: &mut ph2d_ecs::SimWorld| {
        let e = (
            ph2d_ecs::Transform::default(),
            ph2d_skeleton_ecs::Bone::default(),
        );
        sim.world_mut().spawn(e).id().to_bits()
    };
    let (a, b) = (osso(&mut sim), osso(&mut sim));
    let objecto = sim
        .world_mut()
        .spawn(ph2d_ecs::Transform::default())
        .id()
        .to_bits();
    assert_eq!(osso_do_gesto(&sim, Some(a), [b]), Some(a));
    assert_eq!(osso_do_gesto(&sim, None, [b]), Some(b));
    assert_eq!(osso_do_gesto(&sim, Some(objecto), [b]), Some(b));
    assert_eq!(osso_do_gesto(&sim, Some(objecto), []), None);
    let dreno = include_str!("../../../../shells/desktop/src/render_loop/fase_bus_drain.rs");
    assert!(
        dreno.contains("osso_selecionado: ph2d_skeleton_live::bone::osso_do_gesto("),
        "o dreno da shell deixou de perguntar pelo preso"
    );
    let tecla =
        include_str!("../../../../shells/desktop/src/input_dispatch/keyboard_bind_capture.rs");
    assert!(
        tecla.contains("hero.store.note_card_key();"),
        "a tecla da shell deixou de largar o preso"
    );
}

/// ⭐ R5, ponta a ponta: com A preso e B selecionado, o gesto sobre o cartão preso escreve em A e B
/// fica como estava. *Mutação: `osso_do_gesto` a ignorar `preso` ⇒ a escrita vai para B.*
#[test]
fn the_gesture_on_a_pinned_card_writes_the_pinned_bone_not_the_selected_one() {
    use ph2d_app_skeleton::knobs::{BoneKnob, apply_from_panel};
    use ph2d_ecs::{SimWorld, Transform};
    use ph2d_skeleton_ecs::Bone;
    use ph2d_skeleton_live::bone::osso_do_gesto;
    let mut sim = SimWorld::default();
    let mut osso = || {
        sim.world_mut()
            .spawn((
                Transform::IDENTITY,
                Bone {
                    length: 10.0,
                    ..Bone::default()
                },
            ))
            .id()
    };
    let (a, b) = (osso(), osso());
    let doc = ph2d_editor_core::DocPx::new(100.0);
    let alvo = osso_do_gesto(&sim, Some(a.to_bits()), [b.to_bits()]).expect("é osso");
    apply_from_panel(&mut sim, alvo, BoneKnob::Length, 2500.0, doc);
    let comprimento = |e| sim.world().get::<Bone>(e).expect("é osso").length;
    assert!((comprimento(a) - 25.0).abs() < 1e-4, "A {}", comprimento(a));
    assert!((comprimento(b) - 10.0).abs() < 1e-4, "B {}", comprimento(b));
}

struct MesaBones(HeroScreen);

impl super::o_alfinete_fill_stroke_c2::scrub_do_preso::MesaDoPreso for MesaBones {
    fn hero(&mut self) -> &mut HeroScreen {
        &mut self.0
    }
    fn quadros(&mut self, n: usize) {
        for _ in 0..n {
            o_foco();
            skel::publica_presos(&self.0.store.loose_pins_of(bones()), |_| o_preso());
            pinta(&mut self.0, 1);
        }
    }
}

/// ⭐ O censo do scrub no osso: toda pega do cartão Bone preso anda sem o quadro a devolver ao número
/// do osso em foco; o Esc repõe.
#[test]
fn every_scrub_handle_of_a_pinned_bone_card_survives_the_frames() {
    let (h, _l) = preso(CARD_BONE);
    let mut m = MesaBones(h);
    let (n, erros) =
        super::o_alfinete_fill_stroke_c2::scrub_do_preso::censo(&mut m, CARD_BONE, "Bone");
    assert!(n >= 1, "nenhuma pega à vista");
    assert!(erros.is_empty(), "{n} pegas: {erros:#?}");
}
