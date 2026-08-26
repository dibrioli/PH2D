//! Gates for the command-palette search filter. Sibling of `command_palette.rs` (kept a CHILD module by
//! `#[path]` so it reaches the private `filter_model`, and to keep the widget file under the LOC cap).

use super::*;
use ph2d_tokens::ColorToken;

fn item(label: &'static str) -> PaletteItem {
    PaletteItem {
        label: label.into(),
        id: hash_node_id(label),
    }
}

/// Source has Grid + Boids; Forces has a "Physics" sub with Gravity + Wind — enough to exercise the
/// display order (Source before Forces) and the drop-empty-category rule.
fn model() -> PaletteModel {
    PaletteModel {
        title: "Add Node".into(),
        toggle: None,
        groups: vec![
            PaletteGroup {
                title: "Source".into(),
                color: ColorToken::NodeCatSource,
                subs: vec![PaletteSub {
                    title: None,
                    items: vec![item("Grid"), item("Boids")],
                }],
            },
            PaletteGroup {
                title: "Forces".into(),
                color: ColorToken::NodeCatSource,
                subs: vec![PaletteSub {
                    title: Some("Physics".into()),
                    items: vec![item("Gravity"), item("Wind")],
                }],
            },
        ],
    }
}

/// **The one predicate is a case-insensitive substring, and empty matches everything.** The filter and
/// the `Enter` top-match both call this, so what is shown and what is added cannot disagree.
#[test]
fn item_matches_is_case_insensitive_substring_and_empty_matches_all() {
    assert!(item_matches("", "anything"), "empty query matches all");
    assert!(item_matches("boi", "Boids"));
    assert!(item_matches("BOI", "Boids"), "case-insensitive");
    assert!(!item_matches("xyz", "Boids"));
}

/// **A filtered library drops sub-clusters and categories that end up empty** — no coloured header sits
/// over nothing. FALSIFIED if `filter_model` kept a group whose items all filtered out.
#[test]
fn filter_drops_emptied_subs_and_categories() {
    let f = filter_model(&model(), "wind");
    assert_eq!(f.groups.len(), 1, "only the category with a match survives");
    assert_eq!(f.item_count(), 1);
    assert_eq!(f.groups[0].title, "Forces");
}

/// **Enter adds the first FILTERED (visible) item, using the same predicate the filter paints with.** So
/// a click and an Enter can never disagree about what the palette shows vs adds. FALSIFIED if `top_match`
/// used a looser rule than `filter_model` (it would return an id the filter hid).
#[test]
fn enter_picks_the_first_filtered_match_and_it_is_visible() {
    let m = model();
    let q = "g"; // matches Grid (Source) + Gravity (Forces); display order puts Grid first.
    let filtered = filter_model(&m, q);
    let top = top_match(&m, q).expect("`g` matches something");
    assert!(
        filtered.is_item(top),
        "the Enter pick is one of the visible, filtered items"
    );
    assert_eq!(top, hash_node_id("Grid"), "first match in display order");
}

/// **Enter is a no-op with no search text or no match** — it never adds a node the artist did not narrow
/// to. FALSIFIED if `top_match` returned the first item for an empty query.
#[test]
fn enter_is_a_noop_on_empty_or_no_match() {
    assert_eq!(top_match(&model(), ""), None, "empty search adds nothing");
    assert_eq!(top_match(&model(), "zzz"), None, "no match adds nothing");
}

// ─────────────────────────────────────────────────────────────────────────────
// A CASCATA de entrada (F5)
// ─────────────────────────────────────────────────────────────────────────────

/// Pinta a paleta e devolve o retângulo REGISTADO do item `label`, com a cascata em `t`.
fn hit_rect_with_cascade(t: f32) -> crate::zones::Rect {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hit = crate::interaction::HitIndex::new();
    let mut motion = crate::motion::UiMotion::default();
    // A primeira vista CHEGA ao alvo ⇒ isto crava o track em `t` exactamente.
    for i in 0..8 {
        motion.animate(cascade_id(i), t, crate::motion::Role::Travel);
    }
    paint(
        &mut scene,
        &mut ts,
        ph2d_tokens::Theme::Forge,
        &mut hit,
        &model(),
        "",
        crate::zones::Rect::new(0.0, 0.0, 1600.0, 900.0),
        &motion,
        0.0,
    );
    hit.rect_for(hash_node_id("Grid"))
        .expect("o pill Grid regista")
}

