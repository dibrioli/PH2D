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
        slot_tabs::tab_layout(
            &occ,
            slot_tabs::chosen(&h, Slot::RightTop),
            bar,
            &mut TextSystem::without_system_fonts()
        )
        .len(),
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

/// ⭐⭐⭐ **CLICAR NUMA ABA NÃO A MOVE NA FILA** — o report de 2026-09-07.
///
/// > *«quando se clica na aba ela troca de lugar com a outra aba. não permita isso»*
///
/// ⛔⛔ **A causa era um facto a responder a DUAS perguntas.** A fila era ordenada pela **ordem z**,
/// e a ordem z é *«quem foi tocado por último»* — logo responder a *«qual está à frente»* mexia em
/// *«em que ordem elas se sentam»*. Hoje a **ordem é a do registo** (estável) e a **escolha** é o
/// topo do z. *Uma aba só muda de lugar quando o artista a ARRASTA.*
///
/// ⚠️ **A segunda asserção é o controlo**: sem ela, um `chosen` que nunca mudasse passaria — a fila
/// ficaria imóvel e o clique inerte, que é o defeito oposto e igualmente mau.
#[test]
fn clicking_a_tab_never_moves_it_in_the_row() {
    let mut h = settled(&["audio_mixer", "audio_editor", "inspector"]);
    let order = |h: &HeroScreen| -> Vec<&'static str> {
        slot_tabs::occupants(h, Slot::RightTop)
            .iter()
            .map(|o| o.id)
            .collect()
    };

    let before = order(&h);
    assert!(
        before.len() >= 3,
        "a fixtura tem de ter três ocupantes para uma troca ser observável: {before:?}"
    );

    // Escolhe um que NÃO esteja à frente — trocar pelo próprio não move nada e não mede nada.
    let front = slot_tabs::chosen(&h, Slot::RightTop);
    let target = before
        .iter()
        .copied()
        .find(|id| Some(node_of(id)) != front)
        .expect("algum ocupante não está à frente");

    let consumed = h.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(
        slot_tabs::tab_node_id(node_of(target)),
    ));
    assert!(consumed, "o clique na aba não foi consumido");
    paint(&mut h, 3);

    assert_eq!(
        order(&h),
        before,
        "clicar em «{target}» reordenou a fila — era o report do dono"
    );
    assert_eq!(
        slot_tabs::chosen(&h, Slot::RightTop),
        Some(node_of(target)),
        "controlo partido: a fila ficou imóvel porque o clique não trocou a escolha — o defeito \
         oposto, e igualmente mau"
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

/// A fila, pelos ids, na ordem em que ela é vista.
fn row(h: &HeroScreen, slot: Slot) -> Vec<&'static str> {
    slot_tabs::occupants(h, slot).iter().map(|o| o.id).collect()
}

/// **Arrasta a aba de `panel` até `to_x`, dentro da fila do encaixe, e larga.**
///
/// ⚠️ O percurso é o REAL: `begin` no centro da aba (que é onde o `pointer_down` a arma), `update`
/// até ao alvo, `end`. Escrever o `tab_drop` à mão saltaria o limiar — e é ele que separa um clique
/// de um arrasto, logo um gate que o salte não mede o gesto.
fn drag_tab(h: &mut HeroScreen, panel: &str, slot: Slot, to_x: f32) {
    let node = node_of(panel);
    let bar = h.last_layout.expect("layout").slot_tabs[slot as usize];
    let from = h
        .hit_index
        .rect_for(slot_tabs::tab_node_id(node))
        .unwrap_or_else(|| panic!("a aba de {panel} não foi pintada — o gate mediria o nada"));
    let y = bar.y + bar.h * 0.5;
    h.store.begin_tab_drag(node, from.x + from.w * 0.5, y);
    h.store.update_tab_drag(to_x, y);
    h.store.end_tab_drag();
    paint(h, 3);
}

