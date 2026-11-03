use super::anda_com;
use ph2d_editor_core::ids::SKELETON_PANEL;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{NodeId, WidgetStore};

const IK: NodeId = NodeId(7_102_543_075_790_560_757);
const OSSO: u64 = 42;

/// O cartão IK solto do Bones, preso ao osso COM corda (dois cliques no alfinete), rect em `r`.
fn solto_com_corda(store: &mut WidgetStore, r: Rect) {
    store.set_card_subject(SKELETON_PANEL, Some((OSSO, "Bone 5".into())));
    store.detach_card(IK, SKELETON_PANEL, (r.x, r.y), r.w);
    store.set_loose_card_rect(IK, r);
    store.toggle_loose_pin(IK);
    store.toggle_loose_pin(IK);
}

/// ⭐ T44: a corda sai da borda do cartão e chega ao sujeito em ecrã; quando o cartão anda, ela
/// acompanha com inércia (sem saltos, T41) e a ponta continua no sujeito.
#[test]
fn the_pin_rope_runs_from_the_card_edge_to_its_subject() {
    let alvo = |b: u64| (b == OSSO).then_some((450.0, 500.0));
    let mut store = WidgetStore::default();
    solto_com_corda(&mut store, Rect::new(100.0, 100.0, 200.0, 120.0));
    for _ in 0..120 {
        anda_com(&mut store, SKELETON_PANEL, alvo, 1.0 / 60.0);
    }
    let p = store.loose_rope(IK).to_vec();
    assert!(p.len() >= 2, "a corda existe: {p:?}");
    assert_eq!(
        p[0],
        [300.0, 220.0],
        "sai do canto do cartão mais perto do sujeito"
    );
    assert_eq!(*p.last().unwrap(), [450.0, 500.0], "chega ao sujeito");
    let meio = p[p.len() / 2];
    assert!(
        meio[1] > (220.0 + 500.0) / 2.0,
        "pende para baixo: {meio:?}"
    );

    store.set_loose_card_rect(IK, Rect::new(160.0, 100.0, 200.0, 120.0));
    anda_com(&mut store, SKELETON_PANEL, alvo, 1.0 / 60.0);
    let q = store.loose_rope(IK).to_vec();
    assert_eq!(q[0], [360.0, 220.0], "a ponta do cartão vai com ele");
    let d = (q[p.len() / 2][0] - meio[0]).hypot(q[p.len() / 2][1] - meio[1]);
    assert!(d < 30.0, "o meio acompanha com inércia, não salta: {d}");

    anda_com(&mut store, SKELETON_PANEL, |_| None, 1.0 / 60.0);
    assert!(
        store.loose_rope(IK).is_empty(),
        "sujeito que sumiu: sem corda"
    );
}

/// T44: o alfinete cicla — solto → preso → preso com corda → solto; só com corda há pontos.
#[test]
fn the_pin_cycles_pinned_then_rope_then_loose() {
    let alvo = |_| Some((0.0, 0.0));
    let mut store = WidgetStore::default();
    store.set_card_subject(SKELETON_PANEL, Some((OSSO, "Bone 5".into())));
    store.detach_card(IK, SKELETON_PANEL, (100.0, 100.0), 200.0);
    store.set_loose_card_rect(IK, Rect::new(100.0, 100.0, 200.0, 120.0));
    let estado = |s: &WidgetStore| s.loose_card(IK).map(|c| (c.pin, c.rope));
    store.toggle_loose_pin(IK);
    assert_eq!(estado(&store), Some((Some(OSSO), false)));
    anda_com(&mut store, SKELETON_PANEL, alvo, 1.0 / 60.0);
    assert!(
        store.loose_rope(IK).is_empty(),
        "preso sem corda: nenhuma corda"
    );
    store.toggle_loose_pin(IK);
    assert_eq!(estado(&store), Some((Some(OSSO), true)));
    anda_com(&mut store, SKELETON_PANEL, alvo, 1.0 / 60.0);
    assert!(
        !store.loose_rope(IK).is_empty(),
        "preso com corda: a corda aparece"
    );
    store.toggle_loose_pin(IK);
    assert_eq!(estado(&store), Some((None, false)));
    assert!(store.loose_rope(IK).is_empty(), "solto: a corda sai");
}