/// ⭐ **O DESENHO anda, o ALVO não — a lei da wave.**
///
/// Durante a entrada o cartão está 12 px abaixo do sítio onde vai assentar. Se o `hit_index`
/// registasse aí, um clique apressado cairia na row de cima — e um alvo que foge debaixo do dedo é
/// pior que uma entrada que não existe. É a mesma lei que o `hover_lift` já carrega.
///
/// *Mutação: registar em `dy` em vez de `oy` ⇒ sangra (o retângulo desce 12 px).*
#[test]
fn the_entrance_moves_the_drawing_and_never_the_target() {
    let flying = hit_rect_with_cascade(0.0);
    let settled = hit_rect_with_cascade(1.0);
    assert!(
        (flying.y - settled.y).abs() < 0.001,
        "o alvo FUGIU durante a entrada: y {} em voo contra {} assente",
        flying.y,
        settled.y
    );
    assert!((flying.x - settled.x).abs() < 0.001, "nem de lado");
}

/// ⚠️ E ela **DESENHA** mesmo diferente — senão o gate acima é verde por vácuo, sobre uma cascata
/// que não existe. O oráculo é `encoding().n_paths`-vizinho: os dados de caminho REAIS.
#[test]
fn the_entrance_actually_moves_the_drawing() {
    fn drawn(t: f32) -> Vec<u32> {
        let mut scene = ph2d_vector::VectorScene::new();
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let mut hit = crate::interaction::HitIndex::new();
        let mut motion = crate::motion::UiMotion::default();
        for i in 0..8 {
            motion.animate(cascade_id(i), t, crate::motion::Role::Travel);
        }
        paint(
            &mut scene,
            &mut ts,
            ph2d_tokens::Theme::Forge,
            &mut hit,
            &model(),
            "",
            crate::zones::Rect::new(0.0, 0.0, 1600.0, 900.0),
            &motion,
            0.0,
        );
        scene.inner().encoding().path_data.clone()
    }
    assert_ne!(
        drawn(0.0),
        drawn(1.0),
        "a cascata nao desenha nada de diferente — a wave inteira e um no-op"
    );
}

/// ⭐ **A caixa da banda REGISTA o hit** (ADR-0166 / F3), e só quando o modelo tem uma.
///
/// ⚠️ As duas metades: um controlo pintado e **não registado** fica morto sob o dedo — e um report
/// disso lê-se exactamente igual a *"o botão nunca apareceu"*. A metade de ausência garante que os
/// outros dois consumidores da paleta (a biblioteca de nós, o `Ctrl+K`) não ganham um id órfão no
/// índice, que continuaria clicável.
#[test]
fn the_band_toggle_registers_its_hit_only_when_the_model_has_one() {
    use crate::interaction::HitIndex;
    use ph2d_text::TextSystem;
    use ph2d_vector::VectorScene;

    fn painted(m: &PaletteModel) -> bool {
        let mut scene = VectorScene::new();
        let mut ts = TextSystem::new();
        let mut hits = HitIndex::new();
        let motion = crate::motion::UiMotion::default();
        paint(
            &mut scene,
            &mut ts,
            ph2d_tokens::Theme::default(),
            &mut hits,
            m,
            "",
            Rect::new(0.0, 0.0, 1600.0, 900.0),
            &motion,
            0.0,
        );
        hits.rect_for(CMD_PALETTE_SHOW_ALL).is_some()
    }

    assert!(!painted(&model()), "sem caixa no modelo, sem id no indice");
    let mut with = model();
    with.toggle = Some(PaletteToggle {
        label: "Show all".into(),
        on: false,
    });
    assert!(
        painted(&with),
        "a caixa foi pintada e nao registada — morta sob o dedo"
    );
    // ⚠️ E LIGADA continua clicável: um controlo que só aceita o clique no estado em que já está
    // seria uma porta de sentido único.
    let mut on = model();
    on.toggle = Some(PaletteToggle {
        label: "Show all".into(),
        on: true,
    });
    assert!(painted(&on), "ligada, ela tem de continuar clicavel");
}