/// ⭐⭐⭐ **ARRASTAR UMA ABA MOVE-A NA FILA** — o report de 2026-09-08.
///
/// > *«não é possível reordenar as abas arrastando com o mouse»* — Enio.
///
/// A largada só sabia responder *«que ENCAIXE?»*, então arrastar dentro da própria fila era um
/// no-op silencioso: o painel já estava naquele encaixe. A ordem passou a ser um dado — ver
/// `WidgetStore::set_tab_row_order`.
#[test]
fn dragging_a_tab_within_its_row_moves_it() {
    let mut h = settled(&["audio_mixer", "audio_editor", "inspector"]);
    let before = row(&h, Slot::RightTop);
    assert!(
        before.len() >= 3,
        "a fixtura precisa de três abas para uma reordenação ser observável: {before:?}"
    );

    // A ÚLTIMA da fila, largada sobre a metade esquerda da PRIMEIRA ⇒ ela passa a ser a primeira.
    let last = before[before.len() - 1];
    let first_rect = h
        .hit_index
        .rect_for(slot_tabs::tab_node_id(node_of(before[0])))
        .expect("a primeira aba foi pintada");
    drag_tab(&mut h, last, Slot::RightTop, first_rect.x + 1.0);

    let after = row(&h, Slot::RightTop);
    let mut want = vec![last];
    want.extend(before.iter().copied().filter(|id| *id != last));
    assert_eq!(
        after, want,
        "arrastar «{last}» para o início da fila não a moveu — era o report do dono \
         (antes {before:?})"
    );

    // ⚠️ **O controlo do lado oposto:** um arrasto que acaba onde ele já estava não pode mexer em
    //    nada. Sem ele, um código que embaralhasse a fila a cada largada passaria na metade de cima.
    let settled_row = row(&h, Slot::RightTop);
    let own = h
        .hit_index
        .rect_for(slot_tabs::tab_node_id(node_of(last)))
        .expect("a aba arrastada continua pintada");
    drag_tab(&mut h, last, Slot::RightTop, own.x + 1.0);
    assert_eq!(
        row(&h, Slot::RightTop),
        settled_row,
        "largar uma aba no lugar dela própria reordenou a fila"
    );
}

/// ⭐⭐ **E a ordem arrumada SOBREVIVE a fechar e reabrir a coluna** — a involução do
/// `dock_columns` devolve os painéis, e a fila tem de os devolver na ordem que o artista deixou.
///
/// ⛔ Sem isto, o gesto de retracção desfazia a arrumação em silêncio — o defeito que o report de
/// 2026-09-08 juntava ao outro, e que só se vê quando as duas features existem ao mesmo tempo.
#[test]
fn the_row_the_artist_arranged_survives_the_column_closing() {
    use ph2d_editor_core::screens::hero::dock_columns;
    use ph2d_editor_core::screens::layout::DockSide;

    let mut h = settled(&["audio_mixer", "audio_editor", "inspector"]);
    let before = row(&h, Slot::RightTop);
    let last = before[before.len() - 1];
    let first_rect = h
        .hit_index
        .rect_for(slot_tabs::tab_node_id(node_of(before[0])))
        .expect("a primeira aba foi pintada");
    drag_tab(&mut h, last, Slot::RightTop, first_rect.x + 1.0);
    let arranged = row(&h, Slot::RightTop);
    assert_ne!(
        arranged, before,
        "controlo partido: o arrasto não arrumou nada, logo o resto do gate não mede a arrumação"
    );

    assert!(dock_columns::close(&mut h, DockSide::Right, Some(300.0)));
    paint(&mut h, 3);
    assert!(dock_columns::open(&mut h, DockSide::Right));
    paint(&mut h, 3);

    assert_eq!(
        row(&h, Slot::RightTop),
        arranged,
        "fechar e reabrir a coluna desfez a ordem que o artista tinha arrumado"
    );
}