/// T44c (dono 02/11): com o cartão por baixo do sujeito a folga não tinha para onde pender e
/// encolhia num cotovelo atrás do cartão. A folga vem do vão HORIZONTAL: na vertical a corda é um
/// fio de prumo. O caso real: a corda pende de lado e o artista leva o cartão para baixo do osso.
#[test]
fn a_rope_under_its_subject_hangs_straight_without_an_elbow() {
    let mut store = WidgetStore::default();
    solto_com_corda(&mut store, Rect::new(100.0, 500.0, 200.0, 120.0));
    for _ in 0..120 {
        anda_com(
            &mut store,
            SKELETON_PANEL,
            |_| Some((420.0, 100.0)),
            1.0 / 60.0,
        );
    }
    for i in 1..=30 {
        let y = 500.0 - 100.0 * i as f32 / 30.0;
        store.set_loose_card_rect(IK, Rect::new(100.0, y, 200.0, 120.0));
        anda_com(
            &mut store,
            SKELETON_PANEL,
            |_| Some((205.0, 100.0)),
            1.0 / 60.0,
        );
    }
    for _ in 0..90 {
        anda_com(
            &mut store,
            SKELETON_PANEL,
            |_| Some((205.0, 100.0)),
            1.0 / 60.0,
        );
    }
    let p = store.loose_rope(IK).to_vec();
    let fora = p
        .iter()
        .map(|q| (q[0] - 205.0).abs())
        .fold(0.0_f32, f32::max);
    assert!(fora < 2.0, "a corda faz cotovelo: {fora} px fora do prumo");
    let abaixo = p.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
    assert!(
        abaixo <= 400.0 + 0.5,
        "a folga escondeu-se atrás do cartão: y {abaixo}"
    );
}

/// T44c (dono 02/11, «travadinha» ao arrastar): a mão já andou quando a corda dá o passo — a ponta
/// do cartão é a borda do cartão ONDE A MÃO O PÕE neste quadro, não onde a pintura o deixou antes.
#[test]
fn a_dragged_card_pulls_its_rope_end_in_the_same_frame() {
    let alvo = |_| Some((450.0, 500.0));
    let mut store = WidgetStore::default();
    solto_com_corda(&mut store, Rect::new(100.0, 100.0, 200.0, 120.0));
    anda_com(&mut store, SKELETON_PANEL, alvo, 1.0 / 60.0);
    store.begin_card_drag(IK, 150.0, 110.0);
    store.update_card_drag(190.0, 110.0);
    anda_com(&mut store, SKELETON_PANEL, alvo, 1.0 / 60.0);
    assert_eq!(
        store.loose_rope(IK)[0],
        [340.0, 220.0],
        "a ponta ficou um quadro atrás da mão"
    );
}

/// ⭐ T36 (aberto do bloco 1): o alfinete — e a corda — sobrevivem a reabrir. Grava-se o `StableId`
/// com o nome; noutro mundo (bits novos) o cartão volta a prender o MESMO objecto, e o mesmo número
/// com outro nome (outro projecto) não prende ninguém.
#[test]
fn a_pin_and_its_rope_survive_reopening_by_stable_id_and_name() {
    use ph2d_ecs::{Name, StableId, World};
    use ph2d_editor_core::interaction::{restore_section_prefs_text, section_prefs_text};
    let mut antes = World::new();
    let osso = antes.spawn((StableId(7), Name::from("Bone 5"))).id();
    let mut store = WidgetStore::default();
    store.set_card_subject(SKELETON_PANEL, Some((osso.to_bits(), "Bone 5".into())));
    store.detach_card(IK, SKELETON_PANEL, (100.0, 100.0), 200.0);
    store.toggle_loose_pin(IK);
    store.toggle_loose_pin(IK);
    super::chave::resolve(&mut store, &antes);
    let texto = section_prefs_text(&store);

    let reabre = |mundo: &World| {
        let mut s = WidgetStore::default();
        restore_section_prefs_text(&mut s, &texto);
        assert_eq!(
            s.loose_card(IK).map(|c| c.pin),
            Some(None),
            "bits não se gravam"
        );
        super::chave::resolve(&mut s, mundo);
        s.loose_card(IK).unwrap()
    };
    let mut depois = World::new();
    depois.spawn((StableId(3), Name::from("Bone 1")));
    let mesmo = depois.spawn((StableId(7), Name::from("Bone 5"))).id();
    let c = reabre(&depois);
    assert_eq!(
        c.pin,
        Some(mesmo.to_bits()),
        "o alfinete volta ao mesmo osso"
    );
    assert!(c.rope, "e com a corda");

    let mut outro = World::new();
    outro.spawn((StableId(7), Name::from("Circle")));
    assert_eq!(
        reabre(&outro).pin,
        None,
        "o mesmo número noutro projecto não prende"
    );
}