/// ⚠️ **A caixa ATRAVESSA o filtro de busca.** Ela é da BANDA, não do conteúdo — escrever no campo
/// de busca não pode apagar o controlo que mostra mais resultados.
#[test]
fn the_band_toggle_survives_the_search_filter() {
    let mut m = model();
    m.toggle = Some(PaletteToggle {
        label: "Show all".into(),
        on: true,
    });
    let f = filter_model(&m, "wind");
    assert_eq!(
        f.toggle, m.toggle,
        "o filtro comeu a caixa da banda: escrever na busca apagaria o controlo"
    );
}

// ── A ROLAGEM e a ARRUMAÇÃO (F3 / ADR-0166, report do Enio de 25/08) ──────────────────────────

use crate::interaction::HitIndex;
use crate::zones::Rect as ZRect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Um modelo com `n` categorias de `k` itens — alto o suficiente para transbordar.
fn tall(n: usize, k: usize) -> PaletteModel {
    PaletteModel {
        title: "Add Component".into(),
        toggle: None,
        groups: (0..n)
            .map(|i| PaletteGroup {
                title: format!("Cat {i}"),
                color: ColorToken::NodeCatSource,
                subs: vec![PaletteSub {
                    title: None,
                    items: (0..k)
                        .map(|j| item_owned(format!("Item {i}-{j}")))
                        .collect(),
                }],
            })
            .collect(),
    }
}

fn item_owned(label: String) -> PaletteItem {
    let id = hash_node_id(Box::leak(label.clone().into_boxed_str()));
    PaletteItem { label, id }
}

const VP: ZRect = ZRect {
    x: 0.0,
    y: 0.0,
    w: 1187.0,
    h: 953.0,
};

/// Pinta com `scroll` e devolve os rects registados.
fn painted(m: &PaletteModel, scroll: f32) -> Vec<(ph2d_a11y::NodeId, ZRect)> {
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::new();
    let mut hits = HitIndex::new();
    let motion = crate::motion::UiMotion::default();
    paint(
        &mut scene,
        &mut ts,
        ph2d_tokens::Theme::default(),
        &mut hits,
        m,
        "",
        VP,
        &motion,
        scroll,
    );
    hits.iter_registrations().collect()
}

/// ⭐ **NADA se regista fora do cartão** (F3) — o report do Enio: *«veja que componentes estão
/// inacessíveis fora da janela»*.
///
/// ⚠️ **É o `HitIndex::push_clip` que o garante**, e a cena tem o par dela (`push_clip` do
/// `VectorScene`): são DUAS coisas — um decide quem RESPONDE, o outro recorta PIXELS. Antes desta
/// wave o conteúdo desenhava e registava para fora do cartão, e o que saía do ecrã era
/// inalcançável por gesto nenhum.
///
/// (Mutação: tirar o `push_clip` do hit-index ⇒ os itens de baixo voltam a registar fora do cartão.)
#[test]
fn nothing_registers_outside_the_card() {
    let m = tall(24, 15);
    let rects = painted(&m, 0.0);
    let card = rects
        .iter()
        .find(|(n, _)| *n == CMD_PALETTE_CARD)
        .map(|(_, r)| *r)
        .expect("o cartao regista-se");
    let ids: std::collections::BTreeSet<_> = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|i| i.id)
        .collect();
    let escaped: Vec<_> = rects
        .iter()
        .filter(|(n, _)| ids.contains(n))
        .filter(|(_, r)| r.y + r.h > card.y + card.h + 0.5 || r.y < card.y - 0.5)
        .collect();
    assert!(
        escaped.is_empty(),
        "{} items registaram-se FORA do cartao (o cartao vai de {:.0} a {:.0}); \
         o primeiro esta' em y={:.0}",
        escaped.len(),
        card.y,
        card.y + card.h,
        escaped[0].1.y
    );
}