/// ⭐⭐ **A MARCA diz o lugar, e ela cai na fronteira onde a aba vai entrar.**
///
/// ⛔ As zonas de largada realçam a COLUNA; num arrasto dentro da própria fila a coluna realçada é
/// a mesma antes e depois de atravessar a vizinha — sem esta marca o artista larga às cegas.
#[test]
fn the_drop_mark_sits_where_the_tab_will_land() {
    let h = settled(&["audio_mixer", "audio_editor", "inspector"]);
    let r = row(&h, Slot::RightTop);
    assert!(r.len() >= 3, "a fixtura precisa de três abas: {r:?}");

    let first = h
        .hit_index
        .rect_for(slot_tabs::tab_node_id(node_of(r[0])))
        .expect("a primeira aba foi pintada");
    let last = node_of(r[r.len() - 1]);
    let bar = h.last_layout.expect("layout").slot_tabs[Slot::RightTop as usize];
    let y = bar.y + bar.h * 0.5;

    // Sobre a metade ESQUERDA da primeira aba ⇒ a marca fica na borda esquerda dela.
    let caret = slot_tabs::tab_drop_caret(&h, last, (first.x + 1.0, y))
        .expect("o dedo está sobre a fila — tem de haver marca");
    assert!(
        (caret.x + caret.w * 0.5 - first.x).abs() < 2.0,
        "a marca ficou em x={:.1} e a aba de destino começa em x={:.1}",
        caret.x + caret.w * 0.5,
        first.x
    );
    assert!(
        (caret.y - bar.y).abs() < 0.5 && (caret.h - bar.h).abs() < 0.5,
        "a marca não tem a altura da fila: {caret:?} contra {bar:?}"
    );

    // ⚠️ **O controlo:** fora da fila não há marca — senão o gate acima passaria sobre uma marca
    //    que é pintada sempre, em qualquer sítio.
    assert!(
        slot_tabs::tab_drop_caret(&h, last, (first.x + 1.0, bar.y + bar.h + 200.0)).is_none(),
        "há marca com o dedo FORA da fila de abas"
    );
}

/// ⭐⭐⭐ **NENHUMA ABA DESAPARECE ENQUANTO ELAS COUBEREM NO PISO** — e o que não é pintado não se
/// clica.
///
/// ⛔ Uma aba sem rect não está no índice de acerto: o painel dela só voltava fechando-o e
/// reabrindo-o no menu *Window*, que é um caminho que ninguém adivinha. Três nomes desta casa medem
/// ~231 px e a coluna mais estreita deixa **212** úteis — logo o buraco era alcançável arrastando
/// a divisória.
///
/// ⚠️ **O gate mede a coluna REAL**, não uma inventada: é a largura que o app dá de fábrica que
/// tem de servir a população que o app deixa abrir naquele encaixe.
#[test]
fn every_tab_is_painted_while_they_fit_at_the_floor() {
    let h = settled(&[
        "audio_mixer",
        "audio_editor",
        "inspector",
        "physics",
        "tokens",
    ]);
    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let bar = h.last_layout.expect("layout").slot_tabs[Slot::RightTop as usize];
    assert!(bar.h > 0.0, "sem faixa de abas não há o que medir");
    assert!(
        occ.len() >= 5,
        "controlo partido: {} ocupantes é pouco para a fila transbordar sem o piso",
        occ.len()
    );

    let front = slot_tabs::chosen(&h, Slot::RightTop);
    let painted = slot_tabs::tab_layout(&occ, front, bar, &mut TextSystem::without_system_fonts());
    assert_eq!(
        painted.len(),
        occ.len(),
        "{} de {} abas ficaram por pintar na coluna de fábrica ({} px) — e uma aba não pintada é \
         um painel inalcançável",
        occ.len() - painted.len(),
        occ.len(),
        bar.w
    );

    // ⚠️⚠️ **E a metade que MEDE O PISO precisa da faixa em que ele MORDE.** Medido por mutação
    //    em 2026-09-08: pôr o piso a zero deixa esta fixtura intacta — cinco nomes encolhem para
    //    ~58 px, muito acima dos 22, e a asserção abaixo passaria sobre um piso que não existe.
    //    *Uma asserção que a mutação não mata é uma asserção cuja fixtura não produz o fenómeno.*
    //    ⇒ a faixa espremida abaixo é o caso em que as cinco só cabem SE cada uma parar no piso.
    let squeezed = ph2d_editor_core::zones::Rect::new(bar.x, bar.y, 120.0, bar.h);
    let at_floor = slot_tabs::tab_layout(
        &occ,
        front,
        squeezed,
        &mut TextSystem::without_system_fonts(),
    );
    assert_eq!(
        at_floor.len(),
        occ.len(),
        "numa faixa de {} px as {} abas ainda cabem no piso ({} px cada) e mesmo assim {} ficaram \
         por pintar",
        squeezed.w,
        occ.len(),
        bar.h,
        occ.len() - at_floor.len()
    );
    let below: Vec<_> = at_floor
        .iter()
        .filter(|(_, r)| r.w < bar.h - 0.5)
        .map(|(o, r)| format!("{} a {:.1} px, abaixo do piso de {:.0}", o.id, r.w, bar.h))
        .collect();
    assert!(
        below.is_empty(),
        "o piso não segurou — abas mais estreitas do que altas:\n  {}",
        below.join("\n  ")
    );

    // ⚠️ **E o piso é o que torna a coluna de fábrica suficiente:** sem ele a soma dos nomes passa
    //    a coluna. Cada aba tem de ser pelo menos tão larga quanto alta — a forma que o ícone
    //    virá ocupar.
    let thin: Vec<_> = painted
        .iter()
        .filter(|(_, r)| r.w < bar.h - 0.5)
        .map(|(o, r)| {
            format!(
                "{} com {:.0} px de largura para {:.0} de altura",
                o.id, r.w, bar.h
            )
        })
        .collect();
    assert!(
        thin.is_empty(),
        "abas mais estreitas do que altas — deixam de ser alvo:\n  {}",
        thin.join("\n  ")
    );

    // E a soma não passa a faixa: encolher tem de CABER, não empurrar para fora.
    let right = painted
        .iter()
        .map(|(_, r)| r.x + r.w)
        .fold(f32::MIN, f32::max);
    assert!(
        right <= bar.x + bar.w + 0.5,
        "a fila terminou em x={right:.1} e a faixa acaba em {:.1}",
        bar.x + bar.w
    );
}