/// T41: a corda que se solta (o alfinete volta a «solto») não some num quadro — recolhe-se até ao
/// objecto em `LOOSE_ROPE_REEL_S`, a partir de onde estava.
/// *Mutação: a corda que sai descartada (o `retain` de antes) ⇒ nada a recolher.*
#[test]
fn a_released_rope_reels_into_its_object() {
    let alvo = |b: u64| (b == OSSO).then_some((450.0, 500.0));
    let mut store = WidgetStore::default();
    solto_com_corda(&mut store, Rect::new(100.0, 100.0, 200.0, 120.0));
    for _ in 0..60 {
        anda_com(&mut store, SKELETON_PANEL, alvo, 1.0 / 60.0);
    }
    let antes = store.loose_rope(IK).to_vec();
    store.toggle_loose_pin(IK);
    anda_com(&mut store, SKELETON_PANEL, alvo, 1.0 / 60.0);
    let recolhe = store.reeling_ropes_of(SKELETON_PANEL);
    assert_eq!(recolhe.len(), 1, "a corda sumiu num quadro");
    let p = recolhe[0].1;
    let salto = (p[0][0] - antes[0][0]).hypot(p[0][1] - antes[0][1]);
    assert!(salto < 40.0, "a ponta saltou {salto} px");
    assert_eq!(*p.last().unwrap(), [450.0, 500.0], "presa ao objecto");
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let quadros = (ph2d_card_stack::metrics::LOOSE_ROPE_REEL_S * 60.0).ceil() as usize + 2;
    for _ in 0..quadros {
        anda_com(&mut store, SKELETON_PANEL, alvo, 1.0 / 60.0);
    }
    assert!(
        store.reeling_ropes_of(SKELETON_PANEL).is_empty(),
        "não acabou"
    );
}

/// ⭐ T52 (smoke do dono 02/11): «a 2.ª corda não tem gravidade» — a shell anda o Bones e o
/// Inspector no MESMO quadro e o relógio era um só: a 2.ª chamada tinha `dt ≈ 0`. Com o relógio
/// real, as duas cordas pendem. *Mutação: um relógio para todos os painéis ⇒ a 2.ª fica reta.*
#[test]
fn every_panel_rope_sags_under_the_real_clock() {
    let alvo = |_: u64| Some((450.0, 500.0));
    let outro = NodeId(77);
    let k2 = NodeId(55);
    let mut store = WidgetStore::default();
    solto_com_corda(&mut store, Rect::new(100.0, 100.0, 200.0, 120.0));
    store.set_card_subject(outro, Some((OSSO, "Bone 5".into())));
    store.detach_card(k2, outro, (100.0, 100.0), 200.0);
    store.set_loose_card_rect(k2, Rect::new(100.0, 100.0, 200.0, 120.0));
    store.toggle_loose_pin(k2);
    store.toggle_loose_pin(k2);
    let fim = std::time::Instant::now() + std::time::Duration::from_millis(500);
    while std::time::Instant::now() < fim {
        super::anda(&mut store, SKELETON_PANEL, alvo);
        super::anda(&mut store, outro, alvo);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    // A flecha: o quanto o ponto mais baixo desce abaixo da corda reta (ecrã, `y` para baixo).
    let flecha = |p: &[[f32; 2]]| {
        let ([x0, y0], [x1, y1]) = (p[0], p[p.len() - 1]);
        p.iter()
            .map(|q| q[1] - (y0 + (q[0] - x0) * (y1 - y0) / (x1 - x0)))
            .fold(0.0_f32, f32::max)
    };
    let (a, b) = (flecha(store.loose_rope(IK)), flecha(store.loose_rope(k2)));
    assert!(
        a.min(b) > 0.6 * a.max(b) && a > 30.0,
        "flechas {a} e {b} px: uma corda não pende"
    );
}
