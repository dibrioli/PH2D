//! ⭐⭐⭐ **A REGRA 1 DO MODELO DE ÁREAS, medida sobre o quadro real:** *um encaixe hospeda `0..n`
//! painéis, e com `n > 1` eles são **abas***.
//!
//! # Por que estes gates moram AQUI
//!
//! Eles pintam um quadro e leem o que os painéis **publicaram**. Isso precisa do registry de
//! painéis, que vive nesta crate — na `ph2d-editor-core` o `test_support::ensure_panel_registry` é
//! um `{}` e a varredura correria sobre zero painéis, verde e vazia.
//!
//! # ⚠️ O quadro tem de ser pintado MAIS DE UMA VEZ
//!
//! O `DockSides::from_published` lê os rects do quadro **anterior**: no primeiro quadro nenhuma
//! coluna está reservada e a área de desenho ocupa a largura toda. Um gate de um quadro só mediria
//! o estado transitório.

use ph2d_editor_core::screens::hero::{HeroScreen, slot_tabs};
use ph2d_editor_core::screens::slot::Slot;
use ph2d_editor_core::zones::Rect;
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

/// Pinta três quadros com **exactamente** estes painéis visíveis e devolve o hero.
///
/// ⚠️ **Exactamente**, e não «estes mais os de omissão»: o `inspector` e a `hierarchy` nascem
/// visíveis, então uma lista aditiva punha SEMPRE um terceiro ocupante na coluna da direita — e o
/// que se quer medir aqui é o encaixe com os ocupantes que o teste nomeia.
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

/// Quais painéis ancorados publicaram rect neste quadro.
fn published(h: &HeroScreen) -> Vec<(&'static str, Rect)> {
    let mut v = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            if p.manifest.can_float {
                continue;
            }
            if let Some(r) = h.store.panel_rect(p.manifest.panel_node_id) {
                v.push((p.manifest.id, r));
            }
        }
    });
    v
}

/// ⭐ **O estado de omissão do app NÃO tem abas** — e é isto que torna a wave inerte enquanto o
/// artista não abrir dois painéis do mesmo lado.
#[test]
fn the_default_app_shows_no_tab_row_at_all() {
    let h = settled(&[]);
    let l = h.last_layout.expect("o quadro publicou o layout");
    for slot in Slot::ALL {
        assert_eq!(
            l.slot_tabs[slot as usize].h,
            0.0,
            "{slot:?} reservou faixa de abas com {} ocupante(s) no arranque",
            slot_tabs::occupants(&h, slot).len()
        );
    }
}

/// ⭐⭐⭐ **O caso do Enio: MIX e WAVE ao mesmo tempo.** Eles partilham a coluna da direita e
/// **um só** desenha; a fila tem as duas abas.
#[test]
fn the_mixer_and_the_editor_share_one_column_as_two_tabs() {
    let h = settled(&["audio_mixer", "audio_editor"]);
    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let names: Vec<_> = occ.iter().map(|o| o.id).collect();
    assert!(
        names.contains(&"audio_mixer") && names.contains(&"audio_editor"),
        "os dois não ocupam o mesmo encaixe: {names:?}"
    );

    let l = h.last_layout.expect("layout");
    let bar = l.slot_tabs[Slot::RightTop as usize];
    assert!(
        bar.h > 0.0,
        "dois ocupantes e nenhuma faixa de abas ({bar:?})"
    );
    assert_eq!(
        slot_tabs::tab_rects(bar, occ.len()).len(),
        occ.len(),
        "a fila não mostra uma aba por ocupante"
    );

    // ⛔ **E só UM publica rect** — é isto que separa abas de painéis empilhados.
    assert_eq!(occ.len(), 2, "o encaixe tem outros ocupantes: {names:?}");
    let drawn: Vec<_> = published(&h)
        .into_iter()
        .filter(|(id, _)| *id == "audio_mixer" || *id == "audio_editor")
        .collect();
    assert_eq!(
        drawn.len(),
        1,
        "os dois desenharam ao mesmo tempo — a aba não escondeu ninguém: {drawn:?}"
    );

    // ⭐ E a coluna começa DEPOIS da faixa: o painel da frente não fica por baixo das abas.
    assert!(
        drawn[0].1.y >= bar.y + bar.h - 0.001,
        "o painel da frente desenha por cima da fila de abas ({:?} contra {bar:?})",
        drawn[0].1
    );
}

/// ⭐⭐ **Clicar numa aba troca quem desenha** — o gesto, não só a geometria.
#[test]
fn clicking_a_tab_changes_which_panel_draws() {
    let mut h = settled(&["audio_mixer", "audio_editor"]);
    let before = published(&h)
        .into_iter()
        .find(|(id, _)| *id == "audio_mixer" || *id == "audio_editor")
        .expect("um dos dois desenhou")
        .0;
    let other = if before == "audio_mixer" {
        "audio_editor"
    } else {
        "audio_mixer"
    };
    let other_node = node_of(other);

    let consumed = h.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(
        slot_tabs::tab_node_id(other_node),
    ));
    assert!(consumed, "o clique na aba não foi consumido por ninguém");
    paint(&mut h, 2);

    let after = published(&h)
        .into_iter()
        .find(|(id, _)| *id == "audio_mixer" || *id == "audio_editor")
        .expect("um dos dois desenhou")
        .0;
    assert_eq!(
        after, other,
        "a aba foi clicada e o painel que desenha não mudou (antes {before}, depois {after})"
    );
}