/// ⭐⭐ **UMA ABA ENCOLHIDA CONTINUA A LER-SE COMO ABA** — há uma divisória entre vizinhas.
///
/// ⛔ Uma aba inactiva não pinta fundo; encolhida, o nome elide e ela desaparece. Foi o report de
/// 2026-09-08 (*«as abas somem»*) e a razão de o encolhimento ter sido revertido nesse dia.
///
/// ⚠️ **Nenhuma divisória toca a escolhida** — ela já tem corpo próprio, e uma linha ao lado dele
/// leria como uma segunda borda.
#[test]
fn a_squeezed_tab_still_reads_as_a_tab() {
    let h = settled(&[
        "audio_mixer",
        "audio_editor",
        "inspector",
        "physics",
        "tokens",
    ]);
    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let bar = h.last_layout.expect("layout").slot_tabs[Slot::RightTop as usize];
    let front = slot_tabs::chosen(&h, Slot::RightTop);
    let squeezed = ph2d_editor_core::zones::Rect::new(bar.x, bar.y, 120.0, bar.h);
    let painted = slot_tabs::tab_layout(
        &occ,
        front,
        squeezed,
        &mut TextSystem::without_system_fonts(),
    );
    assert_eq!(
        painted.len(),
        occ.len(),
        "controlo partido: nem todas as abas foram pintadas, então não é o caso espremido"
    );

    let dividers = slot_tabs::tab_dividers(&painted, front);
    // Com N abas há N−1 fronteiras; as duas que tocam a escolhida não levam linha.
    let touching = painted
        .windows(2)
        .filter(|p| Some(p[0].0.node) == front || Some(p[1].0.node) == front)
        .count();
    assert_eq!(
        dividers.len(),
        painted.len() - 1 - touching,
        "esperava uma divisória por fronteira que não toca a escolhida ({} fronteiras, {} a tocar)",
        painted.len() - 1,
        touching
    );
    assert!(
        !dividers.is_empty(),
        "controlo partido: nenhuma divisória para medir"
    );

    // Cada uma cai NA fronteira entre as duas abas, e não no meio de uma delas.
    for d in &dividers {
        let on_edge = painted
            .iter()
            .any(|(_, r)| ((r.x + r.w) - (d.x + d.w * 0.5)).abs() < 1.0);
        assert!(
            on_edge,
            "a divisória em x={:.1} não está na borda de aba nenhuma",
            d.x
        );
        assert!(
            d.h > 0.0 && d.h < bar.h,
            "a divisória tem de ser recuada: {d:?}"
        );
    }
}