/// ⭐ **E o que não cabe é ALCANÇÁVEL rolando** — a outra metade, e a que o report pedia.
///
/// Sem ela o gate acima ficaria verde sobre uma paleta que simplesmente **esconde** o excedente,
/// que é o defeito com outra cara: *o que existe e não se alcança lê-se como ausência*.
#[test]
fn everything_is_reachable_by_scrolling() {
    // ⚠️ **A fixture cresceu de `tall(10, 12)` para cá**, e o motivo é a lição: o masonry fez a
    // antiga CABER, e uma fixture sem o fenómeno teria deixado este gate verde sobre nada.
    let m = tall(24, 15);
    let mut ts = TextSystem::new();
    let max = max_scroll(&mut ts, &m, "", VP);
    assert!(
        max > 0.0,
        "a fixture TEM de transbordar, senao nao mede nada"
    );

    let mut seen = std::collections::BTreeSet::new();
    // Duas paradas bastam para esta altura; o passo é meia janela, como uma roda faz.
    for k in 0..=4u8 {
        let s = max * f32::from(k) / 4.0;
        for (id, _) in painted(&m, s) {
            seen.insert(id);
        }
    }
    let missing: Vec<_> = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .filter(|i| !seen.contains(&i.id))
        .map(|i| i.label.clone())
        .collect();
    assert!(
        missing.is_empty(),
        "{} items continuam inalcancaveis mesmo rolando ate' ao fim: {:?}",
        missing.len(),
        &missing[..missing.len().min(5)]
    );
}

/// ⚠️ **A metade de AUSÊNCIA: quando tudo cabe, não há rolagem nenhuma.**
///
/// Um `max_scroll` positivo sobre conteúdo que cabe daria um traço indicador sobre nada e uma roda
/// que move um cartão inteiro sem motivo.
#[test]
fn a_short_palette_does_not_scroll() {
    let mut ts = TextSystem::new();
    assert_eq!(max_scroll(&mut ts, &model(), "", VP), 0.0);
}

/// ⭐ **As colunas são EQUILIBRADAS, não posicionais** (F3).
///
/// ⚠️ **A lei anterior era «as pequenas à esquerda, a grande à direita»** e não olhava para altura
/// nenhuma: na foto do Enio uma coluna transbordava o ecrã enquanto o canto direito ficava vazio
/// por baixo de uma categoria baixa e larga. Medido, o masonry corta o excedente de **345 px para
/// 217** no caso dele.
///
/// A lei aqui é a forma do defeito: **nenhum vão fica VAZIO enquanto outro empilha dois ou mais**.
#[test]
fn no_slot_sits_empty_while_another_stacks() {
    let mut ts = TextSystem::new();
    let m = tall(8, 6);
    let (placed, _) = arrange(&mut ts, &m, 0.0, 1124.0);
    // Os cartões estreitos (largura de uma unidade) contam por vão; os largos ocupam dois e não
    // deixam vão vazio por construção.
    let unit = placed
        .iter()
        .map(|(_, _, _, w)| *w)
        .fold(f32::INFINITY, f32::min);
    let mut per_slot = std::collections::BTreeMap::<i32, usize>::new();
    for (_, ox, _, w) in &placed {
        #[allow(clippy::cast_possible_truncation)]
        let slot = (ox / (unit + COL_GAP)).round() as i32;
        let span = (w / unit).round().max(1.0) as i32;
        for c in slot..slot + span {
            *per_slot.entry(c).or_default() += 1;
        }
    }
    let used = per_slot.len();
    assert_eq!(
        used, 4,
        "o arranjo tem de usar as QUATRO unidades: {per_slot:?}"
    );
    let min = per_slot.values().copied().min().unwrap_or(0);
    let max = per_slot.values().copied().max().unwrap_or(0);
    assert!(
        min > 0 && max - min <= 1,
        "vaos desequilibrados {per_slot:?} — um vao vazio ao lado de uma pilha e' a foto do report"
    );
}