/// ⭐⭐⭐ **O encaixe que um painel DECLARA é onde ele PINTA.**
///
/// ⛔ Esta é a metade que faltava desde que o `DEFAULT_SLOT` nasceu: ele tinha um default de
/// `RightTop`, **20 dos 21** painéis herdavam-no, e **três mentiam** (`hierarchy` publica a coluna
/// da esquerda, `timeline` e `flip_frames` a faixa de baixo). *Uma declaração que ninguém confronta
/// com a realidade é decoração* — e ela só começou a custar quando as abas passaram a derivar dela
/// quem divide o quê.
///
/// ⚠️ **Um painel de cada vez, e a razão é a própria feature:** com todos abertos treze deles
/// ocupam a mesma coluna e **doze ficam escondidos por abas** — a varredura mediria dois e leria
/// como aprovada. *Um gate cuja população a feature nova esvazia passa sobre nada.*
#[test]
fn the_slot_a_panel_declares_is_where_it_paints() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut declared = std::collections::BTreeMap::new();
    let mut floats = std::collections::BTreeSet::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            declared.insert(p.manifest.id, p.manifest.default_slot);
            if p.manifest.can_float {
                floats.insert(p.manifest.id);
            }
        }
    });

    let mut liars = Vec::new();
    let mut measured = 0usize;
    for name in all_panel_ids() {
        if floats.contains(name) {
            continue;
        }
        let h = settled(&[name]);
        let Some(r) = h.store.panel_rect(node_of(name)) else {
            continue; // este painel não desenha sem a ferramenta dele activa
        };
        let l = h.last_layout.expect("layout");
        let slot = declared[name];
        let band = l.slot_rects(slot_tabs::occupied(&h)).get(slot);
        measured += 1;
        // Tolerância de meio pixel: o `min` da banda e a faixa de abas mexem em frações.
        let inside = r.x >= band.x - 0.5
            && r.x + r.w <= band.x + band.w + 0.5
            && r.y >= band.y - 0.5
            && r.y + r.h <= band.y + band.h + 0.5;
        if !inside {
            liars.push(format!(
                "{name} declara {slot:?} ({band:?}) e pinta em {r:?}"
            ));
        }
    }
    assert!(
        measured >= 10,
        "só {measured} painéis ancorados publicaram rect sozinhos — a varredura ficou vazia"
    );
    assert!(
        liars.is_empty(),
        "painéis cuja DECLARAÇÃO de encaixe não descreve onde eles pintam:\n  {}",
        liars.join("\n  ")
    );
}

/// ⛔ **A lista de rects que a reserva de abas empurra tem de cobrir os que os painéis usam.**
///
/// Se um campo novo de coluna aparecer no `HeroLayout` e não entrar no `docked_rects_mut`, os
/// painéis dele desenham **por baixo** das abas — e nenhum outro gate o vê, porque o rect publicado
/// continua dentro da coluna.
///
/// ⚠️ O controlo é o `saw_a_bar`: sem uma faixa reservada em quadro nenhum, este gate mede zero.
#[test]
fn every_docked_layout_rect_is_pushed_by_a_tab_bar() {
    let mut under = Vec::new();
    let mut saw_a_bar = 0usize;
    // Cada painel ancorado emparelhado com o Inspector — dois ocupantes, logo uma faixa.
    for name in all_panel_ids() {
        if name == "inspector" {
            continue;
        }
        let h = settled(&["inspector", name]);
        let l = h.last_layout.expect("layout");
        for slot in Slot::ALL {
            let bar = l.slot_tabs[slot as usize];
            if bar.h <= 0.0 {
                continue;
            }
            saw_a_bar += 1;
            for (id, r) in published(&h) {
                let hits = r.x < bar.x + bar.w
                    && bar.x < r.x + r.w
                    && r.y < bar.y + bar.h - 0.001
                    && bar.y < r.y + r.h;
                if hits {
                    under.push(format!(
                        "{id} ({r:?}) desenha debaixo da fila de {slot:?} ({bar:?})"
                    ));
                }
            }
        }
    }
    assert!(
        saw_a_bar >= 5,
        "só {saw_a_bar} faixas de abas em toda a varredura — o gate mediria o vazio"
    );
    assert!(
        under.is_empty(),
        "rects docados que a reserva de abas não empurrou — o campo deles falta em \
         `HeroLayout::docked_rects_mut`:\n  {}",
        under.join("\n  ")
    );
}