/// ⭐⭐⭐ **UM PAINEL DE FERRAMENTA JUNTA-SE AO INSPECTOR COMO ABA — não o substitui.**
///
/// > *«algumas ferramentas ou painéis não criam abas»* — Enio, 2026-09-08.
///
/// ⛔ Treze painéis publicam o MESMO rect do dock direito e não colidiam por **convenção**: um
/// interruptor de flanco em cada bridge escondia o Inspector enquanto a ferramenta estivesse
/// activa. Com as abas a convenção deixou de ser precisa, e mantê-la custava duas coisas — a
/// fileira não nascia, e o Inspector **desaparecia sem a coluna se recolher**.
///
/// ⚠️ **Nenhuma linha nova decide quem fica à frente:** o [`slot_tabs::reconcile_z`] promove quem
/// ACABOU de ficar visível, e ao sair o `retain_panel_z` poda-o e o Inspector volta. *A lei já
/// existia; o takeover é que a contradizia.*
#[test]
fn a_tool_panel_joins_the_inspector_as_a_tab_instead_of_replacing_it() {
    // ⚠️ A Hierarquia entra na fixtura só para a ORDEM Z ter dois membros — ver a nota abaixo.
    let mut h = settled(&["inspector", "hierarchy"]);
    assert_eq!(
        row(&h, Slot::RightTop),
        vec!["inspector"],
        "controlo: a fixtura começa com o Inspector sozinho na coluna da direita"
    );

    // ⚠️⚠️ **O Inspector é posto à frente PRIMEIRO, e sem isto a asserção seguinte é VÁCUA.**
    //    Medido por mutação em 2026-09-08: com os dois a ler `z = 0` (um por ausência da ordem, o
    //    outro por estar na posição 0), o `max_by_key` desempata pela ordem do REGISTO — e apagar
    //    a promoção do `reconcile_z` deixava este teste VERDE. *Um empate não é uma medição.*
    //    ⚠️ E não basta promover o Inspector: com UM só membro na ordem a posição dele também é
    //    `0`, e o empate volta. São precisos DOIS — daí a Hierarquia na fixtura.
    h.store.bump_panel_z(node_of("hierarchy"));
    h.store.bump_panel_z(node_of("inspector"));
    paint(&mut h, 1);

    // A ferramenta entra — o painel dela fica visível, como o bridge faz.
    h.panel_visibility.insert("upscale", true);
    paint(&mut h, 3);
    let with_tool = row(&h, Slot::RightTop);
    assert!(
        with_tool.contains(&"inspector") && with_tool.contains(&"upscale"),
        "a ferramenta tinha de PARTILHAR a coluna com o Inspector, e a fila é {with_tool:?}"
    );
    assert_eq!(
        slot_tabs::chosen(&h, Slot::RightTop),
        Some(node_of("upscale")),
        "quem acabou de ficar visível tem de ficar à frente — senão a ferramenta abre escondida"
    );

    // E ao sair, o Inspector volta à frente sozinho.
    h.panel_visibility.insert("upscale", false);
    paint(&mut h, 3);
    assert_eq!(
        row(&h, Slot::RightTop),
        vec!["inspector"],
        "a ferramenta saiu e deixou alguém para trás"
    );
    assert_eq!(
        slot_tabs::chosen(&h, Slot::RightTop),
        Some(node_of("inspector")),
        "o Inspector não voltou à frente ao fechar a ferramenta"
    );
}
