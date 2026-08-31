//! ⭐⭐⭐ **O ARTISTA ESCOLHE QUAL PAINEL VAI EM CADA LUGAR** — a decisão **D4**, medida como gesto.
//!
//! > *«Lugares pré-definidos. O artista escolhe **QUAL painel vai em cada lugar**, e arrasta a
//! > divisória — mas não inventa lugares novos.»* — `00_DECISOES_DO_ENIO.md`, D4
//!
//! # ⭐ E a D1 aqui não é uma verificação: é um `Constraint`
//!
//! > *«O erro não é detectado, é **inexprimível**.»*
//!
//! Um encaixe que o painel não permite **não é oferecido**: não se pinta, não se testa, não existe
//! para aquele gesto. É a diferença entre *«largou e o app recusou»* — em que o artista faz o gesto,
//! vê a resposta e não sabe porquê — e *«não havia onde largar»*.

use ph2d_editor_core::screens::hero::{HeroScreen, slot_tabs};
use ph2d_editor_core::screens::slot::Slot;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 1024.0,
};

fn all_panel_ids() -> Vec<&'static str> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            v.push(p.manifest.id);
        }
    });
    v
}

fn node_of(id: &str) -> ph2d_editor_core::NodeId {
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.id == id)
            .map(|p| p.manifest.panel_node_id)
            .unwrap_or_else(|| panic!("painel {id} não está registado"))
    })
}

fn settled(open: &[&str]) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    for n in all_panel_ids() {
        h.panel_visibility.insert(n, open.contains(&n));
    }
    paint(&mut h, 3);
    h
}

fn paint(h: &mut HeroScreen, frames: usize) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..frames {
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
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

/// O gesto real: Down na aba, Move até ao alvo, Up.
fn drag_tab(h: &mut HeroScreen, tab: Rect, to: (f32, f32)) {
    let from = (tab.x + tab.w * 0.5, tab.y + tab.h * 0.5);
    let arena = bumpalo::Bump::new();
    for ev in [
        pointer(PointerKind::Down, from.0, from.1),
        pointer(PointerKind::Move, to.0, to.1),
        pointer(PointerKind::Up, to.0, to.1),
    ] {
        let events =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena);
        let evs: Vec<_> = events.to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

fn rect_of(h: &HeroScreen, id: &str) -> Option<Rect> {
    h.store.panel_rect(node_of(id))
}

/// Traz um ocupante à frente do encaixe dele.
///
/// ⚠️ **É preciso, e é a própria feature a dizê-lo:** num encaixe com duas abas **só uma pinta**,
/// então medir o rect de um painel exige tê-lo à frente. Sem isto o gate media `None` e a falha
/// leria-se como *«o arrasto não funcionou»* quando o que faltava era a aba estar aberta.
fn bring_to_front(h: &mut HeroScreen, id: &str) {
    h.store.bump_panel_z(node_of(id));
    paint(h, 2);
}

/// ⭐⭐⭐ **Arrastar a aba do Mixer para a coluna da esquerda move o painel para lá.**
#[test]
fn a_tab_dragged_to_the_other_column_moves_the_panel() {
    let mut h = settled(&["inspector", "audio_mixer"]);
    bring_to_front(&mut h, "audio_mixer");
    let l = h.last_layout.expect("layout");
    let before = rect_of(&h, "audio_mixer").expect("o mixer desenhou");

    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let bar = l.slot_tabs[Slot::RightTop as usize];
    assert!(bar.h > 0.0, "sem fila de abas não há gesto a medir");
    let tabs = slot_tabs::tab_rects(bar, occ.len());
    let i = occ
        .iter()
        .position(|o| o.id == "audio_mixer")
        .expect("o mixer é ocupante");

    let left = l.slot_rects(slot_tabs::occupied(&h)).get(Slot::LeftTop);
    drag_tab(
        &mut h,
        tabs[i],
        (left.x + left.w * 0.5, left.y + left.h * 0.5),
    );
    paint(&mut h, 2);

    let after = rect_of(&h, "audio_mixer").expect("o mixer continua a desenhar");
    assert_ne!(
        before.x, after.x,
        "o painel não se moveu: antes {before:?}, depois {after:?}"
    );
    assert!(
        after.x < before.x,
        "ele foi para a coluna errada ({after:?}, a esquerda é {left:?})"
    );
    assert_eq!(
        slot_tabs::occupants(&h, Slot::LeftTop)
            .iter()
            .filter(|o| o.id == "audio_mixer")
            .count(),
        1,
        "ele pinta à esquerda e continua a ser CONTADO à direita — a posição tem duas respostas"
    );
}

/// ⭐⭐ **E um TOQUE na aba continua a trocar de aba** — o mesmo dedo, dois gestos.
#[test]
fn a_tap_on_a_tab_still_switches_it_and_never_moves_the_panel() {
    let mut h = settled(&["inspector", "audio_mixer"]);
    let l = h.last_layout.expect("layout");
    let before = rect_of(&h, "audio_mixer").or_else(|| rect_of(&h, "inspector"));

    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let bar = l.slot_tabs[Slot::RightTop as usize];
    let tabs = slot_tabs::tab_rects(bar, occ.len());
    // O que está atrás — largar-lhe um toque tem de o trazer à frente.
    let behind = occ[0].id;
    let i = occ.iter().position(|o| o.id == behind).unwrap();
    let c = (tabs[i].x + tabs[i].w * 0.5, tabs[i].y + tabs[i].h * 0.5);
    drag_tab(&mut h, tabs[i], c); // Down e Up no MESMO ponto: zero deslocamento
    paint(&mut h, 2);

    assert!(
        rect_of(&h, behind).is_some(),
        "o toque na aba de `{behind}` não a trouxe à frente"
    );
    assert_eq!(
        slot_tabs::occupants(&h, Slot::RightTop).len(),
        occ.len(),
        "um toque mudou alguém de encaixe: antes {before:?}"
    );
}

/// ⛔⛔ **O ENCAIXE QUE O PAINEL NÃO PERMITE NÃO É OFERECIDO** — a D1 como `Constraint`.
#[test]
fn a_slot_the_panel_forbids_is_never_offered_as_a_target() {
    let h = settled(&["inspector", "audio_mixer", "timeline"]);

    let column = node_of("inspector");
    let offered: Vec<Slot> = slot_tabs::drop_targets(&h, column)
        .into_iter()
        .map(|(s, _)| s)
        .collect();
    assert!(
        !offered.contains(&Slot::Bottom),
        "o Inspector foi oferecido a faixa de BAIXO ({offered:?}): ela tem 240 px de altura e uma \
         lista de propriedades ali fica com duas linhas"
    );
    assert!(
        offered.contains(&Slot::LeftTop) && offered.contains(&Slot::RightTop),
        "as duas colunas têm de ser oferecidas ({offered:?}) — sem elas o gesto não existe"
    );

    // ⚠️ A tira do Flip não está nas features de omissão desta build; quem declara a faixa de
    // baixo e está sempre registado é o **timeline**. A declaração é o sujeito do teste, e ele
    // tem-na igual.
    let strip = node_of("timeline");
    let offered: Vec<Slot> = slot_tabs::drop_targets(&h, strip)
        .into_iter()
        .map(|(s, _)| s)
        .collect();
    assert_eq!(
        offered,
        vec![Slot::Bottom],
        "uma TIRA foi oferecida uma COLUNA: numa de 304 px ela mostraria dois quadros"
    );

    // ⛔ E o centro nunca é destino de nada: ele é do editor (spec §2, regra 4).
    for name in ["inspector", "audio_mixer", "timeline"] {
        let offered: Vec<Slot> = slot_tabs::drop_targets(&h, node_of(name))
            .into_iter()
            .map(|(s, _)| s)
            .collect();
        assert!(
            !offered.contains(&Slot::Center),
            "`{name}` foi oferecido o CENTRO, que é a área de desenho"
        );
    }
}

/// ⚠️ **Largar FORA de todo destino legal não faz nada** — é a forma de desistir.
#[test]
fn dropping_outside_every_legal_target_changes_nothing() {
    let mut h = settled(&["inspector", "audio_mixer"]);
    bring_to_front(&mut h, "audio_mixer");
    let l = h.last_layout.expect("layout");
    let before = rect_of(&h, "audio_mixer").expect("o mixer desenhou");

    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let bar = l.slot_tabs[Slot::RightTop as usize];
    let tabs = slot_tabs::tab_rects(bar, occ.len());
    let i = occ.iter().position(|o| o.id == "audio_mixer").unwrap();

    // O meio da área de desenho: nenhum encaixe legal o contém.
    let da = l.draw_area;
    drag_tab(&mut h, tabs[i], (da.x + da.w * 0.5, da.y + da.h * 0.5));
    paint(&mut h, 2);

    assert_eq!(
        rect_of(&h, "audio_mixer"),
        Some(before),
        "largar sobre o desenho moveu o painel"
    );
}

/// ⚠️ **UM EMPURRÃO de 5 px sobre a própria aba ainda TROCA de aba.**
///
/// O limiar do arrasto é de poucos pixels, e um dedo que carrega numa aba mexe-se sempre um pouco.
/// Se esse empurrão deixasse de trocar de aba, o gesto mais comum do painel ficaria **intermitente**
/// — funciona ou não conforme a firmeza da mão, que é a pior espécie de defeito de interface.
#[test]
fn a_five_pixel_nudge_on_a_tab_still_switches_it() {
    let mut h = settled(&["inspector", "audio_mixer"]);
    bring_to_front(&mut h, "inspector");
    let l = h.last_layout.expect("layout");

    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let bar = l.slot_tabs[Slot::RightTop as usize];
    let tabs = slot_tabs::tab_rects(bar, occ.len());
    let i = occ.iter().position(|o| o.id == "audio_mixer").unwrap();
    let t = tabs[i];
    assert!(
        rect_of(&h, "audio_mixer").is_none(),
        "o mixer já estava à frente — o gate não mediria a troca"
    );

    // ⚠️ Um empurrão MAIOR que o limiar, mas dentro da própria aba.
    let nudge = ph2d_editor_core::interaction::TAB_DRAG_THRESHOLD_PX + 2.0;
    let from = (t.x + t.w * 0.5, t.y + t.h * 0.5);
    let arena = bumpalo::Bump::new();
    for ev in [
        pointer(PointerKind::Down, from.0, from.1),
        pointer(PointerKind::Move, from.0 + nudge, from.1),
        pointer(PointerKind::Up, from.0 + nudge, from.1),
    ] {
        let events =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena);
        let evs: Vec<_> = events.to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
    paint(&mut h, 2);

    assert!(
        rect_of(&h, "audio_mixer").is_some(),
        "um empurrão de {nudge:.0} px dentro da aba não a trouxe à frente — a troca de aba passou a \
         depender da firmeza da mão"
    );
}

/// ⛔ **UM TOQUE PARADO NUNCA ACENDE AS ZONAS DE LARGADA.**
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE.** Apagar o limiar do arrasto deixava a suíte
/// verde — porque o que ele decide não é o *resultado* do gesto (largar sobre o próprio encaixe é
/// um no-op), é o que se **vê**: sem ele, pousar o dedo numa aba pinta as colunas todas realçadas
/// e apaga-as ao levantar. *Um piscar que nenhuma asserção de estado final apanha.*
#[test]
fn a_still_press_on_a_tab_never_lights_the_drop_zones() {
    let mut h = settled(&["inspector", "audio_mixer"]);
    let l = h.last_layout.expect("layout");
    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let bar = l.slot_tabs[Slot::RightTop as usize];
    let tabs = slot_tabs::tab_rects(bar, occ.len());
    let t = tabs[0];
    let c = (t.x + t.w * 0.5, t.y + t.h * 0.5);

    let arena = bumpalo::Bump::new();
    let _ = ph2d_editor_core::interaction::dispatch_pointer(
        &mut h.store,
        &h.hit_index,
        pointer(PointerKind::Down, c.0, c.1),
        &arena,
    );
    assert!(
        h.store.tab_being_dragged().is_none(),
        "o dedo pousou e as zonas de largada acenderam: um arrasto começou sem ninguém o pedir"
    );

    // ⭐ E o controlo: passado o limiar, elas TÊM de acender — senão este gate estaria a medir uma
    // feature apagada em vez de um limiar a funcionar.
    let far = ph2d_editor_core::interaction::TAB_DRAG_THRESHOLD_PX + 2.0;
    let _ = ph2d_editor_core::interaction::dispatch_pointer(
        &mut h.store,
        &h.hit_index,
        pointer(PointerKind::Move, c.0 + far, c.1),
        &arena,
    );
    assert!(
        h.store.tab_being_dragged().is_some(),
        "passado o limiar o arrasto não começou — o gesto de mover um painel não existe"
    );
}
